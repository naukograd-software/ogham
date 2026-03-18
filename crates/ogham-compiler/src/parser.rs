//! Hand-written recursive descent parser for Ogham.
//!
//! Produces a lossless CST via [`rowan::GreenNodeBuilder`].
//! The parser **never panics** — it always produces a tree, wrapping
//! unexpected tokens in [`SyntaxKind::Error`] nodes.

use rowan::GreenNode;

use crate::lexer;
use crate::syntax_kind::SyntaxKind::{self, *};

// ── Public API ─────────────────────────────────────────────────────────

/// Result of parsing an Ogham source file.
pub struct Parse {
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
}

impl Parse {
    /// Build a rowan [`SyntaxNode`] from the parse result.
    pub fn syntax(&self) -> crate::syntax_kind::SyntaxNode {
        crate::syntax_kind::SyntaxNode::new_root(self.green.clone())
    }
}

/// A parse error with its byte range in source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub range: std::ops::Range<usize>,
}

/// Parse an Ogham source file into a lossless CST.
pub fn parse(source: &str) -> Parse {
    let raw = lexer::lex(source);
    let mut tokens = Vec::with_capacity(raw.len());
    let mut errors = Vec::new();

    for result in raw {
        match result {
            Ok(t) => tokens.push((SyntaxKind::from(t.kind), &source[t.span])),
            Err(span) => {
                errors.push(ParseError {
                    message: "unexpected character".into(),
                    range: span.clone(),
                });
                tokens.push((LexError, &source[span]));
            }
        }
    }

    let mut p = Parser {
        tokens,
        pos: 0,
        builder: rowan::GreenNodeBuilder::new(),
        errors,
        source,
    };
    p.parse_root();

    Parse {
        green: p.builder.finish(),
        errors: p.errors,
    }
}

// ── Parser internals ───────────────────────────────────────────────────

struct Parser<'s> {
    tokens: Vec<(SyntaxKind, &'s str)>,
    pos: usize,
    builder: rowan::GreenNodeBuilder<'static>,
    errors: Vec<ParseError>,
    source: &'s str,
}

impl<'s> Parser<'s> {
    // ── Helpers ────────────────────────────────────────────────────────

    /// Current token kind, skipping nothing.
    fn current(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.pos).map(|(k, _)| *k)
    }

    /// Current token kind, skipping trivia.
    fn current_non_trivia(&self) -> Option<SyntaxKind> {
        let mut i = self.pos;
        while let Some((k, _)) = self.tokens.get(i) {
            if !is_trivia(*k) {
                return Some(*k);
            }
            i += 1;
        }
        None
    }

    /// Peek at the nth non-trivia token ahead (0 = current non-trivia).
    fn peek_non_trivia(&self, n: usize) -> Option<SyntaxKind> {
        let mut count = 0;
        let mut i = self.pos;
        while let Some((k, _)) = self.tokens.get(i) {
            if !is_trivia(*k) {
                if count == n {
                    return Some(*k);
                }
                count += 1;
            }
            i += 1;
        }
        None
    }

    /// Consume and emit the current token.
    fn bump(&mut self) {
        if let Some((kind, text)) = self.tokens.get(self.pos) {
            self.builder.token((*kind).into(), text);
            self.pos += 1;
        }
    }

    /// Consume and emit all trivia at the current position.
    fn eat_trivia(&mut self) {
        while let Some(k) = self.current() {
            if is_trivia(k) {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Expect a specific token kind, emit error if not found.
    fn expect(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        if self.current() == Some(kind) {
            self.bump();
        } else {
            let offset = self.current_offset();
            self.errors.push(ParseError {
                message: format!("expected {:?}", kind),
                range: offset..offset,
            });
        }
    }

    /// If current non-trivia token matches `kind`, eat trivia + bump and return true.
    fn eat(&mut self, kind: SyntaxKind) -> bool {
        self.eat_trivia();
        if self.current() == Some(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn current_offset(&self) -> usize {
        // Compute byte offset from token slices
        if let Some((_, text)) = self.tokens.get(self.pos) {
            let ptr = text.as_ptr() as usize;
            let base = self.source.as_ptr() as usize;
            ptr - base
        } else {
            self.source.len()
        }
    }

    /// Save current position for progress checking.
    fn save_pos(&self) -> usize {
        self.pos
    }

    /// If no tokens were consumed since `saved`, force-consume one in an Error node.
    /// Returns `true` if the parser was stuck (and we force-advanced).
    fn check_progress(&mut self, saved: usize) -> bool {
        if self.pos == saved && self.current().is_some() {
            self.error_recover("unexpected token");
            true
        } else {
            false
        }
    }

    /// Wrap the current token in an Error node and advance.
    fn error_recover(&mut self, msg: &str) {
        let offset = self.current_offset();
        let end = if let Some((_, text)) = self.tokens.get(self.pos) {
            offset + text.len()
        } else {
            offset
        };
        self.errors.push(ParseError {
            message: msg.into(),
            range: offset..end,
        });
        self.builder.start_node(Error.into());
        self.bump();
        self.builder.finish_node();
    }


    // ── Root ───────────────────────────────────────────────────────────

    fn parse_root(&mut self) {
        self.builder.start_node(Root.into());

        self.eat_trivia();
        if self.current_non_trivia() == Some(KwPackage) {
            self.parse_package_decl();
        }

        loop {
            self.eat_trivia();
            if self.current().is_none() {
                break;
            }
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(KwImport) => self.parse_import_decl(),
                Some(KwType) | Some(At) | Some(KwAnnotation) | Some(KwShape)
                | Some(KwEnum) | Some(KwService) => self.parse_top_level_decl(),
                None => break,
                _ => self.error_recover("expected top-level declaration"),
            }
            self.check_progress(saved);
        }

        self.eat_trivia();
        self.builder.finish_node();
    }

    // ── Package ────────────────────────────────────────────────────────

    fn parse_package_decl(&mut self) {
        self.builder.start_node(PackageDecl.into());
        self.eat_trivia();
        self.bump(); // 'package'
        self.eat_trivia();
        self.expect_ident();
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    // ── Import ─────────────────────────────────────────────────────────

    fn parse_import_decl(&mut self) {
        self.builder.start_node(ImportDecl.into());
        self.eat_trivia();
        self.bump(); // 'import'
        self.eat_trivia();
        self.parse_import_path();
        self.eat_trivia();
        if self.current() == Some(KwAs) {
            self.bump(); // 'as'
            self.eat_trivia();
            self.expect_ident();
        }
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_import_path(&mut self) {
        self.builder.start_node(ImportPath.into());
        self.expect_ident();
        while let Some(k) = self.current() {
            if k == Dot || k == Slash || k == Minus {
                self.bump();
                self.expect_ident_or_keyword();
            } else {
                break;
            }
        }
        self.builder.finish_node();
    }

    // ── Top-level declarations ─────────────────────────────────────────

    fn parse_top_level_decl(&mut self) {
        self.eat_trivia();

        // Collect leading annotations
        let mut has_annotations = false;
        while self.current_non_trivia() == Some(At) {
            self.parse_annotation_call();
            has_annotations = true;
        }

        self.eat_trivia();
        match self.current() {
            Some(KwType) => self.parse_type_decl(),
            Some(KwShape) => self.parse_shape_decl(),
            Some(KwEnum) => self.parse_enum_decl(),
            Some(KwService) => self.parse_service_decl(),
            Some(KwAnnotation) => self.parse_annotation_decl(),
            _ => {
                if has_annotations {
                    self.error_recover("expected declaration after annotation");
                }
            }
        }
    }

    // ── Type ───────────────────────────────────────────────────────────

    fn parse_type_decl(&mut self) {
        self.builder.start_node(TypeDecl.into());
        self.eat_trivia();
        self.bump(); // 'type'
        self.eat_trivia();
        self.expect_ident(); // type name

        self.eat_trivia();
        match self.current() {
            Some(LAngle) => {
                self.parse_type_params();
                self.eat_trivia();
                self.parse_type_body();
            }
            Some(Eq) => self.parse_type_alias(),
            Some(LBrace) => self.parse_type_body(),
            _ => {
                let offset = self.current_offset();
                self.errors.push(ParseError {
                    message: "expected '=', '<', or '{'".into(),
                    range: offset..offset,
                });
            }
        }

        self.builder.finish_node();
    }

    fn parse_type_alias(&mut self) {
        self.builder.start_node(TypeAlias.into());
        self.bump(); // '='
        self.eat_trivia();
        match self.current_non_trivia() {
            Some(KwPick) => self.parse_pick_type(),
            Some(KwOmit) => self.parse_omit_type(),
            _ => self.parse_type_ref(),
        }
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_type_body(&mut self) {
        self.builder.start_node(TypeBody.into());
        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(At) => {
                    self.parse_type_member();
                }
                Some(KwOneof) => self.parse_oneof_decl(),
                Some(KwType) => self.parse_nested_type_decl(),
                Some(KwEnum) => self.parse_nested_enum_decl(),
                Some(KwReserved) => self.parse_reserved_decl(),
                _ => {
                    self.parse_field_or_shape_injection();
                }
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_type_member(&mut self) {
        self.eat_trivia();

        // Collect annotations
        while self.current_non_trivia() == Some(At) {
            self.parse_annotation_call();
        }

        self.eat_trivia();
        match self.current() {
            Some(KwOneof) => self.parse_oneof_decl(),
            Some(KwType) => self.parse_nested_type_decl(),
            Some(KwEnum) => self.parse_nested_enum_decl(),
            Some(KwReserved) => self.parse_reserved_decl(),
            _ => self.parse_field_or_shape_injection(),
        }
    }

    /// Parse `reserved 2, 3, 5;`
    fn parse_reserved_decl(&mut self) {
        self.builder.start_node(ReservedDecl.into());
        self.eat_trivia();
        self.bump(); // 'reserved'
        self.eat_trivia();
        self.expect_int(); // first number
        while self.eat(Comma) {
            self.eat_trivia();
            self.expect_int();
        }
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_field_or_shape_injection(&mut self) {
        // Lookahead: if after an identifier-like sequence we see `(` then `int..int)`,
        // it's a shape injection. Otherwise it's a field.
        // shape_injection: Ident(N..M)
        // field: type_ref ident = N [<- ...];
        self.eat_trivia();
        if self.is_shape_injection() {
            self.parse_shape_injection();
        } else {
            self.parse_field_decl();
        }
    }

    fn is_shape_injection(&self) -> bool {
        // Shape injection patterns:
        //   Ident(N..M)                       — simple: MyShape(1..4)
        //   Ident.Ident(N..M)                 — qualified: rpc.PageRequest(1..2)
        //   Ident.Ident<TypeArg>(N..M)        — generic qualified: common.Paginated<Order>(1..1)
        // We scan forward through Ident.Ident... skipping <...> type args,
        // until we find `(` followed by IntLiteral and `..`
        let mut i = 0;
        loop {
            let t = self.peek_non_trivia(i);
            if t != Some(Ident) && !t.map(is_keyword).unwrap_or(false) {
                return false;
            }
            i += 1;
            let next = self.peek_non_trivia(i);
            if next == Some(Dot) {
                i += 1; // skip dot, continue to next ident
                continue;
            }
            // Skip type arguments: <...>
            if next == Some(LAngle) {
                i += 1;
                let mut depth = 1;
                while depth > 0 {
                    match self.peek_non_trivia(i) {
                        Some(LAngle) => { depth += 1; i += 1; }
                        Some(RAngle) => { depth -= 1; i += 1; }
                        None => return false,
                        _ => { i += 1; }
                    }
                }
                // After closing >, check for (N..M)
                if self.peek_non_trivia(i) == Some(LParen) {
                    return self.peek_non_trivia(i + 1) == Some(IntLiteral)
                        && self.peek_non_trivia(i + 2) == Some(DotDot);
                }
                return false;
            }
            if next == Some(LParen) {
                // Check: ( IntLiteral .. IntLiteral )
                return self.peek_non_trivia(i + 1) == Some(IntLiteral)
                    && self.peek_non_trivia(i + 2) == Some(DotDot);
            }
            return false;
        }
    }

    fn parse_shape_injection(&mut self) {
        self.builder.start_node(ShapeInjection.into());
        self.eat_trivia();
        // Parse qualified shape name: Ident or Ident.Ident.Ident...
        self.parse_qualified_name();
        // Optional type arguments: <Type, Type, ...>
        if self.current() == Some(LAngle) {
            self.parse_type_args();
        }
        self.expect(LParen);
        self.eat_trivia();
        self.expect_int(); // start
        self.expect(DotDot);
        self.eat_trivia();
        self.expect_int(); // end
        self.expect(RParen);
        self.builder.finish_node();
    }

    fn parse_field_decl(&mut self) {
        self.builder.start_node(FieldDecl.into());
        self.eat_trivia();
        self.parse_type_ref();
        self.eat_trivia();
        self.expect_ident(); // field name
        self.expect(Eq);
        self.eat_trivia();
        self.expect_int(); // field number

        // Optional projection mapping
        self.eat_trivia();
        if self.current() == Some(BackArrow) {
            self.parse_mapping_source();
        }

        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_mapping_source(&mut self) {
        self.builder.start_node(MappingSource.into());
        self.bump(); // '<-'
        self.eat_trivia();
        self.parse_qualified_name_or_wildcard();
        self.builder.finish_node();
    }

    fn parse_qualified_name_or_wildcard(&mut self) {
        // SourceType.field or SourceType.oneof.*.field
        self.expect_ident();
        while self.current() == Some(Dot) {
            self.bump(); // '.'
            if self.current() == Some(Star) {
                self.bump(); // '*'
            } else {
                self.expect_ident();
            }
        }
    }

    // ── Oneof ──────────────────────────────────────────────────────────

    fn parse_oneof_decl(&mut self) {
        self.builder.start_node(OneofDecl.into());
        self.eat_trivia();
        self.bump(); // 'oneof'
        self.eat_trivia();
        self.expect_ident(); // oneof name
        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(At) => {
                    while self.current_non_trivia() == Some(At) {
                        self.parse_annotation_call();
                    }
                    self.parse_oneof_field();
                }
                _ => self.parse_oneof_field(),
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_oneof_field(&mut self) {
        self.builder.start_node(OneofField.into());
        self.eat_trivia();
        self.parse_type_ref();
        self.eat_trivia();
        self.expect_ident(); // field name
        self.expect(Eq);
        self.eat_trivia();
        self.expect_int(); // field number

        // Optional projection mapping
        self.eat_trivia();
        if self.current() == Some(BackArrow) {
            self.parse_mapping_source();
        }

        self.expect(Semicolon);
        self.builder.finish_node();
    }

    // ── Nested type/enum ───────────────────────────────────────────────

    fn parse_nested_type_decl(&mut self) {
        self.builder.start_node(NestedTypeDecl.into());
        self.parse_type_decl();
        self.builder.finish_node();
    }

    fn parse_nested_enum_decl(&mut self) {
        self.builder.start_node(NestedEnumDecl.into());
        self.parse_enum_decl();
        self.builder.finish_node();
    }

    // ── Shape ──────────────────────────────────────────────────────────

    fn parse_shape_decl(&mut self) {
        self.builder.start_node(ShapeDecl.into());
        self.eat_trivia();
        self.bump(); // 'shape'
        self.eat_trivia();
        self.expect_ident(); // shape name

        self.eat_trivia();
        if self.current() == Some(LAngle) {
            self.parse_type_params();
        }

        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(At) => {
                    while self.current_non_trivia() == Some(At) {
                        self.parse_annotation_call();
                    }
                    self.eat_trivia();
                    self.parse_shape_field();
                }
                _ => {
                    self.parse_shape_member();
                }
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_shape_member(&mut self) {
        self.eat_trivia();
        if self.is_shape_include() {
            self.parse_shape_include();
        } else {
            self.parse_shape_field();
        }
    }

    fn is_shape_include(&self) -> bool {
        // shape_include: Ident [, Ident]* ;
        // shape_field: type_ref ident ;
        // If Ident followed by `;` or `,` -> include.
        let t0 = self.peek_non_trivia(0);
        let t1 = self.peek_non_trivia(1);
        t0 == Some(Ident)
            && (t1 == Some(Semicolon) || t1 == Some(Comma))
    }

    fn parse_shape_include(&mut self) {
        self.builder.start_node(ShapeInclude.into());
        self.eat_trivia();
        self.bump(); // first ident
        while self.eat(Comma) {
            self.eat_trivia();
            self.expect_ident();
        }
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_shape_field(&mut self) {
        self.builder.start_node(ShapeField.into());
        self.eat_trivia();
        self.parse_type_ref();
        self.eat_trivia();
        self.expect_ident(); // field name

        // Trailing annotations on shape fields (e.g. `@default(now)`)
        self.eat_trivia();
        while self.current_non_trivia() == Some(At) {
            self.parse_annotation_call();
            self.eat_trivia();
        }

        self.expect(Semicolon);
        self.builder.finish_node();
    }

    // ── Enum ───────────────────────────────────────────────────────────

    fn parse_enum_decl(&mut self) {
        self.builder.start_node(EnumDecl.into());
        self.eat_trivia();
        self.bump(); // 'enum'
        self.eat_trivia();
        self.expect_ident(); // enum name
        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(At) => {
                    while self.current_non_trivia() == Some(At) {
                        self.parse_annotation_call();
                    }
                    self.eat_trivia();
                    self.parse_enum_value_decl();
                }
                _ => self.parse_enum_value_decl(),
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_enum_value_decl(&mut self) {
        self.builder.start_node(EnumValueDecl.into());
        self.eat_trivia();
        self.expect_ident(); // value name
        self.expect(Eq);
        self.eat_trivia();
        self.expect_int();
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    // ── Service & RPC ──────────────────────────────────────────────────

    fn parse_service_decl(&mut self) {
        self.builder.start_node(ServiceDecl.into());
        self.eat_trivia();
        self.bump(); // 'service'
        self.eat_trivia();
        self.expect_ident(); // service name
        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(At) => {
                    while self.current_non_trivia() == Some(At) {
                        self.parse_annotation_call();
                    }
                    self.eat_trivia();
                    if self.current_non_trivia() == Some(KwRpc) {
                        self.parse_rpc_decl();
                    }
                }
                Some(KwRpc) => self.parse_rpc_decl(),
                _ => self.error_recover("expected 'rpc' declaration"),
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_rpc_decl(&mut self) {
        self.builder.start_node(RpcDecl.into());
        self.eat_trivia();
        self.bump(); // 'rpc'
        self.eat_trivia();
        self.expect_ident(); // rpc name
        self.expect(LParen);
        self.parse_rpc_param();
        self.expect(RParen);
        self.expect(Arrow);
        self.parse_rpc_param();
        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_rpc_param(&mut self) {
        self.builder.start_node(RpcParam.into());
        self.eat_trivia();
        match self.current() {
            Some(KwVoid) => self.bump(),
            Some(KwStream) => {
                self.bump(); // 'stream'
                self.eat_trivia();
                if self.current() == Some(LBrace) {
                    self.parse_inline_type();
                } else {
                    self.parse_type_ref();
                }
            }
            Some(LBrace) => self.parse_inline_type(),
            _ => self.parse_type_ref(),
        }
        self.builder.finish_node();
    }

    fn parse_inline_type(&mut self) {
        self.builder.start_node(InlineType.into());
        self.expect(LBrace);
        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                Some(KwOneof) => self.parse_oneof_decl(),
                Some(At) => {
                    while self.current_non_trivia() == Some(At) {
                        self.parse_annotation_call();
                    }
                    match self.current_non_trivia() {
                        Some(KwOneof) => self.parse_oneof_decl(),
                        _ => self.parse_field_or_shape_injection(),
                    }
                }
                _ => self.parse_field_or_shape_injection(),
            }
            self.check_progress(saved);
        }
        self.expect(RBrace);
        self.builder.finish_node();
    }

    // ── Annotation definition ──────────────────────────────────────────

    fn parse_annotation_decl(&mut self) {
        self.builder.start_node(AnnotationDecl.into());
        self.eat_trivia();
        self.bump(); // 'annotation'
        self.eat_trivia();
        self.expect_ident(); // annotation name
        self.eat_trivia();
        self.expect(KwFor);
        self.parse_annotation_targets();
        self.expect(LBrace);

        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                _ => self.parse_annotation_member(),
            }
            self.check_progress(saved);
        }

        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_annotation_targets(&mut self) {
        self.builder.start_node(AnnotationTargets.into());
        self.eat_trivia();
        self.expect_ident_or_keyword(); // target (type, field, etc.)
        // Optional type constraint: field(string | int32)
        if self.current() == Some(LParen) {
            self.parse_annotation_target_constraint();
        }
        while self.eat(Pipe) {
            self.eat_trivia();
            self.expect_ident_or_keyword();
            if self.current() == Some(LParen) {
                self.parse_annotation_target_constraint();
            }
        }
        self.builder.finish_node();
    }

    /// Parse `(string | int32 | []any | map<K,V> | message | time.Timestamp)`
    fn parse_annotation_target_constraint(&mut self) {
        self.builder.start_node(AnnotationTargetConstraint.into());
        self.bump(); // '('
        self.eat_trivia();
        self.parse_type_constraint_atom();
        self.eat_trivia();
        while self.current_non_trivia() == Some(Pipe) {
            self.bump(); // '|'
            self.eat_trivia();
            self.parse_type_constraint_atom();
            self.eat_trivia();
        }
        self.expect(RParen);
        self.builder.finish_node();
    }

    /// Parse a single type constraint atom: `string`, `[]int32`, `map<K,V>`,
    /// `message`, `enum`, `any`, `time.Timestamp`
    fn parse_type_constraint_atom(&mut self) {
        self.eat_trivia();
        match self.current_non_trivia() {
            // []T — array constraint ([] is a single Brackets token)
            Some(Brackets) => {
                self.bump(); // '[]'
                self.eat_trivia();
                self.parse_type_constraint_atom(); // element type
            }
            // map<K, V>
            Some(KwMap) => {
                self.bump(); // 'map'
                self.expect(LAngle);
                self.eat_trivia();
                self.parse_type_constraint_atom(); // key
                self.expect(Comma);
                self.eat_trivia();
                self.parse_type_constraint_atom(); // value
                self.expect(RAngle);
            }
            // ident — scalar name, `any`, `message`, `enum`, or qualified name
            _ => {
                self.expect_ident_or_keyword();
                // Check for qualified name: time.Timestamp
                while self.current() == Some(Dot) {
                    self.bump(); // '.'
                    self.expect_ident_or_keyword();
                }
            }
        }
    }

    fn parse_annotation_member(&mut self) {
        self.eat_trivia();
        // Could be:
        // 1. type_ref [?] ident [= default] ;           (annotation field)
        // 2. ident : { ... }                             (inline annotation type)
        // 3. qualified::Name(...) ;                      (annotation composition)

        // Check for composition: Ident :: Ident
        if self.is_annotation_composition() {
            self.parse_annotation_composition();
            return;
        }

        // Check for inline type: ident `:` `{`
        if self.is_inline_annotation_type() {
            self.parse_inline_annotation_type_member();
            return;
        }

        // Otherwise it's a field
        self.parse_annotation_field();
    }

    fn is_annotation_composition(&self) -> bool {
        // qualified::Name
        let t0 = self.peek_non_trivia(0);
        let t1 = self.peek_non_trivia(1);
        t0 == Some(Ident) && t1 == Some(ColonColon)
    }

    fn is_inline_annotation_type(&self) -> bool {
        let t0 = self.peek_non_trivia(0);
        let t1 = self.peek_non_trivia(1);
        t0 == Some(Ident) && t1 == Some(Colon)
    }

    fn parse_annotation_composition(&mut self) {
        self.builder.start_node(AnnotationComposition.into());
        self.eat_trivia();
        self.bump(); // qualifier ident
        self.eat_trivia();
        self.bump(); // '::'
        self.eat_trivia();
        self.expect_ident(); // annotation name

        self.eat_trivia();
        if self.current() == Some(LParen) {
            self.bump(); // '('
            self.eat_trivia();
            if self.current_non_trivia() != Some(RParen) {
                self.parse_annotation_args_inner();
            }
            self.expect(RParen);
        }

        self.expect(Semicolon);
        self.builder.finish_node();
    }

    fn parse_inline_annotation_type_member(&mut self) {
        self.builder.start_node(AnnotationField.into());
        self.eat_trivia();
        self.bump(); // name ident
        self.eat_trivia();
        self.bump(); // ':'
        self.parse_inline_annotation_type_body();
        self.builder.finish_node();
    }

    fn parse_inline_annotation_type_body(&mut self) {
        self.builder.start_node(InlineAnnotationType.into());
        self.expect(LBrace);
        loop {
            self.eat_trivia();
            let saved = self.save_pos();
            match self.current_non_trivia() {
                Some(RBrace) | None => break,
                _ => self.parse_annotation_member(),
            }
            self.check_progress(saved);
        }
        self.expect(RBrace);
        self.builder.finish_node();
    }

    fn parse_annotation_field(&mut self) {
        self.builder.start_node(AnnotationField.into());
        self.eat_trivia();
        self.parse_type_ref();
        self.eat_trivia();

        // Optional `?`
        if self.current() == Some(Question) {
            self.bump();
            self.eat_trivia();
        }

        self.expect_ident(); // field name

        // Optional default value
        self.eat_trivia();
        if self.current() == Some(Eq) {
            self.bump(); // '='
            self.eat_trivia();
            self.parse_literal_or_annotation_value();
        }

        self.expect(Semicolon);
        self.builder.finish_node();
    }

    // ── Annotation call ────────────────────────────────────────────────

    fn parse_annotation_call(&mut self) {
        self.builder.start_node(AnnotationCall.into());
        self.eat_trivia();
        self.bump(); // '@'
        self.eat_trivia();

        // Library: @lib::Name(...)
        // Simple: @name(...)
        if self.peek_non_trivia(1) == Some(ColonColon) {
            // Library annotation: ident :: ident ( args )
            self.expect_ident(); // library name
            self.expect(ColonColon);
            self.eat_trivia();
            self.expect_ident(); // annotation name
        } else {
            // Simple annotation: @name(...) e.g. @default(now)
            self.expect_ident(); // annotation name
        }
        self.eat_trivia();
        if self.current() == Some(LParen) {
            self.bump();
            self.eat_trivia();
            if self.current_non_trivia() != Some(RParen) {
                self.parse_annotation_args_inner();
            }
            self.expect(RParen);
        }

        self.builder.finish_node();
    }

    fn parse_annotation_args_inner(&mut self) {
        self.builder.start_node(AnnotationArgs.into());
        self.parse_annotation_arg();
        while self.eat(Comma) {
            self.eat_trivia();
            if self.current_non_trivia() == Some(RParen) {
                break; // trailing comma
            }
            self.parse_annotation_arg();
        }
        self.builder.finish_node();
    }

    fn parse_annotation_arg(&mut self) {
        self.builder.start_node(AnnotationArg.into());
        self.eat_trivia();
        // name = value   OR   just a literal value (for positional)
        if self.peek_non_trivia(1) == Some(Eq) {
            self.expect_ident_or_keyword(); // param name (could be `fallback`)
            self.bump(); // '='
            self.eat_trivia();
        }
        self.parse_annotation_value();
        self.builder.finish_node();
    }

    fn parse_annotation_value(&mut self) {
        self.builder.start_node(AnnotationValue.into());
        self.eat_trivia();
        match self.current() {
            Some(LBrace) => {
                self.bump(); // '{'
                self.eat_trivia();
                if self.current_non_trivia() != Some(RBrace) {
                    self.parse_annotation_value_entries();
                }
                self.expect(RBrace);
            }
            _ => self.parse_literal_or_annotation_value(),
        }
        self.builder.finish_node();
    }

    fn parse_annotation_value_entries(&mut self) {
        // Could be key=value pairs or a plain list
        self.parse_annotation_value_entry();
        while self.eat(Comma) {
            self.eat_trivia();
            if self.current_non_trivia() == Some(RBrace) {
                break;
            }
            self.parse_annotation_value_entry();
        }
    }

    fn parse_annotation_value_entry(&mut self) {
        self.eat_trivia();
        // key = value, key : value, or just value
        let has_key = self.peek_non_trivia(1) == Some(Eq)
            || self.peek_non_trivia(1) == Some(Colon);
        if has_key {
            self.bump(); // key
            self.eat_trivia();
            self.bump(); // '=' or ':'
            self.eat_trivia();
            self.parse_annotation_value();
        } else {
            self.parse_annotation_value();
        }
    }

    fn parse_literal_or_annotation_value(&mut self) {
        match self.current() {
            Some(StringLiteral) | Some(IntLiteral) | Some(FloatLiteral) => self.bump(),
            Some(KwTrue) | Some(KwFalse) => self.bump(),
            Some(Minus) => {
                // Negative number
                self.bump(); // '-'
                if self.current() == Some(IntLiteral) || self.current() == Some(FloatLiteral) {
                    self.bump();
                }
            }
            Some(Ident) => self.bump(), // enum value reference, etc.
            Some(LBrace) => {
                // Nested struct value
                self.bump(); // '{'
                self.eat_trivia();
                if self.current_non_trivia() != Some(RBrace) {
                    self.parse_annotation_value_entries();
                }
                self.expect(RBrace);
            }
            _ => {
                self.error_recover("expected value");
            }
        }
    }

    // ── Type references ────────────────────────────────────────────────

    fn parse_type_ref(&mut self) {
        self.builder.start_node(TypeRef.into());
        self.eat_trivia();
        match self.current() {
            Some(Brackets) => {
                // []T
                self.builder.start_node(ArrayType.into());
                self.bump(); // '[]'
                self.parse_type_ref(); // element type (recursive)
                self.builder.finish_node();
            }
            Some(KwMap) => {
                self.builder.start_node(MapType.into());
                self.bump(); // 'map'
                self.expect(LAngle);
                self.parse_type_ref(); // key
                self.expect(Comma);
                self.parse_type_ref(); // value
                self.expect(RAngle);
                self.builder.finish_node();
            }
            Some(KwPick) => self.parse_pick_type(),
            Some(KwOmit) => self.parse_omit_type(),
            _ => {
                // qualified_name possibly with type args: Name<T, U>
                self.parse_qualified_name();
                self.eat_trivia();
                if self.current() == Some(LAngle) {
                    self.parse_type_args();
                }
            }
        }

        // Optional `?`
        self.eat_trivia();
        if self.current() == Some(Question) {
            self.builder.start_node(OptionalMarker.into());
            self.bump();
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_type_args(&mut self) {
        // <Type, Type, ...>
        self.bump(); // '<'
        self.parse_type_ref();
        while self.eat(Comma) {
            self.parse_type_ref();
        }
        self.expect(RAngle);
    }

    fn parse_qualified_name(&mut self) {
        self.builder.start_node(QualifiedName.into());
        self.eat_trivia();
        self.expect_ident();
        while self.current() == Some(Dot) {
            self.bump(); // '.'
            self.expect_ident();
        }
        self.builder.finish_node();
    }

    fn parse_pick_type(&mut self) {
        self.builder.start_node(PickType.into());
        self.eat_trivia();
        self.bump(); // 'Pick'
        self.expect(LAngle);
        self.parse_type_ref(); // source type
        self.expect(Comma);
        self.parse_ident_list();
        self.expect(RAngle);
        self.builder.finish_node();
    }

    fn parse_omit_type(&mut self) {
        self.builder.start_node(OmitType.into());
        self.eat_trivia();
        self.bump(); // 'Omit'
        self.expect(LAngle);
        self.parse_type_ref(); // source type
        self.expect(Comma);
        self.parse_ident_list();
        self.expect(RAngle);
        self.builder.finish_node();
    }

    fn parse_ident_list(&mut self) {
        self.builder.start_node(IdentList.into());
        self.eat_trivia();
        self.parse_qualified_name(); // supports both Ident and Ident.Ident
        while self.eat(Comma) {
            self.eat_trivia();
            // Stop before '>'
            if self.current_non_trivia() == Some(RAngle) {
                break;
            }
            self.parse_qualified_name();
        }
        self.builder.finish_node();
    }

    fn parse_type_params(&mut self) {
        self.builder.start_node(TypeParams.into());
        self.bump(); // '<'
        self.eat_trivia();
        self.expect_ident();
        while self.eat(Comma) {
            self.eat_trivia();
            self.expect_ident();
        }
        self.expect(RAngle);
        self.builder.finish_node();
    }

    // ── Token expectations ─────────────────────────────────────────────

    /// Accept an identifier or any keyword in identifier position.
    /// Keywords are valid identifiers in contexts like package names,
    /// field names, type names, import paths, etc.
    fn expect_ident(&mut self) {
        self.eat_trivia();
        if let Some(k) = self.current() {
            if k == Ident || is_keyword(k) {
                self.bump();
                return;
            }
        }
        let offset = self.current_offset();
        self.errors.push(ParseError {
            message: "expected identifier".into(),
            range: offset..offset,
        });
    }

    /// Alias for `expect_ident` — kept for readability at call sites
    /// where the intent is specifically to accept keywords (annotation targets, etc.).
    fn expect_ident_or_keyword(&mut self) {
        self.expect_ident();
    }

    fn expect_int(&mut self) {
        self.eat_trivia();
        if self.current() == Some(IntLiteral) {
            self.bump();
        } else {
            let offset = self.current_offset();
            self.errors.push(ParseError {
                message: "expected integer".into(),
                range: offset..offset,
            });
        }
    }
}

fn is_trivia(kind: SyntaxKind) -> bool {
    matches!(kind, Whitespace | LineComment | BlockComment)
}

fn is_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        KwPackage | KwImport | KwAs | KwType | KwShape | KwEnum | KwOneof
        | KwService | KwRpc | KwAnnotation | KwFor | KwVoid | KwStream
        | KwMap | KwPick | KwOmit | KwTrue | KwFalse | KwSelf
        | KwReserved
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax_kind::SyntaxNode;

    /// Parse and assert no errors.
    fn parse_ok(source: &str) -> SyntaxNode {
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "unexpected parse errors: {:?}",
            result.errors
        );
        result.syntax()
    }

    /// Collect all node kinds in pre-order (no tokens).
    fn node_kinds(node: &SyntaxNode) -> Vec<SyntaxKind> {
        let mut kinds = vec![];
        for event in node.preorder() {
            if let rowan::WalkEvent::Enter(n) = event {
                kinds.push(n.kind());
            }
        }
        kinds
    }

    /// Assert that the source text round-trips through the CST.
    fn assert_lossless(source: &str) {
        let result = parse(source);
        assert_eq!(result.syntax().text().to_string(), source);
    }

    // ── Lossless round-trip ────────────────────────────────────────────

    #[test]
    fn roundtrip_empty() {
        assert_lossless("");
    }

    #[test]
    fn roundtrip_package() {
        assert_lossless("package example;\n");
    }

    #[test]
    fn roundtrip_full_file() {
        let source = r#"package example;

import uuid;
import github.com/org/database;

type User {
    uuid.UUID id = 1;
    string email = 2;
    []Order orders = 3;
    map<string, string> metadata = 4;
    string? nickname = 5;
    @reserved(6)
}
"#;
        assert_lossless(source);
    }

    // ── Package ────────────────────────────────────────────────────────

    #[test]
    fn parse_package_decl() {
        let root = parse_ok("package example;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&Root));
        assert!(kinds.contains(&PackageDecl));
    }

    // ── Import ─────────────────────────────────────────────────────────

    #[test]
    fn parse_import_simple() {
        let root = parse_ok("package p;\nimport uuid;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ImportDecl));
        assert!(kinds.contains(&ImportPath));
    }

    #[test]
    fn parse_import_qualified() {
        let root = parse_ok("package p;\nimport github.com/org/database;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ImportDecl));
    }

    #[test]
    fn parse_import_with_alias() {
        let root = parse_ok("package p;\nimport github.com/org/database as mydb;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ImportDecl));
    }

    // ── Type ───────────────────────────────────────────────────────────

    #[test]
    fn parse_type_simple() {
        let root = parse_ok("type User { string email = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&TypeDecl));
        assert!(kinds.contains(&TypeBody));
        assert!(kinds.contains(&FieldDecl));
    }

    #[test]
    fn parse_type_alias() {
        let root = parse_ok("type Id = uuid.UUID;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&TypeDecl));
        assert!(kinds.contains(&TypeAlias));
    }

    #[test]
    fn parse_type_alias_bool() {
        let root = parse_ok("type IsAdmin = bool;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&TypeAlias));
    }

    #[test]
    fn parse_type_generic() {
        let root = parse_ok("type Paginated<T> { []T data = 1; int total = 2; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&TypeParams));
        assert!(kinds.contains(&ArrayType));
    }

    #[test]
    fn parse_pick() {
        let root = parse_ok("type Sub = Pick<User, id, email>;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&PickType));
        assert!(kinds.contains(&IdentList));
    }

    #[test]
    fn parse_omit() {
        let root = parse_ok("type Without = Omit<User, name>;");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&OmitType));
    }

    #[test]
    fn parse_nested_type() {
        let root = parse_ok("type Order { type Address { string street = 1; } }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&NestedTypeDecl));
    }

    #[test]
    fn parse_nested_enum() {
        let root = parse_ok("type Order { enum Status { Active = 1; } }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&NestedEnumDecl));
    }

    // ── Fields ─────────────────────────────────────────────────────────

    #[test]
    fn parse_optional_field() {
        let root = parse_ok("type T { string? nickname = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&OptionalMarker));
    }

    #[test]
    fn parse_array_field() {
        let root = parse_ok("type T { []string tags = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ArrayType));
    }

    #[test]
    fn parse_map_field() {
        let root = parse_ok("type T { map<string, string> meta = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&MapType));
    }

    #[test]
    fn parse_qualified_type_field() {
        let root = parse_ok("type T { uuid.UUID id = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&QualifiedName));
    }

    // ── Shape ──────────────────────────────────────────────────────────

    #[test]
    fn parse_shape_simple() {
        let root = parse_ok("shape Timestamps { uint64 created_at; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ShapeDecl));
        assert!(kinds.contains(&ShapeField));
    }

    #[test]
    fn parse_shape_generic() {
        let root = parse_ok("shape Wrapper<T> { T value; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ShapeDecl));
        assert!(kinds.contains(&TypeParams));
    }

    #[test]
    fn parse_shape_include() {
        let root = parse_ok("shape Combined { ShapeA; ShapeB, ShapeC; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ShapeInclude));
    }

    #[test]
    fn parse_shape_injection() {
        let root = parse_ok("type User { MyShape(1..4) string email = 5; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ShapeInjection));
        assert!(kinds.contains(&FieldDecl));
    }

    // ── Enum ───────────────────────────────────────────────────────────

    #[test]
    fn parse_enum() {
        let root = parse_ok("enum Status { Pending = 1; Active = 2; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&EnumDecl));
        assert!(kinds.contains(&EnumValueDecl));
    }

    #[test]
    fn parse_enum_with_annotation() {
        let root = parse_ok(
            "enum Status { Active = 1; @deprecated(reason=Active) Deleted = 2; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationCall));
        assert!(kinds.contains(&EnumValueDecl));
    }

    // ── Oneof ──────────────────────────────────────────────────────────

    #[test]
    fn parse_oneof() {
        let root = parse_ok(
            "type T { oneof address { Address home = 1; Address work = 2; } }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&OneofDecl));
        assert!(kinds.contains(&OneofField));
    }

    // ── Service & RPC ──────────────────────────────────────────────────

    #[test]
    fn parse_service() {
        let root = parse_ok(
            "service UserAPI { rpc GetUser(void) -> User; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ServiceDecl));
        assert!(kinds.contains(&RpcDecl));
        assert!(kinds.contains(&RpcParam));
    }

    #[test]
    fn parse_rpc_streaming() {
        let root = parse_ok(
            "service S { rpc Sync(stream User) -> stream User; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&RpcDecl));
    }

    #[test]
    fn parse_rpc_inline_type() {
        let root = parse_ok(
            "service S { rpc Create({ string name = 1; }) -> void; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&InlineType));
    }

    // ── Projections ────────────────────────────────────────────────────

    #[test]
    fn parse_projection() {
        let root = parse_ok(
            "type UserMini { uuid.UUID id = 1 <- User.id; string email = 2 <- User.email; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&MappingSource));
    }

    #[test]
    fn parse_projection_nested() {
        let root = parse_ok(
            "type Flat { string city = 1 <- Order.billing.city; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&MappingSource));
    }

    #[test]
    fn parse_projection_wildcard() {
        let root = parse_ok(
            "type Flat { string tx = 1 <- Payment.method.*.transaction_id; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&MappingSource));
    }

    #[test]
    fn parse_projection_in_oneof() {
        let root = parse_ok(
            "type T { oneof email { string personal = 1 <- User.personal_email; string work = 2 <- User.work_email; } }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&OneofDecl));
        assert!(kinds.contains(&MappingSource));
    }

    // ── Annotations ────────────────────────────────────────────────────

    #[test]
    fn parse_annotation_call_simple() {
        let root = parse_ok(r#"@database::Table(table_name="users") type User { string id = 1; }"#);
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationCall));
    }

    #[test]
    fn parse_annotation_no_args() {
        let root = parse_ok("@validate::Required type User { string id = 1; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationCall));
    }

    #[test]
    fn parse_builtin_default() {
        let root = parse_ok("shape S { uint64 created_at @default(now); }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationCall));
    }

    #[test]
    fn parse_reserved_single() {
        let root = parse_ok("type T { string a = 1; reserved 2; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ReservedDecl));
    }

    #[test]
    fn parse_reserved_multiple() {
        let root = parse_ok("type T { string a = 1; reserved 2, 3, 5; }");
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&ReservedDecl));
    }

    #[test]
    fn parse_annotation_decl() {
        let root = parse_ok(
            "annotation Table for type { string table_name; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationDecl));
        assert!(kinds.contains(&AnnotationTargets));
        assert!(kinds.contains(&AnnotationField));
    }

    #[test]
    fn parse_annotation_multi_target() {
        let root = parse_ok(
            "annotation A for type|field|enum { bool flag; }",
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationTargets));
    }

    #[test]
    fn parse_annotation_composition() {
        let root = parse_ok(
            r#"annotation Email for field { validate::Pattern("^[a-z]+$"); validate::Length(min=3, max=255); }"#,
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationComposition));
    }

    #[test]
    fn parse_annotation_with_optional_field() {
        let root = parse_ok(
            r#"annotation Col for field { string? column_name = ""; bool primary_key = false; }"#,
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationField));
    }

    // ── Annotation with nested values ──────────────────────────────────

    #[test]
    fn parse_annotation_nested_value() {
        let root = parse_ok(
            r#"@lib::A(metadata={key1:"value1", key2:"value2"}) type T { string id = 1; }"#,
        );
        let kinds = node_kinds(&root);
        assert!(kinds.contains(&AnnotationCall));
        assert!(kinds.contains(&AnnotationArgs));
    }

    // ── Error recovery ─────────────────────────────────────────────────

    #[test]
    fn error_missing_semicolon() {
        let result = parse("package example");
        assert!(!result.errors.is_empty());
        // Still produces a tree
        let root = result.syntax();
        assert_eq!(root.kind(), Root);
    }

    #[test]
    fn error_still_lossless() {
        // Even with errors the full source is in the tree
        let source = "package example";
        let result = parse(source);
        assert_eq!(result.syntax().text().to_string(), source);
    }

    // ── Full files from syntax examples ────────────────────────────────

    #[test]
    fn parse_models_ogham() {
        let source = r#"package example;

import uuid;

type Id = uuid.UUID;

shape Timestamps {
    uint64 created_at;
    uint64 updated_at;
}

shape SafeDelete {
    uint64? deleted_at;
}

type User {
    MyShape(1..4)
    string email = 5;
    string name = 6;
    []Order orders = 7;
    oneof address {
        Address home = 8;
        Address work = 9;
        string string_address = 10;
    }
    float score = 11;
    map<string, string> metadata = 12;
    reserved 13;
}

type PublicUser = Pick<User, id, email, name>;

type WithoutPassword = Omit<User, password_hash>;

enum OrderStatus {
    Pending = 1;
    Processing = 2;
    Completed = 3;
    Cancelled = 5;
}

type Order {
    uuid.UUID id = 1;
    User owner = 2;
    []Item items = 3;
    OrderStatus status = 4;
}
"#;
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "parse errors: {:?}",
            result.errors
        );
        assert_eq!(result.syntax().text().to_string(), source);
    }

    #[test]
    fn parse_service_ogham() {
        let source = r#"package example;

type Paginated<T> {
    []T data = 1;
    int total = 2;
    string next_page_token = 3;
}

service UserAPI {
    rpc GetUser(uuid.UUID) -> User;
    rpc CreateUser({ string name = 1; string email = 2; }) -> User;
    rpc ListUsers(void) -> stream User;
    rpc SyncUsers(stream User) -> stream User;
}
"#;
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "parse errors: {:?}",
            result.errors
        );
        assert_eq!(result.syntax().text().to_string(), source);
    }

    #[test]
    fn parse_projections_ogham() {
        let source = r#"package example;

import uuid;

type User {
    uuid.UUID id = 1;
    string first_name = 2;
    string email = 3;
}

type UserMini {
    uuid.UUID id = 1 <- User.id;
    string email = 2 <- User.email;
}

type UserDashboard {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
}
"#;
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "parse errors: {:?}",
            result.errors
        );
        assert_eq!(result.syntax().text().to_string(), source);
    }

    #[test]
    fn parse_annotation_def_ogham() {
        let source = r#"annotation Table for type {
    string table_name;
}
annotation Column for field {
    string column_name;
    bool primary_key = false;
}
"#;
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "parse errors: {:?}",
            result.errors
        );
    }

    // ── Hang prevention ────────────────────────────────────────────────

    #[test]
    fn no_hang_on_invalid_annotation_syntax() {
        // This pseudo-syntax previously caused an infinite loop
        let result = parse("@<library>::<name>() type T { string id = 1; }");
        // Should produce errors but NOT hang
        assert!(!result.errors.is_empty());
        // Tree is still produced
        assert_eq!(result.syntax().kind(), Root);
    }

    #[test]
    fn no_hang_on_garbage_in_type_body() {
        let result = parse("type T { ??? }");
        assert!(!result.errors.is_empty());
        assert_eq!(result.syntax().kind(), Root);
    }

    #[test]
    fn no_hang_on_garbage_in_enum() {
        let result = parse("enum E { ??? }");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn no_hang_on_garbage_in_service() {
        let result = parse("service S { ??? }");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn no_hang_on_garbage_in_shape() {
        let result = parse("shape S { ??? }");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn no_hang_on_garbage_in_annotation_def() {
        let result = parse("annotation A for type { ??? }");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn no_hang_on_deeply_broken_file() {
        let result = parse("@@ {{ }} << >> !! type { enum { oneof { @@ }}");
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn no_hang_on_empty_braces_everywhere() {
        let result = parse("type T {} enum E {} service S {} shape Sh {}");
        // No hang, and these are actually valid (empty bodies)
        assert_eq!(result.syntax().kind(), Root);
    }
}

//! Typed AST layer over the lossless rowan CST.
//!
//! Each struct wraps a [`SyntaxNode`] of a specific [`SyntaxKind`] and
//! provides typed accessor methods for its children. This keeps all
//! downstream code (type checker, LSP, linter) free from raw tree traversal.

use crate::syntax_kind::{SyntaxKind, SyntaxNode, SyntaxToken};

// ── AstNode trait ──────────────────────────────────────────────────────

/// Common interface for all typed AST nodes.
pub trait AstNode: Sized {
    fn can_cast(kind: SyntaxKind) -> bool;
    fn cast(node: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

// ── Macro for boilerplate ──────────────────────────────────────────────

macro_rules! ast_node {
    ($name:ident, $kind:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == SyntaxKind::$kind
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self { syntax: node })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    };
}

// ── Helper functions ───────────────────────────────────────────────────

/// Find the first child node of a given kind.
fn child_node(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxNode> {
    parent.children().find(|c| c.kind() == kind)
}

/// Find the first token of a given kind among direct children.
fn child_token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
    parent.children_with_tokens().filter_map(|el| {
        match el {
            rowan::NodeOrToken::Token(t) if t.kind() == kind => Some(t),
            _ => None,
        }
    }).next()
}

/// Find the first identifier token — skipping structural keywords that introduce
/// declarations (`type`, `enum`, `shape`, `service`, `rpc`, `annotation`, `oneof`,
/// `package`, `import`, `for`, `as`). Other keywords in identifier position are accepted.
fn first_ident_token(parent: &SyntaxNode) -> Option<SyntaxToken> {
    parent.children_with_tokens().find_map(|el| match el {
        rowan::NodeOrToken::Token(t) => {
            let k = t.kind();
            if k == SyntaxKind::Ident || (k.is_keyword() && !k.is_structural_keyword()) {
                Some(t)
            } else {
                None
            }
        }
        _ => None,
    })
}

/// Collect all typed child nodes that can be cast to `T`.
fn children_of_type<T: AstNode>(parent: &SyntaxNode) -> Vec<T> {
    parent.children().filter_map(T::cast).collect()
}

/// Find the first child that can be cast to `T`.
fn first_child_of_type<T: AstNode>(parent: &SyntaxNode) -> Option<T> {
    parent.children().find_map(T::cast)
}

// ── Root ───────────────────────────────────────────────────────────────

ast_node!(Root, Root);

impl Root {
    pub fn package_decl(&self) -> Option<PackageDecl> {
        first_child_of_type(&self.syntax)
    }

    pub fn imports(&self) -> Vec<ImportDecl> {
        children_of_type(&self.syntax)
    }

    pub fn type_decls(&self) -> Vec<TypeDecl> {
        children_of_type(&self.syntax)
    }

    pub fn shape_decls(&self) -> Vec<ShapeDecl> {
        children_of_type(&self.syntax)
    }

    pub fn enum_decls(&self) -> Vec<EnumDecl> {
        children_of_type(&self.syntax)
    }

    pub fn service_decls(&self) -> Vec<ServiceDecl> {
        children_of_type(&self.syntax)
    }

    pub fn annotation_decls(&self) -> Vec<AnnotationDecl> {
        children_of_type(&self.syntax)
    }
}

// ── Package ────────────────────────────────────────────────────────────

ast_node!(PackageDecl, PackageDecl);

impl PackageDecl {
    /// The package name identifier.
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }
}

// ── Import ─────────────────────────────────────────────────────────────

ast_node!(ImportDecl, ImportDecl);

impl ImportDecl {
    pub fn path(&self) -> Option<ImportPath> {
        first_child_of_type(&self.syntax)
    }

    /// The alias identifier after `as`, if present.
    pub fn alias(&self) -> Option<SyntaxToken> {
        // Find the `as` keyword, then the next Ident token after it
        let mut found_as = false;
        for el in self.syntax.children_with_tokens() {
            match el {
                rowan::NodeOrToken::Token(ref t) if t.kind() == SyntaxKind::KwAs => {
                    found_as = true;
                }
                rowan::NodeOrToken::Token(ref t) if found_as && (t.kind() == SyntaxKind::Ident || t.kind().is_keyword()) => {
                    return Some(t.clone());
                }
                _ => {}
            }
        }
        None
    }
}

ast_node!(ImportPath, ImportPath);

impl ImportPath {
    /// The full import path as text (e.g., `github.com/org/database`).
    pub fn text(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if !t.kind().is_trivia() => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect()
    }
}

impl SyntaxKind {
    fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::LineComment | SyntaxKind::BlockComment)
    }

    /// Keywords that introduce declarations — NOT valid as identifiers
    /// in `first_ident_token` context (they precede the name).
    pub fn is_structural_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::KwType | SyntaxKind::KwShape | SyntaxKind::KwEnum
            | SyntaxKind::KwOneof | SyntaxKind::KwService | SyntaxKind::KwRpc
            | SyntaxKind::KwAnnotation | SyntaxKind::KwPackage | SyntaxKind::KwImport
            | SyntaxKind::KwFor | SyntaxKind::KwAs | SyntaxKind::KwStream
            | SyntaxKind::KwPick | SyntaxKind::KwOmit
        )
    }

    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::KwPackage | SyntaxKind::KwImport | SyntaxKind::KwAs
            | SyntaxKind::KwType | SyntaxKind::KwShape | SyntaxKind::KwEnum
            | SyntaxKind::KwOneof | SyntaxKind::KwService | SyntaxKind::KwRpc
            | SyntaxKind::KwAnnotation | SyntaxKind::KwFor | SyntaxKind::KwVoid
            | SyntaxKind::KwStream | SyntaxKind::KwMap | SyntaxKind::KwPick
            | SyntaxKind::KwOmit | SyntaxKind::KwTrue | SyntaxKind::KwFalse
            | SyntaxKind::KwNow | SyntaxKind::KwSelf | SyntaxKind::KwDefault
            | SyntaxKind::KwCast | SyntaxKind::KwRemoved | SyntaxKind::KwReserved
            | SyntaxKind::KwFallback
        )
    }
}

// ── Type ───────────────────────────────────────────────────────────────

ast_node!(TypeDecl, TypeDecl);

impl TypeDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn type_params(&self) -> Option<TypeParams> {
        first_child_of_type(&self.syntax)
    }

    pub fn alias(&self) -> Option<TypeAlias> {
        first_child_of_type(&self.syntax)
    }

    pub fn body(&self) -> Option<TypeBody> {
        first_child_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        // Annotations are siblings before this node in the parent,
        // but in our CST they are children of the parent node, not of TypeDecl.
        // However, annotations on types ARE inside the top_level_decl flow.
        // Actually in our parser, annotations are emitted before the type_decl node.
        // So we look at preceding siblings.
        preceding_annotations(&self.syntax)
    }
}

ast_node!(TypeAlias, TypeAlias);

impl TypeAlias {
    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn pick_type(&self) -> Option<PickType> {
        first_child_of_type(&self.syntax)
    }

    pub fn omit_type(&self) -> Option<OmitType> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(TypeBody, TypeBody);

impl TypeBody {
    pub fn fields(&self) -> Vec<FieldDecl> {
        children_of_type(&self.syntax)
    }

    pub fn shape_injections(&self) -> Vec<ShapeInjection> {
        children_of_type(&self.syntax)
    }

    pub fn oneofs(&self) -> Vec<OneofDecl> {
        children_of_type(&self.syntax)
    }

    pub fn nested_types(&self) -> Vec<NestedTypeDecl> {
        children_of_type(&self.syntax)
    }

    pub fn nested_enums(&self) -> Vec<NestedEnumDecl> {
        children_of_type(&self.syntax)
    }

    pub fn reserved_decls(&self) -> Vec<ReservedDecl> {
        children_of_type(&self.syntax)
    }
}

ast_node!(TypeParams, TypeParams);

impl TypeParams {
    /// All type parameter names.
    pub fn params(&self) -> Vec<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => Some(t),
                _ => None,
            })
            .collect()
    }
}

// ── Field ──────────────────────────────────────────────────────────────

ast_node!(FieldDecl, FieldDecl);

impl FieldDecl {
    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    /// The field name identifier.
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    /// The field number.
    pub fn field_number(&self) -> Option<u32> {
        child_token(&self.syntax, SyntaxKind::IntLiteral)
            .and_then(|t| t.text().parse().ok())
    }

    pub fn mapping(&self) -> Option<MappingSource> {
        first_child_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(MappingSource, MappingSource);

impl MappingSource {
    /// The full mapping path as segments (e.g., `["User", "id"]` or `["Payment", "method", "*", "amount"]`).
    pub fn segments(&self) -> Vec<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) => match t.kind() {
                    SyntaxKind::Ident | SyntaxKind::Star => Some(t.text().to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}

// ── Shape ──────────────────────────────────────────────────────────────

ast_node!(ShapeDecl, ShapeDecl);

impl ShapeDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn type_params(&self) -> Option<TypeParams> {
        first_child_of_type(&self.syntax)
    }

    pub fn fields(&self) -> Vec<ShapeField> {
        children_of_type(&self.syntax)
    }

    pub fn includes(&self) -> Vec<ShapeInclude> {
        children_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(ShapeField, ShapeField);

impl ShapeField {
    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    /// Trailing annotations (e.g., `@default(now)` after field name).
    pub fn annotations(&self) -> Vec<AnnotationCall> {
        children_of_type(&self.syntax)
    }
}

ast_node!(ShapeInclude, ShapeInclude);

impl ShapeInclude {
    /// Names of included shapes.
    pub fn names(&self) -> Vec<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => Some(t),
                _ => None,
            })
            .collect()
    }
}

ast_node!(ShapeInjection, ShapeInjection);

impl ShapeInjection {
    /// Shape name being injected.
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    /// Range start.
    pub fn range_start(&self) -> Option<u32> {
        let ints: Vec<_> = self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::IntLiteral => Some(t),
                _ => None,
            })
            .collect();
        ints.first().and_then(|t| t.text().parse().ok())
    }

    /// Range end.
    pub fn range_end(&self) -> Option<u32> {
        let ints: Vec<_> = self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::IntLiteral => Some(t),
                _ => None,
            })
            .collect();
        ints.get(1).and_then(|t| t.text().parse().ok())
    }
}

// ── Enum ───────────────────────────────────────────────────────────────

ast_node!(EnumDecl, EnumDecl);

impl EnumDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn values(&self) -> Vec<EnumValueDecl> {
        children_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(EnumValueDecl, EnumValueDecl);

impl EnumValueDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn value(&self) -> Option<u32> {
        child_token(&self.syntax, SyntaxKind::IntLiteral)
            .and_then(|t| t.text().parse().ok())
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

// ── Oneof ──────────────────────────────────────────────────────────────

ast_node!(OneofDecl, OneofDecl);

impl OneofDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn fields(&self) -> Vec<OneofField> {
        children_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(OneofField, OneofField);

impl OneofField {
    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn field_number(&self) -> Option<u32> {
        child_token(&self.syntax, SyntaxKind::IntLiteral)
            .and_then(|t| t.text().parse().ok())
    }

    pub fn mapping(&self) -> Option<MappingSource> {
        first_child_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

// ── Service & RPC ──────────────────────────────────────────────────────

ast_node!(ServiceDecl, ServiceDecl);

impl ServiceDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn rpcs(&self) -> Vec<RpcDecl> {
        children_of_type(&self.syntax)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(RpcDecl, RpcDecl);

impl RpcDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    /// Input and output params (always two: input at index 0, output at index 1).
    pub fn params(&self) -> Vec<RpcParam> {
        children_of_type(&self.syntax)
    }

    pub fn input(&self) -> Option<RpcParam> {
        self.params().into_iter().next()
    }

    pub fn output(&self) -> Option<RpcParam> {
        self.params().into_iter().nth(1)
    }

    pub fn annotations(&self) -> Vec<AnnotationCall> {
        preceding_annotations(&self.syntax)
    }
}

ast_node!(RpcParam, RpcParam);

impl RpcParam {
    pub fn is_void(&self) -> bool {
        child_token(&self.syntax, SyntaxKind::KwVoid).is_some()
    }

    pub fn is_stream(&self) -> bool {
        child_token(&self.syntax, SyntaxKind::KwStream).is_some()
    }

    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn inline_type(&self) -> Option<InlineType> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(InlineType, InlineType);

impl InlineType {
    pub fn fields(&self) -> Vec<FieldDecl> {
        children_of_type(&self.syntax)
    }
}

// ── Annotation definition ──────────────────────────────────────────────

ast_node!(AnnotationDecl, AnnotationDecl);

impl AnnotationDecl {
    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn targets(&self) -> Option<AnnotationTargets> {
        first_child_of_type(&self.syntax)
    }

    pub fn fields(&self) -> Vec<AnnotationField> {
        children_of_type(&self.syntax)
    }

    pub fn compositions(&self) -> Vec<AnnotationComposition> {
        children_of_type(&self.syntax)
    }
}

ast_node!(AnnotationTargets, AnnotationTargets);

impl AnnotationTargets {
    /// Target names as strings (e.g., `["type", "field", "enum"]`).
    pub fn targets(&self) -> Vec<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) => match t.kind() {
                    SyntaxKind::Ident
                    | SyntaxKind::KwType
                    | SyntaxKind::KwShape
                    | SyntaxKind::KwEnum
                    | SyntaxKind::KwService
                    | SyntaxKind::KwRpc
                    | SyntaxKind::KwOneof => Some(t.text().to_string()),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }
}

ast_node!(AnnotationField, AnnotationField);

impl AnnotationField {
    pub fn type_ref(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn name(&self) -> Option<SyntaxToken> {
        first_ident_token(&self.syntax)
    }

    pub fn is_optional(&self) -> bool {
        child_token(&self.syntax, SyntaxKind::Question).is_some()
    }

    pub fn inline_type(&self) -> Option<InlineAnnotationType> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(AnnotationComposition, AnnotationComposition);

impl AnnotationComposition {
    /// The qualifier (library name) and annotation name as `(qualifier, name)`.
    pub fn qualified_name(&self) -> Option<(String, String)> {
        let idents: Vec<_> = self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();
        if idents.len() >= 2 {
            Some((idents[0].clone(), idents[1].clone()))
        } else {
            None
        }
    }

    pub fn args(&self) -> Option<AnnotationArgs> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(InlineAnnotationType, InlineAnnotationType);

impl InlineAnnotationType {
    pub fn fields(&self) -> Vec<AnnotationField> {
        children_of_type(&self.syntax)
    }

    pub fn compositions(&self) -> Vec<AnnotationComposition> {
        children_of_type(&self.syntax)
    }
}

// ── Annotation call ────────────────────────────────────────────────────

ast_node!(AnnotationCall, AnnotationCall);

impl AnnotationCall {
    /// Returns `true` if this is a built-in annotation (`@default`, `@cast`, `@removed`, `@reserved`).
    pub fn is_builtin(&self) -> bool {
        self.syntax.children_with_tokens().any(|el| matches!(
            el,
            rowan::NodeOrToken::Token(ref t) if matches!(
                t.kind(),
                SyntaxKind::KwDefault | SyntaxKind::KwCast | SyntaxKind::KwRemoved | SyntaxKind::KwReserved
            )
        ))
    }

    /// For built-in annotations, the keyword name.
    pub fn builtin_name(&self) -> Option<SyntaxToken> {
        self.syntax.children_with_tokens().find_map(|el| match el {
            rowan::NodeOrToken::Token(t) if matches!(
                t.kind(),
                SyntaxKind::KwDefault | SyntaxKind::KwCast | SyntaxKind::KwRemoved | SyntaxKind::KwReserved
            ) => Some(t),
            _ => None,
        })
    }

    /// For library annotations, returns `(library, name)`.
    pub fn library_name(&self) -> Option<(String, String)> {
        let idents: Vec<_> = self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();
        if idents.len() >= 2 {
            Some((idents[0].clone(), idents[1].clone()))
        } else {
            None
        }
    }

    pub fn args(&self) -> Option<AnnotationArgs> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(AnnotationArgs, AnnotationArgs);

impl AnnotationArgs {
    pub fn args(&self) -> Vec<AnnotationArg> {
        children_of_type(&self.syntax)
    }
}

ast_node!(AnnotationArg, AnnotationArg);

impl AnnotationArg {
    /// The parameter name (if `name=value` form).
    pub fn name(&self) -> Option<SyntaxToken> {
        // Name is the ident before `=`
        let mut tokens: Vec<_> = self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if !t.kind().is_trivia() => Some(t),
                _ => None,
            })
            .collect();
        if tokens.len() >= 2 && tokens[1].kind() == SyntaxKind::Eq {
            Some(tokens.remove(0))
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<AnnotationValue> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(AnnotationValue, AnnotationValue);

// ── Type references ────────────────────────────────────────────────────

ast_node!(TypeRef, TypeRef);

impl TypeRef {
    pub fn is_optional(&self) -> bool {
        child_node(&self.syntax, SyntaxKind::OptionalMarker).is_some()
    }

    pub fn array_type(&self) -> Option<ArrayType> {
        first_child_of_type(&self.syntax)
    }

    pub fn map_type(&self) -> Option<MapType> {
        first_child_of_type(&self.syntax)
    }

    pub fn pick_type(&self) -> Option<PickType> {
        first_child_of_type(&self.syntax)
    }

    pub fn omit_type(&self) -> Option<OmitType> {
        first_child_of_type(&self.syntax)
    }

    pub fn qualified_name(&self) -> Option<QualifiedName> {
        first_child_of_type(&self.syntax)
    }

    /// Type arguments for generic instantiation (e.g., `Paginated<User>` → `[User]`).
    /// Returns child TypeRef nodes (the generic type arguments).
    pub fn type_args(&self) -> Vec<TypeRef> {
        children_of_type(&self.syntax)
    }
}

ast_node!(ArrayType, ArrayType);

impl ArrayType {
    /// The element type.
    pub fn element_type(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(MapType, MapType);

impl MapType {
    /// Key and value types.
    pub fn types(&self) -> Vec<TypeRef> {
        children_of_type(&self.syntax)
    }

    pub fn key_type(&self) -> Option<TypeRef> {
        self.types().into_iter().next()
    }

    pub fn value_type(&self) -> Option<TypeRef> {
        self.types().into_iter().nth(1)
    }
}

ast_node!(OptionalMarker, OptionalMarker);

ast_node!(PickType, PickType);

impl PickType {
    pub fn source_type(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn fields(&self) -> Option<IdentList> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(OmitType, OmitType);

impl OmitType {
    pub fn source_type(&self) -> Option<TypeRef> {
        first_child_of_type(&self.syntax)
    }

    pub fn fields(&self) -> Option<IdentList> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(QualifiedName, QualifiedName);

impl QualifiedName {
    /// All segments of the qualified name (e.g., `["uuid", "UUID"]`).
    pub fn segments(&self) -> Vec<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect()
    }

    /// The full dotted name as a string.
    pub fn text(&self) -> String {
        self.segments().join(".")
    }
}

ast_node!(IdentList, IdentList);

impl IdentList {
    pub fn names(&self) -> Vec<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|el| match el {
                rowan::NodeOrToken::Token(t) if t.kind() == SyntaxKind::Ident => Some(t),
                _ => None,
            })
            .collect()
    }
}

// ── Nested & Reserved ──────────────────────────────────────────────────

ast_node!(NestedTypeDecl, NestedTypeDecl);

impl NestedTypeDecl {
    pub fn type_decl(&self) -> Option<TypeDecl> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(NestedEnumDecl, NestedEnumDecl);

impl NestedEnumDecl {
    pub fn enum_decl(&self) -> Option<EnumDecl> {
        first_child_of_type(&self.syntax)
    }
}

ast_node!(ReservedDecl, ReservedDecl);

impl ReservedDecl {
    pub fn field_number(&self) -> Option<u32> {
        child_token(&self.syntax, SyntaxKind::IntLiteral)
            .and_then(|t| t.text().parse().ok())
    }
}

// ── Annotation-finding helpers ─────────────────────────────────────────

/// Find annotation calls that are preceding siblings of this node.
fn preceding_annotations(node: &SyntaxNode) -> Vec<AnnotationCall> {
    let mut annotations = Vec::new();
    let mut sibling = node.prev_sibling();
    while let Some(s) = sibling {
        if s.kind() == SyntaxKind::AnnotationCall {
            if let Some(a) = AnnotationCall::cast(s.clone()) {
                annotations.push(a);
            }
            sibling = s.prev_sibling();
        } else {
            break;
        }
    }
    annotations.reverse();
    annotations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn parse_root(source: &str) -> Root {
        let result = parser::parse(source);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        Root::cast(result.syntax()).unwrap()
    }

    #[test]
    fn package_name() {
        let root = parse_root("package example;");
        let pkg = root.package_decl().unwrap();
        assert_eq!(pkg.name().unwrap().text(), "example");
    }

    #[test]
    fn imports() {
        let root = parse_root("package p;\nimport uuid;\nimport github.com/org/db as mydb;");
        let imports = root.imports();
        assert_eq!(imports.len(), 2);

        assert_eq!(imports[0].path().unwrap().text(), "uuid");
        assert!(imports[0].alias().is_none());

        assert_eq!(imports[1].path().unwrap().text(), "github.com/org/db");
        assert_eq!(imports[1].alias().unwrap().text(), "mydb");
    }

    #[test]
    fn type_with_fields() {
        let root = parse_root("type User { string email = 1; int64 age = 2; }");
        let types = root.type_decls();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name().unwrap().text(), "User");

        let body = types[0].body().unwrap();
        let fields = body.fields();
        assert_eq!(fields.len(), 2);

        assert_eq!(fields[0].name().unwrap().text(), "email");
        assert_eq!(fields[0].field_number(), Some(1));
        assert_eq!(fields[1].name().unwrap().text(), "age");
        assert_eq!(fields[1].field_number(), Some(2));
    }

    #[test]
    fn type_alias() {
        let root = parse_root("type Id = uuid.UUID;");
        let types = root.type_decls();
        let alias = types[0].alias().unwrap();
        let tr = alias.type_ref().unwrap();
        let qn = tr.qualified_name().unwrap();
        assert_eq!(qn.segments(), vec!["uuid", "UUID"]);
    }

    #[test]
    fn type_generic_params() {
        let root = parse_root("type Paginated<T> { []T data = 1; }");
        let types = root.type_decls();
        let params = types[0].type_params().unwrap();
        let names: Vec<_> = params.params().iter().map(|t| t.text().to_string()).collect();
        assert_eq!(names, vec!["T"]);
    }

    #[test]
    fn pick_type() {
        let root = parse_root("type Sub = Pick<User, id, email>;");
        let types = root.type_decls();
        let alias = types[0].alias().unwrap();
        let pick = alias.pick_type().unwrap();
        let fields = pick.fields().unwrap();
        let names: Vec<_> = fields.names().iter().map(|t| t.text().to_string()).collect();
        assert_eq!(names, vec!["id", "email"]);
    }

    #[test]
    fn shape_with_fields_and_includes() {
        let root = parse_root("shape Combined { ShapeA; ShapeB, ShapeC; string name; }");
        let shapes = root.shape_decls();
        assert_eq!(shapes[0].name().unwrap().text(), "Combined");

        let includes = shapes[0].includes();
        assert_eq!(includes.len(), 2); // ShapeA; and ShapeB, ShapeC;
        assert_eq!(includes[0].names().len(), 1);
        assert_eq!(includes[1].names().len(), 2);

        let fields = shapes[0].fields();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name().unwrap().text(), "name");
    }

    #[test]
    fn shape_injection_range() {
        let root = parse_root("type User { MyShape(1..4) string email = 5; }");
        let body = root.type_decls()[0].body().unwrap();
        let injections = body.shape_injections();
        assert_eq!(injections.len(), 1);
        assert_eq!(injections[0].name().unwrap().text(), "MyShape");
        assert_eq!(injections[0].range_start(), Some(1));
        assert_eq!(injections[0].range_end(), Some(4));
    }

    #[test]
    fn enum_values() {
        let root = parse_root("enum Status { Pending = 1; Active = 2; }");
        let enums = root.enum_decls();
        assert_eq!(enums[0].name().unwrap().text(), "Status");

        let values = enums[0].values();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].name().unwrap().text(), "Pending");
        assert_eq!(values[0].value(), Some(1));
        assert_eq!(values[1].name().unwrap().text(), "Active");
        assert_eq!(values[1].value(), Some(2));
    }

    #[test]
    fn oneof_fields() {
        let root = parse_root(
            "type T { oneof addr { Address home = 1; Address work = 2; } }",
        );
        let body = root.type_decls()[0].body().unwrap();
        let oneofs = body.oneofs();
        assert_eq!(oneofs[0].name().unwrap().text(), "addr");
        let fields = oneofs[0].fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name().unwrap().text(), "home");
        assert_eq!(fields[0].field_number(), Some(1));
    }

    #[test]
    fn service_rpcs() {
        let root = parse_root(
            "service S { rpc Get(void) -> User; rpc List(void) -> stream User; }",
        );
        let services = root.service_decls();
        assert_eq!(services[0].name().unwrap().text(), "S");

        let rpcs = services[0].rpcs();
        assert_eq!(rpcs.len(), 2);
        assert_eq!(rpcs[0].name().unwrap().text(), "Get");

        let input = rpcs[0].input().unwrap();
        assert!(input.is_void());

        let output = rpcs[1].output().unwrap();
        assert!(output.is_stream());
    }

    #[test]
    fn field_type_optional() {
        let root = parse_root("type T { string? nick = 1; }");
        let fields = root.type_decls()[0].body().unwrap().fields();
        assert!(fields[0].type_ref().unwrap().is_optional());
    }

    #[test]
    fn field_type_array() {
        let root = parse_root("type T { []string tags = 1; }");
        let fields = root.type_decls()[0].body().unwrap().fields();
        let tr = fields[0].type_ref().unwrap();
        assert!(tr.array_type().is_some());
    }

    #[test]
    fn field_type_map() {
        let root = parse_root("type T { map<string, int> meta = 1; }");
        let fields = root.type_decls()[0].body().unwrap().fields();
        let tr = fields[0].type_ref().unwrap();
        let map = tr.map_type().unwrap();
        assert!(map.key_type().is_some());
        assert!(map.value_type().is_some());
    }

    #[test]
    fn field_mapping() {
        let root = parse_root("type M { string id = 1 <- User.id; }");
        let fields = root.type_decls()[0].body().unwrap().fields();
        let mapping = fields[0].mapping().unwrap();
        assert_eq!(mapping.segments(), vec!["User", "id"]);
    }

    #[test]
    fn field_mapping_wildcard() {
        let root = parse_root("type M { string tx = 1 <- Payment.method.*.tx_id; }");
        let fields = root.type_decls()[0].body().unwrap().fields();
        let mapping = fields[0].mapping().unwrap();
        assert_eq!(mapping.segments(), vec!["Payment", "method", "*", "tx_id"]);
    }

    #[test]
    fn annotation_decl() {
        let root = parse_root("annotation Table for type|field { string table_name; }");
        let decls = root.annotation_decls();
        assert_eq!(decls[0].name().unwrap().text(), "Table");
        let targets = decls[0].targets().unwrap().targets();
        assert_eq!(targets, vec!["type", "field"]);
        assert_eq!(decls[0].fields().len(), 1);
    }

    #[test]
    fn annotation_call_library() {
        let root = parse_root(r#"@database::Table(table_name="users") type T { string id = 1; }"#);
        // Annotations are emitted as preceding siblings to the type node
        // For this test, we verify they parse without errors
        assert_eq!(root.type_decls().len(), 1);
    }

    #[test]
    fn annotation_call_builtin() {
        let root = parse_root("type T { string id = 1; @reserved(2) }");
        let body = root.type_decls()[0].body().unwrap();
        let reserved = body.reserved_decls();
        assert_eq!(reserved.len(), 1);
        assert_eq!(reserved[0].field_number(), Some(2));
    }

    #[test]
    fn qualified_name_text() {
        let root = parse_root("type T { uuid.UUID id = 1; }");
        let field = &root.type_decls()[0].body().unwrap().fields()[0];
        let qn = field.type_ref().unwrap().qualified_name().unwrap();
        assert_eq!(qn.text(), "uuid.UUID");
        assert_eq!(qn.segments(), vec!["uuid", "UUID"]);
    }

    #[test]
    fn nested_type_decl() {
        let root = parse_root("type Order { type Address { string city = 1; } }");
        let body = root.type_decls()[0].body().unwrap();
        let nested = body.nested_types();
        assert_eq!(nested.len(), 1);
        let inner = nested[0].type_decl().unwrap();
        assert_eq!(inner.name().unwrap().text(), "Address");
    }
}

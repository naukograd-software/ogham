//! Ogham lexer built on logos.
//!
//! Converts source text into a flat stream of [`Token`]s.
//! Whitespace and comments are emitted as tokens (not skipped) so that
//! the downstream parser can build a lossless CST via rowan.

use logos::Logos;

/// Every lexical element in the Ogham language.
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    // ── Keywords ───────────────────────────────────────────────────────
    #[token("package")]
    KwPackage,
    #[token("import")]
    KwImport,
    #[token("as")]
    KwAs,
    #[token("type")]
    KwType,
    #[token("shape")]
    KwShape,
    #[token("enum")]
    KwEnum,
    #[token("oneof")]
    KwOneof,
    #[token("service")]
    KwService,
    #[token("rpc")]
    KwRpc,
    #[token("annotation")]
    KwAnnotation,
    #[token("for")]
    KwFor,
    #[token("void")]
    KwVoid,
    #[token("stream")]
    KwStream,
    #[token("map")]
    KwMap,
    #[token("Pick")]
    KwPick,
    #[token("Omit")]
    KwOmit,
    #[token("true")]
    KwTrue,
    #[token("false")]
    KwFalse,
    #[token("self")]
    KwSelf,

    // ── Built-in annotation names (after @) ────────────────────────────
    #[token("reserved")]
    KwReserved,

    // ── Punctuation ────────────────────────────────────────────────────
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("=")]
    Eq,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token("@")]
    At,
    #[token("?")]
    Question,
    #[token("|")]
    Pipe,
    #[token("->")]
    Arrow,
    #[token("<-")]
    BackArrow,
    #[token("..")]
    DotDot,
    #[token("*")]
    Star,
    #[token("[]")]
    Brackets,
    #[token("/")]
    Slash,
    #[token("-")]
    Minus,

    // ── Literals ───────────────────────────────────────────────────────
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,

    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLiteral,

    #[regex(r"[0-9]+")]
    IntLiteral,

    // ── Identifiers ────────────────────────────────────────────────────
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    // ── Trivia (whitespace & comments) ─────────────────────────────────
    #[regex(r"[ \t\r\n]+")]
    Whitespace,

    #[regex(r"//[^\n]*", allow_greedy = true)]
    LineComment,

    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
    BlockComment,
}

impl Token {
    /// Returns `true` for whitespace and comments — trivia that the parser
    /// attaches to the CST but does not interpret semantically.
    pub fn is_trivia(self) -> bool {
        matches!(self, Token::Whitespace | Token::LineComment | Token::BlockComment)
    }
}

/// A token with its byte span in the source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexedToken {
    pub kind: Token,
    pub span: Span,
}

/// Byte offset range in source text.
pub type Span = std::ops::Range<usize>;

/// Lex the entire source into a list of tokens.
/// Unrecognized bytes produce `None` kind — the caller decides how to report errors.
pub fn lex(source: &str) -> Vec<Result<LexedToken, Span>> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(kind) => tokens.push(Ok(LexedToken { kind, span })),
            Err(()) => tokens.push(Err(span)),
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: lex source and return only Ok tokens (kind, slice) pairs.
    fn lex_ok(source: &str) -> Vec<(Token, &str)> {
        let results = lex(source);
        results
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|t| (t.kind, &source[t.span]))
            .collect()
    }

    /// Helper: lex and strip trivia.
    fn lex_meaningful(source: &str) -> Vec<(Token, &str)> {
        lex_ok(source)
            .into_iter()
            .filter(|(k, _)| !k.is_trivia())
            .collect()
    }

    // ── Package & Import ───────────────────────────────────────────────

    #[test]
    fn package_decl() {
        let tokens = lex_meaningful("package example;");
        assert_eq!(
            tokens,
            vec![
                (Token::KwPackage, "package"),
                (Token::Ident, "example"),
                (Token::Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn import_simple() {
        let tokens = lex_meaningful("import uuid;");
        assert_eq!(
            tokens,
            vec![
                (Token::KwImport, "import"),
                (Token::Ident, "uuid"),
                (Token::Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn import_qualified() {
        let tokens = lex_meaningful("import github.com/org/database;");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwImport,
                Token::Ident,  // github
                Token::Dot,
                Token::Ident,  // com
                Token::Slash,
                Token::Ident,  // org
                Token::Slash,
                Token::Ident,  // database
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn import_with_alias() {
        let tokens = lex_meaningful("import github.com/org/database as mydb;");
        assert!(tokens.iter().any(|(k, _)| *k == Token::KwAs));
        assert!(tokens.iter().any(|(k, s)| *k == Token::Ident && *s == "mydb"));
    }

    // ── Type ───────────────────────────────────────────────────────────

    #[test]
    fn type_alias() {
        let tokens = lex_meaningful("type Id = uuid.UUID;");
        assert_eq!(
            tokens,
            vec![
                (Token::KwType, "type"),
                (Token::Ident, "Id"),
                (Token::Eq, "="),
                (Token::Ident, "uuid"),
                (Token::Dot, "."),
                (Token::Ident, "UUID"),
                (Token::Semicolon, ";"),
            ]
        );
    }

    #[test]
    fn type_body_with_fields() {
        let tokens = lex_meaningful("type User { string email = 5; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwType,
                Token::Ident,     // User
                Token::LBrace,
                Token::Ident,     // string
                Token::Ident,     // email
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn type_generic() {
        let tokens = lex_meaningful("type Paginated<T> { []T data = 1; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwType,
                Token::Ident,     // Paginated
                Token::LAngle,
                Token::Ident,     // T
                Token::RAngle,
                Token::LBrace,
                Token::Brackets,
                Token::Ident,     // T
                Token::Ident,     // data
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn pick_and_omit() {
        let tokens = lex_meaningful("type Sub = Pick<User, id, email>;");
        assert!(tokens.iter().any(|(k, _)| *k == Token::KwPick));

        let tokens = lex_meaningful("type Without = Omit<User, name>;");
        assert!(tokens.iter().any(|(k, _)| *k == Token::KwOmit));
    }

    // ── Shape ──────────────────────────────────────────────────────────

    #[test]
    fn shape_simple() {
        let tokens = lex_meaningful("shape Timestamps { uint64 created_at; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwShape,
                Token::Ident,     // Timestamps
                Token::LBrace,
                Token::Ident,     // uint64
                Token::Ident,     // created_at
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn shape_injection() {
        let tokens = lex_meaningful("MyModelMixIn(1..4)");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Ident,      // MyModelMixIn
                Token::LParen,
                Token::IntLiteral, // 1
                Token::DotDot,
                Token::IntLiteral, // 4
                Token::RParen,
            ]
        );
    }

    #[test]
    fn shape_composition() {
        let tokens = lex_meaningful("shape Combined { ShapeA; ShapeB, ShapeC; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwShape,
                Token::Ident,     // Combined
                Token::LBrace,
                Token::Ident,     // ShapeA
                Token::Semicolon,
                Token::Ident,     // ShapeB
                Token::Comma,
                Token::Ident,     // ShapeC
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    // ── Enum ───────────────────────────────────────────────────────────

    #[test]
    fn enum_decl() {
        let tokens = lex_meaningful("enum Status { Pending=1; Active=2; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwEnum,
                Token::Ident,     // Status
                Token::LBrace,
                Token::Ident,     // Pending
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::Ident,     // Active
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn enum_annotation() {
        let tokens = lex_meaningful("@deprecated(reason=Active) Deleted=3;");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::At,
                Token::Ident,     // deprecated
                Token::LParen,
                Token::Ident,     // reason
                Token::Eq,
                Token::Ident,     // Active
                Token::RParen,
                Token::Ident,     // Deleted
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
            ]
        );
    }

    // ── Oneof ──────────────────────────────────────────────────────────

    #[test]
    fn oneof_decl() {
        let tokens = lex_meaningful("oneof address { Address home = 10; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwOneof,
                Token::Ident,     // address
                Token::LBrace,
                Token::Ident,     // Address
                Token::Ident,     // home
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    // ── Service & RPC ──────────────────────────────────────────────────

    #[test]
    fn service_rpc() {
        let tokens = lex_meaningful("service UserAPI { rpc GetUser(void) -> User; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwService,
                Token::Ident,     // UserAPI
                Token::LBrace,
                Token::KwRpc,
                Token::Ident,     // GetUser
                Token::LParen,
                Token::KwVoid,
                Token::RParen,
                Token::Arrow,
                Token::Ident,     // User
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn rpc_streaming() {
        let tokens = lex_meaningful("rpc Sync(stream User) -> stream User;");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwRpc,
                Token::Ident,     // Sync
                Token::LParen,
                Token::KwStream,
                Token::Ident,     // User
                Token::RParen,
                Token::Arrow,
                Token::KwStream,
                Token::Ident,     // User
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn rpc_inline_type() {
        let tokens = lex_meaningful("rpc Create({ string name = 1; }) -> void;");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwRpc,
                Token::Ident,     // Create
                Token::LParen,
                Token::LBrace,
                Token::Ident,     // string
                Token::Ident,     // name
                Token::Eq,
                Token::IntLiteral,
                Token::Semicolon,
                Token::RBrace,
                Token::RParen,
                Token::Arrow,
                Token::KwVoid,
                Token::Semicolon,
            ]
        );
    }

    // ── Annotations ────────────────────────────────────────────────────

    #[test]
    fn annotation_call() {
        let tokens = lex_meaningful(r#"@database::Table(table_name="users")"#);
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::At,
                Token::Ident,        // database
                Token::ColonColon,
                Token::Ident,        // Table
                Token::LParen,
                Token::Ident,        // table_name
                Token::Eq,
                Token::StringLiteral,
                Token::RParen,
            ]
        );
    }

    #[test]
    fn annotation_definition() {
        let tokens = lex_meaningful("annotation Table for type { string table_name; }");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwAnnotation,
                Token::Ident,     // Table
                Token::KwFor,
                Token::KwType,
                Token::LBrace,
                Token::Ident,     // string
                Token::Ident,     // table_name
                Token::Semicolon,
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn annotation_multi_target() {
        let tokens = lex_meaningful("annotation A for type|field|enum { }");
        assert!(tokens.iter().any(|(k, _)| *k == Token::Pipe));
    }

    #[test]
    fn builtin_annotation_default() {
        let tokens = lex_meaningful("@default(now)");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::At,
                Token::Ident,   // "default" is now a regular identifier
                Token::LParen,
                Token::Ident,   // "now" is now a regular identifier
                Token::RParen,
            ]
        );
    }

    #[test]
    fn builtin_annotation_reserved() {
        let tokens = lex_meaningful("@reserved(17)");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::At,
                Token::KwReserved,
                Token::LParen,
                Token::IntLiteral,
                Token::RParen,
            ]
        );
    }

    // ── Container types ────────────────────────────────────────────────

    #[test]
    fn array_type() {
        let tokens = lex_meaningful("[]Order");
        assert_eq!(
            tokens,
            vec![(Token::Brackets, "[]"), (Token::Ident, "Order")]
        );
    }

    #[test]
    fn optional_type() {
        let tokens = lex_meaningful("string?");
        assert_eq!(
            tokens,
            vec![(Token::Ident, "string"), (Token::Question, "?")]
        );
    }

    #[test]
    fn map_type() {
        let tokens = lex_meaningful("map<string, string>");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::KwMap,
                Token::LAngle,
                Token::Ident,
                Token::Comma,
                Token::Ident,
                Token::RAngle,
            ]
        );
    }

    // ── Projections ────────────────────────────────────────────────────

    #[test]
    fn projection_mapping() {
        let tokens = lex_meaningful("uuid.UUID id = 1 <- User.id;");
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Ident,      // uuid
                Token::Dot,
                Token::Ident,      // UUID
                Token::Ident,      // id
                Token::Eq,
                Token::IntLiteral, // 1
                Token::BackArrow,
                Token::Ident,      // User
                Token::Dot,
                Token::Ident,      // id
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn projection_nested_field() {
        let tokens = lex_meaningful("string city = 3 <- Order.billing.city;");
        assert!(tokens.iter().any(|(k, _)| *k == Token::BackArrow));
        // Check we get three dot-separated idents after <-
        let after_arrow: Vec<(Token, &str)> = tokens
            .iter()
            .skip_while(|(k, _)| *k != Token::BackArrow)
            .skip(1)
            .cloned()
            .collect();
        let kinds: Vec<Token> = after_arrow.iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            vec![
                Token::Ident, // Order
                Token::Dot,
                Token::Ident, // billing
                Token::Dot,
                Token::Ident, // city
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn projection_oneof_wildcard() {
        let tokens = lex_meaningful("string tx = 2 <- Payment.method.*.transaction_id;");
        assert!(tokens.iter().any(|(k, _)| *k == Token::Star));
    }

    // ── Literals ───────────────────────────────────────────────────────

    #[test]
    fn string_literal() {
        let tokens = lex_meaningful(r#""hello world""#);
        assert_eq!(tokens, vec![(Token::StringLiteral, r#""hello world""#)]);
    }

    #[test]
    fn string_literal_with_escapes() {
        let tokens = lex_meaningful(r#""line\nbreak""#);
        assert_eq!(tokens, vec![(Token::StringLiteral, r#""line\nbreak""#)]);
    }

    #[test]
    fn integer_literal() {
        let tokens = lex_meaningful("42");
        assert_eq!(tokens, vec![(Token::IntLiteral, "42")]);
    }

    #[test]
    fn float_literal() {
        let tokens = lex_meaningful("3.14");
        assert_eq!(tokens, vec![(Token::FloatLiteral, "3.14")]);
    }

    #[test]
    fn bool_literals() {
        let tokens = lex_meaningful("true false");
        assert_eq!(
            tokens,
            vec![(Token::KwTrue, "true"), (Token::KwFalse, "false")]
        );
    }

    // ── Trivia ─────────────────────────────────────────────────────────

    #[test]
    fn whitespace_preserved() {
        let tokens = lex_ok("type  User");
        assert_eq!(
            tokens,
            vec![
                (Token::KwType, "type"),
                (Token::Whitespace, "  "),
                (Token::Ident, "User"),
            ]
        );
    }

    #[test]
    fn line_comment() {
        let tokens = lex_ok("// this is a comment\ntype");
        assert_eq!(
            tokens,
            vec![
                (Token::LineComment, "// this is a comment"),
                (Token::Whitespace, "\n"),
                (Token::KwType, "type"),
            ]
        );
    }

    #[test]
    fn block_comment() {
        let tokens = lex_ok("/* multi\nline */ type");
        assert_eq!(
            tokens,
            vec![
                (Token::BlockComment, "/* multi\nline */"),
                (Token::Whitespace, " "),
                (Token::KwType, "type"),
            ]
        );
    }

    #[test]
    fn is_trivia() {
        assert!(Token::Whitespace.is_trivia());
        assert!(Token::LineComment.is_trivia());
        assert!(Token::BlockComment.is_trivia());
        assert!(!Token::KwType.is_trivia());
        assert!(!Token::Ident.is_trivia());
    }

    // ── Full snippet ───────────────────────────────────────────────────

    #[test]
    fn full_type_definition() {
        let source = r#"
package example;

import uuid;

type User {
    uuid.UUID id = 1;
    string email = 2;
    []Order orders = 3;
    map<string, string> metadata = 4;
    string? nickname = 5;
    @reserved(6)
}
"#;
        let results = lex(source);
        // No errors
        assert!(results.iter().all(|r| r.is_ok()));
        let tokens = lex_meaningful(source);
        // Check key tokens are present
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert!(kinds.contains(&Token::KwPackage));
        assert!(kinds.contains(&Token::KwImport));
        assert!(kinds.contains(&Token::KwType));
        assert!(kinds.contains(&Token::Brackets));   // []
        assert!(kinds.contains(&Token::KwMap));
        assert!(kinds.contains(&Token::Question));    // ?
        assert!(kinds.contains(&Token::At));
        assert!(kinds.contains(&Token::KwReserved));
    }

    #[test]
    fn full_service_definition() {
        let source = r#"
service UserAPI {
    rpc GetUser(uuid.UUID) -> User;
    rpc CreateUser({ string name = 1; string email = 2; }) -> User;
    rpc ListUsers(void) -> stream User;
    rpc SyncUsers(stream User) -> stream User;
}
"#;
        let results = lex(source);
        assert!(results.iter().all(|r| r.is_ok()));
        let tokens = lex_meaningful(source);
        let kinds: Vec<Token> = tokens.iter().map(|(k, _)| *k).collect();
        assert!(kinds.contains(&Token::KwService));
        assert!(kinds.contains(&Token::KwRpc));
        assert!(kinds.contains(&Token::KwVoid));
        assert!(kinds.contains(&Token::KwStream));
        assert!(kinds.contains(&Token::Arrow));
    }

    #[test]
    fn annotation_with_nested_values() {
        let source = r#"@library::Example(
    metadata={key1:"value1", key2:"value2"},
    example_struct={
        inner_field="hello"
    }
)"#;
        let results = lex(source);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn projection_full() {
        let source = r#"
type UserDashboard {
    uuid.UUID id = 1 <- User.id;
    string name = 2 <- User.first_name;
    int64 total_orders = 3 <- UserStats.total_orders;
    string theme = 4 <- UserSettings.theme;
}
"#;
        let results = lex(source);
        assert!(results.iter().all(|r| r.is_ok()));
        let tokens = lex_meaningful(source);
        let arrow_count = tokens.iter().filter(|(k, _)| *k == Token::BackArrow).count();
        assert_eq!(arrow_count, 4);
    }
}

//! Unified token + node kinds for the Ogham CST.
//!
//! Every value in [`SyntaxKind`] is either a *token* produced by the lexer
//! or a *node* assembled by the parser. The enum is `#[repr(u16)]` so that
//! rowan can round-trip it through `rowan::SyntaxKind(u16)`.

use crate::lexer::Token;

/// All token and node kinds in the Ogham CST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // ── Tokens (must match lexer::Token ordering) ──────────────────────
    // Keywords
    KwPackage = 0,
    KwImport,
    KwAs,
    KwType,
    KwShape,
    KwEnum,
    KwOneof,
    KwService,
    KwRpc,
    KwAnnotation,
    KwFor,
    KwVoid,
    KwStream,
    KwMap,
    KwPick,
    KwOmit,
    KwTrue,
    KwFalse,
    KwSelf,
    // Built-in annotation names
    KwReserved,
    // Punctuation
    LBrace,
    RBrace,
    LParen,
    RParen,
    LAngle,
    RAngle,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Dot,
    Eq,
    Colon,
    ColonColon,
    At,
    Question,
    Pipe,
    Arrow,
    BackArrow,
    DotDot,
    Star,
    Brackets,
    Slash,
    Minus,
    // Literals
    StringLiteral,
    FloatLiteral,
    IntLiteral,
    // Identifiers
    Ident,
    // Trivia
    Whitespace,
    LineComment,
    BlockComment,

    // ── Error token (lexer) ────────────────────────────────────────────
    LexError,

    // ── Composite nodes (parser) ───────────────────────────────────────
    Root,
    PackageDecl,
    ImportDecl,
    ImportPath,
    TypeDecl,
    TypeAlias,
    TypeBody,
    TypeParams,
    FieldDecl,
    MappingSource,
    ShapeDecl,
    ShapeField,
    ShapeInclude,
    ShapeInjection,
    EnumDecl,
    EnumValueDecl,
    OneofDecl,
    OneofField,
    ServiceDecl,
    RpcDecl,
    RpcParam,
    InlineType,
    AnnotationDecl,
    AnnotationTargets,
    AnnotationTargetConstraint,
    AnnotationField,
    AnnotationComposition,
    AnnotationCall,
    AnnotationArgs,
    AnnotationArg,
    AnnotationValue,
    AnnotationValueFields,
    TypeRef,
    ArrayType,
    MapType,
    OptionalMarker,
    PickType,
    OmitType,
    QualifiedName,
    IdentList,
    NestedTypeDecl,
    NestedEnumDecl,
    ReservedDecl,
    InlineAnnotationType,
    Error,
}

impl SyntaxKind {
    /// Number of variants. Keep in sync when adding variants.
    pub const LAST: u16 = SyntaxKind::Error as u16;
}

impl From<Token> for SyntaxKind {
    fn from(token: Token) -> Self {
        match token {
            Token::KwPackage => Self::KwPackage,
            Token::KwImport => Self::KwImport,
            Token::KwAs => Self::KwAs,
            Token::KwType => Self::KwType,
            Token::KwShape => Self::KwShape,
            Token::KwEnum => Self::KwEnum,
            Token::KwOneof => Self::KwOneof,
            Token::KwService => Self::KwService,
            Token::KwRpc => Self::KwRpc,
            Token::KwAnnotation => Self::KwAnnotation,
            Token::KwFor => Self::KwFor,
            Token::KwVoid => Self::KwVoid,
            Token::KwStream => Self::KwStream,
            Token::KwMap => Self::KwMap,
            Token::KwPick => Self::KwPick,
            Token::KwOmit => Self::KwOmit,
            Token::KwTrue => Self::KwTrue,
            Token::KwFalse => Self::KwFalse,
            Token::KwSelf => Self::KwSelf,
            Token::KwReserved => Self::KwReserved,
            Token::LBrace => Self::LBrace,
            Token::RBrace => Self::RBrace,
            Token::LParen => Self::LParen,
            Token::RParen => Self::RParen,
            Token::LAngle => Self::LAngle,
            Token::RAngle => Self::RAngle,
            Token::LBracket => Self::LBracket,
            Token::RBracket => Self::RBracket,
            Token::Semicolon => Self::Semicolon,
            Token::Comma => Self::Comma,
            Token::Dot => Self::Dot,
            Token::Eq => Self::Eq,
            Token::Colon => Self::Colon,
            Token::ColonColon => Self::ColonColon,
            Token::At => Self::At,
            Token::Question => Self::Question,
            Token::Pipe => Self::Pipe,
            Token::Arrow => Self::Arrow,
            Token::BackArrow => Self::BackArrow,
            Token::DotDot => Self::DotDot,
            Token::Star => Self::Star,
            Token::Brackets => Self::Brackets,
            Token::Slash => Self::Slash,
            Token::Minus => Self::Minus,
            Token::StringLiteral => Self::StringLiteral,
            Token::FloatLiteral => Self::FloatLiteral,
            Token::IntLiteral => Self::IntLiteral,
            Token::Ident => Self::Ident,
            Token::Whitespace => Self::Whitespace,
            Token::LineComment => Self::LineComment,
            Token::BlockComment => Self::BlockComment,
        }
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// Language marker for the Ogham CST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OghamLang {}

impl rowan::Language for OghamLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::LAST);
        // SAFETY: SyntaxKind is #[repr(u16)] and raw.0 is within range.
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

/// Convenience type aliases parameterized by [`OghamLang`].
pub type SyntaxNode = rowan::SyntaxNode<OghamLang>;
pub type SyntaxToken = rowan::SyntaxToken<OghamLang>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

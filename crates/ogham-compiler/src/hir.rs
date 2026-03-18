//! High-level IR: arena-allocated, ID-based intermediate representation.
//!
//! During resolution passes (3–9) the compiler works with arena indices
//! (`TypeId`, `EnumId`, etc.) — not inline copies. The final inflation
//! pass (12) converts this into the fully-inline proto IR for plugins.

use la_arena::{Arena, Idx};
use string_interner::backend::StringBackend;
use string_interner::symbol::SymbolU32;

/// Interned string symbol.
pub type Sym = SymbolU32;
use std::collections::HashMap;

// ── String interning ───────────────────────────────────────────────────

/// Shared string interner for all names in the compiler.
#[derive(Debug, Default)]
pub struct Interner {
    pub inner: string_interner::StringInterner<StringBackend>,
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> Sym {
        self.inner.get_or_intern(s)
    }

    pub fn resolve(&self, sym: Sym) -> &str {
        self.inner.resolve(sym).expect("dangling symbol")
    }

    /// Look up a string without interning it. Returns `None` if not found.
    pub fn intern_lookup(&self, s: &str) -> Option<Sym> {
        self.inner.get(s)
    }
}

// ── Arena IDs ──────────────────────────────────────────────────────────

pub type TypeId = Idx<TypeDef>;
pub type EnumId = Idx<EnumDef>;
pub type ServiceId = Idx<ServiceDef>;
pub type ShapeId = Idx<ShapeDef>;
pub type AnnotationDefId = Idx<AnnotationDef>;

// ── Arenas ─────────────────────────────────────────────────────────────

/// Central storage for all declarations.
#[derive(Debug, Default)]
pub struct Arenas {
    pub types: Arena<TypeDef>,
    pub enums: Arena<EnumDef>,
    pub services: Arena<ServiceDef>,
    pub shapes: Arena<ShapeDef>,
    pub annotation_defs: Arena<AnnotationDef>,
}

// ── Symbol table ───────────────────────────────────────────────────────

/// Maps fully-qualified names to declaration IDs.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub types: HashMap<Sym, TypeId>,
    pub enums: HashMap<Sym, EnumId>,
    pub services: HashMap<Sym, ServiceId>,
    pub shapes: HashMap<Sym, ShapeId>,
    pub annotations: HashMap<(Sym, Sym), Vec<AnnotationDefId>>, // (library, name) → overloads
    /// Per-file import maps: file → (short_name → full_name)
    pub imports: HashMap<Sym, HashMap<Sym, Sym>>,
}

// ── Source location ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Loc {
    pub file: Option<Sym>,
    pub span: std::ops::Range<usize>,
}

impl Default for Loc {
    fn default() -> Self {
        Self { file: None, span: 0..0 }
    }
}

// ── Type definitions ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: Sym,
    pub full_name: Sym,
    pub fields: Vec<FieldDef>,
    pub oneofs: Vec<OneofDef>,
    pub nested_types: Vec<TypeId>,
    pub nested_enums: Vec<EnumId>,
    pub annotations: Vec<AnnotationCall>,
    pub back_references: Vec<BackRef>,
    pub trace: Option<TypeTrace>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Sym,
    pub number: u32,
    pub ty: ResolvedType,
    pub is_optional: bool,
    pub is_repeated: bool,
    pub annotations: Vec<AnnotationCall>,
    pub mapping: Option<FieldMapping>,
    pub trace: Option<FieldTrace>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct OneofDef {
    pub name: Sym,
    pub fields: Vec<OneofFieldDef>,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct OneofFieldDef {
    pub name: Sym,
    pub number: u32,
    pub ty: ResolvedType,
    pub annotations: Vec<AnnotationCall>,
    pub mapping: Option<FieldMapping>,
    pub loc: Loc,
}

// ── Enum definitions ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: Sym,
    pub full_name: Sym,
    pub values: Vec<EnumValueDef>,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct EnumValueDef {
    pub name: Sym,
    pub number: i32,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

// ── Service definitions ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ServiceDef {
    pub name: Sym,
    pub full_name: Sym,
    pub rpcs: Vec<RpcDef>,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct RpcDef {
    pub name: Sym,
    pub input: RpcParamDef,
    pub output: RpcParamDef,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct RpcParamDef {
    pub is_void: bool,
    pub is_stream: bool,
    pub ty: ResolvedType,
}

// ── Shape definitions ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShapeDef {
    pub name: Sym,
    pub full_name: Sym,
    pub fields: Vec<ShapeFieldDef>,
    pub includes: Vec<Sym>, // shape names to include
    pub type_params: Vec<Sym>,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct ShapeFieldDef {
    pub name: Sym,
    pub ty: ResolvedType,
    pub annotations: Vec<AnnotationCall>,
    pub loc: Loc,
}

// ── Annotation type constraints ────────────────────────────────────────

/// Type constraint on an annotation target: `for field(string | int32)`.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// No constraint or `any` — matches everything.
    Any,
    /// Scalar type: `string`, `int32`, etc.
    Scalar(ScalarKind),
    /// Any message type: `message`.
    Message,
    /// Any enum type: `enum`.
    Enum,
    /// Specific named type: `time.Timestamp`.
    Named(Sym),
    /// Union: `int32 | int64 | uint32`.
    Union(Vec<TypeConstraint>),
    /// Array: `[]string`, `[]any`, `[]message`.
    Array(Box<TypeConstraint>),
    /// Map: `map<string, any>`.
    Map {
        key: Box<TypeConstraint>,
        value: Box<TypeConstraint>,
    },
}

// ── Annotation definitions ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnnotationDef {
    pub library: Sym,
    pub name: Sym,
    pub full_name: Sym,
    pub targets: Vec<AnnotationTarget>,
    pub params: Vec<AnnotationParamDef>,
    pub compositions: Vec<AnnotationCompositionRef>,
    pub loc: Loc,
}

/// A single annotation target with optional type constraint.
#[derive(Debug, Clone)]
pub struct AnnotationTarget {
    pub kind: Sym,                             // "field", "type", "enum", etc.
    pub type_constraint: Option<TypeConstraint>, // None = any
}

#[derive(Debug, Clone)]
pub struct AnnotationParamDef {
    pub name: Sym,
    pub ty: ResolvedType,
    pub is_optional: bool,
    pub default_value: Option<LiteralValue>,
}

#[derive(Debug, Clone)]
pub struct AnnotationCompositionRef {
    pub library: Sym,
    pub name: Sym,
    pub arguments: Vec<AnnotationArgDef>,
}

// ── Annotation calls ───────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnnotationCall {
    pub library: Sym,
    pub name: Sym,
    pub arguments: Vec<AnnotationArgDef>,
    pub definition: Option<AnnotationDefId>,
    pub loc: Loc,
}

#[derive(Debug, Clone)]
pub struct AnnotationArgDef {
    pub name: Sym,
    pub value: LiteralValue,
}

// ── Resolved type references ───────────────────────────────────────────

/// A type reference during resolution. Uses arena IDs, not inline copies.
#[derive(Debug, Clone)]
pub enum ResolvedType {
    /// Not yet resolved (placeholder during collection).
    Unresolved(Sym),
    /// Resolution failed — error already reported.
    Error,
    /// Primitive scalar.
    Scalar(ScalarKind),
    /// Reference to a message type by arena ID.
    Message(TypeId),
    /// Reference to an enum by arena ID.
    Enum(EnumId),
    /// Map type.
    Map {
        key: Box<ResolvedType>,
        value: Box<ResolvedType>,
    },
    /// Array (repeated).
    Array(Box<ResolvedType>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarKind {
    Bool,
    String,
    Bytes,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float,
    Double,
}

// ── Literals ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum LiteralValue {
    String(Sym),
    Int(i64),
    Float(f64),
    Bool(bool),
    Ident(Sym),
    Struct(Vec<(Sym, LiteralValue)>),
    List(Vec<LiteralValue>),
}

// ── Projection mappings ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FieldMapping {
    pub chain: Vec<MappingLink>,
}

#[derive(Debug, Clone)]
pub struct MappingLink {
    pub source_type: TypeId,
    pub source_field_name: Sym,
    pub path: Vec<Sym>, // for nested: ["billing", "street"]
}

// ── Traces ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TypeTrace {
    Generic {
        source_name: Sym,
        type_arguments: Vec<Sym>,
    },
    PickOmit {
        kind: Sym, // "Pick" or "Omit"
        source_type: TypeId,
        field_names: Vec<Sym>,
    },
}

#[derive(Debug, Clone)]
pub struct FieldTrace {
    pub shape: Option<ShapeOrigin>,
}

#[derive(Debug, Clone)]
pub struct ShapeOrigin {
    pub shape_name: Sym,
    pub shape_id: ShapeId,
    pub range_start: u32,
    pub range_end: u32,
}

// ── Back-references ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BackRef {
    pub referencing_type: TypeId,
    pub field_name: Sym,
}

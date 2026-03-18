//! Pass 2: Index collection.
//!
//! Walks the typed AST and registers all top-level declarations
//! (types, enums, shapes, services, annotation defs) into arenas
//! and the symbol table. No references are resolved here — that
//! happens in subsequent passes.

use crate::ast::{self, AstNode};
use crate::diagnostics::Diagnostics;
use crate::hir::{self, Arenas, Interner, Loc, Sym, SymbolTable};
use crate::syntax_kind::SyntaxNode;

/// A parsed file ready for indexing.
pub struct ParsedFile {
    pub file_name: String,
    pub root: SyntaxNode,
    pub package: String,
    /// Full import path, e.g. "github.com/oghamlang/std/time" or "github.com/org/proj/common".
    /// Falls back to `package` when no module_path is available.
    pub import_path: String,
}

fn make_loc(file: Sym, node: &SyntaxNode) -> Loc {
    let range = node.text_range();
    Loc {
        file: Some(file),
        span: usize::from(range.start())..usize::from(range.end()),
    }
}

/// Index a single type declaration, recursively handling nested types/enums.
/// Returns the TypeId of the indexed type.
fn index_type_decl(
    type_decl: &ast::TypeDecl,
    parent_prefix: &str,
    file_sym: Sym,
    interner: &mut hir::Interner,
    arenas: &mut hir::Arenas,
    symbols: &mut hir::SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
) -> Option<hir::TypeId> {
    let name_text = type_decl.name()?.text().to_string();
    let full = format!("{}.{}", parent_prefix, name_text);
    let name_sym = interner.intern(&name_text);
    let full_sym = interner.intern(&full);

    // Index nested types and enums first
    let mut nested_type_ids = Vec::new();
    let mut nested_enum_ids = Vec::new();

    if let Some(body) = type_decl.body() {
        for nested in body.nested_types() {
            if let Some(inner_decl) = nested.type_decl() {
                if let Some(nested_id) = index_type_decl(
                    &inner_decl,
                    &full, // nested prefix: "pkg.Parent"
                    file_sym,
                    interner,
                    arenas,
                    symbols,
                    diag,
                    file_name,
                ) {
                    nested_type_ids.push(nested_id);
                }
            }
        }

        for nested in body.nested_enums() {
            if let Some(inner_decl) = nested.enum_decl() {
                if let Some(nested_id) = index_enum_decl(
                    &inner_decl,
                    &full,
                    file_sym,
                    interner,
                    arenas,
                    symbols,
                    diag,
                    file_name,
                ) {
                    nested_enum_ids.push(nested_id);
                }
            }
        }
    }

    let type_def = hir::TypeDef {
        name: name_sym,
        full_name: full_sym,
        fields: Vec::new(),
        oneofs: Vec::new(),
        nested_types: nested_type_ids,
        nested_enums: nested_enum_ids,
        annotations: Vec::new(),
        back_references: Vec::new(),
        trace: None,
        loc: Loc {
            file: Some(file_sym),
            span: {
                let r = type_decl.syntax().text_range();
                usize::from(r.start())..usize::from(r.end())
            },
        },
    };

    let id = arenas.types.alloc(type_def);
    if symbols.types.insert(full_sym, id).is_some() {
        let range = type_decl.syntax().text_range();
        diag.error(
            file_name,
            usize::from(range.start())..usize::from(range.end()),
            format!("duplicate type: {}", full),
        );
    }

    Some(id)
}

/// Index a single enum declaration.
fn index_enum_decl(
    enum_decl: &ast::EnumDecl,
    parent_prefix: &str,
    file_sym: Sym,
    interner: &mut hir::Interner,
    arenas: &mut hir::Arenas,
    symbols: &mut hir::SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
) -> Option<hir::EnumId> {
    let name_text = enum_decl.name()?.text().to_string();
    let full = format!("{}.{}", parent_prefix, name_text);
    let name_sym = interner.intern(&name_text);
    let full_sym = interner.intern(&full);

    let loc = Loc {
        file: Some(file_sym),
        span: {
            let r = enum_decl.syntax().text_range();
            usize::from(r.start())..usize::from(r.end())
        },
    };

    let mut values = vec![hir::EnumValueDef {
        name: interner.intern("Unspecified"),
        number: 0,
        annotations: Vec::new(),
        loc: loc.clone(),
    }];

    for val in enum_decl.values() {
        let val_name = match val.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let val_number = val.value().unwrap_or(0) as i32;

        values.push(hir::EnumValueDef {
            name: interner.intern(&val_name),
            number: val_number,
            annotations: Vec::new(),
            loc: Loc {
                file: Some(file_sym),
                span: {
                    let r = val.syntax().text_range();
                    usize::from(r.start())..usize::from(r.end())
                },
            },
        });
    }

    let enum_def = hir::EnumDef {
        name: name_sym,
        full_name: full_sym,
        values,
        annotations: Vec::new(),
        loc,
    };

    let id = arenas.enums.alloc(enum_def);
    if symbols.enums.insert(full_sym, id).is_some() {
        let range = enum_decl.syntax().text_range();
        diag.error(
            file_name,
            usize::from(range.start())..usize::from(range.end()),
            format!("duplicate enum: {}", full),
        );
    }

    Some(id)
}

/// Collect all declarations from a parsed file into arenas and symbol table.
pub fn collect(
    file: &ParsedFile,
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &mut SymbolTable,
    diag: &mut Diagnostics,
) {
    let root = match ast::Root::cast(file.root.clone()) {
        Some(r) => r,
        None => return,
    };

    let file_sym = interner.intern(&file.file_name);
    let pkg = &file.package;
    let ip = &file.import_path;

    // Index types (including nested)
    for type_decl in root.type_decls() {
        index_type_decl(
            &type_decl, ip, file_sym, interner, arenas, symbols, diag, &file.file_name,
        );
    }

    // Index enums (including nested)
    for enum_decl in root.enum_decls() {
        index_enum_decl(
            &enum_decl, ip, file_sym, interner, arenas, symbols, diag, &file.file_name,
        );
    }

    // Index shapes
    for shape_decl in root.shape_decls() {
        let name_text = match shape_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", ip, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let type_params: Vec<Sym> = shape_decl
            .type_params()
            .map(|tp| {
                tp.params()
                    .iter()
                    .map(|t| interner.intern(t.text()))
                    .collect()
            })
            .unwrap_or_default();

        let includes: Vec<Sym> = shape_decl
            .includes()
            .iter()
            .flat_map(|inc| {
                inc.names()
                    .iter()
                    .map(|t| interner.intern(t.text()))
                    .collect::<Vec<_>>()
            })
            .collect();

        let shape_def = hir::ShapeDef {
            name: name_sym,
            full_name: full_sym,
            fields: Vec::new(),
            includes,
            type_params,
            annotations: Vec::new(),
            loc: make_loc(file_sym, shape_decl.syntax()),
        };

        let id = arenas.shapes.alloc(shape_def);
        if symbols.shapes.insert(full_sym, id).is_some() {
            let range = shape_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate shape: {}", full),
            );
        }
    }

    // Index services
    for svc_decl in root.service_decls() {
        let name_text = match svc_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", ip, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let svc_def = hir::ServiceDef {
            name: name_sym,
            full_name: full_sym,
            rpcs: Vec::new(),
            annotations: Vec::new(),
            loc: make_loc(file_sym, svc_decl.syntax()),
        };

        let id = arenas.services.alloc(svc_def);
        if symbols.services.insert(full_sym, id).is_some() {
            let range = svc_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate service: {}", full),
            );
        }
    }

    // Index annotation definitions
    for ann_decl in root.annotation_decls() {
        let name_text = match ann_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let lib_sym = interner.intern(pkg);
        let name_sym = interner.intern(&name_text);
        let full = format!("{}::{}", pkg, name_text);
        let full_sym = interner.intern(&full);

        let targets: Vec<hir::AnnotationTarget> = ann_decl
            .targets()
            .map(|t| {
                t.target_pairs()
                    .into_iter()
                    .map(|(kind, constraint_text)| {
                        let kind_sym = interner.intern(&kind);
                        let type_constraint = constraint_text
                            .map(|text| parse_type_constraint(&text, interner))
                            .unwrap_or(None);
                        hir::AnnotationTarget {
                            kind: kind_sym,
                            type_constraint,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let ann_def = hir::AnnotationDef {
            library: lib_sym,
            name: name_sym,
            full_name: full_sym,
            targets,
            params: Vec::new(),
            compositions: Vec::new(),
            loc: make_loc(file_sym, ann_decl.syntax()),
        };

        let id = arenas.annotation_defs.alloc(ann_def);
        symbols.annotations
            .entry((lib_sym, name_sym))
            .or_default()
            .push(id);
    }
}

/// Parse a type constraint string like "string | int32", "[]any", "map<string, any>".
fn parse_type_constraint(text: &str, interner: &mut hir::Interner) -> Option<hir::TypeConstraint> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Union: split by top-level '|' (not inside <> or [])
    let parts = split_top_level(text, '|');
    if parts.len() > 1 {
        let constraints: Vec<hir::TypeConstraint> = parts
            .iter()
            .filter_map(|p| parse_type_constraint(p, interner))
            .collect();
        return if constraints.is_empty() { None } else { Some(hir::TypeConstraint::Union(constraints)) };
    }

    // Array: []T
    if text.starts_with("[]") {
        let inner = &text[2..];
        let inner_c = parse_type_constraint(inner, interner)
            .unwrap_or(hir::TypeConstraint::Any);
        return Some(hir::TypeConstraint::Array(Box::new(inner_c)));
    }

    // Map: map<K, V>
    if text.starts_with("map<") && text.ends_with('>') {
        let inner = &text[4..text.len() - 1];
        let kv = split_top_level(inner, ',');
        if kv.len() == 2 {
            let key = parse_type_constraint(kv[0].trim(), interner)
                .unwrap_or(hir::TypeConstraint::Any);
            let value = parse_type_constraint(kv[1].trim(), interner)
                .unwrap_or(hir::TypeConstraint::Any);
            return Some(hir::TypeConstraint::Map {
                key: Box::new(key),
                value: Box::new(value),
            });
        }
    }

    // Keywords
    match text {
        "any" => Some(hir::TypeConstraint::Any),
        "message" => Some(hir::TypeConstraint::Message),
        "enum" => Some(hir::TypeConstraint::Enum),
        // Scalars
        "bool" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Bool)),
        "string" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::String)),
        "bytes" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Bytes)),
        "i8" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Int8)),
        "int16" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Int16)),
        "int32" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Int32)),
        "int64" | "int" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Int64)),
        "uint8" | "byte" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Uint8)),
        "uint16" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Uint16)),
        "uint32" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Uint32)),
        "uint64" | "uint" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Uint64)),
        "float" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Float)),
        "double" => Some(hir::TypeConstraint::Scalar(hir::ScalarKind::Double)),
        // Named type: time.Timestamp, etc.
        _ => Some(hir::TypeConstraint::Named(interner.intern(text))),
    }
}

/// Split a string by a delimiter, respecting `<>` and `[]` nesting.
fn split_top_level(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' | '[' => depth += 1,
            '>' | ']' => depth = depth.saturating_sub(1),
            c if c == delim && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn index_source(source: &str) -> (Interner, Arenas, SymbolTable, Diagnostics) {
        let parse = parser::parse(source);
        assert!(parse.errors.is_empty(), "parse errors: {:?}", parse.errors);

        let root = parse.syntax();
        let pkg = {
            let r = ast::Root::cast(root.clone()).unwrap();
            r.package_decl()
                .and_then(|p| p.name().map(|t| t.text().to_string()))
                .unwrap_or_else(|| "default".to_string())
        };

        let file = ParsedFile {
            file_name: "test.ogham".to_string(),
            root,
            package: pkg.clone(),
            import_path: pkg,
        };

        let mut interner = Interner::default();
        let mut arenas = Arenas::default();
        let mut symbols = SymbolTable::default();
        let mut diag = Diagnostics::new();

        collect(&file, &mut interner, &mut arenas, &mut symbols, &mut diag);

        (interner, arenas, symbols, diag)
    }

    #[test]
    fn index_type() {
        let (int, arenas, symbols, diag) =
            index_source("package example;\ntype User { string email = 1; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.User").unwrap();
        assert!(symbols.types.contains_key(&key));
        assert_eq!(arenas.types.len(), 1);
    }

    #[test]
    fn index_enum() {
        let (int, arenas, symbols, diag) =
            index_source("package example;\nenum Status { Active = 1; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.Status").unwrap();
        assert!(symbols.enums.contains_key(&key));
        // Implicit Unspecified=0 + Active=1
        let id = symbols.enums[&key];
        assert_eq!(arenas.enums[id].values.len(), 2);
    }

    #[test]
    fn index_shape() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nshape Timestamps { uint64 created_at; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.Timestamps").unwrap();
        assert!(symbols.shapes.contains_key(&key));
    }

    #[test]
    fn index_service() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nservice UserAPI { rpc Get(void) -> void; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.UserAPI").unwrap();
        assert!(symbols.services.contains_key(&key));
    }

    #[test]
    fn index_annotation_def() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nannotation Table for type { string table_name; }");
        assert!(!diag.has_errors());
        let lib = int.inner.get("example").unwrap();
        let name = int.inner.get("Table").unwrap();
        assert!(!symbols.annotations[&(lib, name)].is_empty());
    }

    #[test]
    fn duplicate_type_error() {
        let (_, _, _, diag) = index_source(
            "package example;\ntype User { string a = 1; }\ntype User { string b = 1; }",
        );
        assert!(diag.has_errors());
    }

    #[test]
    fn multiple_declarations() {
        let (_, arenas, _, diag) = index_source(
            r#"package example;
type User { string email = 1; }
type Order { string id = 1; }
enum Status { Active = 1; }
shape Timestamps { uint64 created_at; }
service UserAPI { rpc Get(void) -> void; }
annotation Table for type { string name; }
"#,
        );
        assert!(!diag.has_errors());
        assert_eq!(arenas.types.len(), 2);
        assert_eq!(arenas.enums.len(), 1);
        assert_eq!(arenas.shapes.len(), 1);
        assert_eq!(arenas.services.len(), 1);
        assert_eq!(arenas.annotation_defs.len(), 1);
    }

    #[test]
    fn annotation_type_constraint_scalar() {
        let (int, arenas, symbols, diag) = index_source(
            "package v;\nannotation Length for field(string | bytes) { uint32? max; }",
        );
        assert!(!diag.has_errors());
        let lib = int.inner.get("v").unwrap();
        let name = int.inner.get("Length").unwrap();
        let id = symbols.annotations[&(lib, name)][0];
        let def = &arenas.annotation_defs[id];
        assert_eq!(def.targets.len(), 1);
        let target = &def.targets[0];
        assert_eq!(int.resolve(target.kind), "field");
        match &target.type_constraint {
            Some(hir::TypeConstraint::Union(parts)) => {
                assert_eq!(parts.len(), 2);
                assert_eq!(parts[0], hir::TypeConstraint::Scalar(hir::ScalarKind::String));
                assert_eq!(parts[1], hir::TypeConstraint::Scalar(hir::ScalarKind::Bytes));
            }
            other => panic!("expected Union, got {:?}", other),
        }
    }

    #[test]
    fn annotation_type_constraint_array() {
        let (int, arenas, symbols, diag) = index_source(
            "package v;\nannotation Items for field([]any) { uint32? min; }",
        );
        assert!(!diag.has_errors());
        let lib = int.inner.get("v").unwrap();
        let name = int.inner.get("Items").unwrap();
        let id = symbols.annotations[&(lib, name)][0];
        let target = &arenas.annotation_defs[id].targets[0];
        match &target.type_constraint {
            Some(hir::TypeConstraint::Array(inner)) => {
                assert_eq!(**inner, hir::TypeConstraint::Any);
            }
            other => panic!("expected Array(Any), got {:?}", other),
        }
    }

    #[test]
    fn annotation_type_constraint_map() {
        let (int, arenas, symbols, diag) = index_source(
            "package v;\nannotation Entries for field(map<string, any>) { uint32? min; }",
        );
        assert!(!diag.has_errors());
        let lib = int.inner.get("v").unwrap();
        let name = int.inner.get("Entries").unwrap();
        let id = symbols.annotations[&(lib, name)][0];
        let target = &arenas.annotation_defs[id].targets[0];
        match &target.type_constraint {
            Some(hir::TypeConstraint::Map { key, value }) => {
                assert_eq!(**key, hir::TypeConstraint::Scalar(hir::ScalarKind::String));
                assert_eq!(**value, hir::TypeConstraint::Any);
            }
            other => panic!("expected Map, got {:?}", other),
        }
    }

    #[test]
    fn annotation_no_constraint_is_none() {
        let (_, arenas, symbols, diag) = index_source(
            "package v;\nannotation Required for field { }",
        );
        assert!(!diag.has_errors());
        let ids = symbols.annotations.values().next().unwrap();
        let target = &arenas.annotation_defs[ids[0]].targets[0];
        assert!(target.type_constraint.is_none());
    }
}

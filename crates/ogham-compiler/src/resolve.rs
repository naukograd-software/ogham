//! Resolve passes: type resolution, shape expansion, etc.
//!
//! Takes populated HIR with `ResolvedType::Unresolved` references
//! and resolves them to concrete types via the symbol table.

use std::collections::{HashMap, HashSet};

use crate::ast::{self, AstNode};
use crate::diagnostics::Diagnostics;
use crate::hir::*;
use crate::index::ParsedFile;
/// Extract the short package name from a full_name like "github.com/org/proj/common.Address" → "common".
/// For old-format "common.Address" → "common".
fn split_full_name_pkg_short(full: &str) -> &str {
    if let Some(slash_pos) = full.rfind('/') {
        let after_slash = &full[slash_pos + 1..];
        after_slash.split('.').next().unwrap_or(after_slash)
    } else {
        full.split('.').next().unwrap_or(full)
    }
}

// ── AST annotation → HIR annotation conversion ────────────────────────

fn collect_annotation_calls(
    annotations: &[ast::AnnotationCall],
    interner: &mut Interner,
) -> Vec<AnnotationCall> {
    annotations
        .iter()
        .map(|ann| {
            let (lib, name) = ann.library_name().unwrap_or_default();
            let lib_sym = interner.intern(&lib);
            let name_sym = interner.intern(&name);

            let arguments = ann
                .args()
                .map(|args| collect_annotation_args(&args, interner))
                .unwrap_or_default();

            AnnotationCall {
                library: lib_sym,
                name: name_sym,
                arguments,
                definition: None,
                loc: Loc::default(),
            }
        })
        .collect()
}

fn collect_annotation_args(
    args: &ast::AnnotationArgs,
    interner: &mut Interner,
) -> Vec<AnnotationArgDef> {
    args.args()
        .iter()
        .map(|arg| {
            let name_text = arg
                .name()
                .map(|t| t.text().to_string())
                .unwrap_or_default();
            let name_sym = interner.intern(&name_text);

            let value = arg
                .value()
                .map(|v| parse_annotation_value(&v, interner))
                .unwrap_or(LiteralValue::Bool(false));

            AnnotationArgDef {
                name: name_sym,
                value,
            }
        })
        .collect()
}

fn parse_annotation_value(value: &ast::AnnotationValue, interner: &mut Interner) -> LiteralValue {
    // Extract the text content of the value node
    let text = value.syntax().text().to_string();
    let text = text.trim();

    // Try parsing as different literal types
    if let Some(stripped) = text.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return LiteralValue::String(interner.intern(stripped));
    }
    if text == "true" {
        return LiteralValue::Bool(true);
    }
    if text == "false" {
        return LiteralValue::Bool(false);
    }
    if let Ok(i) = text.parse::<i64>() {
        return LiteralValue::Int(i);
    }
    if let Ok(f) = text.parse::<f64>() {
        return LiteralValue::Float(f);
    }
    // Fallback: treat as identifier
    LiteralValue::Ident(interner.intern(text))
}

// ── Pass 3+4: Populate fields + resolve type references ────────────────

/// Populate type fields, shape fields, oneofs, rpcs from AST and resolve
/// all type references in one pass.
pub fn populate_and_resolve(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };

        let file_sym = interner.intern(&file.file_name);
        let pkg = &file.package;
        let ip = &file.import_path;

        // Collect imports for this file
        let imports = collect_imports(&root, interner, pkg);

        // Populate type fields
        for type_decl in root.type_decls() {
            let name_text = match type_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let full = format!("{}.{}", ip, name_text);
            let full_sym = interner.intern(&full);

            let type_id = match symbols.types.get(&full_sym) {
                Some(id) => *id,
                None => continue,
            };

            // Collect type-level annotations
            arenas.types[type_id].annotations =
                collect_annotation_calls(&type_decl.annotations(), interner);

            if let Some(body) = type_decl.body() {
                let fields = collect_fields(
                    &body.fields(),
                    interner,
                    file_sym,
                    pkg,
                    ip,
                    &imports,
                    symbols,
                    diag,
                    &file.file_name,
                );
                arenas.types[type_id].fields = fields;

                let oneofs = collect_oneofs(
                    &body.oneofs(),
                    interner,
                    file_sym,
                    pkg,
                    ip,
                    &imports,
                    symbols,
                    diag,
                    &file.file_name,
                );
                arenas.types[type_id].oneofs = oneofs;

                // Populate nested types recursively
                for nested in body.nested_types() {
                    if let Some(inner_decl) = nested.type_decl() {
                        let nested_name = match inner_decl.name() {
                            Some(t) => t.text().to_string(),
                            None => continue,
                        };
                        let nested_full = format!("{}.{}", full, nested_name);
                        let nested_full_sym = interner.intern(&nested_full);

                        let nested_id = match symbols.types.get(&nested_full_sym) {
                            Some(id) => *id,
                            None => continue,
                        };

                        if let Some(nested_body) = inner_decl.body() {
                            let nested_fields = collect_fields(
                                &nested_body.fields(),
                                interner, file_sym, pkg, ip, &imports, symbols, diag, &file.file_name,
                            );
                            arenas.types[nested_id].fields = nested_fields;

                            let nested_oneofs = collect_oneofs(
                                &nested_body.oneofs(),
                                interner, file_sym, pkg, ip, &imports, symbols, diag, &file.file_name,
                            );
                            arenas.types[nested_id].oneofs = nested_oneofs;
                        }
                    }
                }
            }
        }

        // Populate shape fields
        for shape_decl in root.shape_decls() {
            let name_text = match shape_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let full = format!("{}.{}", ip, name_text);
            let full_sym = interner.intern(&full);

            let shape_id = match symbols.shapes.get(&full_sym) {
                Some(id) => *id,
                None => continue,
            };

            let fields: Vec<ShapeFieldDef> = shape_decl
                .fields()
                .iter()
                .filter_map(|f| {
                    let name = f.name()?.text().to_string();
                    let ty = f
                        .type_ref()
                        .map(|tr| resolve_type_ref(&tr, interner, pkg, ip, &imports, symbols, diag, &file.file_name))
                        .unwrap_or(ResolvedType::Error);
                    let annotations = collect_annotation_calls(&f.annotations(), interner);
                    Some(ShapeFieldDef {
                        name: interner.intern(&name),
                        ty,
                        annotations,
                        loc: Loc::default(),
                    })
                })
                .collect();

            arenas.shapes[shape_id].fields = fields;
        }

    }
}

/// Resolve service RPC params from AST.
///
/// Run this **after** all type expansion passes (aliases, shapes, generics,
/// Pick/Omit) so that every type name is available in the symbol table.
pub fn resolve_rpcs(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };

        let pkg = &file.package;
        let ip = &file.import_path;
        let imports = collect_imports(&root, interner, pkg);

        for svc_decl in root.service_decls() {
            let name_text = match svc_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let full = format!("{}.{}", ip, name_text);
            let full_sym = interner.intern(&full);

            let svc_id = match symbols.services.get(&full_sym) {
                Some(id) => *id,
                None => continue,
            };

            let rpcs: Vec<RpcDef> = svc_decl
                .rpcs()
                .iter()
                .filter_map(|rpc| {
                    let name = rpc.name()?.text().to_string();
                    let input = rpc
                        .input()
                        .map(|p| resolve_rpc_param(&p, interner, pkg, ip, &imports, symbols, diag, &file.file_name, arenas, &name_text, &name, true))
                        .unwrap_or(RpcParamDef {
                            is_void: true,
                            is_stream: false,
                            ty: ResolvedType::Error,
                        });
                    let output = rpc
                        .output()
                        .map(|p| resolve_rpc_param(&p, interner, pkg, ip, &imports, symbols, diag, &file.file_name, arenas, &name_text, &name, false))
                        .unwrap_or(RpcParamDef {
                            is_void: true,
                            is_stream: false,
                            ty: ResolvedType::Error,
                        });
                    let annotations = collect_annotation_calls(&rpc.annotations(), interner);
                    Some(RpcDef {
                        name: interner.intern(&name),
                        input,
                        output,
                        annotations,
                        loc: Loc::default(),
                    })
                })
                .collect();

            arenas.services[svc_id].rpcs = rpcs;
            // Collect service-level annotations
            arenas.services[svc_id].annotations =
                collect_annotation_calls(&svc_decl.annotations(), interner);
        }
    }
}

// ── Import validation ──────────────────────────────────────────────────

/// Validate that all imports resolve to known packages.
pub fn validate_imports(
    files: &[ParsedFile],
    module_path: Option<&str>,
    diag: &mut Diagnostics,
) {
    // Collect all known package names and import paths from compiled files
    let known_packages: HashSet<String> = files.iter().map(|f| f.package.clone()).collect();
    let known_import_paths: HashSet<String> = files.iter().map(|f| f.import_path.clone()).collect();

    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };

        // Check for import name collisions within each file
        let mut seen: HashMap<String, String> = HashMap::new();
        for imp in root.imports() {
            let path_text = match imp.path() {
                Some(p) => p.text(),
                None => continue,
            };
            let short = if let Some(alias) = imp.alias() {
                alias.text().to_string()
            } else if let Some(std_name) = crate::stdlib::std_package_name(&path_text) {
                std_name.to_string()
            } else {
                path_text.rsplit('/').next().unwrap_or(&path_text).to_string()
            };

            if let Some(existing_path) = seen.get(&short) {
                if existing_path != &path_text {
                    let range = imp.syntax().text_range();
                    diag.error(
                        &file.file_name,
                        usize::from(range.start())..usize::from(range.end()),
                        format!(
                            "import name collision: '{}' already imported from '{}' — use an alias: import {} as <alias>;",
                            short, existing_path, path_text
                        ),
                    );
                }
            }
            seen.insert(short, path_text);
        }

        for imp in root.imports() {
            let path_text = match imp.path() {
                Some(p) => p.text(),
                None => continue,
            };

            // Classify import
            let is_std = path_text.starts_with("github.com/oghamlang/std/");
            let is_local = module_path
                .map(|mp| path_text.starts_with(mp))
                .unwrap_or(false);

            if is_std {
                // Validate std import exists
                if !crate::stdlib::is_std_import(&path_text) {
                    let range = imp.syntax().text_range();
                    diag.error(
                        &file.file_name,
                        usize::from(range.start())..usize::from(range.end()),
                        format!("unknown standard library package: {}", path_text),
                    );
                }
            } else if is_local {
                // Validate local package exists (check both short name and full import path)
                let short = path_text.rsplit('/').next().unwrap_or(&path_text);
                if !known_packages.contains(short) && !known_import_paths.contains(&path_text) {
                    let range = imp.syntax().text_range();
                    diag.error(
                        &file.file_name,
                        usize::from(range.start())..usize::from(range.end()),
                        format!(
                            "local package '{}' not found in module — available packages: {}",
                            short,
                            {
                                let mut pkgs: Vec<_> = known_packages.iter().cloned().collect();
                                pkgs.sort();
                                pkgs.join(", ")
                            }
                        ),
                    );
                }
            }
            // External imports (not std, not local) — skip validation for now
        }
    }
}

// ── Unused import detection ────────────────────────────────────────────

/// Warn about unused imports.
pub fn check_unused_imports(
    files: &[ParsedFile],
    arenas: &Arenas,
    interner: &Interner,
    diag: &mut Diagnostics,
) {
    // Collect all package names that are actually referenced by types in the symbol table
    let mut referenced_packages: HashSet<String> = HashSet::new();

    for (_, ty) in arenas.types.iter() {
        // Check field types for cross-package references
        for field in &ty.fields {
            collect_referenced_packages(&field.ty, arenas, interner, &mut referenced_packages);
        }
        for oneof in &ty.oneofs {
            for field in &oneof.fields {
                collect_referenced_packages(&field.ty, arenas, interner, &mut referenced_packages);
            }
        }
        // Check annotations
        for ann in &ty.annotations {
            let lib = interner.resolve(ann.library);
            if !lib.is_empty() {
                referenced_packages.insert(lib.to_string());
            }
        }
        for field in &ty.fields {
            for ann in &field.annotations {
                let lib = interner.resolve(ann.library);
                if !lib.is_empty() {
                    referenced_packages.insert(lib.to_string());
                }
            }
        }
    }
    // Check service annotations and RPC types
    for (_, svc) in arenas.services.iter() {
        for ann in &svc.annotations {
            let lib = interner.resolve(ann.library);
            if !lib.is_empty() {
                referenced_packages.insert(lib.to_string());
            }
        }
        for rpc in &svc.rpcs {
            collect_referenced_packages(&rpc.input.ty, arenas, interner, &mut referenced_packages);
            collect_referenced_packages(&rpc.output.ty, arenas, interner, &mut referenced_packages);
            for ann in &rpc.annotations {
                let lib = interner.resolve(ann.library);
                if !lib.is_empty() {
                    referenced_packages.insert(lib.to_string());
                }
            }
        }
    }
    // Check shape field types and shape references (injections)
    for (_, shape) in arenas.shapes.iter() {
        for field in &shape.fields {
            collect_referenced_packages(&field.ty, arenas, interner, &mut referenced_packages);
        }
        // Shape package itself is referenced when injected
        let full = interner.resolve(shape.full_name);
        let short = split_full_name_pkg_short(full);
        referenced_packages.insert(short.to_string());
    }
    // Check type field traces — shape injections reference shape packages
    for (_, ty) in arenas.types.iter() {
        for field in &ty.fields {
            if let Some(trace) = &field.trace {
                if let Some(origin) = &trace.shape {
                    let sfull = interner.resolve(arenas.shapes[origin.shape_id].full_name);
                    let short = split_full_name_pkg_short(sfull);
                    referenced_packages.insert(short.to_string());
                }
            }
        }
    }

    // Also scan AST for qualified type references (e.g., uuid.UUID) to catch
    // packages referenced through type aliases that expand to scalars.
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        collect_ast_referenced_packages(&root, &mut referenced_packages);
    }

    // Now check each file's imports against referenced packages
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };

        for imp in root.imports() {
            let path_text = match imp.path() {
                Some(p) => p.text(),
                None => continue,
            };
            // Use std_package_name for std imports to get the correct short name
            let short_owned;
            let short = if let Some(alias) = imp.alias() {
                short_owned = alias.text().to_string();
                &short_owned
            } else if let Some(std_name) = crate::stdlib::std_package_name(&path_text) {
                std_name
            } else {
                path_text.rsplit('/').next().unwrap_or(&path_text)
            };

            // Skip checking imports for the file's own package
            if short == file.package {
                continue;
            }

            if !referenced_packages.contains(short) {
                let range = imp.syntax().text_range();
                diag.warning(
                    &file.file_name,
                    usize::from(range.start())..usize::from(range.end()),
                    format!("unused import: {}", path_text),
                );
            }
        }
    }
}

/// Walk the AST tree and collect package prefixes from qualified names
/// (e.g., `uuid.UUID` → adds `"uuid"`).
fn collect_ast_referenced_packages(root: &ast::Root, packages: &mut HashSet<String>) {
    // Walk all type references in the tree looking for qualified names.
    // Extract first segment as package prefix — handles both Ident and keyword tokens
    // (e.g., `rpc` is KwRpc keyword but valid as package name in `rpc.PageRequest`).
    fn walk(node: &crate::syntax_kind::SyntaxNode, packages: &mut HashSet<String>) {
        if let Some(qn) = ast::QualifiedName::cast(node.clone()) {
            // Get ALL text tokens (idents + keywords) separated by dots
            let all_segments: Vec<String> = qn.syntax()
                .children_with_tokens()
                .filter_map(|el| match el {
                    rowan::NodeOrToken::Token(t) if t.kind() != crate::syntax_kind::SyntaxKind::Dot => {
                        Some(t.text().to_string())
                    }
                    _ => None,
                })
                .collect();
            if all_segments.len() > 1 {
                packages.insert(all_segments[0].clone());
            }
        }
        for child in node.children() {
            walk(&child, packages);
        }
    }
    walk(root.syntax(), packages);
}

fn collect_referenced_packages(
    ty: &ResolvedType,
    arenas: &Arenas,
    interner: &Interner,
    packages: &mut HashSet<String>,
) {
    match ty {
        ResolvedType::Message(id) => {
            let full = interner.resolve(arenas.types[*id].full_name);
            // Extract short package name from full_name
            let short = split_full_name_pkg_short(full);
            packages.insert(short.to_string());
        }
        ResolvedType::Enum(id) => {
            let full = interner.resolve(arenas.enums[*id].full_name);
            let short = split_full_name_pkg_short(full);
            packages.insert(short.to_string());
        }
        ResolvedType::Array(inner) => collect_referenced_packages(inner, arenas, interner, packages),
        ResolvedType::Map { key, value } => {
            collect_referenced_packages(key, arenas, interner, packages);
            collect_referenced_packages(value, arenas, interner, packages);
        }
        _ => {}
    }
}

// ── Import collection ──────────────────────────────────────────────────

/// Map of short name → full package path for a file.
type ImportMap = std::collections::HashMap<String, String>;

fn collect_imports(root: &ast::Root, _interner: &mut Interner, _pkg: &str) -> ImportMap {
    let mut imports = ImportMap::new();
    for imp in root.imports() {
        let path_text = match imp.path() {
            Some(p) => p.text(),
            None => continue,
        };
        let short = if let Some(alias) = imp.alias() {
            alias.text().to_string()
        } else if let Some(std_name) = crate::stdlib::std_package_name(&path_text) {
            // Use the std package short name (handles proto_* prefixed packages)
            std_name.to_string()
        } else {
            // Last segment of path
            path_text
                .rsplit(['.', '/'])
                .next()
                .unwrap_or(&path_text)
                .to_string()
        };
        imports.insert(short, path_text);
    }
    imports
}

// ── Field collection ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn collect_fields(
    fields: &[ast::FieldDecl],
    interner: &mut Interner,
    _file_sym: Sym,
    pkg: &str,
    import_path: &str,
    imports: &ImportMap,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
) -> Vec<FieldDef> {
    fields
        .iter()
        .filter_map(|f| {
            let name = f.name()?.text().to_string();
            let number = f.field_number().unwrap_or(0);
            let type_ref = f.type_ref()?;
            let is_optional = type_ref.is_optional();
            let is_repeated = type_ref.array_type().is_some();

            let ty = resolve_type_ref(&type_ref, interner, pkg, import_path, imports, symbols, diag, file_name);

            let mapping = f.mapping().map(|m| {
                let segments = m.segments();
                FieldMapping {
                    chain: vec![MappingLink {
                        source_type: la_arena::Idx::from_raw(la_arena::RawIdx::from_u32(0)), // placeholder
                        source_field_name: interner.intern(segments.last().unwrap_or(&String::new())),
                        path: segments.iter().map(|s| interner.intern(s)).collect(),
                    }],
                }
            });

            let annotations = collect_annotation_calls(&f.annotations(), interner);

            Some(FieldDef {
                name: interner.intern(&name),
                number,
                ty,
                is_optional,
                is_repeated,
                annotations,
                mapping,
                trace: None,
                loc: Loc::default(),
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn collect_oneofs(
    oneofs: &[ast::OneofDecl],
    interner: &mut Interner,
    _file_sym: Sym,
    pkg: &str,
    import_path: &str,
    imports: &ImportMap,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
) -> Vec<OneofDef> {
    oneofs
        .iter()
        .filter_map(|o| {
            let name = o.name()?.text().to_string();
            let fields: Vec<OneofFieldDef> = o
                .fields()
                .iter()
                .filter_map(|f| {
                    let fname = f.name()?.text().to_string();
                    let number = f.field_number().unwrap_or(0);
                    let type_ref = f.type_ref()?;
                    let ty = resolve_type_ref(&type_ref, interner, pkg, import_path, imports, symbols, diag, file_name);

                    let mapping = f.mapping().map(|m| {
                        let segments = m.segments();
                        FieldMapping {
                            chain: vec![MappingLink {
                                source_type: la_arena::Idx::from_raw(la_arena::RawIdx::from_u32(0)),
                                source_field_name: interner.intern(segments.last().unwrap_or(&String::new())),
                                path: segments.iter().map(|s| interner.intern(s)).collect(),
                            }],
                        }
                    });

                    let annotations = collect_annotation_calls(&f.annotations(), interner);
                    Some(OneofFieldDef {
                        name: interner.intern(&fname),
                        number,
                        ty,
                        annotations,
                        mapping,
                        loc: Loc::default(),
                    })
                })
                .collect();

            let oneof_annotations = collect_annotation_calls(&o.annotations(), interner);
            Some(OneofDef {
                name: interner.intern(&name),
                fields,
                annotations: oneof_annotations,
                loc: Loc::default(),
            })
        })
        .collect()
}

fn resolve_rpc_param(
    param: &ast::RpcParam,
    interner: &mut Interner,
    pkg: &str,
    import_path: &str,
    imports: &ImportMap,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
    arenas: &mut Arenas,
    svc_name: &str,
    rpc_name: &str,
    is_input: bool,
) -> RpcParamDef {
    if param.is_void() {
        return RpcParamDef {
            is_void: true,
            is_stream: false,
            ty: ResolvedType::Error,
        };
    }

    let is_stream = param.is_stream();

    let ty = if let Some(tr) = param.type_ref() {
        resolve_type_ref(&tr, interner, pkg, import_path, imports, symbols, diag, file_name)
    } else if let Some(inline) = param.inline_type() {
        // Create an anonymous message type for inline RPC param.
        // e.g., service UserAPI { rpc CreateUser({ string name = 1; }) -> User; }
        // → generates type UserAPI_CreateUserRequest { string name = 1; }
        let suffix = if is_input { "Request" } else { "Response" };
        let type_name = format!("{}_{}{}", svc_name, rpc_name, suffix);
        let full_name = format!("{}.{}", import_path, type_name);

        let name_sym = interner.intern(&type_name);
        let full_sym = interner.intern(&full_name);

        // Collect fields from inline type
        let mut fields = Vec::new();
        for field_decl in inline.fields() {
            let field_name_text = match field_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let field_number = field_decl.field_number().unwrap_or(0);
            let is_optional = field_decl.type_ref().as_ref().map_or(false, |tr| tr.is_optional());
            let is_repeated = field_decl.type_ref().as_ref().map_or(false, |tr| tr.array_type().is_some());
            let field_ty = field_decl
                .type_ref()
                .map(|tr| resolve_type_ref(&tr, interner, pkg, import_path, imports, symbols, diag, file_name))
                .unwrap_or(ResolvedType::Error);
            let annotations = collect_annotation_calls(&field_decl.annotations(), interner);

            fields.push(FieldDef {
                name: interner.intern(&field_name_text),
                number: field_number,
                ty: field_ty,
                is_optional,
                is_repeated,
                annotations,
                mapping: None,
                trace: None,
                loc: Loc::default(),
            });
        }

        // Collect oneofs from inline type
        let file_sym = interner.intern(file_name);
        let oneofs = collect_oneofs(
            &inline.oneofs(),
            interner,
            file_sym,
            pkg,
            import_path,
            imports,
            symbols,
            diag,
            file_name,
        );

        let type_id = arenas.types.alloc(TypeDef {
            name: name_sym,
            full_name: full_sym,
            fields,
            oneofs,
            nested_types: Vec::new(),
            nested_enums: Vec::new(),
            annotations: Vec::new(),
            back_references: Vec::new(),
            trace: None,
            loc: Loc::default(),
        });

        ResolvedType::Message(type_id)
    } else {
        ResolvedType::Error
    };

    RpcParamDef {
        is_void: false,
        is_stream,
        ty,
    }
}

// ── Type reference resolution ──────────────────────────────────────────

fn resolve_type_ref(
    type_ref: &ast::TypeRef,
    interner: &mut Interner,
    pkg: &str,
    import_path: &str,
    imports: &ImportMap,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
    file_name: &str,
) -> ResolvedType {
    // Array type: []T
    if let Some(arr) = type_ref.array_type() {
        if let Some(inner_ref) = arr.element_type() {
            let inner = resolve_type_ref(&inner_ref, interner, pkg, import_path, imports, symbols, diag, file_name);
            return ResolvedType::Array(Box::new(inner));
        }
        return ResolvedType::Error;
    }

    // Map type: map<K, V>
    if let Some(map) = type_ref.map_type() {
        let key = map
            .key_type()
            .map(|k| resolve_type_ref(&k, interner, pkg, import_path, imports, symbols, diag, file_name))
            .unwrap_or(ResolvedType::Error);
        let value = map
            .value_type()
            .map(|v| resolve_type_ref(&v, interner, pkg, import_path, imports, symbols, diag, file_name))
            .unwrap_or(ResolvedType::Error);
        return ResolvedType::Map {
            key: Box::new(key),
            value: Box::new(value),
        };
    }

    // Qualified name: uuid.UUID, string, int32, etc.
    if let Some(qn) = type_ref.qualified_name() {
        let segments = qn.segments();
        if segments.is_empty() {
            return ResolvedType::Error;
        }

        // Single segment — could be scalar or same-package type
        if segments.len() == 1 {
            let name = &segments[0];
            if let Some(scalar) = try_scalar(name) {
                return ResolvedType::Scalar(scalar);
            }
            // Look up in same package (using import_path for symbol table)
            let full = format!("{}.{}", import_path, name);
            let full_sym = interner.intern(&full);
            if let Some(id) = symbols.types.get(&full_sym) {
                return ResolvedType::Message(*id);
            }
            if let Some(id) = symbols.enums.get(&full_sym) {
                return ResolvedType::Enum(*id);
            }
            // Could be a type param — leave as Unresolved for generics pass
            return ResolvedType::Unresolved(interner.intern(name));
        }

        // Multi-segment: first segment is package alias or direct package ref
        let first = &segments[0];
        let rest = &segments[1..];
        let type_name = rest.last().unwrap();

        // Check imports (first segment = import alias or package short name)
        if let Some(imp_path) = imports.get(first) {
            // The import map value is the full import path (e.g., "github.com/oghamlang/std/uuid").
            // Symbol table keys now use import_path.TypeName format.
            let full = format!("{}.{}", imp_path, type_name);
            let full_sym = interner.intern(&full);
            if let Some(id) = symbols.types.get(&full_sym) {
                return ResolvedType::Message(*id);
            }
            if let Some(id) = symbols.enums.get(&full_sym) {
                return ResolvedType::Enum(*id);
            }
            // Fallback: try with the std package name (handles std import paths)
            if let Some(std_name) = crate::stdlib::std_package_name(imp_path) {
                let std_ip = crate::stdlib::import_path_for_package(std_name).unwrap_or(imp_path);
                let std_full = format!("{}.{}", std_ip, type_name);
                let std_sym = interner.intern(&std_full);
                if let Some(id) = symbols.types.get(&std_sym) {
                    return ResolvedType::Message(*id);
                }
                if let Some(id) = symbols.enums.get(&std_sym) {
                    return ResolvedType::Enum(*id);
                }
            }
        }

        // Try as fully qualified name
        let full = segments.join(".");
        let full_sym = interner.intern(&full);
        if let Some(id) = symbols.types.get(&full_sym) {
            return ResolvedType::Message(*id);
        }

        // Cross-package: try "pkg_alias.TypeName" as a full name key
        // This matches types registered as "uuid.UUID", "time.Timestamp", etc.
        let cross_key = format!("{}.{}", first, type_name);
        let cross_sym = interner.intern(&cross_key);
        if let Some(id) = symbols.types.get(&cross_sym) {
            return ResolvedType::Message(*id);
        }
        if let Some(id) = symbols.enums.get(&cross_sym) {
            return ResolvedType::Enum(*id);
        }

        let dotted = segments.join(".");
        diag.error(file_name, 0..0, format!("unresolved type: {}", dotted));
        return ResolvedType::Error;
    }

    ResolvedType::Error
}

fn try_scalar(name: &str) -> Option<ScalarKind> {
    match name {
        "bool" => Some(ScalarKind::Bool),
        "string" => Some(ScalarKind::String),
        "bytes" => Some(ScalarKind::Bytes),
        "i8" => Some(ScalarKind::Int8),
        "int16" => Some(ScalarKind::Int16),
        "int32" => Some(ScalarKind::Int32),
        "int64" | "int" => Some(ScalarKind::Int64),
        "uint8" | "byte" => Some(ScalarKind::Uint8),
        "uint16" => Some(ScalarKind::Uint16),
        "uint32" => Some(ScalarKind::Uint32),
        "uint64" | "uint" => Some(ScalarKind::Uint64),
        "float" => Some(ScalarKind::Float),
        "double" => Some(ScalarKind::Double),
        _ => None,
    }
}

// ── Pass 4: Type alias expansion ───────────────────────────────────────

/// Expand type aliases: `type Id = uuid.UUID;` → replace all Unresolved("Id")
/// references with the target type.
pub fn expand_type_aliases(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &mut SymbolTable,
    diag: &mut Diagnostics,
) {
    // Collect aliases from AST
    let mut aliases: Vec<(Sym, ResolvedType)> = Vec::new();

    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        let pkg = &file.package;
        let ip = &file.import_path;
        let imports = collect_imports(&root, interner, pkg);

        for type_decl in root.type_decls() {
            let name_text = match type_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };

            if let Some(alias) = type_decl.alias() {
                let full = format!("{}.{}", ip, name_text);
                let full_sym = interner.intern(&full);

                let target = if let Some(tr) = alias.type_ref() {
                    resolve_type_ref(&tr, interner, pkg, ip, &imports, symbols, diag, &file.file_name)
                } else {
                    ResolvedType::Error
                };

                // Set the Alias trace on the type so IR/generators know it's an alias
                if let Some(type_id) = symbols.types.get(&full_sym) {
                    arenas.types[*type_id].trace = Some(TypeTrace::Alias { underlying: target.clone() });
                }

                aliases.push((full_sym, target));
            }
        }
    }

    // Replace alias types everywhere they're referenced
    for (alias_sym, target) in &aliases {
        let alias_type_id = match symbols.types.get(alias_sym) {
            Some(id) => *id,
            None => continue,
        };

        // Replace in all type fields
        for (_, ty) in arenas.types.iter_mut() {
            for field in &mut ty.fields {
                replace_alias(&mut field.ty, alias_type_id, target);
            }
            for oneof in &mut ty.oneofs {
                for field in &mut oneof.fields {
                    replace_alias(&mut field.ty, alias_type_id, target);
                }
            }
        }

        // Replace in shape fields (before shape injection in Pass 6)
        for (_, shape) in arenas.shapes.iter_mut() {
            for field in &mut shape.fields {
                replace_alias(&mut field.ty, alias_type_id, target);
            }
        }

        // Replace in service RPCs
        for (_, svc) in arenas.services.iter_mut() {
            for rpc in &mut svc.rpcs {
                replace_alias(&mut rpc.input.ty, alias_type_id, target);
                replace_alias(&mut rpc.output.ty, alias_type_id, target);
            }
        }
    }
}

fn replace_alias(ty: &mut ResolvedType, alias_id: TypeId, target: &ResolvedType) {
    match ty {
        ResolvedType::Message(id) if *id == alias_id => {
            *ty = target.clone();
        }
        ResolvedType::Array(inner) => replace_alias(inner, alias_id, target),
        ResolvedType::Map { key, value } => {
            replace_alias(key, alias_id, target);
            replace_alias(value, alias_id, target);
        }
        _ => {}
    }
}

// ── Pass 6: Shape injection ────────────────────────────────────────────

/// Expand shape injections: `MyShape(1..4)` inside type bodies.
/// Copies shape fields into the type with assigned field numbers.
pub fn expand_shapes(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    // Collect shape injection info from AST
    struct Injection {
        type_full_name: Sym,
        shape_name: String,
        range_start: u32,
        range_end: u32,
        insert_position: usize,
        import_path: String,
    }

    let mut injections: Vec<Injection> = Vec::new();

    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        let ip = &file.import_path;

        for type_decl in root.type_decls() {
            let name_text = match type_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let full = format!("{}.{}", ip, name_text);
            let full_sym = interner.intern(&full);

            if let Some(body) = type_decl.body() {
                for (i, inj) in body.shape_injections().iter().enumerate() {
                    let shape_name = inj.full_name();
                    if shape_name.is_empty() {
                        continue;
                    }
                    let start = inj.range_start().unwrap_or(0);
                    let end = inj.range_end().unwrap_or(0);

                    injections.push(Injection {
                        type_full_name: full_sym,
                        shape_name,
                        range_start: start,
                        range_end: end,
                        insert_position: i,
                        import_path: ip.to_string(),
                    });
                }
            }
        }
    }

    // Apply injections
    for inj in &injections {
        let type_id = match symbols.types.get(&inj.type_full_name) {
            Some(id) => *id,
            None => continue,
        };

        // Find shape: try same package first, then by qualified name
        let shape_id = resolve_shape_name(
            &inj.shape_name,
            &inj.import_path,
            interner,
            arenas,
            symbols,
        );

        let shape_id = match shape_id {
            Some(id) => id,
            None => {
                diag.error("", 0..0, format!("unresolved shape: {}", inj.shape_name));
                continue;
            }
        };

        // Expand shape fields (including nested includes)
        let expanded = expand_shape_fields(shape_id, arenas, interner, symbols, diag);

        let range_size = (inj.range_end - inj.range_start + 1) as usize;
        if expanded.len() > range_size {
            diag.error(
                "",
                0..0,
                format!(
                    "shape {} has {} fields but injection range {}..{} only fits {}",
                    inj.shape_name,
                    expanded.len(),
                    inj.range_start,
                    inj.range_end,
                    range_size,
                ),
            );
        }

        // Create fields with assigned numbers
        let mut injected_fields: Vec<FieldDef> = Vec::new();
        for (i, sf) in expanded.iter().enumerate() {
            let number = inj.range_start + i as u32;
            injected_fields.push(FieldDef {
                name: sf.name,
                number,
                ty: sf.ty.clone(),
                is_optional: matches!(sf.ty, ResolvedType::Array(_)), // check from type
                is_repeated: false,
                annotations: sf.annotations.clone(),
                mapping: None,
                trace: Some(FieldTrace {
                    shape: Some(ShapeOrigin {
                        shape_name: arenas.shapes[shape_id].name,
                        shape_id,
                        range_start: inj.range_start,
                        range_end: inj.range_end,
                    }),
                }),
                loc: Loc::default(),
            });
        }

        // Insert at the right position
        let fields = &mut arenas.types[type_id].fields;
        let pos = inj.insert_position.min(fields.len());
        for (i, f) in injected_fields.into_iter().enumerate() {
            fields.insert(pos + i, f);
        }
    }
}

/// Resolve a shape name — handles both simple ("MyShape") and qualified ("rpc.PageRequest").
fn resolve_shape_name(
    name: &str,
    current_import_path: &str,
    interner: &mut Interner,
    arenas: &Arenas,
    symbols: &SymbolTable,
) -> Option<ShapeId> {
    // If qualified (contains '.'), first segment is package alias
    if let Some(dot_pos) = name.find('.') {
        let pkg_alias = &name[..dot_pos];
        let shape_name = &name[dot_pos + 1..];

        // Scan all shapes — match by short name and package short name (last path segment)
        for (id, shape) in arenas.shapes.iter() {
            let sname = interner.resolve(shape.name).to_string();
            let sfull = interner.resolve(shape.full_name).to_string();
            // Extract short package name from full_name: "github.com/oghamlang/std/rpc.PageRequest" → "rpc"
            let spkg_short = split_full_name_pkg_short(&sfull);
            if sname == shape_name && spkg_short == pkg_alias {
                return Some(id);
            }
        }
        return None;
    }

    // Simple name — look in same package (using import_path)
    let full = format!("{}.{}", current_import_path, name);
    let full_sym = interner.intern(&full);
    if let Some(id) = symbols.shapes.get(&full_sym) {
        return Some(*id);
    }

    // Try by short name across all shapes
    for (id, shape) in arenas.shapes.iter() {
        if interner.resolve(shape.name) == name {
            return Some(id);
        }
    }

    None
}

/// Recursively expand a shape's fields, including nested shape includes.
fn expand_shape_fields(
    shape_id: ShapeId,
    arenas: &Arenas,
    interner: &Interner,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) -> Vec<ShapeFieldDef> {
    let shape = &arenas.shapes[shape_id];
    let mut result = Vec::new();

    // First, expand includes
    for &include_name in &shape.includes {
        let include_text = interner.resolve(include_name);
        // Try to find shape in same package
        let pkg = {
            let full = interner.resolve(shape.full_name);
            full.rsplit_once('.').map(|(p, _)| p.to_string()).unwrap_or_default()
        };
        let full = format!("{}.{}", pkg, include_text);
        let full_sym = interner.intern_lookup(&full);

        if let Some(inc_id) = full_sym.and_then(|s| symbols.shapes.get(&s)) {
            let expanded = expand_shape_fields(*inc_id, arenas, interner, symbols, diag);
            result.extend(expanded);
        } else {
            diag.error("", 0..0, format!("unresolved shape include: {}", include_text));
        }
    }

    // Then add own fields
    result.extend(shape.fields.iter().cloned());

    result
}

// ── Pass 5: Annotation composition ─────────────────────────────────────

/// Flatten annotation composition: expand `include` chains in annotation defs.
pub fn expand_annotation_compositions(
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    // Topological order: process annotations with no compositions first.
    // For simplicity, do multiple passes until stable.
    let ids: Vec<AnnotationDefId> = arenas.annotation_defs.iter().map(|(id, _)| id).collect();
    let mut expanded: std::collections::HashSet<AnnotationDefId> = std::collections::HashSet::new();
    let max_iterations = ids.len() + 1;

    for _ in 0..max_iterations {
        let mut progress = false;
        for &id in &ids {
            if expanded.contains(&id) {
                continue;
            }
            let compositions = arenas.annotation_defs[id].compositions.clone();
            if compositions.is_empty() {
                expanded.insert(id);
                continue;
            }

            // Check all composition targets are expanded
            let all_ready = compositions.iter().all(|c| {
                symbols
                    .annotations
                    .get(&(c.library, c.name))
                    .map(|ids| ids.iter().all(|tid| expanded.contains(tid)))
                    .unwrap_or(true) // missing = skip
            });

            if all_ready {
                // Flatten: copy params from composed annotations (use first overload)
                let mut extra_params = Vec::new();
                for comp in &compositions {
                    if let Some(target_ids) = symbols.annotations.get(&(comp.library, comp.name)) {
                        if let Some(first_id) = target_ids.first() {
                            let target = &arenas.annotation_defs[*first_id];
                            extra_params.extend(target.params.clone());
                        }
                    }
                }
                arenas.annotation_defs[id].params.extend(extra_params);
                expanded.insert(id);
                progress = true;
            }
        }
        if !progress || expanded.len() == ids.len() {
            break;
        }
    }

    // Anything not expanded = cycle
    for &id in &ids {
        if !expanded.contains(&id) {
            let name = arenas.annotation_defs[id].full_name;
            diag.error("", 0..0, format!("annotation composition cycle detected involving {:?}", name));
        }
    }
}

// ── Pass 7: Generic monomorphization ───────────────────────────────────

/// Monomorphize generic types: `Paginated<User>` → `PaginatedUser`.
pub fn monomorphize_generics(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &mut SymbolTable,
    _diag: &mut Diagnostics,
) {
    // Collect generic instantiations from AST: (import_path, generic_name, args)
    let mut instantiations: Vec<(String, String, Vec<String>)> = Vec::new();

    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        let ip = &file.import_path;
        let pkg = &file.package;
        let imports = collect_imports(&root, interner, pkg);
        collect_generic_usages(&root, ip, &imports, &mut instantiations);
    }

    // Deduplicate
    instantiations.sort();
    instantiations.dedup();

    for (ip, generic_name, args) in &instantiations {
        let full_generic = format!("{}.{}", ip, generic_name);
        let full_sym = interner.intern(&full_generic);

        let generic_id = match symbols.types.get(&full_sym) {
            Some(id) => *id,
            None => continue,
        };

        // Check it actually has type params
        // For the monomorphized name, use short names (last segment) for readability
        let short_args: Vec<&str> = args.iter().map(|a| {
            a.rsplit_once('.').map(|(_, n)| n).unwrap_or(a.as_str())
        }).collect();
        let mono_name = format!("{}{}", generic_name, short_args.join(""));
        let mono_full = format!("{}.{}", ip, mono_name);
        let mono_full_sym = interner.intern(&mono_full);

        // Skip if already created
        if symbols.types.contains_key(&mono_full_sym) {
            continue;
        }

        // Clone the generic type and substitute type params
        let generic_type = arenas.types[generic_id].clone();
        let mono_name_sym = interner.intern(&mono_name);

        let mut mono_type = TypeDef {
            name: mono_name_sym,
            full_name: mono_full_sym,
            fields: generic_type.fields.clone(),
            oneofs: generic_type.oneofs.clone(),
            nested_types: generic_type.nested_types.clone(),
            nested_enums: generic_type.nested_enums.clone(),
            annotations: generic_type.annotations.clone(),
            back_references: Vec::new(),
            trace: Some(TypeTrace::Generic {
                source_name: generic_type.name,
                type_arguments: args.iter().map(|a| interner.intern(a)).collect(),
            }),
            loc: generic_type.loc.clone(),
        };

        // Substitute type params in fields
        // For now, resolve unresolved type params to concrete types
        for field in &mut mono_type.fields {
            substitute_type_param(&mut field.ty, args, ip, interner, symbols);
        }
        for oneof in &mut mono_type.oneofs {
            for field in &mut oneof.fields {
                substitute_type_param(&mut field.ty, args, ip, interner, symbols);
            }
        }

        let id = arenas.types.alloc(mono_type);
        symbols.types.insert(mono_full_sym, id);
    }
}

fn substitute_type_param(
    ty: &mut ResolvedType,
    args: &[String],
    pkg: &str,
    interner: &mut Interner,
    symbols: &SymbolTable,
) {
    match ty {
        ResolvedType::Unresolved(sym) => {
            let name = interner.resolve(*sym);
            // Single-letter or simple name that matches a type param → substitute with first arg
            // This is simplified: real impl would match param names
            if name.len() <= 2 && !args.is_empty() {
                let arg = &args[0];
                // Try qualified name directly (e.g., "fleet.Vehicle" → already a full name)
                let full = if arg.contains('.') {
                    arg.clone()
                } else {
                    format!("{}.{}", pkg, arg)
                };
                let full_sym = interner.intern(&full);
                if let Some(id) = symbols.types.get(&full_sym) {
                    *ty = ResolvedType::Message(*id);
                } else if let Some(id) = symbols.enums.get(&full_sym) {
                    *ty = ResolvedType::Enum(*id);
                } else if let Some(scalar) = try_scalar(arg) {
                    *ty = ResolvedType::Scalar(scalar);
                }
            }
        }
        ResolvedType::Array(inner) => substitute_type_param(inner, args, pkg, interner, symbols),
        ResolvedType::Map { key, value } => {
            substitute_type_param(key, args, pkg, interner, symbols);
            substitute_type_param(value, args, pkg, interner, symbols);
        }
        _ => {}
    }
}

fn collect_generic_usages(root: &ast::Root, import_path: &str, imports: &ImportMap, out: &mut Vec<(String, String, Vec<String>)>) {
    // Walk service RPCs for generic return types like Paginated<User>
    for svc in root.service_decls() {
        for rpc in svc.rpcs() {
            if let Some(output) = rpc.output() {
                if let Some(tr) = output.type_ref() {
                    check_type_ref_generic(&tr, import_path, imports, out);
                }
            }
            if let Some(input) = rpc.input() {
                if let Some(tr) = input.type_ref() {
                    check_type_ref_generic(&tr, import_path, imports, out);
                }
            }
        }
    }
    // Walk type fields
    for type_decl in root.type_decls() {
        if let Some(body) = type_decl.body() {
            for field in body.fields() {
                if let Some(tr) = field.type_ref() {
                    check_type_ref_generic(&tr, import_path, imports, out);
                }
            }
        }
    }
}

fn check_type_ref_generic(tr: &ast::TypeRef, import_path: &str, imports: &ImportMap, out: &mut Vec<(String, String, Vec<String>)>) {
    if let Some(qn) = tr.qualified_name() {
        let type_args = tr.type_args();
        if !type_args.is_empty() {
            let name = qn.segments().join(".");
            let args: Vec<String> = type_args
                .iter()
                .filter_map(|a| {
                    a.qualified_name().map(|q| {
                        let segs = q.segments();
                        if segs.len() == 1 {
                            // Simple name like "Order" — same package, use current import_path
                            segs[0].clone()
                        } else {
                            // Qualified name like "fleet.Vehicle" — resolve through imports
                            let first = &segs[0];
                            let type_name = &segs[segs.len() - 1];
                            if let Some(imp_path) = imports.get(first.as_str()) {
                                // "fleet" → "github.com/org/proj/fleet" → "github.com/org/proj/fleet.Vehicle"
                                format!("{}.{}", imp_path, type_name)
                            } else {
                                segs.join(".")
                            }
                        }
                    })
                })
                .collect();
            if !args.is_empty() {
                out.push((import_path.to_string(), name, args));
            }
        }
    }
}

// ── Pass 8: Pick/Omit expansion ────────────────────────────────────────

/// Expand `type Sub = Pick<User, id, email>` and `type Without = Omit<User, pass>`.
pub fn expand_pick_omit(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &mut SymbolTable,
    diag: &mut Diagnostics,
) {
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        let ip = &file.import_path;

        for type_decl in root.type_decls() {
            let name_text = match type_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let full = format!("{}.{}", ip, name_text);
            let full_sym = interner.intern(&full);

            let type_id = match symbols.types.get(&full_sym) {
                Some(id) => *id,
                None => continue,
            };

            let alias = match type_decl.alias() {
                Some(a) => a,
                None => continue,
            };

            // Pick
            if let Some(pick) = alias.pick_type() {
                let source_type = pick.source_type();
                let field_list = pick.fields();

                let source_id = source_type
                    .and_then(|tr| tr.qualified_name())
                    .and_then(|qn| {
                        let name = qn.segments().last()?.clone();
                        let source_full = format!("{}.{}", ip, name);
                        let sym = interner.intern(&source_full);
                        symbols.types.get(&sym).copied()
                    });

                if let Some(src_id) = source_id {
                    let picked_names: Vec<String> = field_list
                        .map(|fl| fl.names().iter().map(|t| t.text().to_string()).collect())
                        .unwrap_or_default();

                    let source = &arenas.types[src_id];
                    let fields: Vec<FieldDef> = source
                        .fields
                        .iter()
                        .filter(|f| {
                            let fname = interner.resolve(f.name);
                            picked_names.iter().any(|n| n == fname)
                        })
                        .cloned()
                        .collect();

                    let kind_sym = interner.intern("Pick");
                    let field_name_syms: Vec<Sym> = picked_names.iter().map(|n| interner.intern(n)).collect();

                    arenas.types[type_id].fields = fields;
                    arenas.types[type_id].trace = Some(TypeTrace::PickOmit {
                        kind: kind_sym,
                        source_type: src_id,
                        field_names: field_name_syms,
                    });
                }
            }

            // Omit
            if let Some(omit) = alias.omit_type() {
                let source_type = omit.source_type();
                let field_list = omit.fields();

                let source_id = source_type
                    .and_then(|tr| tr.qualified_name())
                    .and_then(|qn| {
                        let name = qn.segments().last()?.clone();
                        let source_full = format!("{}.{}", ip, name);
                        let sym = interner.intern(&source_full);
                        symbols.types.get(&sym).copied()
                    });

                if let Some(src_id) = source_id {
                    // Collect field names to omit — expand shape references to their field names
                    let mut omitted_names: Vec<String> = Vec::new();
                    if let Some(fl) = field_list {
                        for qn in fl.qualified_names() {
                            let text = qn.text();
                            // Check if this name refers to a shape
                            if let Some(shape_id) = resolve_shape_name(&text, ip, interner, arenas, symbols) {
                                let shape_fields = expand_shape_fields(shape_id, arenas, interner, symbols, diag);
                                for sf in &shape_fields {
                                    omitted_names.push(interner.resolve(sf.name).to_string());
                                }
                            } else {
                                // It's a plain field name (last segment)
                                let segments = qn.segments();
                                if let Some(last) = segments.last() {
                                    omitted_names.push(last.clone());
                                }
                            }
                        }
                    }

                    let source = &arenas.types[src_id];
                    let fields: Vec<FieldDef> = source
                        .fields
                        .iter()
                        .filter(|f| {
                            let fname = interner.resolve(f.name);
                            !omitted_names.iter().any(|n| n == fname)
                        })
                        .cloned()
                        .collect();

                    let kind_sym = interner.intern("Omit");
                    let field_name_syms: Vec<Sym> = omitted_names.iter().map(|n| interner.intern(n)).collect();

                    arenas.types[type_id].fields = fields;
                    arenas.types[type_id].trace = Some(TypeTrace::PickOmit {
                        kind: kind_sym,
                        source_type: src_id,
                        field_names: field_name_syms,
                    });
                }
            }
        }
    }
}

// ── Pass 9: Projection resolution ──────────────────────────────────────

/// Resolve projection mappings: validate source types/fields and unwind chains.
pub fn resolve_projections(
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    let type_ids: Vec<TypeId> = arenas.types.iter().map(|(id, _)| id).collect();

    for type_id in type_ids {
        let fields_len = arenas.types[type_id].fields.len();
        for fi in 0..fields_len {
            let mapping = arenas.types[type_id].fields[fi].mapping.clone();
            if let Some(ref m) = mapping {
                if let Some(link) = m.chain.first() {
                    let resolved = resolve_mapping_chain(
                        &link.path,
                        interner,
                        arenas,
                        symbols,
                        diag,
                        &mut std::collections::HashSet::new(),
                    );
                    arenas.types[type_id].fields[fi].mapping = Some(FieldMapping { chain: resolved });
                }
            }
        }

        // Also resolve oneof field mappings
        let oneofs_len = arenas.types[type_id].oneofs.len();
        for oi in 0..oneofs_len {
            let fields_len = arenas.types[type_id].oneofs[oi].fields.len();
            for fi in 0..fields_len {
                let mapping = arenas.types[type_id].oneofs[oi].fields[fi].mapping.clone();
                if let Some(ref m) = mapping {
                    if let Some(link) = m.chain.first() {
                        let resolved = resolve_mapping_chain(
                            &link.path,
                            interner,
                            arenas,
                            symbols,
                            diag,
                            &mut std::collections::HashSet::new(),
                        );
                        arenas.types[type_id].oneofs[oi].fields[fi].mapping =
                            Some(FieldMapping { chain: resolved });
                    }
                }
            }
        }
    }
}

#[allow(clippy::only_used_in_recursion)]
fn resolve_mapping_chain(
    path: &[Sym],
    interner: &Interner,
    arenas: &Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
    visited: &mut std::collections::HashSet<(TypeId, Sym)>,
) -> Vec<MappingLink> {
    if path.is_empty() {
        return Vec::new();
    }

    // First segment is the source type name
    let type_name = interner.resolve(path[0]);

    // Try to find the type
    let type_id = arenas
        .types
        .iter()
        .find(|(_, t)| interner.resolve(t.name) == type_name)
        .map(|(id, _)| id);

    let type_id = match type_id {
        Some(id) => id,
        None => {
            diag.error("", 0..0, format!("projection: unresolved type {}", type_name));
            return vec![MappingLink {
                source_type: la_arena::Idx::from_raw(la_arena::RawIdx::from_u32(0)),
                source_field_name: *path.last().unwrap_or(&path[0]),
                path: path.to_vec(),
            }];
        }
    };

    let field_name = path.last().copied().unwrap_or(path[0]);
    let key = (type_id, field_name);

    // Cycle detection
    if visited.contains(&key) {
        diag.error("", 0..0, format!("projection cycle detected at {}.{}", type_name, interner.resolve(field_name)));
        return Vec::new();
    }
    visited.insert(key);

    let mut chain = vec![MappingLink {
        source_type: type_id,
        source_field_name: field_name,
        path: path.to_vec(),
    }];

    // Check if the source field itself has a mapping (chain unwinding)
    let source_field = arenas.types[type_id]
        .fields
        .iter()
        .find(|f| f.name == field_name);

    if let Some(sf) = source_field {
        if let Some(ref sub_mapping) = sf.mapping {
            if let Some(sub_link) = sub_mapping.chain.first() {
                let sub_chain = resolve_mapping_chain(
                    &sub_link.path,
                    interner,
                    arenas,
                    symbols,
                    diag,
                    visited,
                );
                chain.extend(sub_chain);
            }
        }
    }

    chain
}

// ── Pass 11: Cycle detection ───────────────────────────────────────────

/// Detect cycles in type references (structural recursion).
/// Reports all types involved in cycles.
pub fn detect_cycles(
    arenas: &Arenas,
    interner: &Interner,
    diag: &mut Diagnostics,
) {
    let type_ids: Vec<TypeId> = arenas.types.iter().map(|(id, _)| id).collect();

    // DFS-based cycle detection
    let mut visited = std::collections::HashSet::new();
    let mut in_stack = std::collections::HashSet::new();

    for &id in &type_ids {
        if !visited.contains(&id) {
            detect_cycles_dfs(id, arenas, interner, diag, &mut visited, &mut in_stack);
        }
    }
}

fn detect_cycles_dfs(
    id: TypeId,
    arenas: &Arenas,
    interner: &Interner,
    diag: &mut Diagnostics,
    visited: &mut std::collections::HashSet<TypeId>,
    in_stack: &mut std::collections::HashSet<TypeId>,
) {
    visited.insert(id);
    in_stack.insert(id);

    let ty = &arenas.types[id];
    for field in &ty.fields {
        if let ResolvedType::Message(target) = &field.ty {
            if !field.is_optional && !field.is_repeated {
                // Only non-optional, non-repeated fields create mandatory cycles
                if in_stack.contains(target) {
                    let source_name = interner.resolve(ty.full_name);
                    let target_name = interner.resolve(arenas.types[*target].full_name);
                    diag.warning(
                        "",
                        0..0,
                        format!(
                            "recursive type reference: {}.{} → {} (consider making it optional or repeated)",
                            source_name,
                            interner.resolve(field.name),
                            target_name,
                        ),
                    );
                } else if !visited.contains(target) {
                    detect_cycles_dfs(*target, arenas, interner, diag, visited, in_stack);
                }
            }
        }
    }

    in_stack.remove(&id);
}

// ── Pass 10: Back-references ───────────────────────────────────────────

// ── Container nesting validation ──────────────────────────────────────

/// Returns `true` if the type is a container (Array or Map).
fn is_container(ty: &ResolvedType) -> bool {
    matches!(ty, ResolvedType::Array(_) | ResolvedType::Map { .. })
}

/// Check whether a `ResolvedType` has nested containers and report an error.
fn check_nesting_depth(
    ty: &ResolvedType,
    interner: &Interner,
    parent_name: Sym,
    field_name: Sym,
    diag: &mut Diagnostics,
) {
    match ty {
        ResolvedType::Array(inner) if is_container(inner) => {
            diag.error(
                "",
                0..0,
                format!(
                    "nested container types are not supported in field `{}` of type `{}` — define a wrapper type for the inner container",
                    interner.resolve(field_name),
                    interner.resolve(parent_name),
                ),
            );
        }
        ResolvedType::Map { key, value, .. } => {
            if is_container(key) {
                diag.error(
                    "",
                    0..0,
                    format!(
                        "nested container types are not supported in map key of field `{}` of type `{}` — define a wrapper type for the inner container",
                        interner.resolve(field_name),
                        interner.resolve(parent_name),
                    ),
                );
            }
            if is_container(value) {
                diag.error(
                    "",
                    0..0,
                    format!(
                        "nested container types are not supported in map value of field `{}` of type `{}` — define a wrapper type for the inner container",
                        interner.resolve(field_name),
                        interner.resolve(parent_name),
                    ),
                );
            }
        }
        _ => {}
    }
}

/// Validate that container types are not nested (no `[][]T`, `[]map<K,V>`, `map<K,[]V>`, etc.).
/// Ogham requires flat containers — extract nested containers to separate types.
pub fn validate_container_nesting(
    arenas: &Arenas,
    interner: &Interner,
    diag: &mut Diagnostics,
) {
    for (_, ty) in arenas.types.iter() {
        for field in &ty.fields {
            check_nesting_depth(&field.ty, interner, ty.full_name, field.name, diag);
        }
        for oneof in &ty.oneofs {
            for field in &oneof.fields {
                check_nesting_depth(&field.ty, interner, ty.full_name, field.name, diag);
            }
        }
    }
}

pub fn compute_back_references(arenas: &mut Arenas) {
    // Collect all type→type references
    let mut refs: Vec<(TypeId, TypeId, Sym)> = Vec::new();

    for (id, ty) in arenas.types.iter() {
        for field in &ty.fields {
            if let ResolvedType::Message(target) = &field.ty {
                refs.push((*target, id, field.name));
            }
        }
        for oneof in &ty.oneofs {
            for field in &oneof.fields {
                if let ResolvedType::Message(target) = &field.ty {
                    refs.push((*target, id, field.name));
                }
            }
        }
    }

    for (target, referencing, field_name) in refs {
        if target.into_raw().into_u32() < arenas.types.len() as u32 {
            arenas.types[target].back_references.push(BackRef {
                referencing_type: referencing,
                field_name,
            });
        }
    }
}

// ── Pass 13: Populate annotation params from AST ─────────────────────

/// Populate `AnnotationDef.params` from the AST `AnnotationField` nodes.
pub fn populate_annotation_params(
    files: &[ParsedFile],
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    diag: &mut Diagnostics,
) {
    for file in files {
        let root = match ast::Root::cast(file.root.clone()) {
            Some(r) => r,
            None => continue,
        };
        let pkg = &file.package;
        let ip = &file.import_path;
        let imports = collect_imports(&root, interner, pkg);

        for ann_decl in root.annotation_decls() {
            let name_text = match ann_decl.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            // Annotation library key uses the short package name (not import_path)
            let lib_sym = interner.intern(pkg);
            let name_sym = interner.intern(&name_text);

            let ids = match symbols.annotations.get(&(lib_sym, name_sym)) {
                Some(ids) => ids.clone(),
                None => continue,
            };

            let params: Vec<AnnotationParamDef> = ann_decl
                .fields()
                .iter()
                .filter_map(|field| {
                    // Skip inline types for now (handled separately in Phase 4)
                    if field.inline_type().is_some() {
                        return None;
                    }
                    let name = field.name()?.text().to_string();
                    let name_sym = interner.intern(&name);
                    let is_optional = field.is_optional();

                    let ty = field
                        .type_ref()
                        .map(|tr| resolve_type_ref(&tr, interner, pkg, ip, &imports, symbols, diag, &file.file_name))
                        .unwrap_or(ResolvedType::Error);

                    Some(AnnotationParamDef {
                        name: name_sym,
                        ty,
                        is_optional,
                        default_value: None,
                    })
                })
                .collect();

            // Apply params to matching annotation def(s) for this declaration.
            // For overloaded annotations, find the one that matches this specific declaration
            // by checking that it was allocated from this file (use last in the vec as heuristic).
            // In practice each annotation_decl creates exactly one AnnotationDef.
            if let Some(&id) = ids.last() {
                arenas.annotation_defs[id].params = params;
            }
        }
    }
}

// ── Pass 14: Annotation overload resolution + validation ─────────────

/// Check that a `ResolvedType` matches a `TypeConstraint`.
fn matches_constraint(
    ty: &ResolvedType,
    constraint: &TypeConstraint,
    arenas: &Arenas,
    interner: &Interner,
) -> bool {
    match constraint {
        TypeConstraint::Any => true,
        TypeConstraint::Scalar(expected) => matches!(ty, ResolvedType::Scalar(s) if s == expected),
        TypeConstraint::Message => matches!(ty, ResolvedType::Message(_)),
        TypeConstraint::Enum => matches!(ty, ResolvedType::Enum(_)),
        TypeConstraint::Named(name_sym) => {
            let expected_name = interner.resolve(*name_sym);
            match ty {
                ResolvedType::Message(id) => {
                    let full = interner.resolve(arenas.types[*id].full_name);
                    full == expected_name
                        || full.ends_with(&format!(".{}", expected_name))
                }
                ResolvedType::Enum(id) => {
                    let full = interner.resolve(arenas.enums[*id].full_name);
                    full == expected_name
                        || full.ends_with(&format!(".{}", expected_name))
                }
                _ => false,
            }
        }
        TypeConstraint::Union(parts) => {
            parts.iter().any(|c| matches_constraint(ty, c, arenas, interner))
        }
        TypeConstraint::Array(inner) => match ty {
            ResolvedType::Array(elem) => matches_constraint(elem, inner, arenas, interner),
            _ => false,
        },
        TypeConstraint::Map { key, value } => match ty {
            ResolvedType::Map { key: k, value: v } => {
                matches_constraint(k, key, arenas, interner)
                    && matches_constraint(v, value, arenas, interner)
            }
            _ => false,
        },
    }
}

/// Score a constraint match (higher = more specific).
fn constraint_specificity(constraint: &TypeConstraint) -> u32 {
    match constraint {
        TypeConstraint::Any => 0,
        TypeConstraint::Message | TypeConstraint::Enum => 1,
        TypeConstraint::Scalar(_) | TypeConstraint::Named(_) => 3,
        TypeConstraint::Union(parts) => {
            // Union specificity = max of parts (a smaller union is more specific)
            2 + parts.iter().map(constraint_specificity).max().unwrap_or(0)
        }
        TypeConstraint::Array(inner) => 1 + constraint_specificity(inner),
        TypeConstraint::Map { key, value } => {
            1 + constraint_specificity(key) + constraint_specificity(value)
        }
    }
}

/// Resolve annotation overloads and validate annotation calls on all types.
pub fn resolve_annotation_calls(
    arenas: &mut Arenas,
    symbols: &SymbolTable,
    interner: &Interner,
    diag: &mut Diagnostics,
) {
    // Collect all (type_id, field_index, annotation_index) triples to update
    let type_ids: Vec<TypeId> = arenas.types.iter().map(|(id, _)| id).collect();

    for type_id in type_ids {
        let num_fields = arenas.types[type_id].fields.len();
        for fi in 0..num_fields {
            let field_ty = arenas.types[type_id].fields[fi].ty.clone();
            let num_anns = arenas.types[type_id].fields[fi].annotations.len();

            for ai in 0..num_anns {
                let ann = &arenas.types[type_id].fields[fi].annotations[ai];
                let key = (ann.library, ann.name);

                if let Some(candidates) = symbols.annotations.get(&key) {
                    // Find all overloads matching the field type
                    let mut matches: Vec<(AnnotationDefId, u32)> = Vec::new();
                    for &cand_id in candidates {
                        let def = &arenas.annotation_defs[cand_id];
                        for target in &def.targets {
                            let constraint = target
                                .type_constraint
                                .as_ref()
                                .unwrap_or(&TypeConstraint::Any);
                            if matches_constraint(&field_ty, constraint, arenas, interner) {
                                matches.push((cand_id, constraint_specificity(constraint)));
                            }
                        }
                    }

                    // Implicit element matching: if field is []T, try matching against T
                    if matches.is_empty() {
                        let element_ty = match &field_ty {
                            ResolvedType::Array(inner) => Some(inner.as_ref()),
                            ResolvedType::Map { value, .. } => Some(value.as_ref()),
                            _ => None,
                        };
                        if let Some(elem) = element_ty {
                            for &cand_id in candidates {
                                let def = &arenas.annotation_defs[cand_id];
                                for target in &def.targets {
                                    let constraint = target
                                        .type_constraint
                                        .as_ref()
                                        .unwrap_or(&TypeConstraint::Any);
                                    if matches_constraint(elem, constraint, arenas, interner) {
                                        matches.push((cand_id, constraint_specificity(constraint)));
                                    }
                                }
                            }
                        }
                    }

                    if matches.is_empty() {
                        let lib = interner.resolve(ann.library);
                        let name = interner.resolve(ann.name);
                        diag.error("", 0..0, format!(
                            "no overload of {}::{} matches the field type",
                            lib, name
                        ));
                    } else {
                        // Pick most specific
                        matches.sort_by(|a, b| b.1.cmp(&a.1));
                        if matches.len() > 1 && matches[0].1 == matches[1].1 {
                            let lib = interner.resolve(ann.library);
                            let name = interner.resolve(ann.name);
                            diag.error("", 0..0, format!(
                                "ambiguous overload for {}::{}",
                                lib, name
                            ));
                        }
                        arenas.types[type_id].fields[fi].annotations[ai].definition =
                            Some(matches[0].0);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index;
    use crate::parser;

    fn build(source: &str) -> (Interner, Arenas, SymbolTable, Diagnostics) {
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

        index::collect(&file, &mut interner, &mut arenas, &mut symbols, &mut diag);
        let files = [file];
        populate_and_resolve(&files, &mut interner, &mut arenas, &symbols, &mut diag);
        resolve_rpcs(&files, &mut interner, &mut arenas, &symbols, &mut diag);

        (interner, arenas, symbols, diag)
    }

    #[test]
    fn resolve_scalar_fields() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype User { string email = 1; int64 age = 2; bool active = 3; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        let fields = &arenas.types[id].fields;
        assert_eq!(fields.len(), 3);
        assert!(matches!(fields[0].ty, ResolvedType::Scalar(ScalarKind::String)));
        assert!(matches!(fields[1].ty, ResolvedType::Scalar(ScalarKind::Int64)));
        assert!(matches!(fields[2].ty, ResolvedType::Scalar(ScalarKind::Bool)));
    }

    #[test]
    fn resolve_same_package_type() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype Address { string city = 1; }\ntype User { Address addr = 1; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        let fields = &arenas.types[id].fields;
        assert!(matches!(fields[0].ty, ResolvedType::Message(_)));
    }

    #[test]
    fn resolve_enum_type() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\nenum Status { Active = 1; }\ntype User { Status status = 1; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        let fields = &arenas.types[id].fields;
        assert!(matches!(fields[0].ty, ResolvedType::Enum(_)));
    }

    #[test]
    fn resolve_array_type() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype Order { string id = 1; }\ntype User { []Order orders = 1; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        let fields = &arenas.types[id].fields;
        assert!(matches!(fields[0].ty, ResolvedType::Array(_)));
        if let ResolvedType::Array(inner) = &fields[0].ty {
            assert!(matches!(**inner, ResolvedType::Message(_)));
        }
        assert!(fields[0].is_repeated);
    }

    #[test]
    fn resolve_map_type() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype User { map<string, string> meta = 1; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        let fields = &arenas.types[id].fields;
        assert!(matches!(fields[0].ty, ResolvedType::Map { .. }));
    }

    #[test]
    fn resolve_optional_type() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype User { string? nick = 1; }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        assert!(arenas.types[id].fields[0].is_optional);
    }

    #[test]
    fn resolve_oneof_fields() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype Addr { string city = 1; }\ntype User { oneof location { Addr home = 1; Addr work = 2; } }",
        );
        let key = int.inner.get("example.User").unwrap();
        let id = symbols.types[&key];
        assert_eq!(arenas.types[id].oneofs.len(), 1);
        assert_eq!(arenas.types[id].oneofs[0].fields.len(), 2);
        assert!(matches!(arenas.types[id].oneofs[0].fields[0].ty, ResolvedType::Message(_)));
    }

    #[test]
    fn resolve_service_rpcs() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype User { string name = 1; }\nservice UserAPI { rpc Get(void) -> User; }",
        );
        let key = int.inner.get("example.UserAPI").unwrap();
        let id = symbols.services[&key];
        let rpcs = &arenas.services[id].rpcs;
        assert_eq!(rpcs.len(), 1);
        assert!(rpcs[0].input.is_void);
        assert!(!rpcs[0].output.is_void);
        assert!(matches!(rpcs[0].output.ty, ResolvedType::Message(_)));
    }

    #[test]
    fn resolve_shape_fields() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\nshape Timestamps { uint64 created_at; uint64 updated_at; }",
        );
        let key = int.inner.get("example.Timestamps").unwrap();
        let id = symbols.shapes[&key];
        assert_eq!(arenas.shapes[id].fields.len(), 2);
        assert!(matches!(arenas.shapes[id].fields[0].ty, ResolvedType::Scalar(ScalarKind::Uint64)));
    }

    #[test]
    fn back_references() {
        let (int, mut arenas, symbols, _diag) = build(
            "package example;\ntype Address { string city = 1; }\ntype User { Address addr = 1; }\ntype Order { Address billing = 1; }",
        );
        compute_back_references(&mut arenas);

        let key = int.inner.get("example.Address").unwrap();
        let id = symbols.types[&key];
        assert_eq!(arenas.types[id].back_references.len(), 2);
    }

    #[test]
    fn field_mapping_collected() {
        let (int, arenas, symbols, _diag) = build(
            "package example;\ntype User { string name = 1; }\ntype Mini { string name = 1 <- User.name; }",
        );
        let key = int.inner.get("example.Mini").unwrap();
        let id = symbols.types[&key];
        assert!(arenas.types[id].fields[0].mapping.is_some());
    }

    #[test]
    fn full_example() {
        let (_, arenas, _, diag) = build(
            r#"package example;

type User {
    string email = 1;
    string name = 2;
    int64 age = 3;
    bool active = 4;
    float score = 5;
    []string tags = 6;
    map<string, string> meta = 7;
    string? nickname = 8;
}

enum Status {
    Active = 1;
    Inactive = 2;
}

type Order {
    string id = 1;
    User owner = 2;
    Status status = 3;
}

shape Timestamps {
    uint64 created_at;
    uint64 updated_at;
}

service OrderAPI {
    rpc Get(void) -> Order;
    rpc List(void) -> stream Order;
}
"#,
        );
        assert!(!diag.has_errors(), "errors: {:?}", diag.all());
        assert_eq!(arenas.types.len(), 2);
        assert_eq!(arenas.enums.len(), 1);
        assert_eq!(arenas.shapes.len(), 1);
        assert_eq!(arenas.services.len(), 1);
    }
}

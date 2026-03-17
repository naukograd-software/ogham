//! Document and workspace state for the LSP.

#![allow(dead_code)]
use dashmap::DashMap;
use ogham_compiler::ast::{self, AstNode};
use ogham_compiler::parser::Parse;
use tower_lsp::lsp_types::*;

/// Per-document state cached after each parse.
pub struct DocumentState {
    pub source: String,
    pub parse: Parse,
}

/// A symbol definition in the workspace.
#[derive(Debug, Clone)]
pub struct SymbolDef {
    pub name: String,
    pub kind: SymbolKind,
    pub uri: Url,
    pub range: std::ops::Range<usize>,
    pub detail: String,
    /// Children (fields, enum values, rpcs)
    pub children: Vec<SymbolDef>,
}

/// Cross-file workspace index.
pub struct WorkspaceIndex {
    /// name → definitions (can have multiple for same name across packages)
    pub symbols: DashMap<String, Vec<SymbolDef>>,
}

impl WorkspaceIndex {
    pub fn new() -> Self {
        Self {
            symbols: DashMap::new(),
        }
    }

    /// Rebuild index for a single document.
    pub fn index_document(&self, uri: &Url, source: &str, parse: &Parse) {
        // Remove old entries for this URI
        self.symbols.retain(|_, defs| {
            defs.retain(|d| d.uri != *uri);
            !defs.is_empty()
        });

        let root = match ast::Root::cast(parse.syntax()) {
            Some(r) => r,
            None => return,
        };

        let pkg = root
            .package_decl()
            .and_then(|p| p.name())
            .map(|t| t.text().to_string())
            .unwrap_or_default();

        // Index types
        for ty in root.type_decls() {
            if let Some(name_tok) = ty.name() {
                let name = name_tok.text().to_string();
                let r = ty.syntax().text_range();
                let range = usize::from(r.start())..usize::from(r.end());

                let fields = ty.body().map(|b| b.fields().len()).unwrap_or(0);
                let is_alias = ty.alias().is_some();

                let detail = if is_alias {
                    format!("type alias ({})", pkg)
                } else {
                    format!("type ({} fields, {})", fields, pkg)
                };

                let children = collect_type_children(source, uri, &ty);

                let def = SymbolDef {
                    name: name.clone(),
                    kind: SymbolKind::STRUCT,
                    uri: uri.clone(),
                    range,
                    detail,
                    children,
                };
                self.symbols.entry(name).or_default().push(def);
            }
        }

        // Index enums
        for en in root.enum_decls() {
            if let Some(name_tok) = en.name() {
                let name = name_tok.text().to_string();
                let r = en.syntax().text_range();
                let range = usize::from(r.start())..usize::from(r.end());

                let children: Vec<SymbolDef> = en
                    .values()
                    .iter()
                    .filter_map(|v| {
                        let vn = v.name()?.text().to_string();
                        let vr = v.syntax().text_range();
                        Some(SymbolDef {
                            name: vn,
                            kind: SymbolKind::ENUM_MEMBER,
                            uri: uri.clone(),
                            range: usize::from(vr.start())..usize::from(vr.end()),
                            detail: format!("= {}", v.value().unwrap_or(0)),
                            children: Vec::new(),
                        })
                    })
                    .collect();

                let def = SymbolDef {
                    name: name.clone(),
                    kind: SymbolKind::ENUM,
                    uri: uri.clone(),
                    range,
                    detail: format!("enum ({} values, {})", children.len(), pkg),
                    children,
                };
                self.symbols.entry(name).or_default().push(def);
            }
        }

        // Index shapes
        for sh in root.shape_decls() {
            if let Some(name_tok) = sh.name() {
                let name = name_tok.text().to_string();
                let r = sh.syntax().text_range();
                let range = usize::from(r.start())..usize::from(r.end());

                let children: Vec<SymbolDef> = sh
                    .fields()
                    .iter()
                    .filter_map(|f| {
                        let fn_name = f.name()?.text().to_string();
                        let fr = f.syntax().text_range();
                        Some(SymbolDef {
                            name: fn_name,
                            kind: SymbolKind::FIELD,
                            uri: uri.clone(),
                            range: usize::from(fr.start())..usize::from(fr.end()),
                            detail: "shape field".into(),
                            children: Vec::new(),
                        })
                    })
                    .collect();

                let def = SymbolDef {
                    name: name.clone(),
                    kind: SymbolKind::INTERFACE,
                    uri: uri.clone(),
                    range,
                    detail: format!("shape ({})", pkg),
                    children,
                };
                self.symbols.entry(name).or_default().push(def);
            }
        }

        // Index services
        for svc in root.service_decls() {
            if let Some(name_tok) = svc.name() {
                let name = name_tok.text().to_string();
                let r = svc.syntax().text_range();
                let range = usize::from(r.start())..usize::from(r.end());

                let children: Vec<SymbolDef> = svc
                    .rpcs()
                    .iter()
                    .filter_map(|rpc| {
                        let rn = rpc.name()?.text().to_string();
                        let rr = rpc.syntax().text_range();
                        Some(SymbolDef {
                            name: rn,
                            kind: SymbolKind::METHOD,
                            uri: uri.clone(),
                            range: usize::from(rr.start())..usize::from(rr.end()),
                            detail: "rpc".into(),
                            children: Vec::new(),
                        })
                    })
                    .collect();

                let def = SymbolDef {
                    name: name.clone(),
                    kind: SymbolKind::MODULE,
                    uri: uri.clone(),
                    range,
                    detail: format!("service ({} rpcs, {})", children.len(), pkg),
                    children,
                };
                self.symbols.entry(name).or_default().push(def);
            }
        }

        // Index annotations
        for ann in root.annotation_decls() {
            if let Some(name_tok) = ann.name() {
                let name = name_tok.text().to_string();
                let r = ann.syntax().text_range();
                let range = usize::from(r.start())..usize::from(r.end());
                let targets = ann
                    .targets()
                    .map(|t| t.targets().join("|"))
                    .unwrap_or_default();

                let def = SymbolDef {
                    name: name.clone(),
                    kind: SymbolKind::PROPERTY,
                    uri: uri.clone(),
                    range,
                    detail: format!("annotation for {} ({})", targets, pkg),
                    children: Vec::new(),
                };
                self.symbols.entry(name).or_default().push(def);
            }
        }
    }

    /// Find definition by name across workspace.
    pub fn find_definition(&self, name: &str) -> Option<SymbolDef> {
        self.symbols
            .get(name)
            .and_then(|defs| defs.first().cloned())
    }

    /// Find all references to a name.
    pub fn find_references(&self, name: &str) -> Vec<SymbolDef> {
        self.symbols
            .get(name)
            .map(|defs| defs.clone())
            .unwrap_or_default()
    }

    /// Get all symbols matching a query (for workspace/symbol search).
    pub fn search(&self, query: &str) -> Vec<SymbolDef> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        for entry in self.symbols.iter() {
            for def in entry.value() {
                if def.name.to_lowercase().contains(&query_lower) {
                    results.push(def.clone());
                }
            }
        }
        results
    }

    /// Get document symbols for a specific URI.
    pub fn document_symbols(&self, uri: &Url) -> Vec<SymbolDef> {
        let mut result = Vec::new();
        for entry in self.symbols.iter() {
            for def in entry.value() {
                if def.uri == *uri {
                    result.push(def.clone());
                }
            }
        }
        result.sort_by_key(|d| d.range.start);
        result
    }

    /// Index embedded std library.
    pub fn index_std(&self) {
        for source_file in ogham_compiler::stdlib::all_std_sources() {
            let uri = Url::parse(&format!("ogham-std:///{}", source_file.name))
                .unwrap_or_else(|_| Url::parse("ogham-std:///unknown").unwrap());
            let parse = ogham_compiler::parser::parse(&source_file.content);
            self.index_document(&uri, &source_file.content, &parse);
        }
    }
}

fn collect_type_children(_source: &str, uri: &Url, ty: &ast::TypeDecl) -> Vec<SymbolDef> {
    let mut children = Vec::new();

    if let Some(body) = ty.body() {
        for field in body.fields() {
            if let Some(name_tok) = field.name() {
                let r = field.syntax().text_range();
                let num = field.field_number().unwrap_or(0);
                children.push(SymbolDef {
                    name: name_tok.text().to_string(),
                    kind: SymbolKind::FIELD,
                    uri: uri.clone(),
                    range: usize::from(r.start())..usize::from(r.end()),
                    detail: format!("= {}", num),
                    children: Vec::new(),
                });
            }
        }

        for oneof in body.oneofs() {
            if let Some(name_tok) = oneof.name() {
                let r = oneof.syntax().text_range();
                let oneof_children: Vec<SymbolDef> = oneof
                    .fields()
                    .iter()
                    .filter_map(|f| {
                        let fn_name = f.name()?.text().to_string();
                        let fr = f.syntax().text_range();
                        Some(SymbolDef {
                            name: fn_name,
                            kind: SymbolKind::FIELD,
                            uri: uri.clone(),
                            range: usize::from(fr.start())..usize::from(fr.end()),
                            detail: format!("= {}", f.field_number().unwrap_or(0)),
                            children: Vec::new(),
                        })
                    })
                    .collect();

                children.push(SymbolDef {
                    name: name_tok.text().to_string(),
                    kind: SymbolKind::OBJECT,
                    uri: uri.clone(),
                    range: usize::from(r.start())..usize::from(r.end()),
                    detail: "oneof".into(),
                    children: oneof_children,
                });
            }
        }
    }

    children
}

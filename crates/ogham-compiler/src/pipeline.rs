//! Compiler pipeline: orchestrates all passes from source to IR.

use crate::ast::{self, AstNode};
use crate::diagnostics::Diagnostics;
use crate::hir::{Arenas, Interner, SymbolTable};
use crate::index::{self, ParsedFile};
use crate::parser;
use crate::resolve;
use crate::stdlib;

/// Result of compiling a set of Ogham source files.
pub struct CompileResult {
    pub interner: Interner,
    pub arenas: Arenas,
    pub symbols: SymbolTable,
    pub diagnostics: Diagnostics,
}

/// A source file to compile.
pub struct SourceFile {
    pub name: String,
    pub content: String,
}

/// Compile a set of Ogham source files through the full pipeline.
pub fn compile(sources: &[SourceFile]) -> CompileResult {
    let mut interner = Interner::default();
    let mut arenas = Arenas::default();
    let mut symbols = SymbolTable::default();
    let mut diag = Diagnostics::new();

    // Phase 1: Parse all user files and collect std imports
    let mut files: Vec<ParsedFile> = Vec::new();
    let mut std_imports: Vec<String> = Vec::new();

    for source in sources {
        let parse = parser::parse(&source.content);

        for err in &parse.errors {
            diag.error(&source.name, err.range.clone(), &err.message);
        }

        let root = parse.syntax();
        let pkg = ast::Root::cast(root.clone())
            .and_then(|r| r.package_decl())
            .and_then(|p| p.name().map(|t| t.text().to_string()))
            .unwrap_or_else(|| "default".to_string());

        // Collect std imports from this file
        if let Some(r) = ast::Root::cast(root.clone()) {
            for imp in r.imports() {
                if let Some(path) = imp.path() {
                    let path_text = path.text();
                    if stdlib::is_std_import(&path_text) {
                        std_imports.push(path_text);
                    }
                }
            }
        }

        files.push(ParsedFile {
            file_name: source.name.clone(),
            root,
            package: pkg,
        });
    }

    // Parse and add required std sources
    let std_sources = stdlib::resolve_std_imports(&std_imports);
    for source in &std_sources {
        let parse = parser::parse(&source.content);
        let root = parse.syntax();
        let pkg = ast::Root::cast(root.clone())
            .and_then(|r| r.package_decl())
            .and_then(|p| p.name().map(|t| t.text().to_string()))
            .unwrap_or_else(|| "default".to_string());

        files.push(ParsedFile {
            file_name: source.name.clone(),
            root,
            package: pkg,
        });
    }

    // Phase 1: Index collection (Pass 2)
    for file in &files {
        index::collect(file, &mut interner, &mut arenas, &mut symbols, &mut diag);
    }

    // Phase 2: Populate fields + resolve type references (Pass 3)
    resolve::populate_and_resolve(&files, &mut interner, &mut arenas, &symbols, &mut diag);

    // Pass 4: Type alias expansion
    resolve::expand_type_aliases(&files, &mut interner, &mut arenas, &mut symbols, &mut diag);

    // Pass 5: Annotation composition
    resolve::expand_annotation_compositions(&mut arenas, &symbols, &mut diag);

    // Pass 6: Shape injection
    resolve::expand_shapes(&files, &mut interner, &mut arenas, &symbols, &mut diag);

    // Pass 7: Generic monomorphization
    resolve::monomorphize_generics(&files, &mut interner, &mut arenas, &mut symbols, &mut diag);

    // Pass 8: Pick/Omit expansion
    resolve::expand_pick_omit(&files, &mut interner, &mut arenas, &mut symbols, &mut diag);

    // Pass 9: Projection resolution
    resolve::resolve_projections(&mut interner, &mut arenas, &symbols, &mut diag);

    // Pass 11: Cycle detection
    resolve::detect_cycles(&arenas, &interner, &mut diag);

    // Pass 10: Back-references
    resolve::compute_back_references(&mut arenas);

    CompileResult {
        interner,
        arenas,
        symbols,
        diagnostics: diag,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_one(source: &str) -> CompileResult {
        compile(&[SourceFile {
            name: "test.ogham".to_string(),
            content: source.to_string(),
        }])
    }

    #[test]
    fn compile_simple() {
        let result = compile_one(
            r#"package example;

type User {
    string email = 1;
    string name = 2;
}

enum Status {
    Active = 1;
}

service UserAPI {
    rpc Get(void) -> User;
}
"#,
        );
        assert!(!result.diagnostics.has_errors());
        assert_eq!(result.arenas.types.len(), 1);
        assert_eq!(result.arenas.enums.len(), 1);
        assert_eq!(result.arenas.services.len(), 1);
    }

    #[test]
    fn compile_cross_type_reference() {
        let result = compile_one(
            r#"package example;
type Address { string city = 1; }
type User { Address home = 1; []Address all = 2; }
"#,
        );
        assert!(!result.diagnostics.has_errors());

        let key = result.interner.inner.get("example.Address").unwrap();
        let addr_id = result.symbols.types[&key];
        // User references Address — back-ref should exist
        assert!(!result.arenas.types[addr_id].back_references.is_empty());
    }

    #[test]
    fn compile_multi_file() {
        let result = compile(&[
            SourceFile {
                name: "models.ogham".to_string(),
                content: "package example;\ntype User { string name = 1; }".to_string(),
            },
            SourceFile {
                name: "api.ogham".to_string(),
                content: "package example;\nservice API { rpc Get(void) -> User; }".to_string(),
            },
        ]);
        assert!(!result.diagnostics.has_errors());
        assert_eq!(result.arenas.types.len(), 1);
        assert_eq!(result.arenas.services.len(), 1);

        // RPC output should resolve to User
        let key = result.interner.inner.get("example.API").unwrap();
        let svc_id = result.symbols.services[&key];
        let rpc = &result.arenas.services[svc_id].rpcs[0];
        assert!(matches!(rpc.output.ty, crate::hir::ResolvedType::Message(_)));
    }

    #[test]
    fn compile_with_parse_errors() {
        let result = compile_one("package example\ntype User {");
        assert!(result.diagnostics.has_errors());
    }

    #[test]
    fn compile_pick() {
        let result = compile_one(
            r#"package example;
type User { string email = 1; string name = 2; string password = 3; }
type PublicUser = Pick<User, email, name>;
"#,
        );
        assert!(!result.diagnostics.has_errors(), "errors: {:?}", result.diagnostics.all());
        let key = result.interner.inner.get("example.PublicUser").unwrap();
        let id = result.symbols.types[&key];
        let fields = &result.arenas.types[id].fields;
        assert_eq!(fields.len(), 2);
        assert!(result.arenas.types[id].trace.is_some());
    }

    #[test]
    fn compile_omit() {
        let result = compile_one(
            r#"package example;
type User { string email = 1; string name = 2; string password = 3; }
type SafeUser = Omit<User, password>;
"#,
        );
        assert!(!result.diagnostics.has_errors(), "errors: {:?}", result.diagnostics.all());
        let key = result.interner.inner.get("example.SafeUser").unwrap();
        let id = result.symbols.types[&key];
        let fields = &result.arenas.types[id].fields;
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn compile_projection_mapping() {
        let result = compile_one(
            r#"package example;
type User { string name = 1; string email = 2; }
type UserMini { string name = 1 <- User.name; string email = 2 <- User.email; }
"#,
        );
        assert!(!result.diagnostics.has_errors(), "errors: {:?}", result.diagnostics.all());
        let key = result.interner.inner.get("example.UserMini").unwrap();
        let id = result.symbols.types[&key];
        // Mappings should be resolved with source_type pointing to User
        for field in &result.arenas.types[id].fields {
            assert!(field.mapping.is_some());
            let chain = &field.mapping.as_ref().unwrap().chain;
            assert!(!chain.is_empty());
        }
    }

    #[test]
    fn compile_shape_injection() {
        let result = compile_one(
            r#"package example;
shape Timestamps { uint64 created_at; uint64 updated_at; }
type User { Timestamps(1..2) string email = 3; }
"#,
        );
        assert!(!result.diagnostics.has_errors(), "errors: {:?}", result.diagnostics.all());
        let key = result.interner.inner.get("example.User").unwrap();
        let id = result.symbols.types[&key];
        let fields = &result.arenas.types[id].fields;
        // 2 injected + 1 own = 3
        assert_eq!(fields.len(), 3);
        // First two should have shape trace
        assert!(fields[0].trace.is_some());
        assert!(fields[1].trace.is_some());
        assert!(fields[2].trace.is_none());
    }

    #[test]
    fn compile_full_pipeline() {
        let result = compile_one(
            r#"package example;

shape Timestamps {
    uint64 created_at;
    uint64 updated_at;
}

type User {
    Timestamps(1..2)
    string email = 3;
    string name = 4;
    string password = 5;
}

type PublicUser = Pick<User, email, name>;

enum Status {
    Active = 1;
    Inactive = 2;
}

type Order {
    string id = 1;
    User owner = 2;
    Status status = 3;
}

type OrderMini {
    string id = 1 <- Order.id;
}

service OrderAPI {
    rpc Get(void) -> Order;
    rpc List(void) -> stream Order;
}
"#,
        );
        assert!(!result.diagnostics.has_errors(), "errors: {:?}", result.diagnostics.all());
    }
}

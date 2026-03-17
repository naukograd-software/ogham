//! Built-in standard library.
//!
//! All std .ogham sources are embedded into the compiler binary via `include_str!`.
//! When the compiler encounters `import github.com/oghamlang/std/<pkg>`, it
//! resolves it from these embedded sources — no network, no filesystem lookup.

use crate::pipeline::SourceFile;

/// Embedded std source files.
static STD_SOURCES: &[(&str, &str)] = &[
    (
        "std/uuid/uuid.ogham",
        include_str!("../../../std/uuid/uuid.ogham"),
    ),
    (
        "std/ulid/ulid.ogham",
        include_str!("../../../std/ulid/ulid.ogham"),
    ),
    (
        "std/time/time.ogham",
        include_str!("../../../std/time/time.ogham"),
    ),
    (
        "std/duration/duration.ogham",
        include_str!("../../../std/duration/duration.ogham"),
    ),
    (
        "std/decimal/decimal.ogham",
        include_str!("../../../std/decimal/decimal.ogham"),
    ),
    (
        "std/geo/geo.ogham",
        include_str!("../../../std/geo/geo.ogham"),
    ),
    (
        "std/empty/empty.ogham",
        include_str!("../../../std/empty/empty.ogham"),
    ),
    (
        "std/fieldmask/fieldmask.ogham",
        include_str!("../../../std/fieldmask/fieldmask.ogham"),
    ),
    (
        "std/money/money.ogham",
        include_str!("../../../std/money/money.ogham"),
    ),
    (
        "std/any/any.ogham",
        include_str!("../../../std/any/any.ogham"),
    ),
    (
        "std/struct/struct.ogham",
        include_str!("../../../std/struct/struct.ogham"),
    ),
    (
        "std/wrappers/wrappers.ogham",
        include_str!("../../../std/wrappers/wrappers.ogham"),
    ),
    (
        "std/validate/validate.ogham",
        include_str!("../../../std/validate/validate.ogham"),
    ),
    (
        "std/rpc/rpc.ogham",
        include_str!("../../../std/rpc/rpc.ogham"),
    ),
];

/// Map of import path prefix → std package short name.
/// e.g., "github.com/oghamlang/std/uuid" → "uuid"
static STD_IMPORT_MAP: &[(&str, &str)] = &[
    ("github.com/oghamlang/std/uuid", "uuid"),
    ("github.com/oghamlang/std/ulid", "ulid"),
    ("github.com/oghamlang/std/time", "time"),
    ("github.com/oghamlang/std/duration", "duration"),
    ("github.com/oghamlang/std/decimal", "decimal"),
    ("github.com/oghamlang/std/geo", "geo"),
    ("github.com/oghamlang/std/empty", "empty"),
    ("github.com/oghamlang/std/fieldmask", "fieldmask"),
    ("github.com/oghamlang/std/money", "money"),
    ("github.com/oghamlang/std/any", "any"),
    ("github.com/oghamlang/std/struct", "struct"),
    ("github.com/oghamlang/std/wrappers", "wrappers"),
    ("github.com/oghamlang/std/validate", "validate"),
    ("github.com/oghamlang/std/rpc", "rpc"),
];

/// Check if an import path refers to a std package.
pub fn is_std_import(path: &str) -> bool {
    STD_IMPORT_MAP.iter().any(|(prefix, _)| *prefix == path)
}

/// Get the package short name for a std import path.
pub fn std_package_name(path: &str) -> Option<&'static str> {
    STD_IMPORT_MAP
        .iter()
        .find(|(prefix, _)| *prefix == path)
        .map(|(_, name)| *name)
}

/// Collect all std source files that are referenced by imports in user code.
/// Returns SourceFile entries ready to be compiled alongside user files.
pub fn resolve_std_imports(user_imports: &[String]) -> Vec<SourceFile> {
    let mut result = Vec::new();
    let mut added = std::collections::HashSet::new();

    for import_path in user_imports {
        if let Some(pkg_name) = std_package_name(import_path) {
            if added.contains(pkg_name) {
                continue;
            }
            // Find the matching std source
            for (file_name, content) in STD_SOURCES {
                if file_name.contains(&format!("/{}/", pkg_name)) {
                    result.push(SourceFile {
                        name: file_name.to_string(),
                        content: content.to_string(),
                    });
                }
            }
            added.insert(pkg_name);

            // Also resolve transitive std imports from this std package
            let transitive = collect_imports_from_source(pkg_name);
            for t in transitive {
                if let Some(t_name) = std_package_name(&t) {
                    if added.contains(t_name) {
                        continue;
                    }
                    for (file_name, content) in STD_SOURCES {
                        if file_name.contains(&format!("/{}/", t_name)) {
                            result.push(SourceFile {
                                name: file_name.to_string(),
                                content: content.to_string(),
                            });
                        }
                    }
                    added.insert(t_name);
                }
            }
        }
    }

    result
}

/// Extract import paths from a std source file (simple text scan).
fn collect_imports_from_source(pkg_name: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for (file_name, content) in STD_SOURCES {
        if file_name.contains(&format!("/{}/", pkg_name)) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    let path = rest.trim_end_matches(';').trim();
                    if path.starts_with("github.com/oghamlang/std/") {
                        imports.push(path.to_string());
                    }
                }
            }
        }
    }
    imports
}

/// Return ALL std source files (for testing or full std compilation).
pub fn all_std_sources() -> Vec<SourceFile> {
    STD_SOURCES
        .iter()
        .map(|(name, content)| SourceFile {
            name: name.to_string(),
            content: content.to_string(),
        })
        .collect()
}

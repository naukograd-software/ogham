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
        "std/money/money.ogham",
        include_str!("../../../std/money/money.ogham"),
    ),
    // Proto-compatible WKT types (std/proto/*)
    (
        "std/proto/timestamp/timestamp.ogham",
        include_str!("../../../std/proto/timestamp/timestamp.ogham"),
    ),
    (
        "std/proto/duration/duration.ogham",
        include_str!("../../../std/proto/duration/duration.ogham"),
    ),
    (
        "std/proto/any/any.ogham",
        include_str!("../../../std/proto/any/any.ogham"),
    ),
    (
        "std/proto/struct/struct.ogham",
        include_str!("../../../std/proto/struct/struct.ogham"),
    ),
    (
        "std/proto/wrappers/wrappers.ogham",
        include_str!("../../../std/proto/wrappers/wrappers.ogham"),
    ),
    (
        "std/proto/empty/empty.ogham",
        include_str!("../../../std/proto/empty/empty.ogham"),
    ),
    (
        "std/proto/fieldmask/fieldmask.ogham",
        include_str!("../../../std/proto/fieldmask/fieldmask.ogham"),
    ),
    (
        "std/validate/validate.ogham",
        include_str!("../../../std/validate/validate.ogham"),
    ),
    (
        "std/rpc/rpc.ogham",
        include_str!("../../../std/rpc/rpc.ogham"),
    ),
    (
        "std/default/default.ogham",
        include_str!("../../../std/default/default.ogham"),
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
    ("github.com/oghamlang/std/money", "money"),
    // Proto-compatible WKT types
    ("github.com/oghamlang/std/proto/timestamp", "timestamp"),
    ("github.com/oghamlang/std/proto/duration", "proto_duration"),
    ("github.com/oghamlang/std/proto/any", "any"),
    ("github.com/oghamlang/std/proto/struct", "struct"),
    ("github.com/oghamlang/std/proto/wrappers", "wrappers"),
    ("github.com/oghamlang/std/proto/empty", "empty"),
    ("github.com/oghamlang/std/proto/fieldmask", "fieldmask"),
    ("github.com/oghamlang/std/validate", "validate"),
    ("github.com/oghamlang/std/rpc", "rpc"),
    ("github.com/oghamlang/std/default", "default"),
];

/// Check if a package short name belongs to std.
pub fn is_std_package(pkg: &str) -> bool {
    STD_IMPORT_MAP.iter().any(|(_, name)| *name == pkg)
}

/// Look up the full import path from a short package name.
/// e.g., "uuid" → Some("github.com/oghamlang/std/uuid")
pub fn import_path_for_package(pkg: &str) -> Option<&'static str> {
    STD_IMPORT_MAP
        .iter()
        .find(|(_, name)| *name == pkg)
        .map(|(path, _)| *path)
}

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

/// Find the std source file for a given import path.
fn find_std_source(import_path: &str) -> Option<(&'static str, &'static str)> {
    // Derive file path from import: github.com/oghamlang/std/X/Y → std/X/Y/*.ogham
    let suffix = import_path.strip_prefix("github.com/oghamlang/")?;
    // suffix = "std/proto/any" or "std/uuid"
    for (file_name, content) in STD_SOURCES {
        // file_name = "std/proto/any/any.ogham" — check if it starts with suffix + "/"
        if file_name.starts_with(&format!("{}/", suffix)) {
            return Some((file_name, content));
        }
    }
    None
}

/// Collect all std source files that are referenced by imports in user code.
/// Returns SourceFile entries ready to be compiled alongside user files.
pub fn resolve_std_imports(user_imports: &[String]) -> Vec<SourceFile> {
    let mut result = Vec::new();
    let mut added = std::collections::HashSet::new();

    let mut queue: Vec<String> = user_imports
        .iter()
        .filter(|p| is_std_import(p))
        .cloned()
        .collect();

    while let Some(import_path) = queue.pop() {
        if added.contains(&import_path) {
            continue;
        }
        added.insert(import_path.clone());

        if let Some((file_name, content)) = find_std_source(&import_path) {
            // Scan for transitive std imports
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("import ") {
                    let path = rest.trim_end_matches(';').trim();
                    if path.starts_with("github.com/oghamlang/std/") && !added.contains(path) {
                        queue.push(path.to_string());
                    }
                }
            }

            result.push(SourceFile {
                name: file_name.to_string(),
                content: content.to_string(),
            });
        }
    }

    result
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

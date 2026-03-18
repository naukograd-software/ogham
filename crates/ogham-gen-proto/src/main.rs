//! ogham-gen-proto — export .proto files from Ogham schemas.
//!
//! Output layout: one .proto file per ogham package with proper imports.
//!
//!   proto/
//!   ├── store.proto        ← own package
//!   ├── common.proto       ← dependency
//!   └── user.proto         ← dependency
//!
//! Options (from ogham.gen.yaml opts):
//!
//!   go_package_prefix       — Go import prefix (default: module_path/out)
//!   java_package_prefix     — Java package prefix (e.g. "com.myteam")
//!   java_outer_classname    — explicit outer class (default: derived from filename)
//!   csharp_namespace_prefix — C# namespace prefix (e.g. "MyTeam.Proto")
//!   swift_prefix            — Swift class prefix
//!   php_namespace_prefix    — PHP namespace prefix
//!   ruby_package_prefix     — Ruby module prefix
//!   objc_class_prefix       — Objective-C class prefix
//!   import_prefix           — prefix for proto package and import paths
//!                             e.g. "proto" → package proto.store; import "proto/common.proto";
//!   optimize_for            — SPEED, CODE_SIZE, or LITE_RUNTIME

use std::collections::{BTreeMap, BTreeSet, HashMap};
use oghamgen::*;

fn main() {
    run(|req: CompileRequest| {
        let module = req.module.as_ref().ok_or("no module in request")?;
        let opts = &req.options;
        let module_path = &req.module_path;
        let output_dir = &req.output_dir;

        // import_prefix: prepended to proto package, import paths, and filenames.
        // e.g. "proto" → package proto.store; import "proto/common.proto";
        let import_prefix = opts.get("import_prefix")
            .filter(|v| !v.is_empty())
            .map(|v| v.trim_matches('/').trim_matches('.').to_string())
            .unwrap_or_default();

        // Build alias map: full_name → underlying TypeReference for type aliases.
        let alias_map = collect_alias_map(module);

        // Group declarations by source package.
        let bundles = group_by_package(module);

        // Collect ALL annotation ext defs globally (across all bundles).
        let global_ext_defs = collect_global_annotation_ext_defs(&bundles);

        // Group ext defs by annotation library.
        let ext_defs_by_lib = group_ext_defs_by_library(&global_ext_defs);

        let mut resp = CompileResponse::default();

        // Generate one options file per annotation library.
        // Determine module_path for each annotation library.
        // Some libraries (validate) are annotation-only — no data bundle exists.
        // Infer from existing bundles or from annotations themselves.
        let std_module_path = "github.com/oghamlang/std";
        let is_std_lib = |lib: &str| -> bool {
            // Check if any type in any bundle uses this lib with std module_path
            bundles.values().any(|b| {
                b.module_path == std_module_path
                    || b.types.iter().any(|t| t.annotations.iter().any(|a| a.library == lib))
                        && b.module_path == std_module_path
            }) || ["validate", "default", "rpc"].contains(&lib)
        };

        for (lib, defs) in &ext_defs_by_lib {
            let lib_module = bundles.get(lib)
                .map(|b| b.module_path.as_str())
                .unwrap_or_else(|| if is_std_lib(lib) { std_module_path } else { module_path });
            let prefix = output_prefix(lib_module, module_path);
            let file = gen_options_file(lib, defs, &import_prefix, &prefix);
            resp.files.push(file);
        }

        // Generate data files (types, enums, services).
        // Skip bundles that would produce empty files (all types are WKTs).
        for (_, bundle) in &bundles {
            let has_types = bundle.types.iter().any(|t| well_known_proto_type(&t.full_name).is_none());
            let has_enums = bundle.enums.iter().any(|e| well_known_proto_type(&e.full_name).is_none());
            let has_services = !bundle.services.is_empty();
            if !has_types && !has_enums && !has_services {
                continue;
            }
            let file = gen_proto_file(bundle, module_path, output_dir, opts, &bundles, &import_prefix, &alias_map, &global_ext_defs, &ext_defs_by_lib);
            resp.files.push(file);
        }

        Ok(resp)
    });
}

// ── Package bundling ──────────────────────────────────────────────────

struct PkgBundle<'a> {
    ogham_pkg: String,
    module_path: String,
    types: Vec<&'a Type>,
    enums: Vec<&'a Enum>,
    services: Vec<&'a Service>,
}

fn group_by_package<'a>(module: &'a Module) -> BTreeMap<String, PkgBundle<'a>> {
    let mut bundles: BTreeMap<String, PkgBundle<'a>> = BTreeMap::new();

    let ensure = |bundles: &mut BTreeMap<String, PkgBundle<'a>>, pkg: &str, mp: &str| {
        bundles.entry(pkg.to_string()).or_insert_with(|| PkgBundle {
            ogham_pkg: pkg.to_string(),
            module_path: mp.to_string(),
            types: Vec::new(),
            enums: Vec::new(),
            services: Vec::new(),
        });
    };

    for ty in &module.types {
        let pkg = source_package(ty.module.as_ref(), &module.package);
        let mp = ty.module.as_ref().map(|m| m.module_path.as_str()).unwrap_or("");
        ensure(&mut bundles, &pkg, mp);
        bundles.get_mut(&pkg).unwrap().types.push(ty);
    }
    for en in &module.enums {
        let pkg = source_package(en.module.as_ref(), &module.package);
        let mp = en.module.as_ref().map(|m| m.module_path.as_str()).unwrap_or("");
        ensure(&mut bundles, &pkg, mp);
        bundles.get_mut(&pkg).unwrap().enums.push(en);
    }
    for svc in &module.services {
        let pkg = source_package(svc.module.as_ref(), &module.package);
        let mp = svc.module.as_ref().map(|m| m.module_path.as_str()).unwrap_or("");
        ensure(&mut bundles, &pkg, mp);
        bundles.get_mut(&pkg).unwrap().services.push(svc);
    }

    bundles
}

/// Collect a map of alias type full_names to their underlying TypeReference.
fn collect_alias_map(module: &Module) -> HashMap<String, TypeReference> {
    let mut map = HashMap::new();
    for ty in &module.types {
        if let Some(ref trace) = ty.trace {
            if let Some(type_trace::Origin::Alias(ref alias)) = trace.origin {
                if let Some(ref underlying) = alias.underlying {
                    map.insert(ty.full_name.clone(), underlying.clone());
                }
            }
        }
    }
    map
}

/// Collect all unique annotation ext defs across ALL bundles (globally).
fn collect_global_annotation_ext_defs(bundles: &BTreeMap<String, PkgBundle>) -> BTreeMap<String, AnnotationExtDef> {
    let mut defs: BTreeMap<String, AnnotationExtDef> = BTreeMap::new();

    let mut register = |ann: &AnnotationCall, target: &str| {
        let key = format!("{}::{}", ann.library, ann.name);
        let def = defs.entry(key.clone()).or_insert_with(|| AnnotationExtDef {
            library: ann.library.clone(),
            name: ann.name.clone(),
            ext_name: ext_name_for(&ann.library, &ann.name),
            msg_name: ext_msg_name_for(&ann.library, &ann.name),
            field_number: annotation_ext_number(&ann.library, &ann.name),
            params: Vec::new(),
            targets: BTreeSet::new(),
        });
        def.targets.insert(target.to_string());
        for arg in &ann.arguments {
            let param_name = if arg.name.is_empty() { "value".to_string() } else { arg.name.clone() };
            let proto_type = arg.value.as_ref().map(infer_proto_type).unwrap_or_else(|| "string".to_string());
            if !def.params.iter().any(|(n, _)| *n == param_name) {
                def.params.push((param_name, proto_type));
            }
        }
    };

    for (_, bundle) in bundles {
        for ty in &bundle.types {
            for ann in &ty.annotations { register(ann, "message"); }
            for f in &ty.fields {
                for ann in &f.annotations { register(ann, "field"); }
            }
            for o in &ty.oneofs {
                for ann in &o.annotations { register(ann, "oneof"); }
                for f in &o.fields {
                    for ann in &f.annotations { register(ann, "field"); }
                }
            }
        }
        for en in &bundle.enums {
            for ann in &en.annotations { register(ann, "enum"); }
            for v in &en.values {
                for ann in &v.annotations { register(ann, "enum_value"); }
            }
        }
        for svc in &bundle.services {
            for ann in &svc.annotations { register(ann, "service"); }
            for rpc in &svc.rpcs {
                for ann in &rpc.annotations { register(ann, "method"); }
            }
        }
    }

    defs
}

/// Group global ext defs by their annotation library name.
fn group_ext_defs_by_library<'a>(global_ext_defs: &'a BTreeMap<String, AnnotationExtDef>) -> BTreeMap<String, Vec<&'a AnnotationExtDef>> {
    let mut by_lib: BTreeMap<String, Vec<&'a AnnotationExtDef>> = BTreeMap::new();
    for (_, def) in global_ext_defs {
        by_lib.entry(def.library.clone()).or_default().push(def);
    }
    by_lib
}

/// Generate a single options proto file for one annotation library.
fn gen_options_file(
    lib: &str,
    defs: &[&AnnotationExtDef],
    import_prefix: &str,
    output_prefix_str: &str,
) -> GeneratedFile {
    let mut w = CodeWriter::with_indent("  ");
    w.raw("// Generated by ogham-gen-proto. DO NOT EDIT.");
    w.raw("syntax = \"proto3\";");
    w.newline();
    w.raw(&format!("package {};", lib));
    w.newline();
    w.raw("import \"google/protobuf/descriptor.proto\";");

    // Check if any def needs google.protobuf.Struct
    let needs_struct = defs.iter().any(|d| d.params.iter().any(|(_, t)| t == "google.protobuf.Struct"));
    let needs_list_value = defs.iter().any(|d| d.params.iter().any(|(_, t)| t == "google.protobuf.ListValue"));
    if needs_struct || needs_list_value {
        w.raw("import \"google/protobuf/struct.proto\";");
    }
    w.newline();

    // Emit all message definitions
    for def in defs {
        w.open(&format!("message {} {{", def.msg_name));
        for (i, (param_name, proto_type)) in def.params.iter().enumerate() {
            w.line(&format!("{} {} = {};", proto_type, param_name, i + 1));
        }
        w.close("}");
        w.newline();
    }

    // Group extends by target type
    let mut extends: BTreeMap<&str, Vec<(&AnnotationExtDef, &str)>> = BTreeMap::new();
    for def in defs {
        for target in &def.targets {
            let (options_type, suffix) = match target.as_str() {
                "message" => ("google.protobuf.MessageOptions", "msg"),
                "field" => ("google.protobuf.FieldOptions", "field"),
                "oneof" => ("google.protobuf.OneofOptions", "oneof"),
                "enum" => ("google.protobuf.EnumOptions", "enum"),
                "enum_value" => ("google.protobuf.EnumValueOptions", "enum_value"),
                "service" => ("google.protobuf.ServiceOptions", "svc"),
                "method" => ("google.protobuf.MethodOptions", "method"),
                _ => continue,
            };
            extends.entry(options_type).or_default().push((def, suffix));
        }
    }

    for (options_type, entries) in &extends {
        w.open(&format!("extend {} {{", options_type));
        for (def, suffix) in entries {
            w.line(&format!(
                "optional {} {}_{} = {};",
                def.msg_name, def.ext_name, suffix, def.field_number
            ));
        }
        w.close("}");
        w.newline();
    }

    let lib_short = lib.rsplit('/').next().unwrap_or(lib);
    let filename = format!("{}{}/{}_options.proto", output_prefix_str, lib, lib_short);
    let prefixed = prefixed_path(import_prefix, &filename);
    w.to_file(&prefixed)
}

/// Collect which annotation libraries are used in a bundle.
fn collect_annotation_libraries_used(bundle: &PkgBundle) -> BTreeSet<String> {
    let mut libs = BTreeSet::new();
    let mut collect = |anns: &[AnnotationCall]| {
        for ann in anns {
            libs.insert(ann.library.clone());
        }
    };
    for ty in &bundle.types {
        collect(&ty.annotations);
        for f in &ty.fields { collect(&f.annotations); }
        for o in &ty.oneofs {
            collect(&o.annotations);
            for f in &o.fields { collect(&f.annotations); }
        }
    }
    for en in &bundle.enums {
        collect(&en.annotations);
        for v in &en.values { collect(&v.annotations); }
    }
    for svc in &bundle.services {
        collect(&svc.annotations);
        for rpc in &svc.rpcs { collect(&rpc.annotations); }
    }
    libs
}

fn source_package(mi: Option<&ModuleInfo>, fallback: &str) -> String {
    mi.and_then(|m| {
        if m.package.is_empty() {
            None
        } else {
            Some(m.package.clone())
        }
    })
    .unwrap_or_else(|| fallback.to_string())
}

/// Compute output directory prefix for a bundle.
/// User packages (same module_path as request) → "" (root).
/// External packages → relative module identifier (e.g., "std/").
fn output_prefix(bundle_module_path: &str, request_module_path: &str) -> String {
    if bundle_module_path == request_module_path || bundle_module_path.is_empty() {
        String::new()
    } else {
        // Extract short module name from module_path
        // "github.com/oghamlang/std" → "std"
        // "github.com/other/lib" → "lib"
        let short = bundle_module_path.rsplit('/').next().unwrap_or(bundle_module_path);
        format!("{}/", short)
    }
}

// ── Proto file generation ─────────────────────────────────────────────

fn gen_proto_file(
    bundle: &PkgBundle,
    module_path: &str,
    output_dir: &str,
    opts: &std::collections::HashMap<String, String>,
    all_bundles: &BTreeMap<String, PkgBundle>,
    import_prefix: &str,
    alias_map: &HashMap<String, TypeReference>,
    global_ext_defs: &BTreeMap<String, AnnotationExtDef>,
    ext_defs_by_lib: &BTreeMap<String, Vec<&AnnotationExtDef>>,
) -> GeneratedFile {
    let pkg = &bundle.ogham_pkg;
    let proto_pkg = prefixed_proto_package(import_prefix, pkg);
    let mut w = CodeWriter::with_indent("  ");

    // Header
    w.raw("// Generated by ogham-gen-proto. DO NOT EDIT.");
    w.raw("syntax = \"proto3\";");
    w.newline();
    w.raw(&format!("package {};", proto_pkg));
    w.newline();

    // File-level options
    emit_file_options(&mut w, pkg, module_path, output_dir, opts);

    // Cross-package imports
    let imported_pkgs = collect_imported_packages(bundle, pkg, all_bundles);
    for imp_pkg in &imported_pkgs {
        let imp_short = imp_pkg.rsplit('/').next().unwrap_or(imp_pkg);
        let imp_prefix = all_bundles.get(imp_pkg)
            .map(|b| output_prefix(&b.module_path, module_path))
            .unwrap_or_default();
        let import_path = prefixed_path(import_prefix, &format!("{}{}/{}.proto", imp_prefix, imp_pkg, imp_short));
        w.raw(&format!("import \"{}\";", import_path));
    }

    // Well-known type imports
    let needs_empty = bundle.services.iter().any(|s| {
        s.rpcs.iter().any(|r| {
            r.input.as_ref().is_some_and(|p| p.is_void)
                || r.output.as_ref().is_some_and(|p| p.is_void)
        })
    });
    let has_annotations = bundle_has_annotations(bundle);
    let wkt_imports = collect_wkt_imports(bundle);

    if needs_empty {
        w.raw("import \"google/protobuf/empty.proto\";");
    }

    // Import annotation options files instead of descriptor.proto
    if has_annotations {
        let libs_used = collect_annotation_libraries_used(bundle);
        for lib in &libs_used {
            if ext_defs_by_lib.contains_key(lib) {
                let lib_short = lib.rsplit('/').next().unwrap_or(lib);
                let std_mp = "github.com/oghamlang/std";
                let lib_mp = all_bundles.get(lib)
                    .map(|b| b.module_path.as_str())
                    .unwrap_or_else(|| if ["validate", "default", "rpc"].contains(&lib.as_str()) { std_mp } else { module_path });
                let lib_prefix = output_prefix(lib_mp, module_path);
                let options_path = prefixed_path(import_prefix, &format!("{}{}/{}_options.proto", lib_prefix, lib, lib_short));
                w.raw(&format!("import \"{}\";", options_path));
            }
        }
    }

    for wkt_import in &wkt_imports {
        w.raw(&format!("import \"{}\";", wkt_import));
    }
    if !imported_pkgs.is_empty() || needs_empty || has_annotations || !wkt_imports.is_empty() {
        w.newline();
    }

    // Use global ext_defs for option formatting (no inline extension emission)

    // Enums (skip well-known proto types)
    for en in &bundle.enums {
        if well_known_proto_type(&en.full_name).is_some() {
            continue;
        }
        emit_enum(&mut w, en, global_ext_defs);
        w.newline();
    }

    // Messages (skip well-known proto types)
    for ty in &bundle.types {
        if well_known_proto_type(&ty.full_name).is_some() {
            continue;
        }
        // Alias types → wrapper message with single `value` field
        if let Some(underlying) = alias_map.get(&ty.full_name) {
            emit_alias_wrapper(&mut w, ty, underlying, pkg, alias_map, global_ext_defs);
            w.newline();
            continue;
        }
        emit_message(&mut w, ty, pkg, alias_map, global_ext_defs);
        w.newline();
    }

    // Services
    for svc in &bundle.services {
        emit_service(&mut w, svc, pkg, alias_map, global_ext_defs);
        w.newline();
    }

    let prefix = output_prefix(&bundle.module_path, module_path);
    let short = pkg.rsplit('/').next().unwrap_or(pkg);
    let filename = format!("{}{}/{}.proto", prefix, pkg, short);
    w.to_file(&filename)
}

// ── File-level options ────────────────────────────────────────────────

fn emit_file_options(
    w: &mut CodeWriter,
    pkg: &str,
    module_path: &str,
    output_dir: &str,
    opts: &std::collections::HashMap<String, String>,
) {
    let mut any_option = false;

    // go_package
    if let Some(val) = compute_go_package(opts, pkg, module_path, output_dir) {
        w.raw(&format!("option go_package = \"{}\";", val));
        any_option = true;
    }

    // java_package
    if let Some(val) = compute_dot_package(opts, "java_package_prefix", pkg) {
        w.raw(&format!("option java_package = \"{}\";", val));
        any_option = true;
    }

    // java_outer_classname
    if let Some(val) = opts.get("java_outer_classname") {
        if !val.is_empty() {
            w.raw(&format!("option java_outer_classname = \"{}\";", val));
            any_option = true;
        }
    }

    // csharp_namespace
    if let Some(val) = compute_namespace_option(opts, "csharp_namespace_prefix", pkg) {
        w.raw(&format!("option csharp_namespace = \"{}\";", val));
        any_option = true;
    }

    // swift_prefix
    if let Some(val) = opts.get("swift_prefix") {
        if !val.is_empty() {
            w.raw(&format!("option swift_prefix = \"{}\";", val));
            any_option = true;
        }
    }

    // php_namespace
    if let Some(val) = compute_namespace_option(opts, "php_namespace_prefix", pkg) {
        w.raw(&format!("option php_namespace = \"{}\";", val));
        any_option = true;
    }

    // ruby_package
    if let Some(val) = compute_namespace_option(opts, "ruby_package_prefix", pkg) {
        w.raw(&format!("option ruby_package = \"{}\";", val));
        any_option = true;
    }

    // objc_class_prefix
    if let Some(val) = opts.get("objc_class_prefix") {
        if !val.is_empty() {
            w.raw(&format!("option objc_class_prefix = \"{}\";", val));
            any_option = true;
        }
    }

    // optimize_for
    if let Some(val) = opts.get("optimize_for") {
        if !val.is_empty() {
            w.raw(&format!("option optimize_for = {};", val.to_uppercase()));
            any_option = true;
        }
    }

    if any_option {
        w.newline();
    }
}

/// Compute a path-style option (go_package).
/// If `<key>` is set in opts, use prefix + "/" + pkg.
/// For go_package, default to module_path/output_dir/pkg.
fn compute_go_package(
    opts: &std::collections::HashMap<String, String>,
    pkg: &str,
    module_path: &str,
    output_dir: &str,
) -> Option<String> {
    // Explicit prefix
    if let Some(prefix) = opts.get("go_package_prefix") {
        if !prefix.is_empty() {
            let prefix = prefix.trim_end_matches('/');
            return Some(format!("{}/{}", prefix, pkg));
        }
    }

    // Default: module_path/output_dir/pkg
    if !module_path.is_empty() {
        let out = output_dir
            .trim_end_matches('/')
            .trim_start_matches("./");
        let base = if out.is_empty() || out == "." {
            module_path.to_string()
        } else {
            format!("{}/{}", module_path, out)
        };
        return Some(format!("{}/{}", base, pkg));
    }

    None
}

/// Compute a dot-separated package option (java_package).
/// prefix + "." + pkg (slashes → dots).
fn compute_dot_package(
    opts: &std::collections::HashMap<String, String>,
    prefix_key: &str,
    pkg: &str,
) -> Option<String> {
    opts.get(prefix_key).and_then(|prefix| {
        if prefix.is_empty() {
            None
        } else {
            let dotted = pkg.replace('/', ".");
            Some(format!("{}.{}", prefix.trim_end_matches('.'), dotted))
        }
    })
}

/// Compute a namespace-style option (csharp, php, ruby).
/// prefix + "." + PascalCase(each segment).
fn compute_namespace_option(
    opts: &std::collections::HashMap<String, String>,
    prefix_key: &str,
    pkg: &str,
) -> Option<String> {
    opts.get(prefix_key).and_then(|prefix| {
        if prefix.is_empty() {
            None
        } else {
            let parts: Vec<String> = pkg.split('/').map(|s| to_pascal_case(s)).collect();
            Some(format!("{}.{}", prefix, parts.join(".")))
        }
    })
}

// ── Cross-package imports ─────────────────────────────────────────────

fn collect_imported_packages(
    bundle: &PkgBundle,
    current_pkg: &str,
    all_bundles: &BTreeMap<String, PkgBundle>,
) -> BTreeSet<String> {
    let mut needed = BTreeSet::new();

    for ty in &bundle.types {
        for f in &ty.fields {
            collect_type_ref_packages(f.r#type.as_ref(), current_pkg, &mut needed);
        }
        for o in &ty.oneofs {
            for f in &o.fields {
                collect_type_ref_packages(f.r#type.as_ref(), current_pkg, &mut needed);
            }
        }
        // Also scan alias underlying types (type aliases have no fields but reference other types)
        if let Some(ref trace) = ty.trace {
            if let Some(type_trace::Origin::Alias(ref alias)) = trace.origin {
                if let Some(ref underlying) = alias.underlying {
                    collect_type_ref_packages(Some(underlying), current_pkg, &mut needed);
                }
            }
        }
    }
    for svc in &bundle.services {
        for rpc in &svc.rpcs {
            if let Some(ref p) = rpc.input {
                collect_type_ref_packages(p.r#type.as_ref(), current_pkg, &mut needed);
            }
            if let Some(ref p) = rpc.output {
                collect_type_ref_packages(p.r#type.as_ref(), current_pkg, &mut needed);
            }
        }
    }

    // Only keep packages that actually exist in bundles
    needed.retain(|p| all_bundles.contains_key(p));
    needed
}

fn collect_type_ref_packages(
    tr: Option<&TypeReference>,
    current_pkg: &str,
    needed: &mut BTreeSet<String>,
) {
    let tr = match tr {
        Some(t) => t,
        None => return,
    };
    match &tr.kind {
        Some(type_reference::Kind::MessageType(m)) => {
            if let Some(pkg) = package_from_full_name(&m.full_name) {
                if pkg != current_pkg {
                    needed.insert(pkg.to_string());
                }
            }
            for f in &m.fields {
                collect_type_ref_packages(f.r#type.as_ref(), current_pkg, needed);
            }
        }
        Some(type_reference::Kind::EnumType(e)) => {
            if let Some(pkg) = package_from_full_name(&e.full_name) {
                if pkg != current_pkg {
                    needed.insert(pkg.to_string());
                }
            }
        }
        Some(type_reference::Kind::Map(m)) => {
            collect_type_ref_packages(m.key.as_deref(), current_pkg, needed);
            collect_type_ref_packages(m.value.as_deref(), current_pkg, needed);
        }
        _ => {}
    }
}

/// Extract the short package name from a full_name.
/// "github.com/org/proj/common.Address" → Some("common")
/// "common.Address" → Some("common")
fn package_from_full_name(full_name: &str) -> Option<&str> {
    if let Some(slash_pos) = full_name.rfind('/') {
        let after = &full_name[slash_pos + 1..];
        Some(after.split('.').next().unwrap_or(after))
    } else {
        // Fallback: old format
        full_name.rsplit_once('.').map(|(p, _)| p)
    }
}

// ── Proto package name ────────────────────────────────────────────────

fn proto_package(ogham_package: &str) -> String {
    ogham_package.replace('/', ".")
}

/// Apply import_prefix to a proto package name.
/// "" + "store" → "store", "proto" + "store" → "proto.store"
fn prefixed_proto_package(import_prefix: &str, pkg: &str) -> String {
    if import_prefix.is_empty() {
        proto_package(pkg)
    } else {
        format!("{}.{}", import_prefix, proto_package(pkg))
    }
}

/// Apply import_prefix to a file path.
/// "" + "common.proto" → "common.proto", "proto" + "common.proto" → "proto/common.proto"
fn prefixed_path(import_prefix: &str, path: &str) -> String {
    if import_prefix.is_empty() {
        path.to_string()
    } else {
        format!("{}/{}", import_prefix, path)
    }
}

// ── Emit helpers ──────────────────────────────────────────────────────


fn emit_enum(w: &mut CodeWriter, en: &Enum, ext_defs: &BTreeMap<String, AnnotationExtDef>) {
    w.open(&format!("enum {} {{", en.name));
    emit_options(w, &en.annotations, "enum", ext_defs);
    for val in &en.values {
        let opts = format_enum_value_options(&val.annotations, ext_defs);
        if val.is_removed && !val.fallback.is_empty() {
            w.line(&format!(
                "{} = {}{};  // removed(fallback={})",
                val.name, val.number, opts, val.fallback
            ));
        } else {
            w.line(&format!("{} = {}{};", val.name, val.number, opts));
        }
    }
    w.close("}");
}

/// Emit a wrapper message for a type alias: `type UUID = bytes;` → `message UUID { bytes value = 1; }`
fn emit_alias_wrapper(
    w: &mut CodeWriter,
    ty: &Type,
    underlying: &TypeReference,
    current_pkg: &str,
    alias_map: &HashMap<String, TypeReference>,
    ext_defs: &BTreeMap<String, AnnotationExtDef>,
) {
    w.open(&format!("message {} {{", ty.name));
    emit_options(w, &ty.annotations, "msg", ext_defs);
    let proto_type = type_ref_to_proto(Some(underlying), current_pkg, alias_map);
    w.line(&format!("{} value = 1;", proto_type));
    w.close("}");
}

/// Format enum value annotations as proto enum value options.
fn format_enum_value_options(annotations: &[AnnotationCall], ext_defs: &BTreeMap<String, AnnotationExtDef>) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for ann in annotations {
        let key = format!("{}::{}", ann.library, ann.name);
        let def = match ext_defs.get(&key) {
            Some(d) => d,
            None => continue,
        };
        let ext_ref = format!("{}_{}", def.ext_name, "enum_value");
        if ann.arguments.is_empty() {
            parts.push(format!("({}) = {{}}", ext_ref));
        } else {
            let args: Vec<String> = ann.arguments.iter().map(|a| {
                let name = if a.name.is_empty() { "value" } else { &a.name };
                let val = a.value.as_ref().map(literal_to_proto_text).unwrap_or_default();
                format!("{}: {}", name, val)
            }).collect();
            parts.push(format!("({}) = {{ {} }}", ext_ref, args.join(", ")));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(", "))
    }
}

fn emit_message(w: &mut CodeWriter, ty: &Type, current_pkg: &str, alias_map: &HashMap<String, TypeReference>, ext_defs: &BTreeMap<String, AnnotationExtDef>) {
    w.open(&format!("message {} {{", ty.name));
    emit_options(w, &ty.annotations, "msg", ext_defs);

    // Fields
    for field in &ty.fields {
        emit_field(w, field, current_pkg, alias_map, ext_defs);
    }

    // Oneofs
    for oneof in &ty.oneofs {
        w.open(&format!("oneof {} {{", oneof.name));
        // Oneof-level options are not directly supported in proto3 syntax for oneof,
        // but we can emit them as option statements inside the oneof block.
        emit_options(w, &oneof.annotations, "oneof", ext_defs);
        for field in &oneof.fields {
            let proto_type = type_ref_to_proto(field.r#type.as_ref(), current_pkg, alias_map);
            let opts = format_field_options(&field.annotations, ext_defs);
            w.line(&format!("{} {} = {}{};", proto_type, field.name, field.number, opts));
        }
        w.close("}");
    }

    // Nested enums
    for nested_enum in &ty.nested_enums {
        w.newline();
        emit_enum(w, nested_enum, ext_defs);
    }

    // Nested types
    for nested_type in &ty.nested_types {
        w.newline();
        emit_message(w, nested_type, current_pkg, alias_map, ext_defs);
    }

    w.close("}");
}

fn emit_field(w: &mut CodeWriter, field: &Field, current_pkg: &str, alias_map: &HashMap<String, TypeReference>, ext_defs: &BTreeMap<String, AnnotationExtDef>) {
    let proto_type = type_ref_to_proto(field.r#type.as_ref(), current_pkg, alias_map);
    let mut prefix = String::new();

    if field.is_repeated {
        prefix = "repeated ".to_string();
    } else if field.is_optional {
        prefix = "optional ".to_string();
    }

    // Mapping comment
    let mapping_comment = if let Some(ref mapping) = field.mapping {
        if let Some(link) = mapping.chain.first() {
            let path: Vec<&str> = link.path.iter().map(|s| s.as_str()).collect();
            format!(" // <- {}", path.join("."))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let opts = format_field_options(&field.annotations, ext_defs);
    w.line(&format!(
        "{}{} {} = {}{};{}",
        prefix, proto_type, field.name, field.number, opts, mapping_comment
    ));
}

fn emit_service(w: &mut CodeWriter, svc: &Service, current_pkg: &str, alias_map: &HashMap<String, TypeReference>, ext_defs: &BTreeMap<String, AnnotationExtDef>) {
    w.open(&format!("service {} {{", svc.name));
    emit_options(w, &svc.annotations, "svc", ext_defs);

    for rpc in &svc.rpcs {
        let input = rpc_param_to_proto(rpc.input.as_ref(), current_pkg, alias_map);
        let output = rpc_param_to_proto(rpc.output.as_ref(), current_pkg, alias_map);
        if rpc.annotations.is_empty() {
            w.line(&format!("rpc {}({}) returns ({}) {{}}", rpc.name, input, output));
        } else {
            // RPC with method-level options: use block syntax
            w.open(&format!("rpc {}({}) returns ({}) {{", rpc.name, input, output));
            emit_options(w, &rpc.annotations, "method", ext_defs);
            w.close("}");
        }
    }

    w.close("}");
}

/// Information about a collected annotation definition for proto extension generation.
#[allow(dead_code)]
struct AnnotationExtDef {
    library: String,
    name: String,
    ext_name: String,      // snake_case extension name
    msg_name: String,      // PascalCase message name
    field_number: u32,     // deterministic extension field number
    params: Vec<(String, String)>, // (param_name, proto_type) — ordered, deduped
    targets: BTreeSet<String>,     // "message", "field", "enum", etc.
}

/// Deterministic extension field number from annotation library::name.
fn annotation_ext_number(library: &str, name: &str) -> u32 {
    let key = format!("{}::{}", library, name);
    let mut hash: u32 = 2166136261; // FNV-1a offset basis
    for byte in key.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    50000 + (hash % 49999) // range 50000-99998
}

/// Determine annotation target from calling context.
/// This is set when we collect annotations from the bundle.
fn ext_name_for(library: &str, name: &str) -> String {
    if library.is_empty() {
        to_snake_case(name)
    } else {
        format!("{}_{}", to_snake_case(library), to_snake_case(name))
    }
}

fn ext_msg_name_for(library: &str, name: &str) -> String {
    if library.is_empty() {
        format!("{}Options", to_pascal_case(name))
    } else {
        format!("{}{}Options", to_pascal_case(library), to_pascal_case(name))
    }
}

/// Infer proto type from an annotation literal value.
fn infer_proto_type(lit: &AnnotationLiteral) -> String {
    use annotation_literal::Value;
    match &lit.value {
        Some(Value::StringValue(_)) => "string".to_string(),
        Some(Value::IntValue(_)) => "int64".to_string(),
        Some(Value::FloatValue(_)) => "double".to_string(),
        Some(Value::BoolValue(_)) => "bool".to_string(),
        Some(Value::StructValue(_)) => "google.protobuf.Struct".to_string(),
        Some(Value::ListValue(_)) => "google.protobuf.ListValue".to_string(),
        None => "string".to_string(),
    }
}

/// Format an annotation literal as a proto text-format value.
fn literal_to_proto_text(lit: &AnnotationLiteral) -> String {
    use annotation_literal::Value;
    match &lit.value {
        Some(Value::StringValue(s)) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Some(Value::IntValue(i)) => i.to_string(),
        Some(Value::FloatValue(f)) => f.to_string(),
        Some(Value::BoolValue(b)) => b.to_string(),
        Some(Value::StructValue(st)) => {
            let fields: Vec<String> = st.fields.iter().map(|(k, v)| {
                format!("{}: {}", k, literal_to_proto_text(v))
            }).collect();
            format!("{{ {} }}", fields.join(", "))
        }
        Some(Value::ListValue(list)) => {
            let items: Vec<String> = list.values.iter().map(literal_to_proto_text).collect();
            format!("[{}]", items.join(", "))
        }
        None => "\"\"".to_string(),
    }
}

/// Emit option statements for message/enum/service-level annotations.
fn emit_options(w: &mut CodeWriter, annotations: &[AnnotationCall], target_suffix: &str, ext_defs: &BTreeMap<String, AnnotationExtDef>) {
    for ann in annotations {
        let key = format!("{}::{}", ann.library, ann.name);
        let def = match ext_defs.get(&key) {
            Some(d) => d,
            None => continue,
        };
        let ext_ref = format!("{}_{}", def.ext_name, target_suffix);

        if ann.arguments.is_empty() {
            w.line(&format!("option ({}) = {{}};", ext_ref));
        } else {
            let args: Vec<String> = ann.arguments.iter().map(|a| {
                let name = if a.name.is_empty() { "value" } else { &a.name };
                let val = a.value.as_ref().map(literal_to_proto_text).unwrap_or_default();
                format!("{}: {}", name, val)
            }).collect();
            w.line(&format!("option ({}) = {{ {} }};", ext_ref, args.join(", ")));
        }
    }
}

/// Format field-level annotations as proto field options string.
/// Returns the options suffix like ` [(ext) = { ... }]` or empty string.
fn format_field_options(annotations: &[AnnotationCall], ext_defs: &BTreeMap<String, AnnotationExtDef>) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    let mut parts = Vec::new();
    for ann in annotations {
        let key = format!("{}::{}", ann.library, ann.name);
        let def = match ext_defs.get(&key) {
            Some(d) => d,
            None => continue,
        };
        let ext_ref = format!("{}_{}", def.ext_name, "field");

        if ann.arguments.is_empty() {
            parts.push(format!("({}) = {{}}", ext_ref));
        } else {
            let args: Vec<String> = ann.arguments.iter().map(|a| {
                let name = if a.name.is_empty() { "value" } else { &a.name };
                let val = a.value.as_ref().map(literal_to_proto_text).unwrap_or_default();
                format!("{}: {}", name, val)
            }).collect();
            parts.push(format!("({}) = {{ {} }}", ext_ref, args.join(", ")));
        }
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" [{}]", parts.join(", "))
    }
}


// ── Type conversion ───────────────────────────────────────────────────

/// Convert a TypeReference to its proto3 type name.
/// Cross-package types use fully qualified proto names (e.g., "common.Money").
/// Alias types are resolved to their underlying type.
fn type_ref_to_proto(tr: Option<&TypeReference>, current_pkg: &str, alias_map: &HashMap<String, TypeReference>) -> String {
    let tr = match tr {
        Some(t) => t,
        None => return "bytes".to_string(),
    };

    match &tr.kind {
        Some(type_reference::Kind::Scalar(s)) => scalar_to_proto(s.scalar_kind),
        Some(type_reference::Kind::MessageType(m)) => {
            qualify_name(&m.name, &m.full_name, current_pkg)
        }
        Some(type_reference::Kind::EnumType(e)) => {
            qualify_name(&e.name, &e.full_name, current_pkg)
        }
        Some(type_reference::Kind::Map(m)) => {
            let key = type_ref_to_proto(m.key.as_deref(), current_pkg, alias_map);
            let val = type_ref_to_proto(m.value.as_deref(), current_pkg, alias_map);
            format!("map<{}, {}>", key, val)
        }
        None => "bytes".to_string(),
    }
}

/// If the type is from the current package, use short name.
/// If from another package, use fully qualified proto name.
/// Ogham std types are mapped to well-known proto types.
fn qualify_name(name: &str, full_name: &str, current_pkg: &str) -> String {
    // Map ogham std types → google.protobuf well-known types
    if let Some(wkt) = well_known_proto_type(full_name) {
        return wkt.to_string();
    }

    match package_from_full_name(full_name) {
        Some(pkg) if pkg != current_pkg => {
            format!("{}.{}", proto_package(pkg), name)
        }
        _ => name.to_string(),
    }
}

/// Collect google/protobuf/*.proto imports needed by WKT references in this bundle.
fn collect_wkt_imports(bundle: &PkgBundle) -> BTreeSet<String> {
    let mut wkt_full_names = BTreeSet::new();

    let mut collect_from_ref = |tr: Option<&TypeReference>| {
        collect_wkt_from_type_ref(tr, &mut wkt_full_names);
    };

    for ty in &bundle.types {
        for f in &ty.fields {
            collect_from_ref(f.r#type.as_ref());
        }
        for o in &ty.oneofs {
            for f in &o.fields {
                collect_from_ref(f.r#type.as_ref());
            }
        }
    }
    for svc in &bundle.services {
        for rpc in &svc.rpcs {
            if let Some(ref p) = rpc.input {
                collect_from_ref(p.r#type.as_ref());
            }
            if let Some(ref p) = rpc.output {
                collect_from_ref(p.r#type.as_ref());
            }
        }
    }

    // Convert full_names to import paths, dedup
    wkt_full_names
        .iter()
        .filter_map(|name| wkt_proto_import(name))
        .map(|s| s.to_string())
        .collect()
}

fn collect_wkt_from_type_ref(tr: Option<&TypeReference>, out: &mut BTreeSet<String>) {
    let tr = match tr {
        Some(t) => t,
        None => return,
    };
    match &tr.kind {
        Some(type_reference::Kind::MessageType(m)) => {
            if well_known_proto_type(&m.full_name).is_some() {
                out.insert(m.full_name.clone());
            }
            for f in &m.fields {
                collect_wkt_from_type_ref(f.r#type.as_ref(), out);
            }
        }
        Some(type_reference::Kind::EnumType(e)) => {
            if well_known_proto_type(&e.full_name).is_some() {
                out.insert(e.full_name.clone());
            }
        }
        Some(type_reference::Kind::Map(m)) => {
            collect_wkt_from_type_ref(m.key.as_deref(), out);
            collect_wkt_from_type_ref(m.value.as_deref(), out);
        }
        _ => {}
    }
}

/// Map ogham std full_name to its proto import path.
fn wkt_proto_import(full_name: &str) -> Option<&'static str> {
    match full_name {
        s if s.starts_with("github.com/oghamlang/std/proto/struct.") => Some("google/protobuf/struct.proto"),
        "github.com/oghamlang/std/proto/timestamp.Timestamp" => Some("google/protobuf/timestamp.proto"),
        "github.com/oghamlang/std/proto/duration.Duration" => Some("google/protobuf/duration.proto"),
        s if s.starts_with("github.com/oghamlang/std/proto/wrappers.") => Some("google/protobuf/wrappers.proto"),
        s if s.starts_with("github.com/oghamlang/std/proto/any.") => Some("google/protobuf/any.proto"),
        "github.com/oghamlang/std/proto/empty.Empty" => Some("google/protobuf/empty.proto"),
        "github.com/oghamlang/std/proto/fieldmask.FieldMask" => Some("google/protobuf/field_mask.proto"),
        _ => None,
    }
}

/// Map ogham std/proto full_name to google.protobuf well-known type.
/// Only types in `std/proto/*` packages map to WKTs.
/// Full names now use import_path format: "github.com/oghamlang/std/proto/struct.Struct"
fn well_known_proto_type(full_name: &str) -> Option<&'static str> {
    match full_name {
        // struct
        "github.com/oghamlang/std/proto/struct.Struct" => Some("google.protobuf.Struct"),
        "github.com/oghamlang/std/proto/struct.Value" => Some("google.protobuf.Value"),
        "github.com/oghamlang/std/proto/struct.ListValue" => Some("google.protobuf.ListValue"),
        "github.com/oghamlang/std/proto/struct.NullValue" => Some("google.protobuf.NullValue"),
        // timestamp
        "github.com/oghamlang/std/proto/timestamp.Timestamp" => Some("google.protobuf.Timestamp"),
        // duration
        "github.com/oghamlang/std/proto/duration.Duration" => Some("google.protobuf.Duration"),
        // wrappers
        "github.com/oghamlang/std/proto/wrappers.BoolValue" => Some("google.protobuf.BoolValue"),
        "github.com/oghamlang/std/proto/wrappers.BytesValue" => Some("google.protobuf.BytesValue"),
        "github.com/oghamlang/std/proto/wrappers.DoubleValue" => Some("google.protobuf.DoubleValue"),
        "github.com/oghamlang/std/proto/wrappers.FloatValue" => Some("google.protobuf.FloatValue"),
        "github.com/oghamlang/std/proto/wrappers.Int32Value" => Some("google.protobuf.Int32Value"),
        "github.com/oghamlang/std/proto/wrappers.Int64Value" => Some("google.protobuf.Int64Value"),
        "github.com/oghamlang/std/proto/wrappers.StringValue" => Some("google.protobuf.StringValue"),
        "github.com/oghamlang/std/proto/wrappers.UInt32Value" => Some("google.protobuf.UInt32Value"),
        "github.com/oghamlang/std/proto/wrappers.UInt64Value" => Some("google.protobuf.UInt64Value"),
        // any
        "github.com/oghamlang/std/proto/any.Any" => Some("google.protobuf.Any"),
        // empty
        "github.com/oghamlang/std/proto/empty.Empty" => Some("google.protobuf.Empty"),
        // fieldmask
        "github.com/oghamlang/std/proto/fieldmask.FieldMask" => Some("google.protobuf.FieldMask"),
        _ => None,
    }
}

fn scalar_to_proto(kind: i32) -> String {
    match ScalarKind::try_from(kind) {
        Ok(ScalarKind::Bool) => "bool",
        Ok(ScalarKind::String) => "string",
        Ok(ScalarKind::Bytes) => "bytes",
        Ok(ScalarKind::Int8) | Ok(ScalarKind::Int16) | Ok(ScalarKind::Int32) => "int32",
        Ok(ScalarKind::Int64) => "int64",
        Ok(ScalarKind::Uint8) | Ok(ScalarKind::Uint16) | Ok(ScalarKind::Uint32) => "uint32",
        Ok(ScalarKind::Uint64) => "uint64",
        Ok(ScalarKind::Float) => "float",
        Ok(ScalarKind::Double) => "double",
        _ => "bytes",
    }
    .to_string()
}

fn rpc_param_to_proto(param: Option<&RpcParam>, current_pkg: &str, alias_map: &HashMap<String, TypeReference>) -> String {
    let param = match param {
        Some(p) => p,
        None => return "google.protobuf.Empty".to_string(),
    };

    if param.is_void {
        return "google.protobuf.Empty".to_string();
    }

    let mut s = String::new();
    if param.is_stream {
        s.push_str("stream ");
    }
    s.push_str(&type_ref_to_proto(param.r#type.as_ref(), current_pkg, alias_map));
    s
}


fn bundle_has_annotations(bundle: &PkgBundle) -> bool {
    bundle.types.iter().any(|t| {
        !t.annotations.is_empty()
            || t.fields.iter().any(|f| !f.annotations.is_empty())
            || t.oneofs.iter().any(|o| {
                !o.annotations.is_empty()
                    || o.fields.iter().any(|f| !f.annotations.is_empty())
            })
    }) || bundle
        .enums
        .iter()
        .any(|e| !e.annotations.is_empty() || e.values.iter().any(|v| !v.annotations.is_empty()))
        || bundle.services.iter().any(|s| {
            !s.annotations.is_empty() || s.rpcs.iter().any(|r| !r.annotations.is_empty())
        })
}


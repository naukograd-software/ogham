//! `ogham generate` — compile .ogham files and run plugins.

use crate::cli::GenerateArgs;
use ogham_compiler::ast::AstNode;
use ogham_compiler::lower;
use ogham_compiler::manifest;
use ogham_compiler::pipeline::{self, SourceFile};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Compile a project directory and return the IR module + compile result.
pub fn compile_project(
    dir: &Path,
) -> Result<(ogham_proto::ogham::ir::Module, pipeline::CompileResult), String> {
    let mod_file = manifest::load_mod_file(dir).ok();
    if let Some(ref m) = mod_file {
        eprintln!("module: {} v{}", m.module, m.version);
    }

    let schemas_dir = dir.join("schemas");
    let search_dir = if schemas_dir.is_dir() { &schemas_dir } else { dir };

    let mut sources = discover_ogham_files(search_dir)?;
    if sources.is_empty() {
        return Err(format!("no .ogham files found in {}", search_dir.display()));
    }

    // Resolve and add dependency sources
    if let Ok(deps) = ogham_compiler::pkg::resolve_deps(dir) {
        if let Ok(dep_sources) = ogham_compiler::pkg::collect_dep_sources(&deps) {
            let dep_count = dep_sources.len();
            sources.extend(dep_sources);
            if dep_count > 0 {
                eprintln!("  + {} file(s) from dependencies", dep_count);
            }
        }
    }

    eprintln!("compiling {} file(s)...", sources.len());

    let result = pipeline::compile(&sources);

    let source_pairs: Vec<(String, String)> = sources
        .iter()
        .map(|s| (s.name.clone(), s.content.clone()))
        .collect();
    ogham_compiler::diagnostics::render_diagnostics(&result.diagnostics, &source_pairs);
    ogham_compiler::diagnostics::render_summary(&result.diagnostics);

    if result.diagnostics.has_errors() {
        return Err("compilation failed".to_string());
    }

    let package = sources
        .first()
        .and_then(|s| {
            let parse = ogham_compiler::parser::parse(&s.content);
            let root = ogham_compiler::ast::Root::cast(parse.syntax())?;
            root.package_decl()
                .and_then(|p| p.name())
                .map(|t| t.text().to_string())
        })
        .unwrap_or_else(|| "default".to_string());

    let module = lower::inflate(&result.interner, &result.arenas, &result.symbols, &package);
    Ok((module, result))
}

pub fn run(args: GenerateArgs) -> Result<(), String> {
    let dir = &args.dir;
    let (module, _result) = compile_project(dir)?;
    let request_bytes = serialize_request(&module, &args)?;

    eprintln!("compiled successfully ({} bytes IR)", request_bytes.len());

    // Breaking change check (if configured)
    if !args.skip_breaking {
        run_breaking_check(dir, &module)?;
    }

    // Determine which plugins to run
    let plugins = resolve_plugins(dir, &args)?;

    if plugins.is_empty() {
        eprintln!("no plugins configured (use --plugin <name> or create ogham.gen.yaml)");
        return Ok(());
    }

    // Run each plugin in order
    for plugin in &plugins {
        let out_dir = dir.join(&plugin.out);

        let req = lower::build_request(
            module.clone(),
            env!("CARGO_PKG_VERSION"),
            plugin.opts.clone(),
            &out_dir.to_string_lossy(),
        );
        let request_with_out = {
            use prost::Message;
            let mut buf = Vec::new();
            req.encode(&mut buf).map_err(|e| format!("encode error: {}", e))?;
            buf
        };

        let plugin_name = plugin.name.as_deref()
            .or(plugin.path.as_deref())
            .unwrap_or("unknown");

        if let Some(ref grpc_addr) = plugin.grpc {
            run_plugin_grpc(plugin_name, grpc_addr, req)?;
        } else {
            run_plugin(plugin_name, plugin.path.as_deref(), &request_with_out)?;
        }
    }

    Ok(())
}

fn run_breaking_check(
    dir: &Path,
    new_module: &ogham_proto::ogham::ir::Module,
) -> Result<(), String> {
    let mod_file = match manifest::load_mod_file(dir) {
        Ok(m) => m,
        Err(_) => return Ok(()), // no mod file = no breaking check
    };

    let breaking_config = match mod_file.breaking {
        Some(b) => b,
        None => return Ok(()), // no breaking section = skip
    };

    if breaking_config.policy == "off" {
        return Ok(());
    }

    eprintln!("checking breaking changes against {}...", breaking_config.against);

    // Load old sources
    let old_sources = crate::cmd::breaking::load_reference(&breaking_config.against, dir);
    let old_sources = match old_sources {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: breaking check skipped: {}", e);
            return Ok(());
        }
    };

    let old_result = ogham_compiler::pipeline::compile(&old_sources);
    if old_result.diagnostics.has_errors() {
        eprintln!("warning: breaking check skipped: old schemas failed to compile");
        return Ok(());
    }

    let old_package = old_sources
        .first()
        .and_then(|s| {
            let parse = ogham_compiler::parser::parse(&s.content);
            use ogham_compiler::ast::AstNode;
            let root = ogham_compiler::ast::Root::cast(parse.syntax())?;
            root.package_decl()
                .and_then(|p| p.name())
                .map(|t| t.text().to_string())
        })
        .unwrap_or_else(|| "default".to_string());

    let old_module = lower::inflate(
        &old_result.interner,
        &old_result.arenas,
        &old_result.symbols,
        &old_package,
    );

    let violations = ogham_compiler::breaking::compare(&old_module, new_module);

    if violations.is_empty() {
        eprintln!("no breaking changes detected");
        return Ok(());
    }

    let mut errors = 0;
    let mut warnings = 0;

    for v in &violations {
        match v.level {
            ogham_compiler::breaking::Level::Error => {
                errors += 1;
                eprintln!("error[{}]: {}", v.code, v.message);
            }
            ogham_compiler::breaking::Level::Warning => {
                warnings += 1;
                eprintln!("warning[{}]: {}", v.code, v.message);
            }
            ogham_compiler::breaking::Level::Info => {
                eprintln!("info[{}]: {}", v.code, v.message);
            }
        }
    }

    if breaking_config.policy == "error" && (errors > 0 || warnings > 0) {
        return Err(format!(
            "breaking changes detected ({} error(s), {} warning(s)). Use --skip-breaking to override.",
            errors, warnings
        ));
    }

    Ok(())
}

fn serialize_request(
    module: &ogham_proto::ogham::ir::Module,
    _args: &GenerateArgs,
) -> Result<Vec<u8>, String> {
    use prost::Message;
    let request = lower::build_request(
        module.clone(),
        env!("CARGO_PKG_VERSION"),
        Default::default(),
        ".",
    );
    let mut buf = Vec::new();
    request.encode(&mut buf).map_err(|e| format!("failed to encode request: {}", e))?;
    Ok(buf)
}

struct ResolvedPlugin {
    name: Option<String>,
    path: Option<String>,
    grpc: Option<String>,
    out: String,
    opts: std::collections::HashMap<String, String>,
}

fn resolve_plugins(dir: &Path, args: &GenerateArgs) -> Result<Vec<ResolvedPlugin>, String> {
    // --plugin flag takes priority
    if let Some(ref plugin_name) = args.plugin {
        return Ok(vec![ResolvedPlugin {
            name: Some(plugin_name.clone()),
            path: None,
            grpc: None,
            out: ".".to_string(),
            opts: Default::default(),
        }]);
    }

    // Try ogham.gen.yaml
    match manifest::load_gen_file(dir) {
        Ok(gen) => {
            Ok(gen.generate.plugins.into_iter().map(|p| ResolvedPlugin {
                name: p.name,
                path: p.path,
                grpc: p.grpc,
                out: p.out,
                opts: p.opts,
            }).collect())
        }
        Err(_) => Ok(Vec::new()),
    }
}

pub fn discover_ogham_files(dir: &Path) -> Result<Vec<SourceFile>, String> {
    let mut sources = Vec::new();
    discover_recursive(dir, &mut sources)?;
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sources)
}

fn discover_recursive(dir: &Path, sources: &mut Vec<SourceFile>) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }

    let entries = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("read error: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.starts_with('.') || name == "vendor" || name == "node_modules" || name == "target" || name == "gen" {
                continue;
            }
            discover_recursive(&path, sources)?;
        } else if path.extension().is_some_and(|ext| ext == "ogham") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
            sources.push(SourceFile {
                name: path.to_string_lossy().to_string(),
                content,
            });
        }
    }

    Ok(())
}

/// Resolve plugin binary path.
///
/// Resolution order:
/// 1. `path:` in gen.yaml → use as-is (relative or absolute)
/// 2. `name:` → derive binary name `ogham-gen-<short>`, then search:
///    a. `$OGHAM_BIN/` (explicit binary directory)
///    b. `$OGHAM_HOME/bin/` (default: `~/.ogham/bin/`)
///    c. `$PATH` (system PATH)
fn resolve_plugin_binary(name: &str, explicit_path: Option<&str>) -> Result<String, String> {
    // 1. Explicit path
    if let Some(path) = explicit_path {
        let p = Path::new(path);
        if p.exists() {
            return Ok(path.to_string());
        }
        return Err(format!("plugin binary not found: {}", path));
    }

    // 2. Derive binary name from module path
    let short = name.rsplit('/').next().unwrap_or(name);
    let bin_name = if short.starts_with("ogham-gen-") {
        short.to_string()
    } else {
        format!("ogham-gen-{}", short)
    };

    // 2a. $OGHAM_BIN/
    if let Ok(ogham_bin) = std::env::var("OGHAM_BIN") {
        let candidate = Path::new(&ogham_bin).join(&bin_name);
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().to_string());
        }
    }

    // 2b. $OGHAM_HOME/bin/ (default: ~/.ogham/bin/)
    let ogham_home = std::env::var("OGHAM_HOME")
        .ok()
        .or_else(|| {
            dirs_fallback().map(|h| format!("{}/.ogham", h))
        });
    if let Some(home) = ogham_home {
        let candidate = Path::new(&home).join("bin").join(&bin_name);
        if candidate.exists() {
            return Ok(candidate.to_string_lossy().to_string());
        }
    }

    // 2c. $PATH — just return the name, let OS resolve it
    Ok(bin_name)
}

/// Get home directory without pulling in a crate.
fn dirs_fallback() -> Option<String> {
    std::env::var("HOME").ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
}

fn run_plugin(name: &str, explicit_path: Option<&str>, request_bytes: &[u8]) -> Result<(), String> {
    let bin_name = resolve_plugin_binary(name, explicit_path)?;

    eprintln!("running plugin: {}", bin_name);

    let mut child = Command::new(&bin_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to start {}: {} (is it installed?)", bin_name, e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(request_bytes)
            .map_err(|e| format!("failed to write to plugin stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("failed to wait for plugin: {}", e))?;

    if !output.status.success() {
        return Err(format!("plugin {} exited with {}", bin_name, output.status));
    }

    use prost::Message;
    let response = ogham_proto::ogham::compiler::OghamCompileResponse::decode(output.stdout.as_slice())
        .map_err(|e| format!("failed to decode plugin response: {}", e))?;

    for err in &response.errors {
        eprintln!("plugin {}: {}: {}", bin_name, err.severity, err.message);
    }

    for file in &response.files {
        let path = Path::new(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create dir {}: {}", parent.display(), e))?;
        }

        if file.append {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .map_err(|e| format!("cannot open {}: {}", path.display(), e))?;
            f.write_all(&file.content)
                .map_err(|e| format!("cannot write {}: {}", path.display(), e))?;
        } else {
            std::fs::write(path, &file.content)
                .map_err(|e| format!("cannot write {}: {}", path.display(), e))?;
        }

        eprintln!("  wrote {}", file.name);
    }

    eprintln!("plugin {} generated {} file(s)", bin_name, response.files.len());
    Ok(())
}

fn run_plugin_grpc(
    name: &str,
    addr: &str,
    request: ogham_proto::ogham::compiler::OghamCompileRequest,
) -> Result<(), String> {
    use ogham_proto::ogham::compiler::ogham_plugin_api_client::OghamPluginApiClient;

    let addr = if addr.starts_with("http") {
        addr.to_string()
    } else {
        format!("http://{}", addr)
    };

    eprintln!("calling plugin '{}' via gRPC at {}", name, addr);

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("runtime error: {}", e))?;

    let response = rt.block_on(async {
        let mut client = OghamPluginApiClient::connect(addr.clone())
            .await
            .map_err(|e| format!("failed to connect to {}: {}", addr, e))?;

        let resp = client
            .compile(request)
            .await
            .map_err(|e| format!("gRPC error: {}", e))?;

        Ok::<_, String>(resp.into_inner())
    })?;

    for err in &response.errors {
        eprintln!("plugin {}: {}: {}", name, err.severity, err.message);
    }

    for file in &response.files {
        let path = Path::new(&file.name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create dir {}: {}", parent.display(), e))?;
        }
        std::fs::write(path, &file.content)
            .map_err(|e| format!("cannot write {}: {}", path.display(), e))?;
        eprintln!("  wrote {}", file.name);
    }

    eprintln!("plugin {} (gRPC) generated {} file(s)", name, response.files.len());
    Ok(())
}

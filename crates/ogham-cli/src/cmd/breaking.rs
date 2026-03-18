//! `ogham breaking` — detect breaking changes against a reference.

use crate::cli::BreakingArgs;
use crate::cmd::generate::compile_project;
use ogham_compiler::breaking::{self, Level};
use ogham_compiler::lower;
use ogham_compiler::pipeline::SourceFile;
use std::path::Path;
use std::process::Command;

pub fn run(args: BreakingArgs) -> Result<(), String> {
    // Compile current schemas
    let dir = Path::new(".");
    let (new_module, _, _) = compile_project(dir)?;

    // Load and compile old schemas
    let old_sources = load_reference(&args.against, dir)?;
    let old_result = ogham_compiler::pipeline::compile(&old_sources);

    if old_result.diagnostics.has_errors() {
        return Err(format!(
            "failed to compile reference '{}': {} error(s)",
            args.against,
            old_result.diagnostics.errors().count()
        ));
    }

    let old_package = old_sources
        .first()
        .and_then(|s| {
            let parse = ogham_compiler::parser::parse(&s.content);
            let root = ogham_compiler::ast::Root::cast(parse.syntax())?;
            use ogham_compiler::ast::AstNode;
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
        None,
    );

    // Compare
    let violations = breaking::compare(&old_module, &new_module);

    if violations.is_empty() {
        eprintln!("no breaking changes detected");
        return Ok(());
    }

    // Render violations
    let mut errors = 0;
    let mut warnings = 0;
    let mut infos = 0;

    for v in &violations {
        match v.level {
            Level::Error => {
                errors += 1;
                eprintln!("error[{}]: {}", v.code, v.message);
            }
            Level::Warning => {
                warnings += 1;
                eprintln!("warning[{}]: {}", v.code, v.message);
            }
            Level::Info => {
                infos += 1;
                eprintln!("info[{}]: {}", v.code, v.message);
            }
        }
    }

    eprintln!();
    eprintln!(
        "{} error(s), {} warning(s), {} info(s)",
        errors, warnings, infos
    );

    // Determine exit code based on flags
    if args.force {
        // Nothing blocks
        Ok(())
    } else if args.allow {
        // Only errors block
        if errors > 0 {
            Err(format!("{} breaking change(s) detected", errors))
        } else {
            Ok(())
        }
    } else {
        // Errors and warnings block
        if errors > 0 || warnings > 0 {
            Err(format!(
                "{} breaking change(s) detected (use --allow to ignore warnings, --force to ignore all)",
                errors + warnings
            ))
        } else {
            Ok(())
        }
    }
}

pub fn load_reference(against: &str, project_dir: &Path) -> Result<Vec<SourceFile>, String> {
    if let Some(git_ref) = against.strip_prefix("git:") {
        load_from_git(git_ref, project_dir)
    } else if against.starts_with("./") || against.starts_with('/') {
        load_from_dir(Path::new(against))
    } else {
        Err(format!(
            "unsupported reference format: '{}'. Use git:<ref> or ./path/",
            against
        ))
    }
}

fn load_from_git(git_ref: &str, project_dir: &Path) -> Result<Vec<SourceFile>, String> {
    // Find .ogham files in the git ref
    let schemas_dir = project_dir.join("schemas");
    let search_prefix = if schemas_dir.is_dir() {
        "schemas/"
    } else {
        ""
    };

    // List .ogham files at the ref
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", git_ref])
        .output()
        .map_err(|e| format!("git error: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git ref '{}' not found: {}",
            git_ref,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let file_list = String::from_utf8_lossy(&output.stdout);
    let ogham_files: Vec<&str> = file_list
        .lines()
        .filter(|l| l.ends_with(".ogham"))
        .filter(|l| search_prefix.is_empty() || l.starts_with(search_prefix))
        .collect();

    let mut sources = Vec::new();
    for file_path in ogham_files {
        let output = Command::new("git")
            .args(["show", &format!("{}:{}", git_ref, file_path)])
            .output()
            .map_err(|e| format!("git show error: {}", e))?;

        if output.status.success() {
            sources.push(SourceFile {
                name: file_path.to_string(),
                content: String::from_utf8_lossy(&output.stdout).to_string(),
            });
        }
    }

    if sources.is_empty() {
        return Err(format!("no .ogham files found at git ref '{}'", git_ref));
    }

    Ok(sources)
}

fn load_from_dir(dir: &Path) -> Result<Vec<SourceFile>, String> {
    // Reuse discover logic
    let schemas_dir = dir.join("schemas");
    let search_dir = if schemas_dir.is_dir() { &schemas_dir } else { dir };
    crate::cmd::generate::discover_ogham_files(search_dir)
}

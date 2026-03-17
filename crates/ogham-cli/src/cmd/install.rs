pub fn run() -> Result<(), String> {
    let dir = std::path::Path::new(".");
    eprintln!("installing dependencies...");
    let deps = ogham_compiler::pkg::install(dir)?;

    // Integrity check
    let warnings = ogham_compiler::pkg::check_integrity(&deps);
    for w in &warnings {
        eprintln!("warning: {}", w);
    }

    // Write lock file
    ogham_compiler::pkg::write_lock_file(dir, &deps)?;

    eprintln!("{} dependency(ies) resolved", deps.len());
    Ok(())
}

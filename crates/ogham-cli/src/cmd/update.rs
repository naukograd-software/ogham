pub fn run() -> Result<(), String> {
    let dir = std::path::Path::new(".");
    eprintln!("updating dependencies...");
    let deps = ogham_compiler::pkg::update_deps(dir)?;
    for dep in &deps {
        eprintln!("  {} v{}", dep.module, dep.version);
    }
    eprintln!("{} dependency(ies) updated", deps.len());
    Ok(())
}

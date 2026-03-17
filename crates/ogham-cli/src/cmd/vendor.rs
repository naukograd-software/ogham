pub fn run() -> Result<(), String> {
    let dir = std::path::Path::new(".");
    eprintln!("vendoring dependencies...");
    ogham_compiler::pkg::vendor(dir)?;
    eprintln!("done");
    Ok(())
}

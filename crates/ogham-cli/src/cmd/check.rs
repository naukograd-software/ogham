//! `ogham check` — validate schemas, compile without running plugins.

use crate::cli::CheckArgs;
use crate::cmd::generate::compile_project;

pub fn run(args: CheckArgs) -> Result<(), String> {
    let (_module, _result, _) = compile_project(&args.dir)?;
    eprintln!("check passed");
    Ok(())
}

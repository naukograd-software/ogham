//! `ogham dump` — compile and dump IR as JSON for debugging.

use crate::cli::DumpArgs;
use crate::cmd::generate::compile_project;

pub fn run(args: DumpArgs) -> Result<(), String> {
    let (module, _, _) = compile_project(&args.dir)?;

    let json = serde_json::to_string_pretty(&module)
        .map_err(|e| format!("failed to serialize IR: {}", e))?;

    if let Some(ref output) = args.output {
        std::fs::write(output, &json)
            .map_err(|e| format!("cannot write {}: {}", output.display(), e))?;
        eprintln!("IR dumped to {}", output.display());
    } else {
        println!("{}", json);
    }

    Ok(())
}

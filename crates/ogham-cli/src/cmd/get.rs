use crate::cli::GetArgs;

pub fn run(args: GetArgs) -> Result<(), String> {
    let dir = std::path::Path::new(".");

    // Auto-detect source for known hosts
    let _source = ogham_compiler::pkg::auto_detect_source(&args.dependency);

    ogham_compiler::pkg::add_dependency(dir, &args.dependency)
}

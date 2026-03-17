//! Ogham Plugin SDK for Rust.
//!
//! Build code generation plugins for the Ogham schema language.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use oghamgen::{run, CompileRequest, CompileResponse, GeneratedFile};
//!
//! fn main() {
//!     run(|req: CompileRequest| {
//!         let mut resp = CompileResponse::default();
//!
//!         if let Some(module) = &req.module {
//!             for ty in &module.types {
//!                 let code = format!("// Generated from {}\n", ty.name);
//!                 resp.files.push(GeneratedFile {
//!                     name: format!("{}.rs", ty.name.to_lowercase()),
//!                     content: code.into_bytes(),
//!                     append: false,
//!                 });
//!             }
//!         }
//!
//!         Ok(resp)
//!     });
//! }
//! ```

mod runner;
mod codegen;

// Re-export proto types for plugin authors
pub use ogham_proto::oghamproto::ir::*;
pub use ogham_proto::oghamproto::compiler::{
    OghamCompileRequest as CompileRequest,
    OghamCompileResponse as CompileResponse,
    GeneratedFile,
    CompileError,
    Severity,
};

// Re-export SDK utilities
pub use runner::run;
pub use codegen::{
    CodeWriter,
    to_pascal_case, to_snake_case, to_camel_case, to_screaming_snake_case,
};

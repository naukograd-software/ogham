//! Ogham compiler: lexer, parser, type checker, linter, and AST → IR lowering.

pub mod lexer;
pub mod syntax_kind;
pub mod parser;
pub mod ast;
pub mod diagnostics;
pub mod hir;
pub mod index;
pub mod manifest;
pub mod resolve;
pub mod stdlib;
pub mod lower;
pub mod breaking;
pub mod pkg;
pub mod pipeline;

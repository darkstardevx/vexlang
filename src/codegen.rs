//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

use crate::ast::Stmt;

/// QBE code generation is intentionally disabled until the interpreter AST is stable.
pub fn generate_qbe(_stmts: &[Stmt]) -> Result<String, String> {
    Err("QBE code generation is not implemented yet".into())
}

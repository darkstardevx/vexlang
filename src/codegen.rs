//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

use crate::ast::Op;
use crate::ast::{Expr, Stmt};

pub fn generate_qbe(stmts: &[Stmt]) {
    println!("# QBE Target output");
    // You need this data section for printf to function
    println!("data $fmt_int = {{ b \"%d\\n\", b 0 }}");

    println!("export function w $main() {{");
    println!("@start");

    for stmt in stmts {
        match stmt {
            Stmt::Let { name, value, .. } => {
                println!("    %{name} =w copy {}", qbe_expr(value));
            }
            Stmt::ExprStmt(expr) => {
                // This calls printf
                println!("    call $printf(l $fmt_int, w {})", qbe_expr(expr));
            }
            _ => todo!(),
        }
    }
    println!("    ret 0");
    println!("}}");
}

fn qbe_expr(expr: &Expr) -> String {
    match expr {
        Expr::Int(val) => { /* ... */ }
        Expr::Var(name) => { /* ... */ }
        Expr::BinaryOp(lhs, op, rhs) => { /* ... */ }
        Expr::UnaryOp(op, rhs) => {
            // Implement your logic for Unary Operations here
            // e.g., generating "neg" in QBE
        }
    }
}

//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

use crate::ast::{Stmt, Type};
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            self.analyze_stmt(stmt)?;
        }
        Ok(())
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, ty, value: _ } => {
                let var_type = ty.clone();
                self.symbols.insert(name.clone(), var_type);
                eprintln!("Analyzer: Registered variable '{}'", name);
                Ok(())
            }

            Stmt::ExprStmt(_expr) => {
                // Rename to _expr
                eprintln!("Analyzer: Validating expression statement...");
                Ok(())
            }

            _ => todo!("Implement semantic analysis for other statements"),
        }
    }
}

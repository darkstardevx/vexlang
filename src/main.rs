//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md

#![allow(dead_code)]
#![allow(deprecated)]

mod analyzer;
mod ast;
mod builder;
mod codegen;
mod evaluator;
mod macros;
mod parser;

// Bring the modules into scope
use crate::analyzer::SemanticAnalyzer;
use crate::builder::build_ast;
use crate::parser::parse_vex;

fn main() {
    let source = "let x = 10 + 5;";

    // 1. Parse and build the AST
    let pairs = parse_vex(source).expect("Failed to parse");

    // Build the statements (this replaces the undefined 'ast_data')
    let stmts = build_ast(pairs);

    // Create the program struct using the parsed statements
    let program = ast::Program {
        stmts: stmts.clone(),
    };

    // 2. Evaluate
    // We clone stmts here because we need to reuse them for Analysis/Codegen below
    match evaluator::eval(program.stmts.clone()) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => eprintln!("Runtime Error: {:?}", e),
    }

    // 3. Log status after data is ready
    note!("AST Generation complete. Nodes: {}", stmts.len());

    // 4. Analyze and Codegen
    let mut analyzer = SemanticAnalyzer::new();

    // Pass the statements to the analyzer
    if analyzer.analyze(&stmts).is_ok() {
        eprintln!("Analysis successful! Generating QBE...");
        crate::codegen::generate_qbe(&stmts);
    }
}

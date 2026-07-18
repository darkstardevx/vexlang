//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

use pest::iterators::Pair;
use pest::iterators::Pairs;

use crate::ast::{Expr, Op as AstOp};
use crate::parser::Rule;
use pest::pratt_parser::{Assoc, Op, PrattParser};

lazy_static::lazy_static! {
    static ref PRATT: PrattParser<Rule> = {
        PrattParser::new()
        .op(Op::infix(Rule::eq, Assoc::Left) | Op::infix(Rule::lt, Assoc::Left) | Op::infix(Rule::gt, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::prefix(Rule::sub)) // Unary minus
    };
}

pub fn build_ast(pairs: Pairs<Rule>) -> Vec<Stmt> {
    let mut stmts = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            // Match your top-level rules (e.g., Rule::stmt)
            Rule::stmt => stmts.push(build_stmt(pair)),
            _ => {} // Ignore whitespace or other trivia
        }
    }
    stmts
}

pub fn build_stmt(pair: Pair<Rule>) -> Stmt {
    // Unwrap the 'stmt' wrapper to get the specific statement type (e.g., let_decl)
    let inner = pair
        .into_inner()
        .next()
        .expect("Stmt should have an inner variant");

    match inner.as_rule() {
        Rule::let_decl => {
            let mut inner_tokens = inner.into_inner();
            let name = inner_tokens.next().unwrap().as_str().to_string();

            let mut ty = None;
            let next = inner_tokens.next().expect("Expected type or expr");

            let value = if next.as_rule() == Rule::type_name {
                ty = Some(Type::Custom(next.as_str().to_string()));
                build_expr(inner_tokens.next().unwrap())
            } else {
                build_expr(next)
            };

            Stmt::Let {
                name,
                ty: ty.unwrap_or(Type::Custom("Unknown".to_string())),
                value,
            }
        }

        Rule::expr_stmt => {
            let inner = inner.into_inner().next().unwrap();
            Stmt::ExprStmt(build_expr(inner))
        }

        _ => todo!("Implement lowering for {:?}", inner.as_rule()),
    }
}

pub fn build_expr(pairs: pest::iterators::Pairs<Rule>) -> Expr {
    PRATT
        .map_primary(|primary| match primary.as_rule() {
            Rule::int => Expr::Int(primary.as_str().parse().unwrap()),
            Rule::ident => Expr::Var(primary.as_str().to_string()),
            Rule::expr => build_expr(primary.into_inner()), // This correctly recurses
            _ => unreachable!(),
        })
        .map_infix(|lhs, op, rhs| {
            let op_type = match op.as_rule() {
                Rule::add => AstOp::Add,
                Rule::sub => AstOp::Sub,
                Rule::mul => AstOp::Mul,
                Rule::div => AstOp::Div,
                Rule::eq => AstOp::Eq,
                Rule::lt => AstOp::Lt,
                Rule::gt => AstOp::Gt,
                _ => unreachable!(),
            };
            Expr::BinaryOp(Box::new(lhs), op_type, Box::new(rhs))
        })
        .map_prefix(|op, rhs| {
            let op_type = match op.as_rule() {
                Rule::sub => AstOp::Sub,
                _ => unreachable!(),
            };
            Expr::UnaryOp(op_type, Box::new(rhs))
        })
        .parse(pairs)
}

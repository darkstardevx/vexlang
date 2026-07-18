//! ## Vex [Vexlang] ##
//! # Version: 0.0.1 (SemVer)
//! # Inspired by Rust & Zig
//! # Built for Cybercore ecosystem code consistency and ease
//! # GitHub: [Vex] (https://github.com/darkstardevx/vex)
//! # Tag reference doc in /home/raven/devspace/docs/tags/TAG_API.md
//!

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    U64,
    F64,
    Bool,
    Res, // The unique Vex resource type
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)] // Add PartialEq here
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Var(String),
    BinaryOp(Box<Expr>, Op, Box<Expr>), // The core of arithmetic
    UnaryOp(Op, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Type,
        value: Expr,
    },
    ExprStmt(Expr),
    FnDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    LayoutDecl {
        name: String,
        fields: Vec<String>,
    },
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    // Add others as you grow (Float, String, etc.)
}

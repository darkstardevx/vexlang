//! Typed, backend-independent intermediate representation.
//!
//! This is deliberately a small textual IR rather than QBE.  It gives the
//! compiler pipeline a checked lowering target without claiming that a
//! machine-code backend exists.

use crate::ast::{Expr, Op, Stmt, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    I32,
    U64,
    Bool,
    String,
    Unit,
}

impl std::fmt::Display for IrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::I32 => "i32",
            Self::U64 => "u64",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Unit => "unit",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrExpr {
    pub ty: IrType,
    pub kind: IrExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrExprKind {
    Int(i64),
    Bool(bool),
    String(String),
    Var(String),
    Binary(Box<IrExpr>, Op, Box<IrExpr>),
    Unary(Op, Box<IrExpr>),
    Block(Vec<IrStmt>, Option<Box<IrExpr>>),
    If {
        condition: Box<IrExpr>,
        then_branch: Box<IrExpr>,
        else_branch: Option<Box<IrExpr>>,
    },
    Call(String, Vec<IrExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrStmt {
    Function(IrFunction),
    Let {
        name: String,
        ty: IrType,
        value: IrExpr,
    },
    Assign {
        name: String,
        value: IrExpr,
    },
    Expr(IrExpr),
    If {
        condition: IrExpr,
        then_branch: Box<IrExpr>,
        else_branch: Option<Box<IrExpr>>,
    },
    While {
        condition: IrExpr,
        body: Box<IrExpr>,
    },
    Break,
    Continue,
    Return(Option<IrExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub return_type: IrType,
    pub body: IrExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub body: Vec<IrStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LowerError {
    Unsupported(String),
    Invalid(String),
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(message) | Self::Invalid(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for LowerError {}

pub fn lower(stmts: &[Stmt]) -> Result<IrProgram, LowerError> {
    let mut functions = HashMap::new();
    for stmt in stmts {
        if let Stmt::Function {
            name,
            params,
            return_type,
            ..
        } = stmt
        {
            functions.insert(
                name.clone(),
                (
                    params
                        .iter()
                        .map(|(_, ty)| ir_type(ty))
                        .collect::<Result<Vec<_>, _>>()?,
                    ir_type(return_type)?,
                ),
            );
        }
    }
    let mut body = Vec::new();
    let mut scope = HashMap::new();
    for stmt in stmts {
        body.push(lower_stmt(stmt, &functions, &mut scope)?);
    }
    Ok(IrProgram {
        functions: body
            .iter()
            .filter_map(|stmt| match stmt {
                IrStmt::Function(function) => Some(function.clone()),
                _ => None,
            })
            .collect(),
        body,
    })
}

fn ir_type(ty: &Type) -> Result<IrType, LowerError> {
    match ty {
        Type::I32 | Type::Inferred => Ok(IrType::I32),
        Type::U64 => Ok(IrType::U64),
        Type::Bool => Ok(IrType::Bool),
        Type::String => Ok(IrType::String),
        Type::Unit => Ok(IrType::Unit),
        other => Err(LowerError::Unsupported(format!(
            "IR lowering does not support type annotation {other:?}"
        ))),
    }
}

type Functions = HashMap<String, (Vec<IrType>, IrType)>;
type Scope = HashMap<String, IrType>;

fn lower_stmt(stmt: &Stmt, functions: &Functions, scope: &mut Scope) -> Result<IrStmt, LowerError> {
    match stmt {
        Stmt::Function {
            name,
            params,
            return_type,
            body,
        } => {
            let mut local = params
                .iter()
                .map(|(name, ty)| Ok((name.clone(), ir_type(ty)?)))
                .collect::<Result<Scope, LowerError>>()?;
            Ok(IrStmt::Function(IrFunction {
                name: name.clone(),
                params: params
                    .iter()
                    .map(|(name, ty)| Ok((name.clone(), ir_type(ty)?)))
                    .collect::<Result<Vec<_>, LowerError>>()?,
                return_type: ir_type(return_type)?,
                body: lower_expr(body, functions, &mut local)?,
            }))
        }
        Stmt::Let { name, ty, value } => {
            let value = lower_expr(value, functions, scope)?;
            let ty = if *ty == Type::Inferred {
                value.ty.clone()
            } else {
                ir_type(ty)?
            };
            scope.insert(name.clone(), ty.clone());
            Ok(IrStmt::Let {
                name: name.clone(),
                ty,
                value,
            })
        }
        Stmt::Assign { name, value } => Ok(IrStmt::Assign {
            name: name.clone(),
            value: lower_expr(value, functions, scope)?,
        }),
        Stmt::ExprStmt(expr) => Ok(IrStmt::Expr(lower_expr(expr, functions, scope)?)),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => Ok(IrStmt::If {
            condition: lower_expr(condition, functions, scope)?,
            then_branch: Box::new(lower_expr(then_branch, functions, scope)?),
            else_branch: else_branch
                .as_ref()
                .map(|expr| lower_expr(expr, functions, scope))
                .transpose()?
                .map(Box::new),
        }),
        Stmt::While { condition, body } => Ok(IrStmt::While {
            condition: lower_expr(condition, functions, scope)?,
            body: Box::new(lower_expr(body, functions, scope)?),
        }),
        Stmt::Break => Ok(IrStmt::Break),
        Stmt::Continue => Ok(IrStmt::Continue),
        Stmt::Return(expr) => Ok(IrStmt::Return(
            expr.as_ref()
                .map(|expr| lower_expr(expr, functions, scope))
                .transpose()?,
        )),
    }
}

fn lower_expr(expr: &Expr, functions: &Functions, scope: &mut Scope) -> Result<IrExpr, LowerError> {
    let (ty, kind) = match expr {
        Expr::Int(value) => (IrType::I32, IrExprKind::Int(*value)),
        Expr::Bool(value) => (IrType::Bool, IrExprKind::Bool(*value)),
        Expr::String(value) => (IrType::String, IrExprKind::String(value.clone())),
        Expr::Var(name) => (
            scope
                .get(name)
                .cloned()
                .ok_or_else(|| LowerError::Invalid(format!("undefined variable `{name}`")))?,
            IrExprKind::Var(name.clone()),
        ),
        Expr::BinaryOp(left, op, right) => {
            let left = lower_expr(left, functions, scope)?;
            let right = lower_expr(right, functions, scope)?;
            let ty = if matches!(op, Op::Eq | Op::Lt | Op::Gt | Op::And | Op::Or) {
                IrType::Bool
            } else {
                left.ty.clone()
            };
            (
                ty,
                IrExprKind::Binary(Box::new(left), op.clone(), Box::new(right)),
            )
        }
        Expr::UnaryOp(op, value) => {
            let value = lower_expr(value, functions, scope)?;
            let ty = if *op == Op::Not {
                IrType::Bool
            } else {
                value.ty.clone()
            };
            (ty, IrExprKind::Unary(op.clone(), Box::new(value)))
        }
        Expr::Block(stmts, tail) => {
            let mut local = scope.clone();
            let lowered = stmts
                .iter()
                .map(|stmt| lower_stmt(stmt, functions, &mut local))
                .collect::<Result<Vec<_>, _>>()?;
            let tail = tail
                .as_ref()
                .map(|expr| lower_expr(expr, functions, &mut local))
                .transpose()?;
            let ty = tail.as_ref().map_or(IrType::Unit, |expr| expr.ty.clone());
            (ty, IrExprKind::Block(lowered, tail.map(Box::new)))
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = lower_expr(condition, functions, scope)?;
            let then_branch = lower_expr(then_branch, functions, scope)?;
            let else_branch = else_branch
                .as_ref()
                .map(|expr| lower_expr(expr, functions, scope))
                .transpose()?;
            let ty = else_branch
                .as_ref()
                .map_or(IrType::Unit, |expr| expr.ty.clone());
            (
                ty,
                IrExprKind::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                },
            )
        }
        Expr::Call(name, args) => {
            let args = args
                .iter()
                .map(|arg| lower_expr(arg, functions, scope))
                .collect::<Result<Vec<_>, _>>()?;
            let ty = match name.as_str() {
                "print" | "println" => IrType::Unit,
                "i32" => IrType::I32,
                "u64" => IrType::U64,
                _ => functions
                    .get(name)
                    .map(|(_, ty)| ty.clone())
                    .ok_or_else(|| LowerError::Invalid(format!("undefined function `{name}`")))?,
            };
            (ty, IrExprKind::Call(name.clone(), args))
        }
    };
    Ok(IrExpr { ty, kind })
}

/// Render a stable, human-readable artifact for reviews and golden tests.
pub fn render(program: &IrProgram) -> String {
    let mut out = String::new();
    for stmt in &program.body {
        render_stmt(stmt, 0, &mut out);
    }
    out
}

fn render_stmt(stmt: &IrStmt, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match stmt {
        IrStmt::Function(function) => {
            let params = function
                .params
                .iter()
                .map(|(name, ty)| format!("{name}: {ty}"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "{pad}fn {}({params}) -> {} {{\n",
                function.name, function.return_type
            ));
            render_expr(&function.body, indent + 1, out);
            out.push_str(&format!("{pad}}}\n"));
        }
        IrStmt::Let { name, ty, value } => {
            out.push_str(&format!("{pad}let {name}: {ty} = "));
            render_expr(value, 0, out);
            out.push_str(";\n");
        }
        IrStmt::Assign { name, value } => {
            out.push_str(&format!("{pad}{name} = "));
            render_expr(value, 0, out);
            out.push_str(";\n");
        }
        IrStmt::Expr(expr) => {
            out.push_str(&pad);
            render_expr(expr, 0, out);
            out.push_str(";\n");
        }
        IrStmt::If { condition, .. } => {
            out.push_str(&format!("{pad}if {} ...\n", display_expr(condition)))
        }
        IrStmt::While { condition, .. } => {
            out.push_str(&format!("{pad}while {} ...\n", display_expr(condition)))
        }
        IrStmt::Break => out.push_str(&format!("{pad}break;\n")),
        IrStmt::Continue => out.push_str(&format!("{pad}continue;\n")),
        IrStmt::Return(expr) => {
            out.push_str(&format!("{pad}return"));
            if let Some(expr) = expr {
                out.push(' ');
                render_expr(expr, 0, out);
            }
            out.push_str(";\n");
        }
    }
}

fn render_expr(expr: &IrExpr, _indent: usize, out: &mut String) {
    out.push_str(&format!("[{}]{}", expr.ty, display_expr(expr)));
}

fn display_expr(expr: &IrExpr) -> String {
    match &expr.kind {
        IrExprKind::Int(value) => value.to_string(),
        IrExprKind::Bool(value) => value.to_string(),
        IrExprKind::String(value) => format!("{value:?}"),
        IrExprKind::Var(name) => name.clone(),
        IrExprKind::Binary(left, op, right) => {
            format!("({} {op:?} {})", display_expr(left), display_expr(right))
        }
        IrExprKind::Unary(op, value) => format!("({op:?}{})", display_expr(value)),
        IrExprKind::Block(_, _) => "{ ... }".into(),
        IrExprKind::If { .. } => "if ...".into(),
        IrExprKind::Call(name, args) => format!("{name}({} args)", args.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::{lower, render};
    use crate::{ast::Value, builder::build_ast, evaluator, parser::parse_vex};

    fn compile(source: &str) -> String {
        let pairs = parse_vex(source).unwrap();
        render(&lower(&build_ast(pairs).unwrap()).unwrap())
    }

    #[test]
    fn lowers_typed_expressions_and_control_flow() {
        let ir = compile("let x: i32 = 1 + 2; while x < 3 { break; } x;");
        assert!(ir.contains("let x: i32"));
        assert!(ir.contains("[i32](1 Add 2)"));
        assert!(ir.contains("while"));
    }

    #[test]
    fn lowers_functions() {
        let ir = compile("fn add(a: i32, b: i32) -> i32 { a + b } add(1, 2);");
        assert!(ir.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(ir.contains("[i32]add(2 args)"));
    }

    #[test]
    fn lowering_agrees_with_interpreter_for_supported_programs() {
        let source = "fn inc(value: i32) -> i32 { value + 1 } let x: i32 = 2; while x < 4 { x = inc(x); } x;";
        let pairs = parse_vex(source).unwrap();
        let ast = build_ast(pairs).unwrap();
        let program = lower(&ast).unwrap();
        assert_eq!(evaluator::eval(&ast), Ok(Value::Int(4)));
        assert!(render(&program).contains("while"));
        assert!(render(&program).contains("fn inc(value: i32) -> i32"));
    }
}

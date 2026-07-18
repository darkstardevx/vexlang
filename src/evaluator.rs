use crate::ast::{Expr, Stmt, Value};

#[derive(Debug)]
pub enum EvalError {
    RuntimeError(String),
}

pub fn eval(stmts: Vec<Stmt>) -> Result<Value, EvalError> {
    let mut last_value = Value::Int(0);

    for stmt in stmts {
        last_value = eval_stmt(stmt)?;
    }

    Ok(last_value)
}

fn eval_stmt(stmt: Stmt) -> Result<Value, EvalError> {
    match stmt {
        Stmt::ExprStmt(expr) => eval_expr(expr),
        // Stmt::If { .. } => todo!("Implement If"),
        _ => Err(EvalError::RuntimeError(
            "Statement type not implemented yet".to_string(),
        )),
    }
}

fn eval_expr(expr: Expr) -> Result<Value, EvalError> {
    match expr {
        Expr::Int(val) => Ok(Value::Int(val)),
        // Expr::Var(name) => // You'll need an environment/scope here eventually
        _ => Err(EvalError::RuntimeError(
            "Expression type not implemented yet".to_string(),
        )),
    }
}

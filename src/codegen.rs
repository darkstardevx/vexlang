//! Backend contracts and deliberately non-native emission.
//!
//! This module is the target boundary.  A backend must accept only the
//! `FIRST_TARGET` subset; it must never infer a runtime representation for
//! values outside that contract.

use crate::ir::{self, IrExpr, IrExprKind, IrProgram, IrStmt, IrType};

/// The first native-target contract.  It is intentionally target-independent:
/// a future QBE or machine backend can implement this contract without
/// changing language semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetContract {
    pub name: &'static str,
    pub entry_point: &'static str,
    pub calling_convention: &'static str,
    pub overflow: &'static str,
    pub division: &'static str,
    pub control_flow: &'static str,
    pub runtime_boundary: &'static str,
}

pub const FIRST_TARGET: TargetContract = TargetContract {
    name: "vex-scalar-v1",
    entry_point: "vex_main",
    calling_convention: "vex_main() -> i32; internal scalar functions pass i32/bool left-to-right and return one scalar",
    overflow: "i32 add/sub/mul/neg trap on overflow; no wrapping is permitted",
    division: "i32 division traps on zero and on MIN / -1",
    control_flow: "if/while are structured; break exits the innermost loop, continue jumps to its condition",
    runtime_boundary: "no I/O, allocation, or external calls; traps are runtime-owned and entry has no arguments",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    pub code: &'static str,
    pub backend: &'static str,
    pub message: String,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]: {}", self.code, self.backend, self.message)
    }
}

impl std::error::Error for BackendError {}

/// Validate that an IR program is representable by the first target.
pub fn validate_first_target(program: &IrProgram) -> Result<(), BackendError> {
    ir::verify(program).map_err(|error| BackendError {
        code: error.code,
        backend: FIRST_TARGET.name,
        message: error.to_string(),
    })?;
    for (index, stmt) in program.body.iter().enumerate() {
        validate_stmt(stmt, &format!("body[{index}]"))?;
    }
    Ok(())
}

fn validate_type(ty: &IrType, path: &str) -> Result<(), BackendError> {
    match ty {
        IrType::I32 | IrType::Bool | IrType::Unit => Ok(()),
        _ => Err(unsupported(
            path,
            format!("type `{ty}` is outside the scalar target"),
        )),
    }
}

fn validate_stmt(stmt: &IrStmt, path: &str) -> Result<(), BackendError> {
    match stmt {
        IrStmt::Function(function) => {
            validate_type(&function.return_type, &format!("{path}.return"))?;
            for (name, ty) in &function.params {
                validate_type(ty, &format!("{path}.param.{name}"))?;
            }
            validate_expr(&function.body, &format!("{path}.body"))
        }
        IrStmt::Let { ty, value, .. } => {
            validate_type(ty, &format!("{path}.type"))?;
            validate_expr(value, &format!("{path}.value"))
        }
        IrStmt::Assign { value, .. } => validate_expr(value, &format!("{path}.value")),
        IrStmt::Expr(expr) => validate_expr(expr, &format!("{path}.expr")),
        IrStmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            validate_expr(condition, &format!("{path}.condition"))?;
            validate_expr(then_branch, &format!("{path}.then"))?;
            if let Some(branch) = else_branch {
                validate_expr(branch, &format!("{path}.else"))?;
            }
            Ok(())
        }
        IrStmt::While { condition, body } => {
            validate_expr(condition, &format!("{path}.condition"))?;
            validate_expr(body, &format!("{path}.body"))
        }
        IrStmt::Break | IrStmt::Continue => Ok(()),
        IrStmt::Return(value) => value
            .as_ref()
            .map(|expr| validate_expr(expr, &format!("{path}.value")))
            .unwrap_or(Ok(())),
        IrStmt::Record { .. } | IrStmt::Enum { .. } | IrStmt::AssignIndex { .. } => {
            Err(unsupported(
                path,
                "aggregate values and indexed mutation are not in the first target",
            ))
        }
    }
}

fn validate_expr(expr: &IrExpr, path: &str) -> Result<(), BackendError> {
    validate_type(&expr.ty, &format!("{path}.type"))?;
    match &expr.kind {
        IrExprKind::Int(_) | IrExprKind::Bool(_) | IrExprKind::Var(_) => Ok(()),
        IrExprKind::Binary(left, _, right) => {
            validate_expr(left, &format!("{path}.left"))?;
            validate_expr(right, &format!("{path}.right"))
        }
        IrExprKind::Unary(_, value) => validate_expr(value, &format!("{path}.value")),
        IrExprKind::Block(stmts, tail) => {
            for (index, stmt) in stmts.iter().enumerate() {
                validate_stmt(stmt, &format!("{path}.stmt[{index}]"))?;
            }
            tail.as_ref()
                .map(|expr| validate_expr(expr, &format!("{path}.tail")))
                .unwrap_or(Ok(()))
        }
        IrExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            validate_expr(condition, &format!("{path}.condition"))?;
            validate_expr(then_branch, &format!("{path}.then"))?;
            else_branch
                .as_ref()
                .map(|expr| validate_expr(expr, &format!("{path}.else")))
                .unwrap_or(Ok(()))
        }
        IrExprKind::Call(name, _args) => {
            if name == "i32" {
                return Err(unsupported(
                    path,
                    "integer conversions are outside the first target",
                ));
            }
            Err(unsupported(
                path,
                format!("call `{name}` has no runtime boundary"),
            ))
        }
        IrExprKind::String(_)
        | IrExprKind::Record(_, _)
        | IrExprKind::Field(_, _)
        | IrExprKind::Array(_)
        | IrExprKind::EnumVariant { .. }
        | IrExprKind::Index(_, _) => Err(unsupported(
            path,
            "non-scalar expression is outside the first target",
        )),
    }
}

fn unsupported(path: &str, message: impl Into<String>) -> BackendError {
    BackendError {
        code: "BE002",
        backend: FIRST_TARGET.name,
        message: format!("at {path}: {}", message.into()),
    }
}

pub trait Backend {
    type Error: std::error::Error;

    fn name(&self) -> &'static str;
    fn emit(&self, program: &IrProgram) -> Result<String, Self::Error>;
}

/// A verified, target-independent artifact.  It is useful for acceptance and
/// golden tests, but is not QBE, assembly, or an executable.
pub struct TextBackend;

impl Backend for TextBackend {
    type Error = BackendError;

    fn name(&self) -> &'static str {
        "vex-text-v1"
    }

    fn emit(&self, program: &IrProgram) -> Result<String, Self::Error> {
        validate_first_target(program)?;
        Ok(format!(
            "target {}\nentry {}\ncalling-convention {}\noverflow {}\ndivision {}\ncontrol-flow {}\nruntime {}\n---\n{}",
            FIRST_TARGET.name,
            FIRST_TARGET.entry_point,
            FIRST_TARGET.calling_convention,
            FIRST_TARGET.overflow,
            FIRST_TARGET.division,
            FIRST_TARGET.control_flow,
            FIRST_TARGET.runtime_boundary,
            ir::render(program)
        ))
    }
}

/// QBE remains an explicit boundary: no guessed or partial native code.
pub struct QbeBackend;

impl Backend for QbeBackend {
    type Error = BackendError;

    fn name(&self) -> &'static str {
        "qbe"
    }

    fn emit(&self, _program: &IrProgram) -> Result<String, Self::Error> {
        Err(BackendError {
            code: "BE001",
            backend: self.name(),
            message: "QBE emission is not implemented; use the verified textual backend".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, FIRST_TARGET, TextBackend, validate_first_target};
    use crate::{builder::build_ast, ir, parser::parse_vex};

    fn lower(source: &str) -> ir::IrProgram {
        let pairs = parse_vex(source).unwrap();
        ir::lower(&build_ast(pairs).unwrap()).unwrap()
    }

    #[test]
    fn contract_invariants_are_explicit() {
        assert_eq!(FIRST_TARGET.entry_point, "vex_main");
        assert!(FIRST_TARGET.calling_convention.contains("i32/bool"));
        assert!(FIRST_TARGET.overflow.contains("trap"));
        assert!(FIRST_TARGET.division.contains("zero"));
        assert!(FIRST_TARGET.runtime_boundary.contains("no I/O"));
    }

    #[test]
    fn accepts_scalar_control_flow_and_text_emits_only_after_validation() {
        let program = lower("let x: i32 = 1; while x < 3 { x = x + 1; } x;");
        validate_first_target(&program).unwrap();
        let text = TextBackend.emit(&program).unwrap();
        assert!(text.starts_with("target vex-scalar-v1\nentry vex_main\n"));
        assert!(text.contains("let x: i32"));
    }

    #[test]
    fn rejects_u64_io_and_aggregates() {
        for source in ["let x: u64 = u64(1); x;", "let xs = [1, 2]; xs;"] {
            let program = lower(source);
            assert!(validate_first_target(&program).is_err(), "{source}");
            assert!(TextBackend.emit(&program).is_err(), "{source}");
        }
    }
}

use crate::ast::{Expr, Op, Stmt, Type};
use std::collections::HashMap;

#[derive(Clone)]
struct Function {
    params: Vec<Type>,
    ret: Type,
}
type EnumDefinitions = HashMap<String, Vec<(String, Vec<(String, Type)>)>>;

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
    functions: HashMap<String, Function>,
    records: HashMap<String, Vec<(String, Type)>>,
    enums: EnumDefinitions,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            functions: HashMap::new(),
            records: HashMap::new(),
            enums: HashMap::new(),
        }
    }
    pub fn analyze(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for s in stmts {
            if let Stmt::Enum { name, variants } = s {
                if self.enums.contains_key(name) || self.records.contains_key(name) {
                    return Err(format!("duplicate type `{name}`"));
                }
                let mut seen = std::collections::HashSet::new();
                let mut definitions = Vec::new();
                for variant in variants {
                    if !seen.insert(&variant.name) {
                        return Err(format!("duplicate enum variant `{}`", variant.name));
                    }
                    for (_, ty) in &variant.fields {
                        Self::check_supported(ty)?;
                    }
                    definitions.push((variant.name.clone(), variant.fields.clone()));
                }
                self.enums.insert(name.clone(), definitions);
            }
        }
        for s in stmts {
            if let Stmt::Record { name, fields } = s {
                if self.records.contains_key(name) {
                    return Err(format!("duplicate record `{name}`"));
                }
                let mut seen = std::collections::HashSet::new();
                for (field, ty) in fields {
                    self.check_type(ty)?;
                    if !seen.insert(field) {
                        return Err(format!("duplicate field `{field}` in record `{name}`"));
                    }
                }
                self.records.insert(name.clone(), fields.clone());
            }
            if let Stmt::Function {
                name,
                params,
                return_type,
                ..
            } = s
            {
                if self.functions.contains_key(name) {
                    return Err(format!("duplicate function `{name}`"));
                }
                for (_, t) in params {
                    self.check_type(t)?;
                }
                self.check_type(return_type)?;
                self.functions.insert(
                    name.clone(),
                    Function {
                        params: params.iter().map(|(_, t)| t.clone()).collect(),
                        ret: return_type.clone(),
                    },
                );
            }
        }
        let mut scope = self.symbols.clone();
        self.check_stmts(stmts, &mut scope, 0, None)?;
        self.symbols = scope;
        Ok(())
    }
    fn check_supported(t: &Type) -> Result<(), String> {
        if matches!(
            t,
            Type::Inferred
                | Type::I32
                | Type::U64
                | Type::Bool
                | Type::String
                | Type::Unit
                | Type::Custom(_)
                | Type::Array(_)
        ) {
            Ok(())
        } else {
            Err(format!("unsupported type annotation: {t:?}"))
        }
    }
    fn check_type(&self, t: &Type) -> Result<(), String> {
        Self::check_supported(t)?;
        if let Type::Custom(name) = t {
            if !self.records.contains_key(name) && !self.enums.contains_key(name) {
                return Err(format!("undefined type `{name}`"));
            }
        }
        Ok(())
    }
    fn check_stmts(
        &self,
        stmts: &[Stmt],
        scope: &mut HashMap<String, Type>,
        loops: usize,
        ret: Option<&Type>,
    ) -> Result<(), String> {
        for s in stmts {
            self.check_stmt(s, scope, loops, ret)?;
        }
        Ok(())
    }
    fn check_stmt(
        &self,
        s: &Stmt,
        scope: &mut HashMap<String, Type>,
        loops: usize,
        ret: Option<&Type>,
    ) -> Result<(), String> {
        match s {
            Stmt::Record { .. } | Stmt::Enum { .. } => {}
            Stmt::Function {
                params,
                body,
                return_type,
                ..
            } => {
                let mut local = HashMap::new();
                for (n, t) in params {
                    local.insert(n.clone(), t.clone());
                }
                let actual = self.check_expr(body, &mut local, 0, Some(return_type))?;
                if *return_type != Type::Unit && actual != *return_type && actual != Type::Inferred
                {
                    return Err(format!(
                        "function return type mismatch: expected {return_type:?}, got {actual:?}"
                    ));
                }
            }
            Stmt::Let { name, ty, value } => {
                self.check_type(ty)?;
                let vt = self.check_expr(value, scope, loops, ret)?;
                let final_t = if *ty == Type::Inferred {
                    vt.clone()
                } else {
                    ty.clone()
                };
                if *ty != Type::Inferred && !Self::compatible(ty, &vt) {
                    return Err(format!(
                        "type mismatch for `{name}`: annotation is {ty:?}, value is {vt:?}"
                    ));
                }
                scope.insert(name.clone(), final_t);
            }
            Stmt::Assign { name, value } => {
                let declared = scope
                    .get(name)
                    .ok_or_else(|| format!("undefined variable `{name}`"))?
                    .clone();
                let vt = self.check_expr(value, scope, loops, ret)?;
                if !Self::compatible(&declared, &vt) {
                    return Err(format!("type mismatch in assignment to `{name}`"));
                }
            }
            Stmt::AssignIndex {
                name,
                indices,
                value,
            } => {
                let mut ty = scope
                    .get(name)
                    .ok_or_else(|| format!("undefined variable `{name}`"))?
                    .clone();
                for index in indices {
                    if self.check_expr(index, scope, loops, ret)? != Type::I32 {
                        return Err("array index must be an i32".into());
                    }
                    ty = match ty {
                        Type::Array(element) => *element,
                        other => return Err(format!("indexing requires an array, got {other:?}")),
                    };
                }
                let vt = self.check_expr(value, scope, loops, ret)?;
                if !Self::compatible(&ty, &vt) {
                    return Err("type mismatch in array assignment".into());
                }
            }
            Stmt::ExprStmt(e) => {
                self.check_expr(e, scope, loops, ret)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.check_expr(condition, scope, loops, ret)? != Type::Bool {
                    return Err("if condition must be a boolean expression".into());
                }
                self.check_expr(then_branch, scope, loops, ret)?;
                if let Some(e) = else_branch {
                    self.check_expr(e, scope, loops, ret)?;
                }
            }
            Stmt::While { condition, body } => {
                if self.check_expr(condition, scope, loops, ret)? != Type::Bool {
                    return Err("while condition must be a boolean expression".into());
                }
                self.check_expr(body, scope, loops + 1, ret)?;
            }
            Stmt::Break | Stmt::Continue if loops == 0 => {
                return Err("loop control may only be used inside a loop".into());
            }
            Stmt::Break | Stmt::Continue => {}
            Stmt::Return(e) => {
                let actual = e
                    .as_ref()
                    .map(|x| self.check_expr(x, scope, loops, ret))
                    .transpose()?
                    .unwrap_or(Type::Unit);
                let expected = ret.ok_or("return may only be used inside a function")?;
                if !Self::compatible(expected, &actual) {
                    return Err(format!(
                        "return type mismatch: expected {expected:?}, got {actual:?}"
                    ));
                }
            }
        }
        Ok(())
    }
    fn compatible(a: &Type, b: &Type) -> bool {
        a == b
            || *a == Type::Inferred
            || *b == Type::Inferred
            || matches!((a, b), (Type::I32, Type::U64) | (Type::U64, Type::I32))
            || match (a, b) {
                (Type::Array(x), Type::Array(y)) => Self::compatible(x, y),
                _ => false,
            }
    }
    fn check_expr(
        &self,
        e: &Expr,
        scope: &mut HashMap<String, Type>,
        loops: usize,
        ret: Option<&Type>,
    ) -> Result<Type, String> {
        match e {
            Expr::Int(_) => Ok(Type::I32),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::String(_) => Ok(Type::String),
            Expr::Record(name, fields) => {
                let schema = self
                    .records
                    .get(name)
                    .ok_or_else(|| format!("undefined record `{name}`"))?;
                if fields.len() != schema.len() {
                    return Err(format!("record `{name}` requires {} fields", schema.len()));
                }
                for (field, value) in fields {
                    let expected = schema
                        .iter()
                        .find(|(n, _)| n == field)
                        .ok_or_else(|| format!("unknown field `{field}` on record `{name}`"))?;
                    if !Self::compatible(&expected.1, &self.check_expr(value, scope, loops, ret)?) {
                        return Err(format!(
                            "field `{field}` on record `{name}` has the wrong type"
                        ));
                    }
                }
                for (field, _) in schema {
                    if !fields.iter().any(|(n, _)| n == field) {
                        return Err(format!("missing field `{field}` on record `{name}`"));
                    }
                }
                Ok(Type::Custom(name.clone()))
            }
            Expr::EnumVariant {
                enum_name,
                variant,
                fields,
            } => {
                let variants = self
                    .enums
                    .get(enum_name)
                    .ok_or_else(|| format!("undefined enum `{enum_name}`"))?;
                let schema = variants
                    .iter()
                    .find(|(name, _)| name == variant)
                    .ok_or_else(|| format!("unknown variant `{variant}` on enum `{enum_name}`"))?;
                if fields.len() != schema.1.len() {
                    return Err(format!(
                        "enum variant `{enum_name}::{variant}` requires {} fields",
                        schema.1.len()
                    ));
                }
                for (field, value) in fields {
                    let expected = schema
                        .1
                        .iter()
                        .find(|(name, _)| name == field)
                        .ok_or_else(|| format!("unknown field `{field}` on variant `{variant}`"))?;
                    if !Self::compatible(&expected.1, &self.check_expr(value, scope, loops, ret)?) {
                        return Err(format!(
                            "field `{field}` on variant `{variant}` has the wrong type"
                        ));
                    }
                }
                Ok(Type::Custom(enum_name.clone()))
            }
            Expr::Field(value, field) => {
                let ty = self.check_expr(value, scope, loops, ret)?;
                let Type::Custom(name) = ty else {
                    return Err(format!("field access requires a record, got {ty:?}"));
                };
                self.records
                    .get(&name)
                    .and_then(|fields| fields.iter().find(|(n, _)| n == field))
                    .map(|(_, ty)| ty.clone())
                    .ok_or_else(|| format!("unknown field `{field}` on record `{name}`"))
            }
            Expr::Array(values) => {
                let mut element = Type::Inferred;
                for value in values {
                    let ty = self.check_expr(value, scope, loops, ret)?;
                    if element == Type::Inferred {
                        element = ty;
                    } else if !Self::compatible(&element, &ty) {
                        return Err("array elements must have compatible types".into());
                    }
                }
                Ok(Type::Array(Box::new(element)))
            }
            Expr::Index(value, index) => {
                let ty = self.check_expr(value, scope, loops, ret)?;
                if self.check_expr(index, scope, loops, ret)? != Type::I32 {
                    return Err("array index must be an i32".into());
                }
                match ty {
                    Type::Array(element) => Ok(*element),
                    other => Err(format!("indexing requires an array, got {other:?}")),
                }
            }
            Expr::Var(n) => scope
                .get(n)
                .cloned()
                .ok_or_else(|| format!("undefined variable `{n}`")),
            Expr::Call(n, args) => {
                if n == "map" {
                    if args.len() % 2 != 0 {
                        return Err("map expects key/value pairs".into());
                    }
                    for arg in args {
                        self.check_expr(arg, scope, loops, ret)?;
                    }
                    return Ok(Type::Custom("Map".into()));
                }
                if n == "map_get" {
                    if args.len() != 2 {
                        return Err("map_get expects a map and string key".into());
                    }
                    if self.check_expr(&args[0], scope, loops, ret)? != Type::Custom("Map".into())
                        || self.check_expr(&args[1], scope, loops, ret)? != Type::String
                    {
                        return Err("map_get expects a map and string key".into());
                    }
                    return Ok(Type::Inferred);
                }
                if matches!(n.as_str(), "ok" | "err") {
                    if args.len() != 1 {
                        return Err(format!("{n} expects one argument"));
                    }
                    self.check_expr(&args[0], scope, loops, ret)?;
                    return Ok(Type::Custom("Result".into()));
                }
                if matches!(n.as_str(), "print" | "println") {
                    if args.len() != 1 {
                        return Err(format!("{n} expects one argument"));
                    }
                    self.check_expr(&args[0], scope, loops, ret)?;
                    return Ok(Type::Unit);
                }
                if matches!(n.as_str(), "i32" | "u64") {
                    if args.len() != 1 {
                        return Err(format!("{n} expects one argument"));
                    }
                    self.check_expr(&args[0], scope, loops, ret)?;
                    return Ok(if n == "i32" { Type::I32 } else { Type::U64 });
                }
                let f = self
                    .functions
                    .get(n)
                    .ok_or_else(|| format!("undefined function `{n}`"))?;
                if f.params.len() != args.len() {
                    return Err(format!(
                        "function `{n}` expects {} arguments",
                        f.params.len()
                    ));
                }
                for (a, t) in args.iter().zip(&f.params) {
                    if !Self::compatible(t, &self.check_expr(a, scope, loops, ret)?) {
                        return Err(format!("argument type mismatch in call to `{n}`"));
                    }
                }
                Ok(f.ret.clone())
            }
            Expr::BinaryOp(a, op, b) => {
                let x = self.check_expr(a, scope, loops, ret)?;
                let y = self.check_expr(b, scope, loops, ret)?;
                match op {
                    Op::And | Op::Or => {
                        if x == Type::Bool && y == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err("logical operators require boolean operands".into())
                        }
                    }
                    Op::Eq | Op::Lt | Op::Gt => {
                        if Self::compatible(&x, &y) {
                            Ok(Type::Bool)
                        } else {
                            Err("comparison operands must have compatible types".into())
                        }
                    }
                    _ => {
                        if matches!(x, Type::Bool | Type::String)
                            || matches!(y, Type::Bool | Type::String)
                        {
                            Err("arithmetic operators require numeric operands".into())
                        } else {
                            Ok(x)
                        }
                    }
                }
            }
            Expr::UnaryOp(op, x) => {
                let t = self.check_expr(x, scope, loops, ret)?;
                match op {
                    Op::Not if t == Type::Bool => Ok(Type::Bool),
                    Op::Sub if matches!(t, Type::I32 | Type::U64) => Ok(t),
                    _ => Err("invalid unary operator operand".into()),
                }
            }
            Expr::Block(ss, tail) => {
                let mut local = scope.clone();
                self.check_stmts(ss, &mut local, loops, ret)?;
                tail.as_ref()
                    .map(|x| self.check_expr(x, &mut local, loops, ret))
                    .transpose()
                    .map(|x| x.unwrap_or(Type::Unit))
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.check_expr(condition, scope, loops, ret)? != Type::Bool {
                    return Err("if condition must be a boolean expression".into());
                };
                let a = self.check_expr(then_branch, scope, loops, ret)?;
                if let Some(b) = else_branch {
                    let c = self.check_expr(b, scope, loops, ret)?;
                    if !Self::compatible(&a, &c) {
                        return Err("if branches must have compatible types".into());
                    }
                }
                Ok(a)
            }
        }
    }
}

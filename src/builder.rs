use pest::Parser;
use pest::iterators::{Pair, Pairs};

use crate::ast::{EnumVariant, Expr, Op, Stmt, Type};
use crate::parser::{Rule, VexParser};

#[derive(Debug, Clone, PartialEq)]
pub struct BuildError(pub String);

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BuildError {}

pub fn build_ast(pairs: Pairs<Rule>) -> Result<Vec<Stmt>, BuildError> {
    let program = pairs
        .into_iter()
        .find(|pair| pair.as_rule() == Rule::program)
        .ok_or_else(|| BuildError("parser returned no program".into()))?;
    program
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::stmt)
        .map(build_stmt)
        .collect()
}

fn build_stmt(pair: Pair<Rule>) -> Result<Stmt, BuildError> {
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| BuildError("empty statement".into()))?;
    match inner.as_rule() {
        Rule::let_decl => {
            let mut parts = inner.into_inner();
            let name = parts
                .next()
                .ok_or_else(|| BuildError("let declaration is missing a name".into()))?
                .as_str()
                .to_string();
            let first = parts
                .next()
                .ok_or_else(|| BuildError(format!("let `{name}` is missing an initializer")))?;
            let (ty, expr_pair) = if first.as_rule() == Rule::type_name {
                (
                    parse_type(first.as_str()),
                    parts.next().ok_or_else(|| {
                        BuildError(format!("let `{name}` is missing an initializer"))
                    })?,
                )
            } else {
                (Type::Inferred, first)
            };
            Ok(Stmt::Let {
                name,
                ty,
                value: build_expr(expr_pair)?,
            })
        }
        Rule::record_decl => {
            let mut parts = inner.into_inner();
            let name = parts
                .next()
                .ok_or_else(|| BuildError("record is missing a name".into()))?
                .as_str()
                .to_string();
            let mut fields = Vec::new();
            for field in parts {
                let mut p = field.into_inner();
                let field_name = p.next().unwrap().as_str().to_string();
                let ty = parse_type(p.next().unwrap().as_str());
                fields.push((field_name, ty));
            }
            Ok(Stmt::Record { name, fields })
        }
        Rule::enum_decl => {
            let mut parts = inner.into_inner();
            let name = parts.next().unwrap().as_str().to_string();
            let variants = parts
                .map(|variant| {
                    let mut p = variant.into_inner();
                    let variant_name = p.next().unwrap().as_str().to_string();
                    let fields = p
                        .filter(|part| part.as_rule() == Rule::enum_field)
                        .map(|part| {
                            let mut field = part.into_inner();
                            (
                                field.next().unwrap().as_str().to_string(),
                                parse_type(field.next().unwrap().as_str()),
                            )
                        })
                        .collect();
                    EnumVariant {
                        name: variant_name,
                        fields,
                    }
                })
                .collect();
            Ok(Stmt::Enum { name, variants })
        }
        Rule::assign => {
            let mut parts = inner.into_inner();
            let target = parts
                .next()
                .ok_or_else(|| BuildError("assignment is missing a target".into()))?;
            let (name, indices) = if target.as_rule() == Rule::assign_target {
                let mut p = target.into_inner();
                let name = p.next().unwrap().as_str().to_string();
                let indices = p.map(build_expr).collect::<Result<Vec<_>, _>>()?;
                (name, indices)
            } else {
                (target.as_str().to_string(), Vec::new())
            };
            let value = build_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("assignment is missing a value".into()))?,
            )?;
            if indices.is_empty() {
                Ok(Stmt::Assign { name, value })
            } else {
                Ok(Stmt::AssignIndex {
                    name,
                    indices,
                    value,
                })
            }
        }
        Rule::if_stmt => {
            let mut parts = inner.into_inner();
            let condition = build_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("if statement is missing a condition".into()))?,
            )?;
            let then_branch = build_block_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("if statement is missing a then branch".into()))?,
            )?;
            let else_branch = parts
                .next()
                .map(build_block_expr)
                .transpose()?
                .map(Box::new);
            Ok(Stmt::If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch,
            })
        }
        Rule::while_stmt => {
            let mut parts = inner.into_inner();
            let condition = build_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("while statement is missing a condition".into()))?,
            )?;
            let body = build_block_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("while statement is missing a body".into()))?,
            )?;
            Ok(Stmt::While {
                condition,
                body: Box::new(body),
            })
        }
        Rule::break_stmt => Ok(Stmt::Break),
        Rule::continue_stmt => Ok(Stmt::Continue),
        Rule::return_stmt => Ok(Stmt::Return(
            inner.into_inner().next().map(build_expr).transpose()?,
        )),
        Rule::fn_decl => {
            let mut parts = inner.into_inner();
            let name = parts
                .next()
                .ok_or_else(|| BuildError("function is missing a name".into()))?
                .as_str()
                .to_string();
            let mut params = Vec::new();
            let mut return_type = Type::Unit;
            let mut body = None;
            for part in parts {
                match part.as_rule() {
                    Rule::param => {
                        let mut p = part.into_inner();
                        let n = p.next().unwrap().as_str().to_string();
                        let t = parse_type(p.next().unwrap().as_str());
                        params.push((n, t));
                    }
                    Rule::type_name => return_type = parse_type(part.as_str()),
                    Rule::block => body = Some(build_block_expr(part)?),
                    _ => {}
                }
            }
            Ok(Stmt::Function {
                name,
                params,
                return_type,
                body: body.ok_or_else(|| BuildError("function is missing a body".into()))?,
            })
        }
        Rule::expr_stmt => {
            let expr = inner
                .into_inner()
                .next()
                .ok_or_else(|| BuildError("empty expression statement".into()))?;
            Ok(Stmt::ExprStmt(build_expr(expr)?))
        }
        other => Err(BuildError(format!(
            "unsupported statement `{other:?}`; supported statements are let, assignment, if, while, break, continue, and expressions"
        ))),
    }
}

fn parse_type(name: &str) -> Type {
    if name.starts_with('[') && name.ends_with(']') {
        return Type::Array(Box::new(parse_type(&name[1..name.len() - 1])));
    }
    match name {
        "i32" => Type::I32,
        "u64" => Type::U64,
        "f64" => Type::F64,
        "bool" => Type::Bool,
        "string" => Type::String,
        "res" => Type::Res,
        other => Type::Custom(other.to_string()),
    }
}

fn build_block_expr(pair: Pair<Rule>) -> Result<Expr, BuildError> {
    let mut stmts = Vec::new();
    let mut last_expr: Option<Box<Expr>> = None;

    for inner in pair.into_inner() {
        if last_expr.is_some() {
            stmts.push(Stmt::ExprStmt(*last_expr.take().unwrap()));
        }
        match inner.as_rule() {
            Rule::stmt => {
                let stmt = build_stmt(inner)?;
                if let Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                } = stmt
                {
                    last_expr = Some(Box::new(Expr::If {
                        condition: Box::new(condition),
                        then_branch,
                        else_branch,
                    }));
                } else {
                    stmts.push(stmt);
                }
            }
            Rule::expr => last_expr = Some(Box::new(build_expr(inner)?)),
            Rule::if_expr => last_expr = Some(Box::new(build_expr(inner)?)),
            _ => {
                return Err(BuildError(format!(
                    "unsupported block item `{:?}`",
                    inner.as_rule()
                )));
            }
        }
    }

    Ok(Expr::Block(stmts, last_expr))
}

struct TextParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> TextParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<Expr, BuildError> {
        let expr = self.binary(0)?;
        self.ws();
        if self.pos != self.input.len() {
            return Err(BuildError("unexpected token in expression".into()));
        }
        Ok(expr)
    }

    fn binary(&mut self, min: u8) -> Result<Expr, BuildError> {
        let mut left = self.unary()?;
        loop {
            self.ws();
            let (op, precedence) = match self.peek() {
                Some(b'|') if self.input.get(self.pos + 1) == Some(&b'|') => (Op::Or, 1),
                Some(b'&') if self.input.get(self.pos + 1) == Some(&b'&') => (Op::And, 2),
                Some(b'=') if self.input.get(self.pos + 1) == Some(&b'=') => (Op::Eq, 1),
                Some(b'<') => (Op::Lt, 3),
                Some(b'>') => (Op::Gt, 3),
                Some(b'+') => (Op::Add, 4),
                Some(b'-') => (Op::Sub, 4),
                Some(b'*') => (Op::Mul, 5),
                Some(b'/') => (Op::Div, 5),
                _ => break,
            };
            if precedence < min {
                break;
            }
            self.pos += if matches!(op, Op::Or | Op::And | Op::Eq) {
                2
            } else {
                1
            };
            let right = self.binary(precedence + 1)?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, BuildError> {
        self.ws();
        if self.peek() == Some(b'-') {
            self.pos += 1;
            return Ok(Expr::UnaryOp(Op::Sub, Box::new(self.unary()?)));
        }
        if self.peek() == Some(b'!') {
            self.pos += 1;
            return Ok(Expr::UnaryOp(Op::Not, Box::new(self.unary()?)));
        }
        if self.peek() == Some(b'(') {
            self.pos += 1;
            let value = self.binary(0)?;
            self.ws();
            if self.peek() != Some(b')') {
                return Err(BuildError("missing `)`".into()));
            }
            self.pos += 1;
            return Ok(value);
        }
        if self.peek() == Some(b'"') {
            self.pos += 1;
            let start = self.pos;
            while self.peek().is_some_and(|c| c != b'"') {
                self.pos += 1;
            }
            if self.peek() != Some(b'"') {
                return Err(BuildError("unterminated string literal".into()));
            }
            let value = String::from_utf8_lossy(&self.input[start..self.pos]).to_string();
            self.pos += 1;
            return Ok(Expr::String(value));
        }
        if self.peek() == Some(b'[') {
            self.pos += 1;
            let mut values = Vec::new();
            self.ws();
            if self.peek() != Some(b']') {
                loop {
                    values.push(self.binary(0)?);
                    self.ws();
                    if self.peek() != Some(b',') {
                        break;
                    }
                    self.pos += 1;
                    self.ws();
                    if self.peek() == Some(b']') {
                        break;
                    }
                }
            }
            self.ws();
            if self.peek() != Some(b']') {
                return Err(BuildError("missing `]` in array literal".into()));
            }
            self.pos += 1;
            return Ok(Expr::Array(values));
        }
        if self.peek().is_some_and(|c| c.is_ascii_digit()) {
            let start = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }
            return std::str::from_utf8(&self.input[start..self.pos])
                .map_err(|_| BuildError("invalid integer literal".into()))?
                .parse()
                .map(Expr::Int)
                .map_err(|_| BuildError("integer literal out of range".into()));
        }
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(BuildError("expected expression".into()));
        }
        let name = String::from_utf8_lossy(&self.input[start..self.pos]);
        match name.as_ref() {
            "true" => Ok(Expr::Bool(true)),
            "false" => Ok(Expr::Bool(false)),
            _ => {
                self.ws();
                let mut value = if self.peek() == Some(b':')
                    && self.input.get(self.pos + 1) == Some(&b':')
                {
                    self.pos += 2;
                    let variant_start = self.pos;
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.pos += 1;
                    }
                    if variant_start == self.pos {
                        return Err(BuildError("enum variant requires a name".into()));
                    }
                    let variant =
                        String::from_utf8_lossy(&self.input[variant_start..self.pos]).into_owned();
                    let mut fields = Vec::new();
                    self.ws();
                    if self.peek() == Some(b'{') {
                        self.pos += 1;
                        self.ws();
                        if self.peek() != Some(b'}') {
                            loop {
                                let field_start = self.pos;
                                while self
                                    .peek()
                                    .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                                {
                                    self.pos += 1;
                                }
                                if self.pos == field_start || self.peek() != Some(b':') {
                                    return Err(BuildError(
                                        "enum field requires `name: value`".into(),
                                    ));
                                }
                                let field =
                                    String::from_utf8_lossy(&self.input[field_start..self.pos])
                                        .into_owned();
                                self.pos += 1;
                                fields.push((field, self.binary(0)?));
                                self.ws();
                                if self.peek() != Some(b',') {
                                    break;
                                }
                                self.pos += 1;
                                self.ws();
                                if self.peek() == Some(b'}') {
                                    break;
                                }
                            }
                        }
                        self.ws();
                        if self.peek() != Some(b'}') {
                            return Err(BuildError("missing `}` in enum variant".into()));
                        }
                        self.pos += 1;
                    }
                    Expr::EnumVariant {
                        enum_name: name.into_owned(),
                        variant,
                        fields,
                    }
                } else if self.peek() == Some(b'{') {
                    self.pos += 1;
                    let mut fields = Vec::new();
                    self.ws();
                    if self.peek() != Some(b'}') {
                        loop {
                            self.ws();
                            let field_start = self.pos;
                            while self
                                .peek()
                                .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                            {
                                self.pos += 1;
                            }
                            if self.pos == field_start || self.peek() != Some(b':') {
                                return Err(BuildError(
                                    "record field requires `name: value`".into(),
                                ));
                            }
                            let field = String::from_utf8_lossy(&self.input[field_start..self.pos])
                                .into_owned();
                            self.pos += 1;
                            let expr = self.binary(0)?;
                            fields.push((field, expr));
                            self.ws();
                            if self.peek() == Some(b',') {
                                self.pos += 1;
                                self.ws();
                                if self.peek() == Some(b'}') {
                                    break;
                                }
                                continue;
                            }
                            break;
                        }
                    }
                    self.ws();
                    if self.peek() != Some(b'}') {
                        return Err(BuildError("missing `}` in record construction".into()));
                    }
                    self.pos += 1;
                    Expr::Record(name.into_owned(), fields)
                } else if self.peek() == Some(b'(') {
                    self.pos += 1;
                    let mut args = Vec::new();
                    self.ws();
                    if self.peek() != Some(b')') {
                        loop {
                            args.push(self.binary(0)?);
                            self.ws();
                            if self.peek() == Some(b',') {
                                self.pos += 1;
                                continue;
                            }
                            break;
                        }
                    }
                    self.ws();
                    if self.peek() != Some(b')') {
                        return Err(BuildError("missing `)` in call".into()));
                    }
                    self.pos += 1;
                    Expr::Call(name.into_owned(), args)
                } else if self.peek() == Some(b'[') {
                    let mut value = Expr::Var(name.into_owned());
                    while self.peek() == Some(b'[') {
                        self.pos += 1;
                        let index = self.binary(0)?;
                        self.ws();
                        if self.peek() != Some(b']') {
                            return Err(BuildError("missing `]` in index expression".into()));
                        }
                        self.pos += 1;
                        value = Expr::Index(Box::new(value), Box::new(index));
                    }
                    value
                } else {
                    Expr::Var(name.into_owned())
                };
                loop {
                    self.ws();
                    if self.peek() == Some(b'[') {
                        self.pos += 1;
                        let index = self.binary(0)?;
                        self.ws();
                        if self.peek() != Some(b']') {
                            return Err(BuildError("missing `]` in index expression".into()));
                        }
                        self.pos += 1;
                        value = Expr::Index(Box::new(value), Box::new(index));
                        continue;
                    }
                    if self.peek() != Some(b'.') {
                        break;
                    }
                    self.pos += 1;
                    let start = self.pos;
                    while self
                        .peek()
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
                    {
                        self.pos += 1;
                    }
                    if start == self.pos {
                        return Err(BuildError("field access requires a field name".into()));
                    }
                    let field = String::from_utf8_lossy(&self.input[start..self.pos]).into_owned();
                    value = Expr::Field(Box::new(value), field);
                }
                Ok(value)
            }
        }
    }

    fn ws(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }
}

fn build_expr(pair: Pair<Rule>) -> Result<Expr, BuildError> {
    match pair.as_rule() {
        Rule::expr | Rule::term | Rule::atom | Rule::paren => {
            let text = pair.as_str().trim();
            if text.starts_with("if ") {
                let parsed = VexParser::parse(Rule::if_expr, text)
                    .map_err(|error| BuildError(format!("invalid if expression: {error}")))?;
                let expr = parsed
                    .into_iter()
                    .find(|pair| pair.as_rule() == Rule::if_expr)
                    .ok_or_else(|| {
                        BuildError("if expression parse produced no expression".into())
                    })?;
                return build_expr(expr);
            }
            if text.starts_with('{') {
                let parsed = VexParser::parse(Rule::block, text)
                    .map_err(|error| BuildError(format!("invalid block expression: {error}")))?;
                let block = parsed
                    .into_iter()
                    .find(|pair| pair.as_rule() == Rule::block)
                    .ok_or_else(|| BuildError("block expression parse produced no block".into()))?;
                return build_block_expr(block);
            }
            TextParser::new(text).parse()
        }
        Rule::block => build_block_expr(pair),
        Rule::if_expr => {
            let mut parts = pair.into_inner();
            let condition = build_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("if expression is missing a condition".into()))?,
            )?;
            let then_branch = build_block_expr(
                parts
                    .next()
                    .ok_or_else(|| BuildError("if expression is missing a then branch".into()))?,
            )?;
            let else_branch = parts
                .next()
                .map(build_block_expr)
                .transpose()?
                .map(Box::new);
            Ok(Expr::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            })
        }
        Rule::int => pair.as_str().parse::<i64>().map(Expr::Int).map_err(|_| {
            BuildError(format!(
                "integer literal `{}` is out of range",
                pair.as_str()
            ))
        }),
        Rule::string => {
            let raw = &pair.as_str()[1..pair.as_str().len() - 1];
            Ok(Expr::String(
                raw.replace("\\\"", "\"").replace("\\\\", "\\"),
            ))
        }
        Rule::array => TextParser::new(pair.as_str()).parse(),
        Rule::ident if pair.as_str() == "true" => Ok(Expr::Bool(true)),
        Rule::ident if pair.as_str() == "false" => Ok(Expr::Bool(false)),
        Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
        other => Err(BuildError(format!("expected expression, got `{other:?}`"))),
    }
}

mod analyzer;
mod ast;
mod builder;
mod diagnostics;
mod evaluator;
mod ir;
mod parser;
mod project;

use std::fs;
use std::io::{self, BufRead, Read, Write};

use analyzer::SemanticAnalyzer;
use builder::build_ast;
use diagnostics::{Diagnostic, Span, span_for};
use parser::parse_vex;
use pest::error::LineColLocation;

fn pipeline(source: &str) -> Result<Vec<ast::Stmt>, Diagnostic> {
    let pairs = parse_vex(source).map_err(|error| {
        let (line, column) = match error.line_col {
            LineColLocation::Pos(position) => position,
            LineColLocation::Span(start, _) => start,
        };
        let offset = source
            .lines()
            .take(line.saturating_sub(1))
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + column.saturating_sub(1);
        Diagnostic::new(
            "E1001",
            format!("parse error: {error}"),
            Span {
                start: offset,
                end: offset + 1,
            },
        )
        .with_suggestion("check the preceding expression and add a semicolon if needed")
    })?;
    let stmts = build_ast(pairs).map_err(|error| {
        let message = error.to_string();
        let needle = message.split('`').nth(1);
        Diagnostic::new("E1002", message.clone(), span_for(source, needle))
    })?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&stmts).map_err(|error| {
        let needle = error.split('`').nth(1);
        let mut diagnostic = Diagnostic::new("E2001", error.clone(), span_for(source, needle));
        if error.contains("undefined variable") {
            diagnostic =
                diagnostic.with_suggestion("declare the variable with `let` before using it");
        } else if error.contains("boolean") {
            diagnostic =
                diagnostic.with_suggestion("use `true` or `false`, or compare numeric values");
        }

        diagnostic
    })?;
    Ok(stmts)
}

fn lower_source(source: &str) -> Result<(Vec<ast::Stmt>, ir::IrProgram), Diagnostic> {
    let stmts = pipeline(source)?;
    let program = ir::lower(&stmts).map_err(|error| {
        Diagnostic::new(
            "E4002",
            format!("lowering error: {error}"),
            span_for(source, None),
        )
    })?;
    Ok((stmts, program))
}

fn run(source: &str) -> Result<ast::Value, Diagnostic> {
    let stmts = pipeline(source)?;
    evaluator::eval(&stmts).map_err(|error| {
        let message = error.to_string();
        let code = if matches!(error, evaluator::EvalError::TypeError(_)) {
            "E3002"
        } else {
            "E3001"
        };
        Diagnostic::new(code, message.clone(), span_for(source, None)).with_suggestion(
            if message.contains("division by zero") {
                "ensure the divisor is not zero"
            } else {
                "check the values used by this expression"
            },
        )
    })
}

fn read_source(path: Option<&str>) -> Result<String, String> {
    match path {
        None | Some("-") => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|error| format!("could not read stdin: {error}"))?;
            Ok(source)
        }
        Some(path) => {
            fs::read_to_string(path).map_err(|error| format!("could not read `{path}`: {error}"))
        }
    }
}

fn usage() -> &'static str {
    "usage: vexlang [check|run|repl|project|fmt|test|build|ir] [FILE|-]\n       vexlang FILE   (backwards-compatible alias for run)\n       vexlang --version"
}

fn repl() {
    println!("Vexlang REPL (enter `:quit` to exit)");
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut line = String::new();
    loop {
        print!("vex> ");
        let _ = io::stdout().flush();
        line.clear();
        if input.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line.trim() == ":quit" || line.trim() == ":q" {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        match run(&line) {
            Ok(value) => println!("{value:?}"),
            Err(error) => eprintln!("{}", error.render(&line, "<repl>")),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.as_slice(), [flag] if flag == "--version" || flag == "-V") {
        println!("vexlang {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let (command, path) = match args.as_slice() {
        [] => ("run", None),
        [one]
            if !matches!(
                one.as_str(),
                "check" | "run" | "repl" | "project" | "fmt" | "test" | "build" | "ir"
            ) =>
        {
            ("run", Some(one.as_str()))
        }
        [command] => (command.as_str(), None),
        [command, path]
            if matches!(
                command.as_str(),
                "check" | "run" | "project" | "fmt" | "test" | "build" | "ir"
            ) =>
        {
            (command.as_str(), Some(path.as_str()))
        }
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };
    if command == "repl" {
        repl();
        return;
    }
    if command == "project" {
        match project::load(std::path::Path::new(path.unwrap_or("vex.toml"))) {
            Ok(config) => println!("{} {} ({})", config.name, config.version, config.source),
            Err(error) => {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    };
    match command {
        "check" | "run" | "test" => match pipeline(&source) {
            Ok(_stmts) if command == "check" => println!("ok"),
            Ok(stmts) if command == "test" => match evaluator::eval(&stmts) {
                Ok(_) => println!("test passed"),
                Err(error) => {
                    eprintln!(
                        "{}",
                        Diagnostic::new("E3001", error.to_string(), span_for(&source, None))
                            .render(&source, path.unwrap_or("<stdin>"))
                    );
                    std::process::exit(1);
                }
            },
            Ok(_) => match run(&source) {
                Ok(value) => println!("{value:?}"),
                Err(error) => {
                    eprintln!("{}", error.render(&source, path.unwrap_or("<stdin>")));
                    std::process::exit(1);
                }
            },
            Err(error) => {
                eprintln!("{}", error.render(&source, path.unwrap_or("<stdin>")));
                std::process::exit(1);
            }
        },
        "build" | "ir" => match lower_source(&source) {
            Ok((_stmts, program)) => print!("{}", ir::render(&program)),
            Err(error) => {
                eprintln!("{}", error.render(&source, path.unwrap_or("<stdin>")));
                std::process::exit(1);
            }
        },
        "fmt" => {
            if let Err(error) = pipeline(&source) {
                eprintln!("{}", error.render(&source, path.unwrap_or("<stdin>")));
                std::process::exit(1);
            }
            let mut stdout = io::stdout();
            if let Err(error) = stdout.write_all(source.as_bytes()) {
                eprintln!("error: {error}");
                std::process::exit(2);
            }
        }
        _ => {
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::ast::Value;

    #[test]
    fn precedence_and_parentheses() {
        assert_eq!(run("1 + 2 * 3;"), Ok(Value::Int(7)));
        assert_eq!(run("(1 + 2) * 3;"), Ok(Value::Int(9)));
    }

    #[test]
    fn variables_and_unary_minus() {
        assert_eq!(run("let x = 4; -x + 10;"), Ok(Value::Int(6)));
    }

    #[test]
    fn reports_user_errors() {
        assert!(run("unknown;").unwrap_err().contains("undefined variable"));
        assert!(run("1 / 0;").unwrap_err().contains("division by zero"));
        assert!(run("let x: f64 = 1;").unwrap_err().contains("unsupported"));
        assert!(
            run("let x: i32 = 2147483648;")
                .unwrap_err()
                .contains("does not match")
        );
        assert!(
            run("let x: u64 = -1;")
                .unwrap_err()
                .contains("does not match")
        );
        assert!(run("if 1 { 2; }").unwrap_err().contains("boolean"));
    }

    #[test]
    fn evaluates_boolean_logic() {
        assert_eq!(run("true && !false;"), Ok(Value::Bool(true)));
        assert_eq!(
            run("let ready: bool = 2 > 1; ready || false;"),
            Ok(Value::Bool(true))
        );
    }

    #[test]
    fn short_circuits_boolean_logic() {
        assert_eq!(run("false && (1 / 0 == 0);"), Ok(Value::Bool(false)));
        assert_eq!(run("true || (1 / 0 == 0);"), Ok(Value::Bool(true)));
    }

    #[test]
    fn if_else_expressions_work() {
        assert_eq!(
            run("let total = if 1 < 2 { 10 + 5 } else { 33 }; total;"),
            Ok(Value::Int(15))
        );
        assert_eq!(
            run("let value = if false { 1 } else { 2 }; value;"),
            Ok(Value::Int(2))
        );
    }

    #[test]
    fn arrays_support_empty_nested_indexing_and_mutation() {
        assert_eq!(run("let xs: [i32] = []; xs;"), Ok(Value::Array(vec![])));
        assert_eq!(
            run("let xs = [[1, 2], [3, 4]]; xs[1][0];"),
            Ok(Value::Int(3))
        );
        assert_eq!(run("let xs = [1, 2]; xs[0] = 9; xs[0];"), Ok(Value::Int(9)));
    }

    #[test]
    fn arrays_report_type_and_bounds_errors() {
        assert!(
            run("let xs: [i32] = [true];")
                .unwrap_err()
                .contains("type mismatch")
        );
        assert!(
            run("let xs = [1]; xs[2];")
                .unwrap_err()
                .contains("out of bounds")
        );
        assert!(
            run("let xs = [1]; xs[true];")
                .unwrap_err()
                .contains("index")
        );
    }

    #[test]
    fn while_loops_evaluate_conditionally() {
        assert_eq!(run("while false { let x = 5; }"), Ok(Value::Int(0)));
        assert_eq!(
            run("let flag = false; while flag { let x = 1; }"),
            Ok(Value::Int(0))
        );
    }

    #[test]
    fn while_loops_can_update_state() {
        assert_eq!(
            run("let x = 0; while x < 3 { x = x + 1; } x;"),
            Ok(Value::Int(3))
        );
    }

    #[test]
    fn break_and_continue_work_through_nested_control_flow() {
        assert_eq!(
            run(
                "let x = 0; while x < 10 { x = x + 1; if x == 3 { continue; } if x == 6 { break; } } x;"
            ),
            Ok(Value::Int(6))
        );
        assert_eq!(
            run("let x = 0; while x < 3 { if true { x = x + 1; continue; } x = 99; } x;"),
            Ok(Value::Int(3))
        );
        assert_eq!(
            run(
                "let x = 0; while x < 1 { let ignored = if true { break; 1 } else { 2 }; x = 99; } x;"
            ),
            Ok(Value::Int(0))
        );
    }

    #[test]
    fn rejects_loop_control_outside_loops() {
        assert!(
            run("break;")
                .unwrap_err()
                .contains("only be used inside a loop")
        );
        assert!(
            run("if true { continue; }")
                .unwrap_err()
                .contains("only be used inside a loop")
        );
    }

    #[test]
    fn enforces_loop_iteration_limit() {
        let error = run("while true {}").unwrap_err();
        assert!(error.contains("loop iteration limit exceeded"));
    }

    #[test]
    fn enforces_scope_and_assignment_types() {
        assert!(
            run("{ let hidden = 1; }; hidden;")
                .unwrap_err()
                .contains("undefined variable")
        );
        assert!(
            run("let flag: bool = true; flag = 1;")
                .unwrap_err()
                .contains("type mismatch")
        );
    }

    #[test]
    fn accepts_integer_boundaries() {
        assert_eq!(
            run("let low: i32 = -2147483648; let high: i32 = 2147483647; high;"),
            Ok(Value::Int(2147483647))
        );
        assert!(
            run("let low: i32 = -2147483649;")
                .unwrap_err()
                .contains("does not match")
        );
    }

    #[test]
    fn reports_parse_locations() {
        let error = run("let value = 1;\nvalue + ;").unwrap_err();
        assert!(error.contains("parse error"));
        assert!(error.contains("2:"));
    }

    #[test]
    fn evaluates_typed_functions_and_recursion() {
        assert_eq!(
            run("fn add(a: i32, b: i32) -> i32 { a + b } add(2, 3);"),
            Ok(Value::Int(5))
        );
        assert_eq!(
            run(
                "fn factorial(n: i32) -> i32 { if n == 0 { 1 } else { n * factorial(n - 1) } } factorial(5);"
            ),
            Ok(Value::Int(120))
        );
    }

    #[test]
    fn evaluates_distinct_integer_types_and_strings() {
        assert_eq!(run("let value: u64 = u64(4); value;"), Ok(Value::U64(4)));
        assert_eq!(run("\"hello\";"), Ok(Value::String("hello".into())));
        assert_eq!(run("\"hello\" == \"hello\";"), Ok(Value::Bool(true)));
    }

    #[test]
    fn validates_function_arguments_and_conversions() {
        assert!(
            run("fn id(value: i32) -> i32 { value } id(true);")
                .unwrap_err()
                .contains("argument")
        );
        assert!(run("u64(-1);").unwrap_err().contains("cannot convert"));
    }

    #[test]
    fn records_smoke() {
        assert_eq!(
            run("record Point { x: i32, y: i32 } let p: Point = Point { x: 2, y: 3 }; p.x + p.y;"),
            Ok(Value::Int(5))
        );
    }

    #[test]
    fn validates_record_fields() {
        assert!(
            run("record Point { x: i32 } Point { y: 1 };")
                .unwrap_err()
                .contains("unknown field")
        );
        assert!(
            run("record Point { x: i32, y: i32 } Point { x: 1 };")
                .unwrap_err()
                .contains("requires 2 fields")
        );
        assert!(
            run("record Point { x: i32 } Point { x: true };")
                .unwrap_err()
                .contains("wrong type")
        );
        assert!(
            run("record Point { x: i32 } let p = Point { x: 1 }; p.missing;")
                .unwrap_err()
                .contains("unknown field")
        );
    }

    #[test]
    fn evaluates_enum_variants() {
        assert_eq!(
            run(
                "enum Maybe { None, Some { value: i32 } } let value: Maybe = Maybe::Some { value: 7 }; value;"
            ),
            Ok(Value::Enum {
                enum_name: "Maybe".into(),
                variant: "Some".into(),
                fields: [("value".into(), Value::Int(7))].into_iter().collect(),
            })
        );
    }

    #[test]
    fn evaluates_structured_results() {
        assert_eq!(
            run("let success = ok(3); success;"),
            Ok(Value::Result {
                ok: true,
                value: Box::new(Value::Int(3)),
            })
        );
        assert_eq!(
            run("let failure = err(\"bad\"); failure;"),
            Ok(Value::Result {
                ok: false,
                value: Box::new(Value::String("bad".into())),
            })
        );
    }

    #[test]
    fn evaluates_scoped_maps() {
        assert_eq!(
            run(r#"let values = map("answer", 42); map_get(values, "answer");"#),
            Ok(Value::Int(42))
        );
    }
}

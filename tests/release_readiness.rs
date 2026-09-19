use std::fmt::Write as _;
use std::process::{Command, Stdio};
use std::time::Instant;

fn vex(source: &str, command: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vexlang"))
        .args([command, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("vexlang binary should start");
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin should be available")
            .write_all(source.as_bytes())
            .expect("source should be accepted");
    }
    child.wait_with_output().expect("vexlang should exit")
}

#[test]
fn cli_version_matches_package_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_vexlang"))
        .arg("--version")
        .output()
        .expect("vexlang binary should start");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("vexlang {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn compatibility_corpus_runs_and_lowers() {
    let cases = [
        ("1 + 2 * 3;", "Int(7)"),
        ("let x: i32 = 0; while x < 3 { x = x + 1; } x;", "Int(3)"),
        (
            "fn add(a: i32, b: i32) -> i32 { a + b } add(2, 5);",
            "Int(7)",
        ),
        ("let value: u64 = u64(4); value == u64(4);", "Bool(true)"),
        (
            "record Point { x: i32, y: i32 } let p: Point = Point { x: 2, y: 3 }; p.x + p.y;",
            "Int(5)",
        ),
    ];

    for (source, expected) in cases {
        let run = vex(source, "run");
        assert!(
            run.status.success(),
            "run failed for {source:?}: {:?}",
            run.stderr
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), expected);

        let ir = vex(source, "build");
        assert!(
            ir.status.success(),
            "lowering failed for {source:?}: {:?}",
            ir.stderr
        );
        let text = String::from_utf8_lossy(&ir.stdout);
        assert!(!text.trim().is_empty(), "empty IR for {source:?}");
    }
}

#[test]
fn diagnostic_fixture_is_stable() {
    let output = vex("let value = missing;\n", "check");
    assert!(!output.status.success());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        diagnostic,
        r#"-:1:13: error[E2001]: undefined variable `missing`
   1 | let value = missing;
     |             ^^^^^^^
     = help: declare the variable with `let` before using it
"#
    );
}

#[test]
fn generated_arithmetic_corpus_is_consistent() {
    for n in 0..64 {
        let source = format!("({n} + 2) * 3 - {n};");
        let output = vex(&source, "run");
        assert!(output.status.success(), "generated source failed: {source}");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            format!("Int({})", n * 2 + 6)
        );
    }
}

#[test]
fn textual_ir_is_deterministic_and_not_a_native_artifact() {
    let source = "let answer = 6 * 7; answer;";
    let first = vex(source, "build");
    let second = vex(source, "ir");
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    let ir = String::from_utf8_lossy(&first.stdout);
    assert!(ir.contains("i32"));
    assert!(!ir.contains("\u{7f}"));
}

#[test]
fn target_command_emits_contract_header_for_scalar_and_rejects_unsupported() {
    let scalar_source = "let x: i32 = 1; while x < 3 { x = x + 1; } x;";
    let output = vex(scalar_source, "target");
    assert!(
        output.status.success(),
        "target failed: {:?}",
        output.stderr
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("target vex-scalar-v1\nentry vex_main\n"));

    let non_scalar_source = "let xs = [1, 2]; xs;";
    let output = vex(non_scalar_source, "target");
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BE002"));
}

#[test]
#[ignore = "manual performance smoke test; run with cargo test --test release_readiness -- --ignored --nocapture"]
fn performance_smoke() {
    let mut source = String::new();
    for n in 0..2_000 {
        writeln!(&mut source, "{n};").expect("writing to a string cannot fail");
    }
    let start = Instant::now();
    let output = vex(&source, "check");
    assert!(output.status.success());
    eprintln!("checked 2,000 statements in {:?}", start.elapsed());
}

# Vex

Vex is an experimental Rust compiler prototype. It is currently an **alpha**
interpreter for a deliberately small language, not a production compiler.
It is released as an **alpha interpreter and typed-IR prototype** under the MIT
license. It is not a native compiler yet; see [LICENSE](LICENSE) and the
feature-status documentation.

## Supported language

```vex
let base: i32 = 10;
let answer = -(base + 2) * 3;
answer;
```

The interpreter supports integer and boolean literals, `let` declarations,
variables, `+`, `-`, `*`, `/`, `==`, `<`, `>`, `&&`, `||`, unary minus, logical
not, `if`/`else`, `while`, `break`, `continue`, blocks, and expression statements. Blocks may have
local declarations and a final expression.
Variables can be updated with assignment, for example
`let x = 0; while x < 3 { x = x + 1; }`.
`break` exits the innermost loop and `continue` starts its next iteration; both
are rejected outside loops. Loops stop with a runtime error after one million
iterations to prevent runaway programs.
The `i32`, `u64`, and `bool` annotations are supported. `i32` values are
range-checked and `u64` values are represented distinctly at runtime.
Functions support typed parameters, returns, recursion, and a bounded call
depth. Strings and `print`/`println` are available in the standard library.
Floating point, `res`, custom types, collections, filesystem access, modules,
and code generation are intentionally deferred.

## Usage

Run or validate a source file:

```sh
cargo run -- run program.vex
cargo run -- check program.vex
cargo run -- test program.vex
```

`fmt` writes the current source to stdout. `build` validates and lowers the
program to a deterministic textual IR; it does not emit machine code. `ir` is
an explicit alias for the same IR artifact. The legacy
`cargo run -- program.vex` form remains an alias for `run`.

Read source from stdin (also explicitly available as `-`):

```sh
printf 'let x = 6; x * 7;' | cargo run
printf '1 + 2;' | cargo run -- -
```

The result is printed to stdout. Diagnostics include stable error codes,
file/line/column locations, source underlines, and suggestions where useful.
Exit status 0 means success, 1 means a source or test failure, 2 means a CLI
or I/O error, and 3 means an explicitly unsupported operation.

## Development

```sh
cargo fmt -- --check
cargo test
cargo clippy --all-targets -- -D warnings
mdbook test docs
```

Milestone 9 adds a compatibility corpus, stable CLI diagnostic fixture,
deterministic generated arithmetic checks, and an opt-in performance smoke
test. Run the latter with:

```sh
cargo test --test release_readiness -- --ignored --nocapture
```

Release positioning is **alpha interpreter + typed IR prototype**. A native
machine-code backend is intentionally unavailable; `build` emits validated
textual IR only. This is suitable for an explicitly labeled alpha public
release, not a production compiler release.

The minimum supported Rust version (MSRV) is **1.85.0**, matching the Rust
2024 edition. CI tests both the current stable toolchain and the MSRV.

The complete [Vex mdBook documentation](docs/src/intro.md) includes the
[language reference](docs/src/language.md), [CLI and exit codes](docs/src/cli.md),
[diagnostic format](docs/src/diagnostics.md), [feature
status](docs/src/features.md), [development quality gates](docs/src/development.md),
and [roadmap](docs/src/milestones.md).

Project tracking is also available in the standalone
[milestone plan](README_MILESTONES.md) and
[completed phases](README_COMPLETED_PHASES.md) documents.

QBE output is disabled until a correct implementation exists; the internal
generator returns an explicit unsupported error rather than emitting partial
output. The textual IR is therefore the only build artifact in this milestone.

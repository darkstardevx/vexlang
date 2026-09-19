# ⚡ Vex

> **A cybercore language lab for safe experiments in syntax, semantics, and
> typed lowering.**

Vex is an **alpha interpreter and typed-IR prototype** for a deliberately small
language. It is not a production compiler or native toolchain yet. Vex is
released under the [MIT license](LICENSE).

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![CI](https://github.com/darkstardevx/vexlang/actions/workflows/quality.yml/badge.svg)](https://github.com/darkstardevx/vexlang/actions/workflows/quality.yml)
[![Release](https://img.shields.io/badge/release-0.1.0--alpha.1-purple)](https://github.com/darkstardevx/vexlang)

## 🛰️ Signal status

```text
RUNTIME   [ONLINE]   interpreter + typed textual IR
SAFETY    [GREEN]    diagnostics, limits, MSRV, CI gates
BACKEND   [DEFERRED] native machine-code / QBE emission
CHANNEL   [ALPHA]   syntax and APIs may evolve
```

## 🧬 Supported language

```vex
let base: i32 = 10;
let answer = -(base + 2) * 3;
answer;
```

The interpreter supports integer and boolean literals, `let` declarations,
variables, `+`, `-`, `*`, `/`, `==`, `<`, `>`, `&&`, `||`, unary minus, logical
not, `if`/`else`, `while`, `break`, `continue`, blocks, and expression
statements. Blocks may have local declarations and a final expression.

Variables can be updated with assignment:

```vex
let x = 0;
while x < 3 { x = x + 1; }
x;
```

`break` exits the innermost loop and `continue` starts its next iteration; both
are rejected outside loops. Loops stop with a runtime error after one million
iterations to prevent runaway programs.

The `i32`, `u64`, and `bool` annotations are supported. Functions support typed
parameters, returns, recursion, and bounded call depth. User-defined records
support declarations, construction, validated fields, and field access:

```vex
record Point { x: i32, y: i32 }
let p: Point = Point { x: 2, y: 3 };
p.x + p.y;
```

Strings and `print`/`println` are available in the standard library. Arrays
support literals, nesting, typed annotations such as `[i32]`, zero-based
indexing, and indexed assignment:

```vex
let values: [i32] = [1, 2, 3];
values[1] = 9;
values[1];
```

Maps and generics are explicitly deferred. Floating point, `res`, filesystem
access, modules/imports,
structured errors/results, project configuration, and native code generation
are intentionally deferred.

## 🧪 Quick start

Run or validate a source file:

```sh
cargo run -- run program.vex
cargo run -- check program.vex
cargo run -- test program.vex
cargo run -- build program.vex
cargo run -- ir program.vex
cargo run -- target program.vex
```

`fmt` validates source and writes it to stdout. `build` and `ir` validate and
lower the program to deterministic textual IR; they do not emit machine code.
`target` validates the IR against the `vex-scalar-v1` contract and emits
a verified textual target artifact.
Before rendering, the IR verifier checks scopes, types, declarations, control
flow, and backend support. A failed verification is a hard error: no backend
is invoked and no native artifact is produced.
The legacy `cargo run -- program.vex` form remains an alias for `run`.

Read source from stdin:

```sh
printf 'let x = 6; x * 7;' | cargo run
printf '1 + 2;' | cargo run -- -
```

Diagnostics include stable error codes, file/line/column locations, source
underlines, and suggestions. Exit status `0` means success, `1` means a source
or test failure, `2` means a CLI or I/O error, and `3` means an explicitly
unsupported operation.

## 🗺️ Project roadmap

```text
VEX
├── ✅ Phase 1  Core language stabilization
├── ✅ Phase 2  Control flow
├── ✅ Phase 3  Functions and types
├── ✅ Phase 4  Standard library foundation
├── ✅ Phase 5  Diagnostics and CLI tooling
├── ✅ Phase 6  Typed, backend-independent IR
├── ✅ Phase 7  Compatibility and release readiness
├── 🚧 Phase 8  Public alpha hardening
│   ├── release artifacts and version checks
│   ├── stronger fuzzing and benchmarks
│   └── stable + MSRV quality gates
├── 🚧 Phase 9  Language expansion
│   ├── ✅ user-defined record types
│   ├── ✅ typed arrays and indexing
│   ├── maps, generics, and structured errors
│   └── modules and project configuration
├── 🔭 Phase 10 Production diagnostics
│   ├── AST-wide source spans
│   ├── multi-span diagnostics
│   └── machine-readable output
├── 🔭 Phase 11 Native compilation
│   ├── verified backend
│   ├── compiled/interpreted differential tests
│   └── reproducible build artifacts
└── 🔭 Phase 12 Ecosystem and tooling
    ├── package management
    ├── LSP/editor integration
    └── formatter and compatibility policies
```

See the [future milestone plan](README_MILESTONES.md) and
[completed phases](README_COMPLETED_PHASES.md) for detailed scope and status.

## 🛠️ Development

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo +1.85.0 test --all-targets
cargo +1.85.0 clippy --all-targets -- -D warnings
mdbook test docs
```

The release-readiness suite includes compatibility, golden diagnostic, and
generated-input tests, plus an opt-in performance smoke test:

```sh
cargo test --test release_readiness -- --ignored --nocapture
```

The minimum supported Rust version (MSRV) is **1.85.0**, matching Rust 2024.
The complete [Vex mdBook documentation](docs/src/intro.md) includes the
[language reference](docs/src/language.md), [CLI and exit codes](docs/src/cli.md),
[diagnostic format](docs/src/diagnostics.md), [feature status](docs/src/features.md),
[development gates](docs/src/development.md), and [roadmap](docs/src/milestones.md).

## 📦 Public alpha release checklist

Current release: `0.1.0-alpha.1`.

The package version is declared in `Cargo.toml` and exposed by
`vexlang --version`. Before publishing a tagged alpha, run:

```sh
./scripts/check-release.sh
./scripts/release-artifacts.sh dist
```

The second command produces the Cargo source package and a SHA-256 checksum
under `dist/`. The package contains the textual IR implementation only; it
does not contain a native executable. Verify the checksum and attach both
files to the matching Git tag. The complete publish checklist is in the
[release guide](docs/src/release.md).

## 🤝 Contributing

Small, focused changes are welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md) for setup, good-first-contribution ideas,
language-change expectations, pull-request guidance, and all quality gates.
The current Discord contact is **`darkstar_dev`**; a dedicated server invite
will be added when the community space is established. Please review the
[security policy](SECURITY.md) before reporting vulnerabilities.

QBE output is disabled until a correct implementation exists; the internal
generator returns an explicit unsupported error rather than emitting partial
output. Textual IR is the only build artifact in this milestone.

### Native lowering contract

The first native target may consume only verified IR containing `i32`, `u64`,
`bool`, `unit`, calls to declared functions, structured `if`/`while`, records,
and immutable arrays. The target must document calling convention, integer
overflow/division behavior, record and array layout, allocation/lifetime, and
the process entry point. Strings, field projection, indexed mutation, enums,
`Result`, maps, and I/O builtins are currently deferred because no runtime ABI
has been specified. The `Backend` trait is the clean boundary for a future
QBE or Cranelift implementation; it intentionally emits no code today.

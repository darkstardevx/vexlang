# Development and quality

## MSRV

The minimum supported Rust version is **1.85.0**, required by the Rust 2024
edition. Changes must compile and pass tests on both current stable Rust and
Rust 1.85.0.

## Quality gates

Run the complete local gates from the repository root:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo +1.85.0 test --all-targets
cargo +1.85.0 clippy --all-targets -- -D warnings
mdbook test docs
git diff --check
```

Tests cover expression evaluation, malformed input, type and scope errors,
control flow, functions, diagnostics, and CLI behavior. CI runs formatting,
tests, Clippy, documentation tests, and the MSRV checks on every push and pull
request.

When changing syntax, update the authoritative grammar in `src/vex.pest`, the
language reference, focused tests, and any diagnostic examples together.

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
control flow, functions, diagnostics, CLI behavior, and experimental QBE IL.
CI runs formatting, tests, Clippy, QBE backend smoke tests, documentation tests,
and the MSRV checks on every push and pull request.

When changing syntax, update the authoritative grammar in `src/vex.pest`, the
language reference, focused tests, and any diagnostic examples together.

## Release validation

Public alpha releases use the checked-in scripts:

```sh
./scripts/check-release.sh
./scripts/release-artifacts.sh dist
```

The first command checks version and package consistency. The second creates a
locked Cargo source archive and SHA-256 checksum. It never creates a native
binary; `build` remains a deterministic textual IR command. See the
[public alpha release guide](release.md) for the publish checklist.

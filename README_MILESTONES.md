# Vex milestones

This file is the forward-looking project plan. Completed work is recorded
separately in [`README_COMPLETED_PHASES.md`](README_COMPLETED_PHASES.md).

## Milestone 8 — Public alpha hardening

- Keep the `0.1.0-alpha.1` syntax and command behavior stable.
- Check package/CLI/documentation version consistency.
- Produce a reproducible Cargo source artifact and SHA-256 checksum.
- Exercise generated inputs, deterministic textual IR, compatibility, and
  performance without heavyweight dependencies.
- Keep stable, MSRV, documentation, and release-package CI gates green.

**Current status:** Complete for the public alpha scope. A production `0.1`
release remains blocked by the missing native backend and deferred language
features.

## Milestone 9 — Language expansion

- Add user-defined types and records. **Records are complete:** declarations,
  construction, field access, semantic field/type validation, interpreter
  evaluation, and textual-IR lowering are covered by unit and CLI tests.
- Add typed array collections and indexing. **Complete:** array literals,
  nested arrays, annotations, indexing, mutation, diagnostics, and textual IR.
- Maps and generics remain explicitly deferred.
- Add a structured error/result model.
- Define modules, imports, and project configuration.

Collections/indexing, structured error/result values, modules/imports, and
project configuration remain explicitly deferred.

## Milestone 12 — Production diagnostics

- Retain source spans on every AST node.
- Add richer multi-span diagnostics and machine-readable output.
- Add error recovery for multiple diagnostics per invocation.

## Milestone 13 — Native compilation

- Implement a verified backend behind the existing typed IR.
- Add compiled-vs-interpreted differential tests.
- Define target support and reproducible build artifacts.

## Milestone 14 — Ecosystem and tooling

- Add package/project management.
- Add editor/LSP integration.
- Add formatter guarantees and language-server diagnostics.
- Publish stable standard-library and compatibility policies.

## Quality gate

Every milestone must preserve:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo +1.85.0 test --all-targets
cargo +1.85.0 clippy --all-targets -- -D warnings
mdbook test docs
```

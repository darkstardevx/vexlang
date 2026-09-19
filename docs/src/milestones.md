# Vex milestones

This roadmap tracks the work required to move Vex from an alpha interpreter to
a usable language. Each milestone has an explicit quality gate.

## Milestone 1 — Stabilize the core language

- Keep one authoritative grammar and parser contract.
- Preserve source locations in parse and semantic diagnostics.
- Define and enforce integer, boolean, inference, and assignment rules.
- Add malformed-input, boundary-value, scope, and regression tests.
- Add formatting, test, Clippy, and documentation CI gates.

**Gate:** `cargo fmt -- --check`, `cargo test`, `cargo clippy --all-targets
-- -D warnings`, `mdbook test docs`, and the MSRV test on Rust 1.85.0 all pass.

## Milestone 2 — Complete control flow

- Add `break` and `continue`.
- Define loop safety behavior.
- Test nested loops, branches, blocks, and mutation.

**Status:** Complete in the alpha interpreter.

## Milestone 3 — Add functions

- Add declarations, parameters, calls, returns, recursion, and call-depth
  protection.
- Type-check arguments and return values.

**Status:** Complete in the alpha interpreter. Functions use `fn name(args)
-> type { ... }`; recursion is limited to 256 active calls.

## Milestone 4 — Expand the type system

- Distinguish integer widths at runtime.
- Add explicit `i32(value)` and `u64(value)` conversions.
- User-defined types are intentionally deferred until the interpreter has
  records/constructors and a stable module model.

**Status:** Integer widths and checked conversions are implemented.

## Milestone 5 — Build a standard library

- Add strings and safe console I/O (`print` and `println`).
- Arrays are covered by the typed-collections milestone; maps, files, errors,
  and modules are deferred to later milestones.

**Status:** Minimal string and console library implemented.

## Milestone 6 — Production diagnostics

- Add spans to AST nodes and underline source ranges.
- Add error codes and actionable suggestions.

**Status:** The parser, AST builder, semantic analyzer, and evaluator now
produce structured diagnostics with stable `E100x`/`E200x`/`E300x` codes,
source locations, underlines, and targeted help. Full per-node AST span
retention remains a follow-up refinement; diagnostics currently retain the
relevant source span at the pipeline boundary.

## Milestone 7 — Tooling

- Add `check`, `run`, `fmt`, `test`, and `build` commands.
- Define project configuration and lockfile behavior.

**Status:** The five commands are available with documented exit statuses and
stdin support. `build` reports the deliberate interpreter-only limitation
instead of pretending to compile. Project configuration and lockfile
semantics are deferred until a package/project model is designed.

Documentation for the syntax, commands, diagnostics, feature status, MSRV,
and quality gates is maintained as part of the mdBook.

## Milestone 8 — Compilation

- Add a typed, backend-independent lowered IR and a deterministic text form.
- Make `build` validate and lower without claiming to produce machine code.
- Keep backend output behind explicit target validation; unsupported constructs
  must fail explicitly.
- Differential-test interpreter results and lowering for supported expressions,
  control flow, functions, and types.

**Status:** The typed textual IR and `build`/`ir` artifact are implemented.
Experimental QBE IL emission is available for the verified scalar subset;
machine-code executable artifacts remain deferred.

## Milestone 9 — Compatibility and performance

- Add fuzzing, property tests, golden diagnostics, benchmarks, and a language
  compatibility suite.

**Status:** The release-readiness suite now covers interpreter/lowering
compatibility, a stable CLI diagnostic fixture, deterministic generated-input
tests, and an opt-in performance smoke test without adding heavyweight
dependencies. Coverage-guided fuzzing and statistically rigorous benchmarks
remain deferred.

## Milestone 10 — Public alpha hardening

- Check CLI, Cargo metadata, changelog, and documentation version consistency.
- Produce locked Cargo source artifacts with checksums.
- Run deterministic generated-input, compatibility, and textual-IR smoke tests.
- Validate release packages on tagged CI builds.

**Status:** Complete for the `0.1.0-alpha.1` interpreter and typed-IR release.
Native binaries are intentionally not produced.

## Milestone 11 — 0.1 release

- Freeze syntax, remove stale prototype artifacts, publish language
  documentation, and maintain CI for all quality gates.

**Status:** The repository is ready for a labeled `0.1.0-alpha.1` public
release as an interpreter and typed-IR prototype. The MIT license and security
contact are in place. A production `0.1` compiler release remains blocked by
native executable artifacts, broader backend validation, and deferred language
features.

## Milestone 12 — Typed collections

**Status:** Complete for arrays: literals, typed annotations, nested and empty
arrays, indexing, mutation, semantic diagnostics, and textual IR lowering.
Maps, generics, and structured error/result values remain deferred.

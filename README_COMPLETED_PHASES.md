# Vex completed phases

This file records delivered phases. Forward-looking work is tracked in
[`README_MILESTONES.md`](README_MILESTONES.md).

## Phase 1 — Core language stabilization

- Established the interpreter pipeline and authoritative grammar.
- Added arithmetic, comparisons, booleans, variables, declarations, inference,
  assignment, blocks, and scoped evaluation.
- Added overflow, division-by-zero, range, and type validation.

## Phase 2 — Control flow

- Added `if`/`else`, `while`, `break`, and `continue`.
- Added nested control-flow propagation and a one-million-iteration loop limit.

## Phase 3 — Functions and types

- Added typed functions, parameters, returns, calls, recursion, and
  call-depth protection.
- Added distinct `i32`/`u64` runtime values and checked conversions.

## Phase 4 — Standard library foundation

- Added strings, `print`, and `println`.
- Documented deferred collections, filesystem access, modules, and resources.

## Phase 5 — Diagnostics and CLI tooling

- Added stable diagnostic codes, source locations, underlines, and suggestions.
- Added `check`, `run`, `fmt`, `test`, `build`, and `ir` commands.
- Defined CLI exit statuses and preserved the legacy file invocation.

## Phase 6 — Typed IR

- Added backend-independent typed lowering.
- Added deterministic textual IR.
- Added interpreter/lowering consistency tests.
- Kept native and QBE output explicitly unsupported until a verified backend
  exists.

## Phase 7 — Compatibility and release readiness

- Added compatibility corpus, golden diagnostics, generated-input tests, and
  an opt-in performance smoke test.
- Added MIT licensing, security policy, MSRV metadata, CI quality gates, and
  complete mdBook documentation.
- Published the project as `0.1.0-alpha.1`, an alpha interpreter and typed-IR
  prototype.

## Current release posture

The repository is public-ready for the labeled alpha scope. It is not yet a
production compiler release: native code generation, user-defined types,
collections, modules, and project/package management remain future work.

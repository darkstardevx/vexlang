# Features and limitations

## Supported

The interpreter currently supports:

* integer, boolean, and string literals;
* inferred declarations and `i32`, `u64`, `bool`, and `string` annotations;
* checked arithmetic, comparisons, equality, boolean short-circuiting, and
  explicit integer conversions;
* variables, assignment, blocks, `if`/`else`, `while`, `break`, and `continue`;
* typed functions, returns, recursion, and bounded call depth;
* user-defined record declarations, construction, validated fields, and access;
* `print` and `println`;
* structured diagnostics and the `check`, `run`, `fmt`, `test`, `build`, and
  `ir` commands;
* a typed, deterministic, backend-independent textual IR.

## Deferred or intentionally unsupported

Floating point values, resources (`res`), collections, filesystem access,
modules, project configuration, and lockfile semantics are deferred. Record
types are the first supported user-defined type. Milestone 9's lightweight
compatibility, diagnostic,
generated-input, and opt-in performance smoke tests are now present; a
coverage-guided fuzzing service remains deferred.

Vex has no verified machine-code or QBE backend. `build` stops after typed
lowering and emits textual IR; it never emits misleading output. Floating
point, resource, custom types, and other constructs are rejected explicitly by
the lowering stage rather than silently approximated. Record lowering is supported;
native code generation is not.

## Public release posture

Vex is suitable for a labeled `0.1.0-alpha.1` public release as an interpreter
and typed-IR prototype. It is not presented as a production compiler. The MIT
license grants redistribution rights, and security reports should be sent to
`cybercore.sh@gmail.com`. A native backend, fuzzing service, collections,
modules, and project configuration remain post-alpha work.

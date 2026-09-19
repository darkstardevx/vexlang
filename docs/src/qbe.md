# Experimental QBE backend

The `qbe` command emits QBE IL for the verified `vex-scalar-v1` target subset.
It is public but experimental: it produces an intermediate artifact and does not
assemble, link, or package a native executable.

## Local backend smoke tests

Install QBE 1.3 into a local prefix:

```sh
PREFIX="$HOME/.local" ./scripts/install-qbe.sh
```

Then run the same backend-focused smoke tests used by CI:

```sh
./scripts/qbe-smoke.sh
```

The smoke script requires `qbe`, `cc`, and Cargo on `PATH`. It runs the QBE
structural tests, compiled-vs-interpreted conformance cases, trap exit-code
checks, and release-readiness CLI/fixture tests.

## Runtime trap exits

If emitted QBE is assembled and linked with a C driver that calls `vex_main`,
target runtime failures exit with stable experimental codes:

| Exit | Meaning |
| --- | --- |
| `101` | checked `i32` overflow, including unary negation overflow |
| `102` | division by zero |
| `103` | signed division overflow, `i32::MIN / -1` |

The emitter lowers traps to `call $exit(w CODE)` followed by `hlt` so QBE never
falls through a failing target check.

## Current scalar subset

The QBE backend currently accepts only programs that pass `vex-scalar-v1`:

- `i32`, `bool`, and `unit` values;
- declared function calls;
- local variables and assignment;
- structured `if`/`while` control flow;
- `break`, `continue`, and `return`;
- checked scalar arithmetic and boolean short-circuiting.

Strings, `u64`, records, arrays, maps, enums, `Result`, I/O builtins, and native
executable packaging remain outside this target until their ABI/runtime behavior
is specified.

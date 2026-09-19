# Diagnostics

Errors are written to standard error. Each diagnostic has a stable error code,
a source location, an underlined source range, and—when useful—a suggested
next step:

```text
program.vex:1:1: error[E2001]: undefined variable `total`
   1 | total;
     | ^^^^^
     = help: declare the variable with `let` before using it
```

The location uses one-based line and column numbers. Codes identify the
pipeline stage:

| Code family | Stage |
| --- | --- |
| `E100x` | Parsing and AST construction |
| `E200x` | Semantic and type analysis |
| `E300x` | Evaluation and runtime failures |
| `E400x` | Unsupported tooling operations |

Examples include:

```text
program.vex:1:5: error[E1001]: parse error: ...
     = help: check the preceding expression and add a semicolon if needed
```

```text
program.vex:1:1: error[E3001]: division by zero
     = help: ensure the divisor is not zero
```

Diagnostics currently retain a relevant source span at the pipeline boundary.
Per-node span storage in every AST value is planned refinement work; the
display contract and error codes are already stable for the CLI.

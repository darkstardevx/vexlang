# Command-line reference

The executable is available as `vexlang`; during development, use
`cargo run --`. Every command accepts a file path or `-` for standard input.
With no path, input is read from standard input.

`vexlang --version` (or `-V`) prints the package version used by the release
metadata checks.

| Command | Behavior | Success |
| --- | --- | --- |
| `check FILE` | Parse and semantically validate without running | `0` |
| `run FILE` | Validate and evaluate the program | `0` |
| `repl` | Read, validate, and evaluate one source line at a time | `0` |
| `project FILE` | Validate and summarize a `vex.toml` project configuration | `0` |
| `fmt FILE` | Validate and write the source to standard output | `0` |
| `test FILE` | Validate and evaluate as an executable smoke test | `0` |
| `build FILE` | Validate and print the typed textual IR (no machine code) | `0` |
| `ir FILE` | Alias for `build`; print the typed textual IR | `0` |
| `target FILE` | Validate against `vex-scalar-v1` and emit textual target IR | `0` |

`vexlang FILE` remains a backwards-compatible alias for `vexlang run FILE`.
Examples:

```sh
cargo run -- check examples/hello.vex
cargo run -- run examples/hello.vex
 cargo run -- repl
printf 'let n = 6; n * 7;' | cargo run -- run -
cargo run -- fmt program.vex > formatted.vex
```

## Exit codes

* **0** — command completed successfully.
* **1** — source diagnostic or evaluation/test failure.
* **2** — invalid command usage or an input/output error.
* **3** — a recognized operation is deliberately unsupported (for example,
  a future machine-code backend request).

`build` does not create a fake executable or partial machine code. It emits a
stable, human-readable IR artifact instead. Unsupported language types and
backend features fail explicitly with a source diagnostic.

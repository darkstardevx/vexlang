# Contributing

Vex welcomes focused language experiments, regression tests, documentation
improvements, diagnostics work, and roadmap contributions.

Start with the root
[`CONTRIBUTING.md`](../../CONTRIBUTING.md), which defines the complete setup,
review, language-change, and pull-request expectations.

## Community

The current project contact is **`darkstar_dev` on Discord**. A dedicated
server and invite link are not established yet. Until then, use GitHub issues
and discussions for public questions and contribution coordination.

Use the address in [`SECURITY.md`](../../SECURITY.md) for private security
reports.

## Required checks

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo +1.85.0 test --all-targets
cargo +1.85.0 clippy --all-targets -- -D warnings
mdbook test docs
```

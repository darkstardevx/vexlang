# Contributing to Vex

Thanks for your interest in Vex. The project is an alpha interpreter and
typed-IR prototype, so focused experiments, tests, documentation, and
language-design feedback are all useful contributions.

## Before you start

1. Read the [README](README.md) and
   [completed phases](README_COMPLETED_PHASES.md).
2. Check the [milestone plan](README_MILESTONES.md) before proposing new
   syntax or runtime behavior.
3. Search existing issues and pull requests so work is not duplicated.
4. For significant language changes, open an issue describing the syntax,
   semantics, diagnostics, and compatibility impact before implementing.

## Good first contributions

- Add parser, evaluator, or diagnostic regression tests.
- Improve language-reference examples.
- Improve error messages and source suggestions.
- Add examples for existing CLI commands.
- Triage documentation gaps and reproducibility issues.
- Implement a narrowly scoped item from the current roadmap.

## Development setup

```sh
git clone https://github.com/darkstardevx/vexlang.git
cd vexlang
cargo test --all-targets
mdbook test docs
```

The minimum supported Rust version is **1.85.0**. Before opening a pull
request, run the complete quality gates:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo +1.85.0 test --all-targets
cargo +1.85.0 clippy --all-targets -- -D warnings
mdbook test docs
git diff --check
```

## Pull requests

- Keep changes focused and explain the user-visible behavior.
- Add tests for every new language or CLI behavior.
- Update the README, mdBook, changelog, or roadmap when behavior changes.
- Do not emit incomplete compiler output or silently accept unsupported syntax.
- Include the validation commands you ran in the pull request description.
- Keep commits small enough to review.

## Language changes

Every language change should define:

- accepted syntax and precedence;
- runtime and type-checking behavior;
- scope and mutation rules;
- diagnostics for invalid programs;
- interpreter and IR behavior;
- compatibility impact and a migration path, if applicable.

## Community

The current project contact is **`darkstar_dev` on Discord**. A dedicated
server and invite link are not established yet; once available, the invite
will be added here and to the README.

Until then, use GitHub issues and discussions for public project questions,
feature proposals, and contribution coordination. Use the address in
[SECURITY.md](SECURITY.md) for private security reports.

## Code of conduct

Be constructive, specific, and respectful. Review ideas and code on their
technical merits, welcome newcomers, and assume good intent while asking for
clarification when requirements are unclear.

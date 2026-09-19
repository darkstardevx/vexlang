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

### Discord channel copy

When the community server is created, these messages can be pasted into the
first channels:

#### `#welcome`

```text
⚡ Welcome to VEX // CYBERCORE

Vex is an alpha programming-language project: an interpreter and typed-IR
prototype built for experiments in syntax, semantics, diagnostics, and
eventual compilation.

Start here:
• Project: https://github.com/darkstardevx/vexlang
• Contributor guide: https://github.com/darkstardevx/vexlang/blob/main/CONTRIBUTING.md
• Documentation: https://github.com/darkstardevx/vexlang/tree/main/docs
• Current contact: darkstar_dev

What belongs here:
• language design and syntax discussion
• interpreter, analyzer, IR, and tooling development
• documentation, tests, examples, and contributor onboarding
• experiments, demos, and constructive feedback

Quick orientation:
1. Read the README and contributor guide.
2. Check the roadmap before proposing a new feature.
3. Introduce yourself in #general.
4. Look for beginner-friendly work in #good-first-issues.

Vex is intentionally early-stage. APIs and syntax may change, and the current
build command emits validated textual IR rather than native machine code.
Thanks for helping build it carefully.
```

#### `#rules`

```text
🛡️ VEX // CYBERCORE — COMMUNITY RULES

1. Be constructive.
   Critique code, designs, and proposals specifically. Do not attack people.

2. Welcome newcomers.
   Explain context, link documentation, and make room for questions.

3. Keep discussions technical and on-topic.
   Use the appropriate channel for language design, compiler development,
   support, showcases, and off-topic conversation.

4. Respect project boundaries.
   Do not present experimental Vex behavior as stable or production-ready.
   Follow the documented roadmap and compatibility expectations.

5. Do not post security vulnerabilities publicly.
   Send private security reports to cybercore.sh@gmail.com with the affected
   version, reproduction steps, and impact. Do not share exploit details in
   Discord channels.

6. No spam, harassment, discrimination, impersonation, or malicious content.
   Moderators may remove content or restrict access to protect the community.

7. Respect licenses and authorship.
   Do not repost private material or claim another contributor's work.
   Vex is released under the MIT license; follow the terms of other projects
   and dependencies.

8. Keep project decisions discoverable.
   Important design decisions should be summarized in GitHub issues,
   discussions, pull requests, or documentation rather than left only in chat.

By participating here, you agree to follow these rules and reasonable
moderator guidance. If something feels unsafe or unclear, contact
darkstar_dev privately.
```

## Code of conduct

Be constructive, specific, and respectful. Review ideas and code on their
technical merits, welcome newcomers, and assume good intent while asking for
clarification when requirements are unclear.

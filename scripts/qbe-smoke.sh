#!/usr/bin/env bash
set -euo pipefail

command -v qbe >/dev/null
command -v cc >/dev/null
qbe -h >/dev/null
cc --version >/dev/null

cargo test qbe_ -- --nocapture
cargo test --test release_readiness -- --nocapture

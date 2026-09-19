#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output=${1:-"$root/dist"}
cd "$root"

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)
test -n "$version"
rm -rf "$output"
mkdir -p "$output"
cargo package --locked --allow-dirty --no-verify
target_dir=${CARGO_TARGET_DIR:-$(cargo metadata --locked --no-deps --format-version 1 |
    sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p' | head -n 1)}
archive="$target_dir/package/vexlang-$version.crate"
test -f "$archive"
cp "$archive" "$output/"
(cd "$output" && sha256sum "vexlang-$version.crate" > "vexlang-$version.crate.sha256")
printf '%s\n' "wrote $output/vexlang-$version.crate and its SHA-256 checksum"

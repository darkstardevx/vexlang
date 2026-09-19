#!/usr/bin/env bash
set -euo pipefail

version="${QBE_VERSION:-1.3}"
prefix="${PREFIX:-$HOME/.local}"
archive="qbe-${version}.tar.xz"
url="https://c9x.me/compile/release/${archive}"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

curl -fL "$url" -o "$workdir/$archive"
tar -C "$workdir" -xf "$workdir/$archive"
make -C "$workdir/qbe-${version}" -j"$(nproc)"
make -C "$workdir/qbe-${version}" PREFIX="$prefix" install

"$prefix/bin/qbe" -h

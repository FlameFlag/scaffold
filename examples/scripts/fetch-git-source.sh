#!/usr/bin/env bash
# shellcheck shell=bash
set -eu

version="${1:-2.45.2}"
script_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
example_dir="$(CDPATH='' cd -- "$script_dir/.." && pwd)"
source_dir="$example_dir/sources/git"
archive="$example_dir/sources/git-$version.tar.xz"
url="https://mirrors.edge.kernel.org/pub/software/scm/git/git-$version.tar.xz"

mkdir -p "$example_dir/sources"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

curl -L "$url" -o "$archive"
rm -rf "$source_dir"
mkdir -p "$source_dir"
tar -xJf "$archive" -C "$source_dir" --strip-components=1

printf 'Fetched Git %s into %s\n' "$version" "$source_dir"

#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

cd "$project_dir"
cargo build --locked --release --bin tersh
sh "$project_dir/scripts/install-binary.sh" "$project_dir/target/release/tersh"
printf 'Optional visual cd helper: source %s/scripts/tersh-cd.sh from your shell profile.\n' "$project_dir"

#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

if ! command -v cargo >/dev/null 2>&1; then
  echo "fetch-or-build.sh: cargo not found. Install Rust toolchain first." >&2
  exit 1
fi

exec cargo build --release

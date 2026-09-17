#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

VERSION="$(awk -F'"' '/^version = / {print $2; exit}' Cargo.toml)"
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
  Darwin-arm64)  TARGET=aarch64-apple-darwin ;;
  Darwin-x86_64) TARGET=x86_64-apple-darwin ;;
  Linux-x86_64)  TARGET=x86_64-unknown-linux-gnu ;;
  Linux-aarch64) TARGET=aarch64-unknown-linux-gnu ;;
  *)             TARGET="" ;;
esac

fetch_prebuilt() {
  [[ -n "$TARGET" ]] || return 1
  local name="herdr-repo-picker-v${VERSION}-${TARGET}.tar.gz"
  local base="https://github.com/mayaton/herdr-repo-picker/releases/download/v${VERSION}"
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  curl -fsSL -o "$tmp/$name"        "$base/$name"        || return 1
  curl -fsSL -o "$tmp/$name.sha256" "$base/$name.sha256" || return 1
  (cd "$tmp" && shasum -a 256 -c "$name.sha256") || return 1
  mkdir -p target/release
  tar xzf "$tmp/$name" -C target/release
  chmod +x target/release/herdr-repo-picker
}

if fetch_prebuilt; then
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "fetch-or-build.sh: prebuilt not available and cargo not found." >&2
  exit 1
fi

exec cargo build --release

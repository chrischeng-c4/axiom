#!/usr/bin/env sh
# SPEC-MANAGED: projects/guard/tech-design/semantic/guard-install-script.md#text-source-unit
# CODEGEN-BEGIN
set -eu

INSTALL_DIR="${GUARD_INSTALL:-$HOME/.local/bin}"

say() { printf 'guard-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo is required for the source installer" 3
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

cargo build -p guard-cli
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/guard" "$INSTALL_DIR/guard"
say "installed: $INSTALL_DIR/guard"

if "$INSTALL_DIR/guard" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/guard" --version
fi
# CODEGEN-END

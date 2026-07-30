#!/usr/bin/env sh
# SPEC-MANAGED: apps/guard/tech-design/src/distribution.py
# HANDWRITE-BEGIN: gap=existing-project-patch tracker=#2823
set -eu

INSTALL_DIR="${GUARD_INSTALL:-$HOME/.local/bin}"

say() { printf 'guard-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo is required for the source installer" 3
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

cargo build -p guard --bin guard
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/guard" "$INSTALL_DIR/guard"
say "installed: $INSTALL_DIR/guard"

if "$INSTALL_DIR/guard" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/guard" --version
fi
# HANDWRITE-END

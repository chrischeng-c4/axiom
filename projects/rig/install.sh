#!/usr/bin/env sh
# SPEC-MANAGED: projects/rig/tech-design/semantic/rig-install-script.md#text-source-unit
# CODEGEN-BEGIN
set -eu

INSTALL_DIR="${RIG_INSTALL:-$HOME/.local/bin}"

say() { printf 'rig-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo is required for the source installer" 3
fi

if [ -n "${HOME:-}" ]; then
  PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin:$PATH"
  export PATH
fi
export CC="${CC:-/usr/bin/cc}"
export CXX="${CXX:-/usr/bin/c++}"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

cargo build -p rig-cli
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/rig" "$INSTALL_DIR/rig"
say "installed: $INSTALL_DIR/rig"

if "$INSTALL_DIR/rig" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/rig" --version
fi
# CODEGEN-END

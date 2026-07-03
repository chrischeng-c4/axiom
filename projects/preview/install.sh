#!/usr/bin/env sh
// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-install-sh" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
set -eu

INSTALL_DIR="${PREVIEW_INSTALL:-$HOME/.local/bin}"

say() { printf 'preview-install: %s\n' "$*" >&2; }
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

cargo build -p preview
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/preview" "$INSTALL_DIR/preview"
say "installed: $INSTALL_DIR/preview"

if "$INSTALL_DIR/preview" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/preview" --version
fi

// </HANDWRITE>

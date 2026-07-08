#!/usr/bin/env bash
# SPEC-MANAGED: apps/tape/tech-design/semantic/tape-build-script.md#logic
# <HANDWRITE gap="missing-generator:project-bootstrap" tracker="#768" reason="Initial Tape local build/install wrapper matching the service-project shape.">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: apps/tape/build.sh <debug|release>

debug    Build tape and install target/debug/tape to ~/.cargo/bin/tape.
release  Build/install tape with release features.

Set TAPE_INSTALL to choose the install directory; default is ~/.cargo/bin.
EOF
}

MODE="${1:-}"
if [[ $# -gt 1 ]]; then
  usage >&2
  exit 2
fi
case "$MODE" in
  -h|--help|help|"")
    usage
    exit 0
    ;;
  debug|release)
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

INSTALL_DIR="${TAPE_INSTALL:-$HOME/.cargo/bin}"
mkdir -p "$INSTALL_DIR"

if [[ "$MODE" == "debug" ]]; then
  cargo build -p tape --bin tape
  install -m 755 target/debug/tape "$INSTALL_DIR/tape"
else
  cargo build --release -p tape --bin tape --features "self-update issue"
  install -m 755 target/release/tape "$INSTALL_DIR/tape"
fi

codesign -s - -f "$INSTALL_DIR/tape" 2>/dev/null || true
echo "Installed: $("$INSTALL_DIR/tape" --version 2>/dev/null || echo tape)"
echo "Verify with: ${INSTALL_DIR}/tape --version"
# </HANDWRITE>

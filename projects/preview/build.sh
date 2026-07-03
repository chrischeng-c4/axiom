#!/usr/bin/env bash
// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-build-sh" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/preview/build.sh <debug|release>

debug    Build preview and install target/debug/preview to ~/.cargo/bin/preview.
release  Build preview in release mode and install target/release/preview.
EOF
}

MODE="${1:-}"
if [[ $# -ne 1 ]]; then
  usage >&2
  exit 2
fi

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

case "$MODE" in
  debug)
    cargo build -p preview
    install -m 755 target/debug/preview "$HOME/.cargo/bin/preview"
    ;;
  release)
    cargo build --release -p preview
    install -m 755 target/release/preview "$HOME/.cargo/bin/preview"
    ;;
  -h|--help|help)
    usage
    exit 0
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac

codesign -s - -f "$HOME/.cargo/bin/preview" 2>/dev/null || true
echo "Installed: $("$HOME/.cargo/bin/preview" --version 2>/dev/null || echo preview)"

// </HANDWRITE>

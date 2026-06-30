#!/usr/bin/env bash
set -euo pipefail

# /aw:build:release dispatcher.
#
# This skill is intentionally AW-only. It delegates to
# projects/agentic-workflow/build.sh release and rejects other project names.
#
# The AW build.sh owns release-prep: tag-collision check, version bump, release
# build, install, and commit. The skill lands, tags, pushes, and monitors the
# GitHub release after this script prints RELEASE_TAG.

MODE="release"
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

usage() {
  cat <<'EOF' >&2
Usage: /aw:build:release [aw|agentic-workflow]

Releases only Agentic Workflow (`aw`). The optional argument exists only for
compatibility with old invocations and must be `aw` or `agentic-workflow`.
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || "${1:-}" == "help" ]]; then
  usage
  exit 0
fi

if [[ $# -gt 1 ]]; then
  usage
  exit 2
fi

TARGET="${1:-aw}"
case "$TARGET" in
  aw|agentic-workflow)
    ;;
  *)
    echo "error: /aw:build:release only releases Agentic Workflow (aw), got '$TARGET'" >&2
    usage
    exit 2
    ;;
esac

BUILD="projects/agentic-workflow/build.sh"
if [[ ! -x "$BUILD" ]]; then
  echo "error: $BUILD not found or not executable" >&2
  exit 4
fi

exec "$BUILD" "$MODE"

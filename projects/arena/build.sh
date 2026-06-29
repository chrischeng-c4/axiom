#!/usr/bin/env bash
# SPEC-MANAGED: projects/arena/tech-design/semantic/arena-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/arena/build.sh <debug|release>

debug    Build arena-cli and install target/debug/arena to ~/.cargo/bin/arena.
release  Build/install arena, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/arena/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/arena --version"
}

MODE="${1:-}"
if [[ "${2:-}" == "-h" || "${2:-}" == "--help" || "${2:-}" == "help" ]]; then
  usage
  exit 0
fi
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
. scripts/project-build-lib.sh

trap 'fail_hint "$MODE"' ERR

install_arena() {
  local profile="$1"
  install -m 755 "target/${profile}/arena" "$HOME/.cargo/bin/arena"
  codesign -s - -f "$HOME/.cargo/bin/arena" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/arena" --version 2>/dev/null || echo 'arena')"
  echo "Verify with: ~/.cargo/bin/arena --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/arena/Cargo.toml projects/arena/arena-cli/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/arena/arena-cli/Cargo.toml)"
  project_build_prepare_debug_version arena "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p arena-cli
  install_arena debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/arena/Cargo.toml projects/arena/arena-cli/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/arena/arena-cli/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version arena "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p arena-cli
install_arena release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/arena
git commit --allow-empty -m "release(arena): ${TAG}"

project_build_print_release_next_steps arena "$TAG"
# CODEGEN-END

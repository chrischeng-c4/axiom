#!/usr/bin/env bash
# <HANDWRITE gap="project-root-build-script" tracker="#4158" reason="project-specific aw:build dispatch contract">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: apps/courier/build.sh <debug|release>

debug    Build courier and install target/debug/courier to ~/.cargo/bin/courier.
release  Build/install courier, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: apps/courier/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/courier --version"
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

install_courier() {
  local profile="$1"
  install -m 755 "target/${profile}/courier" "$HOME/.cargo/bin/courier"
  codesign -s - -f "$HOME/.cargo/bin/courier" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/courier" --version 2>/dev/null || echo 'courier')"
  echo "Verify with: ~/.cargo/bin/courier --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(apps/courier/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version apps/courier/Cargo.toml)"
  project_build_prepare_debug_version courier "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p courier
  install_courier debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(apps/courier/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version apps/courier/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version courier "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p courier
install_courier release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add apps/courier/Cargo.toml Cargo.lock
git commit --allow-empty -m "release(courier): ${TAG}"

project_build_print_release_next_steps courier "$TAG"
# </HANDWRITE>

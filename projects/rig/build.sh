#!/usr/bin/env bash
# SPEC-MANAGED: projects/rig/tech-design/semantic/rig-build-script.md#text-source-unit
# CODEGEN-BEGIN
# Project-root build dispatch contract (aw:build): debug installs the local
# binary; release installs, commits, and prints the tag to push after git:land.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/rig/build.sh <debug|release>

debug    Build rig-cli and install target/debug/rig to ~/.cargo/bin/rig.
release  Build/install rig, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/rig/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/rig --version"
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

install_rig() {
  local profile="$1"
  install -m 755 "target/${profile}/rig" "$HOME/.cargo/bin/rig"
  codesign -s - -f "$HOME/.cargo/bin/rig" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/rig" --version 2>/dev/null || echo 'rig')"
  echo "Verify with: ~/.cargo/bin/rig --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/rig/Cargo.toml projects/rig/rig-cli/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/rig/rig-cli/Cargo.toml)"
  project_build_prepare_debug_version rig "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p rig-cli
  install_rig debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/rig/Cargo.toml projects/rig/rig-cli/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/rig/rig-cli/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version rig "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p rig-cli
install_rig release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/rig
git commit --allow-empty -m "release(rig): ${TAG}"

project_build_print_release_next_steps rig "$TAG"
# CODEGEN-END

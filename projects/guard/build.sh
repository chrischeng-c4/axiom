#!/usr/bin/env bash
# SPEC-MANAGED: projects/guard/tech-design/semantic/guard-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/guard/build.sh <debug|release>

debug    Build guard-cli and install target/debug/guard to ~/.cargo/bin/guard.
release  Build/install guard, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/guard/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/guard --version"
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

install_guard() {
  local profile="$1"
  install -m 755 "target/${profile}/guard" "$HOME/.cargo/bin/guard"
  codesign -s - -f "$HOME/.cargo/bin/guard" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/guard" --version 2>/dev/null || echo 'guard')"
  echo "Verify with: ~/.cargo/bin/guard --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/guard/Cargo.toml projects/guard/guard-cli/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/guard/guard-cli/Cargo.toml)"
  project_build_prepare_debug_version guard "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p guard-cli
  install_guard debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/guard/Cargo.toml projects/guard/guard-cli/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/guard/guard-cli/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version guard "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p guard-cli
install_guard release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/guard
git commit --allow-empty -m "release(guard): ${TAG}"

project_build_print_release_next_steps guard "$TAG"
# CODEGEN-END

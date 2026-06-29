#!/usr/bin/env bash
# <HANDWRITE gap="project-root-build-script" tracker="mamba-build-skill" reason="project-specific aw:build dispatch contract">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/mamba/build.sh <debug|release>

debug    Build mamba and install target/debug/mamba to ~/.cargo/bin/mamba.
release  Build/install mamba, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/mamba/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/mamba --version"
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

install_mamba() {
  local profile="$1"
  install -m 755 "target/${profile}/mamba" "$HOME/.cargo/bin/mamba"
  codesign -s - -f "$HOME/.cargo/bin/mamba" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/mamba" --version 2>/dev/null || echo 'mamba')"
  echo "Verify with: ~/.cargo/bin/mamba --version"
}

VERSION_FILES=(Cargo.toml)
if [[ -f pyproject.toml ]]; then
  VERSION_FILES+=(pyproject.toml)
fi
CURRENT_VERSION="$(project_build_read_version Cargo.toml)"

if [[ "$MODE" == "debug" ]]; then
  project_build_prepare_debug_version mamba "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p mamba --bin mamba
  install_mamba debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version mamba "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p mamba --bin mamba
install_mamba release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.toml Cargo.lock
if [[ -f pyproject.toml ]]; then
  git add pyproject.toml
fi
git commit --allow-empty -m "release(mamba): ${TAG}"

project_build_print_release_next_steps mamba "$TAG"
# </HANDWRITE>

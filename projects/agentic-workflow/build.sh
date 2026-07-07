#!/usr/bin/env bash
# <HANDWRITE gap="project-root-build-script" tracker="#4158" reason="project-specific aw:build dispatch contract">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/agentic-workflow/build.sh <debug|release>

debug    Build agentic-workflow and install target/debug/aw to ~/.cargo/bin/aw.
release  Build/install aw, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/agentic-workflow/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/aw --version"
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

install_aw() {
  local profile="$1"
  install -m 755 "target/${profile}/aw" "$HOME/.cargo/bin/aw"
  codesign -s - -f "$HOME/.cargo/bin/aw" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/aw" --version 2>/dev/null || echo 'aw')"
  echo "Verify with: ~/.cargo/bin/aw --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(Cargo.toml)
  if [[ -f pyproject.toml ]]; then
    VERSION_FILES+=(pyproject.toml)
  fi
  CURRENT_VERSION="$(project_build_read_version Cargo.toml)"
  project_build_prepare_debug_version aw "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p agentic-workflow
  install_aw debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(Cargo.toml)
if [[ -f pyproject.toml ]]; then
  VERSION_FILES+=(pyproject.toml)
fi
CURRENT_VERSION="$(project_build_read_version Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version aw "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p agentic-workflow
install_aw release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.toml Cargo.lock
if [[ -f pyproject.toml ]]; then
  git add pyproject.toml
fi
git commit --allow-empty -m "release(aw): ${TAG}"

project_build_print_release_next_steps aw "$TAG"
# </HANDWRITE>

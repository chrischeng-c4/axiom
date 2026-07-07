#!/usr/bin/env bash
# SPEC-MANAGED: projects/vat/tech-design/semantic/vat-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/vat/build.sh <debug|release>

debug    Build vat and install target/debug/vat to ~/.cargo/bin/vat.
release  Build/install vat, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/vat/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/vat --version"
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

install_vat() {
  local profile="$1"
  install -m 755 "target/${profile}/vat" "$HOME/.cargo/bin/vat"
  codesign -s - -f "$HOME/.cargo/bin/vat" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/vat" --version 2>/dev/null || echo 'vat')"
  echo "Verify with: ~/.cargo/bin/vat --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/vat/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/vat/Cargo.toml)"
  project_build_prepare_debug_version vat "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p vat
  install_vat debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/vat/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/vat/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version vat "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p vat
install_vat release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/vat
git commit --allow-empty -m "release(vat): ${TAG}"

project_build_print_release_next_steps vat "$TAG"
# CODEGEN-END

#!/usr/bin/env bash
# SPEC-MANAGED: projects/meter/tech-design/semantic/meter-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/meter/build.sh <debug|release>

debug    Build meter-cli and install target/debug/meter to ~/.cargo/bin/meter.
release  Build/install meter, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/meter/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/meter --version"
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

install_meter() {
  local profile="$1"
  install -m 755 "target/${profile}/meter" "$HOME/.cargo/bin/meter"
  codesign -s - -f "$HOME/.cargo/bin/meter" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/meter" --version 2>/dev/null || echo 'meter')"
  echo "Verify with: ~/.cargo/bin/meter --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/meter/Cargo.toml projects/meter/meter-cli/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/meter/meter-cli/Cargo.toml)"
  project_build_prepare_debug_version meter "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p meter-cli
  install_meter debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/meter/Cargo.toml projects/meter/meter-cli/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/meter/meter-cli/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version meter "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p meter-cli
install_meter release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/meter
git commit --allow-empty -m "release(meter): ${TAG}"

project_build_print_release_next_steps meter "$TAG"
# CODEGEN-END

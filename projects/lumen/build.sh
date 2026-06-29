#!/usr/bin/env bash
# SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/lumen/build.sh <debug|release>

debug    Build lumen and install target/debug/lumen to ~/.cargo/bin/lumen.
release  Build/install lumen, create a release commit, and print the tag to push after git:land.

Note: this is the LOCAL/host dev install. Cross-platform release binaries
(macOS arm64 + Linux x64/arm64) are built by .github/workflows/lumen-release.yml
when the lumen@<version> tag is pushed.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/lumen/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/lumen --version"
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

install_lumen() {
  local profile="$1"
  install -m 755 "target/${profile}/lumen" "$HOME/.cargo/bin/lumen"
  codesign -s - -f "$HOME/.cargo/bin/lumen" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/lumen" --version 2>/dev/null || echo 'lumen')"
  echo "Verify with: ~/.cargo/bin/lumen --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/lumen/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/lumen/Cargo.toml)"
  project_build_prepare_debug_version lumen "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p lumen --bin lumen --features relay-wal
  install_lumen debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/lumen/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/lumen/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version lumen "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p lumen --bin lumen --features "otel relay-wal"
install_lumen release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/lumen
git commit --allow-empty -m "release(lumen): ${TAG}"

project_build_print_release_next_steps lumen "$TAG"
# CODEGEN-END

#!/usr/bin/env bash
# SPEC-MANAGED: projects/cap/tech-design/semantic/cap-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/cap/build.sh <debug|release>

debug    Build cap and install target/debug/cap to ~/.cargo/bin/cap.
release  Build/install cap, create a release commit, and print the tag to push after git:land.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/cap/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/cap --version"
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

install_cap() {
  local profile="$1"
  local strip_flag="-Wl,--gc-sections"
  if [[ "$(uname -s)" == "Darwin" ]]; then
    strip_flag="-Wl,-dead_strip"
  fi
  local cflags=(
    -Oz
    -ffunction-sections
    -fdata-sections
    -fno-stack-protector
    -fno-unwind-tables
    -fno-asynchronous-unwind-tables
    "${strip_flag}"
  )
  local frontend_flags=("${cflags[@]}")
  if [[ "$(uname -s)" == "Darwin" && "$(uname -m)" == "arm64" ]]; then
    frontend_flags+=(
      -ffreestanding
      -fno-builtin
      -nostartfiles
      -Wl,-e,_start
    )
  fi
  "${CC:-cc}" \
    "${frontend_flags[@]}" \
    projects/cap/src/cap_frontend.c \
    -o "target/${profile}/cap"
  "${CC:-cc}" \
    "${cflags[@]}" \
    projects/cap/src/cap_fast_frontend.c \
    -o "target/${profile}/cap-fast"
  install -m 755 "target/${profile}/cap" "$HOME/.cargo/bin/cap"
  install -m 755 "target/${profile}/cap-fast" "$HOME/.cargo/bin/cap-fast"
  install -m 755 "target/${profile}/cap-full" "$HOME/.cargo/bin/cap-full"
  codesign -s - -f --options runtime "$HOME/.cargo/bin/cap" 2>/dev/null || true
  codesign -s - -f --options runtime "$HOME/.cargo/bin/cap-fast" 2>/dev/null || true
  codesign -s - -f "$HOME/.cargo/bin/cap-full" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/cap" --version 2>/dev/null || echo 'cap')"
  echo "Verify with: ~/.cargo/bin/cap --version"
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(projects/cap/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version projects/cap/Cargo.toml)"
  project_build_prepare_debug_version cap "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p cap
  install_cap debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(projects/cap/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version projects/cap/Cargo.toml)"
export PROJECT_BUILD_REQUIRE_REMOTE_TAG_CHECK=1
project_build_prepare_release_version cap "$CURRENT_VERSION" "${VERSION_FILES[@]}"

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p cap
install_cap release

TAG="${PROJECT_BUILD_RELEASE_TAG}"
git add Cargo.lock projects/cap
git commit --allow-empty -m "release(cap): ${TAG}"

project_build_print_release_next_steps cap "$TAG"
# CODEGEN-END

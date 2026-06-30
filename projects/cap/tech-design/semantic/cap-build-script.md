---
id: semantic-cap-build-script
summary: Lossless source-unit coverage for the cap project build script.
capability_refs:
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: partial
    rationale: "The project build script keeps cap's CLI lifecycle and local install flow reproducible from TD."
fill_sections: [text-source-unit, changes]
---

# Semantic TD: cap/build.sh

## Source
<!-- type: text-source-unit lang: bash -->

```bash
#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/cap/build.sh <debug|release>

debug    Build cap and install target/debug/{cap,cap-fast,cap-full}.
release  Build/install cap, create a release commit, and print the tag to push after git:land.

Set CAP_INSTALL to choose the install directory; default is ~/.cargo/bin.
EOF
}

fail_hint() {
  local mode="$1"
  local install_dir="${CAP_INSTALL:-$HOME/.cargo/bin}"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/cap/build.sh ${mode}"
  echo "Verify with: ${install_dir}/cap --version"
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

INSTALL_DIR="${CAP_INSTALL:-$HOME/.cargo/bin}"

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
  mkdir -p "$INSTALL_DIR"
  install -m 755 "target/${profile}/cap" "$INSTALL_DIR/cap"
  install -m 755 "target/${profile}/cap-fast" "$INSTALL_DIR/cap-fast"
  install -m 755 "target/${profile}/cap-full" "$INSTALL_DIR/cap-full"
  codesign -s - -f --options runtime "$INSTALL_DIR/cap" 2>/dev/null || true
  codesign -s - -f --options runtime "$INSTALL_DIR/cap-fast" 2>/dev/null || true
  codesign -s - -f "$INSTALL_DIR/cap-full" 2>/dev/null || true
  echo "Installed: $("$INSTALL_DIR/cap" --version 2>/dev/null || echo 'cap')"
  echo "Verify with: ${INSTALL_DIR}/cap --version"
  local active_cap
  active_cap="$(command -v cap 2>/dev/null || true)"
  if [[ -n "$active_cap" && "$active_cap" != "$INSTALL_DIR/cap" ]]; then
    echo "Note: active cap on PATH is ${active_cap}; it shadows ${INSTALL_DIR}/cap."
    echo "      Re-run with CAP_INSTALL=$(dirname "$active_cap") to update that entry."
  fi
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/cap/build.sh"
    action: modify
    section: text-source-unit
    description: "Regenerate the cap project build script from a TD-owned text source unit."
    impl_mode: codegen
```

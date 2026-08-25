#!/usr/bin/env bash
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: apps/lumen/build.sh <debug|release>

debug    Build lumen and install target/debug/lumen to ~/.cargo/bin/lumen.
release  Prepare the local release candidate and print the next candidate workflow step.

Note: this is the LOCAL/host dev install. Cross-platform release binaries
(macOS arm64 + Linux x64/arm64) are built by the candidate workflow after
git land main.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: apps/lumen/build.sh ${mode}"
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

sync_lumen_release_image_pins() {
  local version="$1"
  shift

  local manifest matches
  for manifest in "$@"; do
    matches="$(awk '/^[[:space:]]*image:[[:space:]]*ghcr\.io\/chrischeng-c4\/lumen:[^[:space:]]+$/ { count++ } END { print count + 0 }' "$manifest")"
    if [[ "$matches" -ne 1 ]]; then
      echo "error: expected exactly one Lumen GHCR image pin in ${manifest}; found ${matches}" >&2
      return 1
    fi

    LUMEN_RELEASE_IMAGE_VERSION="$version" perl -0pi -e \
      's#(^[[:space:]]*image:[[:space:]]*ghcr\.io/chrischeng-c4/lumen:)\S+$#$1$ENV{LUMEN_RELEASE_IMAGE_VERSION}#m' \
      "$manifest"
    if ! grep -Fq "image: ghcr.io/chrischeng-c4/lumen:${version}" "$manifest"; then
      echo "error: failed to pin ${manifest} to Lumen ${version}" >&2
      return 1
    fi
  done
}

if [[ "$MODE" == "debug" ]]; then
  VERSION_FILES=(apps/lumen/Cargo.toml)
  CURRENT_VERSION="$(project_build_read_version apps/lumen/Cargo.toml)"
  project_build_prepare_debug_version lumen "$CURRENT_VERSION" "${VERSION_FILES[@]}"
  cargo build -p lumen --bin lumen --features raft-wal
  install_lumen debug
  project_build_restore_manifests
  echo ""
  echo "Build complete (debug ${PROJECT_BUILD_DEBUG_VERSION})."
  exit 0
fi

VERSION_FILES=(apps/lumen/Cargo.toml)
CURRENT_VERSION="$(project_build_read_version apps/lumen/Cargo.toml)"
sync_lumen_release_image_pins "$CURRENT_VERSION" \
  apps/lumen/k8s/base/deployment.yaml \
  apps/lumen/k8s/operator/deployment.yaml

cargo build --release --locked -p lumen --bin lumen --features release
target/release/lumen spec --format openapi > apps/lumen/clients/openapi.json
cargo test -p lumen --test spec_cli openapi_committed_snapshot_matches_live_generation -- --exact
install_lumen release

echo "Local release preparation complete for lumen@${CURRENT_VERSION}."
echo "Next: git land main, then run lumen-release-candidate from main."
# CODEGEN-END

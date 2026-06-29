#!/usr/bin/env bash
# SPEC-MANAGED: projects/cap/tech-design/semantic/cap-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/cap/build.sh <debug|release>

debug    Build cap and install target/debug/{cap,cap-fast,cap-full}.
release  Bump patch version, build/install cap, commit version files, and tag cap@<version>.

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
  cargo build -p cap
  install_cap debug
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' projects/cap/Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

NEW_PATCH=$((PATCH + 1))
NEW_MINOR=$MINOR
NEW_MAJOR=$MAJOR
if [[ "$NEW_PATCH" -gt 63 ]]; then
  NEW_PATCH=0
  NEW_MINOR=$((MINOR + 1))
fi
if [[ "$NEW_MINOR" -gt 63 ]]; then
  NEW_MINOR=0
  NEW_MAJOR=$((MAJOR + 1))
fi
NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" projects/cap/Cargo.toml

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p cap
install_cap release

TAG="cap@${NEW_VERSION}"
git add Cargo.lock projects/cap
git commit -m "release(cap): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. cap ${TAG} installed and tagged."
# CODEGEN-END

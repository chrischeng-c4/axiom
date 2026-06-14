#!/usr/bin/env bash
# SPEC-MANAGED: projects/arena/tech-design/semantic/arena-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/arena/build.sh <debug|release>

debug    Build arena-cli and install target/debug/arena to ~/.cargo/bin/arena.
release  Bump patch version, build/install arena, commit version files, and tag arena@<version>.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/arena/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/arena --version"
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

trap 'fail_hint "$MODE"' ERR

install_arena() {
  local profile="$1"
  install -m 755 "target/${profile}/arena" "$HOME/.cargo/bin/arena"
  codesign -s - -f "$HOME/.cargo/bin/arena" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/arena" --version 2>/dev/null || echo 'arena')"
  echo "Verify with: ~/.cargo/bin/arena --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p arena-cli
  install_arena debug
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' projects/arena/arena-cli/Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
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
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" projects/arena/Cargo.toml projects/arena/arena-cli/Cargo.toml

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p arena-cli
install_arena release

TAG="arena@${NEW_VERSION}"
git add Cargo.lock projects/arena
git commit -m "release(arena): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. arena ${TAG} installed and tagged."
# CODEGEN-END

#!/usr/bin/env bash
# <HANDWRITE gap="project-root-build-script" tracker="#4158" reason="project-specific aw:build dispatch contract">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/cap/build.sh <debug|release>

debug    Build cap and install target/debug/cap to ~/.cargo/bin/cap.
release  Bump patch version, build/install cap, commit version files, and tag cap@<version>.
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

trap 'fail_hint "$MODE"' ERR

install_cap() {
  local profile="$1"
  install -m 755 "target/${profile}/cap" "$HOME/.cargo/bin/cap"
  codesign -s - -f "$HOME/.cargo/bin/cap" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/cap" --version 2>/dev/null || echo 'cap')"
  echo "Verify with: ~/.cargo/bin/cap --version"
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
# </HANDWRITE>

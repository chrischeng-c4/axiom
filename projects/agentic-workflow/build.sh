#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/agentic-workflow/build.sh <debug|release>

debug    Build agentic-workflow and install target/debug/aw to ~/.cargo/bin/aw.
release  Bump patch version, build/install aw, commit version files, and tag.
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

trap 'fail_hint "$MODE"' ERR

install_aw() {
  install -m 755 target/debug/aw "$HOME/.cargo/bin/aw"
  codesign -s - -f "$HOME/.cargo/bin/aw" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/aw" --version 2>/dev/null || echo 'aw')"
  echo "Verify with: ~/.cargo/bin/aw --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p agentic-workflow
  install_aw
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
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
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
if [[ -f pyproject.toml ]]; then
  sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml
fi

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build -p agentic-workflow
install_aw

TAG="v${NEW_VERSION}"
git add Cargo.toml Cargo.lock
if [[ -f pyproject.toml ]]; then
  git add pyproject.toml
fi
git commit -m "release: ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. aw ${TAG} installed and tagged."

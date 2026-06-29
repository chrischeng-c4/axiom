#!/usr/bin/env bash
# <HANDWRITE gap="standardize:claim-code" tracker="projects-jet-build-sh" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/jet/build.sh <debug|release>

debug    Build jet and install target/debug/jet to ~/.cargo/bin/jet.
release  Bump patch version, build/install jet, commit version files, and tag jet@<version>.

Note: this is the local host install. Cross-platform release binaries
(macOS arm64 + Linux x64/arm64) are built by .github/workflows/jet-release.yml
when the jet@<version> tag is pushed.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/jet/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/jet --version"
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

RUSTUP_STABLE="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin"
if [[ -x "$RUSTUP_STABLE/cargo" ]]; then
  export PATH="$RUSTUP_STABLE:$PATH"
fi

trap 'fail_hint "$MODE"' ERR

bump_patch_version() {
  local current_version="$1"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$current_version"

  local new_patch=$((patch + 1))
  local new_minor=$minor
  local new_major=$major
  if [[ "$new_patch" -gt 63 ]]; then
    new_patch=0
    new_minor=$((minor + 1))
  fi
  if [[ "$new_minor" -gt 63 ]]; then
    new_minor=0
    new_major=$((major + 1))
  fi

  printf '%s.%s.%s\n' "$new_major" "$new_minor" "$new_patch"
}

install_jet() {
  local profile="$1"
  install -m 755 "target/${profile}/jet" "$HOME/.cargo/bin/jet"
  codesign -s - -f "$HOME/.cargo/bin/jet" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/jet" --version 2>/dev/null || echo 'jet')"
  echo "Verify with: ~/.cargo/bin/jet --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p jet --bin jet
  install_jet debug
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' projects/jet/Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
NEW_VERSION="$(bump_patch_version "$CURRENT_VERSION")"

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" projects/jet/Cargo.toml

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p jet --bin jet
install_jet release

TAG="jet@${NEW_VERSION}"
git add Cargo.lock README.md libs/cli-std projects/jet
git commit -m "release(jet): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. jet ${TAG} installed and tagged."
echo "Push the branch and tag to publish:"
echo "  git push origin HEAD"
echo "  git push origin ${TAG}"

# </HANDWRITE>

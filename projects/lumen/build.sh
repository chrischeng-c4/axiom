#!/usr/bin/env bash
# SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-build-script.md#text-source-unit
# CODEGEN-BEGIN
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/lumen/build.sh <debug|release>

debug    Build lumen and install target/debug/lumen to ~/.cargo/bin/lumen.
release  Bump patch version, build/install lumen, commit version files, and tag lumen@<version>.

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

trap 'fail_hint "$MODE"' ERR

install_lumen() {
  local profile="$1"
  install -m 755 "target/${profile}/lumen" "$HOME/.cargo/bin/lumen"
  codesign -s - -f "$HOME/.cargo/bin/lumen" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/lumen" --version 2>/dev/null || echo 'lumen')"
  echo "Verify with: ~/.cargo/bin/lumen --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p lumen --bin lumen --features relay-wal
  install_lumen debug
  echo ""
  echo "Build complete."
  exit 0
fi

# lumen uses the shared workspace version (version.workspace = true), so the
# bump lands in the root Cargo.toml — same convention as cap/vat/meter. The
# version number is a shared monotonic counter; the tag lumen@<v> is what makes
# this an independent lumen release.
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

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p lumen --bin lumen --features "otel relay-wal"
install_lumen release

TAG="lumen@${NEW_VERSION}"
git add Cargo.toml Cargo.lock projects/lumen
git commit -m "release(lumen): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. lumen ${TAG} installed and tagged."
echo "Push the tag to trigger cross-platform release binaries:"
echo "  git push origin ${TAG}"
# CODEGEN-END

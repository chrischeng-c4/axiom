#!/bin/bash
set -e

# Use git root as project directory
cd "$(git rev-parse --show-toplevel)"

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -m1 '^version = "' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

# Parse version parts
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Bump patch (base-64: minor and patch range 0–63, overflow carries up)
NEW_PATCH=$((PATCH + 1))
NEW_MINOR=$MINOR
NEW_MAJOR=$MAJOR
if [ "$NEW_PATCH" -gt 63 ]; then
    NEW_PATCH=0
    NEW_MINOR=$((MINOR + 1))
fi
if [ "$NEW_MINOR" -gt 63 ]; then
    NEW_MINOR=0
    NEW_MAJOR=$((MAJOR + 1))
fi
NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"

echo "Bumping version: $CURRENT_VERSION → $NEW_VERSION"

# Update Cargo.toml and pyproject.toml
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml

# Sync Cargo.lock so cargo detects the version change and recompiles
cargo update -w 2>/dev/null || cargo generate-lockfile

# Build and install
cargo build -p cclab-cli -p aw && \
  rm -f ~/.cargo/bin/cclab && cp target/debug/cclab ~/.cargo/bin/cclab && chmod +x ~/.cargo/bin/cclab && \
  rm -f ~/.cargo/bin/aw && cp target/debug/aw ~/.cargo/bin/aw && chmod +x ~/.cargo/bin/aw

echo "Installed: $(~/.cargo/bin/cclab --version)"
echo "Installed: $(~/.cargo/bin/aw --version 2>/dev/null || echo 'aw (no --version)')"

# Git tag
TAG="v${NEW_VERSION}"
git add Cargo.toml Cargo.lock pyproject.toml
git commit -m "release: ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"
echo ""
echo "Build complete. cclab ${TAG} installed and tagged."

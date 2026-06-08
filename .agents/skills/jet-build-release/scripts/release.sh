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
if [ -f pyproject.toml ]; then
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml
fi

# Sync Cargo.lock so cargo detects the version change and recompiles
cargo update -w 2>/dev/null || cargo generate-lockfile

# Build and install jet
cargo build -p jet
rm -f ~/.cargo/bin/jet
cp target/debug/jet ~/.cargo/bin/jet
chmod +x ~/.cargo/bin/jet
codesign -s - -f ~/.cargo/bin/jet 2>/dev/null || true

echo "Installed: $(~/.cargo/bin/jet --version 2>/dev/null || echo 'jet')"

# Git tag
TAG="jet-v${NEW_VERSION}"
git add Cargo.toml Cargo.lock
[ -f pyproject.toml ] && git add pyproject.toml
git commit -m "release(jet): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"
echo ""
echo "Build complete. jet ${TAG} installed and tagged."

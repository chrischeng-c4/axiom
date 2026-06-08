#!/bin/bash
set -e

# Use git root as project directory
cd "$(git rev-parse --show-toplevel)"

# Get current version from workspace Cargo.toml
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

# Update workspace Cargo.toml (mamba uses workspace-inherited version)
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
if [ -f pyproject.toml ]; then
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pyproject.toml
fi

# Sync Cargo.lock so cargo detects the version change and recompiles
cargo update -w 2>/dev/null || cargo generate-lockfile

# Build and install mamba (release profile — benchmark-grade)
cargo build --release -p mamba --bin mamba
rm -f ~/.cargo/bin/mamba
cp target/release/mamba ~/.cargo/bin/mamba
chmod +x ~/.cargo/bin/mamba
codesign -s - -f ~/.cargo/bin/mamba 2>/dev/null || true

echo "Installed: $(~/.cargo/bin/mamba --version 2>/dev/null || echo 'mamba')"

# Git tag
TAG="mamba-v${NEW_VERSION}"
git add Cargo.toml Cargo.lock
[ -f pyproject.toml ] && git add pyproject.toml
git commit -m "release: ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"
echo ""
echo "Build complete. mamba ${TAG} installed and tagged."

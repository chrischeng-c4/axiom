---
id: semantic-rig-build-script
summary: Lossless source-unit coverage for the rig project build script.
capability_refs:
  - id: scenario-engine
    role: primary
    claim: record-contract-check-and-json-report
    coverage: partial
    rationale: "The project-root build script keeps the rig scenario engine CLI build/install workflow runnable for agents."
fill_sections: [text-source-unit, changes]
---

# Semantic TD: rig/build.sh

## Source
<!-- type: text-source-unit lang: bash -->

```bash
#!/usr/bin/env bash
# Project-root build dispatch contract (aw:build): debug installs the local
# binary; release bumps the patch version, installs, commits, and tags.
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/rig/build.sh <debug|release>

debug    Build rig-cli and install target/debug/rig to ~/.cargo/bin/rig.
release  Bump patch version, build/install rig, commit version files, and tag rig@<version>.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/rig/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/rig --version"
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

install_rig() {
  local profile="$1"
  install -m 755 "target/${profile}/rig" "$HOME/.cargo/bin/rig"
  codesign -s - -f "$HOME/.cargo/bin/rig" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/rig" --version 2>/dev/null || echo 'rig')"
  echo "Verify with: ~/.cargo/bin/rig --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p rig-cli
  install_rig debug
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' projects/rig/rig-cli/Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
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
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" projects/rig/Cargo.toml projects/rig/rig-cli/Cargo.toml

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p rig-cli
install_rig release

TAG="rig@${NEW_VERSION}"
git add Cargo.lock projects/rig
git commit -m "release(rig): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. rig ${TAG} installed and tagged."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/build.sh"
    action: modify
    section: text-source-unit
    description: "Regenerate the rig project build script from a TD-owned text source unit."
    impl_mode: codegen
```

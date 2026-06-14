---
id: semantic-guard-build-script
summary: Lossless text-source-unit coverage for `projects/guard/build.sh`.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: compass-backed-diagnostic-scan
    claim: compass-backed-diagnostic-scan
    coverage: full
    rationale: "The source unit implements guard's compass-backed static security scan capability."
fill_sections: [overview, source, changes]
---

# Standardized guard/build.sh

## Overview
<!-- type: overview lang: markdown -->

Lossless text-source-unit coverage for `projects/guard/build.sh`.

## Source
<!-- type: text-source-unit lang: bash -->

````bash
#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: projects/guard/build.sh <debug|release>

debug    Build guard-cli and install target/debug/guard to ~/.cargo/bin/guard.
release  Bump patch version, build/install guard, commit version files, and tag guard@<version>.
EOF
}

fail_hint() {
  local mode="$1"
  echo ""
  echo "Build failed."
  echo "Retry with: projects/guard/build.sh ${mode}"
  echo "Verify with: ~/.cargo/bin/guard --version"
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

install_guard() {
  local profile="$1"
  install -m 755 "target/${profile}/guard" "$HOME/.cargo/bin/guard"
  codesign -s - -f "$HOME/.cargo/bin/guard" 2>/dev/null || true
  echo "Installed: $("$HOME/.cargo/bin/guard" --version 2>/dev/null || echo 'guard')"
  echo "Verify with: ~/.cargo/bin/guard --version"
}

if [[ "$MODE" == "debug" ]]; then
  cargo build -p guard-cli
  install_guard debug
  echo ""
  echo "Build complete."
  exit 0
fi

CURRENT_VERSION="$(grep -m1 '^version = "' projects/guard/guard-cli/Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
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
sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" projects/guard/Cargo.toml projects/guard/guard-cli/Cargo.toml

cargo update -w 2>/dev/null || cargo generate-lockfile
cargo build --release -p guard-cli
install_guard release

TAG="guard@${NEW_VERSION}"
git add Cargo.lock projects/guard
git commit -m "release(guard): ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

echo ""
echo "Build complete. guard ${TAG} installed and tagged."
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/build.sh"
    action: modify
    section: text-source-unit
    impl_mode: codegen
    description: |
      text-source-unit (td_ast) source for `projects/guard/build.sh` captured during guard standardization onto the codegen ladder.
```

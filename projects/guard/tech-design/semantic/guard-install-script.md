---
id: semantic-guard-install-script
summary: Lossless text-source-unit coverage for `projects/guard/install.sh`.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: compass-backed-diagnostic-scan
    claim: compass-backed-diagnostic-scan
    coverage: full
    rationale: "The source unit implements guard's compass-backed static security scan capability."
fill_sections: [overview, source, changes]
---

# Standardized guard/install.sh

## Overview
<!-- type: overview lang: markdown -->

Lossless text-source-unit coverage for `projects/guard/install.sh`.

## Source
<!-- type: text-source-unit lang: bash -->

````bash
#!/usr/bin/env sh
set -eu

INSTALL_DIR="${GUARD_INSTALL:-$HOME/.local/bin}"

say() { printf 'guard-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo is required for the source installer" 3
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

cargo build -p guard-cli
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/guard" "$INSTALL_DIR/guard"
say "installed: $INSTALL_DIR/guard"

if "$INSTALL_DIR/guard" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/guard" --version
fi
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/guard/install.sh"
    action: modify
    section: text-source-unit
    impl_mode: codegen
    description: |
      text-source-unit (td_ast) source for `projects/guard/install.sh` captured during guard standardization onto the codegen ladder.
```

---
id: semantic-rig-install-script
summary: Lossless text-source-unit coverage for `projects/rig/install.sh`.
capability_refs:
  - id: scenario-engine
    role: primary
    claim: record-contract-check-and-json-report
    coverage: partial
    rationale: "The project-local installer keeps rig's CLI runnable for agents from the scenario-engine capability surface."
fill_sections: [overview, source, changes]
---

# Standardized rig/install.sh

## Overview
<!-- type: overview lang: markdown -->

Lossless text-source-unit coverage for `projects/rig/install.sh`.

## Source
<!-- type: text-source-unit lang: bash -->

````bash
#!/usr/bin/env sh
set -eu

INSTALL_DIR="${RIG_INSTALL:-$HOME/.local/bin}"

say() { printf 'rig-install: %s\n' "$*" >&2; }
die() { say "error: $*"; exit "${2:-1}"; }

if ! command -v cargo >/dev/null 2>&1; then
  die "cargo is required for the source installer" 3
fi

if [ -n "${HOME:-}" ]; then
  PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.cargo/bin:$PATH"
  export PATH
fi
export CC="${CC:-/usr/bin/cc}"
export CXX="${CXX:-/usr/bin/c++}"

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

cargo build -p rig-cli
mkdir -p "$INSTALL_DIR"
install -m 755 "target/debug/rig" "$INSTALL_DIR/rig"
say "installed: $INSTALL_DIR/rig"

if "$INSTALL_DIR/rig" --version >/dev/null 2>&1; then
  "$INSTALL_DIR/rig" --version
fi
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/rig/install.sh"
    action: modify
    section: text-source-unit
    impl_mode: codegen
    description: |
      text-source-unit (td_ast) source for `projects/rig/install.sh` captured during rig standardization onto the codegen ladder.
```

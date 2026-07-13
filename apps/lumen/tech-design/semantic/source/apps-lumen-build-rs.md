---
id: projects-lumen-build-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: cli-interface
    role: primary
    claim: "service-process-interface"
    coverage: partial
    rationale: "Build script provenance stamping source-unit for lumen's CLI/build identity."
---

# Standardized apps/lumen/build.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/build.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Build script: stamp `LUMEN_GIT_SHA` and `LUMEN_BUILT_AT` into the binary
//! so `GET /version` can report provenance.
//!
//! Both are best-effort: outside a git checkout (e.g. a source tarball) the
//! sha falls back to "unknown", and the handler degrades the same way via
//! `option_env!`. Nothing here fails the build. The actual stamping logic
//! (git short-sha, built-at epoch, target triple) lives in the shared
//! `libs/build-stamp` crate so keep/loom/lumen stop carrying near-identical
//! copies; this file only supplies lumen's `LUMEN` env-var prefix.

fn main() {
    build_stamp::stamp("LUMEN");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/build.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/lumen/build.rs` captured during #39 lumen standardization.
```

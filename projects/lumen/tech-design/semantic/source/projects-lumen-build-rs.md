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

# Standardized projects/lumen/build.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/build.rs` generated from AST during Score force-regeneration standardization.

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
//! `option_env!`. Nothing here fails the build.

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Re-run when HEAD moves so the stamped sha stays current. The workspace
    // `.git` lives 2 levels up from projects/lumen/; in a linked worktree
    // `.git` is a file rather than a dir, so guard the rerun hint.
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let git_sha = short_sha().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LUMEN_GIT_SHA={git_sha}");

    // RFC3339-ish UTC timestamp without pulling in a date crate: seconds since
    // the epoch are unambiguous and trivially formattable.
    let built_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LUMEN_BUILT_AT={built_at}");

    // Stamp the exact target triple cargo built for so `lumen upgrade` can pick
    // the matching `lumen-<target>.tar.gz` release asset. Cargo always sets
    // `TARGET` for build scripts.
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=LUMEN_TARGET={target}");
}

/// Best-effort short SHA of HEAD. Returns `None` outside a git workspace.
fn short_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/build.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/build.rs` captured during #39 lumen standardization.
```

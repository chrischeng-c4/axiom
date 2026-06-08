---
id: projects-agentic-workflow-build-rs
fill_sections: [overview, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Build identity is part of the AW Core CLI runtime invariant exposed to clients and release workflows."
---

# Standardized projects/agentic-workflow/build.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/build.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/build.rs -->
```rust
//! Build script: compute `AW_BUILD_VERSION` at compile time.
//!
//! - debug profile → `<next-patch>-dev.<short-sha>` (e.g. `0.3.48-dev.ce78f95f`)
//! - release profile → `CARGO_PKG_VERSION` unchanged (e.g. `0.3.47`)
//!
//! Next-patch follows the workspace base-64 rule (minor/patch range 0–63 with
//! carry overflow). Mirrors the release-patch skill's bump logic.

use std::process::Command;

fn main() {
    // Invalidate the cached build env when profile/HEAD/version changes.
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    // Workspace Cargo.toml lives 2 levels up from projects/agentic-workflow/.
    if std::path::Path::new("../../.git/HEAD").exists() {
        println!("cargo:rerun-if-changed=../../.git/HEAD");
    }

    let pkg_version =
        std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is always set by cargo");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let build_version = if profile == "debug" {
        let next = bump_patch_base64(&pkg_version).unwrap_or_else(|| pkg_version.clone());
        let sha = short_sha().unwrap_or_else(|| "nogit".to_string());
        format!("{}-dev.{}", next, sha)
    } else {
        pkg_version
    };

    println!("cargo:rustc-env=AW_BUILD_VERSION={}", build_version);
}

/// Bump the patch segment of `X.Y.Z`, with base-64 carry (minor/patch 0-63).
fn bump_patch_base64(v: &str) -> Option<String> {
    let mut parts = v.splitn(3, '.');
    let major: u32 = parts.next()?.parse().ok()?;
    let minor: u32 = parts.next()?.parse().ok()?;
    let patch: u32 = parts.next()?.parse().ok()?;

    let (mut new_patch, mut new_minor, mut new_major) = (patch + 1, minor, major);
    if new_patch > 63 {
        new_patch = 0;
        new_minor += 1;
    }
    if new_minor > 63 {
        new_minor = 0;
        new_major += 1;
    }
    Some(format!("{}.{}.{}", new_major, new_minor, new_patch))
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/build.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```

---
id: projects-meter-src-capture-audit-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/src/capture/audit.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/audit.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `audit` | projects/meter/src/capture/audit.rs | function | pub | 25 | audit(target: impl AsRef<Path>) -> Result<AuditResult, String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/capture/audit.rs -->
````rust
//! `meter audit` capture — run `cargo audit --json` against a target crate.
//!
//! This is a thin caller over the Wave-1-fixed [`RustRunner::run_audit`], which
//! parses `cargo audit --json` stdout REGARDLESS of the process exit status
//! (cargo-audit exits non-zero exactly when advisories exist). A spawn failure
//! (cargo-audit absent) or an unparseable run surfaces here as `Err(String)`, so
//! the dispatch layer can map it to a `ToolError(4)` report rather than ever
//! fake-clean.

use std::path::Path;

use crate::rust_runner::{AuditResult, RustRunner};

/// Run `cargo audit --json` against the crate at `target` and return the parsed
/// [`AuditResult`].
///
/// `Ok(result)` means the audit ran AND parsed: `result.vulnerabilities` /
/// `result.warnings` may be empty (genuinely clean) or populated (advisories).
/// `Err(msg)` means cargo-audit could not be spawned (absent) or its output was
/// unparseable (e.g. the advisory DB could not be fetched) — the caller maps
/// this to a tool error, never a clean result.
pub fn audit(target: impl AsRef<Path>) -> Result<AuditResult, String> {
    RustRunner::for_project(target.as_ref().to_path_buf()).run_audit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_on_missing_dir_propagates_or_runs() {
        // We can't assume cargo-audit is installed in every CI sandbox, so this
        // test only asserts the call returns a Result (never panics). On a host
        // with cargo-audit + advisory DB it Ok(...)s; without it Err(...)s.
        let _ = audit("/nonexistent/path/for/meter/audit/test");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/audit.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/audit.rs` captured during meter full-codegen standardization.
```

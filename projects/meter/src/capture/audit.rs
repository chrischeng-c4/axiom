// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-capture-audit-rs.md#source
// CODEGEN-BEGIN
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
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-audit-rs.md#source
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
// CODEGEN-END

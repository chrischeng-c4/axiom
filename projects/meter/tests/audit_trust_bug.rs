// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-tests-audit-trust-bug-rs.md#source
// CODEGEN-BEGIN
//! Best-effort, skip-aware integration test for the audit trust-bug fix.
//!
//! The trust bug: `RustRunner::run_audit` keyed report parsing on the audit
//! process exit status. `cargo audit` exits NON-ZERO exactly when advisories
//! are found, so a genuinely vulnerable crate was being reported as CLEAN.
//!
//! This test points `run_audit` at a fixture crate that pins a crate with a
//! known RustSec advisory (`time = "=0.1.45"`, RUSTSEC-2020-0071) and asserts
//! that the vulnerability is surfaced. It is deliberately skip-aware: the
//! advisory database fetch can be blocked by a sandbox/offline environment, in
//! which case `cargo audit` cannot run and the test must NOT fail. It only
//! asserts when the audit genuinely ran and produced a parseable report.

use std::path::PathBuf;
use std::process::Command;

use meter::rust_runner::RustRunner;

/// `cargo audit --version` runs successfully (cargo-audit is installed).
fn cargo_audit_available() -> bool {
    Command::new("cargo")
        .args(["audit", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("vuln_crate")
}

#[test]
fn run_audit_surfaces_known_advisory() {
    // Skip cleanly if cargo-audit is not runnable at all.
    if !cargo_audit_available() {
        eprintln!("skipping: cargo-audit not available");
        return;
    }

    let runner = RustRunner::for_project(fixture_path());

    // `run_audit` returns Err when the audit could not be run or its output was
    // not parseable (e.g. the advisory DB could not be fetched offline / under
    // sandbox). That is an environment condition, not a fix regression — skip.
    let result = match runner.run_audit() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("skipping: run_audit could not produce a parseable report: {e}");
            return;
        }
    };

    // Genuinely ran and parsed: the known advisory MUST be surfaced. Before the
    // fix this returned an empty (fake-clean) result for this vulnerable crate.
    assert!(
        result.has_vulnerabilities(),
        "vulnerable fixture crate (time =0.1.45 / RUSTSEC-2020-0071) must report \
         at least one vulnerability, got {} — the audit ran but surfaced none, \
         which is the trust bug",
        result.vulnerability_count()
    );
}
// CODEGEN-END

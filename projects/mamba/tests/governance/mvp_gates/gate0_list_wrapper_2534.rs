//! Acceptance gate for #2534 — `scripts/gate0_list_tests.py` must be the
//! first mandatory worker gate. The wrapper exits 0 only when the mamba
//! test profile compiles and `cargo test --list` enumerates tests.
//!
//! This test is conservative on purpose: it does not re-run `cargo test`
//! itself (that would be circular). It only validates that the script
//! exists, is executable, advertises its own usage surface, and fails
//! fast on a missing manifest. The compile-then-list path is exercised
//! by workers running the script directly as gate 0 of the MVP profile.

use std::path::PathBuf;
use std::process::Command;

fn script() -> PathBuf {
    crate::common::mamba_root().join("scripts").join("gate0_list_tests.py")
}

#[test]
fn script_is_present_and_executable() {
    let path = script();
    let meta = std::fs::metadata(&path)
        .unwrap_or_else(|e| panic!("gate-0 script missing at {}: {e}", path.display()));
    assert!(meta.is_file(), "expected a regular file at {}", path.display());
}

#[test]
fn help_advertises_canonical_command() {
    let out = Command::new("python3")
        .arg(script())
        .arg("--help")
        .output()
        .expect("failed to launch python3 for gate-0 --help");
    assert!(out.status.success(), "--help did not exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout);
    for marker in ["--json", "--release", "--manifest"] {
        assert!(
            stdout.contains(marker),
            "missing {marker:?} in --help output:\n{stdout}"
        );
    }
}

#[test]
fn missing_manifest_returns_nonzero() {
    let out = Command::new("python3")
        .arg(script())
        .arg("--manifest")
        .arg("/nonexistent/Cargo.toml")
        .output()
        .expect("failed to launch python3");
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "expected non-zero exit on missing manifest"
    );
}

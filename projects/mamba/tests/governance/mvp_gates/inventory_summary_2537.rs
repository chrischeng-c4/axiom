//! Acceptance gate for #2537 — `scripts/inventory_summary.py` must run from a
//! clean checkout, exit zero, and emit the documented inventory categories.
//!
//! Keep this test cheap (no `cargo test --list`, no compilation of fixtures).
//! It only invokes Python 3 against a static source-tree scan, so it stays
//! safely under the MVP smoke-gate budget.

use std::process::Command;

fn run_script(args: &[&str]) -> (i32, String, String) {
    let script = crate::common::mamba_root()
        .join("scripts")
        .join("inventory_summary.py");
    let out = Command::new("python3")
        .arg(&script)
        .args(args)
        .output()
        .expect("failed to launch python3 for inventory_summary.py");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn text_inventory_runs_and_lists_categories() {
    let (code, stdout, stderr) = run_script(&[]);
    assert_eq!(
        code, 0,
        "inventory_summary.py exited non-zero. stdout={stdout} stderr={stderr}"
    );

    // Acceptance: separates normal tests, ignored tests, feature-gated tests,
    // and external-tool tests; lists fixture / bench buckets.
    for marker in [
        "Rust tests:",
        "normal:",
        "ignored:",
        "feature-gated:",
        "external-tool:",
        "Datatest fixtures:",
        "conformance (.py):",
        "cpython Lib/test seed:",
        "real-world:",
        "Bench fixtures:",
    ] {
        assert!(
            stdout.contains(marker),
            "missing inventory marker {marker:?} in stdout:\n{stdout}"
        );
    }
}

#[test]
fn json_inventory_is_machine_readable() {
    let (code, stdout, _stderr) = run_script(&["--json"]);
    assert_eq!(code, 0, "--json mode exited non-zero: {stdout}");

    // Validate by structural string checks (avoid pulling serde_json into
    // tests just for this assertion).
    for key in [
        "\"rust_tests\"",
        "\"fixtures\"",
        "\"benches\"",
        "\"normal\"",
        "\"ignored\"",
        "\"feature_gated\"",
        "\"external_tool\"",
        "\"total\"",
        "\"conformance_py\"",
        "\"cpython_lib_test_seed\"",
        "\"real_world_py\"",
    ] {
        assert!(
            stdout.contains(key),
            "missing JSON key {key} in --json output:\n{stdout}"
        );
    }
}

#[test]
fn missing_root_returns_nonzero() {
    // Acceptance: exits non-zero if the inventory cannot be computed.
    let script = crate::common::mamba_root()
        .join("scripts")
        .join("inventory_summary.py");
    let out = Command::new("python3")
        .arg(&script)
        .arg("--root")
        .arg("/nonexistent/projects/mamba/does/not/exist")
        .output()
        .expect("failed to launch python3");
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "expected non-zero exit when root is missing"
    );
}

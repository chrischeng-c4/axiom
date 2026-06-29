//! Executable metadata gate for #711: third-party/C-extension readiness must
//! stay tiered, machine-readable, and wired into replacement readiness.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn mamba_root() -> PathBuf {
    crate::common::project_root()
}

fn py_compile(paths: &[PathBuf]) {
    let output = Command::new("python3.12")
        .arg("-m")
        .arg("py_compile")
        .args(paths)
        .current_dir(mamba_root())
        .output()
        .expect("run py_compile");
    assert!(
        output.status.success(),
        "py_compile failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn third_party_readiness_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/third_party_readiness.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
    ]);
}

#[test]
fn third_party_readiness_emits_required_tiers_and_state_vocabulary() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/third_party_readiness.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run third_party_readiness");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(70),
        "third_party_readiness should be green or red, not crash\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("third-party readiness JSON");
    assert_eq!(payload["owner_issue"], "#711");
    assert_eq!(payload["schema_version"], 1);

    for tier in [
        "pure_python",
        "optional_native_acceleration",
        "mandatory_c_extension",
        "mamba_native_replacement",
    ] {
        assert!(
            payload["by_tier"].get(tier).is_some(),
            "missing #711 compatibility tier {tier}"
        );
    }

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "ready_packages",
        "not_ready_packages",
        "blocked_c_extension_packages",
        "blocked_optional_native_packages",
        "mamba_native_replacement_packages",
        "pure_python_packages",
        "optional_native_acceleration_packages",
        "mandatory_c_extension_packages",
        "managed_environment_gates",
        "missing_managed_environment_gates",
        "mambalibs_native_fixture_count",
        "mambalibs_native_import_gate_count",
        "runtime_ok",
        "runtime_fail",
        "runtime_timeout",
        "runtime_crash",
        "unmeasured",
        "missing_import_smoke_packages",
        "missing_behavior_smoke_packages",
        "missing_install_run_manifest_packages",
    ] {
        assert!(counts.contains_key(key), "missing count key {key}");
    }

    let mambalibs = payload["mambalibs_native_binding_evidence"]
        .as_object()
        .expect("mambalibs_native_binding_evidence object");
    assert_eq!(
        mambalibs
            .get("counts_as_cpython_extension_abi")
            .and_then(|v| v.as_bool()),
        Some(false),
        "mambalibs native binding evidence must not count as CPython C-extension ABI"
    );
    assert!(
        mambalibs
            .get("import_gate_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "#711 report should expose existing mambalibs import-gate evidence"
    );

    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    assert!(
        commands
            .iter()
            .any(|cmd| cmd.as_str().unwrap_or("").contains("mamba add --provider mamba")),
        "#711 evidence must include mamba-managed provider install path"
    );
}

#[test]
fn replacement_readiness_uses_third_party_readiness_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("THIRD_PARTY_READINESS"));
    assert!(text.contains("third_party_dimension"));
    assert!(
        !text.contains(
            "third-party and C-extension strategy is not yet green for replacement readiness"
        ),
        "third-party readiness must not regress to a blocked placeholder"
    );
}

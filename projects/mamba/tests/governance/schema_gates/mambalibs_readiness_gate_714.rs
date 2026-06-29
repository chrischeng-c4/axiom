//! Executable metadata gate for #714: mambalibs/native-kit readiness must stay
//! separate from pure-Python package management and CPython C-extension ABI
//! strategy while remaining wired into replacement readiness.

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

fn mambalibs_readiness_json() -> serde_json::Value {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/mambalibs_readiness.py")
        .args(["--json", "--show", "5"])
        .current_dir(mamba_root())
        .output()
        .expect("run mambalibs_readiness");
    assert!(
        output.status.code() == Some(0) || output.status.code() == Some(70),
        "mambalibs_readiness should be green or red, not crash\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("mambalibs readiness JSON")
}

#[test]
fn mambalibs_readiness_tools_are_python_parseable() {
    let root = mamba_root();
    py_compile(&[
        root.join("tests/harness/cpython/tools/mambalibs_readiness.py"),
        root.join("tests/harness/cpython/tools/replacement_readiness.py"),
        root.join("tests/harness/cpython/tools/third_party_readiness.py"),
    ]);
}

#[test]
fn mambalibs_readiness_emits_native_kit_status_vocabulary() {
    let payload = mambalibs_readiness_json();
    assert_eq!(payload["owner_issue"], "#714");
    assert_eq!(payload["schema_version"], 1);
    assert!(payload["status"] == "green" || payload["status"] == "red");

    let classes = payload["readiness_classes"]
        .as_object()
        .expect("readiness_classes object");
    let native = classes
        .get("mambalibs_native_kit")
        .expect("mambalibs_native_kit class");
    assert_eq!(
        native["counts_as_pure_python_package"].as_bool(),
        Some(false),
        "native kits must not be counted as pure-Python package readiness"
    );
    assert_eq!(
        native["counts_as_cpython_extension_abi"].as_bool(),
        Some(false),
        "native kits must not be counted as CPython C-extension ABI readiness"
    );

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "native_kits",
        "pass",
        "fail",
        "blocker",
        "fixture_manifests",
        "import_gate_manifests",
        "support_status_pass",
        "support_status_xfail",
        "support_status_blocker",
        "registered_runtime_kits",
        "unowned_gap_count",
    ] {
        assert!(counts.contains_key(key), "missing #714 count key {key}");
    }
    assert!(
        counts
            .get("native_kits")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 12,
        "#714 must inventory the native-kit denominator, not just one fixture"
    );
    assert!(
        counts
            .get("import_gate_manifests")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 10,
        "#714 must see existing mambalibs import-gate fixtures"
    );

    let by_library = payload["by_library"].as_object().expect("by_library");
    for lib in ["array", "http", "pg", "schema"] {
        let item = by_library
            .get(lib)
            .unwrap_or_else(|| panic!("missing native kit {lib}"));
        let status = item["status"].as_str().expect("kit status");
        assert!(
            matches!(status, "pass" | "fail" | "blocker"),
            "unexpected native-kit status {status}"
        );
    }
}

#[test]
fn replacement_readiness_uses_mambalibs_readiness_tool() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/replacement_readiness.py"),
    )
    .expect("read replacement_readiness.py");
    assert!(text.contains("MAMBALIBS_READINESS"));
    assert!(text.contains("mambalibs_dimension"));
    assert!(text.contains("mambalibs_native_kit"));
}

#[test]
fn third_party_readiness_delegates_native_kit_accounting_to_714() {
    let text = fs::read_to_string(
        mamba_root().join("tests/harness/cpython/tools/third_party_readiness.py"),
    )
    .expect("read third_party_readiness.py");
    assert!(text.contains("native_kit_readiness_report"));
    assert!(text.contains("mambalibs_readiness.py --json"));
    assert!(text.contains("\"#714\""));
    assert!(text.contains("counts_as_pure_python_package"));
}

#[test]
fn mambalibs_readiness_evidence_commands_are_current() {
    let payload = mambalibs_readiness_json();
    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    for needle in [
        "mambalibs_readiness.py --json",
        "cargo build -p mamba",
        "projects/mamba/mambalibs",
        "target/debug/mamba --help",
        "--test mambalibs",
    ] {
        assert!(
            commands
                .iter()
                .any(|cmd| cmd.as_str().unwrap_or("").contains(needle)),
            "#714 evidence command list missing {needle}"
        );
    }
    assert!(
        commands
            .iter()
            .all(|cmd| !cmd.as_str().unwrap_or("").contains("projects/mambalibs")),
        "#714 evidence must not keep the stale projects/mambalibs path"
    );
}

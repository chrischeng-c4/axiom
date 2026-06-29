//! Executable metadata gate for #715: legacy runtime epics must stay
//! classified against the replacement-readiness taxonomy until each old parent
//! is either proven stale, still-failing runtime work, readiness hardening, or
//! blocked by an explicit replacement-path decision.

use std::path::PathBuf;
use std::process::Command;

fn mamba_root() -> PathBuf {
    crate::common::project_root()
}

fn runtime_gap_bridge() -> PathBuf {
    mamba_root().join("tests/harness/cpython/tools/runtime_gap_bridge.py")
}

#[test]
fn runtime_gap_bridge_tool_is_python_parseable() {
    let output = Command::new("python3.12")
        .arg("-m")
        .arg("py_compile")
        .arg(runtime_gap_bridge())
        .current_dir(mamba_root())
        .output()
        .expect("run py_compile");
    assert!(
        output.status.success(),
        "runtime_gap_bridge.py must parse\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn runtime_gap_bridge_emits_complete_legacy_epic_classification() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/runtime_gap_bridge.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run runtime_gap_bridge");
    assert!(
        output.status.success(),
        "runtime_gap_bridge should produce a report, even when the report is red\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("runtime gap bridge JSON");
    assert_eq!(payload["owner_issue"], "#715");
    assert_eq!(payload["schema_version"], 1);
    assert!(
        payload["status"] == "green" || payload["status"] == "red",
        "#715 bridge status must be an explicit readiness color"
    );

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "legacy_epics",
        "failure_cache_rows",
        "unclassified",
        "closed_stale_candidates",
        "closed_stale_candidate",
        "still_failing_runtime_work",
        "readiness_test_hardening",
        "blocked",
    ] {
        assert!(counts.contains_key(key), "missing #715 count key {key}");
    }
    assert!(
        counts
            .get("legacy_epics")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 40,
        "#715 bridge must cover the old open runtime-epic backlog"
    );
    assert!(
        counts
            .get("failure_cache_rows")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            > 0,
        "#715 bridge must be grounded in the current CPython failure cache"
    );
    assert_eq!(
        counts.get("unclassified").and_then(|value| value.as_u64()),
        Some(0),
        "every old runtime epic must be classified"
    );

    let vocabulary = payload["classification_vocabulary"]
        .as_array()
        .expect("classification_vocabulary array");
    for class_name in [
        "closed_stale_candidate",
        "still_failing_runtime_work",
        "readiness_test_hardening",
        "blocked",
    ] {
        assert!(
            vocabulary
                .iter()
                .any(|value| value.as_str() == Some(class_name)),
            "missing #715 classification {class_name}"
        );
    }

    let taxonomy = payload["readiness_taxonomy"]
        .as_object()
        .expect("readiness_taxonomy object");
    for dimension in [
        "promotion_debt",
        "platform_os_process_network_tls",
        "concurrency_free_threaded",
        "debugger_introspection_profiling",
        "third_party_c_extension_strategy",
    ] {
        assert!(
            taxonomy.contains_key(dimension),
            "missing #715 replacement-readiness dimension {dimension}"
        );
    }

    let epics = payload["epics"].as_array().expect("epics array");
    for required_number in [8_u64, 15, 19, 219, 234, 241, 458] {
        assert!(
            epics
                .iter()
                .any(|epic| epic["number"].as_u64() == Some(required_number)),
            "missing legacy runtime epic #{required_number}"
        );
    }
    for epic in epics {
        assert!(
            epic["reproduction"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "legacy epic #{} needs an exact reproduction command",
            epic["number"]
        );
    }
}

#[test]
fn runtime_gap_bridge_keeps_issue_715_evidence_commands_visible() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/runtime_gap_bridge.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run runtime_gap_bridge");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("runtime gap bridge JSON");

    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    for needle in [
        "gh issue list --label project:mamba",
        "runtime_gap_bridge.py --json",
        "replacement_readiness.py --json",
        "cargo build -p mamba",
    ] {
        assert!(
            commands
                .iter()
                .any(|cmd| cmd.as_str().unwrap_or("").contains(needle)),
            "#715 evidence command list missing {needle}"
        );
    }
}

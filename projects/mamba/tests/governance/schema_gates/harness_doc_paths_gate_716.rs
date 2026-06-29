//! Executable metadata gate for #716: agent-facing CPython harness docs must
//! reference current paths and debug-build development commands.

use std::path::PathBuf;
use std::process::Command;

fn mamba_root() -> PathBuf {
    crate::common::project_root()
}

fn harness_doc_paths() -> PathBuf {
    mamba_root().join("tests/harness/cpython/tools/harness_doc_paths.py")
}

#[test]
fn harness_doc_paths_tool_is_python_parseable() {
    let output = Command::new("python3.12")
        .arg("-m")
        .arg("py_compile")
        .arg(harness_doc_paths())
        .current_dir(mamba_root())
        .output()
        .expect("run py_compile");
    assert!(
        output.status.success(),
        "harness_doc_paths.py must parse\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn harness_doc_paths_report_is_green_and_complete() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/harness_doc_paths.py")
        .args(["--json", "--show", "5"])
        .current_dir(mamba_root())
        .output()
        .expect("run harness_doc_paths");
    assert!(
        output.status.success(),
        "harness_doc_paths should be green\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("harness doc paths JSON");
    assert_eq!(payload["owner_issue"], "#716");
    assert_eq!(payload["schema_version"], 1);
    assert_eq!(payload["status"], "green");

    let counts = payload["counts"].as_object().expect("counts object");
    for key in [
        "docs_scanned",
        "required_paths",
        "missing_docs",
        "missing_required_paths",
        "forbidden_references",
        "python_commands_checked",
        "missing_python_command_paths",
    ] {
        assert!(counts.contains_key(key), "missing #716 count key {key}");
    }
    assert!(
        counts
            .get("docs_scanned")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            >= 7,
        "#716 must scan the README, production gate, conventions, issue template, and issue-loop docs"
    );
    for zero_key in [
        "missing_docs",
        "missing_required_paths",
        "forbidden_references",
        "missing_python_command_paths",
    ] {
        assert_eq!(
            counts.get(zero_key).and_then(|value| value.as_u64()),
            Some(0),
            "#716 report must have zero {zero_key}"
        );
    }

    let docs = payload["docs"].as_array().expect("docs array");
    for required_doc in [
        "projects/mamba/README.md",
        "projects/mamba/tests/README.md",
        "projects/mamba/tests/PRODUCTION-GATE.md",
        ".github/ISSUE_TEMPLATE/axis1-seed.md",
        "projects/mamba/issue-loop.md",
    ] {
        assert!(
            docs.iter().any(|doc| doc.as_str() == Some(required_doc)),
            "missing scanned doc {required_doc}"
        );
    }
}

#[test]
fn harness_doc_paths_evidence_commands_are_current() {
    let output = Command::new("python3.12")
        .arg("tests/harness/cpython/tools/harness_doc_paths.py")
        .args(["--json", "--show", "1"])
        .current_dir(mamba_root())
        .output()
        .expect("run harness_doc_paths");
    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("harness doc paths JSON");

    let commands = payload["evidence_commands"]
        .as_array()
        .expect("evidence_commands array");
    for needle in [
        "harness_doc_paths.py --json",
        "wall_status.py --help",
        "gate_check.py --help",
        "harness_doc_paths_gate_716",
    ] {
        assert!(
            commands
                .iter()
                .any(|cmd| cmd.as_str().unwrap_or("").contains(needle)),
            "#716 evidence command list missing {needle}"
        );
    }
}

// HANDWRITE-BEGIN gap="missing-generator:e2e-test:python-ec-review-lock" tracker="#2294" reason="The fixture drives direct Python EC review and lock invalidation without generating an EC scaffold."
//! Python EC review and lock acceptance tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-ec-review-lock.md#unit-test

use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write_project(root: &Path) {
    fs::create_dir_all(root.join("projects/demo/external-contracts/src")).unwrap();
    fs::create_dir_all(root.join("projects/demo/external-contracts/evidence")).unwrap();
    fs::write(
        root.join("aw.toml"),
        r#"
[[projects]]
name = "demo"
path = "projects/demo"
artifact_model = "python-v1"

[[projects.workspaces]]
name = "demo"
paths = ["projects/demo/**"]
target = "python"
test_cmd = "python -m pytest"
"#,
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/CAPABILITIES.md"),
        r#"
# Demo

## Brief

Python EC review fixture.

## Capabilities

### Capability Index

### Contract Boundary

ID: demo-contract
Promise: A hand-authored Python EC proves the fixture boundary.
"#,
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/pyproject.toml"),
        r#"
[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "python-ec-author"
efficiency_policy = "required"

[[tool.aw.python-ec.cases]]
id = "demo-contract-behavior"
artifact_id = "artifact:demo/contract"
capability_id = "demo-contract"
use_case_id = "happy-path"
dimension = "behavior"
applicability = "td"
test_path = "src/behavior.py"
promise = "The contract boundary accepts the happy path."
oracle = "fixture-target"
target = "python"
command = "test -s projects/demo/external-contracts/evidence/behavior.json"
evidence_paths = ["evidence/behavior.json"]

[[tool.aw.python-ec.cases]]
id = "demo-contract-security"
artifact_id = "artifact:demo/contract"
capability_id = "demo-contract"
use_case_id = "authz-boundary"
dimension = "security"
applicability = "td"
test_path = "src/security.py"
promise = "The contract boundary rejects unauthorized access."
oracle = "fixture-target"
target = "python"
command = "test -s projects/demo/external-contracts/evidence/security.json"
evidence_paths = ["evidence/security.json"]

[[tool.aw.python-ec.cases]]
id = "demo-contract-stability"
artifact_id = "artifact:demo/contract"
capability_id = "demo-contract"
use_case_id = "restart-boundary"
dimension = "stability"
applicability = "post-gen"
test_path = "src/stability.py"
promise = "The contract remains available across a restart boundary."
oracle = "fixture-target"
threshold = "restart completes within 5 seconds"
target = "python"
command = "test -s projects/demo/external-contracts/evidence/stability.json"
evidence_paths = ["evidence/stability.json"]

[[tool.aw.python-ec.cases]]
id = "demo-contract-efficiency"
artifact_id = "artifact:demo/contract"
capability_id = "demo-contract"
use_case_id = "latency-budget"
dimension = "efficiency"
applicability = "post-gen"
test_path = "src/efficiency.py"
promise = "The contract respects its latency budget."
oracle = "fixture-target"
threshold = "p95 under 100ms"
target = "python"
command = "test -s projects/demo/external-contracts/evidence/efficiency.json"
evidence_paths = ["evidence/efficiency.json"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/runner.py"),
        "raise RuntimeError('review and lock must not run Python')\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/behavior.py"),
        "def contract() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/security.py"),
        "def contract() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/stability.py"),
        "def contract() -> None:\n    assert True\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/efficiency.py"),
        "def contract() -> None:\n    assert True\n",
    )
    .unwrap();
}

fn run_aw(root: &Path, args: &[&str]) -> (std::process::Output, Value) {
    let output = run_aw_raw(root, args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "aw did not emit JSON: {error}\nargs={args:?}\nstdout={stdout}\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output, value)
}

fn run_aw_raw(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run aw")
}

fn accept_payload(path: &Path, reviewed_by: &str) {
    let mut payload: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    payload["decision"] = Value::String("accepted".to_string());
    payload["reviewer_kind"] = Value::String("agent".to_string());
    payload["reviewed_by"] = Value::String(reviewed_by.to_string());
    payload["summary"] = Value::String(
        "Independent reviewer confirmed the Python bundle has concrete, independent contract cases."
            .to_string(),
    );
    payload["checklist"] = serde_json::json!({
        "capability_claim_coverage": true,
        "required_dimensions": true,
        "assertions_specific": true,
        "oracle_independent": true,
        "loopholes_checked": true,
        "false_green_risk_checked": true,
    });
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
}

fn assert_staged_dimensions(root: &Path) {
    for name in [
        "behavior.json",
        "security.json",
        "stability.json",
        "efficiency.json",
    ] {
        fs::write(
            root.join("projects/demo/external-contracts/evidence")
                .join(name),
            "external target evidence\n",
        )
        .unwrap();
    }
    let (core, core_summary) = run_aw(
        root,
        &[
            "ec",
            "--project",
            "demo",
            "verify",
            "--stage",
            "core",
            "--json",
        ],
    );
    assert!(core.status.success());
    assert_eq!(core_summary["passed_count"], 2);
    assert_eq!(core_summary["failed_count"], 0);
    assert!(core_summary["results"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|result| result["category"] == "stability" || result["category"] == "efficiency")
        .all(|result| result["status"] == "skipped"));

    fs::remove_file(root.join("projects/demo/external-contracts/evidence/efficiency.json"))
        .unwrap();
    let (operational, operational_summary) = run_aw(
        root,
        &[
            "ec",
            "--project",
            "demo",
            "verify",
            "--stage",
            "operational",
            "--json",
        ],
    );
    assert!(!operational.status.success());
    assert_eq!(operational_summary["passed_count"], 1);
    assert_eq!(operational_summary["failed_count"], 1);
}

#[test]
fn ec_python_review_lock_binds_complete_bundle_and_rejects_self_review() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path());

    let (locked, lock) = run_aw(temp.path(), &["ec", "--project", "demo", "lock", "--json"]);
    assert!(locked.status.success());
    assert_eq!(lock["clean"], true);
    assert_eq!(
        lock["inventory_path"],
        "projects/demo/external-contracts/pyproject.toml"
    );
    assert_eq!(
        lock["source_count"], 6,
        "five Python sources plus pyproject"
    );
    let lock_source =
        fs::read_to_string(temp.path().join("projects/demo/external-contracts/ec.lock")).unwrap();
    assert!(lock_source.contains("src/behavior.py"));
    assert!(lock_source.contains("pyproject.toml"));

    let (pending, review) = run_aw(
        temp.path(),
        &["ec", "--project", "demo", "review", "--json"],
    );
    assert!(pending.status.success());
    assert_eq!(review["status"], "pending_agent_review");
    let payload_path = Path::new(review["payload_path"].as_str().unwrap());
    assert!(payload_path.is_file());

    accept_payload(payload_path, "python-ec-author");
    let evidence = payload_path.to_string_lossy().into_owned();
    let self_review = run_aw_raw(
        temp.path(),
        &[
            "ec",
            "--project",
            "demo",
            "review",
            "--evidence-file",
            &evidence,
            "--json",
        ],
    );
    assert!(
        !self_review.status.success(),
        "same author must not self-review"
    );
    assert!(String::from_utf8_lossy(&self_review.stderr).contains("not independent"));

    accept_payload(payload_path, "independent-python-reviewer");
    let (accepted, accepted_review) = run_aw(
        temp.path(),
        &[
            "ec",
            "--project",
            "demo",
            "review",
            "--evidence-file",
            &evidence,
            "--json",
        ],
    );
    assert!(accepted.status.success());
    assert_eq!(accepted_review["status"], "accepted");
    assert_eq!(
        accepted_review["next"], "aw ec lock --project demo",
        "direct Python EC review must proceed to locking, never legacy EC generation"
    );
    let accepted_digest = accepted_review["source_digest"]
        .as_str()
        .unwrap()
        .to_string();

    fs::write(
        temp.path()
            .join("projects/demo/external-contracts/src/behavior.py"),
        "def contract() -> None:\n    assert 'changed' == 'changed'\n",
    )
    .unwrap();
    let (source_stale, source_lock) = run_aw(
        temp.path(),
        &["ec", "--project", "demo", "lock", "--check", "--json"],
    );
    assert!(!source_stale.status.success());
    assert_eq!(source_lock["status"], "stale");
    assert!(source_lock["changed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry == "source:projects/demo/external-contracts/src/behavior.py"));

    let (new_pending, new_review) = run_aw(
        temp.path(),
        &["ec", "--project", "demo", "review", "--json"],
    );
    assert!(new_pending.status.success());
    assert_eq!(new_review["status"], "pending_agent_review");
    assert_ne!(new_review["source_digest"], accepted_digest);

    let (relocked, _) = run_aw(temp.path(), &["ec", "--project", "demo", "lock", "--json"]);
    assert!(relocked.status.success());
    let pyproject = temp
        .path()
        .join("projects/demo/external-contracts/pyproject.toml");
    let contents = fs::read_to_string(&pyproject).unwrap();
    fs::write(
        &pyproject,
        contents.replace("python-ec-author", "new-python-ec-author"),
    )
    .unwrap();
    let (dependency_stale, dependency_lock) = run_aw(
        temp.path(),
        &["ec", "--project", "demo", "lock", "--check", "--json"],
    );
    assert!(!dependency_stale.status.success());
    assert_eq!(dependency_lock["status"], "stale");
    assert!(dependency_lock["changed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry == "source:projects/demo/external-contracts/pyproject.toml"));
}

#[test]
fn ec_python_staged_dimensions() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path());

    let (pending, review) = run_aw(
        temp.path(),
        &["ec", "--project", "demo", "review", "--json"],
    );
    assert!(pending.status.success());
    let payload_path = Path::new(review["payload_path"].as_str().unwrap());
    accept_payload(payload_path, "independent-python-reviewer");
    let evidence = payload_path.to_string_lossy().into_owned();
    let (accepted, _) = run_aw(
        temp.path(),
        &[
            "ec",
            "--project",
            "demo",
            "review",
            "--evidence-file",
            &evidence,
            "--json",
        ],
    );
    assert!(accepted.status.success());

    assert_staged_dimensions(temp.path());
}
// HANDWRITE-END

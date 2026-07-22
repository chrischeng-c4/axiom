// HANDWRITE-BEGIN gap="missing-generator:e2e-test:python-ec-inventory" tracker="#2293" reason="The fixture executes the real aw ec check boundary against a hand-authored Python project without invoking a Python module."
//! Python EC inventory acceptance tests.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-ec-inventory-check.md#unit-test

use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write_project(root: &Path, cases: &str) {
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

Python EC fixture.

## Capabilities

### Capability Index

### Contract Boundary

ID: demo-contract
Promise: The fixture exposes one capability target for its external contract.
"#,
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/pyproject.toml"),
        format!(
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
efficiency_policy = "not-applicable"

{cases}
"#
        ),
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/runner.py"),
        "raise RuntimeError('ec inventory check must not import or run Python')\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/behavior.py"),
        "def contract() -> None:\n    pass\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/security.py"),
        "def contract() -> None:\n    pass\n",
    )
    .unwrap();
}

fn run_check(root: &Path) -> (std::process::Output, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(["ec", "--project", "demo", "check", "--json"])
        .current_dir(root)
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run aw ec check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "aw ec check did not emit the inventory summary: {error}\nstdout={stdout}\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output, value)
}

const VALID_CASES: &str = r#"
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
"#;

#[test]
fn ec_python_inventory_check_accepts_hand_authored_project_without_generated_aw_toml() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), VALID_CASES);

    let (output, summary) = run_check(temp.path());

    assert!(
        output.status.success(),
        "python inventory check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(summary["clean"], true);
    assert_eq!(summary["configured"], true);
    assert_eq!(summary["case_count"], 2);
    assert_eq!(
        summary["inventory_path"],
        "projects/demo/external-contracts/pyproject.toml"
    );
    assert!(
        !temp.path().join("projects/demo/aw.toml").exists(),
        "Python mode must not require or generate a project aw.toml inventory"
    );
}

#[test]
fn ec_python_inventory_check_rejects_duplicate_unknown_dimension_missing_reference_and_bad_applicability(
) {
    let temp = tempfile::tempdir().unwrap();
    write_project(
        temp.path(),
        r#"
[[tool.aw.python-ec.cases]]
id = "duplicate-case"
capability_id = "demo-contract"
use_case_id = "happy-path"
dimension = "usability"
applicability = "later"
test_path = "src/behavior.py"

[[tool.aw.python-ec.cases]]
id = "duplicate-case"
capability_id = ""
use_case_id = "security-path"
dimension = "security"
applicability = "td"
test_path = "src/security.py"
"#,
    );

    let (output, summary) = run_check(temp.path());
    let findings = summary["findings"].as_array().unwrap();
    let findings = findings
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !output.status.success(),
        "invalid inventory must fail closed"
    );
    assert_eq!(summary["clean"], false);
    assert!(findings.contains("duplicate case id `duplicate-case`"));
    assert!(findings.contains("unknown dimension `usability`"));
    assert!(findings.contains("missing `artifact_id`"));
    assert!(findings.contains("missing `capability_id`"));
    assert!(findings.contains("invalid applicability `later`"));
}

#[test]
fn ec_python_inventory_check_keeps_legacy_dispatch_when_python_mode_is_not_opted_in() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), VALID_CASES);
    let config_path = temp.path().join("aw.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    fs::write(
        &config_path,
        config.replace("artifact_model = \"python-v1\"\n", ""),
    )
    .unwrap();

    let (output, summary) = run_check(temp.path());

    assert!(
        output.status.success(),
        "legacy check must ignore the Python inventory unless opted in"
    );
    assert_eq!(summary["clean"], true);
    assert_eq!(summary["configured"], false);
    assert_eq!(summary["inventory_path"], "projects/demo/aw.toml");
}
// HANDWRITE-END

// HANDWRITE-BEGIN gap="missing-generator:e2e-test:python-artifact-readiness" tracker="#2304" reason="The fixture proves the shared Python readiness consumer without a generated Python framework."
use agentic_workflow::services::python_artifact_readiness::evaluate;
use std::{fs, path::Path};

fn write_project(root: &Path, write_evidence: bool) {
    fs::create_dir_all(root.join("projects/demo/tech-design/src/demo/domain")).unwrap();
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
        root.join("projects/demo/tech-design/src/demo/domain/order.py"),
        "class Order:\n    pass\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/runner.py"),
        "raise RuntimeError('readiness must not execute Python')\n",
    )
    .unwrap();
    fs::write(
        root.join("projects/demo/external-contracts/src/behavior.py"),
        "def contract() -> None:\n    pass\n",
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
author = "fixture-author"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "order-behavior"
artifact_id = "artifact:orders/create-order"
capability_id = "orders"
use_case_id = "create-order"
dimension = "behavior"
applicability = "td"
test_path = "src/behavior.py"
promise = "orders are created"
oracle = "fixture-target"
target = "python"
command = "test -s evidence/behavior.json"
evidence_paths = ["evidence/behavior.json"]
"#,
    )
    .unwrap();
    if write_evidence {
        fs::write(
            root.join("projects/demo/external-contracts/evidence/behavior.json"),
            "{\"ok\":true}\n",
        )
        .unwrap();
    }
}

#[test]
fn python_artifact_readiness_reports_shared_ids_dimensions_and_digests() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), true);

    let readiness = evaluate(root.path(), "demo")
        .unwrap()
        .expect("python-v1 projection");

    assert!(readiness.ready, "{:#?}", readiness.blockers);
    assert_eq!(
        readiness.td_module_ids,
        vec!["module:src.demo.domain.order"]
    );
    assert_eq!(readiness.required_case_count, 1);
    assert_eq!(readiness.ready_case_count, 1);
    assert_eq!(readiness.cases[0].id, "order-behavior");
    assert_eq!(readiness.cases[0].dimension, "behavior");
    assert_eq!(readiness.cases[0].applicability, "td");
    assert!(readiness
        .ec_source_digest
        .as_deref()
        .unwrap()
        .starts_with("sha256:"));
    assert!(readiness
        .dependency_lock_digest
        .as_deref()
        .unwrap()
        .starts_with("sha256:"));
}

#[test]
fn python_artifact_readiness_routes_missing_evidence_to_one_stage_command() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), false);

    let readiness = evaluate(root.path(), "demo")
        .unwrap()
        .expect("python-v1 projection");

    assert!(!readiness.ready);
    assert_eq!(readiness.ready_case_count, 0);
    assert_eq!(
        readiness.next_command.as_deref(),
        Some("aw ec verify --project demo --stage td")
    );
}

#[test]
fn python_artifact_readiness_leaves_legacy_projects_unchanged() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), true);
    let config = fs::read_to_string(root.path().join("aw.toml")).unwrap();
    fs::write(
        root.path().join("aw.toml"),
        config.replace(
            "artifact_model = \"python-v1\"",
            "artifact_model = \"legacy\"",
        ),
    )
    .unwrap();

    assert!(evaluate(root.path(), "demo").unwrap().is_none());
}
// HANDWRITE-END

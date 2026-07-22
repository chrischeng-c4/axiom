// @spec apps/agentic-workflow/tech-design/core/logic/issues/project-plan-transaction.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-project-plan-transaction" tracker="#2388" reason="The compiled CLI fixture proves digest-bound snapshot preflight, ordered publication, duplicate-free reapply, and pre-write tracker drift rejection."

use agentic_workflow::issues::LocalBackend;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn write_project(root: &Path) {
    fs::write(
        root.join("aw.toml"),
        r#"
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
"#,
    )
    .unwrap();
}

fn write_epic(root: &Path) -> PathBuf {
    let backend = LocalBackend::from_project_root(root);
    let open = backend.issues_dir().join("open");
    fs::create_dir_all(&open).unwrap();
    let path = open.join("42.md");
    fs::write(
        &path,
        r#"---
type: epic
title: Reviewed delivery
state: open
github_id: 42
labels:
  - type:epic
  - app:demo
  - priority:p1
---

## Requirements

- R1: Publish one reviewed atomic change.
"#,
    )
    .unwrap();
    path
}

fn write_mixed_horizon_epic(root: &Path, with_active_change: bool) {
    let backend = LocalBackend::from_project_root(root);
    let open = backend.issues_dir().join("open");
    fs::create_dir_all(&open).unwrap();
    fs::write(
        open.join("42.md"),
        r#"---
type: epic
title: Mixed reviewed delivery
state: open
github_id: 42
labels:
  - type:epic
  - app:demo
  - priority:p1
---

## Requirements

- R1: Deliver the active reviewed boundary.
- R2: Later phase: deliver the deferred reviewed boundary.
"#,
    )
    .unwrap();
    if with_active_change {
        fs::write(
            open.join("43.md"),
            r#"---
type: change
title: Deliver the active reviewed boundary
state: open
github_id: 43
labels:
  - type:change
  - app:demo
  - priority:p1
  - epic:42
---

## Capability Alignment

Capability: work-item-planning

## Scope

### In Scope
- Deliver the active reviewed boundary.

### Out of Scope
- Other delivery.

## Acceptance Criteria

- The active reviewed boundary is delivered.

## Reference Context

- Parent: #42.
"#,
        )
        .unwrap();
    }
}

fn run_aw(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .env("AW_AGENT_ID", "author-agent")
        .env("AW_DISABLE_CAP", "1")
        .output()
        .expect("run repo-built aw")
}

fn successful_json(output: &Output, command: &str) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{command} failed:\nstdout={stdout}\nstderr={stderr}"
    );
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!("{command} did not emit JSON: {error}\nstdout={stdout}\nstderr={stderr}")
    })
}

fn accept_review(path: &Path) {
    let mut payload: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    payload["decision"] = Value::String("accepted".to_string());
    payload["reviewed_by"] = Value::String("independent-reviewer".to_string());
    payload["summary"] = Value::String(
        "Reviewed the exact tracker snapshot and complete mutation manifest.".to_string(),
    );
    for key in [
        "scope_coverage",
        "bounded_candidates",
        "tracker_reconciliation",
        "priority_consistent",
        "no_duplicate_wis",
        "publication_safe",
    ] {
        payload["checklist"][key] = Value::Bool(true);
    }
    payload["findings"] = Value::Array(Vec::new());
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
}

fn open_issue_bodies(root: &Path) -> Vec<String> {
    let backend = LocalBackend::from_project_root(root);
    let mut bodies = fs::read_dir(backend.issues_dir().join("open"))
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect::<Vec<_>>();
    bodies.sort();
    bodies
}

#[test]
fn accepted_project_plan_applies_once_and_reapply_is_clean_noop() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_epic(root.path());

    let planned = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    let planned = successful_json(&planned, "aw wi plan");
    let payload_path = PathBuf::from(planned["payload_path"].as_str().unwrap());
    let payload: Value = serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    let manifest_path = PathBuf::from(payload["manifest_path"].as_str().unwrap());
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["schema"], "aw.wi.project-plan-transaction.v1");
    assert_eq!(manifest["project"], "demo");
    assert_eq!(manifest["issue_snapshots"].as_array().unwrap().len(), 1);
    assert!(
        manifest["tracker_snapshot_digest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:"))
    );
    assert_eq!(manifest["apply_command"], payload["next_command"]);
    assert_eq!(planned["next"], manifest["apply_command"]);
    assert!(
        manifest["mutations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mutation| mutation["action"] == "create")
    );
    assert!(
        manifest["mutations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|mutation| mutation["action"] != "update")
    );

    accept_review(&payload_path);
    let evidence = payload_path.to_string_lossy().to_string();
    let applied = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let applied = successful_json(&applied, "aw wi plan-review");
    assert_eq!(applied["status"], "accepted");
    assert_eq!(applied["transaction"]["status"], "complete");
    assert_eq!(applied["transaction"]["no_op"], false);
    assert_eq!(applied["transaction"]["created_issue_count"], 1);
    assert_eq!(applied["next"]["command"], manifest["terminal_next"]);

    let bodies = open_issue_bodies(root.path());
    assert_eq!(bodies.len(), 2);
    assert_eq!(
        bodies
            .iter()
            .filter(|body| body.contains("aw:planning-transaction"))
            .count(),
        1,
        "a graph-clean epic must not receive a provenance-only mutation"
    );
    assert!(
        bodies
            .iter()
            .any(|body| body.contains("epic:42") && body.contains("type: change"))
    );

    let review_path = PathBuf::from(applied["review_path"].as_str().unwrap());
    let review_evidence = review_path.to_string_lossy().to_string();
    let repeated = run_aw(
        root.path(),
        &[
            "wi",
            "plan-review",
            "--evidence-file",
            &review_evidence,
            "--json",
        ],
    );
    let repeated = successful_json(&repeated, "reapply aw wi plan-review");
    assert_eq!(repeated["transaction"]["no_op"], true);
    assert_eq!(repeated["transaction"]["applied_count"], 0);
    assert_eq!(open_issue_bodies(root.path()).len(), 2);
    assert!(
        review_path.exists(),
        "reapply must retain durable review evidence"
    );

    let post_apply = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "post-apply aw wi plan",
    );
    let post_payload = PathBuf::from(post_apply["payload_path"].as_str().unwrap());
    let post_payload: Value =
        serde_json::from_str(&fs::read_to_string(post_payload).unwrap()).unwrap();
    let post_manifest = PathBuf::from(post_payload["manifest_path"].as_str().unwrap());
    let post_manifest: Value =
        serde_json::from_str(&fs::read_to_string(post_manifest).unwrap()).unwrap();
    assert!(
        post_manifest["mutations"].as_array().unwrap().is_empty(),
        "post-apply plan must converge to zero mutations:\n{}",
        serde_json::to_string_pretty(&post_manifest).unwrap()
    );

    let unchanged = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "unchanged post-apply aw wi plan",
    );
    assert_eq!(unchanged["plan_digest"], post_apply["plan_digest"]);
}

#[test]
fn mixed_horizon_publication_reparents_existing_changes_and_converges() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_mixed_horizon_epic(root.path(), true);

    let planned = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "initial mixed-horizon aw wi plan",
    );
    let payload_path = PathBuf::from(planned["payload_path"].as_str().unwrap());
    let payload: Value = serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    let plan_path = PathBuf::from(payload["plan_path"].as_str().unwrap());
    let plan: Value = serde_json::from_str(&fs::read_to_string(plan_path).unwrap()).unwrap();
    assert_eq!(
        plan["proposed_epics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|epic| epic["source_epic"] == "42")
            .count(),
        2,
        "the initial mixed epic must create one active and one deferred sibling"
    );

    accept_review(&payload_path);
    let evidence = payload_path.to_string_lossy().to_string();
    let applied = successful_json(
        &run_aw(
            root.path(),
            &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
        ),
        "apply mixed-horizon project plan",
    );
    assert_eq!(applied["transaction"]["created_issue_count"], 3);
    successful_json(
        &run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]),
        "post-apply mixed-horizon aw wi graph",
    );

    let post_apply = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "post-apply mixed-horizon aw wi plan",
    );
    let post_payload_path = PathBuf::from(post_apply["payload_path"].as_str().unwrap());
    let post_payload: Value =
        serde_json::from_str(&fs::read_to_string(post_payload_path).unwrap()).unwrap();
    let post_plan_path = PathBuf::from(post_payload["plan_path"].as_str().unwrap());
    let post_plan: Value =
        serde_json::from_str(&fs::read_to_string(post_plan_path).unwrap()).unwrap();
    assert!(
        post_plan["proposed_epics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|epic| epic["source_epic"] != "42"),
        "the original mixed epic must be represented by its published siblings"
    );
    let retained_change = post_plan["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|change| change["id"] == "43")
        .collect::<Vec<_>>();
    assert_eq!(
        retained_change.len(),
        1,
        "an existing source change must remain represented exactly once"
    );
    let active_sibling = post_plan["epics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|epic| {
            epic["horizon"] == "active" && epic["title"] == "Mixed reviewed delivery - active"
        })
        .expect("published active sibling must be in the replan");
    assert_eq!(retained_change[0]["owner_epic"], active_sibling["id"]);
    let manifest_path = PathBuf::from(post_payload["manifest_path"].as_str().unwrap());
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    assert!(
        manifest["mutations"].as_array().unwrap().is_empty(),
        "mixed-horizon post-publication plan must converge:\n{}",
        serde_json::to_string_pretty(&manifest).unwrap()
    );

    let unchanged = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "unchanged mixed-horizon aw wi plan",
    );
    assert_eq!(unchanged["plan_digest"], post_apply["plan_digest"]);
}

#[test]
fn tracker_drift_after_review_fails_before_any_mutation() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let epic_path = write_epic(root.path());
    let planned = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    let planned = successful_json(&planned, "aw wi plan");
    let payload_path = PathBuf::from(planned["payload_path"].as_str().unwrap());
    accept_review(&payload_path);

    let body = fs::read_to_string(&epic_path).unwrap();
    fs::write(
        &epic_path,
        body.replace(
            "title: Reviewed delivery",
            "title: Externally drifted delivery",
        ),
    )
    .unwrap();
    let evidence = payload_path.to_string_lossy().to_string();
    let rejected = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("reviewed issue `42` changed"), "{stderr}");
    let bodies = open_issue_bodies(root.path());
    assert_eq!(bodies.len(), 1);
    assert!(!bodies[0].contains("aw:planning-transaction"));
}

// HANDWRITE-END

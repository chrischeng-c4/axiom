// @spec apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md#cli
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:inventory-plan-agent-review" tracker="#2190" reason="The fixture drives every compiled compatibility planning verb through one digest-bound independent project-plan review."

use agentic_workflow::issues::LocalBackend;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn write_project(root: &Path, review_backing: Option<&str>) {
    let policy = review_backing
        .map(|backing| format!("planning_review_backing = \"{backing}\"\n"))
        .unwrap_or_default();
    fs::write(
        root.join("aw.toml"),
        format!(
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
{policy}
[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"
"#,
        ),
    )
    .unwrap();

    let local = LocalBackend::from_project_root(root);
    let open = local.issues_dir().join("open");
    fs::create_dir_all(&open).unwrap();
    fs::write(
        open.join("42.md"),
        r#"---
type: epic
title: Demo planning epic
state: open
github_id: 42
labels:
  - type:epic
  - app:demo
  - priority:p1
---
## Problem

Plan one bounded project outcome.

## Requirements

- R1: Reconcile the existing epic and change inventory.

## Scope

### In Scope
- Planning review.

### Out of Scope
- Product implementation.
"#,
    )
    .unwrap();
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
        panic!("{command} did not emit one JSON value: {error}\nstdout={stdout}\nstderr={stderr}")
    })
}

fn accept_review_payload(path: &Path, reviewer: &str) {
    let mut payload: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    payload["decision"] = Value::String("accepted".to_string());
    payload["reviewed_by"] = Value::String(reviewer.to_string());
    payload["summary"] = Value::String("Independent inventory review accepted.".to_string());
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

fn request_revision_payload(path: &Path, reviewer: &str) {
    let mut payload: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    payload["decision"] = Value::String("needs_revision".to_string());
    payload["reviewed_by"] = Value::String(reviewer.to_string());
    payload["summary"] = Value::String("Inventory review found a planning gap.".to_string());
    payload["findings"] = Value::Array(vec![Value::String(
        "Split the mixed-scope candidate before publication.".to_string(),
    )]);
    fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
}

#[test]
fn planning_verbs_delegate_to_one_agent_first_digest_bound_project_plan() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), None);
    let root_plan = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "aw wi plan",
    );
    assert_eq!(root_plan["root"]["kind"], "project_plan");
    assert_eq!(root_plan["current"]["kind"], "normalize");
    assert_eq!(root_plan["completion"]["workflow_complete"], false);
    assert!(root_plan["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage reconcile")));

    let atomized = successful_json(
        &run_aw(
            root.path(),
            &["wi", "atomize", "--project", "demo", "--json"],
        ),
        "aw wi atomize",
    );
    let epicized = successful_json(
        &run_aw(
            root.path(),
            &["wi", "epicize", "--project", "demo", "--json"],
        ),
        "aw wi epicize",
    );
    assert_eq!(atomized["current"]["kind"], "atomize");
    assert_eq!(atomized["plan"]["digest"], epicized["plan"]["digest"]);
    assert!(atomized["agent_prompt"]
        .as_str()
        .is_some_and(|prompt| prompt.contains("Independently review")));
    let payload_path = PathBuf::from(atomized["next"]["payload_path"].as_str().unwrap());
    let original: Value =
        serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    assert_eq!(original["kind"], "project_plan");
    assert_eq!(original["decision"], "pending");

    accept_review_payload(&payload_path, "reviewer-agent");
    let evidence = payload_path.to_string_lossy().to_string();
    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let reviewed = successful_json(&review, "aw wi plan-review");
    assert_eq!(reviewed["action"], "reviewed");
    assert_eq!(reviewed["next"]["kind"], "hitl");
    assert_eq!(reviewed["completion"]["requires_hitl"], true);
    assert!(reviewed["hitl_question"]["choices"][0]["resume_command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw wi plan-answer")));

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

#[test]
fn inventory_plan_rejects_same_agent_review() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), None);
    let output = run_aw(
        root.path(),
        &["wi", "atomize", "--project", "demo", "--json"],
    );
    let plan = successful_json(&output, "aw wi atomize");
    let payload_path = PathBuf::from(plan["next"]["payload_path"].as_str().unwrap());
    accept_review_payload(&payload_path, "author-agent");
    let evidence = payload_path.to_string_lossy().to_string();

    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    assert!(!review.status.success());
    assert!(
        String::from_utf8_lossy(&review.stderr).contains("not independent"),
        "stderr={}",
        String::from_utf8_lossy(&review.stderr)
    );

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

#[test]
fn inventory_plan_needs_revision_returns_to_its_producer() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), None);
    let output = run_aw(
        root.path(),
        &["wi", "atomize", "--project", "demo", "--json"],
    );
    let plan = successful_json(&output, "aw wi atomize");
    let payload_path = PathBuf::from(plan["next"]["payload_path"].as_str().unwrap());
    request_revision_payload(&payload_path, "reviewer-agent");
    let evidence = payload_path.to_string_lossy().to_string();

    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let reviewed = successful_json(&review, "aw wi plan-review");
    assert_eq!(reviewed["action"], "reviewed");
    assert_eq!(reviewed["status"], "continue");
    assert!(reviewed["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage atomize")));

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

#[test]
fn inventory_plan_rejects_next_command_not_bound_by_manifest() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), None);
    let output = run_aw(
        root.path(),
        &["wi", "epicize", "--project", "demo", "--json"],
    );
    let plan = successful_json(&output, "aw wi epicize");
    let payload_path = PathBuf::from(plan["next"]["payload_path"].as_str().unwrap());
    accept_review_payload(&payload_path, "reviewer-agent");
    let mut payload: Value =
        serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    payload["next_command"] = Value::String("aw wi close 42 --push".to_string());
    fs::write(
        &payload_path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
    let evidence = payload_path.to_string_lossy().to_string();

    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    assert!(!review.status.success());
    assert!(
        String::from_utf8_lossy(&review.stderr).contains("digest-bound manifest"),
        "stderr={}",
        String::from_utf8_lossy(&review.stderr)
    );

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

#[test]
fn explicit_human_only_inventory_policy_reaches_apply_and_verify() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), Some("human"));
    let output = run_aw(
        root.path(),
        &["wi", "atomize", "--project", "demo", "--json"],
    );
    let plan = successful_json(&output, "aw wi atomize");

    assert_eq!(plan["completion"]["requires_hitl"], true);
    assert_eq!(plan["next"]["kind"], "hitl");
    assert_eq!(
        plan["hitl_question"]["interaction"]["kind"],
        "user_question"
    );
    let approve_review_command = plan["hitl_question"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .find(|choice| choice["id"] == "approve")
        .and_then(|choice| choice["resume_command"].as_str())
        .unwrap();
    assert!(approve_review_command.contains("--human-choice approve"));
    let review_payload = PathBuf::from(plan["next"]["payload_path"].as_str().unwrap());
    let reviewed = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan-review",
                "--evidence-file",
                review_payload.to_str().unwrap(),
                "--human-choice",
                "approve",
                "--json",
            ],
        ),
        "human aw wi plan-review",
    );
    assert_eq!(reviewed["next"]["kind"], "hitl");
    let decision_payload = PathBuf::from(reviewed["next"]["payload_path"].as_str().unwrap());
    let question = reviewed["hitl_question"]["id"].as_str().unwrap();
    successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan-answer",
                "--payload",
                decision_payload.to_str().unwrap(),
                "--question",
                question,
                "--choice",
                "approve",
                "--json",
            ],
        ),
        "human-confirmed aw wi plan-answer",
    );
    let applied = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan-apply",
                "--evidence-file",
                decision_payload.to_str().unwrap(),
                "--json",
            ],
        ),
        "human-reviewed aw wi plan-apply",
    );
    assert!(applied["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage verify")));
    let root_id = plan["root"]["id"].as_str().unwrap();
    let verified = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan",
                "--project",
                "demo",
                "--stage",
                "verify",
                "--root",
                root_id,
                "--json",
            ],
        ),
        "verify human-only project-plan root",
    );
    assert_eq!(verified["completion"]["workflow_complete"], true);

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

// HANDWRITE-END

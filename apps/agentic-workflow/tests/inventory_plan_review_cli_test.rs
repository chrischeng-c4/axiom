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
    let mut authoritative_path = None;
    let mut authoritative_digest = None;
    let mut authoritative_bytes = None;
    let mut authoritative_payload = None;

    for verb in ["plan", "epicize", "atomize", "prioritize"] {
        let output = run_aw(root.path(), &["wi", verb, "--project", "demo", "--json"]);
        let plan = successful_json(&output, &format!("aw wi {verb}"));
        assert_eq!(plan["kind"], "project_plan", "{verb}: {plan:#}");
        assert_eq!(plan["invoked_as"], verb, "{verb}: {plan:#}");
        assert_eq!(plan["requires_hitl"], false, "{verb}: {plan:#}");
        assert_eq!(plan["hitl_status"], "pending_agent_review");
        assert_eq!(plan["review_backing"], "either");
        assert!(plan["agent_review_prompt"]
            .as_str()
            .is_some_and(|prompt| prompt.contains("Independently review")));
        assert!(plan["next"]
            .as_str()
            .is_some_and(|command| command.starts_with("aw wi plan-review")));

        let payload_path = PathBuf::from(plan["payload_path"].as_str().unwrap());
        let plan_path = PathBuf::from(plan["path"].as_str().unwrap());
        let plan_bytes = fs::read(&plan_path).unwrap();
        match (
            &authoritative_path,
            &authoritative_digest,
            &authoritative_bytes,
        ) {
            (Some(path), Some(digest), Some(bytes)) => {
                assert_eq!(path, &plan_path, "{verb} created a competing artifact");
                assert_eq!(digest, plan["plan_digest"].as_str().unwrap());
                assert_eq!(bytes, &plan_bytes, "{verb} changed canonical plan bytes");
            }
            _ => {
                authoritative_path = Some(plan_path);
                authoritative_digest = Some(plan["plan_digest"].as_str().unwrap().to_string());
                authoritative_bytes = Some(plan_bytes);
                authoritative_payload = Some(payload_path.clone());
            }
        }
        assert_eq!(
            authoritative_payload.as_ref().unwrap(),
            &payload_path,
            "{verb} created a competing review payload"
        );
        let original: Value =
            serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
        assert_eq!(original["kind"], "project_plan");
        assert_eq!(original["decision"], "pending");
        assert!(original["next_command"]
            .as_str()
            .is_some_and(|command| command.starts_with("aw ")));
    }

    let payload_path = authoritative_payload.unwrap();
    accept_review_payload(&payload_path, "reviewer-agent");
    let evidence = payload_path.to_string_lossy().to_string();
    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let reviewed = successful_json(&review, "aw wi plan-review");
    assert_eq!(reviewed["action"], "project_plan_review");
    assert_eq!(reviewed["status"], "accepted");
    assert_eq!(reviewed["requires_hitl"], false);
    assert_eq!(reviewed["transaction"]["status"], "complete");
    assert!(reviewed["next"]["command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw ")));

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
    let payload_path = PathBuf::from(plan["payload_path"].as_str().unwrap());
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
    let payload_path = PathBuf::from(plan["payload_path"].as_str().unwrap());
    request_revision_payload(&payload_path, "reviewer-agent");
    let evidence = payload_path.to_string_lossy().to_string();

    let review = run_aw(
        root.path(),
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let reviewed = successful_json(&review, "aw wi plan-review");
    assert_eq!(reviewed["status"], "needs_revision");
    assert_eq!(reviewed["published_issue_count"], 0);
    assert_eq!(reviewed["next"]["command"], "aw wi plan --project demo");

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
    let payload_path = PathBuf::from(plan["payload_path"].as_str().unwrap());
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
fn explicit_human_only_inventory_policy_remains_blocking() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path(), Some("human"));
    let output = run_aw(
        root.path(),
        &["wi", "prioritize", "--project", "demo", "--json"],
    );
    let plan = successful_json(&output, "aw wi prioritize");

    assert_eq!(plan["requires_hitl"], true);
    assert_eq!(plan["hitl_status"], "pending_human");
    assert_eq!(plan["review_backing"], "human");
    assert!(plan["agent_review_prompt"].is_null());

    let local = LocalBackend::from_project_root(root.path());
    let _ = fs::remove_dir_all(local.issues_dir());
}

// HANDWRITE-END

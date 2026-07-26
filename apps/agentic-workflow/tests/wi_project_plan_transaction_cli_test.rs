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

fn plan_atomize(root: &Path) -> Value {
    successful_json(
        &run_aw(
            root,
            &[
                "wi",
                "plan",
                "--project",
                "demo",
                "--stage",
                "atomize",
                "--json",
            ],
        ),
        "aw wi plan --stage atomize",
    )
}

fn review_and_human_approve(root: &Path, planned: &Value) -> PathBuf {
    let review_payload = PathBuf::from(planned["next"]["payload_path"].as_str().unwrap());
    accept_review(&review_payload);
    let reviewed = successful_json(
        &run_aw(
            root,
            &[
                "wi",
                "plan-review",
                "--evidence-file",
                review_payload.to_str().unwrap(),
                "--json",
            ],
        ),
        "aw wi plan-review",
    );
    assert_eq!(reviewed["next"]["kind"], "hitl");
    let decision_payload = PathBuf::from(reviewed["next"]["payload_path"].as_str().unwrap());
    let question = reviewed["hitl_question"]["id"].as_str().unwrap();
    let answered = successful_json(
        &run_aw(
            root,
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
        "aw wi plan-answer",
    );
    assert!(answered["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw wi plan-apply")));
    decision_payload
}

#[test]
fn accepted_project_plan_applies_once_and_reapply_is_clean_noop() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_epic(root.path());

    let planned = plan_atomize(root.path());
    let review_payload = PathBuf::from(planned["next"]["payload_path"].as_str().unwrap());
    let payload: Value =
        serde_json::from_str(&fs::read_to_string(&review_payload).unwrap()).unwrap();
    let manifest_path = PathBuf::from(payload["manifest_path"].as_str().unwrap());
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    assert_eq!(manifest["schema"], "aw.wi.project-plan-transaction.v2");
    assert_eq!(manifest["stage"], "atomize");
    assert_eq!(manifest["project"], "demo");
    assert_eq!(manifest["issue_snapshots"].as_array().unwrap().len(), 1);
    assert!(manifest["tracker_snapshot_digest"]
        .as_str()
        .is_some_and(|digest| digest.starts_with("sha256:")));
    assert!(manifest["mutations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|mutation| mutation["action"] == "create"));
    assert!(manifest["mutations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|mutation| mutation["action"] != "update"));

    let decision_payload = review_and_human_approve(root.path(), &planned);
    assert_eq!(open_issue_bodies(root.path()).len(), 1);
    let evidence = decision_payload.to_string_lossy().to_string();
    let applied = run_aw(
        root.path(),
        &["wi", "plan-apply", "--evidence-file", &evidence, "--json"],
    );
    let applied = successful_json(&applied, "aw wi plan-apply");
    assert_eq!(applied["action"], "applied");
    assert!(applied["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage verify")));

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
    assert!(bodies
        .iter()
        .any(|body| body.contains("epic:42") && body.contains("type: change")));

    let repeated = run_aw(
        root.path(),
        &["wi", "plan-apply", "--evidence-file", &evidence, "--json"],
    );
    let repeated = successful_json(&repeated, "reapply aw wi plan-apply");
    assert_eq!(repeated["action"], "applied");
    assert_eq!(open_issue_bodies(root.path()).len(), 2);

    let post_apply = plan_atomize(root.path());
    let post_plan = PathBuf::from(post_apply["plan"]["path"].as_str().unwrap());
    let post_manifest = post_plan.with_extension("manifest.json");
    let post_manifest: Value =
        serde_json::from_str(&fs::read_to_string(post_manifest).unwrap()).unwrap();
    assert!(
        post_manifest["mutations"].as_array().unwrap().is_empty(),
        "post-apply plan must converge to zero mutations:\n{}",
        serde_json::to_string_pretty(&post_manifest).unwrap()
    );

    let replay_after_canonical_overwrite = run_aw(
        root.path(),
        &["wi", "plan-apply", "--evidence-file", &evidence, "--json"],
    );
    successful_json(
        &replay_after_canonical_overwrite,
        "reapply accepted transaction after canonical plan overwrite",
    );
    assert_eq!(open_issue_bodies(root.path()).len(), 2);

    let unchanged = plan_atomize(root.path());
    assert_eq!(unchanged["plan"]["digest"], post_apply["plan"]["digest"]);
}

#[test]
fn mixed_horizon_publication_reparents_existing_changes_and_converges() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_mixed_horizon_epic(root.path(), true);

    let planned = plan_atomize(root.path());
    let plan_path = PathBuf::from(planned["plan"]["path"].as_str().unwrap());
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

    let decision_payload = review_and_human_approve(root.path(), &planned);
    let evidence = decision_payload.to_string_lossy().to_string();
    let applied = successful_json(
        &run_aw(
            root.path(),
            &["wi", "plan-apply", "--evidence-file", &evidence, "--json"],
        ),
        "apply mixed-horizon project plan",
    );
    assert_eq!(applied["action"], "applied");
    successful_json(
        &run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]),
        "post-apply mixed-horizon aw wi graph",
    );

    let post_apply = plan_atomize(root.path());
    let post_plan_path = PathBuf::from(post_apply["plan"]["path"].as_str().unwrap());
    let post_plan: Value =
        serde_json::from_str(&fs::read_to_string(&post_plan_path).unwrap()).unwrap();
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
    let manifest_path = post_plan_path.with_extension("manifest.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    assert!(
        manifest["mutations"].as_array().unwrap().is_empty(),
        "mixed-horizon post-publication plan must converge:\n{}",
        serde_json::to_string_pretty(&manifest).unwrap()
    );

    let unchanged = plan_atomize(root.path());
    assert_eq!(unchanged["plan"]["digest"], post_apply["plan"]["digest"]);
}

#[test]
fn tracker_drift_after_review_fails_before_any_mutation() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let epic_path = write_epic(root.path());
    let planned = plan_atomize(root.path());
    let decision_payload = review_and_human_approve(root.path(), &planned);

    let body = fs::read_to_string(&epic_path).unwrap();
    fs::write(
        &epic_path,
        body.replace(
            "title: Reviewed delivery",
            "title: Externally drifted delivery",
        ),
    )
    .unwrap();
    let evidence = decision_payload.to_string_lossy().to_string();
    let rejected = run_aw(
        root.path(),
        &["wi", "plan-apply", "--evidence-file", &evidence, "--json"],
    );
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("reviewed issue `42` changed"), "{stderr}");
    let bodies = open_issue_bodies(root.path());
    assert_eq!(bodies.len(), 1);
    assert!(!bodies[0].contains("aw:planning-transaction"));
}

#[test]
fn manifest_drift_after_human_answer_invalidates_digest_bound_evidence() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_epic(root.path());
    let planned = plan_atomize(root.path());
    let decision_payload = review_and_human_approve(root.path(), &planned);
    let decision: Value =
        serde_json::from_str(&fs::read_to_string(&decision_payload).unwrap()).unwrap();
    let manifest_path = PathBuf::from(decision["manifest_path"].as_str().unwrap());
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["apply_command"] = Value::String("aw wi show 42".to_string());
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();

    let rejected = run_aw(
        root.path(),
        &[
            "wi",
            "plan-apply",
            "--evidence-file",
            decision_payload.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("stale for its plan and root"), "{stderr}");
    assert_eq!(
        open_issue_bodies(root.path()).len(),
        1,
        "stale evidence must fail before creating a change"
    );
}

#[test]
fn forged_normalize_manifest_cannot_self_declare_deterministic_authority() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_epic(root.path());
    let planned = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "create canonical normalize plan",
    );
    let plan_path = PathBuf::from(planned["plan"]["path"].as_str().unwrap());
    let manifest_path = plan_path.with_extension("manifest.json");
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let snapshot = manifest["issue_snapshots"][0].clone();
    manifest["mutations"] = serde_json::json!([{
        "order": 0,
        "idempotency_key": "forged-normalize-update",
        "action": "update",
        "target": "42",
        "issue_type": "epic",
        "body": "forged body",
        "add_labels": [],
        "remove_labels": [],
        "reason": "forged",
        "stage": "normalize",
        "certainty": "deterministic",
        "evidence": ["issue:42"],
        "decision_source": "explicit_metadata",
        "requires_hitl": false
    }]);
    manifest["issue_snapshots"] = Value::Array(vec![snapshot]);
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();

    let rejected = run_aw(
        root.path(),
        &[
            "wi",
            "plan-apply",
            "--evidence-file",
            manifest_path.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        stderr.contains("does not match the canonical live-inventory projection"),
        "{stderr}"
    );
    let bodies = open_issue_bodies(root.path());
    assert_eq!(bodies.len(), 1);
    assert!(!bodies[0].contains("forged body"));
}

#[test]
fn one_project_plan_root_relays_until_terminal_verification() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_epic(root.path());

    let normalized = successful_json(
        &run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]),
        "start project-plan root",
    );
    let root_id = normalized["root"]["id"].as_str().unwrap();
    assert_eq!(normalized["current"]["kind"], "normalize");
    assert_eq!(normalized["completion"]["workflow_complete"], false);

    let reconciled = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan",
                "--project",
                "demo",
                "--stage",
                "reconcile",
                "--root",
                root_id,
                "--json",
            ],
        ),
        "reconcile project-plan root",
    );
    assert_eq!(reconciled["root"]["id"], root_id);
    assert!(reconciled["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage atomize")));

    let atomized = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "plan",
                "--project",
                "demo",
                "--stage",
                "atomize",
                "--root",
                root_id,
                "--json",
            ],
        ),
        "atomize project-plan root",
    );
    assert_eq!(atomized["root"]["id"], root_id);
    let decision_payload = review_and_human_approve(root.path(), &atomized);
    successful_json(
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
        "apply atomize project-plan root",
    );

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
        "verify project-plan root",
    );
    assert_eq!(verified["root"]["id"], root_id);
    assert_eq!(verified["status"], "done");
    assert_eq!(verified["next"]["kind"], "done");
    assert_eq!(verified["completion"]["workflow_complete"], true);
    assert!(verified["invoke"]["command"]
        .as_str()
        .is_some_and(str::is_empty));
}

// HANDWRITE-END

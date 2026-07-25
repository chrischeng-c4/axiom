// @spec apps/agentic-workflow/tech-design/core/logic/issues/two-stage-project-plan.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-two-stage-project-plan" tracker="#2387" reason="The compiled CLI fixture proves the canonical staged model, compatibility delegation, deterministic artifact, and graph fail-closed behavior."

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

fn write_issue(root: &Path, id: u64, issue_type: &str, title: &str, labels: &[&str], body: &str) {
    let backend = LocalBackend::from_project_root(root);
    let open = backend.issues_dir().join("open");
    fs::create_dir_all(&open).unwrap();
    let labels = labels
        .iter()
        .map(|label| format!("  - {label}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        open.join(format!("{id}.md")),
        format!(
            "---\ntype: {issue_type}\ntitle: {title}\nstate: open\ngithub_id: {id}\nlabels:\n{labels}\n---\n\n{body}\n"
        ),
    )
    .unwrap();
}

fn structured_body(requirement: &str) -> String {
    format!(
        r#"## Capability Alignment

Capability: demo
Capability Gap: planning
Progress Evidence: compiled fixture

## Scope

### In Scope
- {requirement}

### Out of Scope
- Unrelated behavior.

## Acceptance Criteria

- {requirement} is observable.

## Reference Context

- `aw.toml`
"#
    )
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

fn stdout_json(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "planning command did not emit one JSON value: {error}\nstdout={stdout}\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn read_plan(output: &Value) -> (PathBuf, Value, Vec<u8>) {
    let path = PathBuf::from(output["plan"]["path"].as_str().unwrap());
    let bytes = fs::read(&path).unwrap();
    let plan = serde_json::from_slice(&bytes).unwrap();
    (path, plan, bytes)
}

#[test]
fn compiled_cli_builds_one_deterministic_two_stage_project_plan() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        100,
        "epic",
        "Mixed delivery",
        &["type:epic", "app:demo", "priority:p1"],
        r#"## Requirements

- R1: Reconcile existing epic and change inventory.
- R2: Rewrite the entire platform.
- R3: Later phase: publish the transaction.
- R4: Add the missing audit boundary.
"#,
    );
    write_issue(
        root.path(),
        101,
        "change",
        "Reconcile existing epic and change inventory",
        &["type:change", "app:demo", "epic:100"],
        &structured_body("Reconcile existing epic and change inventory"),
    );
    write_issue(
        root.path(),
        102,
        "change",
        "Reconcile existing epic and change inventory",
        &["type:change", "app:demo", "epic:100"],
        &structured_body("Reconcile existing epic and change inventory"),
    );
    write_issue(
        root.path(),
        103,
        "change",
        "Rewrite the entire platform",
        &["type:change", "app:demo", "epic:100"],
        &structured_body("Rewrite the entire platform"),
    );
    write_issue(
        root.path(),
        104,
        "change",
        "Legacy unstructured leaf",
        &["type:change", "app:demo", "epic:100"],
        "Needs triage.",
    );
    write_issue(
        root.path(),
        200,
        "epic",
        "Ready delivery",
        &["type:epic", "app:demo", "priority:p0"],
        "## Requirements\n\n- R1: Deliver the ready boundary.\n",
    );
    write_issue(
        root.path(),
        250,
        "epic",
        "Later active delivery",
        &["type:epic", "app:demo", "priority:p2"],
        "## Requirements\n\n- R1: Deliver another active boundary.\n",
    );
    write_issue(
        root.path(),
        201,
        "change",
        "Deliver the ready boundary",
        &["type:change", "app:demo", "epic:200", "depends-on:202"],
        &structured_body("Deliver the ready boundary"),
    );
    write_issue(
        root.path(),
        202,
        "change",
        "Prepare the dependency",
        &["type:change", "app:demo", "epic:200"],
        &structured_body("Prepare the dependency"),
    );

    let normalized = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    assert!(normalized.status.success());
    let normalized = stdout_json(&normalized);
    assert_eq!(normalized["schema_version"], "aw.cli.v1");
    assert_eq!(normalized["root"]["kind"], "project_plan");
    assert_eq!(normalized["current"]["kind"], "normalize");
    assert_eq!(normalized["completion"]["workflow_complete"], false);
    assert!(normalized["invoke"]["command"].as_str().is_some());

    let output = run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = stdout_json(&output);
    let (path, plan, bytes) = read_plan(&output);
    assert_eq!(plan["schema"], "aw.wi.project-plan.v2");
    assert_eq!(plan["stage"], "atomize");
    assert_eq!(plan["action"], "done");
    assert_eq!(plan["valid"], true);
    assert_eq!(plan["stages"][0]["node_type"], "epic");
    assert_eq!(plan["stages"][1]["node_type"], "change");
    assert_eq!(plan["epic_order"][0], "200");
    assert!(
        plan["epic_order"]
            .as_array()
            .unwrap()
            .iter()
            .position(|id| id == "250")
            .unwrap()
            < plan["epic_order"]
                .as_array()
                .unwrap()
                .iter()
                .position(|id| id == "proposal:epic:100:deferred")
                .unwrap()
    );

    let split = plan["proposed_epics"].as_array().unwrap();
    assert_eq!(split.len(), 2, "{plan:#}");
    assert!(split.iter().all(|epic| epic["source_epic"] == "100"));
    assert!(split
        .iter()
        .any(|epic| epic["horizon"] == "deferred" && epic["priority"] == "p3"));

    let changes = plan["changes"].as_array().unwrap();
    let duplicate = changes.iter().find(|change| change["id"] == "102").unwrap();
    assert_eq!(duplicate["duplicate_of"], "101");
    assert_eq!(duplicate["lane"], "duplicate");
    let oversized = changes.iter().find(|change| change["id"] == "103").unwrap();
    assert_eq!(oversized["lane"], "needs_atomize");
    assert_eq!(oversized["replacement_ids"].as_array().unwrap().len(), 1);
    let blocked = changes.iter().find(|change| change["id"] == "201").unwrap();
    assert_eq!(blocked["lane"], "blocked_by_dependency");
    assert!(changes
        .iter()
        .all(|change| change["owner_epic"].is_string()));

    let proposals = plan["proposed_changes"].as_array().unwrap();
    let replacements = proposals
        .iter()
        .filter(|change| change["source_change"] == "103")
        .collect::<Vec<_>>();
    assert_eq!(replacements.len(), 1);
    assert!(replacements
        .iter()
        .all(|change| change["owner_epic"] == "proposal:epic:100:active"));
    assert!(replacements
        .iter()
        .all(|change| !change["covers"].as_array().unwrap().is_empty()));
    let mixed = plan["epics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|epic| epic["id"] == "100")
        .unwrap();
    assert!(mixed["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .any(|requirement| {
            requirement["text"] == "Rewrite the entire platform."
                && requirement["status"] == "planned"
        }));
    assert!(!proposals.iter().any(|change| {
        change["reason"] == "missing_requirement_coverage"
            && change["title"]
                .as_str()
                .is_some_and(|title| title.contains("Rewrite the entire platform"))
    }));
    assert!(proposals
        .iter()
        .all(|change| change["owner_epic"].is_string()));

    for verb in ["epicize", "atomize"] {
        let rerun = run_aw(root.path(), &["wi", verb, "--project", "demo", "--json"]);
        assert!(
            rerun.status.success(),
            "{verb}: {}",
            String::from_utf8_lossy(&rerun.stderr)
        );
        let rerun = stdout_json(&rerun);
        let (rerun_path, rerun_plan, rerun_bytes) = read_plan(&rerun);
        assert_eq!(rerun_path, path);
        assert_eq!(rerun_plan["digest"], plan["digest"]);
        assert_eq!(rerun_bytes, bytes);
    }
}

#[test]
fn project_plan_routes_unowned_change_to_hitl_without_bootstrap_epic() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        300,
        "epic",
        "Owned root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Own changes.\n",
    );
    write_issue(
        root.path(),
        301,
        "change",
        "Unowned change",
        &["type:change", "app:demo"],
        &structured_body("Remain unowned"),
    );

    let output = run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "reconcile",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = stdout_json(&output);
    assert_eq!(output["status"], "blocked");
    assert_eq!(output["next"]["kind"], "hitl");
    assert_eq!(
        output["hitl_question"]["interaction"]["kind"],
        "user_question"
    );
    assert!(output["hitl_question"]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .any(|choice| choice["id"] == "300"));
    let (_, plan, _) = read_plan(&output);
    assert_eq!(plan["action"], "done");
    assert_eq!(plan["valid"], true);
    assert!(plan["proposed_epics"].as_array().unwrap().is_empty());
    assert!(!plan.to_string().contains("AW unclassified active backlog"));

    let payload = output["next"]["payload_path"].as_str().unwrap();
    let question = output["hitl_question"]["id"].as_str().unwrap();
    let answered = run_aw(
        root.path(),
        &[
            "wi",
            "plan-answer",
            "--payload",
            payload,
            "--question",
            question,
            "--choice",
            "300",
            "--json",
        ],
    );
    assert!(answered.status.success());
    let applied = run_aw(
        root.path(),
        &["wi", "plan-apply", "--evidence-file", payload, "--json"],
    );
    assert!(applied.status.success());
    let graph = run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]);
    assert!(graph.status.success());
}

#[test]
fn reconcile_does_not_reask_owner_for_explicit_child_of_mixed_horizon_epic() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        300,
        "epic",
        "Mixed delivery",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Deliver the active boundary.\n- R2: Later phase: publish the deferred boundary.\n",
    );
    write_issue(
        root.path(),
        301,
        "change",
        "Publish the deferred boundary",
        &["type:change", "app:demo", "priority:p3", "epic:300"],
        &structured_body("Later phase: publish the deferred boundary"),
    );

    let output = run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "reconcile",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = stdout_json(&output);
    assert_eq!(output["status"], "continue");
    assert_eq!(output["requires_hitl"], false);
    assert_eq!(output["next"]["kind"], "run_command");
    assert!(output["invoke"]["command"]
        .as_str()
        .unwrap()
        .contains("--stage atomize"));
    let (_, plan, _) = read_plan(&output);
    let change = plan["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == "301")
        .unwrap();
    assert_eq!(change["owner_epic"], "proposal:epic:300:deferred");
}

#[test]
fn project_plan_still_fails_closed_on_multiple_owners() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    for (id, title) in [(300, "First root"), (302, "Second root")] {
        write_issue(
            root.path(),
            id,
            "epic",
            title,
            &["type:epic", "app:demo", "priority:p1"],
            "## Requirements\n\n- R1: Own changes.\n",
        );
    }
    write_issue(
        root.path(),
        301,
        "change",
        "Multiply owned change",
        &["type:change", "app:demo", "epic:300", "parent:302"],
        &structured_body("Remain invalid"),
    );

    let output = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    assert!(output.status.success());
    let output = stdout_json(&output);
    assert_eq!(output["action"], "blocked");
    assert!(output["plan"]["path"].is_string());
    let (_, plan, _) = read_plan(&output);
    assert!(plan["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["code"] == "multiple_epic_owners"));
    assert_eq!(plan["action"], "blocked");
    assert_eq!(plan["valid"], false);
}

#[test]
fn closed_change_counts_as_requirement_coverage() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        10,
        "epic",
        "Delivered root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Deliver the stable boundary.\n",
    );
    let backend = LocalBackend::from_project_root(root.path());
    let closed = backend.issues_dir().join("closed");
    fs::create_dir_all(&closed).unwrap();
    fs::write(
        closed.join("11.md"),
        r#"---
type: change
title: Deliver the stable boundary
state: closed
github_id: 11
labels:
  - type:change
  - app:demo
  - priority:p1
  - epic:10
---

## Scope

- Deliver the stable boundary.
"#,
    )
    .unwrap();

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (_, plan, _) = read_plan(&output);
    assert!(plan["proposed_changes"].as_array().unwrap().is_empty());
    assert_eq!(plan["epics"][0]["requirements"][0]["status"], "covered");
    assert_eq!(plan["epics"][0]["requirements"][0]["covered_by"][0], "11");
}

#[test]
fn completed_explicit_child_plan_covers_the_aggregate_epic_contract() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        20,
        "epic",
        "Delivered root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Establish the stable protocol.\n- R2: Prove the terminal workflow.\n\n## Child Work Items\n\n| WI | Bounded outcome |\n|---|---|\n| #21 | Protocol implementation |\n| [#22](https://example.test/issues/22) | Workflow evidence |\n",
    );
    let backend = LocalBackend::from_project_root(root.path());
    let closed = backend.issues_dir().join("closed");
    fs::create_dir_all(&closed).unwrap();
    for (id, title) in [(21, "Implement protocol"), (22, "Record evidence")] {
        fs::write(
            closed.join(format!("{id}.md")),
            format!(
                "---\ntype: change\ntitle: {title}\nstate: closed\ngithub_id: {id}\nlabels:\n  - type:change\n  - app:demo\n  - epic:20\n---\n\n## Scope\n\n- {title}.\n"
            ),
        )
        .unwrap();
    }

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (_, plan, _) = read_plan(&output);
    assert!(plan["proposed_changes"].as_array().unwrap().is_empty());
    for requirement in plan["epics"][0]["requirements"].as_array().unwrap() {
        assert_eq!(requirement["status"], "covered");
        assert_eq!(requirement["covered_by"], serde_json::json!(["21", "22"]));
    }
}

#[test]
fn partial_child_plan_covers_only_explicitly_mapped_parent_requirements() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        25,
        "epic",
        "Partially delivered root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Establish the stable protocol.\n- R2: Prove the terminal workflow.\n\n## Child Work Items\n\n| WI | Covers | Bounded outcome |\n|---|---|---|\n| #26 | R1 | Protocol implementation |\n| #27 | R2 | Workflow evidence |\n",
    );
    let backend = LocalBackend::from_project_root(root.path());
    let closed = backend.issues_dir().join("closed");
    fs::create_dir_all(&closed).unwrap();
    fs::write(
        closed.join("26.md"),
        "---\ntype: change\ntitle: Implement protocol\nstate: closed\ngithub_id: 26\nlabels:\n  - type:change\n  - app:demo\n  - epic:25\n---\n\n## Scope\n\n- Implement protocol.\n",
    )
    .unwrap();
    write_issue(
        root.path(),
        27,
        "change",
        "Record workflow evidence",
        &["type:change", "app:demo", "epic:25"],
        "Not yet structured.",
    );

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (_, plan, _) = read_plan(&output);
    let requirements = plan["epics"][0]["requirements"].as_array().unwrap();
    assert_eq!(requirements[0]["status"], "covered");
    assert_eq!(requirements[0]["covered_by"], serde_json::json!(["26"]));
    assert_eq!(requirements[1]["status"], "gap");
    assert_eq!(
        plan["proposed_changes"][0]["covers"],
        serde_json::json!(["25:requirement-2"])
    );
}

#[test]
fn multiline_requirement_remains_complete_in_plan_and_transaction_scope() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        28,
        "epic",
        "Coordination protocol",
        &["type:epic", "app:demo", "priority:p1"],
        "## Capability Alignment\n\nCapability: coordination-control-plane\n\n## Requirements\n\n- R1: Define versioned task, dispatch, message, and gate\n  schemas with deterministic compatibility.\n\n## Verification Inventory\n\n| Requirement | Gate | Oracle |\n|---|---|---|\n| R1 | `cargo test -p agentic-workflow --test coordination_contract_cli_test` | The versioned schema round-trips and rejects an unknown version. |\n",
    );

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (plan_path, plan, _) = read_plan(&output);
    let requirement =
        "Define versioned task, dispatch, message, and gate schemas with deterministic compatibility.";
    assert_eq!(plan["epics"][0]["requirements"][0]["text"], requirement);
    let manifest: Value =
        serde_json::from_slice(&fs::read(plan_path.with_extension("manifest.json")).unwrap())
            .unwrap();
    let mutation = &manifest["mutations"][0];
    assert!(mutation["body"].as_str().unwrap().contains(requirement));
    assert!(mutation["body"]
        .as_str()
        .unwrap()
        .contains("Capability: coordination-control-plane"));
    assert!(mutation["body"]
        .as_str()
        .unwrap()
        .contains("cargo test -p agentic-workflow --test coordination_contract_cli_test"));
    assert!(mutation["body"]
        .as_str()
        .unwrap()
        .contains("The versioned schema round-trips and rejects an unknown version."));
}

#[test]
fn normative_deferred_policy_does_not_split_an_active_epic() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        30,
        "epic",
        "Planning policy",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Split an epic when active and deferred outcomes have materially different scheduling horizons.\n",
    );

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (_, plan, _) = read_plan(&output);
    assert_eq!(plan["epics"][0]["horizon"], "active");
    assert!(plan["proposed_epics"].as_array().unwrap().is_empty());
}

#[test]
fn ordinary_draft_does_not_count_as_delivered_requirement_coverage() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        12,
        "epic",
        "Undelivered root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: Deliver the pending boundary.\n",
    );
    let backend = LocalBackend::from_project_root(root.path());
    let open = backend.issues_dir().join("open");
    fs::write(
        open.join("13.md"),
        format!(
            r#"---
type: change
title: Deliver the pending boundary
state: draft
github_id: 13
labels:
  - type:change
  - app:demo
  - priority:p1
  - epic:12
---

{}
"#,
            structured_body("Deliver the pending boundary")
        ),
    )
    .unwrap();

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    let (_, plan, _) = read_plan(&output);
    let requirement = &plan["epics"][0]["requirements"][0];
    assert_eq!(requirement["status"], "gap");
    assert!(requirement["covered_by"]
        .as_array()
        .unwrap()
        .iter()
        .all(|reference| reference != "13"));
    assert!(plan["proposed_changes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|proposal| {
            proposal["reason"] == "missing_requirement_coverage"
                && !proposal["covers"].as_array().unwrap().is_empty()
        }));
}

#[test]
fn scope_and_acceptance_criteria_do_not_invent_atomization_requirements() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        20,
        "epic",
        "Legacy root",
        &["type:epic", "app:demo", "priority:p1"],
        "## Scope\n\n- This used to become a change.\n\n## Acceptance Criteria\n\n- This also used to become a change.\n",
    );

    let output = stdout_json(&run_aw(
        root.path(),
        &[
            "wi",
            "plan",
            "--project",
            "demo",
            "--stage",
            "atomize",
            "--json",
        ],
    ));
    assert_eq!(output["status"], "blocked");
    let (_, plan, _) = read_plan(&output);
    assert!(plan["proposed_changes"].as_array().unwrap().is_empty());
    assert!(plan["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["code"] == "missing_authoritative_requirements"));
}

// HANDWRITE-END

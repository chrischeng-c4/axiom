// @spec apps/agentic-workflow/tech-design/core/logic/issues/two-stage-project-plan.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-two-stage-project-plan" tracker="#2387" reason="The compiled CLI fixture proves the canonical two-stage model, compatibility delegation, deterministic artifact, and graph fail-closed behavior."

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
    let path = PathBuf::from(output["path"].as_str().unwrap());
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

    let output = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = stdout_json(&output);
    let (path, plan, bytes) = read_plan(&output);
    assert_eq!(plan["schema"], "aw.wi.project-plan.v1");
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
    assert!(
        split
            .iter()
            .any(|epic| epic["horizon"] == "deferred" && epic["priority"] == "p3")
    );

    let changes = plan["changes"].as_array().unwrap();
    let duplicate = changes.iter().find(|change| change["id"] == "102").unwrap();
    assert_eq!(duplicate["duplicate_of"], "101");
    assert_eq!(duplicate["lane"], "duplicate");
    let oversized = changes.iter().find(|change| change["id"] == "103").unwrap();
    assert_eq!(oversized["lane"], "needs_atomize");
    assert_eq!(oversized["replacement_ids"].as_array().unwrap().len(), 2);
    let blocked = changes.iter().find(|change| change["id"] == "201").unwrap();
    assert_eq!(blocked["lane"], "blocked_by_dependency");
    assert!(
        changes
            .iter()
            .all(|change| change["owner_epic"].is_string())
    );

    let proposals = plan["proposed_changes"].as_array().unwrap();
    let replacements = proposals
        .iter()
        .filter(|change| change["source_change"] == "103")
        .collect::<Vec<_>>();
    assert_eq!(replacements.len(), 2);
    assert!(
        replacements
            .iter()
            .all(|change| change["owner_epic"] == "proposal:epic:100:active")
    );
    let mixed = plan["epics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|epic| epic["id"] == "100")
        .unwrap();
    assert!(
        mixed["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .any(|requirement| {
                requirement["text"] == "Rewrite the entire platform."
                    && requirement["status"] == "planned"
            })
    );
    assert!(!proposals.iter().any(|change| {
        change["reason"] == "missing_requirement_coverage"
            && change["title"]
                .as_str()
                .is_some_and(|title| title.contains("Rewrite the entire platform"))
    }));
    assert!(
        proposals
            .iter()
            .all(|change| change["owner_epic"].is_string())
    );

    for verb in ["epicize", "atomize", "prioritize"] {
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
fn project_plan_bootstraps_unowned_change_into_reviewable_epic() {
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

    let output = run_aw(root.path(), &["wi", "plan", "--project", "demo", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = stdout_json(&output);
    assert_eq!(output["action"], "planned");
    let (_, plan, _) = read_plan(&output);
    assert_eq!(plan["action"], "done");
    assert_eq!(plan["valid"], true);
    let proposal = plan["proposed_epics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|epic| epic["reason"] == "unowned_change_bootstrap")
        .unwrap();
    assert_eq!(proposal["source_epic"], "bootstrap:demo");
    assert_eq!(proposal["horizon"], "active");
    assert_eq!(proposal["priority"], "p2");
    assert_eq!(
        plan["changes"][0]["owner_epic"], proposal["id"],
        "the reviewed plan must assign exactly one proposed owner"
    );
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
    assert!(!output.status.success());
    let output = stdout_json(&output);
    assert_eq!(output["action"], "blocked");
    assert!(
        output["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["code"] == "multiple_epic_owners")
    );
    let (_, plan, _) = read_plan(&output);
    assert_eq!(plan["action"], "blocked");
    assert_eq!(plan["valid"], false);
}

// HANDWRITE-END

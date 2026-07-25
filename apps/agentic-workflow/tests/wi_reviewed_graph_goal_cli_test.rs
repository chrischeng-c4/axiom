// @spec apps/agentic-workflow/tech-design/core/logic/issues/reviewed-graph-goal-selection.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-reviewed-graph-goal-selection" tracker="#2389" reason="The compiled fixture proves epic and backlog roots consume one published graph, share priority/readiness selection, fail closed on graph drift, and reuse terminal epic rollup."

use agentic_workflow::issues::{IssueBackend, LocalBackend};
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

fn structured_body(scope: &str) -> String {
    format!(
        r#"## Capability Alignment

Capability: workflow-root-runner
Capability Gap: reviewed-graph-selection
Progress Evidence: compiled fixture

## Scope

### In Scope
- {scope}

### Out of Scope
- Unrelated graph behavior.

## Acceptance Criteria

- {scope} is observable.

## Reference Context

- Issue #2389
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

fn json(output: &Output, command: &str) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{command} failed:\nstdout={stdout}\nstderr={stderr}"
    );
    if let Ok(value) = serde_json::from_str(stdout.trim()) {
        return value;
    }
    stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap_or_else(|| panic!("{command} did not emit JSON\nstdout={stdout}\nstderr={stderr}"))
}

fn publish_reviewed_graph(root: &Path) -> Value {
    let plan = run_aw(
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
    );
    let plan = json(&plan, "aw wi plan");
    if plan["invoke"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("--stage verify"))
    {
        let root_id = plan["root"]["id"].as_str().unwrap();
        let verified = run_aw(
            root,
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
        );
        let verified = json(&verified, "aw wi plan --stage verify");
        assert_eq!(verified["completion"]["workflow_complete"], true);
        return verified;
    }
    let payload_path = PathBuf::from(plan["next"]["payload_path"].as_str().unwrap());
    let mut payload: Value =
        serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    payload["decision"] = Value::String("accepted".to_string());
    payload["reviewed_by"] = Value::String("independent-reviewer".to_string());
    payload["summary"] = Value::String(
        "Reviewed the complete epic/change snapshot, priority order, and mutation manifest."
            .to_string(),
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
        &payload_path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
    let evidence = payload_path.to_string_lossy().to_string();
    let reviewed = run_aw(
        root,
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    let reviewed = json(&reviewed, "aw wi plan-review");
    assert_eq!(reviewed["action"], "reviewed");
    assert_eq!(reviewed["status"], "blocked");
    assert_eq!(reviewed["requires_hitl"], true);
    let decision_path = reviewed["next"]["payload_path"].as_str().unwrap();
    let question_id = reviewed["hitl_question"]["id"].as_str().unwrap();
    let answered = run_aw(
        root,
        &[
            "wi",
            "plan-answer",
            "--payload",
            decision_path,
            "--question",
            question_id,
            "--choice",
            "approve",
            "--json",
        ],
    );
    let answered = json(&answered, "aw wi plan-answer");
    let approved_path = answered["next"]["payload_path"].as_str().unwrap();
    let applied = run_aw(
        root,
        &[
            "wi",
            "plan-apply",
            "--evidence-file",
            approved_path,
            "--json",
        ],
    );
    let applied = json(&applied, "aw wi plan-apply");
    assert_eq!(applied["action"], "applied");
    let root_id = applied["root"]["id"].as_str().unwrap();
    let verified = run_aw(
        root,
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
    );
    let verified = json(&verified, "aw wi plan --stage verify");
    assert_eq!(verified["completion"]["workflow_complete"], true);
    verified
}

fn write_priority_fixture(root: &Path) {
    write_project(root);
    write_issue(
        root,
        100,
        "epic",
        "Lower project direction",
        &["type:epic", "app:demo", "priority:p1"],
        "## Requirements\n\n- R1: deliver the lower-epic urgent leaf.\n",
    );
    write_issue(
        root,
        101,
        "change",
        "Urgent leaf in lower epic",
        &["type:change", "app:demo", "epic:100", "priority:p0"],
        &structured_body("deliver the lower-epic urgent leaf"),
    );
    write_issue(
        root,
        200,
        "epic",
        "Highest project direction",
        &["type:epic", "app:demo", "priority:p0"],
        "## Requirements\n\n- R1: deliver the blocked first leaf.\n- R2: deliver the ready dependency.\n- R3: deliver the later sibling.\n",
    );
    write_issue(
        root,
        201,
        "change",
        "Blocked first leaf",
        &[
            "type:change",
            "app:demo",
            "epic:200",
            "priority:p0",
            "depends-on:202",
        ],
        &structured_body("deliver the blocked first leaf"),
    );
    write_issue(
        root,
        202,
        "change",
        "Ready dependency",
        &["type:change", "app:demo", "epic:200", "priority:p1"],
        &structured_body("deliver the ready dependency"),
    );
    write_issue(
        root,
        203,
        "change",
        "Later sibling",
        &["type:change", "app:demo", "epic:200", "priority:p2"],
        &structured_body("deliver the later sibling"),
    );
}

#[test]
fn epic_and_backlog_roots_choose_the_same_ready_leaf_from_epic_first_priority() {
    let root = tempfile::tempdir().unwrap();
    write_priority_fixture(root.path());
    publish_reviewed_graph(root.path());

    let backlog = json(
        &run_aw(root.path(), &["goal", "backlog", "--project", "demo"]),
        "aw goal backlog",
    );
    let epic = json(
        &run_aw(root.path(), &["goal", "wi", "200"]),
        "aw goal wi 200",
    );
    assert_eq!(backlog["next"]["command"], "aw goal wi 202");
    assert_eq!(epic["next"]["command"], "aw goal wi 202");
    assert!(backlog["completion"]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|line| line.as_str().unwrap().contains("201")
            && line.as_str().unwrap().contains("blocked")));
    assert!(!backlog["next"]["command"].as_str().unwrap().contains("101"));
}

#[tokio::test]
async fn closed_children_roll_up_epic_and_graph_label_drift_fails_closed() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        10,
        "epic",
        "Terminal epic",
        &["type:epic", "app:demo", "priority:p0"],
        "## Requirements\n\n- R1: finish the child.\n",
    );
    write_issue(
        root.path(),
        11,
        "change",
        "Terminal child",
        &["type:change", "app:demo", "epic:10", "priority:p0"],
        &structured_body("finish the child"),
    );
    publish_reviewed_graph(root.path());
    let backend = LocalBackend::from_project_root(root.path());
    backend.close("11", None).await.unwrap();

    let backlog_terminal = json(
        &run_aw(root.path(), &["goal", "backlog", "--project", "demo"]),
        "aw goal backlog",
    );
    assert_eq!(backlog_terminal["next"]["command"], "aw goal wi 10");

    let terminal = json(&run_aw(root.path(), &["goal", "wi", "10"]), "aw goal wi 10");
    assert_eq!(terminal["next"]["command"], "aw wi close 10 --push");
    assert!(!terminal["next"]["command"]
        .as_str()
        .unwrap()
        .contains("atomize"));

    let closed_path = backend.issues_dir().join("closed").join("11.md");
    let body = fs::read_to_string(&closed_path).unwrap();
    fs::write(&closed_path, body.replace("priority:p0", "priority:p2")).unwrap();
    let stale = json(
        &run_aw(root.path(), &["goal", "backlog", "--project", "demo"]),
        "stale aw goal backlog",
    );
    assert_eq!(stale["action"], "blocked");
    assert_eq!(stale["next"]["command"], "aw wi plan --project demo --json");
    assert!(stale["next"]["reason"]
        .as_str()
        .unwrap()
        .contains("issue `11` graph labels changed"));
}

#[test]
fn invalid_ownership_after_publication_uses_issue_specific_remediation() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        10,
        "epic",
        "Owned epic",
        &["type:epic", "app:demo", "priority:p0"],
        "## Requirements\n\n- Keep one owner.\n",
    );
    write_issue(
        root.path(),
        11,
        "change",
        "Owned child",
        &["type:change", "app:demo", "epic:10"],
        &structured_body("keep one owner"),
    );
    publish_reviewed_graph(root.path());

    let backend = LocalBackend::from_project_root(root.path());
    let child_path = backend.issues_dir().join("open").join("11.md");
    let body = fs::read_to_string(&child_path).unwrap();
    assert!(body.contains("epic:10"), "{body}");
    fs::write(&child_path, body.replace("epic:10", "unowned:10")).unwrap();
    let invalid = json(
        &run_aw(root.path(), &["goal", "backlog", "--project", "demo"]),
        "invalid aw goal backlog",
    );
    assert_eq!(invalid["action"], "blocked");
    assert_eq!(invalid["next"]["command"], "aw wi show 11");
    assert!(invalid["next"]["reason"]
        .as_str()
        .unwrap()
        .contains("does not resolve to exactly one owning epic"));
}

// HANDWRITE-END

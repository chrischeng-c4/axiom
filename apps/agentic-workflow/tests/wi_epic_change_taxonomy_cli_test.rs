// @spec apps/agentic-workflow/tech-design/surface/specs/aw-capability-alignment-wi-planning.md#cli
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-epic-change-taxonomy" tracker="#2385" reason="The compiled CLI fixture proves canonical authoring, compatibility filtering, deterministic output, and history-preserving local reads."

use agentic_workflow::issues::{IssueBackend, LocalBackend};
use serde_json::Value;
use std::fs;
use std::path::Path;
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

fn write_issue(root: &Path, id: u64, issue_type: &str) {
    let backend = LocalBackend::from_project_root(root);
    let open = backend.issues_dir().join("open");
    fs::create_dir_all(&open).unwrap();
    fs::write(
        open.join(format!("{id}.md")),
        format!(
            "---\ntype: {issue_type}\ntitle: {issue_type} fixture\nstate: open\ngithub_id: {id}\nlabels:\n  - type:{issue_type}\n  - app:demo\n---\n\nbody for {issue_type}\n"
        ),
    )
    .unwrap();
}

fn run_aw(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
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

#[tokio::test]
async fn compiled_cli_exposes_epic_change_and_preserves_legacy_local_history() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());

    let create_help = run_aw(root.path(), &["wi", "create", "--help"]);
    let create_help = String::from_utf8_lossy(&create_help.stdout);
    assert!(create_help.contains("[possible values: epic, change]"));
    assert!(create_help.contains("Closed enum: epic | change"));

    let list_help = run_aw(root.path(), &["wi", "list", "--help"]);
    let list_help = String::from_utf8_lossy(&list_help.stdout);
    assert!(list_help.contains("[possible values: epic, change]"));
    assert!(list_help.contains("matches legacy non-epic labels"));

    let rejected = run_aw(
        root.path(),
        &[
            "wi",
            "create",
            "--title",
            "Legacy authoring",
            "--type",
            "bug",
            "--project",
            "demo",
        ],
    );
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("invalid value 'bug'"));

    for (offset, legacy) in ["bug", "enhancement", "refactor", "test"]
        .into_iter()
        .enumerate()
    {
        write_issue(root.path(), 100 + offset as u64, legacy);
    }
    write_issue(root.path(), 200, "epic");

    let changes = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "list",
                "--project",
                "demo",
                "--type",
                "change",
                "--json",
            ],
        ),
        "aw wi list --type change",
    );
    let changes = changes.as_array().unwrap();
    assert_eq!(changes.len(), 4);
    assert!(changes.iter().all(|issue| issue["type"] == "change"));
    assert_eq!(
        changes
            .iter()
            .filter_map(|issue| issue["github_id"].as_u64())
            .collect::<Vec<_>>(),
        vec![100, 101, 102, 103]
    );

    let epics = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "list",
                "--project",
                "demo",
                "--type",
                "epic",
                "--json",
            ],
        ),
        "aw wi list --type epic",
    );
    let epics = epics.as_array().unwrap();
    assert_eq!(epics.len(), 1);
    assert_eq!(epics[0]["type"], "epic");
    assert_eq!(epics[0]["github_id"], 200);

    let backend = LocalBackend::from_project_root(root.path());
    for (offset, legacy) in ["bug", "enhancement", "refactor", "test"]
        .into_iter()
        .enumerate()
    {
        let id = 100 + offset as u64;
        let issue = backend.get(&id.to_string()).await.unwrap().unwrap();
        assert_eq!(issue.issue_type.as_str(), "change");
        assert_eq!(issue.body.trim(), format!("body for {legacy}"));
        assert_eq!(issue.labels[0], format!("type:{legacy}"));
        let raw =
            fs::read_to_string(backend.issues_dir().join("open").join(format!("{id}.md"))).unwrap();
        assert!(raw.contains(&format!("type: {legacy}")));
        assert!(raw.contains(&format!("type:{legacy}")));
    }

    let created = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "create",
                "--title",
                "Canonical change fixture",
                "--type",
                "change",
                "--project",
                "demo",
                "--priority",
                "p1",
            ],
        ),
        "aw wi create --type change",
    );
    assert_eq!(created["action"], "dispatch");
    let slug = created["slug"].as_str().unwrap();
    let created_issue = backend.get(slug).await.unwrap().unwrap();
    assert_eq!(created_issue.issue_type.as_str(), "change");
    assert!(created_issue.labels.contains(&"type:change".to_string()));
    assert!(!created_issue.labels.iter().any(|label| matches!(
        label.as_str(),
        "type:bug" | "type:enhancement" | "type:refactor" | "type:test"
    )));
    let created_raw = fs::read_to_string(backend.issue_path(&created_issue)).unwrap();
    assert!(created_raw.contains("type: change"));
    assert!(created_raw.contains("type:change"));

    let updated = run_aw(
        root.path(),
        &[
            "wi",
            "update",
            slug,
            "--add-label",
            "type:refactor",
            "--json",
        ],
    );
    assert!(
        updated.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&updated.stdout),
        String::from_utf8_lossy(&updated.stderr)
    );
    let updated_issue = backend.get(slug).await.unwrap().unwrap();
    assert!(updated_issue.labels.contains(&"type:change".to_string()));
    assert!(!updated_issue.labels.contains(&"type:refactor".to_string()));

    let _ = fs::remove_dir_all(backend.issues_dir());
}

// HANDWRITE-END

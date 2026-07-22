// @spec apps/agentic-workflow/tech-design/core/logic/issues/epic-change-graph.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-epic-change-graph" tracker="#2386" reason="The compiled CLI fixture proves deterministic read-only projection and fail-closed tracker graph diagnostics."

use agentic_workflow::issues::LocalBackend;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
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

fn write_issue(root: &Path, id: u64, issue_type: &str, state: &str, labels: &[&str], body: &str) {
    let backend = LocalBackend::from_project_root(root);
    let directory = backend.issues_dir().join(match state {
        "closed" => "closed",
        _ => "open",
    });
    fs::create_dir_all(&directory).unwrap();
    let labels = labels
        .iter()
        .map(|label| format!("  - {label}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        directory.join(format!("{id}.md")),
        format!(
            "---\ntype: {issue_type}\ntitle: fixture {id}\nstate: {state}\ngithub_id: {id}\nlabels:\n{labels}\n---\n\n{body}\n"
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

fn stdout_json(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "graph command did not emit one JSON value: {error}\nstdout={stdout}\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn issue_snapshot(root: &Path) -> BTreeMap<String, String> {
    let backend = LocalBackend::from_project_root(root);
    let issues_dir = backend.issues_dir();
    let mut snapshot = BTreeMap::new();
    for state in ["open", "closed"] {
        let directory = issues_dir.join(state);
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                snapshot.insert(
                    format!("{state}/{}", entry.file_name().to_string_lossy()),
                    fs::read_to_string(entry.path()).unwrap(),
                );
            }
        }
    }
    snapshot
}

#[test]
fn compiled_cli_projects_stable_graph_and_fails_closed_without_tracker_writes() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    write_issue(
        root.path(),
        100,
        "epic",
        "open",
        &["type:epic", "app:demo", "priority:p1"],
        "",
    );
    write_issue(
        root.path(),
        101,
        "change",
        "open",
        &[
            "type:change",
            "app:demo",
            "epic:100",
            "priority:p0",
            "depends-on:103",
        ],
        "",
    );
    write_issue(
        root.path(),
        102,
        "change",
        "open",
        &["type:change", "app:demo", "supersedes:103"],
        "- **Parent Epic:** `#100`",
    );
    write_issue(
        root.path(),
        103,
        "change",
        "closed",
        &["type:change", "app:demo", "epic:100", "superseded-by:102"],
        "",
    );

    let help = run_aw(root.path(), &["wi", "graph", "--help"]);
    assert!(help.status.success());
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--project <PROJECT>"));
    assert!(help.contains("--json"));

    let before = issue_snapshot(root.path());
    let first = run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]);
    assert!(
        first.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );
    let first = stdout_json(&first);
    let second = run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]);
    assert!(second.status.success());
    let second = stdout_json(&second);
    assert_eq!(first, second);
    assert_eq!(first["schema"], "aw.wi.graph.v1");
    assert_eq!(first["action"], "done");
    assert_eq!(first["valid"], true);
    assert!(first.get("next").is_none());
    assert_eq!(
        first["epics"][0]["children"],
        serde_json::json!(["101", "102", "103"])
    );
    assert_eq!(first["changes"][0]["priority"]["source"], "explicit");
    assert_eq!(first["changes"][1]["priority"]["source"], "inherited");
    assert_eq!(
        first["changes"][1]["supersedes"],
        serde_json::json!(["103"])
    );
    assert_eq!(
        first["changes"][2]["superseded_by"],
        serde_json::json!(["102"])
    );
    assert_eq!(before, issue_snapshot(root.path()));

    let invalid_root = tempfile::tempdir().unwrap();
    write_project(invalid_root.path());
    for (id, project, priority) in [
        (200, "app:demo", "p0"),
        (206, "app:demo", "p1"),
        (300, "app:other", "p0"),
    ] {
        write_issue(
            invalid_root.path(),
            id,
            "epic",
            "open",
            &["type:epic", project, &format!("priority:{priority}")],
            "",
        );
    }
    write_issue(
        invalid_root.path(),
        201,
        "change",
        "open",
        &["type:change", "app:demo"],
        "",
    );
    write_issue(
        invalid_root.path(),
        202,
        "change",
        "open",
        &["type:change", "app:demo", "epic:404"],
        "",
    );
    write_issue(
        invalid_root.path(),
        203,
        "change",
        "open",
        &["type:change", "app:demo", "epic:300"],
        "",
    );
    write_issue(
        invalid_root.path(),
        204,
        "change",
        "open",
        &["type:change", "app:demo", "epic:201"],
        "",
    );
    write_issue(
        invalid_root.path(),
        205,
        "change",
        "open",
        &["type:change", "app:demo", "epic:200", "parent:206"],
        "",
    );

    let invalid_before = issue_snapshot(invalid_root.path());
    let invalid = run_aw(
        invalid_root.path(),
        &["wi", "graph", "--project", "demo", "--json"],
    );
    assert!(!invalid.status.success());
    let invalid = stdout_json(&invalid);
    assert_eq!(invalid["action"], "blocked");
    assert_eq!(invalid["valid"], false);
    assert!(invalid["next"]["command"]
        .as_str()
        .is_some_and(|command| command.starts_with("aw wi show ")));
    let codes = invalid["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .collect::<BTreeSet<_>>();
    for expected in [
        "unowned_change",
        "missing_epic_parent",
        "cross_project_epic_parent",
        "change_cannot_parent",
        "multiple_epic_owners",
    ] {
        assert!(codes.contains(expected), "missing {expected}: {invalid:#}");
    }
    assert!(invalid["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .all(|diagnostic| diagnostic["remediation_target"]
            .as_str()
            .is_some_and(|value| !value.is_empty())));
    assert_eq!(invalid_before, issue_snapshot(invalid_root.path()));
}

// HANDWRITE-END

// @spec apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/taxonomy.py
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-four-type-taxonomy" tracker="#2593" reason="The compiled CLI fixture proves canonical four-type labels and that only change leaves enter the executable graph."

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
        panic!("{command} did not emit JSON: {error}\nstdout={stdout}\nstderr={stderr}")
    })
}

#[tokio::test]
async fn four_types_round_trip_and_only_change_enters_executable_graph() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());

    let help = run_aw(root.path(), &["wi", "create", "--help"]);
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("[possible values: epic, change, spike, report]"));
    assert!(help.contains("Closed enum: epic | change | spike | report"));

    let backend = LocalBackend::from_project_root(root.path());
    for (index, kind) in ["epic", "change", "spike", "report"]
        .into_iter()
        .enumerate()
    {
        let mut args = vec![
            "wi",
            "create",
            "--title",
            kind,
            "--type",
            kind,
            "--project",
            "demo",
        ];
        if kind == "epic" {
            args.extend(["--priority", "p1"]);
        }
        let created = successful_json(&run_aw(root.path(), &args), "aw wi create");
        assert_eq!(created["action"], "dispatch");
        let slug = created["slug"].as_str().unwrap();
        let issue = backend.get(slug).await.unwrap().unwrap();
        assert_eq!(issue.issue_type.as_str(), kind);
        assert!(
            issue.labels.contains(&format!("type:{kind}")),
            "missing canonical label for fixture {index}: {:?}",
            issue.labels
        );
    }

    let graph = successful_json(
        &run_aw(root.path(), &["wi", "graph", "--project", "demo", "--json"]),
        "aw wi graph",
    );
    assert_eq!(graph["valid"], true);
    assert_eq!(graph["changes"].as_array().unwrap().len(), 1);
    assert!(graph["diagnostics"].as_array().unwrap().is_empty());
}

// HANDWRITE-END

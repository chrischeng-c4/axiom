// @spec apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/spike_terminal.py
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:wi-spike-terminal" tracker="#2595" reason="The compiled CLI fixture proves decision/no-action and gave_up terminal records without entering the product lifecycle."

use agentic_workflow::issues::{IssueBackend, IssueState, LocalBackend};
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
        panic!(
            "{command} did not emit one JSON envelope: {error}\nstdout={stdout}\nstderr={stderr}"
        )
    })
}

fn final_json(output: &Output, command: &str) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{command} failed:\nstdout={stdout}\nstderr={stderr}"
    );
    serde_json::from_str(stdout.lines().last().unwrap_or_default()).unwrap_or_else(|error| {
        panic!("{command} has no final JSON envelope: {error}\nstdout={stdout}\nstderr={stderr}")
    })
}

#[tokio::test]
async fn spike_converges_to_decided_or_gave_up_and_never_enters_product_loop() {
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let backend = LocalBackend::from_project_root(root.path());

    let decided = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "create",
                "--title",
                "Choose retry policy",
                "--type",
                "spike",
                "--project",
                "demo",
            ],
        ),
        "create decided Spike",
    );
    let decided_slug = decided["slug"].as_str().unwrap();
    let goal = final_json(
        &run_aw(root.path(), &["goal", "wi", decided_slug]),
        "inspect Spike goal",
    );
    assert_eq!(goal["action"], "blocked");
    assert!(goal["next"]["command"]
        .as_str()
        .unwrap()
        .starts_with("aw wi spike resolve"));
    assert!(goal["artifact_quality_profile"].is_null());
    let missing_exit = run_aw(
        root.path(),
        &[
            "wi",
            "spike",
            "resolve",
            decided_slug,
            "--decision",
            "Use bounded exponential backoff.",
        ],
    );
    assert!(!missing_exit.status.success());

    let terminal = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "spike",
                "resolve",
                decided_slug,
                "--decision",
                "Use bounded exponential backoff.",
                "--no-action",
            ],
        ),
        "resolve Spike",
    );
    assert_eq!(terminal["terminal_state"], "decided");
    assert_eq!(terminal["completion"]["workflow_complete"], true);
    let decided_issue = backend.get(decided_slug).await.unwrap().unwrap();
    assert_eq!(decided_issue.state, IssueState::Closed);
    assert!(decided_issue.body.contains("Status: decided"));
    assert!(decided_issue.body.contains("Follow-up: no-action"));

    let expired_body = "## Question\n\nWhat should expire?\n\n## Evidence Plan\n\n- Gather evidence.\n\n## Exit Criteria\n\n- Record a decision.\n\n## Timebox\n\nExpires At: 2000-01-01T00:00:00Z\n";
    let expired = successful_json(
        &run_aw(
            root.path(),
            &[
                "wi",
                "create",
                "--title",
                "Expired investigation",
                "--type",
                "spike",
                "--project",
                "demo",
                "--body",
                expired_body,
            ],
        ),
        "create expired Spike",
    );
    let expired_slug = expired["slug"].as_str().unwrap();
    let terminal = successful_json(
        &run_aw(root.path(), &["wi", "spike", "expire", expired_slug]),
        "expire Spike",
    );
    assert_eq!(terminal["terminal_state"], "gave_up");
    let expired_issue = backend.get(expired_slug).await.unwrap().unwrap();
    assert_eq!(expired_issue.state, IssueState::Closed);
    assert!(expired_issue.body.contains("Status: gave_up"));
}

// HANDWRITE-END

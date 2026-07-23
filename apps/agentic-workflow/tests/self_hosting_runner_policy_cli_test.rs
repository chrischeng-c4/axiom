// @spec apps/agentic-workflow/tech-design/surface/specs/aw-self-hosting-runner-policy.md#e2e-test
// HANDWRITE-BEGIN gap="missing-generator:e2e-test:self-hosting-runner-policy" tracker="#1501" reason="The regression proof needs isolated process invocations and a local issue backend to prove admission occurs before runner mutation."

use agentic_workflow::issues::{Issue, IssueBackend, IssueState, IssueType, LocalBackend};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn write_aw_config(root: &Path) {
    std::fs::create_dir_all(root.join("tech-design")).unwrap();
    std::fs::write(
        root.join("aw.toml"),
        r#"
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "agentic-workflow"
label = "app:agentic-workflow"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "agentic-workflow"
paths = ["**"]
target = "rust"
"#,
    )
    .unwrap();
}

fn run_aw(root: &Path, args: &[&str], local_backend: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aw"));
    command.args(args).current_dir(root);
    if local_backend {
        command.env("AW_FIXTURE_LOCAL_BACKEND", "1");
    }
    command.output().expect("run aw fixture command")
}

fn json_output(output: &Output, args: &[&str]) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| {
        panic!(
            "aw {} did not emit one JSON policy envelope: {error}\nstdout={stdout}\nstderr={}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_policy_envelope(value: &Value, expected_root_kind: &str) {
    assert_eq!(value["schema_version"], "aw.cli.v1");
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["action"], "self_hosting_policy");
    assert_eq!(value["root"]["kind"], expected_root_kind);
    assert_eq!(value["completion"]["workflow_complete"], false);
    assert_eq!(value["next"]["kind"], "policy");
    assert!(value["next"].get("command").is_none());
    assert_eq!(value["policy_mode"], "sanctioned_direct_commit");
    assert!(value["hard_gates"].as_array().is_some_and(|gates| gates
        .iter()
        .any(|gate| gate == "capability_work_root_alignment")));
    assert!(value["advisory_axes"]
        .as_array()
        .is_some_and(|axes| axes.iter().any(|axis| axis == "traceability")));
    assert!(value.get("invoke").is_none());
    assert!(!value["next"].to_string().contains("aw goal capability"));
}

fn tree_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = std::fs::read_dir(current)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                collect(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in self_hosting_runner_policy_cli_test.rs is hand-written pending codegen support">
#[test]
fn self_hosting_project_and_capability_roots_are_rejected_before_mutation() {
    let temp = tempfile::tempdir().unwrap();
    write_aw_config(temp.path());
    let before = tree_snapshot(temp.path());

    let project_args = [
        "goal",
        "capability",
        "--project",
        "agentic-workflow",
        "--non-interactive",
        "--max-ticks",
        "1",
    ];
    let project = run_aw(temp.path(), &project_args, false);
    assert!(project.status.success());
    assert_policy_envelope(&json_output(&project, &project_args), "project");

    let capability_args = [
        "goal",
        "capability",
        "workflow-root-runner",
        "--project",
        "agentic-workflow",
    ];
    let capability = run_aw(temp.path(), &capability_args, false);
    assert!(capability.status.success());
    assert_policy_envelope(&json_output(&capability, &capability_args), "capability");
    assert_eq!(tree_snapshot(temp.path()), before);
}
// </HANDWRITE>

#[tokio::test]
async fn self_hosting_work_item_root_is_rejected_before_loop_state_or_dispatch() {
    let temp = tempfile::tempdir().unwrap();
    write_aw_config(temp.path());
    let backend = LocalBackend::from_project_root(temp.path());
    backend
        .create(&Issue {
            issue_type: IssueType::Bug,
            title: "Self-hosting runner policy fixture".to_string(),
            state: IssueState::Open,
            id: None,
            github_id: Some(1501),
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec!["app:agentic-workflow".to_string()],
            created_at: None,
            updated_at: None,
            slug: "1501".to_string(),
            body: "# Self-hosting runner policy fixture\n".to_string(),
            related: Vec::new(),
            implements: Vec::new(),
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: Vec::new(),
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        })
        .await
        .unwrap();
    let before = tree_snapshot(temp.path());

    let args = ["goal", "wi", "1501"];
    let output = run_aw(temp.path(), &args, true);
    assert!(output.status.success());
    assert_policy_envelope(&json_output(&output, &args), "wi");
    assert_eq!(tree_snapshot(temp.path()), before);
}

#[test]
fn self_hosting_health_reports_policy_and_never_points_back_to_root_runner() {
    let temp = tempfile::tempdir().unwrap();
    write_aw_config(temp.path());
    let args = ["health", "--project", "agentic-workflow"];
    let output = run_aw(temp.path(), &args, false);
    let value = json_output(&output, &args);

    assert_eq!(value["policy_mode"], "sanctioned_direct_commit");
    assert!(value["hard_gates"].as_array().is_some_and(|gates| gates
        .iter()
        .any(|gate| gate == "configured_ec_claim_verification")));
    assert!(value["advisory_axes"]
        .as_array()
        .is_some_and(|axes| axes.iter().any(|axis| axis == "cold_rebuild")));
    assert!(!value["next"].to_string().contains("aw goal capability"));
}

// HANDWRITE-END

use agentic_workflow::issues::types::{IssueState, IssueType};
use agentic_workflow::issues::{Issue, IssueBackend, LocalBackend};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

const WI: &str = "2307";
const PROJECT: &str = "python-project";
const EPIC_ID: u64 = 9_230_700;
const MAX_STEPS: usize = 20;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/python-project")
}

fn copy_fixture(target: &Path) {
    for entry in walkdir::WalkDir::new(fixture_root()) {
        let entry = entry.expect("walk dogfood fixture");
        let relative = entry
            .path()
            .strip_prefix(fixture_root())
            .expect("fixture-relative path");
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&destination).expect("create fixture directory");
        } else {
            std::fs::copy(entry.path(), &destination).expect("copy fixture file");
        }
    }
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aw"))
        .args(args)
        .current_dir(root)
        .env("AW_FIXTURE_LOCAL_BACKEND", "1")
        .env("AW_DISABLE_CAP", "1")
        .env("RUST_BACKTRACE", "1")
        .output()
        .unwrap_or_else(|error| panic!("run aw {args:?}: {error}"))
}

fn run_ok(root: &Path, args: &[&str]) -> Output {
    let output = run(root, args);
    assert!(
        output.status.success(),
        "aw {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn last_json(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim())
        .ok()
        .or_else(|| {
            stdout
                .lines()
                .rev()
                .find(|line| !line.trim().is_empty())
                .and_then(|line| serde_json::from_str(line).ok())
        })
        .unwrap_or_else(|| {
            panic!(
                "expected terminal JSON record\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        })
}

fn goal(root: &Path, id: &str) -> Value {
    last_json(&run_ok(root, &["goal", "wi", id]))
}

fn next_command(envelope: &Value) -> String {
    envelope["invoke"]["command"]
        .as_str()
        .or_else(|| envelope["next"]["command"].as_str())
        .expect("goal envelope must contain a runnable command")
        .to_string()
}

fn run_command(root: &Path, command: &str) -> Output {
    assert!(command.starts_with("aw "), "unexpected command: {command}");
    let args = command.split_whitespace().skip(1).collect::<Vec<_>>();
    run_ok(root, &args)
}

fn accept_review_payload(path: &Path) {
    let mut payload: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read review payload"))
            .expect("parse review payload");
    payload["decision"] = Value::String("accepted".to_string());
    payload["reviewer_kind"] = Value::String("agent".to_string());
    payload["reviewed_by"] = Value::String("independent-python-dogfood-reviewer".to_string());
    payload["summary"] = Value::String(
        "The bounded user-model/import cases have independent oracles and complete staged coverage."
            .to_string(),
    );
    payload["checklist"] = serde_json::json!({
        "capability_claim_coverage": true,
        "required_dimensions": true,
        "assertions_specific": true,
        "oracle_independent": true,
        "loopholes_checked": true,
        "false_green_risk_checked": true
    });
    std::fs::write(
        path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .expect("write accepted review payload");
}

fn init_git(root: &Path) {
    for args in [
        vec!["init", "-b", "project-test"],
        vec!["config", "user.email", "dogfood@example.test"],
        vec!["config", "user.name", "Python Dogfood"],
        vec!["config", "commit.gpgsign", "false"],
        vec!["add", "-A"],
        vec!["commit", "-m", "seed python artifact dogfood"],
    ] {
        let output = Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .expect("run git fixture command");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn issue(slug: &str, issue_type: IssueType, state: IssueState) -> Issue {
    Issue {
        issue_type,
        title: format!("{slug} Python artifact dogfood"),
        state,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("app:{PROJECT}")],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!(
            "# Python artifact dogfood\n\n## Capability Alignment\n\nuser-model-import\n\n## Scope\n\nBounded user-model/import slice.\n\n## Acceptance Criteria\n\n- terminal Python artifact lifecycle\n\n## Reference Context\n\nIssue #2307.\n"
        ),
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
    }
}

async fn seed_graph(root: &Path) {
    let backend = LocalBackend::from_project_root(root);
    let mut epic = issue("python-project-epic", IssueType::Epic, IssueState::Closed);
    epic.github_id = Some(EPIC_ID);
    backend
        .create(&epic)
        .await
        .expect("seed closed parent epic");

    let mut change = issue(WI, IssueType::Enhancement, IssueState::Open);
    change.related = vec![format!("#{EPIC_ID}")];
    backend.create(&change).await.expect("seed dogfood change");
}

#[tokio::test]
async fn python_artifact_dogfood_reaches_terminal_workflow_completion() {
    let started = Instant::now();
    let temp = tempfile::tempdir().expect("dogfood tempdir");
    copy_fixture(temp.path());
    init_git(temp.path());
    seed_graph(temp.path()).await;

    let mut steps = 0usize;
    let first = goal(temp.path(), WI);
    assert_eq!(
        next_command(&first),
        format!("aw ec check --project {PROJECT} --wi {WI}")
    );
    run_command(temp.path(), &next_command(&first));
    steps += 1;

    let review_goal = goal(temp.path(), WI);
    assert_eq!(
        next_command(&review_goal),
        format!("aw ec review --project {PROJECT} --wi {WI}")
    );
    let pending = last_json(&run_ok(
        temp.path(),
        &["ec", "review", "--project", PROJECT, "--wi", WI, "--json"],
    ));
    assert_eq!(pending["status"], "pending_agent_review");
    let payload = PathBuf::from(
        pending["payload_path"]
            .as_str()
            .expect("review payload path"),
    );
    accept_review_payload(&payload);
    run_ok(
        temp.path(),
        &[
            "ec",
            "review",
            "--project",
            PROJECT,
            "--wi",
            WI,
            "--evidence-file",
            payload.to_str().unwrap(),
            "--json",
        ],
    );
    steps += 1;

    for expected in [
        "aw ec lock --project python-project --wi 2307",
        "aw td check ",
        "aw ec verify --project python-project --required-only --stage td --wi 2307",
        "aw cb gen --target python --source-root ",
        "aw cb fill 2307",
        "aw cb check 2307",
        "aw ec verify --project python-project --required-only --stage cb --wi 2307",
        "aw wi close 2307 --push",
    ] {
        assert!(steps < MAX_STEPS, "dogfood exceeded bounded step budget");
        let envelope = goal(temp.path(), WI);
        let command = next_command(&envelope);
        assert!(
            command.starts_with(expected),
            "expected next command prefix {expected:?}, got {command:?}\nenvelope={envelope:#}"
        );
        let output = run_command(temp.path(), &command);
        if expected == "aw cb check 2307" {
            let cb_check = last_json(&output);
            assert!(
                next_command(&cb_check).starts_with(
                    "aw ec verify --project python-project --required-only --stage cb --wi 2307"
                ),
                "Python cb check did not hand off to CB-stage EC verification: {cb_check:#}"
            );
        }
        steps += 1;
    }

    let closed = goal(temp.path(), WI);
    assert_eq!(closed["action"], "done");
    assert_eq!(closed["completion"]["root_complete"], true);

    let parent = goal(temp.path(), &EPIC_ID.to_string());
    assert_eq!(parent["action"], "done");
    let rollup_command = next_command(&parent);
    assert!(rollup_command.contains(PROJECT), "{rollup_command}");
    let rollup = last_json(&run_command(temp.path(), &rollup_command));
    assert_eq!(
        rollup["completion"]["workflow_complete"], true,
        "Python dogfood project did not reach workflow completion: {rollup:#}"
    );

    let backend = LocalBackend::from_project_root(temp.path());
    let persisted = backend
        .get(WI)
        .await
        .expect("read dogfood issue")
        .expect("dogfood issue exists");
    assert_eq!(persisted.state, IssueState::Closed);
    assert!(temp.path().join("tech-design/td.lock").is_file());
    assert!(temp.path().join("external-contracts/ec.lock").is_file());
    assert!(temp.path().join("src/user_model/model.py").is_file());
    for evidence in ["behavior", "security", "stability", "efficiency"] {
        assert!(
            temp.path()
                .join(format!("external-contracts/evidence/{evidence}.json"))
                .is_file(),
            "missing {evidence} evidence"
        );
    }
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "bounded dogfood took {:?}",
        started.elapsed()
    );
}

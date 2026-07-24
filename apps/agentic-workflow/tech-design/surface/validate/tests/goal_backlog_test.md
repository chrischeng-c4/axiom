---
id: projects-agentic-workflow-tests-cli-tests-goal-backlog-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: goal-unified-loop-verb
    claim: goal-unified-loop-verb
    coverage: full
    rationale: "Real-binary fixture proof for `aw goal backlog --project <p>` (#1899 R7, #2389): an accepted reviewed epic graph parks one HITL-blocked change, dispatches its runnable sibling, and then reports the parked leaf without re-atomizing the reviewed epic."
---

# Standardized apps/agentic-workflow/tests/cli/tests/goal_backlog_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/tests/cli/tests/goal_backlog_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/tests/cli/tests/goal_backlog_test.rs -->
```rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/goal_backlog_test.md#source
// CODEGEN-BEGIN
//! `aw goal backlog --project <p>` fixture proof (#1899 R7, AC6): a mixed
//! reviewed graph -- one HITL-blocked change and one runnable sibling under
//! one prioritized epic -- drains deterministically. The first invocation
//! parks the blocked leaf and dispatches the runnable sibling. Once runnable
//! closes, later invocations remain terminal with the blocked leaf reported;
//! the open epic is never redispatched for atomization.

use std::path::Path;
use std::process::Command;

use agentic_workflow::cli::loop_state::{upsert_loop_state, LoopState};
use agentic_workflow::issues;
use agentic_workflow::issues::{Issue, IssueBackend, IssueState, IssueType, LocalBackend};

fn skip_unless_binaries() -> Option<(std::path::PathBuf, String)> {
    let git = agentic_workflow::git::find_git_bin()?;
    let aw_bin = std::env::var("CARGO_BIN_EXE_aw").ok()?;
    Some((git, aw_bin))
}

/// Matches `fixture_loop_test.rs::init_seed_repo`: a from-scratch repo on a
/// non-"main" branch is enough for `find_project_root`/`aw.toml` discovery;
/// this fixture never drives a WI's TD/CB lifecycle, only `aw goal
/// backlog`'s own probe-and-select tick.
fn init_seed_repo(git: &Path, root: &Path) {
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "project-test"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();
}

/// Matches `fixture_loop_test.rs::write_fixture_aw_toml_with_local_issue_platform`:
/// a resolvable project row plus `[agentic_workflow.issue_platform] type =
/// "local"`, the config shape `issues::resolve_default_backend` needs under
/// the fixture-only `AW_FIXTURE_LOCAL_BACKEND=1` escape hatch (#1348).
fn write_fixture_aw_toml_with_local_issue_platform(root: &Path, project: &str) {
    std::fs::write(
        root.join("aw.toml"),
        format!(
            "[agentic_workflow.workspace]\nmode = \"in_place\"\n\n\
             [[projects]]\nname = \"{project}\"\nlabel = \"app:{project}\"\npath = \".\"\n\n\
             [[projects.workspaces]]\nname = \"{project}\"\npaths = [\"**\"]\ntarget = \"rust\"\n\n\
             [agentic_workflow.issue_platform]\ntype = \"local\"\n"
        ),
    )
    .unwrap();
}

fn structured_change_body(scope: &str) -> String {
    format!(
        "## Capability Alignment\n\n\
         Capability: workflow-root-runner\n\
         Capability Gap: reviewed-graph-selection\n\
         Progress Evidence: compiled backlog fixture\n\n\
         ## Scope\n\n\
         ### In Scope\n- {scope}\n\n\
         ### Out of Scope\n- Unrelated graph behavior.\n\n\
         ## Acceptance Criteria\n\n- {scope} is observable.\n\n\
         ## Reference Context\n\n- Issue #2389\n"
    )
}

/// `github_id` must be set: `LocalBackend::create` silently demotes any
/// `state: Open` issue with no `github_id`/`gitlab_id` to `Draft` (real
/// local-only drafts start unpublished), which would drop it out of the
/// planner and graph's open issue inventory. Fabricated numeric ids also give
/// the fixture stable issue-platform identity and deterministic graph order.
fn base_issue(issue_type: IssueType, slug: &str, github_id: u64, labels: Vec<String>) -> Issue {
    Issue {
        issue_type,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: Some(github_id),
        gitlab_id: None,
        url: None,
        author: None,
        labels,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
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

fn run_backlog(aw_bin: &str, root: &Path, project: &str) -> serde_json::Value {
    let output = Command::new(aw_bin)
        .args(["goal", "backlog", "--project", project])
        .current_dir(root)
        .env(issues::AW_FIXTURE_LOCAL_BACKEND_ENV, "1")
        .output()
        .expect("spawn `aw goal backlog`");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "`aw goal backlog --project {project}` exited non-zero ({:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not a single JSON envelope: {e}\nstdout:\n{stdout}"))
}

fn run_json(aw_bin: &str, root: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::new(aw_bin)
        .args(args)
        .current_dir(root)
        .env(issues::AW_FIXTURE_LOCAL_BACKEND_ENV, "1")
        .env("AW_AGENT_ID", "author-agent")
        .output()
        .expect("spawn aw fixture command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    if let Ok(value) = serde_json::from_str(stdout.trim()) {
        return value;
    }
    stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
        .unwrap_or_else(|| panic!("aw fixture command did not emit JSON: {stdout}"))
}

fn publish_reviewed_graph(aw_bin: &str, root: &Path) {
    let plan = run_json(aw_bin, root, &["wi", "plan", "--project", "demo", "--json"]);
    let payload_path = std::path::PathBuf::from(plan["payload_path"].as_str().unwrap());
    let mut payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&payload_path).unwrap()).unwrap();
    payload["decision"] = serde_json::Value::String("accepted".to_string());
    payload["reviewed_by"] = serde_json::Value::String("independent-reviewer".to_string());
    payload["summary"] = serde_json::Value::String(
        "Reviewed the exact backlog graph and transaction manifest.".to_string(),
    );
    for key in [
        "scope_coverage",
        "bounded_candidates",
        "tracker_reconciliation",
        "priority_consistent",
        "no_duplicate_wis",
        "publication_safe",
    ] {
        payload["checklist"][key] = serde_json::Value::Bool(true);
    }
    payload["findings"] = serde_json::Value::Array(Vec::new());
    std::fs::write(
        &payload_path,
        format!("{}\n", serde_json::to_string_pretty(&payload).unwrap()),
    )
    .unwrap();
    let evidence = payload_path.to_string_lossy().to_string();
    let applied = run_json(
        aw_bin,
        root,
        &["wi", "plan-review", "--evidence-file", &evidence, "--json"],
    );
    assert_eq!(applied["transaction"]["status"], "complete");
}

#[tokio::test]
async fn goal_backlog_drains_runnable_blocked_and_epic_mix() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);
    write_fixture_aw_toml_with_local_issue_platform(root, "demo");

    let backend = LocalBackend::from_project_root(root);

    let mut epic = base_issue(
        IssueType::Epic,
        "ccc-epic-wi",
        103,
        vec![
            "app:demo".to_string(),
            "type:epic".to_string(),
            "priority:p0".to_string(),
        ],
    );
    epic.body = "## Requirements\n\n- R1: park the blocked child.\n- R2: run the ready sibling.\n"
        .to_string();
    backend.create(&epic).await.expect("seed epic WI");

    // The high-priority child carries the project and epic labels required by
    // the reviewed graph, plus an `<!-- aw:loop-state -->`
    // block with `next_action: None`, the one `wi_envelope` path that
    // reports `requires_hitl: true`/`action: "blocked"` for a WI the
    // project's backlog can actually enumerate (`loop_state_envelope`'s
    // `None => blocked_envelope(..., "loop has no next act: ec verifier is
    // blocked or undefined -- human input required", true)` arm).
    let mut blocked = base_issue(
        IssueType::Enhancement,
        "aaa-blocked-wi",
        101,
        vec![
            "app:demo".to_string(),
            "type:change".to_string(),
            "epic:103".to_string(),
            "priority:p0".to_string(),
        ],
    );
    blocked.body = structured_change_body("park the blocked child");
    let blocked_loop_state = LoopState {
        version: 1,
        issue_id: "aaa-blocked-wi".to_string(),
        next_action: None,
        ..Default::default()
    };
    blocked.body = upsert_loop_state(&blocked.body, &blocked_loop_state)
        .expect("render blocked WI's loop-state block");
    backend.create(&blocked).await.expect("seed blocked WI");

    // Sorts second: a plain open change WI with a resolvable project label
    // and no phase yet -- `wi_change_lifecycle_step` dispatches it straight
    // to `aw ec draft ...` (no TD/CB state required), never blocking.
    let mut runnable = base_issue(
        IssueType::Enhancement,
        "bbb-runnable-wi",
        102,
        vec![
            "app:demo".to_string(),
            "type:change".to_string(),
            "epic:103".to_string(),
            "priority:p1".to_string(),
        ],
    );
    runnable.body = structured_change_body("run the ready sibling");
    backend.create(&runnable).await.expect("seed runnable WI");

    publish_reviewed_graph(&aw_bin, root);

    // Tick 1: the drain walks blocked -> runnable -> selects runnable
    // (parking blocked along the way) within this single invocation.
    let first = run_backlog(&aw_bin, root, "demo");
    assert_eq!(
        first["action"], "dispatch",
        "expected the runnable WI to be selected, got: {first:#?}"
    );
    assert_eq!(first["completion"]["workflow_complete"], false);
    let first_command = first["next"]["command"]
        .as_str()
        .expect("dispatch envelope must carry next.command");
    assert!(
        first_command.contains("102"),
        "expected the runnable WI's own `aw goal wi <id>` hand-off (id 102), got: {first_command}"
    );
    assert!(
        first_command.starts_with("aw goal wi "),
        "backlog hand-off must reuse the shared `aw goal wi <id>` command, got: {first_command}"
    );

    // Simulate the host driving the selected WI to its own terminal (out of
    // scope for this fixture -- `fixture_loop_test.rs` already covers the
    // real `aw goal wi <id>` chain end to end).
    backend
        .close("bbb-runnable-wi", None)
        .await
        .expect("close runnable WI");

    // Tick 2: the blocked WI stays parked and the reviewed epic has no other
    // ready leaf. The drain is terminal for this pass and never re-atomizes
    // the already-reviewed epic.
    let second = run_backlog(&aw_bin, root, "demo");
    assert_eq!(
        second["action"], "done",
        "expected only the parked reviewed leaf to remain, got: {second:#?}"
    );
    assert_eq!(second["completion"]["workflow_complete"], true);
    assert!(!second.to_string().contains("atomize"));

    // Tick 3: repeated invocation remains terminal and reports the parked set.
    let third = run_backlog(&aw_bin, root, "demo");
    assert_eq!(
        third["action"], "done",
        "expected a terminal envelope once only the parked WI remains open, got: {third:#?}"
    );
    assert_eq!(third["completion"]["workflow_complete"], true);
    let agent_prompt = third["agent_prompt"]
        .as_str()
        .expect("terminal envelope must carry agent_prompt");
    assert!(
        agent_prompt.contains("101"),
        "terminal report must name the still-parked WI (id 101), got: {agent_prompt}"
    );

    // A fourth invocation must not spin/re-select anything already
    // accounted for -- still terminal, same parked WI.
    let fourth = run_backlog(&aw_bin, root, "demo");
    assert_eq!(fourth["action"], "done");
    assert_eq!(fourth["completion"]["workflow_complete"], true);
}
// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tests/cli/tests/goal_backlog_test.rs
    action: create
    impl_mode: codegen
    section: source
    description: |
      Issue #1899 R7 and #2389 real-binary fixture proof for `aw goal backlog
      --project <p>`: seeds one p0 epic with one HITL-blocked p0 child and one
      runnable p1 child, publishes the exact graph through `aw wi plan` plus
      independent accepted review, and drives the compiled backlog root.
      Tick 1 parks the blocked leaf and dispatches the runnable sibling via
      `aw goal wi <id>`; after the runnable child closes, ticks 2-4 remain
      terminal and name the parked leaf. Every result proves the already
      reviewed epic is never sent back through atomization.
```

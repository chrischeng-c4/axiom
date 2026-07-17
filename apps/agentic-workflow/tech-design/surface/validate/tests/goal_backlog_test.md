---
id: projects-agentic-workflow-tests-cli-tests-goal-backlog-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: goal-unified-loop-verb
    claim: goal-unified-loop-verb
    coverage: full
    rationale: "Real-binary fixture proof for `aw goal backlog --project <p>` (#1899 R7, AC6): a mixed open backlog of one HITL-blocked change WI, one runnable change WI, and one open epic drains deterministically across three invocations -- runnable dispatch via the shared `aw goal wi <id>` hand-off, epic dispatch via the existing atomize dispatch rule, then a terminal envelope naming the still-parked WI and its reason, with a fourth invocation proving no spinning/premature completion."
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
//! open backlog -- one HITL-blocked change WI, one runnable change WI, and
//! one open epic -- drains deterministically. A single invocation parks the
//! blocked WI (recording its reason) and dispatches the first non-blocked
//! candidate in priority/id order; once that candidate closes, the next
//! invocation advances to the epic (routed through the existing atomize
//! dispatch rule); once the epic closes too, the final invocation reports
//! terminal `completion.workflow_complete = true` and names the still-open
//! parked WI plus its reason. No invocation ever spins (each is one bounded
//! probe-and-select tick) or completes prematurely while the blocked WI is
//! still open and unresolved.

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
            "[[projects]]\nname = \"{project}\"\npath = \".\"\n\n\
             [[projects.workspaces]]\nname = \"{project}\"\npaths = [\"**\"]\ntarget = \"rust\"\n\n\
             [agentic_workflow.issue_platform]\ntype = \"local\"\n"
        ),
    )
    .unwrap();
}

/// `github_id` must be set: `LocalBackend::create` silently demotes any
/// `state: Open` issue with no `github_id`/`gitlab_id` to `Draft` (real
/// local-only drafts start unpublished), which would drop it out of
/// `list_open_project_issues`'s `state: Some(IssueState::Open)` filter.
/// Fabricated numeric ids also give the fixture a deterministic id-order
/// tiebreak matching `list_open_project_issues`'s sort.
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

    // Alphabetically first so it sorts to the front of the priority-tied
    // (no priority label -> rank 4 for all three) drain order. Carries the
    // project label (so `list_open_project_issues` can even see it -- a
    // WI missing the project label entirely is invisible to this query, not
    // a per-project "blocked" candidate) plus an `<!-- aw:loop-state -->`
    // block with `next_action: None`, the one `wi_envelope` path that
    // reports `requires_hitl: true`/`action: "blocked"` for a WI the
    // project's backlog can actually enumerate (`loop_state_envelope`'s
    // `None => blocked_envelope(..., "loop has no next act: ec verifier is
    // blocked or undefined -- human input required", true)` arm).
    let mut blocked = base_issue(
        IssueType::Enhancement,
        "aaa-blocked-wi",
        101,
        vec!["app:demo".to_string()],
    );
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
    let runnable = base_issue(
        IssueType::Enhancement,
        "bbb-runnable-wi",
        102,
        vec!["app:demo".to_string()],
    );
    backend.create(&runnable).await.expect("seed runnable WI");

    // Sorts third: an open epic with a resolvable project label --
    // `open_epic_envelope` dispatches `aw wi atomize --project demo`,
    // never blocking either.
    let epic = base_issue(
        IssueType::Epic,
        "ccc-epic-wi",
        103,
        vec!["app:demo".to_string()],
    );
    backend.create(&epic).await.expect("seed epic WI");

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

    // Tick 2: the blocked WI stays parked (still open, still unresolved);
    // the runnable WI is gone from the open set; the epic is next in
    // priority order. The drain always hands off via the shared
    // `aw goal wi <id>` command (never the underlying envelope's own
    // `next.command`, e.g. `aw wi atomize --project demo`) -- the host
    // re-observes the epic's actual atomize routing itself once it runs
    // that command (`open_epic_envelope`, exercised end to end by
    // `fixture_loop_test.rs`'s epic-rollup coverage).
    let second = run_backlog(&aw_bin, root, "demo");
    assert_eq!(
        second["action"], "dispatch",
        "expected the open epic to be selected next, got: {second:#?}"
    );
    assert_eq!(second["completion"]["workflow_complete"], false);
    let second_command = second["next"]["command"]
        .as_str()
        .expect("dispatch envelope must carry next.command");
    assert_eq!(
        second_command, "aw goal wi 103",
        "epic candidates must still hand off via the shared `aw goal wi <id>` command"
    );

    backend
        .close("ccc-epic-wi", None)
        .await
        .expect("close epic WI");

    // Tick 3: every open WI is now either closed (runnable, epic) or parked
    // (blocked) -- terminal, reporting the parked set.
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
      Issue #1899 R7 (AC6) real-binary fixture proof for `aw goal backlog
      --project <p>`: seeds one HITL-blocked change WI (project label plus
      an `<!-- aw:loop-state -->` block with `next_action: None`, the one
      `wi_envelope` path that reports `requires_hitl: true` for a WI the
      project's backlog can actually enumerate -- a WI missing the project
      label entirely is invisible to `list_open_project_issues`'s label
      filter, not a per-project "blocked" candidate), one runnable change WI
      (project label, no phase, no loop-state block), and one open epic
      (project label) via `LocalBackend` under the fixture-only
      `AW_FIXTURE_LOCAL_BACKEND=1` escape hatch (#1348) with fabricated
      `github_id`s (`LocalBackend::create` silently demotes a labelless-id
      `state: Open` issue to `Draft`, dropping it out of the open-state
      filter), matching `fixture_loop_test.rs`'s repo/aw.toml seeding
      pattern. Drives four subprocess invocations of the real `aw` binary:
      tick 1 dispatches the runnable WI via the shared `aw goal wi <id>`
      hand-off while the blocked WI is parked; tick 2 (after closing the
      runnable WI out of band) selects the open epic next, still via the
      same shared `aw goal wi <id>` hand-off (the drain never forwards a
      candidate's own underlying `next.command`, e.g. the epic's
      `aw wi atomize --project demo` route); tick 3 (after closing the epic)
      reports a terminal `completion.workflow_complete = true` envelope
      naming the still-parked WI's id in `agent_prompt`; tick 4 proves no
      spinning/premature completion.
```

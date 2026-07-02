---
id: projects-agentic-workflow-tests-cli-tests-td-no-merge-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: remove-td-merge-command
    claim: remove-td-merge-command
    coverage: full
    rationale: "Regression tests prove the TD merge command is absent from the CLI surface and terminal lifecycle closure uses Cb-CodeCheck."
---

# Standardized projects/agentic-workflow/tests/cli/tests/td_no_merge_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/td_no_merge_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/td_no_merge_test.rs -->
```rust
//! Regression tests proving the removed TD merge command is no longer part of the CLI surface.

use agentic_workflow::cli::commands::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn test_td_merge_subcommand_is_removed() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    assert!(
        td.find_subcommand("merge").is_none(),
        "removed TD merge command must not be registered"
    );
}

#[test]
fn test_td_merge_parse_fails() {
    let err = match Cli::try_parse_from(["aw", "td", "merge", "4124"]) {
        Ok(_) => panic!("removed TD merge command unexpectedly parsed"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("unrecognized subcommand 'merge'"),
        "unexpected parse error: {err}"
    );
}

#[test]
fn test_cb_code_check_is_terminal_lifecycle_trailer() {
    use agentic_workflow::issues::types::lifecycle_trailer;

    assert_eq!(lifecycle_trailer::CB_CODE_CHECK, "Cb-CodeCheck");
}

/// Count git log entries whose message has an exact-line `Lifecycle-Stage:
/// Cb-CodeCheck` trailer. Mirrors the line-exact matching
/// `run_check_lifecycle_terminal`'s retry gate uses internally (not a
/// substring scan), so this assertion helper proves the same thing the
/// production idempotency check proves.
fn count_cb_code_check_trailer_commits(git: &std::path::Path, root: &std::path::Path) -> usize {
    let log = std::process::Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%B%x1e"])
        .output()
        .expect("git log");
    let text = String::from_utf8_lossy(&log.stdout);
    text.split('\x1e')
        .filter(|entry| {
            entry
                .lines()
                .any(|line| line.trim_end() == "Lifecycle-Stage: Cb-CodeCheck")
        })
        .count()
}

/// #846 AC1/AC2/AC3: a WI stranded exactly where the bug leaves it — phase
/// already advanced to `td_merged` and the issue already closed (as
/// `backend.update` in the terminal path does), `score:locked` still
/// present, and no `Cb-CodeCheck` trailer commit in the log — is the exact
/// partial-failure shape left behind when `maybe_push_remote` or
/// `commit_cb_code_check_terminal` errors after `backend.update` already
/// ran. Re-running `aw td code-check <slug>` must complete the missing
/// terminal steps and exit `done` (AC1), release `score:locked` (AC2), and
/// land the `Cb-CodeCheck` trailer commit (AC3). A second retry against the
/// now fully-completed `td_merged` issue must be a clean idempotent no-op —
/// not a duplicate commit.
#[tokio::test]
async fn test_code_check_retry_completes_partial_terminal_failure() {
    use std::process::Command;

    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git binary not on PATH");
        return;
    };
    let Ok(aw_bin) = std::env::var("CARGO_BIN_EXE_aw") else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();

    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .status()
        .expect("git init");
    for (k, v) in [
        ("user.email", "test@test"),
        ("user.name", "test"),
        ("commit.gpgsign", "false"),
    ] {
        Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(["config", k, v])
            .status()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "."])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "seed"])
        .status()
        .unwrap();

    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();

    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

    let slug = "code-check-retry-test";
    let backend = LocalBackend::from_project_root(root);
    let stranded = Issue {
        issue_type: IssueType::Enhancement,
        title: "stranded td_merged WI".to_string(),
        state: IssueState::Closed,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![
            format!("phase:{}", td_phase::TD_MERGED),
            "score:locked".to_string(),
            "score:lock:td".to_string(),
        ],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: "# stranded td_merged WI\n".to_string(),
        related: Vec::new(),
        implements: Vec::new(),
        phase: Some(td_phase::TD_MERGED.to_string()),
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
    };
    backend
        .create(&stranded)
        .await
        .expect("seed stranded td_merged issue");
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        0,
        "sanity: no Cb-CodeCheck trailer commit exists before the retry"
    );

    // Retry: re-run `aw td code-check <slug>` exactly as a caller unsticking
    // issue #846 would.
    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "retry code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "retry must emit a done envelope, got:\n{}",
        stdout
    );

    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "retry must land exactly one Cb-CodeCheck trailer commit"
    );
    let log = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%B"])
        .output()
        .expect("git log");
    let log_text = String::from_utf8_lossy(&log.stdout);
    assert!(
        log_text.contains(&format!("Lifecycle-Slug: {}", slug)),
        "Lifecycle-Slug trailer missing from log:\n{}",
        log_text
    );

    // AC2: score:locked is released.
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert!(
        !after.labels.iter().any(|l| l == "score:locked"),
        "score:locked must be released by a successful retry, labels: {:?}",
        after.labels
    );

    // Idempotent re-retry: running code-check again at the now
    // fully-completed td_merged phase must be a clean no-op done, not a
    // duplicate commit.
    let output2 = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check (second retry)");
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        output2.status.success(),
        "second retry at fully-completed td_merged should also exit 0: {}",
        stdout2
    );
    assert!(
        stdout2.contains("\"action\":\"done\""),
        "second retry must also emit done, got:\n{}",
        stdout2
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "second retry must not add a duplicate Cb-CodeCheck trailer commit"
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/td_no_merge_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source snapshot for the regression test that proves the removed
      TD merge command is absent from the CLI surface. Also covers #846: a
      partial-terminal-failure regression proving `aw td code-check` resumes
      and completes from a stranded `td_merged` phase (remote push, the
      `Cb-CodeCheck` trailer commit, and `score:locked` release), idempotently.
```

// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md#source
// CODEGEN-BEGIN
//! Regression tests proving the removed TD merge command is no longer part of the CLI surface.
//!
//! The `td merge`-removal clap-parsing assertions that used to live here
//! (`test_td_merge_subcommand_is_removed` / `test_td_merge_parse_fails`)
//! moved to `legacy_cli_removal_test.rs` (issue #856f): that file already
//! defines the identical `Cli { command: Commands }` clap harness this file
//! duplicated, and is the dedicated home for removed-command assertions.
//! This file keeps its lifecycle (terminal `aw td code-check`) tests, which
//! don't use that harness at all — they drive the real `aw` binary via
//! `CARGO_BIN_EXE_aw`.

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
    std::fs::write(root.join("aw.toml"), "").unwrap();

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
    // #842 AC (no-`td-<slug>`-branch case): this sandbox never creates a
    // `td-<slug>` branch, so the new landing step must be a structural
    // no-op and must not attempt to check out or merge anything.
    assert!(
        stdout.contains("\"status\":\"skipped\""),
        "retry with no td-<slug> branch must report landing status=skipped, got:\n{}",
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

// ---------------------------------------------------------------------------
// #807 / #1275: terminal `aw td code-check` must refuse to perform ANY
// mutation (the phase-advancing `backend.update`, remote closure, branch
// landing, terminal commit, or lock release) while a file in the WI's own
// touched scope is dirty in git — the exact shape that let a WI's
// implementation sit uncommitted while the issue still closed (Jet #797).
// ---------------------------------------------------------------------------

/// AC1a: an untracked touched-scope file (never `git add`ed at all) must
/// refuse completion, naming the dirty file and a remediation next command,
/// and must leave the issue completely untouched — no phase advance, no
/// close, no `Cb-CodeCheck` trailer commit.
#[tokio::test]
async fn test_code_check_refuses_dirty_touched_scope_untracked() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, IssueState, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);
    // The WI's own touched-scope file: present on disk but never `git add`ed
    // — the exact "implementation sitting uncommitted" shape from #807.
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/demo.rs"), "// implemented\n").unwrap();

    let slug = "dirty-scope-untracked-test";
    seed_847_open_issue(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "an untracked touched-scope file must refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/demo.rs"),
        "error message must name the dirty touched file, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("git commit") || stdout.contains("git restore"),
        "error message must carry a commit-or-restore remediation next step, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains(&format!("aw td code-check {slug}")),
        "error message must name the re-run command, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_FILLED),
        "a dirty-scope refusal must not advance phase past cb_filled"
    );
    assert_ne!(
        after.state,
        IssueState::Closed,
        "a dirty-scope refusal must not close the issue, got: {:?}",
        after.state
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        0,
        "a dirty-scope refusal must not land any Cb-CodeCheck trailer commit"
    );
}

/// AC1b: a touched-scope file that was already committed on an earlier run
/// but has since been modified again (dirty-but-tracked, not merely
/// untracked) must also refuse completion — the gate scans `git status
/// --porcelain` broadly, not only untracked entries.
#[tokio::test]
async fn test_code_check_refuses_dirty_touched_scope_modified_tracked() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, IssueState, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/demo.rs"), "// implemented\n").unwrap();
    commit_all(&git, root);
    // Modify the already-committed touched file again, without committing —
    // ordinary tracked-file dirt, distinct from the untracked case above.
    std::fs::write(root.join("src/demo.rs"), "// implemented\n// more\n").unwrap();

    let slug = "dirty-scope-modified-test";
    seed_847_open_issue(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "a modified-but-uncommitted touched-scope file must refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/demo.rs"),
        "error message must name the dirty touched file, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_FILLED),
        "a dirty-scope refusal must not advance phase past cb_filled"
    );
    assert_ne!(
        after.state,
        IssueState::Closed,
        "a dirty-scope refusal must not close the issue, got: {:?}",
        after.state
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        0,
        "a dirty-scope refusal must not land any Cb-CodeCheck trailer commit"
    );
}

/// #842 AC1-AC4: a main-launched lifecycle whose `td-<slug>` branch still
/// holds an implementation commit — the shape left behind once `td create`
/// provisions `td-<slug>` from `main` and every later TD/CB verb (gen, fill,
/// code-check) stays on it, per `should_use_td_branch` /
/// `activate_td_workspace_for_lifecycle` in td.rs — must have that branch
/// landed onto `main` as part of the terminal `code-check` step sequence:
/// the implementation commit becomes reachable from `main` (AC1), the
/// terminal trailer commit lands on `main` too instead of a branch that's
/// about to be deleted (AC2), and `td-<slug>` itself is cleaned up (AC3).
/// A second run after the branch is already gone must be an idempotent
/// landing no-op (`"status":"skipped"`), not a second merge attempt or a
/// duplicate trailer commit.
///
/// Seeds directly at the `td_merged` retry entry point (same technique as
/// the partial-terminal-failure test above) so the test exercises the new
/// landing step without also having to satisfy the fresh-entry HANDWRITE
/// marker gate.
#[tokio::test]
async fn test_code_check_lands_td_slug_branch_onto_main() {
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
    // `aw.toml` is committed as part of the seed commit (as it
    // would be in a real project) so the working tree is clean going into
    // the landing step's dirty-tree guard below.
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();
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

    let slug = "code-check-landing-test";
    let td_branch = format!("td-{}", slug);

    // Simulate `td create` provisioning `td-<slug>` from `main`, plus the
    // gen/fill implementation commit that lands on it before code-check.
    // HEAD stays on `td-<slug>` here (as every real TD/CB verb since `td
    // create` does) — code-check runs from wherever the caller last left
    // off, which is exactly the shape issue #842 reports.
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["checkout", "-b", &td_branch])
        .status()
        .unwrap();
    std::fs::write(root.join("impl.txt"), "implementation\n").unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["add", "impl.txt"])
        .status()
        .unwrap();
    Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "implement thing"])
        .status()
        .unwrap();
    let impl_sha_out = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("rev-parse impl commit");
    let impl_sha = String::from_utf8_lossy(&impl_sha_out.stdout)
        .trim()
        .to_string();

    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

    let backend = LocalBackend::from_project_root(root);
    let stranded = Issue {
        issue_type: IssueType::Enhancement,
        title: "stranded td_merged WI with implementation on td-<slug>".to_string(),
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
        branch: Some(td_branch.clone()),
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

    // AC1/AC2/AC3: code-check lands `td-<slug>` onto `main` — the
    // implementation commit becomes reachable from `main`, the trailer
    // commit ends up on `main`, and the branch is deleted.
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
        "landing code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "landing run must emit a done envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("\"status\":\"landed\""),
        "landing run must report status=landed, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("\"target\":\"main\""),
        "landing run must report target=main, got:\n{}",
        stdout
    );

    let head = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("rev-parse HEAD");
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        "main",
        "HEAD must end on the landing target (main)"
    );

    // AC1: the implementation commit is reachable from `main`.
    let ancestor = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["merge-base", "--is-ancestor", &impl_sha, "main"])
        .status()
        .expect("merge-base --is-ancestor");
    assert!(
        ancestor.success(),
        "implementation commit {} must be reachable from main after landing",
        impl_sha
    );
    assert!(
        root.join("impl.txt").exists(),
        "implementation file must be present in the main working tree after landing"
    );

    // AC3: `td-<slug>` is cleaned up.
    let branch_gone = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", td_branch),
        ])
        .status()
        .expect("show-ref td-<slug>");
    assert!(
        !branch_gone.success(),
        "'{}' must be deleted after landing",
        td_branch
    );

    // AC2: the trailer commit landed on `main`, not stranded on the
    // now-deleted `td-<slug>`.
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "exactly one Cb-CodeCheck trailer commit must land on main"
    );
    let log = Command::new(&git)
        .arg("-C")
        .arg(root)
        .args(["log", "main", "--format=%B"])
        .output()
        .expect("git log main");
    let log_text = String::from_utf8_lossy(&log.stdout);
    assert!(
        log_text.contains(&format!("Lifecycle-Slug: {}", slug)),
        "Lifecycle-Slug trailer must be on main:\n{}",
        log_text
    );

    // Idempotent retry: `td-<slug>` is already gone, so a second run must
    // be a clean landing no-op (`"status":"skipped"`), not a second merge
    // attempt or a duplicate trailer commit.
    let output2 = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check (idempotent retry)");
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        output2.status.success(),
        "idempotent retry should exit 0: {}",
        stdout2
    );
    assert!(
        stdout2.contains("\"status\":\"skipped\""),
        "idempotent retry must report landing status=skipped, got:\n{}",
        stdout2
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "idempotent retry must not add a duplicate Cb-CodeCheck trailer commit"
    );
}

// ---------------------------------------------------------------------------
// #847: the removed `aw td merge` "Bug 2" empty-implementation gate, wired
// back into the terminal `aw td code-check` fresh-entry path (before the
// phase-advancing `backend.update`, so a refusal leaves the issue untouched).
// ---------------------------------------------------------------------------

/// Seed a fresh git repo + empty `aw.toml`, matching the setup the
/// `#846` retry tests above use, minus the `td_merged` issue seed (fresh
/// #847 tests seed their own issue at a pre-terminal phase).
fn init_847_seed_repo(git: &std::path::Path, root: &std::path::Path) {
    use std::process::Command;

    Command::new(git)
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

/// Commit every current working-tree change (`git add -A && git commit`).
/// Real `aw td gen`/`aw td fill` already commit generated/filled
/// implementation files before terminal `aw td code-check` ever runs
/// (`commit_lifecycle` in td.rs, `stage_and_commit_cb_fill` in cb_fill.rs);
/// fixtures below that hand-write a WI's touched-scope file directly
/// (simulating a hand-written `impl_mode` completion with no gen/fill step)
/// must commit it the same way so they stay a realistic "ready for
/// code-check" precondition and don't trip the #807/#1275 clean-touched-
/// scope precondition for a reason unrelated to the gate each test targets.
fn commit_all(git: &std::path::Path, root: &std::path::Path) {
    use std::process::Command;

    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["add", "-A"])
        .status()
        .unwrap();
    Command::new(git)
        .arg("-C")
        .arg(root)
        .args(["commit", "-m", "wip: touched-scope fixture"])
        .status()
        .unwrap();
}

/// Repo-root-relative path `write_847_changes_spec` always writes to —
/// shared by `#847`/`#854` tests as the `Issue.implements` entry that scopes
/// both terminal gates to this WI's own spec (issue #854).
const DEMO_SPEC_REL: &str = ".aw/tech-design/specs/demo.md";

/// Write a minimal TD spec at `.aw/tech-design/specs/demo.md` (the default
/// `tech_design_path` fallback for an empty `aw.toml`) whose
/// `## Changes` section lists the given `(path, action)` entries, each
/// `impl_mode: hand-written` so `aw td gen` would have emitted nothing —
/// the exact "gen-code skipped" shape the gate detects.
fn write_847_changes_spec(root: &std::path::Path, entries: &[(&str, &str)]) {
    let mut yaml = String::from("changes:\n");
    for (path, action) in entries {
        yaml.push_str(&format!(
            "  - path: {path}\n    action: {action}\n    impl_mode: hand-written\n"
        ));
    }
    let content = format!(
        "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n\
         <!-- type: changes lang: yaml -->\n\n```yaml\n{yaml}```\n"
    );
    let spec_dir = root.join(".aw/tech-design/specs");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("demo.md"), content).unwrap();
}

/// Seed an open issue at `phase` with no `td-<slug>` branch — the shape of a
/// real `cb_genned`/`cb_filled` WI walking into `aw td code-check` for the
/// first time (fresh entry, not the #846 retry path). `spec_rel` is recorded
/// as `Issue.implements` (issue #854) so the terminal marker gate and
/// empty-implementation gate scope to this WI's own spec instead of the
/// whole worktree / whole `tech_design_path` tree.
async fn seed_847_open_issue(root: &std::path::Path, slug: &str, phase: &str, spec_rel: &str) {
    use agentic_workflow::issues::types::IssueType;
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", phase)],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(phase.to_string()),
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
    backend.create(&issue).await.expect("seed open issue");
}

/// AC1/AC3: a TD whose `## Changes` section lists N `create`/`modify` paths
/// with **all** N missing from disk (the 0-of-N "gen-code skipped" signature)
/// must refuse terminal code-check completion, naming the missing paths, and
/// must not advance phase / close the issue.
#[tokio::test]
async fn test_code_check_refuses_when_all_changes_paths_missing() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(
        root,
        &[("src/demo.rs", "create"), ("src/demo2.rs", "create")],
    );

    let slug = "empty-impl-gate-test";
    seed_847_open_issue(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "0-of-N missing paths must refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("refusing to complete code-check"),
        "error message must explain the empty-implementation refusal, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/demo.rs") && stdout.contains("src/demo2.rs"),
        "error message must name the missing paths, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_FILLED),
        "refused code-check must not advance phase past cb_filled"
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        0,
        "refused code-check must not land a Cb-CodeCheck trailer commit"
    );
}

/// AC1: `--allow-empty-impl` is the restored escape hatch — it skips the
/// refusal (with a warning) and lets the same 0-of-N spec complete.
#[tokio::test]
async fn test_code_check_allow_empty_impl_skips_refusal() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);

    let slug = "empty-impl-gate-override-test";
    seed_847_open_issue(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .arg("--allow-empty-impl")
        .current_dir(root)
        .output()
        .expect("run aw td code-check --allow-empty-impl");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "code-check --allow-empty-impl should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "--allow-empty-impl must let the 0-of-N spec complete, got:\n{}",
        stdout
    );
    assert!(
        stderr.contains("--allow-empty-impl"),
        "--allow-empty-impl must emit a warning line, got stderr:\n{}",
        stderr
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "--allow-empty-impl must still advance phase to td_merged"
    );
}

/// Partial presence (some but not all Changes paths exist) is warn-only and
/// must not block completion — only the 0-of-N signature blocks.
#[tokio::test]
async fn test_code_check_partial_implementation_completes() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(
        root,
        &[("src/demo.rs", "create"), ("src/demo2.rs", "create")],
    );
    // Partial presence: one of the two declared paths actually exists.
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/demo.rs"), "// implemented\n").unwrap();
    commit_all(&git, root);

    let slug = "empty-impl-gate-partial-test";
    seed_847_open_issue(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

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
        "partial-presence code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "partial presence must not block completion, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "partial presence must still advance phase to td_merged"
    );
}

// ---------------------------------------------------------------------------
// #854 — the terminal marker gate (and, by the same scoping mechanism, the
// #847 empty-implementation gate) must scope to the completing WI's own TD
// spec instead of the whole worktree / whole `tech_design_path` tree, so an
// unrelated inherited HANDWRITE marker elsewhere in a monorepo checkout can
// no longer block this WI's own code-check.
// ---------------------------------------------------------------------------

/// Write an unfilled HANDWRITE marker at `rel_path` (comment-style
/// begin/end, matching `crate::generate::apply::scaffold_handwrite_file`'s
/// output) if `filled` is `false`, or a filled block (real body content, no
/// `TODO: hand-write content` sentinel) if `filled` is `true`. Only unfilled
/// markers are returned by `enumerate_worktree_markers`
/// (`marker_body_is_unfilled` in `cb_fill.rs`).
fn write_854_marker_file(root: &std::path::Path, rel_path: &str, gap: &str, filled: bool) {
    let path = root.join(rel_path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let body = if filled {
        format!(
            "// HANDWRITE-BEGIN gap=\"{gap}\" tracker=\"none\" reason=\"filled\"\n\
             // implemented\n\
             // HANDWRITE-END\n"
        )
    } else {
        format!(
            "// HANDWRITE-BEGIN gap=\"{gap}\" tracker=\"none\" reason=\"unfilled\"\n\
             // TODO: hand-write content for `{rel_path}`.\n\
             // HANDWRITE-END\n"
        )
    };
    std::fs::write(path, body).unwrap();
}

/// (a) An unfilled HANDWRITE marker outside the WI's own Changes-listed
/// scope (`src/unrelated.rs`, e.g. inherited from other unmerged work on a
/// monorepo `main`) must not block completion when the WI's own
/// Changes-listed file (`src/demo.rs`) is present and filled.
#[tokio::test]
async fn test_code_check_ignores_unrelated_marker_outside_wi_scope() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);
    // WI's own Changes-listed file: present and filled.
    write_854_marker_file(root, "src/demo.rs", "demo-marker", true);
    // Unrelated stub marker elsewhere, outside the WI's Changes scope —
    // the exact repro from issue #854 (an inherited unfilled marker from
    // other unmerged work on the same checkout).
    write_854_marker_file(root, "src/unrelated.rs", "unrelated-marker", false);
    commit_all(&git, root);

    // Issue #859 part a2: seeded at cb_genned (not cb_filled) so this
    // fixture still exercises `run_cb_check_gate_scoped` at code-check's
    // fresh entry — a cb_filled entry is now trusted-skipped (fill's own
    // apply loop already proved this gate true before advancing phase).
    let slug = "marker-gate-scoped-test";
    seed_847_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL).await;

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
        "code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "an unrelated marker outside WI scope must not block completion, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "code-check must still advance phase to td_merged"
    );
}

/// (b) An unfilled HANDWRITE marker in a file the WI's own Changes section
/// names must still block completion, naming the file in the refusal.
#[tokio::test]
async fn test_code_check_blocks_on_marker_inside_wi_scope() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);
    // The WI's own Changes-listed file exists on disk (so the #847
    // empty-implementation gate does not fire) but still carries an
    // unfilled HANDWRITE marker. Committed so this test isolates the marker
    // gate from the #807/#1275 clean-touched-scope precondition, which
    // would otherwise refuse first (also naming `src/demo.rs`) for an
    // unrelated reason.
    write_854_marker_file(root, "src/demo.rs", "demo-marker", false);
    commit_all(&git, root);

    // Issue #859 part a2: seeded at cb_genned — see comment in
    // `test_code_check_ignores_unrelated_marker_outside_wi_scope` above.
    let slug = "marker-gate-blocks-test";
    seed_847_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL).await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "an unfilled marker inside WI scope must refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/demo.rs"),
        "error message must name the in-scope file carrying the unfilled marker, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_GENNED),
        "refused code-check must not advance phase past cb_genned"
    );
}

/// (c) A docs-only WI (empty `## Changes` section) with an empty branch
/// diff against base (HEAD already on `main`, matching every other fixture
/// in this file) must pass vacuously — an unrelated unfilled marker
/// elsewhere in the tree must not block a WI with nothing to scope against.
#[tokio::test]
async fn test_code_check_docs_only_wi_passes_vacuously() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    // Docs-only WI: no Changes entries at all.
    write_847_changes_spec(root, &[]);
    // Unrelated unfilled marker elsewhere in the tree.
    write_854_marker_file(root, "src/unrelated.rs", "unrelated-marker", false);

    // Issue #859 part a2: seeded at cb_genned so this exercises the scoped
    // gate's own vacuous-pass logic (empty scope union), not the separate
    // cb_filled trusted-skip path.
    let slug = "docs-only-wi-test";
    seed_847_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL).await;

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
        "docs-only code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "a docs-only WI with empty branch diff must pass vacuously, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "docs-only code-check must still advance phase to td_merged"
    );
}

// ---------------------------------------------------------------------------
// #859: terminal code-check efficiency + robustness.
// (a) the marker gate's underlying enumeration is scoped, not just its
//     post-walk filter; (b) the terminal fresh-entry write folds the
//     workflow-lock unlock into the same `IssuePatch` that advances phase and
//     closes the issue; (c) a missing local issue emits an actionable
//     rehydration envelope instead of misrouting into `td::run_audit`'s
//     "audit target not found".
// ---------------------------------------------------------------------------

/// (a) `enumerate_markers_for_scope` walks only the given scope paths — a
/// marker in a file outside the scope union is never read. Asserted via a
/// direct call to the enumerator's own return value: through the full CLI,
/// the old "walk everything then filter" and the new "walk only scope"
/// approaches are pass/fail-equivalent, so only a direct call can prove the
/// walk itself was bounded rather than just its filtered result.
#[test]
fn test_enumerate_markers_for_scope_excludes_out_of_scope_marker() {
    use agentic_workflow::cli::cb_fill::enumerate_markers_for_scope;

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_854_marker_file(root, "src/in_scope.rs", "in-scope-marker", false);
    write_854_marker_file(root, "src/out_of_scope.rs", "out-of-scope-marker", false);

    let scope = vec!["src/in_scope.rs".to_string()];
    let found = enumerate_markers_for_scope(root, &scope);

    assert_eq!(
        found.len(),
        1,
        "scoped enumeration must find exactly the in-scope marker, got: {:?}",
        found
    );
    assert_eq!(found[0].source_path, "src/in_scope.rs");
    assert!(
        !found.iter().any(|m| m.source_path == "src/out_of_scope.rs"),
        "scoped enumeration must never read a marker outside its scope paths, got: {:?}",
        found
    );
}

/// (a) An empty scope union (no branch diff, no WI Changes paths — the
/// vacuous-pass case `run_cb_check_gate_scoped` short-circuits on before
/// calling the enumerator at all) must not find any marker even when one
/// sits right at the worktree root, proving the enumerator itself performs
/// zero filesystem walk work for an empty scope rather than relying on a
/// post-walk filter to discard everything.
#[test]
fn test_enumerate_markers_for_scope_empty_scope_finds_nothing() {
    use agentic_workflow::cli::cb_fill::enumerate_markers_for_scope;

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_854_marker_file(root, "src/anything.rs", "any-marker", false);

    let found = enumerate_markers_for_scope(root, &[]);
    assert!(
        found.is_empty(),
        "an empty scope must enumerate zero markers, got: {:?}",
        found
    );
}

/// (b) Terminal completion of a fresh, lock-carrying WI must fold the
/// workflow-lock projection unlock into the same `IssuePatch` that advances
/// phase to `td_merged` and closes the issue, rather than a second
/// local-write + remote-push cycle after the fact. "One write cycle" isn't
/// directly observable from outside the process, so this asserts the
/// observable equivalent: the lock is fully released (label AND projection
/// body) after exactly ONE `aw td code-check` run against a fresh entry —
/// no retry needed, and (unlike the #846 retry fixture) never previously
/// closed.
#[tokio::test]
async fn test_code_check_folds_lock_release_into_single_write() {
    use agentic_workflow::cli::workflow_guard::{
        parse_projection, upsert_projection, WorkflowProjection,
    };
    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_847_changes_spec(root, &[("src/demo.rs", "create")]);
    write_854_marker_file(root, "src/demo.rs", "demo-marker", true);
    commit_all(&git, root);

    let slug = "fold-lock-release-test";
    let projection = WorkflowProjection {
        version: 1,
        issue_id: slug.to_string(),
        locked: true,
        owner: Some("td".to_string()),
        expected_command: Some("aw td code-check".to_string()),
        ..Default::default()
    };
    let body =
        upsert_projection(&format!("# {slug} WI\n"), &projection).expect("upsert projection");

    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![
            format!("phase:{}", td_phase::CB_GENNED),
            "score:locked".to_string(),
            "score:lock:td".to_string(),
        ],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body,
        related: Vec::new(),
        implements: vec![DEMO_SPEC_REL.to_string()],
        phase: Some(td_phase::CB_GENNED.to_string()),
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
    backend.create(&issue).await.expect("seed locked issue");

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
        "code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "a fresh, lock-carrying WI must complete in one run, got:\n{}",
        stdout
    );

    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(after.state, IssueState::Closed, "must be closed");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "must advance to td_merged"
    );
    assert!(
        !after
            .labels
            .iter()
            .any(|l| l == "score:locked" || l == "score:lock:td" || l == "score:lock:cb"),
        "all lock labels must be released by the single fresh-entry write, labels: {:?}",
        after.labels
    );
    let after_projection =
        parse_projection(&after.body).expect("projection block still present in body");
    assert!(
        !after_projection.locked,
        "the projection's own locked flag must be false after the fold, got: {:?}",
        after_projection
    );
    assert!(
        after_projection.owner.is_none(),
        "the projection's owner must be cleared after the fold, got: {:?}",
        after_projection
    );
}

/// (c) A slug with no local issue at all (never seeded, no lifecycle
/// history) must refuse with an explicit, actionable envelope naming the
/// issue and the right remediation — never the unrelated "audit target not
/// found" path-lookup misroute a missing *issue* used to fall through to.
#[tokio::test]
async fn test_code_check_missing_local_issue_emits_actionable_envelope() {
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
    init_847_seed_repo(&git, root);

    let slug = "never-seeded-wi";
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
        "missing-issue refusal still exits 0 (protocol is the stdout envelope):\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "a missing local issue must refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains(slug),
        "the error envelope must name the missing issue's slug, got:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("audit target not found"),
        "a missing issue must not misroute into the audit path-lookup message, got:\n{}",
        stdout
    );
}

/// Issue #939: prove `Issue.implements`, once populated by a REAL `aw td
/// create` call (not a hand-seeded test fixture), is actually consumed by
/// `aw td code-check`'s tier-1 `Issue.implements` scope resolution (#854,
/// `resolve_slug_spec_paths` in `cb.rs`). Uses a custom `--spec-path` that
/// differs from what tier-3's derived-default guess would produce for this
/// issue's labels (`.aw/tech-design/projects/score/logic/...`, per the
/// derivation `td_claim_test.rs` observes for a bare `project:
/// agentic-workflow`-labeled issue): if tier-1 were broken or ignored and the
/// resolver silently fell through to tier-3, code-check would find no
/// `## Changes` content at that (wrong) path and vacuously pass
/// (`"action":"done"`) instead of refusing over the missing file this test
/// declares at the tier-1 (`implements`) path.
#[tokio::test]
async fn test_code_check_consumes_implements_populated_by_real_td_create() {
    use agentic_workflow::issues::types::{td_phase, IssueType};
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};
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
    init_847_seed_repo(&git, root);

    let slug = "create-then-code-check-tier1-test";
    let custom_spec_rel = "custom/td-939-tier1-chain-test.md";

    // Seed a bare open issue the way `aw wi create` would leave one before
    // tech-design ever starts — no `implements` yet.
    let backend = LocalBackend::from_project_root(root);
    let seed_issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec!["app:agentic-workflow".to_string()],
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
    };
    // `write` (not `create`) — `create` force-downgrades a local-only,
    // github_id/gitlab_id-less issue to `draft` state, which would trip `aw
    // td create`'s own state:open guard for reasons unrelated to this test.
    backend
        .write(&seed_issue)
        .await
        .expect("seed bare open issue");

    // Real production code (issue #939's fix): `aw td create --spec-path`
    // must record `custom_spec_rel` in `Issue.implements`.
    let create_out = Command::new(&aw_bin)
        .arg("td")
        .arg("create")
        .arg(slug)
        .arg("--spec-path")
        .arg(custom_spec_rel)
        .current_dir(root)
        .output()
        .expect("run aw td create");
    assert!(
        create_out.status.success(),
        "aw td create should succeed:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&create_out.stdout),
        String::from_utf8_lossy(&create_out.stderr),
    );

    let created = backend
        .get(slug)
        .await
        .expect("read back issue after create")
        .expect("issue still present after create");
    assert!(
        created.implements.iter().any(|p| p == custom_spec_rel),
        "aw td create must have recorded {} in Issue.implements, got: {:?}",
        custom_spec_rel,
        created.implements
    );

    // Bypass the full gen/fill lifecycle (matching the #847/#932 fixture
    // convention elsewhere in this file): rewrite the issue as a fresh,
    // lock-free `cb_filled` entry that carries forward the
    // create-populated `implements`, the exact shape a real WI has by the
    // time it reaches terminal code-check.
    let mut chained = created;
    chained.labels = vec![format!("phase:{}", td_phase::CB_FILLED)];
    chained.phase = Some(td_phase::CB_FILLED.to_string());
    chained.body = format!("# {slug} WI\n");
    backend
        .write(&chained)
        .await
        .expect("rewrite issue as fresh cb_filled entry");

    // Write `## Changes` content at the tier-1 (implements) path only,
    // declaring one file that does not exist on disk — the "gen-code
    // skipped" refusal signature `write_847_changes_spec` also uses, but
    // written directly here since it targets a custom path.
    let spec_abs = root.join(custom_spec_rel);
    std::fs::create_dir_all(spec_abs.parent().unwrap()).unwrap();
    let spec_content = "---\nid: demo\nfill_sections: [changes]\n---\n\n# Demo\n\n## Changes\n\
         <!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: src/tier1_demo.rs\n    action: create\n    impl_mode: hand-written\n```\n";
    std::fs::write(&spec_abs, spec_content).unwrap();

    let check_out = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&check_out.stdout);
    assert!(
        check_out.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "tier-1 resolution must find the custom-path spec's missing Changes \
         entry and refuse; a tier-3 fallback would instead vacuously pass. \
         got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("refusing to complete code-check"),
        "error message must explain the empty-implementation refusal, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/tier1_demo.rs"),
        "error message must name the missing path declared at the tier-1 \
         (implements) spec, proving that path (not a tier-3 guess) was \
         resolved, got:\n{}",
        stdout
    );
}

// ---------------------------------------------------------------------------
// #932: code-check touched-scope standardization gate (Rule A — the forward
// (正流程) loop carries 標準化). For the WI's own touched-file set (branch
// diff ∪ Changes-listed paths, reusing #859's scoped enumeration via
// `cb_fill::resolve_touched_scope`): every in-scope touched file must carry
// a CODEGEN/HANDWRITE marker, and every touched HANDWRITE marker must have
// valid gap/tracker/reason attrs. Fail-mode only once the *rest* of the
// project (excluding this WI's own touched files) is already at 100%
// managed coverage; below that baseline the same violation is warn-only.
// Files outside the touched set never affect the verdict — no
// reintroduction of the #854 inherited-marker class.
// ---------------------------------------------------------------------------

/// Configure a minimal standardize-scoped project (`[[projects]]` +
/// `[[projects.workspaces]] paths = ["src/**"]`, no `path` — so
/// project-root-artifact scanning stays a no-op and the managed inventory
/// is exactly the files under `src/`) so
/// `standardize::project_touched_scope_standardization` has a scope to
/// walk. Overwrites the empty `aw.toml` `init_847_seed_repo` seeds;
/// standardize reads config straight off disk, so this does not need a
/// commit (and the terminal gate's `branch_changed_files` diffs commits,
/// not working-tree state, so it does not leak into any test's touched-file
/// set either).
fn write_932_project_config(root: &std::path::Path, project: &str) {
    let content = format!(
        "[[projects]]\nname = \"{project}\"\n\n[[projects.workspaces]]\npaths = [\"src/**\"]\n"
    );
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join("aw.toml"), content).unwrap();
}

/// A managed, CODEGEN-marked source file — counts toward
/// `StandardizationCoverage::managed_files`.
fn write_932_codegen_file(root: &std::path::Path, rel_path: &str) {
    let path = root.join(rel_path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        "// SPEC-MANAGED: .aw/tech-design/specs/demo.md#source\n// CODEGEN-BEGIN\npub fn demo() {}\n// CODEGEN-END\n",
    )
    .unwrap();
}

/// A plain source file with neither a CODEGEN nor a HANDWRITE marker — the
/// "unmarked" violation shape.
fn write_932_unmarked_file(root: &std::path::Path, rel_path: &str) {
    let path = root.join(rel_path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "pub fn unmarked() {}\n").unwrap();
}

/// A HANDWRITE-marked source file whose `tracker` attr is empty — managed
/// (`markers.handwrite = true`, so never in the `unmarked` list) but flagged
/// as an attr-gap violation (`detect_handwrite_gaps` / `is_missing_tracker`).
/// The body is plain filled content (not the `TODO: hand-write content for`
/// sentinel `marker_body_is_unfilled` looks for), so the pre-existing #859
/// marker gate treats this block as filled and does not itself block —
/// isolating the assertion to the new #932 gate.
fn write_932_handwrite_missing_tracker_file(root: &std::path::Path, rel_path: &str) {
    let path = root.join(rel_path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        "// HANDWRITE-BEGIN gap=\"demo-gap\" tracker=\"\" reason=\"needs manual work\"\npub fn demo() {}\n// HANDWRITE-END\n",
    )
    .unwrap();
}

/// Identical to `seed_847_open_issue` but also carries a `app:<name>`
/// label — the gate's activation key (`cb::project_label_for_wi`). Kept as
/// a separate helper rather than widening `seed_847_open_issue` itself:
/// every pre-#932 fixture in this file relies on that helper producing an
/// issue with **no** project label, which is exactly what makes the new
/// gate vacuously pass (no project configured to check against) for all of
/// them without any fixture changes.
async fn seed_932_open_issue(
    root: &std::path::Path,
    slug: &str,
    phase: &str,
    spec_rel: &str,
    project: &str,
) {
    use agentic_workflow::issues::types::IssueType;
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", phase), format!("app:{}", project)],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
        phase: Some(phase.to_string()),
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
    backend.create(&issue).await.expect("seed open issue");
}

/// (a) AC1: once the rest of the project (excluding this WI's touched
/// files) is already at 100% managed coverage, a touched in-scope file with
/// no CODEGEN/HANDWRITE marker at all must refuse completion, naming the
/// file and remediation, and must not advance phase.
#[tokio::test]
async fn test_code_check_blocks_touched_unmarked_file_post_bootstrap() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_932_project_config(root, "demo");
    // Rest of the managed inventory (excluding the touched file below) is
    // fully marked: baseline coverage excluding `src/touched.rs` is 100%.
    write_932_codegen_file(root, "src/baseline.rs");
    // The WI's own touched file: present on disk (so the #847
    // empty-implementation gate does not fire) but carries no marker at all.
    write_932_unmarked_file(root, "src/touched.rs");
    write_847_changes_spec(root, &[("src/touched.rs", "create")]);
    commit_all(&git, root);

    let slug = "touched-scope-unmarked-blocks-test";
    seed_932_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL, "demo").await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "a touched unmarked file in a post-bootstrap (100%-baseline) project must refuse with an \
         error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/touched.rs"),
        "error message must name the offending touched file, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("marker"),
        "error message must carry marker remediation, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_GENNED),
        "refused code-check must not advance phase past cb_genned"
    );
}

/// (b) AC2: below the 100% baseline (a project still mid-標準化
/// bootstrap — an unrelated, untouched file is also unmarked), the same
/// touched-unmarked-file condition must warn to stderr, naming the file,
/// without blocking completion.
#[tokio::test]
async fn test_code_check_warns_touched_unmarked_file_pre_bootstrap() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_932_project_config(root, "demo");
    write_932_codegen_file(root, "src/baseline.rs");
    // Unrelated, untouched, unmarked file — drags the baseline (excluding
    // the touched file below) under 100%, the pre-bootstrap shape.
    write_932_unmarked_file(root, "src/unrelated_untouched.rs");
    // The WI's own touched file: also unmarked.
    write_932_unmarked_file(root, "src/touched.rs");
    write_847_changes_spec(root, &[("src/touched.rs", "create")]);
    commit_all(&git, root);

    let slug = "touched-scope-unmarked-warns-test";
    seed_932_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL, "demo").await;

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
        "code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "below-baseline (pre-bootstrap) touched-scope violations must warn, not block, got:\n{}",
        stdout
    );
    assert!(
        stderr.contains("src/touched.rs"),
        "stderr must carry a warning naming the touched violation, got:\n{}",
        stderr
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "a warn-only touched-scope violation must still advance phase to td_merged"
    );
}

/// (c) Post-bootstrap block variant for the HANDWRITE-attr-gap shape: a
/// touched file that already carries a HANDWRITE marker (so it is
/// "managed" and never lands in the unmarked list) but whose `tracker` attr
/// is empty must still refuse completion once the rest of the project is at
/// 100% baseline.
#[tokio::test]
async fn test_code_check_blocks_touched_handwrite_missing_tracker_post_bootstrap() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_932_project_config(root, "demo");
    write_932_codegen_file(root, "src/baseline.rs");
    write_932_handwrite_missing_tracker_file(root, "src/touched.rs");
    write_847_changes_spec(root, &[("src/touched.rs", "modify")]);
    commit_all(&git, root);

    let slug = "touched-scope-attr-gap-blocks-test";
    seed_932_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL, "demo").await;

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "code-check refusal still exits 0 (protocol is the stdout envelope): {}",
        stdout
    );
    assert!(
        stdout.contains("\"action\":\"error\""),
        "a touched HANDWRITE marker missing its tracker attr in a post-bootstrap project must \
         refuse with an error envelope, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("src/touched.rs"),
        "error message must name the offending touched file, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("tracker"),
        "error message must call out the missing gap/tracker attrs, got:\n{}",
        stdout
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::CB_GENNED),
        "refused code-check must not advance phase past cb_genned"
    );
}

/// (d) AC3: an unmarked file OUTSIDE the WI's touched set must never affect
/// the verdict — a WI whose own touched file is fully, correctly marked
/// must complete cleanly even while an unrelated unmarked file exists
/// elsewhere in the same (would-be 100%-baseline) project, and the
/// unrelated file's path must never appear in the completion output. This
/// is the #932 counterpart to #854's inherited-marker fix: that issue
/// scoped the *marker* gate to the touched set; this asserts the new
/// *standardization* gate is scoped identically.
#[tokio::test]
async fn test_code_check_touched_scope_ignores_unrelated_unmarked_file() {
    use agentic_workflow::issues::types::td_phase;
    use agentic_workflow::issues::{IssueBackend, LocalBackend};
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
    init_847_seed_repo(&git, root);
    write_932_project_config(root, "demo");
    write_932_codegen_file(root, "src/baseline.rs");
    // Unrelated, untouched, unmarked file — must never affect this WI's
    // verdict regardless of how it drags the whole-project baseline down.
    write_932_unmarked_file(root, "src/unrelated_untouched.rs");
    // The WI's own touched file: properly CODEGEN-marked, no violation.
    write_932_codegen_file(root, "src/touched.rs");
    write_847_changes_spec(root, &[("src/touched.rs", "modify")]);
    commit_all(&git, root);

    let slug = "touched-scope-ignores-unrelated-test";
    seed_932_open_issue(root, slug, td_phase::CB_GENNED, DEMO_SPEC_REL, "demo").await;

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
        "code-check should exit 0:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("\"action\":\"done\""),
        "a WI whose own touched file is fully marked must complete cleanly regardless of \
         unrelated untouched debt, got:\n{}",
        stdout
    );
    assert!(
        !stderr.contains("src/unrelated_untouched.rs"),
        "an untouched file's marker status must never surface in this WI's output, got:\n{}",
        stderr
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        after.phase.as_deref(),
        Some(td_phase::TD_MERGED),
        "code-check must still advance phase to td_merged"
    );
}

// CODEGEN-END

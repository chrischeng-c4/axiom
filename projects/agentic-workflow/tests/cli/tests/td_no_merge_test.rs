// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md#source
// CODEGEN-BEGIN
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
    // `.aw/config.toml` is committed as part of the seed commit (as it
    // would be in a real project) so the working tree is clean going into
    // the landing step's dirty-tree guard below.
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
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

/// Seed a fresh git repo + empty `.aw/config.toml`, matching the setup the
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
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
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

/// Repo-root-relative path `write_847_changes_spec` always writes to —
/// shared by `#847`/`#854` tests as the `Issue.implements` entry that scopes
/// both terminal gates to this WI's own spec (issue #854).
const DEMO_SPEC_REL: &str = ".aw/tech-design/specs/demo.md";

/// Write a minimal TD spec at `.aw/tech-design/specs/demo.md` (the default
/// `tech_design_path` fallback for an empty `.aw/config.toml`) whose
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

    let slug = "marker-gate-scoped-test";
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
    // unfilled HANDWRITE marker.
    write_854_marker_file(root, "src/demo.rs", "demo-marker", false);

    let slug = "marker-gate-blocks-test";
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
        Some(td_phase::CB_FILLED),
        "refused code-check must not advance phase past cb_filled"
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

    let slug = "docs-only-wi-test";
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

// CODEGEN-END

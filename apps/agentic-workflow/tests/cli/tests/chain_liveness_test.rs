// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/chain_liveness_test.md#source
// CODEGEN-BEGIN
//! Tier 2 chain-liveness proof (#921, epic #914 slice G).
//!
//! Tier 1 (`chain.rs::validate_aw_command_string` / `normalize_legacy_next_action`,
//! plus its `emit_registry_entries_are_all_chain_valid` static coverage) proves
//! every emitted next-command string is *shaped* correctly. It does not prove
//! the lifecycle actually *terminates* — the review that opened epic #914
//! found three ways the canonical `aw wi` -> `aw td` (create -> gen -> fill ->
//! code-check) chain could get stuck instead:
//!
//!   - #842 the `td-<slug>` lifecycle branch never lands on the target branch
//!     at terminal code-check (already covered end-to-end by
//!     `td_no_merge_test.rs::test_code_check_lands_td_slug_branch_onto_main`
//!     — not duplicated here).
//!   - #843 `aw td claim --force-rebase` could set the legacy `td_reviewed`
//!     phase, which the linear lifecycle has no outgoing transition from —
//!     a permanent claim deadlock.
//!   - #846 a partial terminal failure (backend already updated, trailer
//!     commit never landed) was unretryable — re-running `aw td code-check`
//!     did not converge.
//!
//! Each test below drives the real `aw` binary against a from-scratch sandbox
//! repo (via `LocalBackend`, matching `td_no_merge_test.rs`'s convention) in a
//! bounded tick loop: livelock (the tick budget exhausting before the chain
//! reaches a terminal state) is a hard test failure, exactly like AC2's
//! "bounded tick count (livelock = failure)" contract.
//!
//! Scope note on `completion.workflow_complete` / `aw wi run`: the
//! `workflow_complete` field named in #921's AC2 lives on the `aw wi run` /
//! `aw capability run` root-driven-runner envelope (`cli/run.rs`), not on any
//! `aw td` verb's own envelope. `aw wi`'s public verbs resolve their backend
//! from `aw.toml`'s `[agentic_workflow.issue_platform]` /
//! `[agentic_workflow.repo_platform]`, and reject `local` there by design
//! (`issues::resolve_default_backend` — "Only `github` / `gitlab` are
//! accepted... the default targets a remote source of truth"), so a fully
//! offline sandbox cannot drive `aw wi run` to `workflow_complete=true`
//! without a live GitHub/GitLab fixture. `aw td claim` / `aw td code-check`
//! are the "internal lifecycle verbs [that] still use `LocalBackend` directly"
//! (same module doc) — the exact layer where #842/#843/#846 actually
//! manifested — so this test proves liveness there instead, which is where a
//! regression would actually reappear.

use std::path::Path;
use std::process::Command;

use agentic_workflow::issues::types::{td_phase, IssueType};
use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

/// AC2's "bounded tick count" — exhausting this many ticks without reaching
/// the expected terminal state is a livelock failure, not a slow pass.
const MAX_LIVENESS_TICKS: usize = 5;

fn skip_unless_binaries() -> Option<(std::path::PathBuf, String)> {
    let git = agentic_workflow::git::find_git_bin()?;
    let aw_bin = std::env::var("CARGO_BIN_EXE_aw").ok()?;
    Some((git, aw_bin))
}

fn init_seed_repo(git: &Path, root: &Path) {
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
    std::fs::create_dir_all(root.join(".aw/issues/open")).unwrap();
    std::fs::create_dir_all(root.join(".aw/tech-design")).unwrap();
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

/// Repo-root-relative path every seeded demo spec lives at (matches the
/// `td_no_merge_test.rs` convention: the default `tech_design_path` fallback
/// for an empty `aw.toml`).
const DEMO_SPEC_REL: &str = ".aw/tech-design/specs/demo.md";

/// Write a minimal TD spec whose `## Changes` section lists the given
/// `(path, action)` entries, each `impl_mode: hand-written`.
fn write_demo_changes_spec(root: &Path, entries: &[(&str, &str)]) {
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

/// Seed an open issue at `phase` with no `td-<slug>` branch — a fresh entry
/// into `aw td code-check`, not the #846 retry path.
async fn seed_open_issue_at_phase(root: &Path, slug: &str, phase: &str, spec_rel: &str) {
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

/// Seed a closed issue exactly where #846's partial terminal failure leaves
/// one: phase already advanced to `td_merged` (as `backend.update` in the
/// terminal path does before the trailer commit lands), `score:locked` still
/// present, no `Cb-CodeCheck` trailer commit in the log yet.
async fn seed_stranded_terminal_issue(root: &Path, slug: &str, spec_rel: &str) {
    let backend = LocalBackend::from_project_root(root);
    let issue = Issue {
        issue_type: IssueType::Enhancement,
        title: format!("{slug} WI"),
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
        body: format!("# {slug} WI\n"),
        related: Vec::new(),
        implements: vec![spec_rel.to_string()],
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
    backend.create(&issue).await.expect("seed stranded issue");
}

/// Count git log entries whose message has an exact-line `Lifecycle-Stage:
/// Cb-CodeCheck` trailer (mirrors `run_check_lifecycle_terminal`'s own
/// idempotency gate — a line-exact match, not a substring scan).
fn count_cb_code_check_trailer_commits(git: &Path, root: &Path) -> usize {
    let log = Command::new(git)
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

/// #843 regression, bounded-tick: `aw td claim` must converge on
/// `td_created` — never the legacy `td_reviewed` phase the linear lifecycle
/// has no outgoing transition from (a permanent claim deadlock) — within the
/// tick budget, and must stay there on repeated re-claims (`--force-rebase`),
/// not oscillate.
#[tokio::test]
async fn chain_liveness_claim_never_lands_on_deadlock_phase() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);

    // The spec source lives OUTSIDE the repo working tree entirely: `td
    // claim` activates a `td-<slug>` branch when launched from `main`, which
    // requires a clean working tree, and a spec file written inside `root`
    // (even outside `.aw/`) would show up as an untracked change and trip
    // that guard before claim ever runs (see td_claim_test.rs precedent).
    let spec_dir = tempfile::tempdir().expect("spec tempdir");
    let spec_path = spec_dir.path().join("external-spec.md");
    std::fs::write(
        &spec_path,
        "---\nslug: chain-liveness-claim-843\n---\n\n# External spec\n\nAdopted via --from-path.\n",
    )
    .unwrap();

    let slug = "chain-liveness-claim-843";
    let mut reached_created_at_tick: Option<usize> = None;

    for tick in 0..MAX_LIVENESS_TICKS {
        let mut cmd = Command::new(&aw_bin);
        cmd.arg("td").arg("claim").arg(slug);
        if tick == 0 {
            cmd.arg("--from-path").arg(&spec_path);
        } else {
            // Re-run every subsequent tick to prove repeated convergence,
            // not just a one-shot no-op.
            cmd.arg("--force-rebase");
        }
        let output = cmd
            .current_dir(root)
            .output()
            .unwrap_or_else(|e| panic!("tick {tick}: run aw td claim: {e}"));
        assert!(
            output.status.success(),
            "tick {tick}: aw td claim should exit 0:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let backend = LocalBackend::from_project_root(root);
        let issue = backend
            .get(slug)
            .await
            .expect("read back issue")
            .unwrap_or_else(|| panic!("tick {tick}: issue must exist after claim"));
        assert_ne!(
            issue.phase.as_deref(),
            Some(td_phase::TD_REVIEWED),
            "tick {tick}: claim must never land on the td_reviewed deadlock phase (#843)"
        );

        if issue.phase.as_deref() == Some(td_phase::TD_CREATED) {
            reached_created_at_tick = Some(tick);
            break;
        }
    }

    assert!(
        reached_created_at_tick.is_some(),
        "livelock: aw td claim did not converge on td_created within {MAX_LIVENESS_TICKS} ticks"
    );
}

/// #846 clean-path liveness: a `cb_filled` WI with its declared changes
/// already on disk must reach terminal `aw td code-check` completion in a
/// single tick (code-check *is* the terminal step of the linear lifecycle —
/// there is nothing further to dispatch to).
#[tokio::test]
async fn chain_liveness_code_check_terminates_within_tick_budget() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);
    write_demo_changes_spec(root, &[("src/demo.rs", "create")]);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/demo.rs"), "// implemented\n").unwrap();

    let slug = "chain-liveness-code-check-clean";
    seed_open_issue_at_phase(root, slug, td_phase::CB_FILLED, DEMO_SPEC_REL).await;

    let mut done_at_tick: Option<usize> = None;
    for tick in 0..MAX_LIVENESS_TICKS {
        let output = Command::new(&aw_bin)
            .arg("td")
            .arg("code-check")
            .arg(slug)
            .current_dir(root)
            .output()
            .unwrap_or_else(|e| panic!("tick {tick}: run aw td code-check: {e}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "tick {tick}: aw td code-check should exit 0:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if stdout.contains("\"action\":\"done\"") {
            done_at_tick = Some(tick);
            break;
        }
    }

    assert_eq!(
        done_at_tick,
        Some(0),
        "livelock: a clean cb_filled WI must reach terminal completion in a single tick"
    );

    let backend = LocalBackend::from_project_root(root);
    let after = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(after.phase.as_deref(), Some(td_phase::TD_MERGED));
}

/// #846 retry-path liveness: a WI stranded exactly where a partial terminal
/// failure leaves it (phase already `td_merged`, issue already closed,
/// `score:locked` still held, no `Cb-CodeCheck` trailer commit landed) must
/// converge within the tick budget, land exactly one trailer commit, and stay
/// idempotently done (no duplicate commit) across a second bounded-tick pass
/// — proving the retry doesn't just complete once but *stays* converged.
#[tokio::test]
async fn chain_liveness_code_check_retry_recovers_stranded_terminal_within_tick_budget() {
    let Some((git, aw_bin)) = skip_unless_binaries() else {
        eprintln!("skipping: git binary or CARGO_BIN_EXE_aw not available");
        return;
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    init_seed_repo(&git, root);
    write_demo_changes_spec(root, &[("src/demo.rs", "create")]);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/demo.rs"), "// implemented\n").unwrap();

    let slug = "chain-liveness-code-check-retry";
    seed_stranded_terminal_issue(root, slug, DEMO_SPEC_REL).await;
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        0,
        "sanity: no Cb-CodeCheck trailer commit exists before the retry"
    );

    let mut done_at_tick: Option<usize> = None;
    for tick in 0..MAX_LIVENESS_TICKS {
        let output = Command::new(&aw_bin)
            .arg("td")
            .arg("code-check")
            .arg(slug)
            .current_dir(root)
            .output()
            .unwrap_or_else(|e| panic!("tick {tick}: run aw td code-check: {e}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "tick {tick}: retry code-check should exit 0:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if stdout.contains("\"action\":\"done\"") {
            done_at_tick = Some(tick);
            break;
        }
    }
    assert!(
        done_at_tick.is_some(),
        "livelock: retry did not converge within {MAX_LIVENESS_TICKS} ticks (#846)"
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "retry must land exactly one Cb-CodeCheck trailer commit"
    );

    // A second bounded-tick pass must be an immediate, idempotent no-op —
    // not a re-livelock and not a duplicate trailer commit.
    let mut second_done_at_tick: Option<usize> = None;
    for tick in 0..MAX_LIVENESS_TICKS {
        let output = Command::new(&aw_bin)
            .arg("td")
            .arg("code-check")
            .arg(slug)
            .current_dir(root)
            .output()
            .unwrap_or_else(|e| panic!("second pass tick {tick}: run aw td code-check: {e}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "second pass tick {tick}: idempotent code-check should exit 0:\nstdout:\n{stdout}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if stdout.contains("\"action\":\"done\"") {
            second_done_at_tick = Some(tick);
            break;
        }
    }
    assert_eq!(
        second_done_at_tick,
        Some(0),
        "an already-completed WI must report done on the very first tick of a re-check"
    );
    assert_eq!(
        count_cb_code_check_trailer_commits(&git, root),
        1,
        "idempotent re-check must not land a second Cb-CodeCheck trailer commit"
    );
}
// CODEGEN-END

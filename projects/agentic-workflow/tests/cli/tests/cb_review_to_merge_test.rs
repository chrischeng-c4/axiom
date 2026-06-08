// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/cb_review_to_merge_test.md#source
// CODEGEN-BEGIN
//! Integration test stubs for the `aw td merge` accepted-phase guard.
//!
//! All tests are `#[ignore]`-flagged because each requires a tempdir +
//! git repo + worktree fixture + `score` CLI invocation harness that
//! `cb gen` cannot yet scaffold. The production-code fix (adding
//! `cb_reviewed` to the guard's accepted-phase set in
//! `projects/agentic-workflow/src/cli/td.rs`) lands in this same TD; the dogfood
//! verification is the live `td-enhancement-pkg-mgr-phase-1-4-lock-file-tracking`
//! worktree at phase=cb_reviewed unblocking after the new aw binary
//! is installed.
//!
//! Once the test-harness generator lands, drop the `#[ignore]` and
//! implement each test.

/// REQ: R1 — aw td merge accepts cb_reviewed phase and proceeds to td_merged
///
/// Provision a minimal worktree fixture with `state: open / phase: cb_reviewed`
/// in the issue frontmatter. Invoke `aw td merge <slug>`. Assert the
/// emitted envelope is `dispatch` (or `done`), NOT `error` with
/// "cannot merge: phase is 'cb_reviewed'".
#[test]
#[ignore = "blocked on test-harness generator (tempdir + worktree fixture + CLI invocation)"]
fn test_cb_reviewed_merge_succeeds() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

/// REQ: R2 — phase=cb_genned still passes the guard without regression
#[test]
#[ignore = "blocked on test-harness generator"]
fn test_cb_genned_still_accepted() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

/// REQ: R2 — phase=cb_filled still passes the guard without regression
#[test]
#[ignore = "blocked on test-harness generator"]
fn test_cb_filled_still_accepted() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

/// REQ: R2 — phase=td_reviewed (no-codegen path) still passes the guard
#[test]
#[ignore = "blocked on test-harness generator"]
fn test_td_reviewed_still_accepted() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

/// REQ: R2 — phase=td_merged (retry path) still passes the guard
#[test]
#[ignore = "blocked on test-harness generator"]
fn test_td_merged_still_accepted() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

/// REQ: R2 — unknown phase causes the guard to emit error envelope
///
/// Provision a fixture with `phase: some_unknown_phase`. Assert the
/// emitted envelope is `error` with message containing
/// "cannot merge: phase is 'some_unknown_phase'".
#[test]
#[ignore = "blocked on test-harness generator"]
fn test_unknown_phase_rejected() {
    todo!("requires worktree fixture + aw binary invocation harness");
}

// CODEGEN-END

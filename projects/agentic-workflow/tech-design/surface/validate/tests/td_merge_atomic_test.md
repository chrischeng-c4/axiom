---
id: projects-score-tests-td-merge-atomic-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/td_merge_atomic_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/td_merge_atomic_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/td_merge_atomic_test.rs -->
```rust
//! Integration test stubs for the atomic `aw td merge` transaction.
//!
//! All tests are `#[ignore]`-flagged because the atomic 5-step transaction
//! and idempotent re-run logic specified in
//! `projects/agentic-workflow/tech-design/surface/specs/score-td-merge-atomic.md` have
//! not yet been implemented in `projects/agentic-workflow/src/cli/td.rs::run_merge`.
//! `cb gen` emitted these test scaffolds, but did not emit the production-code
//! HANDWRITE markers for the atomic-merge handler itself.
//!
//! Once the verb lands, drop the `#[ignore]` and implement each test.

/// REQ: TP-five-step-transaction
///
/// Set up a bare git repo + td worktree + open issue, run `aw td merge`,
/// assert all five steps completed: branch on main has merge commit, issue
/// in `closed/`, frontmatter `phase: td_merged`, `Lifecycle-Stage: Td-Merged`
/// trailer in git log, worktree directory removed.
#[test]
#[ignore = "blocked on atomic 5-step `aw td merge` transaction implementation"]
fn test_five_step_transaction() {
    todo!("requires atomic td merge transaction in td.rs (not yet implemented)");
}

/// REQ: TP-rollback-on-close-fail
///
/// Inject failure at step 2 (file move) via a read-only `closed/` directory.
/// Assert git merge commit is reversed and issue stays in `open/`.
#[test]
#[ignore = "blocked on atomic td merge rollback semantics"]
fn test_rollback_on_close_fail() {
    todo!("requires step-2 failure injection + rollback (not yet implemented)");
}

/// REQ: TP-rollback-on-phase-fail
///
/// Inject failure at step 3 (frontmatter write) via a locked issue file.
/// Assert steps 1-2 are rolled back.
#[test]
#[ignore = "blocked on atomic td merge rollback semantics"]
fn test_rollback_on_phase_fail() {
    todo!("requires step-3 failure injection + rollback (not yet implemented)");
}

/// REQ: TP-rollback-on-commit-fail
///
/// Inject failure at step 4 (git commit) via a broken git index. Assert
/// steps 1-3 are rolled back.
#[test]
#[ignore = "blocked on atomic td merge rollback semantics"]
fn test_rollback_on_commit_fail() {
    todo!("requires step-4 failure injection + rollback (not yet implemented)");
}

/// REQ: TP-rollback-on-prune-fail
///
/// Inject failure at step 5 (worktree remove). Assert commit is reverted
/// and steps 1-3 are rolled back.
#[test]
#[ignore = "blocked on atomic td merge rollback semantics"]
fn test_rollback_on_prune_fail() {
    todo!("requires step-5 failure injection + rollback (not yet implemented)");
}

/// REQ: TP-idempotent-all-complete
///
/// Issue already in `closed/` with `phase: td_merged` and `Td-Merged`
/// trailer on main. Assert `done` envelope with idempotent note, no new
/// commit created.
#[test]
#[ignore = "blocked on idempotent re-run detection in td merge"]
fn test_idempotent_all_complete() {
    todo!("requires idempotent re-run logic (not yet implemented)");
}

/// REQ: TP-idempotent-issue-still-open
///
/// `Td-Merged` trailer on main, issue still open. Assert close + phase
/// advance + trailer commit (no duplicate merge commit).
#[test]
#[ignore = "blocked on idempotent re-run detection in td merge"]
fn test_idempotent_issue_still_open() {
    todo!("requires idempotent re-run logic (not yet implemented)");
}

/// REQ: TP-idempotent-phase-not-advanced
///
/// Issue closed, phase missing `td_merged`. Assert phase write + trailer
/// commit only.
#[test]
#[ignore = "blocked on idempotent re-run detection in td merge"]
fn test_idempotent_phase_not_advanced() {
    todo!("requires idempotent re-run logic (not yet implemented)");
}

/// REQ: TP-idempotent-trailer-missing
///
/// Issue closed, `phase: td_merged`, no trailer. Assert trailer commit only.
#[test]
#[ignore = "blocked on idempotent re-run detection in td merge"]
fn test_idempotent_trailer_missing() {
    todo!("requires idempotent re-run logic (not yet implemented)");
}

/// REQ: TP-partial-state-worktree-pruned
///
/// Worktree already removed, issue still open, `Td-Merged` on main.
/// Assert close + phase + trailer without worktree ops.
#[test]
#[ignore = "blocked on partial-state recovery logic in td merge"]
fn test_partial_state_worktree_pruned() {
    todo!("requires partial-state recovery (not yet implemented)");
}

/// REQ: TP-done-only-after-all
///
/// Assert `done` envelope is NOT emitted after step 4 if step 5 fails;
/// instead exit 1 with `error` envelope.
#[test]
#[ignore = "blocked on atomic td merge step-ordering guarantees"]
fn test_done_only_after_all() {
    todo!("requires step-5 failure path emitting error envelope (not yet implemented)");
}

/// REQ: TP-integration-idempotent-merge (R7)
///
/// Full integration scenario from R7: end-to-end re-run of `aw td merge`
/// against a fully-merged slug exits 0 with idempotent `done` envelope and
/// produces no new commits.
#[test]
#[ignore = "blocked on idempotent re-run detection in td merge"]
fn test_integration_idempotent_merge() {
    todo!("requires R7 idempotent integration scenario (not yet implemented)");
}

/// REQ: TP-td-merged-constant-value (R7 unit)
///
/// Unit test asserting `TD_MERGED` serializes to `"td_merged"` and that
/// the merge handler uses `TD_MERGED`, not `CB_GENNED`.
#[test]
#[ignore = "blocked on TD_MERGED constant + merge handler refactor"]
fn test_td_merged_constant_value() {
    todo!("requires TD_MERGED constant exported from phase enum (not yet implemented)");
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/td_merge_atomic_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

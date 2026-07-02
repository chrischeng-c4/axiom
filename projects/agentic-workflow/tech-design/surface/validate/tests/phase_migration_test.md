---
id: projects-score-tests-phase-migration-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs -->
```rust
//! Tests for the phase-enum reader compatibility (Phase 1 migration).
//!
//! - Reader accepts both `cb_genned` (canonical) and `td_gen_coded`
//!   (legacy alias).
//! - Writer always emits `cb_genned`.
//! - Trailer reader accepts both `Cb-Gen` and `Td-GenCode`; writer
//!   emits `Cb-Gen`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#test-plan

use agentic_workflow::issues::types::{lifecycle_trailer, td_phase};

#[test]
fn test_phase_reader_accepts_legacy() {
    // Legacy phase string normalises to canonical.
    assert_eq!(td_phase::normalize("td_gen_coded"), "cb_genned");
    // Canonical passes through unchanged.
    assert_eq!(td_phase::normalize("cb_genned"), "cb_genned");
}

// issue #850: retired CRRR phases predate the collapse to the linear
// lifecycle and have no outgoing transition of their own. A WI persisted
// at one of these phases must self-heal to a live phase on read, not pass
// through unmigrated (which used to leave `cb_reviewed` routed to a
// terminal code-check guard that rejected it — an unrecoverable
// dispatch-loop with no HITL escalation).
#[test]
fn test_retired_pre_gen_phase_normalizes_to_td_created() {
    // td_reviewed predates the CRRR collapse and is pre-gen, so it
    // migrates to td_created — consistent with the `aw td claim` write
    // path fixed in #843.
    assert_eq!(td_phase::normalize("td_reviewed"), td_phase::TD_CREATED);
}

#[test]
fn test_retired_post_fill_phases_normalize_to_cb_filled() {
    // cb_reviewed / cb_revised / cb_arbitrated are all post-fill CRRR
    // states, so they migrate to cb_filled — ready for terminal
    // `aw td code-check`, which now accepts them.
    for phase in ["cb_reviewed", "cb_revised", "cb_arbitrated"] {
        assert_eq!(
            td_phase::normalize(phase),
            td_phase::CB_FILLED,
            "phase: {phase}"
        );
        assert!(
            td_phase::is_terminal_code_checkable(td_phase::normalize(phase)),
            "phase: {phase}"
        );
    }
}

#[test]
fn test_phase_writer_emits_canonical() {
    // Source-text proof: `td.rs::run_gen_code` writes the canonical
    // phase string. We verify by source inspection because mutating an
    // issue file requires a worktree fixture that is heavier than this
    // pure-string test calls for.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/td.rs");
    let body = std::fs::read_to_string(&path).expect("read td.rs");
    assert!(
        body.contains(r#"phase: Some("cb_genned".to_string())"#),
        "td::run_gen_code must write the canonical 'cb_genned' phase"
    );
    assert!(
        !body.contains(r#"phase: Some("td_gen_coded".to_string())"#),
        "td::run_gen_code must not write the legacy 'td_gen_coded' phase"
    );
}

#[test]
fn test_trailer_reader_accepts_legacy() {
    assert_eq!(lifecycle_trailer::normalize("Td-GenCode"), "Cb-Gen");
    assert_eq!(lifecycle_trailer::normalize("Cb-Gen"), "Cb-Gen");
}

#[test]
fn test_trailer_writer_emits_canonical() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/td.rs");
    let body = std::fs::read_to_string(&path).expect("read td.rs");
    // Writer emits Cb-Gen (post Phase 1).
    assert!(
        body.contains(r#""Cb-Gen""#),
        "td::run_gen_code must commit canonical 'Cb-Gen' trailer"
    );
}

/// #850 AC1/AC2 end-to-end: a WI persisted at the retired `cb_reviewed`
/// phase used to be routed straight into `aw td code-check` by the
/// capability loop (`lifecycle_action_for_work_item`), which the terminal
/// code-check phase guard (`td_phase::is_terminal_code_checkable`) then
/// rejected outright — an unrecoverable dispatch-loop with no HITL
/// escalation. `LocalBackend::parse_issue_file` now normalizes `phase` on
/// every read, so `cb_reviewed` self-heals to `cb_filled` before the guard
/// ever sees it. This proves the specific phase-rejection error is gone;
/// it does not assert full terminal completion, since that also depends on
/// the HANDWRITE marker / empty-implementation gates, which are out of
/// scope for this issue.
#[tokio::test]
async fn test_code_check_accepts_retired_cb_reviewed_phase() {
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

    use agentic_workflow::issues::types::IssueType;
    use agentic_workflow::issues::{Issue, IssueBackend, IssueState, LocalBackend};

    let slug = "retired-cb-reviewed-phase-test";
    let backend = LocalBackend::from_project_root(root);
    let stranded = Issue {
        issue_type: IssueType::Enhancement,
        title: "WI stranded at retired cb_reviewed phase".to_string(),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![format!("phase:{}", "cb_reviewed")],
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        slug: slug.to_string(),
        body: "# WI stranded at retired cb_reviewed phase\n".to_string(),
        related: Vec::new(),
        implements: Vec::new(),
        phase: Some("cb_reviewed".to_string()),
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
        .expect("seed cb_reviewed issue");

    // Sanity: the on-disk read already self-heals the phase before any CLI
    // command runs, independent of the `code-check` gate chain below.
    let reread = backend
        .get(slug)
        .await
        .expect("read back issue")
        .expect("issue still present");
    assert_eq!(
        reread.phase.as_deref(),
        Some("cb_filled"),
        "LocalBackend read must normalize the retired cb_reviewed phase to cb_filled"
    );

    let output = Command::new(&aw_bin)
        .arg("td")
        .arg("code-check")
        .arg(slug)
        .arg("--allow-empty-impl")
        .current_dir(root)
        .output()
        .expect("run aw td code-check");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains("phase is 'cb_reviewed'"),
        "the terminal code-check guard must never see the un-normalized \
         cb_reviewed phase (the #850 dispatch-loop bug), got:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    assert!(
        !stdout.contains("cannot complete code-check: phase is"),
        "a normalized cb_reviewed WI must pass the terminal code-check \
         phase guard, got:\nstdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/phase_migration_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Existing source claimed by `aw standardize managed run`. The code is
      wrapped in a tracked HANDWRITE block until deterministic generator
      coverage can replace it with CODEGEN.
```

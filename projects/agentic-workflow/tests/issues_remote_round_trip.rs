// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/issues_remote_round_trip.md#tests
// CODEGEN-BEGIN

//! End-to-end smoke test for `GitHubBackend` round-tripping CRRR
//! state through the GitHub issue tracker via label-prefix encoding.
//!
//! Skips when `gh auth status` fails or when `SDD_TEST_GH_REPO`
//! (`owner/repo`) is unset — both gates make this safe on CI without
//! credentials.
//!
//! Manual run:
//!
//! ```sh
//! gh auth status   # must pass
//! export SDD_TEST_GH_REPO=owner/sandbox
//! cargo test -p agentic-workflow --test issues_remote_round_trip -- --ignored --nocapture
//! ```
//!
//! The test creates exactly one issue on the sandbox repo, mutates it
//! through the CRRR phases, and closes it. Closed issues are left in
//! place — they're cheap and they make the test trail auditable.

use agentic_workflow::issues::{
    GitHubBackend, Issue, IssueBackend, IssuePatch, IssueSection, IssueState, IssueType,
};

fn gh_authenticated() -> bool {
    std::process::Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn sandbox_repo() -> Option<String> {
    std::env::var("SDD_TEST_GH_REPO")
        .ok()
        .filter(|s| !s.is_empty())
}

fn fixture_issue(suffix: &str) -> Issue {
    Issue {
        issue_type: IssueType::Bug,
        title: format!("[sdd-test] round-trip CRRR labels — {}", suffix),
        state: IssueState::Open,
        id: None,
        github_id: None,
        gitlab_id: None,
        url: None,
        author: None,
        labels: vec![],
        created_at: None,
        updated_at: None,
        slug: format!("sdd-test-round-trip-{}", suffix),
        body: "Synthetic issue created by `tests/issues_remote_round_trip.rs`. Safe to delete."
            .to_string(),
        related: vec![],
        implements: vec![],
        phase: Some("td_inited".to_string()),
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
        validation_errors: vec![],
        review_count: Some(1),
        flagged_sections: Some(vec![IssueSection::Requirements]),
        fill_retry_count: Some(0),
        ship_status: None,
        ship_commit: None,
        regen_verified_at: None,
    }
}

#[tokio::test]
#[ignore = "requires gh auth + SDD_TEST_GH_REPO env"]
async fn github_backend_round_trips_crrr_state_via_labels() {
    if !gh_authenticated() {
        eprintln!("SKIP: `gh auth status` failed — run `gh auth login` first");
        return;
    }
    let Some(repo) = sandbox_repo() else {
        eprintln!("SKIP: SDD_TEST_GH_REPO not set");
        return;
    };

    let backend = GitHubBackend::new(Some(repo.clone()));
    let suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let staged = fixture_issue(&suffix);

    // 1. Create — should land with phase + review_count + flagged
    //    sections encoded as labels.
    let created = backend
        .create(&staged)
        .await
        .expect("create should succeed against the sandbox repo");
    let number = created
        .github_id
        .expect("created issue must carry github_id");
    let id = number.to_string();

    // Best-effort cleanup: if anything below panics, the issue stays
    // open on the sandbox so the failure is visible. We close on the
    // happy path at the end.

    // 2. Re-fetch via .get and assert CRRR labels round-trip back.
    let fetched = backend
        .get(&id)
        .await
        .expect("get should succeed")
        .expect("created issue must be retrievable by number");
    assert_eq!(
        fetched.phase.as_deref(),
        Some("td_inited"),
        "phase label must round-trip"
    );
    assert_eq!(
        fetched.review_count,
        Some(1),
        "review:1 label must round-trip"
    );
    assert_eq!(
        fetched.flagged_sections.as_deref(),
        Some(&[IssueSection::Requirements][..]),
        "flagged-section label must round-trip"
    );

    // 3. Patch the CRRR state and re-write through .update.
    let patch = IssuePatch {
        phase: Some("td_reviewed".to_string()),
        review_count: Some(2),
        flagged_sections: Some(vec![]),
        ..Default::default()
    };
    let updated = backend
        .update(&id, &patch)
        .await
        .expect("update should succeed");
    assert_eq!(
        updated.phase.as_deref(),
        Some("td_reviewed"),
        "patched phase must persist"
    );
    assert_eq!(
        updated.review_count,
        Some(2),
        "patched review_count must persist"
    );
    assert!(
        updated
            .flagged_sections
            .as_ref()
            .map(|v| v.is_empty())
            .unwrap_or(true),
        "flagged_sections cleared by patch must round-trip empty"
    );

    // 4. Close — leaves a paper trail on the sandbox repo.
    backend
        .close(&id, Some("sdd round-trip test complete"))
        .await
        .expect("close should succeed");
    let closed = backend
        .get(&id)
        .await
        .expect("get after close")
        .expect("issue should still be retrievable after close");
    assert_eq!(
        closed.state,
        IssueState::Closed,
        "issue must report closed state"
    );
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/issues_remote_round_trip_regen.md#tests
// CODEGEN-BEGIN

// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END

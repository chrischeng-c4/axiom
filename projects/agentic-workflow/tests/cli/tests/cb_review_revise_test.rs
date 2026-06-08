// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/cb_review_revise_test.md#source
// CODEGEN-BEGIN

//! Integration tests for `aw cb review` / `aw cb revise`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#test-plan
//!
//! These tests are placeholders — the fixture harness needed to spin up a
//! synthetic worktree with a primed issue + cb_review.md / cb_revise.md
//! payloads is not yet extracted from the existing td-side test setup.
//! Tracking issue: enhancement-cli-test-fixture-generator-worktree-mock.

#[test]
fn smoke_phase_constants_compile() {
    assert_eq!(
        agentic_workflow::issues::types::td_phase::CB_REVIEWED,
        "cb_reviewed"
    );
    assert_eq!(
        agentic_workflow::issues::types::td_phase::CB_REVISED,
        "cb_revised"
    );
}

#[test]
fn smoke_lifecycle_trailers_compile() {
    assert_eq!(
        agentic_workflow::issues::types::lifecycle_trailer::CB_REVIEW,
        "Cb-Review"
    );
    assert_eq!(
        agentic_workflow::issues::types::lifecycle_trailer::CB_REVISE,
        "Cb-Revise"
    );
    assert_eq!(
        agentic_workflow::issues::types::lifecycle_trailer::CB_ARBITRATE,
        "Cb-Arbitrate"
    );
}

#[test]
fn smoke_is_mergeable_includes_new_phases() {
    use agentic_workflow::issues::types::td_phase::{is_mergeable, CB_REVIEWED, CB_REVISED};
    assert!(is_mergeable(CB_REVIEWED));
    assert!(is_mergeable(CB_REVISED));
}

// CODEGEN-END

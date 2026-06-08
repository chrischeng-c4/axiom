---
id: projects-score-tests-cb-review-revise-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized projects/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs -->
```rust

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
    assert_eq!(agentic_workflow::issues::types::td_phase::CB_REVIEWED, "cb_reviewed");
    assert_eq!(agentic_workflow::issues::types::td_phase::CB_REVISED, "cb_revised");
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/cb_review_revise_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```

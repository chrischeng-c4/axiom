//! Shared phase transition functions.
//!
//! Extracted from `state_update.rs` so that both `state_update` (legacy)
//! and `run_change` can reuse the same parse/validate logic.

use crate::models::state::StatePhase;
use crate::Result;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/phase_transition/source.md#source
// CODEGEN-BEGIN
// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#R1
/// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#logic
/// Parse phase string to StatePhase
pub fn parse_phase(s: &str) -> Result<StatePhase> {
    match s {
        // Primary strings
        "change_inited" => Ok(StatePhase::ChangeInited),
        // Backward compat: removed phases all map to PostClarificationsCreated
        "input_restructured" => Ok(StatePhase::ChangeInited),
        "pre_clarifications_created" => Ok(StatePhase::ChangeInited),
        "reference_context_created" => Ok(StatePhase::ChangeInited),
        "reference_context_reviewed" => Ok(StatePhase::ChangeInited),
        "reference_context_revised" => Ok(StatePhase::ChangeInited),
        "post_clarifications_created" => Ok(StatePhase::ChangeInited),
        "change_spec_created" => Ok(StatePhase::ChangeSpecCreated),
        "change_spec_reviewed" => Ok(StatePhase::ChangeSpecReviewed),
        "change_spec_revised" => Ok(StatePhase::ChangeSpecRevised),
        "change_implementation_created" => Ok(StatePhase::ChangeImplementationCreated),
        "change_implementation_reviewed" => Ok(StatePhase::ChangeImplementationReviewed),
        "change_implementation_revised" => Ok(StatePhase::ChangeImplementationRevised),
        "test_check" => Ok(StatePhase::TestCheck),
        "docs_check" => Ok(StatePhase::DocsCheck),
        "docs_created" => Ok(StatePhase::DocsCreated),
        "docs_reviewed" => Ok(StatePhase::DocsReviewed),
        "docs_revised" => Ok(StatePhase::DocsRevised),
        "change_merge_created" => Ok(StatePhase::ChangeMergeCreated),
        "change_merge_reviewed" => Ok(StatePhase::ChangeMergeReviewed),
        "change_merge_revised" => Ok(StatePhase::ChangeMergeRevised),
        "change_archived" => Ok(StatePhase::ChangeArchived),
        "change_rejected" => Ok(StatePhase::ChangeRejected),
        // Backward compat: removed variants
        "pre_clarifications_reviewed"
        | "pre_clarifications_revised"
        | "pre_clarifications_approved" => Ok(StatePhase::ChangeInited),
        "reference_context_approved" => Ok(StatePhase::ChangeInited),
        "post_clarifications_reviewed"
        | "post_clarifications_revised"
        | "post_clarifications_approved" => Ok(StatePhase::ChangeInited),
        "change_spec_approved" => Ok(StatePhase::ChangeImplementationCreated),
        "change_implementation_approved" => Ok(StatePhase::ChangeMergeCreated),
        "change_merge_approved" => Ok(StatePhase::ChangeArchived),
        "archived" => Ok(StatePhase::ChangeArchived),
        "rejected" => Ok(StatePhase::ChangeRejected),
        // Legacy aliases
        "implementing" | "testing" | "implemented" => Ok(StatePhase::ChangeImplementationCreated),
        "impl_reviewed" | "code_reviewing" => Ok(StatePhase::ChangeImplementationReviewed),
        "impl_revised" => Ok(StatePhase::ChangeImplementationRevised),
        "impl_approved" => Ok(StatePhase::ChangeMergeCreated),
        "merging" => Ok(StatePhase::ChangeMergeCreated),
        // CRRR-terminal phase on issue frontmatter (written by `aw wi merge` with
        // Lifecycle-Stage: Merge). When `score workflow init-change` reads this, the change
        // should start fresh at the change-lifecycle-initial phase, NOT jump to ChangeMergeCreated.
        // See projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md (CRRR-terminal
        // -> change-lifecycle-initial handoff).
        "merged" => Ok(StatePhase::ChangeInited),
        "merge_reviewed" => Ok(StatePhase::ChangeMergeReviewed),
        "merge_revised" => Ok(StatePhase::ChangeMergeRevised),
        "merge_approved" => Ok(StatePhase::ChangeArchived),
        // v2 legacy aliases
        "clarified" => Ok(StatePhase::ChangeInited),
        "clarifications_reviewed" | "clarifications_revised" | "clarifications_approved" => {
            Ok(StatePhase::ChangeInited)
        }
        "clarifications_rejected"
        | "reference_context_rejected"
        | "post_clarifications_rejected"
        | "spec_rejected" => Ok(StatePhase::ChangeRejected),
        "decided" => Ok(StatePhase::ChangeInited),
        "spec_created" => Ok(StatePhase::ChangeSpecCreated),
        "spec_reviewed" => Ok(StatePhase::ChangeSpecReviewed),
        "spec_revised" => Ok(StatePhase::ChangeSpecRevised),
        "spec_approved" | "all_specs_approved" | "tasks_generated" | "planned" => {
            Ok(StatePhase::ChangeImplementationCreated)
        }
        // Older legacy phases - all absorbed into PostClarificationsCreated
        "explored" | "needs_followup" | "needs_second_clarification" => {
            Ok(StatePhase::ChangeInited)
        }
        "spec_context_created"
        | "spec_context_approved"
        | "knowledge_context_created"
        | "knowledge_context_approved"
        | "codebase_context_created"
        | "codebase_context_approved" => Ok(StatePhase::ChangeInited),
        "gap_codebase_spec_created"
        | "gap_codebase_spec_approved"
        | "gap_codebase_knowledge_created"
        | "gap_codebase_knowledge_approved"
        | "gap_spec_knowledge_created"
        | "gap_spec_knowledge_approved" => Ok(StatePhase::ChangeInited),
        "proposal_created" | "proposal_approved" | "proposal_reviewed" | "proposal_revised"
        | "proposal_rejected" => Ok(StatePhase::ChangeInited),
        _ => anyhow::bail!("Invalid phase: {}", s),
    }
}

/// Convert StatePhase to string
/// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#logic
pub fn phase_to_string(phase: &StatePhase) -> &'static str {
    match phase {
        StatePhase::ChangeInited => "change_inited",
        StatePhase::ChangeSpecCreated => "change_spec_created",
        StatePhase::ChangeSpecReviewed => "change_spec_reviewed",
        StatePhase::ChangeSpecRevised => "change_spec_revised",
        StatePhase::ChangeImplementationCreated => "change_implementation_created",
        StatePhase::ChangeImplementationReviewed => "change_implementation_reviewed",
        StatePhase::ChangeImplementationRevised => "change_implementation_revised",
        StatePhase::TestCheck => "test_check",
        StatePhase::DocsCheck => "docs_check",
        StatePhase::DocsCreated => "docs_created",
        StatePhase::DocsReviewed => "docs_reviewed",
        StatePhase::DocsRevised => "docs_revised",
        StatePhase::ChangeMergeCreated => "change_merge_created",
        StatePhase::ChangeMergeReviewed => "change_merge_reviewed",
        StatePhase::ChangeMergeRevised => "change_merge_revised",
        StatePhase::ChangeArchived => "change_archived",
        StatePhase::ChangeRejected => "change_rejected",
    }
}

/// Get phase order for transition validation
/// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#logic
pub fn phase_order(phase: &StatePhase) -> u8 {
    match phase {
        // Decide workflow: 0-4
        StatePhase::ChangeInited => 0,
        // Spec workflow: 10-11
        StatePhase::ChangeSpecCreated => 10,
        StatePhase::ChangeSpecReviewed => 11,
        StatePhase::ChangeSpecRevised => 11,
        // Implementation workflow: 16-17
        StatePhase::ChangeImplementationCreated => 16,
        StatePhase::ChangeImplementationReviewed => 17,
        StatePhase::ChangeImplementationRevised => 17,
        // Test gate: 17 (transient, same level as impl reviewed)
        StatePhase::TestCheck => 17,
        // Docs workflow: 18
        StatePhase::DocsCheck => 18,
        StatePhase::DocsCreated => 18,
        StatePhase::DocsReviewed => 18,
        StatePhase::DocsRevised => 18,
        // Merge workflow: 19-20
        StatePhase::ChangeMergeCreated => 19,
        StatePhase::ChangeMergeReviewed => 20,
        StatePhase::ChangeMergeRevised => 20,
        StatePhase::ChangeArchived => 22,
        // Error state
        StatePhase::ChangeRejected => 99,
    }
}

/// Validate that the transition is allowed
/// @spec projects/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md#logic
pub fn validate_transition(from: &StatePhase, to: &StatePhase) -> Result<()> {
    if from == to {
        return Ok(());
    }

    if matches!(to, StatePhase::ChangeRejected) {
        return Ok(());
    }

    if matches!(
        from,
        StatePhase::ChangeArchived | StatePhase::ChangeRejected
    ) {
        anyhow::bail!(
            "Cannot transition from '{}' - change is in terminal state",
            phase_to_string(from)
        );
    }

    let from_order = phase_order(from);
    let to_order = phase_order(to);

    let allowed = match (from, to) {
        // ChangeInited -> PostClarificationsCreated (issue lifecycle absorbs pre-clarifications + reference context)
        (StatePhase::ChangeInited, StatePhase::ChangeInited) => true,
        // Post-Clarifications -> Spec (skip approved)
        (StatePhase::ChangeInited, StatePhase::ChangeSpecCreated) => true,

        // Spec workflow
        (StatePhase::ChangeSpecCreated, StatePhase::ChangeSpecReviewed) => true,
        (StatePhase::ChangeSpecReviewed, StatePhase::ChangeSpecRevised) => true,
        (StatePhase::ChangeSpecReviewed, StatePhase::ChangeSpecCreated) => true,
        (StatePhase::ChangeSpecRevised, StatePhase::ChangeSpecReviewed) => true,
        // Spec -> Implementation (APPROVED verdict -> skip approved)
        (StatePhase::ChangeSpecReviewed, StatePhase::ChangeImplementationCreated) => true,
        (StatePhase::ChangeSpecRevised, StatePhase::ChangeImplementationCreated) => true,
        (StatePhase::ChangeInited, StatePhase::ChangeImplementationCreated) => true,

        // Implementation workflow
        (StatePhase::ChangeImplementationCreated, StatePhase::ChangeImplementationReviewed) => true,
        (StatePhase::ChangeImplementationReviewed, StatePhase::ChangeImplementationRevised) => true,
        (StatePhase::ChangeImplementationRevised, StatePhase::ChangeImplementationReviewed) => true,
        // Implementation -> TestCheck or Docs or Merge (APPROVED verdict)
        (StatePhase::ChangeImplementationReviewed, StatePhase::TestCheck) => true,
        (StatePhase::ChangeImplementationReviewed, StatePhase::DocsCheck) => true,
        (StatePhase::ChangeImplementationReviewed, StatePhase::DocsCreated) => true,
        (StatePhase::ChangeImplementationReviewed, StatePhase::ChangeMergeCreated) => true,
        (StatePhase::ChangeImplementationRevised, StatePhase::TestCheck) => true,
        (StatePhase::ChangeImplementationRevised, StatePhase::ChangeMergeCreated) => true,
        (StatePhase::ChangeImplementationRevised, StatePhase::DocsCheck) => true,
        (StatePhase::ChangeImplementationRevised, StatePhase::DocsCreated) => true,

        // TestCheck -> DocsCheck/Merge (pass/skip) or back to Implementation (fail)
        (StatePhase::TestCheck, StatePhase::DocsCheck) => true,
        (StatePhase::TestCheck, StatePhase::DocsCreated) => true,
        (StatePhase::TestCheck, StatePhase::ChangeMergeCreated) => true,
        (StatePhase::TestCheck, StatePhase::ChangeImplementationCreated) => true,

        // Docs workflow (CRR cycle)
        (StatePhase::DocsCheck, StatePhase::DocsCreated) => true,
        (StatePhase::DocsCheck, StatePhase::ChangeMergeCreated) => true,
        (StatePhase::DocsCreated, StatePhase::DocsReviewed) => true,
        (StatePhase::DocsReviewed, StatePhase::DocsRevised) => true,
        (StatePhase::DocsReviewed, StatePhase::ChangeMergeCreated) => true,
        (StatePhase::DocsRevised, StatePhase::DocsReviewed) => true,
        (StatePhase::DocsRevised, StatePhase::ChangeMergeCreated) => true,

        // Merge workflow
        (StatePhase::ChangeMergeCreated, StatePhase::ChangeMergeReviewed) => true,
        (StatePhase::ChangeMergeReviewed, StatePhase::ChangeMergeRevised) => true,
        (StatePhase::ChangeMergeRevised, StatePhase::ChangeMergeReviewed) => true,
        // Merge -> Archived (APPROVED verdict -> skip approved)
        (StatePhase::ChangeMergeReviewed, StatePhase::ChangeArchived) => true,
        (StatePhase::ChangeMergeRevised, StatePhase::ChangeArchived) => true,
        (StatePhase::ChangeMergeCreated, StatePhase::ChangeArchived) => true,

        // Skip transitions (forward jumps within 3 steps)
        _ if to_order > from_order && to_order - from_order <= 3 => true,

        _ => false,
    };

    if !allowed {
        anyhow::bail!(
            "Invalid state transition: '{}' -> '{}'. \
            Transitions must follow the workflow order.",
            phase_to_string(from),
            phase_to_string(to)
        );
    }

    Ok(())
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;

    // REQ: REQ-R1, REQ-R3, REQ-R5
    /// T1: "merged" is the CRRR-terminal phase written by `aw wi merge`
    /// (Lifecycle-Stage: Merge, state: open, phase: merged). When `score workflow
    /// init-change` reads this frontmatter it must start the change lifecycle fresh
    /// at ChangeInited — NOT jump to ChangeMergeCreated. This test guards the
    /// CRRR-terminal → change-lifecycle-initial handoff translation in parse_phase.
    #[test]
    fn parse_phase_merged_returns_change_inited() {
        assert_eq!(
            parse_phase("merged").unwrap(),
            StatePhase::ChangeInited,
            "\"merged\" is the CRRR-terminal alias and must map to ChangeInited, \
             not ChangeMergeCreated"
        );
    }

    // REQ: REQ-R2
    /// T2: "merging" is a legacy change-lifecycle alias for an in-flight merge.
    /// It must continue to map to ChangeMergeCreated after the arm split.
    #[test]
    fn parse_phase_merging_still_maps_to_change_merge_created() {
        assert_eq!(
            parse_phase("merging").unwrap(),
            StatePhase::ChangeMergeCreated,
            "\"merging\" is a legacy change-lifecycle alias and must still map to ChangeMergeCreated"
        );
    }

    // REQ: REQ-R1, REQ-R2, REQ-R5
    /// T3: Spot-check other aliases to guard against broad regressions.
    #[test]
    fn parse_phase_other_aliases_unchanged() {
        assert_eq!(
            parse_phase("spec_created").unwrap(),
            StatePhase::ChangeSpecCreated,
            "\"spec_created\" alias must still map to ChangeSpecCreated"
        );
        assert_eq!(
            parse_phase("archived").unwrap(),
            StatePhase::ChangeArchived,
            "\"archived\" alias must still map to ChangeArchived"
        );
        assert_eq!(
            parse_phase("change_inited").unwrap(),
            StatePhase::ChangeInited,
            "\"change_inited\" primary string must still map to ChangeInited"
        );
    }
}

// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Replay-failed-step control affordance for open mode (#2886).
//!
//! When a reviewer opens a finished case in `jet e2e open`, they
//! often want to re-run *just* the failed step — same inputs, same
//! browser state up to the boundary — instead of replaying the whole
//! case. This module owns the affordance state: when replay is
//! available, when it isn't, and a structured reason the UI renders
//! next to the disabled button.
//!
//! Full time-travel debugging (arbitrary step replay) is out of
//! scope.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`ReplayStepAffordance`].
pub const REPLAY_STEP_SCHEMA_VERSION: &str = "jet.e2e.replay-step.v1";

/// Why replay isn't available for the selected step. The UI surfaces
/// the variant to render an explanation panel; the runner uses the
/// same reasons in evidence so a later reviewer can audit what the
/// UI showed.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum ReplayUnavailableReason {
    /// The selected step did not fail.
    StepDidNotFail,
    /// The runner did not capture enough context (no recorded
    /// inputs / no DOM snapshot) to recreate the step.
    NoRecordedContext { detail: String },
    /// The runner reported it cannot resume from this case (e.g.
    /// closed session, ephemeral fixture).
    SessionClosed,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ReplayUnavailableReason {
    /// Short, reviewer-facing label suitable for a one-line UI hint.
    pub fn ui_label(&self) -> String {
        match self {
            Self::StepDidNotFail => "Replay only available for failed steps".into(),
            Self::NoRecordedContext { detail } => {
                format!("Cannot replay: {detail}")
            }
            Self::SessionClosed => "Cannot replay: browser session closed".into(),
        }
    }
}

/// Snapshot of what the runner recorded about the step we might
/// replay. The control surface reads this to decide availability.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayContext {
    pub case_id: String,
    pub step_id: String,
    pub step_failed: bool,
    pub session_open: bool,
    /// True when the runner stored enough context (inputs + DOM /
    /// trace) to re-execute the step.
    pub has_recorded_context: bool,
    /// Optional detail to feed
    /// [`ReplayUnavailableReason::NoRecordedContext`] when the
    /// recorded context is missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing_context_detail: Option<String>,
}

/// What the UI should render for the replay control.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReplayStepAffordance {
    /// Reviewer can trigger replay — UI shows an enabled button.
    Available {
        schema_version: String,
        context: ReplayContext,
    },
    /// Reviewer cannot trigger replay — UI shows a disabled control
    /// and explains why.
    Unavailable {
        schema_version: String,
        context: ReplayContext,
        reason: ReplayUnavailableReason,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ReplayStepAffordance {
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub fn unavailable_reason(&self) -> Option<&ReplayUnavailableReason> {
        match self {
            Self::Unavailable { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

/// Decide which affordance the UI should render given the recorded
/// context.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn affordance_for(context: ReplayContext) -> ReplayStepAffordance {
    if !context.step_failed {
        return ReplayStepAffordance::Unavailable {
            schema_version: REPLAY_STEP_SCHEMA_VERSION.to_string(),
            context,
            reason: ReplayUnavailableReason::StepDidNotFail,
        };
    }
    if !context.session_open {
        return ReplayStepAffordance::Unavailable {
            schema_version: REPLAY_STEP_SCHEMA_VERSION.to_string(),
            context,
            reason: ReplayUnavailableReason::SessionClosed,
        };
    }
    if !context.has_recorded_context {
        let detail = context
            .missing_context_detail
            .clone()
            .unwrap_or_else(|| "no recorded context for step".into());
        return ReplayStepAffordance::Unavailable {
            schema_version: REPLAY_STEP_SCHEMA_VERSION.to_string(),
            context,
            reason: ReplayUnavailableReason::NoRecordedContext { detail },
        };
    }
    ReplayStepAffordance::Available {
        schema_version: REPLAY_STEP_SCHEMA_VERSION.to_string(),
        context,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(step_failed: bool, session_open: bool, has_ctx: bool) -> ReplayContext {
        ReplayContext {
            case_id: "case-buy".into(),
            step_id: "submit".into(),
            step_failed,
            session_open,
            has_recorded_context: has_ctx,
            missing_context_detail: None,
        }
    }

    #[test]
    fn failed_step_with_full_context_exposes_replay() {
        // Stop condition (#2886): a failed step exposes replay when
        // the runner provides enough context.
        let aff = affordance_for(ctx(true, true, true));
        assert!(aff.is_available());
    }

    #[test]
    fn passed_step_renders_unavailable_explaining_pass() {
        // Stop condition (#2886): unavailable replay state explains
        // why it cannot run.
        let aff = affordance_for(ctx(false, true, true));
        assert!(!aff.is_available());
        assert_eq!(
            aff.unavailable_reason().unwrap(),
            &ReplayUnavailableReason::StepDidNotFail
        );
        assert!(
            aff.unavailable_reason()
                .unwrap()
                .ui_label()
                .contains("failed steps"),
            "{:?}",
            aff
        );
    }

    #[test]
    fn closed_session_renders_unavailable_explaining_session() {
        let aff = affordance_for(ctx(true, false, true));
        assert!(!aff.is_available());
        assert_eq!(
            aff.unavailable_reason().unwrap(),
            &ReplayUnavailableReason::SessionClosed
        );
    }

    #[test]
    fn missing_context_renders_unavailable_with_detail() {
        let mut c = ctx(true, true, false);
        c.missing_context_detail = Some("missing input transcript".into());
        let aff = affordance_for(c);
        match aff.unavailable_reason().unwrap() {
            ReplayUnavailableReason::NoRecordedContext { detail } => {
                assert!(detail.contains("input transcript"), "{detail}");
            }
            other => panic!("expected NoRecordedContext, got {other:?}"),
        }
    }

    #[test]
    fn ui_can_trigger_or_explain_replay_for_one_fixture() {
        // Stop condition (#2886): UI can trigger or explain
        // failed-step replay for one fixture — Available vs
        // Unavailable cover the two paths.
        let triggerable = affordance_for(ctx(true, true, true));
        let explainable = affordance_for(ctx(true, true, false));
        assert!(triggerable.is_available());
        assert!(!explainable.is_available());
        assert!(explainable.unavailable_reason().is_some());
    }

    #[test]
    fn affordance_round_trips_through_json() {
        let aff = affordance_for(ctx(false, true, true));
        let json = serde_json::to_string(&aff).unwrap();
        let back: ReplayStepAffordance = serde_json::from_str(&json).unwrap();
        assert_eq!(back, aff);
        assert!(json.contains("\"kind\":\"unavailable\""), "{json}");
        assert!(json.contains("\"reason\":\"step-did-not-fail\""), "{json}");
    }
}
// CODEGEN-END

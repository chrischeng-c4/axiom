// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Open-mode pause / next-step run controls (#2885).
//!
//! In `jet e2e open`, the reviewer drives one case interactively
//! through the desktop shell. Before this slice, the reviewer can
//! only watch a case run end-to-end; this module adds the state
//! machine and shared command surface that lets them pause before or
//! after a step and advance exactly one step at a time.
//!
//! UI bindings consume [`OpenRunControlState`] and emit
//! [`OpenRunCommand`] values; the runner reads the resulting
//! [`OpenRunMode`] to know whether to block or continue between
//! steps. Keyboard shortcuts and a full debugger protocol are out of
//! scope.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`OpenRunControlState`].
pub const OPEN_RUN_CONTROL_SCHEMA_VERSION: &str = "jet.e2e.open-run-control.v1";

/// Where the reviewer wants the runner to stop relative to a step.
/// `Before` blocks the runner just before the named step's first
/// driver call. `After` blocks once the step's outcome is known.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PauseAnchor {
    Before,
    After,
}

/// One pending pause request — the runner reaches this step ID +
/// anchor and parks until the reviewer issues an advance.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PauseRequest {
    pub step_id: String,
    pub anchor: PauseAnchor,
}

/// Run mode the runner observes between steps.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum OpenRunMode {
    /// Runner streams every step without pausing.
    Continuous,
    /// Runner will block when it reaches the named step.
    PauseRequested { request: PauseRequest },
    /// Runner is currently parked at the named step.
    PausedAt { request: PauseRequest },
}

/// One command the UI hands to the control surface. The runner does
/// not see commands directly — it sees the resulting
/// [`OpenRunMode`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum OpenRunCommand {
    /// Arm a pause request. Replaces any previously armed request.
    Pause { request: PauseRequest },
    /// Cancel any armed pause; resume continuous mode.
    Resume,
    /// Advance exactly one step. Only valid while paused.
    NextStep,
}

/// Why a command was rejected. Surfaced to the UI so it can render a
/// disabled affordance instead of silently dropping the click.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenRunCommandError {
    /// `NextStep` was issued while the runner is not paused.
    NotPaused,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl std::fmt::Display for OpenRunCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotPaused => write!(f, "cannot step: runner is not paused"),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl std::error::Error for OpenRunCommandError {}

/// What the runner should do when it observes a step boundary.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerDecision {
    /// Keep running — no pause matches this boundary.
    Continue,
    /// Block at this boundary; runner must wait for `next_step`.
    Park { request: PauseRequest },
}

/// Shared open-mode control state. UI and runner both hold a clone;
/// updates flow UI → state → runner via [`runner_decision_for`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenRunControlState {
    pub schema_version: String,
    pub case_id: String,
    pub mode: OpenRunMode,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl OpenRunControlState {
    pub fn new(case_id: impl Into<String>) -> Self {
        Self {
            schema_version: OPEN_RUN_CONTROL_SCHEMA_VERSION.to_string(),
            case_id: case_id.into(),
            mode: OpenRunMode::Continuous,
        }
    }

    /// Apply a UI command and update [`OpenRunControlState::mode`].
    pub fn apply(&mut self, command: OpenRunCommand) -> Result<(), OpenRunCommandError> {
        match command {
            OpenRunCommand::Pause { request } => {
                self.mode = OpenRunMode::PauseRequested { request };
                Ok(())
            }
            OpenRunCommand::Resume => {
                self.mode = OpenRunMode::Continuous;
                Ok(())
            }
            OpenRunCommand::NextStep => match &self.mode {
                OpenRunMode::PausedAt { .. } => {
                    self.mode = OpenRunMode::Continuous;
                    Ok(())
                }
                _ => Err(OpenRunCommandError::NotPaused),
            },
        }
    }

    /// Mark the runner as parked at the named step. The runner calls
    /// this once it observes the requested boundary.
    pub fn mark_parked(&mut self, request: PauseRequest) {
        self.mode = OpenRunMode::PausedAt { request };
    }

    /// True when the runner is parked at a step boundary.
    pub fn is_paused(&self) -> bool {
        matches!(self.mode, OpenRunMode::PausedAt { .. })
    }

    /// Decide what the runner should do when it hits the named step
    /// boundary. The runner asks once before the step and once after.
    pub fn runner_decision_for(&self, step_id: &str, anchor: PauseAnchor) -> RunnerDecision {
        match &self.mode {
            OpenRunMode::Continuous | OpenRunMode::PausedAt { .. } => RunnerDecision::Continue,
            OpenRunMode::PauseRequested { request } => {
                if request.step_id == step_id && request.anchor == anchor {
                    RunnerDecision::Park {
                        request: request.clone(),
                    }
                } else {
                    RunnerDecision::Continue
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(step_id: &str, anchor: PauseAnchor) -> PauseRequest {
        PauseRequest {
            step_id: step_id.into(),
            anchor,
        }
    }

    #[test]
    fn default_mode_is_continuous() {
        let state = OpenRunControlState::new("case-buy");
        assert!(matches!(state.mode, OpenRunMode::Continuous));
        assert!(!state.is_paused());
    }

    #[test]
    fn pause_command_arms_pause_request_visible_to_runner() {
        // Stop condition (#2885): a selected case can pause before a
        // step.
        let mut state = OpenRunControlState::new("case-buy");
        state
            .apply(OpenRunCommand::Pause {
                request: req("click-confirm", PauseAnchor::Before),
            })
            .unwrap();
        match state.runner_decision_for("click-confirm", PauseAnchor::Before) {
            RunnerDecision::Park { request } => {
                assert_eq!(request.step_id, "click-confirm");
                assert_eq!(request.anchor, PauseAnchor::Before);
            }
            other => panic!("expected park, got {other:?}"),
        }
        assert_eq!(
            state.runner_decision_for("other-step", PauseAnchor::Before),
            RunnerDecision::Continue
        );
    }

    #[test]
    fn pause_after_anchor_blocks_only_after_named_step() {
        // Stop condition (#2885): a selected case can pause after a
        // step.
        let mut state = OpenRunControlState::new("case-buy");
        state
            .apply(OpenRunCommand::Pause {
                request: req("submit", PauseAnchor::After),
            })
            .unwrap();
        // Before the step: runner keeps going.
        assert_eq!(
            state.runner_decision_for("submit", PauseAnchor::Before),
            RunnerDecision::Continue
        );
        // After the step: runner parks.
        assert!(matches!(
            state.runner_decision_for("submit", PauseAnchor::After),
            RunnerDecision::Park { .. }
        ));
    }

    #[test]
    fn next_step_resumes_exactly_one_step() {
        // Stop condition (#2885): next-step resumes exactly one step.
        let mut state = OpenRunControlState::new("case-buy");
        state
            .apply(OpenRunCommand::Pause {
                request: req("step-1", PauseAnchor::Before),
            })
            .unwrap();
        state.mark_parked(req("step-1", PauseAnchor::Before));
        assert!(state.is_paused());

        // Step forward: runner returns to continuous and the next
        // boundary does not re-park.
        state.apply(OpenRunCommand::NextStep).unwrap();
        assert!(!state.is_paused());
        assert_eq!(
            state.runner_decision_for("step-1", PauseAnchor::Before),
            RunnerDecision::Continue
        );
        assert_eq!(
            state.runner_decision_for("step-2", PauseAnchor::Before),
            RunnerDecision::Continue
        );
    }

    #[test]
    fn next_step_is_rejected_when_not_paused() {
        let mut state = OpenRunControlState::new("case-buy");
        let err = state.apply(OpenRunCommand::NextStep).unwrap_err();
        assert_eq!(err, OpenRunCommandError::NotPaused);
    }

    #[test]
    fn resume_cancels_armed_pause() {
        let mut state = OpenRunControlState::new("case-buy");
        state
            .apply(OpenRunCommand::Pause {
                request: req("step-9", PauseAnchor::After),
            })
            .unwrap();
        state.apply(OpenRunCommand::Resume).unwrap();
        assert!(matches!(state.mode, OpenRunMode::Continuous));
        assert_eq!(
            state.runner_decision_for("step-9", PauseAnchor::After),
            RunnerDecision::Continue
        );
    }

    #[test]
    fn manual_review_can_step_through_one_case_step_by_step() {
        // Stop condition (#2885): manual review can step through one
        // case step-by-step.
        let mut state = OpenRunControlState::new("case-buy");
        let steps = ["nav", "fill-form", "submit"];
        let mut log = Vec::new();
        for step in steps {
            // Reviewer arms a pause-before for the next boundary.
            state
                .apply(OpenRunCommand::Pause {
                    request: req(step, PauseAnchor::Before),
                })
                .unwrap();
            // Runner walks until it hits this step boundary and parks.
            assert!(matches!(
                state.runner_decision_for(step, PauseAnchor::Before),
                RunnerDecision::Park { .. }
            ));
            state.mark_parked(req(step, PauseAnchor::Before));
            log.push(("parked", step.to_string()));
            // Reviewer hits next.
            state.apply(OpenRunCommand::NextStep).unwrap();
            log.push(("advanced", step.to_string()));
        }
        assert_eq!(log.len(), 6);
        assert!(matches!(state.mode, OpenRunMode::Continuous));
    }

    #[test]
    fn state_round_trips_through_json() {
        let mut state = OpenRunControlState::new("case-buy");
        state
            .apply(OpenRunCommand::Pause {
                request: req("submit", PauseAnchor::After),
            })
            .unwrap();
        let json = serde_json::to_string(&state).unwrap();
        let back: OpenRunControlState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
        assert!(json.contains("\"kind\":\"pause-requested\""), "{json}");
        assert!(json.contains("\"anchor\":\"after\""), "{json}");
    }
}
// CODEGEN-END

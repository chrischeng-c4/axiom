// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! AUT start/ready/shutdown lifecycle contract for `jet e2e` (#2723).
//!
//! The product-flow E2E runner has two execution surfaces — agent/CI
//! (`jet e2e run`) and local review (`jet e2e open`) — that share the
//! same Application-Under-Test lifecycle. This module defines the
//! deterministic phase contract both surfaces use:
//!
//! 1. `Start` — spawn the AUT process.
//! 2. `Ready` — wait until the AUT reports ready (port open, health
//!    probe succeeded, etc.).
//! 3. `Shutdown` — request shutdown, then reap.
//!
//! Each phase emits an [`AutLifecycleEvent`] into an
//! [`AutLifecycleRecord`]. The record is serialised into the run's
//! evidence bundle so failures (port already bound, ready probe
//! timed out, dirty shutdown) are visible to agents without log
//! scraping. Lifecycle failures map to deterministic exit codes via
//! [`AutLifecycleRecord::exit_code`] so CI can branch on failure
//! flavour without parsing JSON.
//!
//! Browser session launch is intentionally out of scope here
//! (controlled-browser plumbing is #2876); this module only models
//! the AUT process and its evidence shape.

use crate::e2e::{E2E_EXIT_INFRASTRUCTURE, E2E_EXIT_INVALID_CONFIG, E2E_EXIT_OK, E2E_EXIT_TIMEOUT};
use serde::{Deserialize, Serialize};

/// Stable schema tag for the AUT lifecycle record. Bumped on any
/// breaking change to the field shape.
pub const AUT_LIFECYCLE_SCHEMA_VERSION: &str = "jet.e2e.aut-lifecycle.v1";

/// Phases of the AUT lifecycle, ordered start -> ready -> shutdown.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutPhase {
    Start,
    Ready,
    Shutdown,
}

/// Outcome of a single phase. `Skipped` is reserved for shutdown
/// when start/ready failed (no AUT to shut down).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutPhaseOutcome {
    Succeeded,
    Failed,
    TimedOut,
    Skipped,
}

/// Failure category surfaced to the exit-code mapping. Mode-specific
/// runners pick the variant that matches the root cause; "infra"
/// covers process spawn / port bind / file-system failures.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutFailureKind {
    /// Probe or shutdown deadline expired.
    Timeout,
    /// Process spawn, port bind, or other OS-level failure.
    Infrastructure,
    /// Config-level rejection (e.g., missing command, bad port).
    InvalidConfig,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutPhaseFailure {
    pub kind: AutFailureKind,
    pub message: String,
}

/// One phase transition. `duration_ms` is the wall-time spent in the
/// phase; for `Skipped` phases it is always 0.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutLifecycleEvent {
    pub phase: AutPhase,
    pub outcome: AutPhaseOutcome,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<AutPhaseFailure>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl AutLifecycleEvent {
    pub fn succeeded(phase: AutPhase, duration_ms: u64) -> Self {
        Self {
            phase,
            outcome: AutPhaseOutcome::Succeeded,
            duration_ms,
            failure: None,
        }
    }

    pub fn skipped(phase: AutPhase) -> Self {
        Self {
            phase,
            outcome: AutPhaseOutcome::Skipped,
            duration_ms: 0,
            failure: None,
        }
    }

    pub fn failed(
        phase: AutPhase,
        duration_ms: u64,
        kind: AutFailureKind,
        message: impl Into<String>,
    ) -> Self {
        let outcome = match kind {
            AutFailureKind::Timeout => AutPhaseOutcome::TimedOut,
            _ => AutPhaseOutcome::Failed,
        };
        Self {
            phase,
            outcome,
            duration_ms,
            failure: Some(AutPhaseFailure {
                kind,
                message: message.into(),
            }),
        }
    }
}

/// Ordered sequence of lifecycle events for one run.
///
/// Construction is intentionally append-only: callers push events
/// as phases complete, then call [`AutLifecycleRecord::finalize`]
/// to seal the record before serialising it into evidence.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutLifecycleRecord {
    pub schema_version: String,
    pub events: Vec<AutLifecycleEvent>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl AutLifecycleRecord {
    pub fn new() -> Self {
        Self {
            schema_version: AUT_LIFECYCLE_SCHEMA_VERSION.to_string(),
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: AutLifecycleEvent) {
        self.events.push(event);
    }

    /// True when every recorded phase succeeded.
    pub fn is_success(&self) -> bool {
        self.events
            .iter()
            .all(|e| e.outcome == AutPhaseOutcome::Succeeded)
    }

    /// First failing event, if any. Useful for triage rendering.
    pub fn first_failure(&self) -> Option<&AutLifecycleEvent> {
        self.events.iter().find(|e| {
            matches!(
                e.outcome,
                AutPhaseOutcome::Failed | AutPhaseOutcome::TimedOut
            )
        })
    }

    /// Deterministic exit code derived from the first failing event.
    /// Mirrors the precedence already established in [`crate::e2e`]:
    /// infra > timeout > invalid-config > ok.
    pub fn exit_code(&self) -> i32 {
        match self.first_failure().and_then(|e| e.failure.as_ref()) {
            None => E2E_EXIT_OK,
            Some(f) => match f.kind {
                AutFailureKind::Infrastructure => E2E_EXIT_INFRASTRUCTURE,
                AutFailureKind::Timeout => E2E_EXIT_TIMEOUT,
                AutFailureKind::InvalidConfig => E2E_EXIT_INVALID_CONFIG,
            },
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl Default for AutLifecycleRecord {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_serialises_three_succeeded_events() {
        let mut rec = AutLifecycleRecord::new();
        rec.push(AutLifecycleEvent::succeeded(AutPhase::Start, 12));
        rec.push(AutLifecycleEvent::succeeded(AutPhase::Ready, 350));
        rec.push(AutLifecycleEvent::succeeded(AutPhase::Shutdown, 4));
        assert!(rec.is_success());
        assert_eq!(rec.exit_code(), E2E_EXIT_OK);
        let json = serde_json::to_string(&rec).unwrap();
        assert!(json.contains("\"phase\":\"start\""), "{json}");
        assert!(json.contains("\"phase\":\"ready\""), "{json}");
        assert!(json.contains("\"phase\":\"shutdown\""), "{json}");
        // Skip-serialised: succeeded events must not carry a failure key.
        assert!(!json.contains("\"failure\""), "{json}");

        let back: AutLifecycleRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rec);
        assert_eq!(back.schema_version, AUT_LIFECYCLE_SCHEMA_VERSION);
    }

    #[test]
    fn ready_timeout_maps_to_timeout_exit_code() {
        let mut rec = AutLifecycleRecord::new();
        rec.push(AutLifecycleEvent::succeeded(AutPhase::Start, 12));
        rec.push(AutLifecycleEvent::failed(
            AutPhase::Ready,
            5000,
            AutFailureKind::Timeout,
            "Health probe did not respond within 5000ms",
        ));
        rec.push(AutLifecycleEvent::skipped(AutPhase::Shutdown));
        assert!(!rec.is_success());
        assert_eq!(rec.exit_code(), E2E_EXIT_TIMEOUT);
        let first = rec.first_failure().expect("ready failure");
        assert_eq!(first.phase, AutPhase::Ready);
        assert_eq!(first.outcome, AutPhaseOutcome::TimedOut);
        assert_eq!(
            first.failure.as_ref().unwrap().kind,
            AutFailureKind::Timeout,
        );
    }

    #[test]
    fn infrastructure_failure_wins_over_later_timeout() {
        // First failure takes precedence; the exit code uses its kind.
        let mut rec = AutLifecycleRecord::new();
        rec.push(AutLifecycleEvent::failed(
            AutPhase::Start,
            8,
            AutFailureKind::Infrastructure,
            "port 4173 already in use",
        ));
        rec.push(AutLifecycleEvent::skipped(AutPhase::Ready));
        rec.push(AutLifecycleEvent::skipped(AutPhase::Shutdown));
        assert_eq!(rec.exit_code(), E2E_EXIT_INFRASTRUCTURE);
    }

    #[test]
    fn invalid_config_failure_uses_invalid_config_exit_code() {
        let mut rec = AutLifecycleRecord::new();
        rec.push(AutLifecycleEvent::failed(
            AutPhase::Start,
            0,
            AutFailureKind::InvalidConfig,
            "aut.command missing",
        ));
        assert_eq!(rec.exit_code(), E2E_EXIT_INVALID_CONFIG);
    }

    #[test]
    fn skipped_phase_serialises_with_zero_duration_and_no_failure() {
        let event = AutLifecycleEvent::skipped(AutPhase::Shutdown);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"outcome\":\"skipped\""), "{json}");
        assert!(json.contains("\"duration_ms\":0"), "{json}");
        assert!(!json.contains("\"failure\""), "{json}");
    }

    #[test]
    fn fixture_failure_record_round_trips_through_json() {
        // Stop condition (#2723): lifecycle success/failure events
        // serialize for at least one fixture. This is the failure
        // fixture; the happy-path fixture lives in
        // happy_path_serialises_three_succeeded_events above.
        let mut rec = AutLifecycleRecord::new();
        rec.push(AutLifecycleEvent::succeeded(AutPhase::Start, 50));
        rec.push(AutLifecycleEvent::failed(
            AutPhase::Ready,
            5000,
            AutFailureKind::Timeout,
            "probe timeout",
        ));
        rec.push(AutLifecycleEvent::skipped(AutPhase::Shutdown));
        let json = serde_json::to_string(&rec).unwrap();
        let back: AutLifecycleRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, rec);
        assert_eq!(
            back.first_failure()
                .unwrap()
                .failure
                .as_ref()
                .unwrap()
                .message,
            "probe timeout",
        );
    }
}
// CODEGEN-END

//! Task state machine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TaskId;

/// Task state in its lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskState {
    /// Task is waiting to be picked up
    #[default]
    Pending,
    /// Task has been received by a worker
    Received,
    /// Task is being executed
    Started,
    /// Task has been offloaded to external executor (e.g., K8s Job)
    Offloaded,
    /// Task completed successfully
    Success,
    /// Task failed permanently
    Failure,
    /// Task is being retried
    Retry,
    /// Task was revoked/cancelled
    Revoked,
    /// Task was rejected (invalid)
    Rejected,
}

impl TaskState {
    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Success | Self::Failure | Self::Revoked | Self::Rejected)
    }

    /// Check if task is in progress
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Received | Self::Started | Self::Retry | Self::Offloaded)
    }

    /// Check if task is offloaded to external executor
    pub fn is_offloaded(&self) -> bool {
        matches!(self, Self::Offloaded)
    }

    /// Valid state transitions
    pub fn can_transition_to(&self, next: TaskState) -> bool {
        match (self, next) {
            // From PENDING
            (Self::Pending, Self::Received) => true,
            (Self::Pending, Self::Revoked) => true,

            // From RECEIVED
            (Self::Received, Self::Started) => true,
            (Self::Received, Self::Revoked) => true,
            (Self::Received, Self::Rejected) => true,

            // From STARTED
            (Self::Started, Self::Success) => true,
            (Self::Started, Self::Failure) => true,
            (Self::Started, Self::Retry) => true,
            (Self::Started, Self::Revoked) => true,
            (Self::Started, Self::Offloaded) => true,

            // From OFFLOADED (external executor reports final state)
            (Self::Offloaded, Self::Success) => true,
            (Self::Offloaded, Self::Failure) => true,
            (Self::Offloaded, Self::Retry) => true,
            (Self::Offloaded, Self::Revoked) => true,

            // From RETRY
            (Self::Retry, Self::Pending) => true,
            (Self::Retry, Self::Received) => true,
            (Self::Retry, Self::Failure) => true,

            _ => false,
        }
    }
}

/// Full task result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TaskResult {
    pub task_id: TaskId,
    pub state: TaskState,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub traceback: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub runtime_ms: Option<u64>,
    pub retries: u32,
    pub worker_id: Option<String>,
}

impl TaskResult {
    /// Create a new pending result
    pub fn pending(task_id: TaskId) -> Self {
        Self {
            task_id,
            state: TaskState::Pending,
            result: None,
            error: None,
            traceback: None,
            started_at: None,
            completed_at: None,
            runtime_ms: None,
            retries: 0,
            worker_id: None,
        }
    }

    /// Create a success result
    pub fn success(task_id: TaskId, value: serde_json::Value) -> Self {
        Self {
            task_id,
            state: TaskState::Success,
            result: Some(value),
            error: None,
            traceback: None,
            started_at: None,
            completed_at: Some(Utc::now()),
            runtime_ms: None,
            retries: 0,
            worker_id: None,
        }
    }

    /// Create a failure result
    pub fn failure(task_id: TaskId, error: String) -> Self {
        Self {
            task_id,
            state: TaskState::Failure,
            result: None,
            error: Some(error),
            traceback: None,
            started_at: None,
            completed_at: Some(Utc::now()),
            runtime_ms: None,
            retries: 0,
            worker_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── TaskState Enum ──────────────────────────────────────────────

    /// T1: default is Pending
    #[test]
    fn default_is_pending() {
        assert_eq!(TaskState::default(), TaskState::Pending);
    }

    /// T2: serde round-trip for all 9 variants
    #[test]
    fn serde_round_trip() {
        let variants = [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Retry,
            TaskState::Revoked,
            TaskState::Rejected,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let back: TaskState = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "round-trip failed for {v:?}");
        }
    }

    /// T3: each variant serializes to SCREAMING_SNAKE_CASE
    #[test]
    fn serde_screaming_snake_case() {
        let cases = [
            (TaskState::Pending, "\"PENDING\""),
            (TaskState::Received, "\"RECEIVED\""),
            (TaskState::Started, "\"STARTED\""),
            (TaskState::Offloaded, "\"OFFLOADED\""),
            (TaskState::Success, "\"SUCCESS\""),
            (TaskState::Failure, "\"FAILURE\""),
            (TaskState::Retry, "\"RETRY\""),
            (TaskState::Revoked, "\"REVOKED\""),
            (TaskState::Rejected, "\"REJECTED\""),
        ];
        for (variant, expected) in &cases {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, *expected, "variant {variant:?} did not serialize to {expected}");
        }
    }

    // ── is_terminal() ───────────────────────────────────────────────

    /// T4: terminal states
    #[test]
    fn terminal_states() {
        for s in [TaskState::Success, TaskState::Failure, TaskState::Revoked, TaskState::Rejected] {
            assert!(s.is_terminal(), "{s:?} should be terminal");
        }
    }

    /// T5: non-terminal states
    #[test]
    fn non_terminal_states() {
        for s in [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Retry,
        ] {
            assert!(!s.is_terminal(), "{s:?} should NOT be terminal");
        }
    }

    // ── is_active() ─────────────────────────────────────────────────

    /// T6: active states
    #[test]
    fn active_states() {
        for s in [
            TaskState::Received,
            TaskState::Started,
            TaskState::Retry,
            TaskState::Offloaded,
        ] {
            assert!(s.is_active(), "{s:?} should be active");
        }
    }

    /// T7: non-active states
    #[test]
    fn non_active_states() {
        for s in [
            TaskState::Pending,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Revoked,
            TaskState::Rejected,
        ] {
            assert!(!s.is_active(), "{s:?} should NOT be active");
        }
    }

    // ── is_offloaded() ──────────────────────────────────────────────

    /// T8: Offloaded returns true
    #[test]
    fn offloaded_true() {
        assert!(TaskState::Offloaded.is_offloaded());
    }

    /// T9: all non-Offloaded variants return false
    #[test]
    fn offloaded_false_all_others() {
        for s in [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Retry,
            TaskState::Revoked,
            TaskState::Rejected,
        ] {
            assert!(!s.is_offloaded(), "{s:?} should NOT be offloaded");
        }
    }

    // ── can_transition_to() — Valid Transitions ─────────────────────

    /// T10: Pending -> Received
    #[test]
    fn transition_pending_to_received() {
        assert!(TaskState::Pending.can_transition_to(TaskState::Received));
    }

    /// T11: Pending -> Revoked
    #[test]
    fn transition_pending_to_revoked() {
        assert!(TaskState::Pending.can_transition_to(TaskState::Revoked));
    }

    /// T12: Received -> Started
    #[test]
    fn transition_received_to_started() {
        assert!(TaskState::Received.can_transition_to(TaskState::Started));
    }

    /// T13: Received -> Revoked
    #[test]
    fn transition_received_to_revoked() {
        assert!(TaskState::Received.can_transition_to(TaskState::Revoked));
    }

    /// T14: Received -> Rejected
    #[test]
    fn transition_received_to_rejected() {
        assert!(TaskState::Received.can_transition_to(TaskState::Rejected));
    }

    /// T15: Started -> Success
    #[test]
    fn transition_started_to_success() {
        assert!(TaskState::Started.can_transition_to(TaskState::Success));
    }

    /// T16: Started -> Failure
    #[test]
    fn transition_started_to_failure() {
        assert!(TaskState::Started.can_transition_to(TaskState::Failure));
    }

    /// T17: Started -> Retry
    #[test]
    fn transition_started_to_retry() {
        assert!(TaskState::Started.can_transition_to(TaskState::Retry));
    }

    /// T18: Started -> Revoked
    #[test]
    fn transition_started_to_revoked() {
        assert!(TaskState::Started.can_transition_to(TaskState::Revoked));
    }

    /// T19: Started -> Offloaded
    #[test]
    fn transition_started_to_offloaded() {
        assert!(TaskState::Started.can_transition_to(TaskState::Offloaded));
    }

    /// T20: Offloaded -> Success
    #[test]
    fn transition_offloaded_to_success() {
        assert!(TaskState::Offloaded.can_transition_to(TaskState::Success));
    }

    /// T21: Offloaded -> Failure
    #[test]
    fn transition_offloaded_to_failure() {
        assert!(TaskState::Offloaded.can_transition_to(TaskState::Failure));
    }

    /// T22: Offloaded -> Retry
    #[test]
    fn transition_offloaded_to_retry() {
        assert!(TaskState::Offloaded.can_transition_to(TaskState::Retry));
    }

    /// T23: Offloaded -> Revoked
    #[test]
    fn transition_offloaded_to_revoked() {
        assert!(TaskState::Offloaded.can_transition_to(TaskState::Revoked));
    }

    /// T24: Retry -> Pending
    #[test]
    fn transition_retry_to_pending() {
        assert!(TaskState::Retry.can_transition_to(TaskState::Pending));
    }

    /// T25: Retry -> Received
    #[test]
    fn transition_retry_to_received() {
        assert!(TaskState::Retry.can_transition_to(TaskState::Received));
    }

    /// T26: Retry -> Failure
    #[test]
    fn transition_retry_to_failure() {
        assert!(TaskState::Retry.can_transition_to(TaskState::Failure));
    }

    // ── can_transition_to() — Invalid Transitions ───────────────────

    /// T27: terminal states cannot transition to any state
    #[test]
    fn terminal_no_transitions() {
        let all = [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Retry,
            TaskState::Revoked,
            TaskState::Rejected,
        ];
        for terminal in [
            TaskState::Success,
            TaskState::Failure,
            TaskState::Revoked,
            TaskState::Rejected,
        ] {
            for target in &all {
                assert!(
                    !terminal.can_transition_to(*target),
                    "{terminal:?} -> {target:?} should be invalid (terminal state)",
                );
            }
        }
    }

    /// T28: invalid transitions from Pending
    #[test]
    fn invalid_transitions_from_pending() {
        let invalid = [
            TaskState::Pending,
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Retry,
            TaskState::Rejected,
        ];
        for target in &invalid {
            assert!(
                !TaskState::Pending.can_transition_to(*target),
                "Pending -> {target:?} should be invalid",
            );
        }
    }

    /// T29: invalid transitions from Received
    #[test]
    fn invalid_transitions_from_received() {
        let invalid = [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Offloaded,
            TaskState::Success,
            TaskState::Failure,
            TaskState::Retry,
        ];
        for target in &invalid {
            assert!(
                !TaskState::Received.can_transition_to(*target),
                "Received -> {target:?} should be invalid",
            );
        }
    }

    /// T30: invalid transitions from Started
    #[test]
    fn invalid_transitions_from_started() {
        let invalid = [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Rejected,
        ];
        for target in &invalid {
            assert!(
                !TaskState::Started.can_transition_to(*target),
                "Started -> {target:?} should be invalid",
            );
        }
    }

    /// T31: invalid transitions from Offloaded
    #[test]
    fn invalid_transitions_from_offloaded() {
        let invalid = [
            TaskState::Pending,
            TaskState::Received,
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Rejected,
        ];
        for target in &invalid {
            assert!(
                !TaskState::Offloaded.can_transition_to(*target),
                "Offloaded -> {target:?} should be invalid",
            );
        }
    }

    /// T32: invalid transitions from Retry
    #[test]
    fn invalid_transitions_from_retry() {
        let invalid = [
            TaskState::Started,
            TaskState::Offloaded,
            TaskState::Success,
            TaskState::Retry,
            TaskState::Revoked,
            TaskState::Rejected,
        ];
        for target in &invalid {
            assert!(
                !TaskState::Retry.can_transition_to(*target),
                "Retry -> {target:?} should be invalid",
            );
        }
    }

    // ── TaskResult Constructors ─────────────────────────────────────

    /// T33: pending result
    #[test]
    fn pending_result() {
        let id = TaskId::new();
        let r = TaskResult::pending(id.clone());
        assert_eq!(r.state, TaskState::Pending);
        assert!(r.result.is_none());
        assert!(r.error.is_none());
        assert_eq!(r.retries, 0);
        assert!(r.completed_at.is_none());
    }

    /// T34: success result
    #[test]
    fn success_result() {
        let id = TaskId::new();
        let val = json!({"answer": 42});
        let r = TaskResult::success(id.clone(), val.clone());
        assert_eq!(r.state, TaskState::Success);
        assert_eq!(r.result, Some(val));
        assert!(r.completed_at.is_some());
    }

    /// T35: failure result
    #[test]
    fn failure_result() {
        let id = TaskId::new();
        let r = TaskResult::failure(id.clone(), "boom".to_string());
        assert_eq!(r.state, TaskState::Failure);
        assert_eq!(r.error.as_deref(), Some("boom"));
        assert!(r.completed_at.is_some());
    }

    /// T36: serde round-trip for TaskResult
    #[test]
    fn result_serde_round_trip() {
        let id = TaskId::new();
        let r = TaskResult::success(id.clone(), json!({"x": 1}));
        let json_str = serde_json::to_string(&r).unwrap();
        let back: TaskResult = serde_json::from_str(&json_str).unwrap();
        assert_eq!(back.task_id, r.task_id);
        assert_eq!(back.state, r.state);
        assert_eq!(back.result, r.result);
        assert_eq!(back.error, r.error);
        assert_eq!(back.retries, r.retries);
        // completed_at round-trips (both Some)
        assert!(back.completed_at.is_some());
    }

    // ── Trait Bounds ────────────────────────────────────────────────

    /// T37: TaskState is Copy + Clone + Eq
    #[test]
    fn task_state_is_copy_clone_eq() {
        let a = TaskState::Pending;
        let b = a; // Copy
        assert_eq!(a, b); // Eq
    }

    /// T38: TaskState Debug contains variant name
    #[test]
    fn task_state_debug() {
        let dbg = format!("{:?}", TaskState::Pending);
        assert!(dbg.contains("Pending"), "Debug output was: {dbg}");
    }

    /// T39: TaskResult is Clone
    #[test]
    fn task_result_is_clone() {
        let r = TaskResult::pending(TaskId::new());
        let cloned = r.clone();
        assert_eq!(cloned.state, r.state);
        assert_eq!(cloned.task_id, r.task_id);
    }
}

//! Scheduler backend abstraction
//!
//! Provides leader election and task state persistence.

use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::TaskError;

use super::periodic::PeriodicTask;

/// Runtime scheduling mode determined by the backend type.
///
/// `SelfHosted` — backend manages scheduling internally via leader election tick loop.
/// `ExternalPush` — external system manages scheduling (Cloud Scheduler, K8s CronJob),
/// triggers arrive via HTTP push receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingMode {
    /// Leader election tick loop (Ion, Memory backends)
    SelfHosted,
    /// External push receiver (Cloud Scheduler, K8s CronJob backends)
    ExternalPush,
}

/// State of a scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScheduleState {
    /// Whether the task is enabled (not paused)
    pub enabled: bool,
    /// Last time the task was run
    pub last_run_at: Option<DateTime<Utc>>,
    /// Total number of times the task has run
    pub total_run_count: u64,
}

impl Default for TaskScheduleState {
    fn default() -> Self {
        Self {
            enabled: true,
            last_run_at: None,
            total_run_count: 0,
        }
    }
}

/// Backend for scheduler leader election and state persistence
#[async_trait]
pub trait SchedulerBackend: Send + Sync {
    /// Try to acquire leadership for the scheduler.
    /// Returns true if this instance became the leader.
    async fn acquire_leader(&self, ttl: Duration) -> Result<bool, TaskError>;

    /// Renew leadership. Must be called before TTL expires.
    /// Returns true if leadership was successfully renewed.
    async fn renew_leader(&self, ttl: Duration) -> Result<bool, TaskError>;

    /// Release leadership (optional, for graceful shutdown)
    async fn release_leader(&self) -> Result<(), TaskError>;

    /// Get the state of a scheduled task
    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError>;

    /// Set the state of a scheduled task
    async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError>;

    /// Return the scheduling mode for this backend.
    ///
    /// This is a static property of the backend type — not async, not fallible.
    /// Default returns `SelfHosted` for backward compatibility. External backends
    /// (Cloud Scheduler, K8s CronJob) override to return `ExternalPush`.
    fn scheduling_mode(&self) -> SchedulingMode {
        SchedulingMode::SelfHosted
    }

    /// Register a periodic task with the external scheduling system.
    ///
    /// Called for each task during `PeriodicScheduler::start()` in `ExternalPush` mode.
    /// Creates the corresponding external resource (e.g., Cloud Scheduler job, K8s CronJob).
    ///
    /// Default is a no-op (returns `Ok(())`) — self-hosted backends do not need this.
    async fn register_external_schedule(&self, _task: &PeriodicTask) -> Result<(), TaskError> {
        Ok(())
    }

    /// Update last run time and increment run count
    ///
    /// Note: This is not truly atomic - for high-concurrency scenarios,
    /// consider implementing a backend-specific atomic increment.
    ///
    /// The backend's `get_task_state` should return `Ok(default)` for missing keys,
    /// and `Err` only for real backend errors (which are propagated here).
    async fn record_task_run(&self, name: &str) -> Result<(), TaskError> {
        let mut state = self.get_task_state(name).await?;
        state.last_run_at = Some(Utc::now());
        state.total_run_count += 1;
        self.set_task_state(name, &state).await
    }

    /// Pause a task (set enabled = false)
    async fn pause_task(&self, name: &str) -> Result<(), TaskError> {
        let mut state = self.get_task_state(name).await?;
        state.enabled = false;
        self.set_task_state(name, &state).await
    }

    /// Resume a task (set enabled = true)
    async fn resume_task(&self, name: &str) -> Result<(), TaskError> {
        let mut state = self.get_task_state(name).await?;
        state.enabled = true;
        self.set_task_state(name, &state).await
    }

    /// Check if a task is enabled
    async fn is_task_enabled(&self, name: &str) -> Result<bool, TaskError> {
        let state = self.get_task_state(name).await?;
        Ok(state.enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // R1: SchedulingMode enum — derives and variants
    // -----------------------------------------------------------------------

    #[test]
    fn r1_scheduling_mode_has_self_hosted_variant() {
        let mode = SchedulingMode::SelfHosted;
        assert_eq!(mode, SchedulingMode::SelfHosted);
    }

    #[test]
    fn r1_scheduling_mode_has_external_push_variant() {
        let mode = SchedulingMode::ExternalPush;
        assert_eq!(mode, SchedulingMode::ExternalPush);
    }

    #[test]
    fn r1_scheduling_mode_debug_derive() {
        // Verifies Debug derive
        let debug_str = format!("{:?}", SchedulingMode::SelfHosted);
        assert_eq!(debug_str, "SelfHosted");
        let debug_str = format!("{:?}", SchedulingMode::ExternalPush);
        assert_eq!(debug_str, "ExternalPush");
    }

    #[test]
    fn r1_scheduling_mode_clone_derive() {
        let mode = SchedulingMode::ExternalPush;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn r1_scheduling_mode_copy_derive() {
        let mode = SchedulingMode::SelfHosted;
        let copied = mode;
        // Original is still accessible (Copy trait)
        assert_eq!(mode, copied);
    }

    #[test]
    fn r1_scheduling_mode_eq_derive() {
        assert_eq!(SchedulingMode::SelfHosted, SchedulingMode::SelfHosted);
        assert_eq!(SchedulingMode::ExternalPush, SchedulingMode::ExternalPush);
        assert_ne!(SchedulingMode::SelfHosted, SchedulingMode::ExternalPush);
    }

    // -----------------------------------------------------------------------
    // S8: Default scheduling_mode() returns SelfHosted (R2)
    // -----------------------------------------------------------------------

    /// A minimal backend that does NOT override scheduling_mode(),
    /// so it uses the default trait implementation.
    struct MinimalBackend;

    #[async_trait]
    impl SchedulerBackend for MinimalBackend {
        async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
            Ok(true)
        }
        async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
            Ok(true)
        }
        async fn release_leader(&self) -> Result<(), TaskError> {
            Ok(())
        }
        async fn get_task_state(&self, _name: &str) -> Result<TaskScheduleState, TaskError> {
            Ok(TaskScheduleState::default())
        }
        async fn set_task_state(
            &self,
            _name: &str,
            _state: &TaskScheduleState,
        ) -> Result<(), TaskError> {
            Ok(())
        }
    }

    #[test]
    fn s8_default_scheduling_mode_returns_self_hosted() {
        let backend = MinimalBackend;
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    // -----------------------------------------------------------------------
    // R8: Default register_external_schedule() is a no-op
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn r8_default_register_external_schedule_returns_ok() {
        let backend = MinimalBackend;
        let task = super::super::periodic::PeriodicTask {
            name: "test-task".to_string(),
            task_name: "my_task".to_string(),
            schedule: super::super::periodic::PeriodicSchedule::Interval(60),
            args: serde_json::json!({}),
            queue: "default".to_string(),
            enabled: true,
        };
        let result = backend.register_external_schedule(&task).await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // Backend that overrides scheduling_mode to ExternalPush
    // -----------------------------------------------------------------------

    struct ExternalPushBackend;

    #[async_trait]
    impl SchedulerBackend for ExternalPushBackend {
        async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
            Ok(true)
        }
        async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
            Ok(true)
        }
        async fn release_leader(&self) -> Result<(), TaskError> {
            Ok(())
        }
        async fn get_task_state(&self, _name: &str) -> Result<TaskScheduleState, TaskError> {
            Ok(TaskScheduleState::default())
        }
        async fn set_task_state(
            &self,
            _name: &str,
            _state: &TaskScheduleState,
        ) -> Result<(), TaskError> {
            Ok(())
        }
        fn scheduling_mode(&self) -> SchedulingMode {
            SchedulingMode::ExternalPush
        }
    }

    #[test]
    fn s12_external_push_backend_override() {
        // Verifies a backend CAN override scheduling_mode to return ExternalPush
        let backend = ExternalPushBackend;
        assert_eq!(backend.scheduling_mode(), SchedulingMode::ExternalPush);
    }

    // -----------------------------------------------------------------------
    // scheduling_mode() is not async and not fallible (R2 constraint)
    // -----------------------------------------------------------------------

    #[test]
    fn r2_scheduling_mode_is_sync_and_infallible() {
        // This test verifies the constraint that scheduling_mode() returns
        // SchedulingMode directly (not Result, not Future).
        // If it were async or fallible, this would not compile.
        let backend = MinimalBackend;
        let mode: SchedulingMode = backend.scheduling_mode();
        assert_eq!(mode, SchedulingMode::SelfHosted);
    }

    // -----------------------------------------------------------------------
    // Default trait method tests via MemorySchedulerBackend
    // -----------------------------------------------------------------------

    use crate::scheduler::memory_backend::MemorySchedulerBackend;

    #[tokio::test]
    async fn record_task_run_increments_count() {
        let backend = MemorySchedulerBackend::new();

        // First run: count goes from 0 to 1
        backend.record_task_run("task-a").await.unwrap();
        let state = backend.get_task_state("task-a").await.unwrap();
        assert_eq!(state.total_run_count, 1);
        assert!(state.last_run_at.is_some());

        // Second run: count goes to 2
        backend.record_task_run("task-a").await.unwrap();
        let state = backend.get_task_state("task-a").await.unwrap();
        assert_eq!(state.total_run_count, 2);

        // Third run: count goes to 3
        backend.record_task_run("task-a").await.unwrap();
        let state = backend.get_task_state("task-a").await.unwrap();
        assert_eq!(state.total_run_count, 3);
    }

    #[tokio::test]
    async fn record_task_run_updates_last_run_at() {
        let backend = MemorySchedulerBackend::new();

        // Before any run, last_run_at is None
        let state = backend.get_task_state("timed").await.unwrap();
        assert!(state.last_run_at.is_none());

        let before = chrono::Utc::now();
        backend.record_task_run("timed").await.unwrap();
        let after = chrono::Utc::now();

        let state = backend.get_task_state("timed").await.unwrap();
        let last_run = state.last_run_at.expect("last_run_at should be set");
        assert!(last_run >= before && last_run <= after);
    }

    #[tokio::test]
    async fn record_task_run_does_not_affect_enabled() {
        let backend = MemorySchedulerBackend::new();

        // enabled starts as true; record_task_run should not change it
        backend.record_task_run("e-task").await.unwrap();
        let state = backend.get_task_state("e-task").await.unwrap();
        assert!(state.enabled);

        // Pause it, then record a run: enabled stays false
        backend.pause_task("e-task").await.unwrap();
        backend.record_task_run("e-task").await.unwrap();
        let state = backend.get_task_state("e-task").await.unwrap();
        assert!(!state.enabled);
    }

    #[tokio::test]
    async fn record_task_run_independent_tasks() {
        let backend = MemorySchedulerBackend::new();

        backend.record_task_run("x").await.unwrap();
        backend.record_task_run("x").await.unwrap();
        backend.record_task_run("y").await.unwrap();

        let sx = backend.get_task_state("x").await.unwrap();
        let sy = backend.get_task_state("y").await.unwrap();
        assert_eq!(sx.total_run_count, 2);
        assert_eq!(sy.total_run_count, 1);
    }

    #[tokio::test]
    async fn pause_task_sets_enabled_false() {
        let backend = MemorySchedulerBackend::new();

        // Default is enabled=true
        let state = backend.get_task_state("p").await.unwrap();
        assert!(state.enabled);

        backend.pause_task("p").await.unwrap();
        let state = backend.get_task_state("p").await.unwrap();
        assert!(!state.enabled);
    }

    #[tokio::test]
    async fn pause_task_is_idempotent() {
        let backend = MemorySchedulerBackend::new();

        backend.pause_task("p").await.unwrap();
        backend.pause_task("p").await.unwrap();
        let state = backend.get_task_state("p").await.unwrap();
        assert!(!state.enabled);
    }

    #[tokio::test]
    async fn resume_task_sets_enabled_true() {
        let backend = MemorySchedulerBackend::new();

        backend.pause_task("r").await.unwrap();
        assert!(!backend.get_task_state("r").await.unwrap().enabled);

        backend.resume_task("r").await.unwrap();
        assert!(backend.get_task_state("r").await.unwrap().enabled);
    }

    #[tokio::test]
    async fn resume_task_is_idempotent() {
        let backend = MemorySchedulerBackend::new();

        // Already enabled by default; resume should keep it enabled
        backend.resume_task("r").await.unwrap();
        assert!(backend.get_task_state("r").await.unwrap().enabled);
    }

    #[tokio::test]
    async fn pause_resume_cycle() {
        let backend = MemorySchedulerBackend::new();

        assert!(backend.is_task_enabled("c").await.unwrap());

        backend.pause_task("c").await.unwrap();
        assert!(!backend.is_task_enabled("c").await.unwrap());

        backend.resume_task("c").await.unwrap();
        assert!(backend.is_task_enabled("c").await.unwrap());

        backend.pause_task("c").await.unwrap();
        assert!(!backend.is_task_enabled("c").await.unwrap());
    }

    #[tokio::test]
    async fn is_task_enabled_default_true() {
        let backend = MemorySchedulerBackend::new();
        // A never-seen task defaults to enabled
        assert!(backend.is_task_enabled("unknown").await.unwrap());
    }

    #[tokio::test]
    async fn is_task_enabled_after_pause() {
        let backend = MemorySchedulerBackend::new();
        backend.pause_task("e").await.unwrap();
        assert!(!backend.is_task_enabled("e").await.unwrap());
    }

    #[tokio::test]
    async fn is_task_enabled_after_resume() {
        let backend = MemorySchedulerBackend::new();
        backend.pause_task("e").await.unwrap();
        backend.resume_task("e").await.unwrap();
        assert!(backend.is_task_enabled("e").await.unwrap());
    }

    #[tokio::test]
    async fn scheduling_mode_returns_self_hosted_for_memory_backend() {
        let backend = MemorySchedulerBackend::new();
        // MemorySchedulerBackend does NOT override scheduling_mode(),
        // so it inherits the default SelfHosted.
        assert_eq!(backend.scheduling_mode(), SchedulingMode::SelfHosted);
    }

    #[tokio::test]
    async fn register_external_schedule_noop_for_memory_backend() {
        let backend = MemorySchedulerBackend::new();
        let task = super::super::periodic::PeriodicTask {
            name: "ext-task".to_string(),
            task_name: "my_task".to_string(),
            schedule: super::super::periodic::PeriodicSchedule::Interval(30),
            args: serde_json::json!({}),
            queue: "default".to_string(),
            enabled: true,
        };
        // Default implementation is a no-op returning Ok(())
        let result = backend.register_external_schedule(&task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn pause_preserves_run_count() {
        let backend = MemorySchedulerBackend::new();

        backend.record_task_run("prc").await.unwrap();
        backend.record_task_run("prc").await.unwrap();
        assert_eq!(backend.get_task_state("prc").await.unwrap().total_run_count, 2);

        // Pausing should not reset the run count
        backend.pause_task("prc").await.unwrap();
        let state = backend.get_task_state("prc").await.unwrap();
        assert_eq!(state.total_run_count, 2);
        assert!(!state.enabled);
    }

    #[tokio::test]
    async fn resume_preserves_run_count() {
        let backend = MemorySchedulerBackend::new();

        backend.record_task_run("rrc").await.unwrap();
        backend.pause_task("rrc").await.unwrap();
        backend.resume_task("rrc").await.unwrap();

        let state = backend.get_task_state("rrc").await.unwrap();
        assert_eq!(state.total_run_count, 1);
        assert!(state.enabled);
        assert!(state.last_run_at.is_some());
    }

    // -----------------------------------------------------------------------
    // TaskScheduleState default
    // -----------------------------------------------------------------------

    #[test]
    fn task_schedule_state_default() {
        let state = TaskScheduleState::default();
        assert!(state.enabled);
        assert!(state.last_run_at.is_none());
        assert_eq!(state.total_run_count, 0);
    }
}

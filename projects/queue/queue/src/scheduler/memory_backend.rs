//! In-memory scheduler backend for testing
//!
//! Simple implementation without distributed coordination.
//! Uses tokio::sync::RwLock for async compatibility.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::backend::{SchedulerBackend, TaskScheduleState};
use crate::TaskError;

/// In-memory scheduler backend for testing
///
/// Always grants leadership (single instance assumption).
/// Note: This is intended for testing only, not production use.
pub struct MemorySchedulerBackend {
    task_states: RwLock<HashMap<String, TaskScheduleState>>,
    is_leader: RwLock<bool>,
}

impl MemorySchedulerBackend {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            task_states: RwLock::new(HashMap::new()),
            is_leader: RwLock::new(false),
        }
    }
}

impl Default for MemorySchedulerBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SchedulerBackend for MemorySchedulerBackend {
    async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
        let mut is_leader = self.is_leader.write().await;
        *is_leader = true;
        Ok(true)
    }

    async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
        let is_leader = self.is_leader.read().await;
        Ok(*is_leader)
    }

    async fn release_leader(&self) -> Result<(), TaskError> {
        let mut is_leader = self.is_leader.write().await;
        *is_leader = false;
        Ok(())
    }

    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError> {
        let states = self.task_states.read().await;
        Ok(states.get(name).cloned().unwrap_or_default())
    }

    async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError> {
        let mut states = self.task_states.write().await;
        states.insert(name.to_string(), state.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_backend_leadership() {
        let backend = MemorySchedulerBackend::new();

        // Should acquire leadership
        assert!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap());

        // Should be able to renew
        assert!(backend.renew_leader(Duration::from_secs(10)).await.unwrap());

        // Release leadership
        backend.release_leader().await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_backend_task_state() {
        let backend = MemorySchedulerBackend::new();

        // Default state
        let state = backend.get_task_state("test-task").await.unwrap();
        assert!(state.enabled);
        assert_eq!(state.total_run_count, 0);

        // Pause task
        backend.pause_task("test-task").await.unwrap();
        let state = backend.get_task_state("test-task").await.unwrap();
        assert!(!state.enabled);

        // Resume task
        backend.resume_task("test-task").await.unwrap();
        let state = backend.get_task_state("test-task").await.unwrap();
        assert!(state.enabled);

        // Record run
        backend.record_task_run("test-task").await.unwrap();
        let state = backend.get_task_state("test-task").await.unwrap();
        assert_eq!(state.total_run_count, 1);
        assert!(state.last_run_at.is_some());
    }

    // -------------------------------------------------------------------
    // S1: Leader Election
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn test_acquire_leader_returns_true() {
        let backend = MemorySchedulerBackend::new();
        let result = backend.acquire_leader(Duration::from_secs(10)).await;
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_renew_leader_after_acquire() {
        let backend = MemorySchedulerBackend::new();
        backend.acquire_leader(Duration::from_secs(10)).await.unwrap();
        let result = backend.renew_leader(Duration::from_secs(10)).await;
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_renew_leader_before_acquire() {
        let backend = MemorySchedulerBackend::new();
        // is_leader starts as false, so renew without acquire should return false
        let result = backend.renew_leader(Duration::from_secs(10)).await;
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_release_leader() {
        let backend = MemorySchedulerBackend::new();
        backend.acquire_leader(Duration::from_secs(10)).await.unwrap();
        let result = backend.release_leader().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_renew_leader_after_release() {
        let backend = MemorySchedulerBackend::new();
        backend.acquire_leader(Duration::from_secs(10)).await.unwrap();
        backend.release_leader().await.unwrap();
        let result = backend.renew_leader(Duration::from_secs(10)).await;
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_leader_full_cycle() {
        let backend = MemorySchedulerBackend::new();

        // acquire -> true
        assert_eq!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap(), true);
        // renew -> true
        assert_eq!(backend.renew_leader(Duration::from_secs(10)).await.unwrap(), true);
        // release -> ok
        backend.release_leader().await.unwrap();
        // renew after release -> false
        assert_eq!(backend.renew_leader(Duration::from_secs(10)).await.unwrap(), false);
        // re-acquire -> true
        assert_eq!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_acquire_leader_ignores_ttl() {
        let backend = MemorySchedulerBackend::new();
        // 0s TTL still returns true
        assert_eq!(backend.acquire_leader(Duration::from_secs(0)).await.unwrap(), true);

        // Reset for next check
        backend.release_leader().await.unwrap();

        // 1000s TTL also returns true
        assert_eq!(backend.acquire_leader(Duration::from_secs(1000)).await.unwrap(), true);
    }

    // -------------------------------------------------------------------
    // S2: Task State CRUD
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn test_get_task_state_default() {
        let backend = MemorySchedulerBackend::new();
        let state = backend.get_task_state("unknown-task").await.unwrap();
        assert!(state.enabled);
        assert!(state.last_run_at.is_none());
        assert_eq!(state.total_run_count, 0);
    }

    #[tokio::test]
    async fn test_set_and_get_task_state() {
        use chrono::Utc;

        let backend = MemorySchedulerBackend::new();
        let now = Utc::now();
        let state = TaskScheduleState {
            enabled: false,
            last_run_at: Some(now),
            total_run_count: 42,
        };
        backend.set_task_state("my-task", &state).await.unwrap();

        let retrieved = backend.get_task_state("my-task").await.unwrap();
        assert_eq!(retrieved.enabled, false);
        assert_eq!(retrieved.last_run_at.unwrap(), now);
        assert_eq!(retrieved.total_run_count, 42);
    }

    #[tokio::test]
    async fn test_set_task_state_upsert() {
        use chrono::Utc;

        let backend = MemorySchedulerBackend::new();
        let first = TaskScheduleState {
            enabled: true,
            last_run_at: None,
            total_run_count: 1,
        };
        backend.set_task_state("task-a", &first).await.unwrap();

        let second = TaskScheduleState {
            enabled: false,
            last_run_at: Some(Utc::now()),
            total_run_count: 99,
        };
        backend.set_task_state("task-a", &second).await.unwrap();

        let retrieved = backend.get_task_state("task-a").await.unwrap();
        assert_eq!(retrieved.enabled, false);
        assert_eq!(retrieved.total_run_count, 99);
        assert!(retrieved.last_run_at.is_some());
    }

    #[tokio::test]
    async fn test_multiple_tasks_isolated() {
        let backend = MemorySchedulerBackend::new();

        let state_a = TaskScheduleState {
            enabled: false,
            last_run_at: None,
            total_run_count: 10,
        };
        let state_b = TaskScheduleState {
            enabled: true,
            last_run_at: None,
            total_run_count: 20,
        };
        backend.set_task_state("task-a", &state_a).await.unwrap();
        backend.set_task_state("task-b", &state_b).await.unwrap();

        let a = backend.get_task_state("task-a").await.unwrap();
        let b = backend.get_task_state("task-b").await.unwrap();

        assert_eq!(a.enabled, false);
        assert_eq!(a.total_run_count, 10);
        assert_eq!(b.enabled, true);
        assert_eq!(b.total_run_count, 20);
    }

    // -------------------------------------------------------------------
    // S3: Pause & Resume
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn test_pause_task() {
        let backend = MemorySchedulerBackend::new();
        backend.pause_task("my-task").await.unwrap();
        let state = backend.get_task_state("my-task").await.unwrap();
        assert_eq!(state.enabled, false);
    }

    #[tokio::test]
    async fn test_resume_task() {
        let backend = MemorySchedulerBackend::new();
        backend.pause_task("my-task").await.unwrap();
        backend.resume_task("my-task").await.unwrap();
        let state = backend.get_task_state("my-task").await.unwrap();
        assert_eq!(state.enabled, true);
    }

    #[tokio::test]
    async fn test_is_task_enabled_default() {
        let backend = MemorySchedulerBackend::new();
        let result = backend.is_task_enabled("nonexistent").await;
        assert_eq!(result.unwrap(), true);
    }

    #[tokio::test]
    async fn test_is_task_enabled_after_pause() {
        let backend = MemorySchedulerBackend::new();
        backend.pause_task("my-task").await.unwrap();
        let result = backend.is_task_enabled("my-task").await;
        assert_eq!(result.unwrap(), false);
    }

    // -------------------------------------------------------------------
    // S4: Record Task Run
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn test_record_task_run_increments_count() {
        let backend = MemorySchedulerBackend::new();

        backend.record_task_run("counter-task").await.unwrap();
        let state = backend.get_task_state("counter-task").await.unwrap();
        assert_eq!(state.total_run_count, 1);

        backend.record_task_run("counter-task").await.unwrap();
        let state = backend.get_task_state("counter-task").await.unwrap();
        assert_eq!(state.total_run_count, 2);
    }

    #[tokio::test]
    async fn test_record_task_run_updates_last_run_at() {
        let backend = MemorySchedulerBackend::new();
        backend.record_task_run("timed-task").await.unwrap();
        let state = backend.get_task_state("timed-task").await.unwrap();
        assert!(state.last_run_at.is_some());
    }

    #[tokio::test]
    async fn test_record_task_run_preserves_enabled() {
        let backend = MemorySchedulerBackend::new();
        // Default is enabled=true
        backend.record_task_run("enabled-task").await.unwrap();
        let state = backend.get_task_state("enabled-task").await.unwrap();
        assert_eq!(state.enabled, true);
    }

    // -------------------------------------------------------------------
    // S5: Construction & Bounds
    // -------------------------------------------------------------------

    #[tokio::test]
    async fn test_new_and_default_equivalent() {
        let from_new = MemorySchedulerBackend::new();
        let from_default = MemorySchedulerBackend::default();

        // Both should produce empty state (no tasks, not leader)
        let new_state = from_new.get_task_state("any").await.unwrap();
        let default_state = from_default.get_task_state("any").await.unwrap();

        assert_eq!(new_state.enabled, default_state.enabled);
        assert_eq!(new_state.last_run_at, default_state.last_run_at);
        assert_eq!(new_state.total_run_count, default_state.total_run_count);

        // Neither is leader
        assert_eq!(from_new.renew_leader(Duration::from_secs(1)).await.unwrap(), false);
        assert_eq!(from_default.renew_leader(Duration::from_secs(1)).await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MemorySchedulerBackend>();
    }
}

// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#schema
// HANDWRITE-BEGIN gap="missing-generator:schema:defer-core-types" tracker="#766" reason="Core DTOs for delayed push-queue scheduling and dispatch attempts."
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

pub const DEFAULT_PRIORITY: u8 = 10;

pub type TaskId = String;
pub type QueueName = String;
pub type AttemptId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Target {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

fn default_method() -> String {
    "POST".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CreateTask {
    pub task_id: TaskId,
    pub target: Target,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub schedule_at: DateTime<Utc>,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
}

pub fn default_priority() -> u8 {
    DEFAULT_PRIORITY
}

fn default_max_attempts() -> u32 {
    3
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct QueuePolicy {
    pub max_in_flight: usize,
    pub max_dispatch_per_tick: usize,
    pub max_dispatches_per_second: u32,
    pub max_burst_size: usize,
    pub lease_ttl_ms: u64,
    pub retry_backoff_ms: u64,
}

impl Default for QueuePolicy {
    fn default() -> Self {
        Self {
            max_in_flight: 100,
            max_dispatch_per_tick: 100,
            max_dispatches_per_second: 100,
            max_burst_size: 100,
            lease_ttl_ms: 30_000,
            retry_backoff_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum QueueControlState {
    #[default]
    Running,
    Paused,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct QueueSnapshot {
    pub queue: QueueName,
    pub control_state: QueueControlState,
    pub policy: QueuePolicy,
    pub task_count: usize,
    pub scheduled_count: usize,
    pub in_flight_count: usize,
    pub terminal_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum TaskStatus {
    Scheduled,
    Leased {
        attempt_id: AttemptId,
        executor_node: u64,
        epoch: u64,
        expires_at: DateTime<Utc>,
    },
    Succeeded,
    DeadLettered,
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchLease {
    pub attempt_id: AttemptId,
    pub task_id: TaskId,
    pub queue: QueueName,
    pub target: Target,
    pub payload: serde_json::Value,
    pub priority: u8,
    pub attempt: u32,
    /// Stable across every retry of this logical task.
    pub idempotency_key: String,
    pub executor_node: u64,
    pub epoch: u64,
    pub leased_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// One externally observed HTTP result to commit against its fenced lease.
/// The proposer resolves `completed_at`; replicas only apply these bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptSettlement {
    pub attempt_id: AttemptId,
    pub epoch: u64,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementOutcome {
    Acked(bool),
    Nacked(Option<NackOutcome>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NackOutcome {
    Retried { next_at: DateTime<Utc> },
    DeadLettered,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerError {
    #[error("queue `{0}` is not configured")]
    QueueMissing(String),
    #[error("queue `{0}` is disabled")]
    QueueDisabled(String),
    #[error("task `{0}` already exists")]
    TaskExists(String),
    #[error("task `{0}` was not found")]
    TaskMissing(String),
    #[error("attempt `{0}` was not found")]
    AttemptMissing(String),
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;
// HANDWRITE-END

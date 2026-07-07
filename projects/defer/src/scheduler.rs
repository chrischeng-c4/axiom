// SPEC-MANAGED: projects/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:defer-core-scheduler" tracker="#766" reason="In-memory delayed push-queue scheduler core."
use chrono::{DateTime, Duration, Utc};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use crate::types::{
    AttemptId, CreateTask, DispatchLease, NackOutcome, QueueControlState, QueueName, QueuePolicy,
    QueueSnapshot, SchedulerError, SchedulerResult, TaskId, TaskStatus,
};

const PRIORITY_BANDS: usize = u8::MAX as usize + 1;

#[derive(Debug, Clone)]
struct TaskRecord {
    queue: QueueName,
    create: CreateTask,
    created_seq: u64,
    attempts: u32,
    status: TaskStatus,
}

#[derive(Debug)]
struct QueueState {
    policy: QueuePolicy,
    control_state: QueueControlState,
    tasks: HashMap<TaskId, TaskRecord>,
    due: BinaryHeap<Reverse<(i64, u64, TaskId)>>,
    ready: Vec<BinaryHeap<Reverse<(u64, TaskId)>>>,
    in_flight: HashMap<AttemptId, TaskId>,
    next_seq: u64,
    next_attempt_seq: u64,
    dispatch_tokens_milli: u64,
    last_refill_at: Option<DateTime<Utc>>,
}

impl QueueState {
    fn new(policy: QueuePolicy) -> Self {
        Self {
            dispatch_tokens_milli: burst_tokens_milli(policy.max_burst_size),
            policy,
            control_state: QueueControlState::Running,
            tasks: HashMap::new(),
            due: BinaryHeap::new(),
            ready: (0..PRIORITY_BANDS).map(|_| BinaryHeap::new()).collect(),
            in_flight: HashMap::new(),
            next_seq: 0,
            next_attempt_seq: 0,
            last_refill_at: None,
        }
    }

    fn update_policy(&mut self, policy: QueuePolicy) {
        self.dispatch_tokens_milli = self
            .dispatch_tokens_milli
            .min(burst_tokens_milli(policy.max_burst_size));
        self.policy = policy;
    }

    fn in_flight_capacity(&self) -> usize {
        self.policy
            .max_in_flight
            .saturating_sub(self.in_flight.len())
    }

    fn push_due(&mut self, task_id: &str) {
        if let Some(task) = self.tasks.get(task_id) {
            self.due.push(Reverse((
                task.create.schedule_at.timestamp_millis(),
                task.created_seq,
                task_id.to_string(),
            )));
        }
    }

    fn promote_due(&mut self, now: DateTime<Utc>) {
        let cutoff = now.timestamp_millis();
        while let Some(Reverse((at, _, task_id))) = self.due.peek().cloned() {
            if at > cutoff {
                break;
            }
            self.due.pop();
            let Some(task) = self.tasks.get(&task_id) else {
                continue;
            };
            if task.create.schedule_at.timestamp_millis() != at {
                continue;
            }
            if matches!(task.status, TaskStatus::Scheduled) {
                self.ready[task.create.priority as usize]
                    .push(Reverse((task.created_seq, task_id)));
            }
        }
    }

    fn refill_dispatch_tokens(&mut self, now: DateTime<Utc>) {
        let Some(last_refill_at) = self.last_refill_at else {
            self.last_refill_at = Some(now);
            return;
        };
        if now <= last_refill_at {
            return;
        }
        let elapsed_ms = (now - last_refill_at).num_milliseconds().max(0) as u64;
        let added = elapsed_ms.saturating_mul(self.policy.max_dispatches_per_second as u64);
        self.dispatch_tokens_milli = self
            .dispatch_tokens_milli
            .saturating_add(added)
            .min(burst_tokens_milli(self.policy.max_burst_size));
        self.last_refill_at = Some(now);
    }

    fn rate_capacity(&self) -> usize {
        (self.dispatch_tokens_milli / 1_000) as usize
    }

    fn consume_dispatch_tokens(&mut self, count: usize) {
        self.dispatch_tokens_milli = self
            .dispatch_tokens_milli
            .saturating_sub((count as u64).saturating_mul(1_000));
    }
}

#[derive(Debug, Default)]
pub struct DeferScheduler {
    queues: HashMap<QueueName, QueueState>,
}

impl DeferScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure_queue(&mut self, queue: impl Into<String>, policy: QueuePolicy) {
        let queue = queue.into();
        if let Some(state) = self.queues.get_mut(&queue) {
            state.update_policy(policy);
        } else {
            self.queues.insert(queue, QueueState::new(policy));
        }
    }

    pub fn update_queue_policy(&mut self, queue: &str, policy: QueuePolicy) -> SchedulerResult<()> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        state.update_policy(policy);
        Ok(())
    }

    pub fn pause_queue(&mut self, queue: &str) -> SchedulerResult<()> {
        self.set_queue_control_state(queue, QueueControlState::Paused)
    }

    pub fn resume_queue(&mut self, queue: &str) -> SchedulerResult<()> {
        self.set_queue_control_state(queue, QueueControlState::Running)
    }

    pub fn disable_queue(&mut self, queue: &str) -> SchedulerResult<()> {
        self.set_queue_control_state(queue, QueueControlState::Disabled)
    }

    pub fn queue_snapshot(&self, queue: &str) -> SchedulerResult<QueueSnapshot> {
        let state = self
            .queues
            .get(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        let scheduled_count = state
            .tasks
            .values()
            .filter(|task| matches!(task.status, TaskStatus::Scheduled))
            .count();
        let terminal_count = state
            .tasks
            .values()
            .filter(|task| {
                matches!(
                    task.status,
                    TaskStatus::Succeeded | TaskStatus::DeadLettered | TaskStatus::Canceled
                )
            })
            .count();
        Ok(QueueSnapshot {
            queue: queue.to_string(),
            control_state: state.control_state,
            policy: state.policy.clone(),
            task_count: state.tasks.len(),
            scheduled_count,
            in_flight_count: state.in_flight.len(),
            terminal_count,
        })
    }

    pub fn create_task(
        &mut self,
        queue: impl Into<String>,
        task: CreateTask,
    ) -> SchedulerResult<()> {
        let queue = queue.into();
        let state = self
            .queues
            .get_mut(&queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.clone()))?;
        if state.control_state == QueueControlState::Disabled {
            return Err(SchedulerError::QueueDisabled(queue));
        }
        if state.tasks.contains_key(&task.task_id) {
            return Err(SchedulerError::TaskExists(task.task_id));
        }
        let created_seq = state.next_seq;
        state.next_seq += 1;
        let task_id = task.task_id.clone();
        state.tasks.insert(
            task_id.clone(),
            TaskRecord {
                queue,
                create: task,
                created_seq,
                attempts: 0,
                status: TaskStatus::Scheduled,
            },
        );
        state.push_due(&task_id);
        Ok(())
    }

    /// Return dispatch attempts for due tasks.
    ///
    /// Defer owns the consume rate: the caller can ask for many tasks, but this
    /// method caps the result by queue `max_dispatch_per_tick` and
    /// `max_in_flight`. ETA is evaluated before priority, then higher priority
    /// wins, and same-priority tasks use creation FIFO.
    pub fn lease_due(
        &mut self,
        queue: &str,
        now: DateTime<Utc>,
        requested: usize,
    ) -> SchedulerResult<Vec<DispatchLease>> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        if state.control_state != QueueControlState::Running {
            return Ok(Vec::new());
        }
        state.promote_due(now);
        state.refill_dispatch_tokens(now);
        let limit = requested
            .min(state.policy.max_dispatch_per_tick)
            .min(state.in_flight_capacity())
            .min(state.rate_capacity());
        let mut out = Vec::with_capacity(limit);
        for _ in 0..limit {
            let Some(task_id) = pick_ready(state) else {
                break;
            };
            let task = state
                .tasks
                .get_mut(&task_id)
                .expect("ready task must exist");
            task.attempts += 1;
            let attempt_id = format!("{}:{}:{}", task.queue, task_id, state.next_attempt_seq);
            state.next_attempt_seq += 1;
            let expires_at = now + Duration::milliseconds(state.policy.lease_ttl_ms as i64);
            task.status = TaskStatus::Leased {
                attempt_id: attempt_id.clone(),
                expires_at,
            };
            state.in_flight.insert(attempt_id.clone(), task_id.clone());
            out.push(DispatchLease {
                attempt_id,
                task_id: task_id.clone(),
                queue: task.queue.clone(),
                target: task.create.target.clone(),
                payload: task.create.payload.clone(),
                priority: task.create.priority,
                attempt: task.attempts,
                leased_at: now,
                expires_at,
            });
        }
        state.consume_dispatch_tokens(out.len());
        Ok(out)
    }

    pub fn ack(&mut self, queue: &str, attempt_id: &str) -> SchedulerResult<bool> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        let Some(task_id) = state.in_flight.remove(attempt_id) else {
            return Ok(false);
        };
        let Some(task) = state.tasks.get_mut(&task_id) else {
            return Err(SchedulerError::TaskMissing(task_id));
        };
        if matches!(&task.status, TaskStatus::Leased { attempt_id: live, .. } if live == attempt_id)
        {
            task.status = TaskStatus::Succeeded;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn nack(
        &mut self,
        queue: &str,
        attempt_id: &str,
        now: DateTime<Utc>,
    ) -> SchedulerResult<Option<NackOutcome>> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        let Some(task_id) = state.in_flight.remove(attempt_id) else {
            return Ok(None);
        };
        let Some(task) = state.tasks.get_mut(&task_id) else {
            return Err(SchedulerError::TaskMissing(task_id));
        };
        if !matches!(&task.status, TaskStatus::Leased { attempt_id: live, .. } if live == attempt_id)
        {
            return Ok(None);
        }
        if task.attempts >= task.create.max_attempts {
            task.status = TaskStatus::DeadLettered;
            return Ok(Some(NackOutcome::DeadLettered));
        }
        let next_at = now + retry_backoff(state.policy.retry_backoff_ms, task.attempts);
        task.create.schedule_at = next_at;
        task.status = TaskStatus::Scheduled;
        state.push_due(&task_id);
        Ok(Some(NackOutcome::Retried { next_at }))
    }

    pub fn reclaim_expired(
        &mut self,
        queue: &str,
        now: DateTime<Utc>,
    ) -> SchedulerResult<Vec<TaskId>> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        let expired: Vec<_> = state
            .tasks
            .iter()
            .filter_map(|(id, task)| match &task.status {
                TaskStatus::Leased {
                    attempt_id,
                    expires_at,
                } if *expires_at <= now => Some((id.clone(), attempt_id.clone())),
                _ => None,
            })
            .collect();
        for (task_id, attempt_id) in &expired {
            let _ = state.in_flight.remove(attempt_id);
            let Some(task) = state.tasks.get_mut(task_id) else {
                continue;
            };
            if task.attempts >= task.create.max_attempts {
                task.status = TaskStatus::DeadLettered;
            } else {
                task.create.schedule_at =
                    now + retry_backoff(state.policy.retry_backoff_ms, task.attempts);
                task.status = TaskStatus::Scheduled;
                state.push_due(task_id);
            }
        }
        Ok(expired.into_iter().map(|(task_id, _)| task_id).collect())
    }

    pub fn cancel(&mut self, queue: &str, task_id: &str) -> SchedulerResult<bool> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        let Some(task) = state.tasks.get_mut(task_id) else {
            return Ok(false);
        };
        match &task.status {
            TaskStatus::Succeeded | TaskStatus::DeadLettered | TaskStatus::Canceled => Ok(false),
            TaskStatus::Leased { attempt_id, .. } => {
                state.in_flight.remove(attempt_id);
                task.status = TaskStatus::Canceled;
                Ok(true)
            }
            TaskStatus::Scheduled => {
                task.status = TaskStatus::Canceled;
                Ok(true)
            }
        }
    }

    pub fn status(&self, queue: &str, task_id: &str) -> SchedulerResult<Option<TaskStatus>> {
        let state = self
            .queues
            .get(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        Ok(state.tasks.get(task_id).map(|t| t.status.clone()))
    }

    fn set_queue_control_state(
        &mut self,
        queue: &str,
        control_state: QueueControlState,
    ) -> SchedulerResult<()> {
        let state = self
            .queues
            .get_mut(queue)
            .ok_or_else(|| SchedulerError::QueueMissing(queue.to_string()))?;
        state.control_state = control_state;
        Ok(())
    }
}

fn pick_ready(state: &mut QueueState) -> Option<TaskId> {
    for priority in (0..PRIORITY_BANDS).rev() {
        while let Some(Reverse((_, task_id))) = state.ready[priority].pop() {
            let Some(task) = state.tasks.get(&task_id) else {
                continue;
            };
            if matches!(task.status, TaskStatus::Scheduled) {
                return Some(task_id);
            }
        }
    }
    None
}

fn retry_backoff(base_ms: u64, delivered_attempt: u32) -> Duration {
    let shift = delivered_attempt.saturating_sub(1).min(16);
    Duration::milliseconds(base_ms.saturating_mul(1u64 << shift) as i64)
}

fn burst_tokens_milli(max_burst_size: usize) -> u64 {
    (max_burst_size as u64).saturating_mul(1_000)
}
// HANDWRITE-END

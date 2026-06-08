//! Group - execute tasks in parallel

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{Broker, ResultBackend, TaskError, TaskId, TaskMessage, TaskState};
use super::{TaskOptions, TaskSignature};

/// A group of tasks executed in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Unique group ID
    pub id: TaskId,
    /// Tasks to execute in parallel
    pub tasks: Vec<TaskSignature>,
    /// Group-level options
    pub options: TaskOptions,
}

impl Group {
    /// Create a new group
    pub fn new(tasks: Vec<TaskSignature>) -> Self {
        Self {
            id: TaskId::new(),
            tasks,
            options: TaskOptions::default(),
        }
    }

    /// Create a group with options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Execute all tasks in parallel by publishing them to the broker
    pub async fn apply_async<B: Broker>(
        &self,
        broker: &B,
    ) -> Result<GroupResult, TaskError> {
        if self.tasks.is_empty() {
            return Err(TaskError::InvalidWorkflow(
                "Group must have at least one task".to_string(),
            ));
        }

        let mut task_ids = Vec::with_capacity(self.tasks.len());

        // Publish all tasks
        for task_sig in &self.tasks {
            let task_id = TaskId::new();
            let mut message = TaskMessage::new(task_sig.task_name.clone(), task_sig.args.clone())
                .with_kwargs(task_sig.kwargs.clone());

            message.id = task_id.clone();
            message.root_id = Some(self.id.clone());
            message.parent_id = Some(self.id.clone());

            // Apply options (task-level overrides group-level)
            if let Some(eta) = task_sig.options.eta.or(self.options.eta) {
                message.eta = Some(eta);
            }
            if let Some(expires) = task_sig.options.expires.or(self.options.expires) {
                message.expires = Some(expires);
            }

            // Determine target queue
            let queue = task_sig
                .options
                .queue
                .as_ref()
                .or(self.options.queue.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("default");

            // Publish task
            broker.publish(queue, message).await?;

            task_ids.push(task_id);
        }

        Ok(GroupResult {
            group_id: self.id.clone(),
            task_ids,
        })
    }
}

/// Handle to track group execution
#[derive(Debug, Clone)]
pub struct GroupResult {
    /// Group ID
    pub group_id: TaskId,
    /// Task IDs in the group
    pub task_ids: Vec<TaskId>,
}

impl GroupResult {
    /// Wait for all tasks to complete and return their results
    pub async fn get<R: ResultBackend>(
        &self,
        backend: &R,
        timeout: Option<Duration>,
    ) -> Result<Vec<serde_json::Value>, TaskError> {
        let results = backend.get_many(&self.task_ids).await?;

        let mut final_results = Vec::with_capacity(self.task_ids.len());

        for (i, result_opt) in results.iter().enumerate() {
            match result_opt {
                Some(result) => match result.state {
                    TaskState::Success => {
                        final_results.push(
                            result
                                .result
                                .clone()
                                .ok_or_else(|| {
                                    TaskError::Internal(format!(
                                        "Task {} succeeded but has no result",
                                        self.task_ids[i]
                                    ))
                                })?,
                        );
                    }
                    TaskState::Failure => {
                        return Err(TaskError::Internal(
                            result
                                .error
                                .clone()
                                .unwrap_or_else(|| format!("Task {} failed", self.task_ids[i])),
                        ));
                    }
                    other => {
                        // If not complete, wait for it
                        let poll_interval = Duration::from_millis(100);
                        let task_result = backend
                            .wait_for_result(&self.task_ids[i], timeout, poll_interval)
                            .await?;

                        match task_result.state {
                            TaskState::Success => {
                                final_results.push(
                                    task_result.result.ok_or_else(|| {
                                        TaskError::Internal(format!(
                                            "Task {} succeeded but has no result",
                                            self.task_ids[i]
                                        ))
                                    })?,
                                );
                            }
                            TaskState::Failure => {
                                return Err(TaskError::Internal(
                                    task_result.error.unwrap_or_else(|| {
                                        format!("Task {} failed", self.task_ids[i])
                                    }),
                                ));
                            }
                            _ => {
                                return Err(TaskError::Internal(format!(
                                    "Task {} in unexpected state: {:?}",
                                    self.task_ids[i], other
                                )));
                            }
                        }
                    }
                },
                None => {
                    // Task result not found, wait for it
                    let poll_interval = Duration::from_millis(100);
                    let task_result = backend
                        .wait_for_result(&self.task_ids[i], timeout, poll_interval)
                        .await?;

                    match task_result.state {
                        TaskState::Success => {
                            final_results.push(
                                task_result
                                    .result
                                    .ok_or_else(|| {
                                        TaskError::Internal(format!(
                                            "Task {} succeeded but has no result",
                                            self.task_ids[i]
                                        ))
                                    })?,
                            );
                        }
                        TaskState::Failure => {
                            return Err(TaskError::Internal(
                                task_result
                                    .error
                                    .unwrap_or_else(|| format!("Task {} failed", self.task_ids[i])),
                            ));
                        }
                        other => {
                            return Err(TaskError::Internal(format!(
                                "Task {} in unexpected state: {:?}",
                                self.task_ids[i], other
                            )));
                        }
                    }
                }
            }
        }

        Ok(final_results)
    }

    /// Check if all tasks are ready (completed)
    pub async fn ready<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        let states = backend.get_many(&self.task_ids).await?;

        for state_opt in states {
            match state_opt {
                Some(result) => {
                    if !result.state.is_terminal() {
                        return Ok(false);
                    }
                }
                None => return Ok(false),
            }
        }

        Ok(true)
    }

    /// Get all results that are currently available (non-blocking)
    pub async fn get_ready<R: ResultBackend>(
        &self,
        backend: &R,
    ) -> Result<Vec<Option<serde_json::Value>>, TaskError> {
        let results = backend.get_many(&self.task_ids).await?;

        let mut ready_results = Vec::with_capacity(self.task_ids.len());

        for result_opt in results {
            match result_opt {
                Some(result) => match result.state {
                    TaskState::Success => ready_results.push(result.result),
                    _ => ready_results.push(None),
                },
                None => ready_results.push(None),
            }
        }

        Ok(ready_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new() {
        let tasks = vec![
            TaskSignature::new("task1", serde_json::json!([1])),
            TaskSignature::new("task2", serde_json::json!([2])),
            TaskSignature::new("task3", serde_json::json!([3])),
        ];

        let group = Group::new(tasks.clone());
        assert_eq!(group.tasks.len(), 3);
        assert_eq!(group.tasks[0].task_name, "task1");
    }

    #[test]
    fn test_group_new_generates_unique_id() {
        let tasks = vec![TaskSignature::new("t", serde_json::json!([]))];
        let group_a = Group::new(tasks.clone());
        let group_b = Group::new(tasks);
        assert_ne!(group_a.id, group_b.id);
    }

    #[test]
    fn test_group_new_default_options() {
        let group = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        assert!(group.options.queue.is_none());
        assert!(group.options.countdown.is_none());
        assert!(group.options.eta.is_none());
        assert!(group.options.expires.is_none());
        assert!(group.options.retry_policy.is_none());
    }

    #[test]
    fn test_group_with_options() {
        let tasks = vec![TaskSignature::new("task1", serde_json::json!([]))];
        let options = TaskOptions::new().with_queue("bulk");

        let group = Group::new(tasks).with_options(options);
        assert_eq!(group.options.queue, Some("bulk".to_string()));
    }

    #[test]
    fn test_group_with_options_replaces_previous() {
        let tasks = vec![TaskSignature::new("t", serde_json::json!([]))];
        let group = Group::new(tasks)
            .with_options(TaskOptions::new().with_queue("first").with_countdown(30))
            .with_options(TaskOptions::new().with_queue("second"));
        assert_eq!(group.options.queue, Some("second".to_string()));
        // Fully replaced, not merged
        assert!(group.options.countdown.is_none());
    }

    #[test]
    fn test_group_with_options_preserves_tasks() {
        let tasks = vec![
            TaskSignature::new("a", serde_json::json!([1])),
            TaskSignature::new("b", serde_json::json!([2])),
        ];
        let group = Group::new(tasks).with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(group.tasks.len(), 2);
        assert_eq!(group.tasks[0].task_name, "a");
        assert_eq!(group.tasks[1].task_name, "b");
    }

    #[test]
    fn test_group_with_options_preserves_id() {
        let tasks = vec![TaskSignature::new("t", serde_json::json!([]))];
        let group = Group::new(tasks);
        let original_id = group.id.clone();
        let group = group.with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(group.id, original_id);
    }

    #[test]
    fn test_empty_group() {
        let group = Group::new(vec![]);
        assert!(group.tasks.is_empty());
    }

    #[test]
    fn test_single_task_group() {
        let group = Group::new(vec![
            TaskSignature::new("only_task", serde_json::json!(["data"])),
        ]);
        assert_eq!(group.tasks.len(), 1);
        assert_eq!(group.tasks[0].task_name, "only_task");
        assert_eq!(group.tasks[0].args, serde_json::json!(["data"]));
    }

    #[test]
    fn test_group_preserves_task_order() {
        let names = vec!["alpha", "beta", "gamma", "delta", "epsilon"];
        let tasks: Vec<TaskSignature> = names
            .iter()
            .enumerate()
            .map(|(i, n)| TaskSignature::new(*n, serde_json::json!([i])))
            .collect();
        let group = Group::new(tasks);
        for (i, name) in names.iter().enumerate() {
            assert_eq!(group.tasks[i].task_name, *name);
        }
    }

    #[test]
    fn test_group_tasks_with_kwargs() {
        let sig = TaskSignature::new("task1", serde_json::json!([1]))
            .with_kwargs(serde_json::json!({"key": "value", "n": 42}));
        let group = Group::new(vec![sig]);
        assert_eq!(
            group.tasks[0].kwargs,
            serde_json::json!({"key": "value", "n": 42})
        );
    }

    #[test]
    fn test_group_tasks_with_individual_options() {
        let sig = TaskSignature::new("task1", serde_json::json!([]))
            .set_queue("task-specific-queue");
        let group = Group::new(vec![sig])
            .with_options(TaskOptions::new().with_queue("group-queue"));
        // Task-level and group-level queues are independent
        assert_eq!(
            group.tasks[0].options.queue,
            Some("task-specific-queue".to_string())
        );
        assert_eq!(group.options.queue, Some("group-queue".to_string()));
    }

    #[test]
    fn test_group_tasks_mixed_options() {
        let tasks = vec![
            TaskSignature::new("t1", serde_json::json!([])).set_queue("q1"),
            TaskSignature::new("t2", serde_json::json!([])), // no queue override
            TaskSignature::new("t3", serde_json::json!([])).set_queue("q3"),
        ];
        let group = Group::new(tasks).with_options(TaskOptions::new().with_queue("default-q"));
        assert_eq!(group.tasks[0].options.queue, Some("q1".to_string()));
        assert!(group.tasks[1].options.queue.is_none());
        assert_eq!(group.tasks[2].options.queue, Some("q3".to_string()));
        assert_eq!(group.options.queue, Some("default-q".to_string()));
    }

    #[test]
    fn test_group_serde_roundtrip() {
        let tasks = vec![
            TaskSignature::new("fetch", serde_json::json!(["url1"]))
                .with_kwargs(serde_json::json!({"timeout": 10})),
            TaskSignature::new("parse", serde_json::json!(["html"])),
            TaskSignature::new("store", serde_json::json!([42, true])),
        ];
        let group = Group::new(tasks).with_options(TaskOptions::new().with_queue("io"));

        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: Group = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, group.id);
        assert_eq!(deserialized.tasks.len(), 3);
        assert_eq!(deserialized.tasks[0].task_name, "fetch");
        assert_eq!(deserialized.tasks[0].args, serde_json::json!(["url1"]));
        assert_eq!(
            deserialized.tasks[0].kwargs,
            serde_json::json!({"timeout": 10})
        );
        assert_eq!(deserialized.tasks[1].task_name, "parse");
        assert_eq!(deserialized.tasks[2].task_name, "store");
        assert_eq!(deserialized.tasks[2].args, serde_json::json!([42, true]));
        assert_eq!(deserialized.options.queue, Some("io".to_string()));
    }

    #[test]
    fn test_group_serde_roundtrip_empty() {
        let group = Group::new(vec![]);
        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: Group = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, group.id);
        assert!(deserialized.tasks.is_empty());
    }

    #[test]
    fn test_group_serde_roundtrip_default_options() {
        let group = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: Group = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.options.queue.is_none());
        assert!(deserialized.options.countdown.is_none());
    }

    #[test]
    fn test_group_serde_json_value_roundtrip() {
        let group = Group::new(vec![
            TaskSignature::new("t1", serde_json::json!([1])),
            TaskSignature::new("t2", serde_json::json!([2])),
        ]);
        let value = serde_json::to_value(&group).expect("to_value");
        assert!(value.is_object());
        assert!(value.get("id").is_some());
        assert!(value.get("tasks").unwrap().is_array());
        assert_eq!(value.get("tasks").unwrap().as_array().unwrap().len(), 2);
        let back: Group = serde_json::from_value(value).expect("from_value");
        assert_eq!(back.id, group.id);
        assert_eq!(back.tasks.len(), 2);
    }

    #[test]
    fn test_group_clone() {
        let group = Group::new(vec![
            TaskSignature::new("t1", serde_json::json!([1])),
            TaskSignature::new("t2", serde_json::json!([2])),
        ])
        .with_options(TaskOptions::new().with_queue("q"));
        let cloned = group.clone();
        assert_eq!(cloned.id, group.id);
        assert_eq!(cloned.tasks.len(), group.tasks.len());
        assert_eq!(cloned.options.queue, group.options.queue);
    }

    #[test]
    fn test_group_many_tasks() {
        let tasks: Vec<TaskSignature> = (0..100)
            .map(|i| TaskSignature::new(format!("worker_{}", i), serde_json::json!([i])))
            .collect();
        let group = Group::new(tasks);
        assert_eq!(group.tasks.len(), 100);
        assert_eq!(group.tasks[0].task_name, "worker_0");
        assert_eq!(group.tasks[99].task_name, "worker_99");
    }

    #[test]
    fn test_group_result_creation() {
        let group_id = TaskId::new();
        let task_ids = vec![TaskId::new(), TaskId::new()];

        let result = GroupResult {
            group_id: group_id.clone(),
            task_ids: task_ids.clone(),
        };

        assert_eq!(result.group_id, group_id);
        assert_eq!(result.task_ids.len(), 2);
    }

    #[test]
    fn test_group_result_clone() {
        let result = GroupResult {
            group_id: TaskId::new(),
            task_ids: vec![TaskId::new(), TaskId::new(), TaskId::new()],
        };
        let cloned = result.clone();
        assert_eq!(cloned.group_id, result.group_id);
        assert_eq!(cloned.task_ids.len(), result.task_ids.len());
        for (a, b) in cloned.task_ids.iter().zip(result.task_ids.iter()) {
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_group_result_empty_task_ids() {
        let result = GroupResult {
            group_id: TaskId::new(),
            task_ids: vec![],
        };
        assert!(result.task_ids.is_empty());
    }

    #[test]
    fn test_group_result_single_task_id() {
        let task_id = TaskId::new();
        let result = GroupResult {
            group_id: TaskId::new(),
            task_ids: vec![task_id.clone()],
        };
        assert_eq!(result.task_ids.len(), 1);
        assert_eq!(result.task_ids[0], task_id);
    }

    #[test]
    fn test_group_with_immutable_tasks() {
        let tasks = vec![
            TaskSignature::new("t1", serde_json::json!([1])).immutable(),
            TaskSignature::new("t2", serde_json::json!([2])),
        ];
        let group = Group::new(tasks);
        assert!(group.tasks[0].immutable);
        assert!(!group.tasks[1].immutable);
    }

    #[test]
    fn test_group_serde_with_complex_args() {
        let tasks = vec![
            TaskSignature::new("process", serde_json::json!([
                {"nested": {"deep": [1, 2, 3]}},
                null,
                "string_arg",
                42.5
            ])),
        ];
        let group = Group::new(tasks);

        let json = serde_json::to_string(&group).expect("serialize");
        let deserialized: Group = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.tasks[0].args, group.tasks[0].args);
    }
}

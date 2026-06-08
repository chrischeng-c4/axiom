//! Chain - execute tasks sequentially, passing results

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{Broker, ResultBackend, TaskError, TaskId, TaskMessage, TaskState};
use super::{ChainMeta, TaskOptions, TaskSignature};

/// A chain of tasks executed sequentially
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    /// Unique chain ID
    pub id: TaskId,
    /// Tasks to execute in order
    pub tasks: Vec<TaskSignature>,
    /// Chain-level options
    pub options: TaskOptions,
}

impl Chain {
    /// Create a new chain
    pub fn new(tasks: Vec<TaskSignature>) -> Self {
        Self {
            id: TaskId::new(),
            tasks,
            options: TaskOptions::default(),
        }
    }

    /// Create a chain with options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Execute the chain by publishing the first task
    ///
    /// The chain metadata is stored in the result backend for the worker to continue
    /// execution after each task completes.
    pub async fn apply_async<B: Broker>(
        &self,
        broker: &B,
    ) -> Result<AsyncChainResult, TaskError> {
        if self.tasks.is_empty() {
            return Err(TaskError::InvalidWorkflow(
                "Chain must have at least one task".to_string(),
            ));
        }

        let first_task = &self.tasks[0];
        let first_task_id = TaskId::new();
        let last_task_id = if self.tasks.len() == 1 {
            first_task_id.clone()
        } else {
            TaskId::new() // Placeholder, actual ID will be assigned during execution
        };

        // Create task message for the first task
        let mut message = TaskMessage::new(first_task.task_name.clone(), first_task.args.clone())
            .with_kwargs(first_task.kwargs.clone());

        message.id = first_task_id.clone();
        message.root_id = Some(self.id.clone());

        // Apply options
        if let Some(eta) = first_task.options.eta.or(self.options.eta) {
            message.eta = Some(eta);
        }
        if let Some(expires) = first_task.options.expires.or(self.options.expires) {
            message.expires = Some(expires);
        }

        // Determine target queue
        let queue = first_task
            .options
            .queue
            .as_ref()
            .or(self.options.queue.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("default");

        // Publish first task
        broker.publish(queue, message).await?;

        Ok(AsyncChainResult {
            chain_id: self.id.clone(),
            first_task_id,
            last_task_id,
        })
    }

    /// Get chain metadata for storage
    ///
    /// Returns the serialized chain metadata that can be stored in the backend.
    /// This is called internally by apply_async and also by workers when
    /// processing chain tasks.
    pub fn get_metadata(&self) -> Result<(String, String), TaskError> {
        let meta = ChainMeta::new(self.id.clone(), self.tasks.clone());
        let key = format!("chain:{}", self.id);
        let data = serde_json::to_string(&meta)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;
        Ok((key, data))
    }
}

/// Handle to track chain execution
#[derive(Debug, Clone)]
pub struct AsyncChainResult {
    /// Chain ID
    pub chain_id: TaskId,
    /// ID of the first task in the chain
    pub first_task_id: TaskId,
    /// ID of the last task in the chain
    pub last_task_id: TaskId,
}

impl AsyncChainResult {
    /// Wait for the final result of the chain
    ///
    /// This polls the result backend until the last task completes or timeout occurs.
    pub async fn get<R: ResultBackend>(
        &self,
        backend: &R,
        timeout: Option<Duration>,
    ) -> Result<serde_json::Value, TaskError> {
        let poll_interval = Duration::from_millis(100);
        let result = backend
            .wait_for_result(&self.last_task_id, timeout, poll_interval)
            .await?;

        match result.state {
            TaskState::Success => result
                .result
                .ok_or_else(|| TaskError::Internal("Success state but no result".to_string())),
            TaskState::Failure => Err(TaskError::Internal(
                result.error.unwrap_or_else(|| "Task failed".to_string()),
            )),
            other => Err(TaskError::Internal(format!(
                "Unexpected task state: {:?}",
                other
            ))),
        }
    }

    /// Check if the chain is ready (last task completed)
    pub async fn ready<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        match backend.get_state(&self.last_task_id).await? {
            Some(state) => Ok(state.is_terminal()),
            None => Ok(false),
        }
    }

    /// Get the current state of the chain
    pub async fn state<R: ResultBackend>(&self, backend: &R) -> Result<Option<TaskState>, TaskError> {
        backend.get_state(&self.last_task_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_new() {
        let tasks = vec![
            TaskSignature::new("task1", serde_json::json!([1])),
            TaskSignature::new("task2", serde_json::json!([2])),
        ];

        let chain = Chain::new(tasks.clone());
        assert_eq!(chain.tasks.len(), 2);
        assert_eq!(chain.tasks[0].task_name, "task1");
        assert_eq!(chain.tasks[1].task_name, "task2");
    }

    #[test]
    fn test_chain_new_generates_unique_id() {
        let tasks = vec![TaskSignature::new("task1", serde_json::json!([]))];
        let chain_a = Chain::new(tasks.clone());
        let chain_b = Chain::new(tasks);
        assert_ne!(chain_a.id, chain_b.id);
    }

    #[test]
    fn test_chain_new_default_options() {
        let chain = Chain::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        assert!(chain.options.queue.is_none());
        assert!(chain.options.countdown.is_none());
        assert!(chain.options.eta.is_none());
        assert!(chain.options.expires.is_none());
        assert!(chain.options.retry_policy.is_none());
    }

    #[test]
    fn test_chain_with_options() {
        let tasks = vec![TaskSignature::new("task1", serde_json::json!([]))];
        let options = TaskOptions::new().with_queue("priority");

        let chain = Chain::new(tasks).with_options(options);
        assert_eq!(chain.options.queue, Some("priority".to_string()));
    }

    #[test]
    fn test_chain_with_options_replaces_previous() {
        let tasks = vec![TaskSignature::new("task1", serde_json::json!([]))];
        let chain = Chain::new(tasks)
            .with_options(TaskOptions::new().with_queue("first"))
            .with_options(TaskOptions::new().with_queue("second"));
        assert_eq!(chain.options.queue, Some("second".to_string()));
        // The first options should be fully replaced, not merged
        assert!(chain.options.countdown.is_none());
    }

    #[test]
    fn test_chain_with_options_preserves_tasks() {
        let tasks = vec![
            TaskSignature::new("a", serde_json::json!([1])),
            TaskSignature::new("b", serde_json::json!([2])),
        ];
        let chain = Chain::new(tasks).with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(chain.tasks.len(), 2);
        assert_eq!(chain.tasks[0].task_name, "a");
        assert_eq!(chain.tasks[1].task_name, "b");
    }

    #[test]
    fn test_chain_with_options_preserves_id() {
        let tasks = vec![TaskSignature::new("t", serde_json::json!([]))];
        let chain = Chain::new(tasks);
        let original_id = chain.id.clone();
        let chain = chain.with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(chain.id, original_id);
    }

    #[test]
    fn test_empty_chain() {
        let chain = Chain::new(vec![]);
        assert!(chain.tasks.is_empty());
    }

    #[test]
    fn test_single_task_chain() {
        let chain = Chain::new(vec![
            TaskSignature::new("only_task", serde_json::json!(["arg1"])),
        ]);
        assert_eq!(chain.tasks.len(), 1);
        assert_eq!(chain.tasks[0].task_name, "only_task");
        assert_eq!(chain.tasks[0].args, serde_json::json!(["arg1"]));
    }

    #[test]
    fn test_chain_preserves_task_order() {
        let names: Vec<&str> = vec!["first", "second", "third", "fourth", "fifth"];
        let tasks: Vec<TaskSignature> = names
            .iter()
            .enumerate()
            .map(|(i, name)| TaskSignature::new(*name, serde_json::json!([i])))
            .collect();
        let chain = Chain::new(tasks);
        for (i, name) in names.iter().enumerate() {
            assert_eq!(chain.tasks[i].task_name, *name);
        }
    }

    #[test]
    fn test_chain_tasks_with_kwargs() {
        let sig = TaskSignature::new("task1", serde_json::json!([1]))
            .with_kwargs(serde_json::json!({"key": "value"}));
        let chain = Chain::new(vec![sig]);
        assert_eq!(
            chain.tasks[0].kwargs,
            serde_json::json!({"key": "value"})
        );
    }

    #[test]
    fn test_chain_tasks_with_individual_options() {
        let sig = TaskSignature::new("task1", serde_json::json!([]))
            .set_queue("task-specific-queue");
        let chain = Chain::new(vec![sig]).with_options(TaskOptions::new().with_queue("chain-queue"));
        // Task-level queue should be preserved independently of chain-level
        assert_eq!(
            chain.tasks[0].options.queue,
            Some("task-specific-queue".to_string())
        );
        assert_eq!(chain.options.queue, Some("chain-queue".to_string()));
    }

    #[test]
    fn test_chain_serde_roundtrip() {
        let tasks = vec![
            TaskSignature::new("task1", serde_json::json!([1, "hello"])),
            TaskSignature::new("task2", serde_json::json!([2, true]))
                .with_kwargs(serde_json::json!({"x": 42})),
        ];
        let chain = Chain::new(tasks).with_options(TaskOptions::new().with_queue("myqueue"));

        let json = serde_json::to_string(&chain).expect("serialize");
        let deserialized: Chain = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, chain.id);
        assert_eq!(deserialized.tasks.len(), 2);
        assert_eq!(deserialized.tasks[0].task_name, "task1");
        assert_eq!(deserialized.tasks[0].args, serde_json::json!([1, "hello"]));
        assert_eq!(deserialized.tasks[1].task_name, "task2");
        assert_eq!(deserialized.tasks[1].kwargs, serde_json::json!({"x": 42}));
        assert_eq!(deserialized.options.queue, Some("myqueue".to_string()));
    }

    #[test]
    fn test_chain_serde_roundtrip_empty() {
        let chain = Chain::new(vec![]);
        let json = serde_json::to_string(&chain).expect("serialize");
        let deserialized: Chain = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, chain.id);
        assert!(deserialized.tasks.is_empty());
    }

    #[test]
    fn test_chain_serde_roundtrip_default_options() {
        let chain = Chain::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let json = serde_json::to_string(&chain).expect("serialize");
        let deserialized: Chain = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.options.queue.is_none());
        assert!(deserialized.options.countdown.is_none());
    }

    #[test]
    fn test_chain_serde_json_value_roundtrip() {
        let chain = Chain::new(vec![
            TaskSignature::new("add", serde_json::json!([1, 2])),
        ]);
        let value = serde_json::to_value(&chain).expect("to_value");
        assert!(value.is_object());
        assert!(value.get("id").is_some());
        assert!(value.get("tasks").unwrap().is_array());
        let back: Chain = serde_json::from_value(value).expect("from_value");
        assert_eq!(back.id, chain.id);
    }

    #[test]
    fn test_chain_get_metadata() {
        let tasks = vec![
            TaskSignature::new("step1", serde_json::json!(["a"])),
            TaskSignature::new("step2", serde_json::json!(["b"])),
        ];
        let chain = Chain::new(tasks);
        let (key, data) = chain.get_metadata().expect("get_metadata should succeed");

        assert!(key.starts_with("chain:"));
        assert!(key.contains(&chain.id.to_string()));

        // Data should be valid JSON containing chain meta
        let meta: ChainMeta = serde_json::from_str(&data).expect("parse metadata");
        assert_eq!(meta.chain_id, chain.id);
        assert_eq!(meta.tasks.len(), 2);
        assert_eq!(meta.current_index, 0);
        assert!(meta.results.is_empty());
    }

    #[test]
    fn test_chain_get_metadata_single_task() {
        let chain = Chain::new(vec![
            TaskSignature::new("only", serde_json::json!([])),
        ]);
        let (_, data) = chain.get_metadata().expect("get_metadata");
        let meta: ChainMeta = serde_json::from_str(&data).expect("parse");
        assert_eq!(meta.tasks.len(), 1);
        assert_eq!(meta.tasks[0].task_name, "only");
    }

    #[test]
    fn test_chain_clone() {
        let chain = Chain::new(vec![
            TaskSignature::new("t1", serde_json::json!([1])),
        ])
        .with_options(TaskOptions::new().with_queue("q"));
        let cloned = chain.clone();
        assert_eq!(cloned.id, chain.id);
        assert_eq!(cloned.tasks.len(), chain.tasks.len());
        assert_eq!(cloned.options.queue, chain.options.queue);
    }

    #[test]
    fn test_async_chain_result_creation() {
        let chain_id = TaskId::new();
        let first_task_id = TaskId::new();
        let last_task_id = TaskId::new();

        let result = AsyncChainResult {
            chain_id: chain_id.clone(),
            first_task_id: first_task_id.clone(),
            last_task_id: last_task_id.clone(),
        };

        assert_eq!(result.chain_id, chain_id);
        assert_eq!(result.first_task_id, first_task_id);
        assert_eq!(result.last_task_id, last_task_id);
    }

    #[test]
    fn test_async_chain_result_clone() {
        let result = AsyncChainResult {
            chain_id: TaskId::new(),
            first_task_id: TaskId::new(),
            last_task_id: TaskId::new(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.chain_id, result.chain_id);
        assert_eq!(cloned.first_task_id, result.first_task_id);
        assert_eq!(cloned.last_task_id, result.last_task_id);
    }

    #[test]
    fn test_async_chain_result_same_first_and_last() {
        // Single-task chain: first and last are the same
        let task_id = TaskId::new();
        let result = AsyncChainResult {
            chain_id: TaskId::new(),
            first_task_id: task_id.clone(),
            last_task_id: task_id.clone(),
        };
        assert_eq!(result.first_task_id, result.last_task_id);
    }

    #[test]
    fn test_chain_many_tasks() {
        let tasks: Vec<TaskSignature> = (0..100)
            .map(|i| TaskSignature::new(format!("task_{}", i), serde_json::json!([i])))
            .collect();
        let chain = Chain::new(tasks);
        assert_eq!(chain.tasks.len(), 100);
        assert_eq!(chain.tasks[0].task_name, "task_0");
        assert_eq!(chain.tasks[99].task_name, "task_99");
    }

    #[test]
    fn test_chain_with_immutable_tasks() {
        let tasks = vec![
            TaskSignature::new("step1", serde_json::json!([1])),
            TaskSignature::new("step2", serde_json::json!([2])).immutable(),
            TaskSignature::new("step3", serde_json::json!([3])),
        ];
        let chain = Chain::new(tasks);
        assert!(!chain.tasks[0].immutable);
        assert!(chain.tasks[1].immutable);
        assert!(!chain.tasks[2].immutable);
    }
}

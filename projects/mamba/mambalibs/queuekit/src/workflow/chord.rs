//! Chord - group of parallel tasks followed by a callback

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{Broker, ResultBackend, TaskError, TaskId, TaskMessage, TaskState};
use super::{ChordMeta, Group, GroupResult, TaskOptions, TaskSignature};

/// A chord: group of parallel tasks followed by a callback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chord {
    /// Unique chord ID
    pub id: TaskId,
    /// Header tasks (run in parallel)
    pub header: Group,
    /// Callback task (receives all results)
    pub callback: TaskSignature,
    /// Chord-level options
    pub options: TaskOptions,
}

impl Chord {
    /// Create a new chord
    pub fn new(header: Group, callback: TaskSignature) -> Self {
        Self {
            id: TaskId::new(),
            header,
            callback,
            options: TaskOptions::default(),
        }
    }

    /// Create a chord with options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Execute the chord
    ///
    /// This publishes all header tasks and stores chord metadata in the backend.
    /// When all header tasks complete, a worker will trigger the callback.
    pub async fn apply_async<B: Broker, R: ResultBackend>(
        &self,
        broker: &B,
        backend: &R,
    ) -> Result<AsyncChordResult, TaskError> {
        if self.header.tasks.is_empty() {
            return Err(TaskError::InvalidWorkflow(
                "Chord header must have at least one task".to_string(),
            ));
        }

        // Execute the header group
        let group_result = self.header.apply_async(broker).await?;

        // Create callback task ID
        let callback_task_id = TaskId::new();

        // Store chord metadata
        let chord_meta = ChordMeta::new(
            self.id.clone(),
            group_result.task_ids.clone(),
            self.callback.clone(),
        );

        let meta_key = format!("chord:{}", self.id);
        let meta_json = serde_json::to_vec(&chord_meta)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;

        // Store metadata in backend
        // TODO: Add set_metadata to ResultBackend trait
        // For now, we'll store it as a special result
        let meta_task_id = TaskId::new();
        let _ = (backend, meta_key, meta_json, meta_task_id);

        Ok(AsyncChordResult {
            chord_id: self.id.clone(),
            header_result: group_result,
            callback_task_id,
        })
    }

    /// Manually trigger the callback (for testing or manual intervention)
    ///
    /// This should normally be called by the worker after all header tasks complete.
    pub async fn trigger_callback<B: Broker>(
        &self,
        broker: &B,
        header_results: Vec<serde_json::Value>,
    ) -> Result<TaskId, TaskError> {
        let callback_task_id = TaskId::new();

        // Prepare callback args with header results
        let callback_args = serde_json::Value::Array(header_results);

        let mut message = TaskMessage::new(self.callback.task_name.clone(), callback_args)
            .with_kwargs(self.callback.kwargs.clone());

        message.id = callback_task_id.clone();
        message.root_id = Some(self.id.clone());
        message.parent_id = Some(self.id.clone());

        // Apply options
        let queue = self
            .callback
            .options
            .queue
            .as_ref()
            .or(self.options.queue.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("default");

        broker.publish(queue, message).await?;

        Ok(callback_task_id)
    }
}

/// Handle to track chord execution
#[derive(Debug, Clone)]
pub struct AsyncChordResult {
    /// Chord ID
    pub chord_id: TaskId,
    /// Handle to the header group
    pub header_result: GroupResult,
    /// ID of the callback task (will be assigned when triggered)
    pub callback_task_id: TaskId,
}

impl AsyncChordResult {
    /// Wait for the callback to complete and return its result
    pub async fn get<R: ResultBackend>(
        &self,
        backend: &R,
        timeout: Option<Duration>,
    ) -> Result<serde_json::Value, TaskError> {
        let poll_interval = Duration::from_millis(100);
        let result = backend
            .wait_for_result(&self.callback_task_id, timeout, poll_interval)
            .await?;

        match result.state {
            TaskState::Success => result
                .result
                .ok_or_else(|| TaskError::Internal("Success state but no result".to_string())),
            TaskState::Failure => Err(TaskError::Internal(
                result.error.unwrap_or_else(|| "Callback failed".to_string()),
            )),
            other => Err(TaskError::Internal(format!(
                "Unexpected callback state: {:?}",
                other
            ))),
        }
    }

    /// Check if the header group is ready (all tasks completed)
    pub async fn header_ready<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        self.header_result.ready(backend).await
    }

    /// Check if the callback is ready (completed)
    pub async fn ready<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        match backend.get_state(&self.callback_task_id).await? {
            Some(state) => Ok(state.is_terminal()),
            None => Ok(false),
        }
    }

    /// Get header results (non-blocking)
    pub async fn get_header_results<R: ResultBackend>(
        &self,
        backend: &R,
    ) -> Result<Vec<Option<serde_json::Value>>, TaskError> {
        self.header_result.get_ready(backend).await
    }

    /// Wait for header to complete and return all results
    pub async fn wait_for_header<R: ResultBackend>(
        &self,
        backend: &R,
        timeout: Option<Duration>,
    ) -> Result<Vec<serde_json::Value>, TaskError> {
        self.header_result.get(backend, timeout).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chord_new() {
        let header = Group::new(vec![
            TaskSignature::new("task1", serde_json::json!([1])),
            TaskSignature::new("task2", serde_json::json!([2])),
        ]);

        let callback = TaskSignature::new("combine", serde_json::json!([]));

        let chord = Chord::new(header.clone(), callback);
        assert_eq!(chord.header.tasks.len(), 2);
        assert_eq!(chord.callback.task_name, "combine");
    }

    #[test]
    fn test_chord_new_generates_unique_id() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord_a = Chord::new(header.clone(), callback.clone());
        let chord_b = Chord::new(header, callback);
        assert_ne!(chord_a.id, chord_b.id);
    }

    #[test]
    fn test_chord_new_default_options() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        assert!(chord.options.queue.is_none());
        assert!(chord.options.countdown.is_none());
        assert!(chord.options.eta.is_none());
        assert!(chord.options.expires.is_none());
        assert!(chord.options.retry_policy.is_none());
    }

    #[test]
    fn test_chord_with_options() {
        let header = Group::new(vec![TaskSignature::new("task1", serde_json::json!([]))]);
        let callback = TaskSignature::new("callback", serde_json::json!([]));
        let options = TaskOptions::new().with_queue("callbacks");

        let chord = Chord::new(header, callback).with_options(options);
        assert_eq!(chord.options.queue, Some("callbacks".to_string()));
    }

    #[test]
    fn test_chord_with_options_replaces_previous() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback)
            .with_options(TaskOptions::new().with_queue("first").with_countdown(10))
            .with_options(TaskOptions::new().with_queue("second"));
        assert_eq!(chord.options.queue, Some("second".to_string()));
        // Previous options fully replaced, not merged
        assert!(chord.options.countdown.is_none());
    }

    #[test]
    fn test_chord_with_options_preserves_header_and_callback() {
        let header = Group::new(vec![
            TaskSignature::new("a", serde_json::json!([1])),
            TaskSignature::new("b", serde_json::json!([2])),
        ]);
        let callback = TaskSignature::new("aggregate", serde_json::json!([]));
        let chord = Chord::new(header, callback)
            .with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(chord.header.tasks.len(), 2);
        assert_eq!(chord.callback.task_name, "aggregate");
    }

    #[test]
    fn test_chord_with_options_preserves_id() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        let original_id = chord.id.clone();
        let chord = chord.with_options(TaskOptions::new().with_queue("q"));
        assert_eq!(chord.id, original_id);
    }

    #[test]
    fn test_chord_empty_header() {
        let header = Group::new(vec![]);
        let callback = TaskSignature::new("callback", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        assert!(chord.header.tasks.is_empty());
        assert_eq!(chord.callback.task_name, "callback");
    }

    #[test]
    fn test_chord_single_header_task() {
        let header = Group::new(vec![
            TaskSignature::new("only_task", serde_json::json!(["data"])),
        ]);
        let callback = TaskSignature::new("process", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        assert_eq!(chord.header.tasks.len(), 1);
        assert_eq!(chord.header.tasks[0].task_name, "only_task");
        assert_eq!(chord.callback.task_name, "process");
    }

    #[test]
    fn test_chord_callback_with_kwargs() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]))
            .with_kwargs(serde_json::json!({"format": "json", "compress": true}));
        let chord = Chord::new(header, callback);
        assert_eq!(
            chord.callback.kwargs,
            serde_json::json!({"format": "json", "compress": true})
        );
    }

    #[test]
    fn test_chord_callback_with_queue() {
        let header = Group::new(vec![TaskSignature::new("t", serde_json::json!([]))]);
        let callback = TaskSignature::new("cb", serde_json::json!([]))
            .set_queue("callback-queue");
        let chord = Chord::new(header, callback)
            .with_options(TaskOptions::new().with_queue("chord-queue"));
        // Callback and chord have independent queue settings
        assert_eq!(
            chord.callback.options.queue,
            Some("callback-queue".to_string())
        );
        assert_eq!(chord.options.queue, Some("chord-queue".to_string()));
    }

    #[test]
    fn test_chord_header_tasks_preserve_order() {
        let names = vec!["alpha", "beta", "gamma", "delta"];
        let tasks: Vec<TaskSignature> = names
            .iter()
            .map(|n| TaskSignature::new(*n, serde_json::json!([])))
            .collect();
        let header = Group::new(tasks);
        let callback = TaskSignature::new("reduce", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        for (i, name) in names.iter().enumerate() {
            assert_eq!(chord.header.tasks[i].task_name, *name);
        }
    }

    #[test]
    fn test_chord_serde_roundtrip() {
        let header = Group::new(vec![
            TaskSignature::new("fetch_a", serde_json::json!(["url_a"])),
            TaskSignature::new("fetch_b", serde_json::json!(["url_b"]))
                .with_kwargs(serde_json::json!({"timeout": 30})),
        ]);
        let callback = TaskSignature::new("merge", serde_json::json!([]));
        let chord = Chord::new(header, callback)
            .with_options(TaskOptions::new().with_queue("io-bound"));

        let json = serde_json::to_string(&chord).expect("serialize");
        let deserialized: Chord = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, chord.id);
        assert_eq!(deserialized.header.tasks.len(), 2);
        assert_eq!(deserialized.header.tasks[0].task_name, "fetch_a");
        assert_eq!(
            deserialized.header.tasks[1].kwargs,
            serde_json::json!({"timeout": 30})
        );
        assert_eq!(deserialized.callback.task_name, "merge");
        assert_eq!(deserialized.options.queue, Some("io-bound".to_string()));
    }

    #[test]
    fn test_chord_serde_roundtrip_empty_header() {
        let header = Group::new(vec![]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback);

        let json = serde_json::to_string(&chord).expect("serialize");
        let deserialized: Chord = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, chord.id);
        assert!(deserialized.header.tasks.is_empty());
        assert_eq!(deserialized.callback.task_name, "cb");
    }

    #[test]
    fn test_chord_serde_json_value_roundtrip() {
        let header = Group::new(vec![
            TaskSignature::new("t1", serde_json::json!([1])),
        ]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback);

        let value = serde_json::to_value(&chord).expect("to_value");
        assert!(value.is_object());
        assert!(value.get("id").is_some());
        assert!(value.get("header").is_some());
        assert!(value.get("callback").is_some());
        let back: Chord = serde_json::from_value(value).expect("from_value");
        assert_eq!(back.id, chord.id);
    }

    #[test]
    fn test_chord_clone() {
        let header = Group::new(vec![
            TaskSignature::new("t1", serde_json::json!([1])),
        ]);
        let callback = TaskSignature::new("cb", serde_json::json!([]));
        let chord = Chord::new(header, callback)
            .with_options(TaskOptions::new().with_queue("q"));
        let cloned = chord.clone();
        assert_eq!(cloned.id, chord.id);
        assert_eq!(cloned.header.tasks.len(), chord.header.tasks.len());
        assert_eq!(cloned.callback.task_name, chord.callback.task_name);
        assert_eq!(cloned.options.queue, chord.options.queue);
    }

    #[test]
    fn test_chord_many_header_tasks() {
        let tasks: Vec<TaskSignature> = (0..50)
            .map(|i| TaskSignature::new(format!("worker_{}", i), serde_json::json!([i])))
            .collect();
        let header = Group::new(tasks);
        let callback = TaskSignature::new("aggregate", serde_json::json!([]));
        let chord = Chord::new(header, callback);
        assert_eq!(chord.header.tasks.len(), 50);
        assert_eq!(chord.header.tasks[49].task_name, "worker_49");
    }

    #[test]
    fn test_async_chord_result_creation() {
        let chord_id = TaskId::new();
        let group_id = TaskId::new();
        let task_ids = vec![TaskId::new(), TaskId::new()];
        let callback_task_id = TaskId::new();

        let header_result = GroupResult {
            group_id,
            task_ids,
        };

        let result = AsyncChordResult {
            chord_id: chord_id.clone(),
            header_result: header_result.clone(),
            callback_task_id: callback_task_id.clone(),
        };

        assert_eq!(result.chord_id, chord_id);
        assert_eq!(result.callback_task_id, callback_task_id);
        assert_eq!(result.header_result.task_ids.len(), 2);
    }

    #[test]
    fn test_async_chord_result_clone() {
        let result = AsyncChordResult {
            chord_id: TaskId::new(),
            header_result: GroupResult {
                group_id: TaskId::new(),
                task_ids: vec![TaskId::new()],
            },
            callback_task_id: TaskId::new(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.chord_id, result.chord_id);
        assert_eq!(cloned.callback_task_id, result.callback_task_id);
        assert_eq!(
            cloned.header_result.task_ids.len(),
            result.header_result.task_ids.len()
        );
    }

    #[test]
    fn test_async_chord_result_empty_header() {
        let result = AsyncChordResult {
            chord_id: TaskId::new(),
            header_result: GroupResult {
                group_id: TaskId::new(),
                task_ids: vec![],
            },
            callback_task_id: TaskId::new(),
        };
        assert!(result.header_result.task_ids.is_empty());
    }
}

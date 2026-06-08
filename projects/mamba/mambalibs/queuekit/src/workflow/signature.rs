//! Task signature - represents a task call that hasn't been executed yet

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::RetryPolicy;

/// Options for task execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskOptions {
    /// Target queue (overrides task default)
    pub queue: Option<String>,
    /// Countdown in seconds before execution
    pub countdown: Option<u64>,
    /// Absolute ETA for execution
    pub eta: Option<DateTime<Utc>>,
    /// Expiration time
    pub expires: Option<DateTime<Utc>>,
    /// Custom retry policy
    pub retry_policy: Option<RetryPolicy>,
}

impl TaskOptions {
    /// Create new empty options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set target queue
    pub fn with_queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = Some(queue.into());
        self
    }

    /// Set countdown (delay in seconds)
    pub fn with_countdown(mut self, seconds: u64) -> Self {
        self.countdown = Some(seconds);
        self
    }

    /// Set ETA (absolute execution time)
    pub fn with_eta(mut self, eta: DateTime<Utc>) -> Self {
        self.eta = Some(eta);
        self
    }

    /// Set expiration time
    pub fn with_expires(mut self, expires: DateTime<Utc>) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set custom retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }
}

/// A task signature - represents a task call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSignature {
    /// Task name
    pub task_name: String,
    /// Positional arguments (JSON array)
    pub args: serde_json::Value,
    /// Keyword arguments (JSON object)
    pub kwargs: serde_json::Value,
    /// Execution options
    pub options: TaskOptions,
    /// Whether to pass previous result as first argument (for chains)
    /// If true, this task won't receive the chain's previous result
    pub immutable: bool,
}

impl TaskSignature {
    /// Create a new task signature
    pub fn new(task_name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            task_name: task_name.into(),
            args,
            kwargs: serde_json::Value::Null,
            options: TaskOptions::default(),
            immutable: false,
        }
    }

    /// Set keyword arguments
    pub fn with_kwargs(mut self, kwargs: serde_json::Value) -> Self {
        self.kwargs = kwargs;
        self
    }

    /// Set execution options
    pub fn with_options(mut self, options: TaskOptions) -> Self {
        self.options = options;
        self
    }

    /// Mark as immutable (don't receive chain result)
    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }

    /// Set target queue
    pub fn set_queue(mut self, queue: impl Into<String>) -> Self {
        self.options.queue = Some(queue.into());
        self
    }

    /// Set countdown (delay in seconds)
    pub fn set_countdown(mut self, seconds: u64) -> Self {
        self.options.countdown = Some(seconds);
        self
    }

    /// Set ETA
    pub fn set_eta(mut self, eta: DateTime<Utc>) -> Self {
        self.options.eta = Some(eta);
        self
    }

    /// Set expiration
    pub fn set_expires(mut self, expires: DateTime<Utc>) -> Self {
        self.options.expires = Some(expires);
        self
    }

    /// Set retry policy
    pub fn set_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.options.retry_policy = Some(policy);
        self
    }

    /// Get the args with previous result prepended (for chain execution)
    /// If immutable, returns args unchanged
    pub fn args_with_result(&self, previous_result: serde_json::Value) -> serde_json::Value {
        if self.immutable {
            return self.args.clone();
        }

        match &self.args {
            serde_json::Value::Array(arr) => {
                let mut new_args = vec![previous_result];
                new_args.extend(arr.clone());
                serde_json::Value::Array(new_args)
            }
            serde_json::Value::Null => {
                serde_json::Value::Array(vec![previous_result])
            }
            other => {
                // If args is not an array, wrap both in array
                serde_json::Value::Array(vec![previous_result, other.clone()])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_signature_new() {
        let sig = TaskSignature::new("my_task", serde_json::json!([1, 2, 3]));
        assert_eq!(sig.task_name, "my_task");
        assert_eq!(sig.args, serde_json::json!([1, 2, 3]));
        assert!(sig.kwargs.is_null());
        assert!(!sig.immutable);
    }

    #[test]
    fn test_task_signature_with_kwargs() {
        let sig = TaskSignature::new("my_task", serde_json::json!([]))
            .with_kwargs(serde_json::json!({"key": "value"}));
        assert_eq!(sig.kwargs, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn test_task_signature_immutable() {
        let sig = TaskSignature::new("my_task", serde_json::json!([])).immutable();
        assert!(sig.immutable);
    }

    #[test]
    fn test_task_options_builder() {
        let options = TaskOptions::new()
            .with_queue("priority")
            .with_countdown(60);

        assert_eq!(options.queue, Some("priority".to_string()));
        assert_eq!(options.countdown, Some(60));
    }

    #[test]
    fn test_args_with_result_array() {
        let sig = TaskSignature::new("task", serde_json::json!([2, 3]));
        let result = sig.args_with_result(serde_json::json!(1));
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_args_with_result_null() {
        let sig = TaskSignature::new("task", serde_json::Value::Null);
        let result = sig.args_with_result(serde_json::json!(42));
        assert_eq!(result, serde_json::json!([42]));
    }

    #[test]
    fn test_args_with_result_immutable() {
        let sig = TaskSignature::new("task", serde_json::json!([2, 3])).immutable();
        let result = sig.args_with_result(serde_json::json!(1));
        assert_eq!(result, serde_json::json!([2, 3])); // Original args unchanged
    }

    #[test]
    fn test_set_queue() {
        let sig = TaskSignature::new("task", serde_json::json!([]))
            .set_queue("high_priority");
        assert_eq!(sig.options.queue, Some("high_priority".to_string()));
    }

    #[test]
    fn test_set_countdown() {
        let sig = TaskSignature::new("task", serde_json::json!([]))
            .set_countdown(120);
        assert_eq!(sig.options.countdown, Some(120));
    }
}

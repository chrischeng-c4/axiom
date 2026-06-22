//! Task message format for broker communication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::TaskId;

/// Executor type for task execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum ExecutorType {
    /// Execute in-process by worker (default)
    #[default]
    InProcess,
    /// Execute as Kubernetes Job
    K8sJob,
}

/// K8s resource requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct K8sResources {
    /// CPU request/limit (e.g., "100m", "2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// Memory request/limit (e.g., "128Mi", "2Gi")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
    /// GPU count (nvidia.com/gpu)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<u32>,
    /// TPU count (google.com/tpu)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpu: Option<u32>,
    /// Extended resources (e.g., "amd.com/gpu": "1")
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extended: BTreeMap<String, String>,
}

/// K8s toleration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct K8sToleration {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
}

/// K8s executor configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct K8sConfig {
    /// Container image (defaults to base runner image)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Resource requirements
    #[serde(default, skip_serializing_if = "is_default")]
    pub resources: K8sResources,
    /// Node selector labels
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub node_selector: BTreeMap<String, String>,
    /// Tolerations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tolerations: Vec<K8sToleration>,
    /// Namespace (defaults to current namespace)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Service account name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,
    /// Active deadline seconds (job timeout)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_deadline_seconds: Option<i64>,
    /// Backoff limit (max retries within K8s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_limit: Option<i32>,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    *t == T::default()
}

impl PartialEq for K8sResources {
    fn eq(&self, other: &Self) -> bool {
        self.cpu == other.cpu
            && self.memory == other.memory
            && self.gpu == other.gpu
            && self.tpu == other.tpu
            && self.extended == other.extended
    }
}

/// Task message sent through the broker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TaskMessage {
    /// Unique task identifier
    pub id: TaskId,
    /// Name of the task to execute
    pub task_name: String,
    /// Positional arguments (JSON array)
    pub args: serde_json::Value,
    /// Keyword arguments (JSON object)
    pub kwargs: serde_json::Value,
    /// Number of retry attempts so far
    pub retries: u32,
    /// Earliest time to execute (for delayed tasks)
    pub eta: Option<DateTime<Utc>>,
    /// Task expiration time
    pub expires: Option<DateTime<Utc>>,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    /// Parent task ID (for chains)
    pub parent_id: Option<TaskId>,
    /// Root task ID (for workflows)
    pub root_id: Option<TaskId>,
    /// Executor type (in-process or k8s-job)
    #[serde(default)]
    pub executor: ExecutorType,
    /// K8s configuration (only used when executor is K8sJob)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub k8s_config: Option<K8sConfig>,
}

impl TaskMessage {
    /// Create a new task message
    pub fn new(task_name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            id: TaskId::new(),
            task_name: task_name.into(),
            args,
            kwargs: serde_json::Value::Null,
            retries: 0,
            eta: None,
            expires: None,
            correlation_id: None,
            parent_id: None,
            root_id: None,
            executor: ExecutorType::default(),
            k8s_config: None,
        }
    }

    /// Set executor type
    pub fn with_executor(mut self, executor: ExecutorType) -> Self {
        self.executor = executor;
        self
    }

    /// Set K8s configuration
    pub fn with_k8s_config(mut self, config: K8sConfig) -> Self {
        self.executor = ExecutorType::K8sJob;
        self.k8s_config = Some(config);
        self
    }

    /// Check if this task should be executed as K8s Job
    pub fn is_k8s_job(&self) -> bool {
        matches!(self.executor, ExecutorType::K8sJob)
    }

    /// Set keyword arguments
    pub fn with_kwargs(mut self, kwargs: serde_json::Value) -> Self {
        self.kwargs = kwargs;
        self
    }

    /// Set ETA for delayed execution
    pub fn with_eta(mut self, eta: DateTime<Utc>) -> Self {
        self.eta = Some(eta);
        self
    }

    /// Set expiration time
    pub fn with_expires(mut self, expires: DateTime<Utc>) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Set parent task ID
    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set root task ID
    pub fn with_root(mut self, root_id: TaskId) -> Self {
        self.root_id = Some(root_id);
        self
    }

    /// Check if the task has expired
    pub fn is_expired(&self) -> bool {
        self.expires.map(|e| e < Utc::now()).unwrap_or(false)
    }

    /// Check if the task is ready to execute (ETA passed)
    pub fn is_ready(&self) -> bool {
        self.eta.map(|e| e <= Utc::now()).unwrap_or(true)
    }

    /// Increment retry count and return new message
    pub fn for_retry(mut self) -> Self {
        self.retries += 1;
        self.eta = None; // Clear ETA for immediate retry (delay handled separately)
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------
    // Construction: TaskMessage::new()
    // -------------------------------------------------------------------

    #[test]
    fn test_new_message() {
        let msg = TaskMessage::new("my_task", serde_json::json!([1, 2, 3]));
        assert_eq!(msg.task_name, "my_task");
        assert_eq!(msg.args, serde_json::json!([1, 2, 3]));
        assert_eq!(msg.retries, 0);
        assert!(msg.is_ready());
        assert!(!msg.is_expired());
    }

    #[test]
    fn test_new_message_default_fields() {
        let msg = TaskMessage::new("t", serde_json::json!([]));
        // kwargs defaults to Null
        assert_eq!(msg.kwargs, serde_json::Value::Null);
        // Optional fields default to None
        assert!(msg.eta.is_none());
        assert!(msg.expires.is_none());
        assert!(msg.correlation_id.is_none());
        assert!(msg.parent_id.is_none());
        assert!(msg.root_id.is_none());
        assert!(msg.k8s_config.is_none());
        // Executor defaults to InProcess
        assert_eq!(msg.executor, ExecutorType::InProcess);
    }

    #[test]
    fn test_new_message_id_is_unique() {
        let a = TaskMessage::new("t", serde_json::json!([]));
        let b = TaskMessage::new("t", serde_json::json!([]));
        assert_ne!(a.id, b.id);
    }

    // -------------------------------------------------------------------
    // Construction: edge-case args
    // -------------------------------------------------------------------

    #[test]
    fn test_new_with_empty_args() {
        let msg = TaskMessage::new("t", serde_json::json!([]));
        assert_eq!(msg.args, serde_json::json!([]));
    }

    #[test]
    fn test_new_with_null_args() {
        let msg = TaskMessage::new("t", serde_json::Value::Null);
        assert_eq!(msg.args, serde_json::Value::Null);
    }

    #[test]
    fn test_new_with_large_args() {
        // Create a JSON array with 10000 elements
        let large: Vec<i32> = (0..10_000).collect();
        let val = serde_json::json!(large);
        let msg = TaskMessage::new("big", val.clone());
        assert_eq!(msg.args, val);
    }

    #[test]
    fn test_new_with_nested_object_args() {
        let nested = serde_json::json!({
            "a": {"b": {"c": [1, 2, 3]}},
            "d": null,
            "e": true,
        });
        let msg = TaskMessage::new("nested", nested.clone());
        assert_eq!(msg.args, nested);
    }

    // -------------------------------------------------------------------
    // Serde roundtrip
    // -------------------------------------------------------------------

    #[test]
    fn test_serde_roundtrip_minimal() {
        let msg = TaskMessage::new("roundtrip", serde_json::json!([42, "hello"]));
        let json = serde_json::to_string(&msg).expect("serialize");
        let deser: TaskMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, msg.id);
        assert_eq!(deser.task_name, msg.task_name);
        assert_eq!(deser.args, msg.args);
        assert_eq!(deser.kwargs, msg.kwargs);
        assert_eq!(deser.retries, msg.retries);
        assert_eq!(deser.executor, msg.executor);
    }

    #[test]
    fn test_serde_roundtrip_all_fields() {
        let parent = TaskId::new();
        let root = TaskId::new();
        let eta = Utc::now() + chrono::Duration::hours(1);
        let expires = Utc::now() + chrono::Duration::hours(2);

        let msg = TaskMessage::new("full", serde_json::json!(["a"]))
            .with_kwargs(serde_json::json!({"key": "val"}))
            .with_eta(eta)
            .with_expires(expires)
            .with_correlation_id("corr-123")
            .with_parent(parent.clone())
            .with_root(root.clone())
            .with_k8s_config(K8sConfig {
                image: Some("my-image:latest".to_string()),
                namespace: Some("prod".to_string()),
                ..Default::default()
            });

        let json = serde_json::to_string(&msg).expect("serialize");
        let deser: TaskMessage = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deser.task_name, "full");
        assert_eq!(deser.kwargs, serde_json::json!({"key": "val"}));
        assert_eq!(deser.correlation_id.as_deref(), Some("corr-123"));
        assert_eq!(deser.parent_id, Some(parent));
        assert_eq!(deser.root_id, Some(root));
        assert_eq!(deser.executor, ExecutorType::K8sJob);
        assert!(deser.k8s_config.is_some());
        assert_eq!(
            deser.k8s_config.as_ref().unwrap().image.as_deref(),
            Some("my-image:latest")
        );
    }

    #[test]
    fn test_serde_roundtrip_preserves_retries() {
        let msg = TaskMessage::new("r", serde_json::json!([]))
            .for_retry()
            .for_retry()
            .for_retry();
        let json = serde_json::to_string(&msg).unwrap();
        let deser: TaskMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.retries, 3);
    }

    // -------------------------------------------------------------------
    // is_expired()
    // -------------------------------------------------------------------

    #[test]
    fn test_is_expired_none_returns_false() {
        let msg = TaskMessage::new("t", serde_json::json!([]));
        assert!(msg.expires.is_none());
        assert!(!msg.is_expired());
    }

    #[test]
    fn test_is_expired_future_returns_false() {
        let future = Utc::now() + chrono::Duration::hours(1);
        let msg = TaskMessage::new("t", serde_json::json!([])).with_expires(future);
        assert!(!msg.is_expired());
    }

    #[test]
    fn test_is_expired_past_returns_true() {
        let past = Utc::now() - chrono::Duration::hours(1);
        let msg = TaskMessage::new("t", serde_json::json!([])).with_expires(past);
        assert!(msg.is_expired());
    }

    // -------------------------------------------------------------------
    // is_ready()
    // -------------------------------------------------------------------

    #[test]
    fn test_is_ready_no_eta() {
        let msg = TaskMessage::new("t", serde_json::json!([]));
        assert!(msg.is_ready());
    }

    #[test]
    fn test_is_ready_past_eta() {
        let past = Utc::now() - chrono::Duration::hours(1);
        let msg = TaskMessage::new("t", serde_json::json!([])).with_eta(past);
        assert!(msg.is_ready());
    }

    #[test]
    fn test_delayed_message() {
        let future = Utc::now() + chrono::Duration::hours(1);
        let msg = TaskMessage::new("delayed_task", serde_json::json!([])).with_eta(future);
        assert!(!msg.is_ready());
    }

    // -------------------------------------------------------------------
    // Builder methods
    // -------------------------------------------------------------------

    #[test]
    fn test_with_correlation_id() {
        let msg = TaskMessage::new("t", serde_json::json!([])).with_correlation_id("abc-123");
        assert_eq!(msg.correlation_id.as_deref(), Some("abc-123"));
    }

    #[test]
    fn test_with_parent() {
        let parent = TaskId::new();
        let msg = TaskMessage::new("t", serde_json::json!([])).with_parent(parent.clone());
        assert_eq!(msg.parent_id, Some(parent));
    }

    #[test]
    fn test_with_root() {
        let root = TaskId::new();
        let msg = TaskMessage::new("t", serde_json::json!([])).with_root(root.clone());
        assert_eq!(msg.root_id, Some(root));
    }

    #[test]
    fn test_with_kwargs() {
        let msg =
            TaskMessage::new("t", serde_json::json!([])).with_kwargs(serde_json::json!({"x": 1}));
        assert_eq!(msg.kwargs, serde_json::json!({"x": 1}));
    }

    #[test]
    fn test_with_executor() {
        let msg = TaskMessage::new("t", serde_json::json!([])).with_executor(ExecutorType::K8sJob);
        assert_eq!(msg.executor, ExecutorType::K8sJob);
        assert!(msg.is_k8s_job());
    }

    #[test]
    fn test_with_k8s_config_sets_executor() {
        let msg =
            TaskMessage::new("t", serde_json::json!([])).with_k8s_config(K8sConfig::default());
        // with_k8s_config implicitly sets executor to K8sJob
        assert_eq!(msg.executor, ExecutorType::K8sJob);
        assert!(msg.k8s_config.is_some());
    }

    // -------------------------------------------------------------------
    // Retry
    // -------------------------------------------------------------------

    #[test]
    fn test_retry() {
        let msg = TaskMessage::new("retry_task", serde_json::json!([]));
        assert_eq!(msg.retries, 0);
        let msg = msg.for_retry();
        assert_eq!(msg.retries, 1);
        let msg = msg.for_retry();
        assert_eq!(msg.retries, 2);
    }

    #[test]
    fn test_for_retry_clears_eta() {
        let eta = Utc::now() + chrono::Duration::hours(1);
        let msg = TaskMessage::new("t", serde_json::json!([])).with_eta(eta);
        assert!(msg.eta.is_some());
        let retried = msg.for_retry();
        assert!(retried.eta.is_none());
        assert_eq!(retried.retries, 1);
    }

    #[test]
    fn test_for_retry_preserves_other_fields() {
        let msg = TaskMessage::new("t", serde_json::json!([1]))
            .with_correlation_id("corr")
            .with_kwargs(serde_json::json!({"k": "v"}));
        let retried = msg.for_retry();
        assert_eq!(retried.task_name, "t");
        assert_eq!(retried.args, serde_json::json!([1]));
        assert_eq!(retried.correlation_id.as_deref(), Some("corr"));
        assert_eq!(retried.kwargs, serde_json::json!({"k": "v"}));
    }

    // -------------------------------------------------------------------
    // ExecutorType serde
    // -------------------------------------------------------------------

    #[test]
    fn test_executor_type_serde_kebab_case() {
        let in_proc = serde_json::to_string(&ExecutorType::InProcess).unwrap();
        assert_eq!(in_proc, "\"in-process\"");
        let k8s = serde_json::to_string(&ExecutorType::K8sJob).unwrap();
        assert_eq!(k8s, "\"k8s-job\"");

        let deser: ExecutorType = serde_json::from_str("\"in-process\"").unwrap();
        assert_eq!(deser, ExecutorType::InProcess);
        let deser: ExecutorType = serde_json::from_str("\"k8s-job\"").unwrap();
        assert_eq!(deser, ExecutorType::K8sJob);
    }

    #[test]
    fn test_executor_type_default_is_in_process() {
        assert_eq!(ExecutorType::default(), ExecutorType::InProcess);
    }

    // -------------------------------------------------------------------
    // is_k8s_job()
    // -------------------------------------------------------------------

    #[test]
    fn test_is_k8s_job_false_by_default() {
        let msg = TaskMessage::new("t", serde_json::json!([]));
        assert!(!msg.is_k8s_job());
    }

    #[test]
    fn test_is_k8s_job_true_after_with_k8s_config() {
        let msg =
            TaskMessage::new("t", serde_json::json!([])).with_k8s_config(K8sConfig::default());
        assert!(msg.is_k8s_job());
    }
}

//! Core task traits and types

use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::RetryPolicy;

/// Unique task identifier using UUID v7 (time-ordered)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TaskId(pub uuid::Uuid);

impl TaskId {
    /// Create a new task ID
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// Parse from string
    pub fn from_string(s: &str) -> Result<Self, crate::TaskError> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|e| crate::TaskError::InvalidTaskId(e.to_string()))
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task execution context
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// Unique task ID
    pub task_id: TaskId,
    /// Task name
    pub task_name: String,
    /// Queue the task was received from
    pub queue: String,
    /// Current retry count
    pub retry_count: u32,
    /// Maximum allowed retries
    pub max_retries: u32,
    /// Correlation ID for tracing
    pub correlation_id: Option<String>,
    /// Parent task ID (for chains)
    pub parent_id: Option<TaskId>,
    /// Root task ID (for workflows)
    pub root_id: Option<TaskId>,
}

/// Result returned by task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    /// Task completed successfully with a result
    Success(serde_json::Value),
    /// Task failed
    Failure {
        /// Error message
        error: String,
        /// Whether this error is retryable
        retryable: bool,
    },
    /// Task requests explicit retry
    Retry {
        /// Reason for retry
        reason: String,
        /// Optional countdown before retry
        countdown: Option<Duration>,
    },
}

/// When to acknowledge message to broker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AckMode {
    /// Ack before execution (at-most-once delivery)
    BeforeExecution,
    /// Ack after successful execution (at-least-once delivery)
    #[default]
    AfterExecution,
    /// Manual ack (task controls when to ack)
    Manual,
}

/// Core task trait - implemented by task handlers
#[async_trait]
pub trait Task: Send + Sync + 'static {
    /// Task name (must be unique, used for routing)
    fn name(&self) -> &'static str;

    /// Default queue for this task
    fn queue(&self) -> &'static str {
        "default"
    }

    /// Execute the task with given arguments
    async fn execute(&self, ctx: TaskContext, args: serde_json::Value) -> TaskOutcome;

    /// Retry policy for this task
    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::default()
    }

    /// Soft time limit (task receives warning)
    fn soft_time_limit(&self) -> Option<Duration> {
        None
    }

    /// Hard time limit (task is killed)
    fn hard_time_limit(&self) -> Option<Duration> {
        None
    }

    /// Rate limit (tasks per second, 0 = unlimited)
    fn rate_limit(&self) -> f64 {
        0.0
    }

    /// Acknowledgment mode
    fn ack_mode(&self) -> AckMode {
        AckMode::AfterExecution
    }
}

/// Registry of all registered tasks
pub struct TaskRegistry {
    tasks: DashMap<String, Arc<dyn Task>>,
    router: Option<Arc<crate::routing::Router>>,
}

impl TaskRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tasks: DashMap::new(),
            router: None,
        }
    }

    /// Set the router for this registry (builder pattern)
    pub fn with_router(mut self, router: crate::routing::Router) -> Self {
        self.router = Some(Arc::new(router));
        self
    }

    /// Set router after creation
    pub fn set_router(&mut self, router: crate::routing::Router) {
        self.router = Some(Arc::new(router));
    }

    /// Get the router
    pub fn router(&self) -> Option<&Arc<crate::routing::Router>> {
        self.router.as_ref()
    }

    /// Route a task to its target queue
    ///
    /// Returns the queue name based on configured routing rules.
    /// Falls back to "default" if no router is configured.
    pub fn route_task(&self, task_name: &str, args: &serde_json::Value) -> String {
        if let Some(router) = &self.router {
            router.route(task_name, args)
        } else {
            "default".to_string()
        }
    }

    /// Register a task handler
    pub fn register<T: Task>(&self, task: T) {
        let name = task.name().to_string();
        tracing::debug!("Registering task: {}", name);
        self.tasks.insert(name, Arc::new(task));
    }

    /// Get task by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Task>> {
        self.tasks.get(name).map(|r| r.clone())
    }

    /// Check if task exists
    pub fn contains(&self, name: &str) -> bool {
        self.tasks.contains_key(name)
    }

    /// List all registered task names
    pub fn list(&self) -> Vec<String> {
        self.tasks.iter().map(|r| r.key().clone()).collect()
    }

    /// Number of registered tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTask;

    #[async_trait]
    impl Task for TestTask {
        fn name(&self) -> &'static str {
            "test_task"
        }

        async fn execute(&self, _ctx: TaskContext, args: serde_json::Value) -> TaskOutcome {
            TaskOutcome::Success(args)
        }
    }

    #[test]
    fn test_task_id() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);

        // Use Display trait instead of inherent to_string()
        let id_str = format!("{}", id1);
        let parsed = TaskId::from_string(&id_str).unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_registry() {
        let registry = TaskRegistry::new();
        assert!(registry.is_empty());

        registry.register(TestTask);
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test_task"));
        assert!(!registry.contains("nonexistent"));

        let task = registry.get("test_task").unwrap();
        assert_eq!(task.name(), "test_task");
        assert_eq!(task.queue(), "default");
    }

    #[test]
    fn test_registry_with_router() {
        use crate::routing::RouterConfig;

        let router = RouterConfig::new()
            .route("math.add", "math")
            .route_glob("email.*", "email")
            .route("test_task", "testing")
            .build();

        let registry = TaskRegistry::new().with_router(router);

        // Verify router is set
        assert!(registry.router().is_some());

        // Test routing
        assert_eq!(registry.route_task("math.add", &serde_json::json!({})), "math");
        assert_eq!(registry.route_task("email.send", &serde_json::json!({})), "email");
        assert_eq!(registry.route_task("test_task", &serde_json::json!({})), "testing");
        assert_eq!(registry.route_task("unknown", &serde_json::json!({})), "default");
    }

    #[test]
    fn test_registry_without_router() {
        let registry = TaskRegistry::new();

        // Without router, should return "default"
        assert!(registry.router().is_none());
        assert_eq!(registry.route_task("any_task", &serde_json::json!({})), "default");
    }

    #[test]
    fn test_registry_set_router() {
        use crate::routing::RouterConfig;

        let mut registry = TaskRegistry::new();
        assert!(registry.router().is_none());

        let router = RouterConfig::new()
            .route("task_a", "queue_a")
            .build();

        registry.set_router(router);
        assert!(registry.router().is_some());
        assert_eq!(registry.route_task("task_a", &serde_json::json!({})), "queue_a");
    }
}

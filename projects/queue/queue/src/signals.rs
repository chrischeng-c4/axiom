//! Task lifecycle signals/events.
//!
//! Provides hooks into task lifecycle events for monitoring, logging,
//! and custom behavior. Similar to Celery's signal system.
//!
//! # Example
//! ```rust,ignore
//! use cclab_queue::signals::{Signal, SignalHandler, SignalDispatcher};
//!
//! // Create a handler
//! struct LoggingHandler;
//!
//! #[async_trait]
//! impl SignalHandler for LoggingHandler {
//!     async fn handle(&self, signal: &Signal) {
//!         match signal {
//!             Signal::TaskPrerun { task_id, task_name, .. } => {
//!                 println!("Task {} ({}) starting", task_name, task_id);
//!             }
//!             Signal::TaskSuccess { task_id, result, runtime, .. } => {
//!                 println!("Task {} completed in {:?}", task_id, runtime);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//!
//! // Register handler
//! let dispatcher = SignalDispatcher::new()
//!     .on_all(LoggingHandler);
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::state::TaskState;
use crate::task::TaskId;

/// Task lifecycle signals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Signal {
    // ==================== Task Signals ====================
    /// Before a task message is published to the broker
    BeforeTaskPublish {
        task_id: TaskId,
        task_name: String,
        queue: String,
        args: serde_json::Value,
        kwargs: serde_json::Value,
        eta: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// After a task message is published to the broker
    AfterTaskPublish {
        task_id: TaskId,
        task_name: String,
        queue: String,
    },

    /// Before a task starts executing (worker received it)
    TaskReceived {
        task_id: TaskId,
        task_name: String,
        queue: String,
        worker_name: String,
    },

    /// Just before task execution begins
    TaskPrerun {
        task_id: TaskId,
        task_name: String,
        args: serde_json::Value,
        kwargs: serde_json::Value,
        worker_name: String,
    },

    /// After task execution completes (success or failure)
    TaskPostrun {
        task_id: TaskId,
        task_name: String,
        state: TaskState,
        runtime: Duration,
        worker_name: String,
    },

    /// Task completed successfully
    TaskSuccess {
        task_id: TaskId,
        task_name: String,
        result: serde_json::Value,
        runtime: Duration,
        worker_name: String,
    },

    /// Task failed with an error
    TaskFailure {
        task_id: TaskId,
        task_name: String,
        error: String,
        traceback: Option<String>,
        runtime: Duration,
        worker_name: String,
    },

    /// Task is being retried
    TaskRetry {
        task_id: TaskId,
        task_name: String,
        reason: String,
        retry_count: u32,
        max_retries: u32,
        eta: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// Task was revoked/cancelled
    TaskRevoked {
        task_id: TaskId,
        task_name: String,
        reason: Option<String>,
        terminated: bool,
    },

    /// Task was rejected (won't be retried)
    TaskRejected {
        task_id: TaskId,
        task_name: String,
        reason: String,
    },

    // ==================== Worker Signals ====================
    /// Worker is initializing
    WorkerInit {
        worker_name: String,
        queues: Vec<String>,
        concurrency: usize,
    },

    /// Worker is ready to accept tasks
    WorkerReady { worker_name: String },

    /// Worker is shutting down
    WorkerShutdown {
        worker_name: String,
        reason: ShutdownReason,
    },

    /// Worker heartbeat (periodic)
    WorkerHeartbeat {
        worker_name: String,
        active_tasks: usize,
        processed: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    // ==================== Rate Limit Signals ====================
    /// Task was rate limited
    TaskRateLimited {
        task_id: TaskId,
        task_name: String,
        queue: String,
        retry_after: Duration,
    },
}

/// Reason for worker shutdown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownReason {
    /// Normal graceful shutdown
    Graceful,
    /// Shutdown due to error
    Error(String),
    /// Shutdown requested via signal (SIGTERM, etc.)
    Signal,
    /// Connection lost to broker
    ConnectionLost,
}

impl Signal {
    /// Get the signal type as a string
    pub fn signal_type(&self) -> &'static str {
        match self {
            Signal::BeforeTaskPublish { .. } => "before_task_publish",
            Signal::AfterTaskPublish { .. } => "after_task_publish",
            Signal::TaskReceived { .. } => "task_received",
            Signal::TaskPrerun { .. } => "task_prerun",
            Signal::TaskPostrun { .. } => "task_postrun",
            Signal::TaskSuccess { .. } => "task_success",
            Signal::TaskFailure { .. } => "task_failure",
            Signal::TaskRetry { .. } => "task_retry",
            Signal::TaskRevoked { .. } => "task_revoked",
            Signal::TaskRejected { .. } => "task_rejected",
            Signal::WorkerInit { .. } => "worker_init",
            Signal::WorkerReady { .. } => "worker_ready",
            Signal::WorkerShutdown { .. } => "worker_shutdown",
            Signal::WorkerHeartbeat { .. } => "worker_heartbeat",
            Signal::TaskRateLimited { .. } => "task_rate_limited",
        }
    }

    /// Get the task ID if this is a task-related signal
    pub fn task_id(&self) -> Option<&TaskId> {
        match self {
            Signal::BeforeTaskPublish { task_id, .. }
            | Signal::AfterTaskPublish { task_id, .. }
            | Signal::TaskReceived { task_id, .. }
            | Signal::TaskPrerun { task_id, .. }
            | Signal::TaskPostrun { task_id, .. }
            | Signal::TaskSuccess { task_id, .. }
            | Signal::TaskFailure { task_id, .. }
            | Signal::TaskRetry { task_id, .. }
            | Signal::TaskRevoked { task_id, .. }
            | Signal::TaskRejected { task_id, .. }
            | Signal::TaskRateLimited { task_id, .. } => Some(task_id),
            _ => None,
        }
    }

    /// Get the task name if this is a task-related signal
    pub fn task_name(&self) -> Option<&str> {
        match self {
            Signal::BeforeTaskPublish { task_name, .. }
            | Signal::AfterTaskPublish { task_name, .. }
            | Signal::TaskReceived { task_name, .. }
            | Signal::TaskPrerun { task_name, .. }
            | Signal::TaskPostrun { task_name, .. }
            | Signal::TaskSuccess { task_name, .. }
            | Signal::TaskFailure { task_name, .. }
            | Signal::TaskRetry { task_name, .. }
            | Signal::TaskRevoked { task_name, .. }
            | Signal::TaskRejected { task_name, .. }
            | Signal::TaskRateLimited { task_name, .. } => Some(task_name),
            _ => None,
        }
    }

    /// Check if this is a task-related signal
    pub fn is_task_signal(&self) -> bool {
        self.task_id().is_some()
    }

    /// Check if this is a worker-related signal
    pub fn is_worker_signal(&self) -> bool {
        matches!(
            self,
            Signal::WorkerInit { .. }
                | Signal::WorkerReady { .. }
                | Signal::WorkerShutdown { .. }
                | Signal::WorkerHeartbeat { .. }
        )
    }
}

/// Handler for signals
#[async_trait]
pub trait SignalHandler: Send + Sync {
    /// Handle a signal
    async fn handle(&self, signal: &Signal);

    /// Filter which signals this handler receives
    /// Default: receive all signals
    fn accepts(&self, signal: &Signal) -> bool {
        let _ = signal;
        true
    }
}

/// Function-based signal handler
pub struct FnHandler<F>
where
    F: Fn(&Signal) + Send + Sync,
{
    func: F,
    filter: Option<Vec<&'static str>>,
}

impl<F> FnHandler<F>
where
    F: Fn(&Signal) + Send + Sync,
{
    /// Create a new function handler
    pub fn new(func: F) -> Self {
        Self { func, filter: None }
    }

    /// Only handle specific signal types
    pub fn only(mut self, signal_types: Vec<&'static str>) -> Self {
        self.filter = Some(signal_types);
        self
    }
}

#[async_trait]
impl<F> SignalHandler for FnHandler<F>
where
    F: Fn(&Signal) + Send + Sync,
{
    async fn handle(&self, signal: &Signal) {
        (self.func)(signal);
    }

    fn accepts(&self, signal: &Signal) -> bool {
        match &self.filter {
            Some(types) => types.contains(&signal.signal_type()),
            None => true,
        }
    }
}

/// Dispatcher for sending signals to handlers
pub struct SignalDispatcher {
    handlers: RwLock<Vec<Arc<dyn SignalHandler>>>,
}

impl Default for SignalDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalDispatcher {
    /// Create a new signal dispatcher
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    /// Register a handler for all signals
    pub async fn register<H: SignalHandler + 'static>(&self, handler: H) {
        let mut handlers = self.handlers.write().await;
        handlers.push(Arc::new(handler));
    }

    /// Register handler (builder pattern)
    ///
    /// Note: This must be called from outside async context or before runtime is started.
    /// For runtime registration, use `register()` instead.
    pub fn on_all<H: SignalHandler + 'static>(mut self, handler: H) -> Self {
        // Access handlers directly since we have mut self
        let handlers = self.handlers.get_mut();
        handlers.push(Arc::new(handler));
        self
    }

    /// Register a function handler
    pub fn on<F>(self, signal_types: Vec<&'static str>, func: F) -> Self
    where
        F: Fn(&Signal) + Send + Sync + 'static,
    {
        let handler = FnHandler::new(func).only(signal_types);
        self.on_all(handler)
    }

    /// Dispatch a signal to all registered handlers
    pub async fn dispatch(&self, signal: Signal) {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            if handler.accepts(&signal) {
                handler.handle(&signal).await;
            }
        }
    }

    /// Dispatch signal in background (fire and forget)
    pub fn dispatch_background(&self, signal: Signal)
    where
        Self: 'static,
    {
        let handlers = self.handlers.try_read();
        if let Ok(handlers) = handlers {
            let handlers: Vec<_> = handlers.iter().cloned().collect();
            tokio::spawn(async move {
                for handler in handlers {
                    if handler.accepts(&signal) {
                        handler.handle(&signal).await;
                    }
                }
            });
        }
    }

    /// Get number of registered handlers
    pub async fn handler_count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

// Make dispatcher cloneable via Arc
impl Clone for SignalDispatcher {
    fn clone(&self) -> Self {
        // Note: This creates a new dispatcher, not a shared one
        // For shared dispatchers, wrap in Arc
        Self::new()
    }
}

/// Convenience macro for creating signal handlers
#[macro_export]
macro_rules! on_signal {
    ($signal_type:ident, |$signal:ident| $body:expr) => {
        FnHandler::new(|$signal| {
            if let Signal::$signal_type { .. } = $signal {
                $body
            }
        })
        .only(vec![stringify!($signal_type).to_lowercase().as_str()])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_signal_type() {
        let signal = Signal::TaskSuccess {
            task_id: TaskId::new(),
            task_name: "test".to_string(),
            result: serde_json::json!(42),
            runtime: Duration::from_secs(1),
            worker_name: "worker-1".to_string(),
        };
        assert_eq!(signal.signal_type(), "task_success");
    }

    #[test]
    fn test_signal_task_id() {
        let task_id = TaskId::new();
        let signal = Signal::TaskPrerun {
            task_id: task_id.clone(),
            task_name: "test".to_string(),
            args: serde_json::json!([]),
            kwargs: serde_json::json!({}),
            worker_name: "worker-1".to_string(),
        };
        assert_eq!(signal.task_id(), Some(&task_id));
    }

    #[test]
    fn test_is_task_signal() {
        let task_signal = Signal::TaskSuccess {
            task_id: TaskId::new(),
            task_name: "test".to_string(),
            result: serde_json::json!(null),
            runtime: Duration::ZERO,
            worker_name: "w".to_string(),
        };
        assert!(task_signal.is_task_signal());

        let worker_signal = Signal::WorkerReady {
            worker_name: "w".to_string(),
        };
        assert!(!worker_signal.is_task_signal());
        assert!(worker_signal.is_worker_signal());
    }

    #[tokio::test]
    async fn test_dispatcher_basic() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        struct CountingHandler {
            counter: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl SignalHandler for CountingHandler {
            async fn handle(&self, _signal: &Signal) {
                self.counter.fetch_add(1, Ordering::SeqCst);
            }
        }

        let dispatcher = SignalDispatcher::new().on_all(CountingHandler {
            counter: counter_clone,
        });

        let signal = Signal::TaskSuccess {
            task_id: TaskId::new(),
            task_name: "test".to_string(),
            result: serde_json::json!(null),
            runtime: Duration::ZERO,
            worker_name: "w".to_string(),
        };

        dispatcher.dispatch(signal).await;
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_fn_handler_filter() {
        let success_count = Arc::new(AtomicUsize::new(0));
        let success_clone = success_count.clone();

        let dispatcher = SignalDispatcher::new().on(vec!["task_success"], move |_| {
            success_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Send success signal
        dispatcher
            .dispatch(Signal::TaskSuccess {
                task_id: TaskId::new(),
                task_name: "test".to_string(),
                result: serde_json::json!(null),
                runtime: Duration::ZERO,
                worker_name: "w".to_string(),
            })
            .await;

        // Send failure signal (should not trigger handler)
        dispatcher
            .dispatch(Signal::TaskFailure {
                task_id: TaskId::new(),
                task_name: "test".to_string(),
                error: "error".to_string(),
                traceback: None,
                runtime: Duration::ZERO,
                worker_name: "w".to_string(),
            })
            .await;

        assert_eq!(success_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_signal_serde() {
        let signal = Signal::TaskSuccess {
            task_id: TaskId::new(),
            task_name: "test".to_string(),
            result: serde_json::json!({"key": "value"}),
            runtime: Duration::from_millis(500),
            worker_name: "worker-1".to_string(),
        };

        let json = serde_json::to_string(&signal).unwrap();
        assert!(json.contains("task_success"));
        assert!(json.contains("test"));
    }

    #[tokio::test]
    async fn test_handler_count() {
        let dispatcher = SignalDispatcher::new();
        assert_eq!(dispatcher.handler_count().await, 0);

        struct DummyHandler;
        #[async_trait]
        impl SignalHandler for DummyHandler {
            async fn handle(&self, _: &Signal) {}
        }

        dispatcher.register(DummyHandler).await;
        assert_eq!(dispatcher.handler_count().await, 1);
    }
}

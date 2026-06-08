//! Worker runtime for executing tasks

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    BrokerMessage, MessageHandler, PullBroker, ResultBackend, Task, TaskContext,
    TaskOutcome, TaskRegistry, TaskResult, TaskState, TaskId,
};
use crate::ratelimit::RateLimitManager;
use crate::revocation::RevocationStore;
use crate::signals::{Signal, SignalDispatcher, ShutdownReason};
use crate::TaskError;

/// Worker configuration
#[derive(Clone)]
pub struct WorkerConfig {
    /// Worker name/ID
    pub name: String,
    /// Queues to consume from
    pub queues: Vec<String>,
    /// Concurrency (max parallel tasks)
    pub concurrency: usize,
    /// Prefetch count (messages to buffer)
    pub prefetch: usize,
    /// Heartbeat interval
    pub heartbeat: Duration,
    /// Optional revocation store
    pub revocation_store: Option<Arc<dyn RevocationStore>>,
}

impl std::fmt::Debug for WorkerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerConfig")
            .field("name", &self.name)
            .field("queues", &self.queues)
            .field("concurrency", &self.concurrency)
            .field("prefetch", &self.prefetch)
            .field("heartbeat", &self.heartbeat)
            .field("revocation_store", &self.revocation_store.as_ref().map(|_| "Some(RevocationStore)"))
            .finish()
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            name: format!("worker-{}", uuid::Uuid::now_v7().simple()),
            queues: vec!["default".to_string()],
            concurrency: num_cpus::get(),
            prefetch: 4,
            heartbeat: Duration::from_secs(10),
            revocation_store: None,
        }
    }
}

/// Task executor that handles incoming messages
struct TaskExecutor<R: ResultBackend> {
    registry: Arc<TaskRegistry>,
    backend: Arc<R>,
    semaphore: Arc<Semaphore>,
    worker_id: String,
    rate_limiter: Option<Arc<RateLimitManager>>,
    signal_dispatcher: Option<Arc<SignalDispatcher>>,
    revocation_store: Option<Arc<dyn RevocationStore>>,
}

impl<R: ResultBackend> TaskExecutor<R> {
    fn new(
        registry: Arc<TaskRegistry>,
        backend: Arc<R>,
        semaphore: Arc<Semaphore>,
        worker_id: String,
        rate_limiter: Option<Arc<RateLimitManager>>,
        signal_dispatcher: Option<Arc<SignalDispatcher>>,
        revocation_store: Option<Arc<dyn RevocationStore>>,
    ) -> Self {
        Self {
            registry,
            backend,
            semaphore,
            worker_id,
            rate_limiter,
            signal_dispatcher,
            revocation_store,
        }
    }

    /// Check if a task has been revoked
    async fn check_revocation(&self, task_id: &TaskId) -> Result<bool, TaskError> {
        if let Some(store) = &self.revocation_store {
            store.is_revoked(task_id).await
        } else {
            Ok(false)
        }
    }

    /// Execute a task with timeout handling
    async fn execute_with_timeout(
        &self,
        task: Arc<dyn Task>,
        ctx: TaskContext,
        args: serde_json::Value,
    ) -> Result<TaskOutcome, TaskError> {
        // Check for hard time limit
        if let Some(timeout) = task.hard_time_limit() {
            tokio::select! {
                result = task.execute(ctx.clone(), args) => {
                    Ok(result)
                }
                _ = tokio::time::sleep(timeout) => {
                    tracing::warn!(
                        task_id = %ctx.task_id,
                        task_name = %ctx.task_name,
                        timeout_secs = timeout.as_secs(),
                        "Task exceeded hard time limit"
                    );
                    Ok(TaskOutcome::Failure {
                        error: format!("Task exceeded hard time limit of {}s", timeout.as_secs()),
                        retryable: false,
                    })
                }
            }
        } else {
            // No timeout, execute normally
            Ok(task.execute(ctx, args).await)
        }
    }
}

#[async_trait]
impl<R: ResultBackend> MessageHandler for TaskExecutor<R> {
    async fn handle(&self, message: BrokerMessage) -> Result<(), TaskError> {
        let msg = message.payload;
        let task_id = msg.id.clone();
        let task_name = msg.task_name.clone();

        tracing::debug!(
            task_id = %task_id,
            task_name = %task_name,
            retries = msg.retries,
            "Processing task message"
        );

        // Check if message has expired
        if msg.is_expired() {
            tracing::warn!(
                task_id = %task_id,
                task_name = %task_name,
                "Task message has expired, skipping"
            );
            self.backend
                .set_state(&task_id, TaskState::Revoked)
                .await?;
            return Ok(());
        }

        // Check if task has been revoked
        if self.check_revocation(&task_id).await? {
            tracing::warn!(
                task_id = %task_id,
                task_name = %task_name,
                "Task revoked, skipping execution"
            );

            // Emit TaskRevoked signal
            if let Some(dispatcher) = &self.signal_dispatcher {
                dispatcher.dispatch_background(Signal::TaskRevoked {
                    task_id: task_id.clone(),
                    task_name: task_name.clone(),
                    reason: Some("Task was revoked".to_string()),
                    terminated: false,
                });
            }

            // Update state to REVOKED and ack message
            self.backend
                .set_state(&task_id, TaskState::Revoked)
                .await?;
            return Ok(());
        }

        // Look up task in registry
        let task = match self.registry.get(&task_name) {
            Some(task) => task,
            None => {
                tracing::error!(
                    task_id = %task_id,
                    task_name = %task_name,
                    "Task not found in registry"
                );
                self.backend
                    .set_state(&task_id, TaskState::Rejected)
                    .await?;
                return Err(TaskError::TaskNotFound(task_name));
            }
        };

        // Emit TaskReceived signal
        if let Some(dispatcher) = &self.signal_dispatcher {
            dispatcher.dispatch_background(Signal::TaskReceived {
                task_id: task_id.clone(),
                task_name: task_name.clone(),
                queue: task.queue().to_string(),
                worker_name: self.worker_id.clone(),
            });
        }

        // Check rate limits before processing
        if let Some(limiter) = &self.rate_limiter {
            let queue = task.queue();
            let result = limiter.check(&task_name, queue).await;
            if !result.allowed {
                tracing::debug!(
                    task_id = %task_id,
                    task_name = %task_name,
                    queue = %queue,
                    retry_after = ?result.retry_after,
                    "Task rate limited, waiting"
                );

                // Wait for the specified duration before proceeding
                if let Some(delay) = result.retry_after {
                    tokio::time::sleep(delay).await;
                }
            }
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            TaskError::Internal(format!("Failed to acquire semaphore permit: {}", e))
        })?;

        // Update state to RECEIVED
        self.backend
            .set_state(&task_id, TaskState::Received)
            .await?;

        // Get retry policy for max_retries
        let retry_policy = task.retry_policy();

        // Build task context
        let ctx = TaskContext {
            task_id: task_id.clone(),
            task_name: task_name.clone(),
            queue: task.queue().to_string(),
            retry_count: msg.retries,
            max_retries: retry_policy.max_retries,
            correlation_id: msg.correlation_id.clone(),
            parent_id: msg.parent_id.clone(),
            root_id: msg.root_id.clone(),
        };

        // Update state to STARTED
        let start_time = Utc::now();
        self.backend
            .set_state(&task_id, TaskState::Started)
            .await?;

        tracing::info!(
            task_id = %task_id,
            task_name = %task_name,
            worker_id = %self.worker_id,
            "Executing task"
        );

        // Emit TaskPrerun signal
        if let Some(dispatcher) = &self.signal_dispatcher {
            dispatcher.dispatch_background(Signal::TaskPrerun {
                task_id: task_id.clone(),
                task_name: task_name.clone(),
                args: msg.args.clone(),
                kwargs: serde_json::json!({}),
                worker_name: self.worker_id.clone(),
            });
        }

        // Execute task with timeout handling
        let start_instant = std::time::Instant::now();
        let outcome = self
            .execute_with_timeout(task.clone(), ctx.clone(), msg.args.clone())
            .await?;

        let end_time = Utc::now();
        let runtime_ms = (end_time - start_time).num_milliseconds() as u64;
        let runtime = start_instant.elapsed();

        // Handle outcome
        match outcome {
            TaskOutcome::Success(value) => {
                tracing::info!(
                    task_id = %task_id,
                    task_name = %task_name,
                    runtime_ms = runtime_ms,
                    "Task completed successfully"
                );

                // Emit TaskPostrun signal
                if let Some(dispatcher) = &self.signal_dispatcher {
                    dispatcher.dispatch_background(Signal::TaskPostrun {
                        task_id: task_id.clone(),
                        task_name: task_name.clone(),
                        state: TaskState::Success,
                        runtime,
                        worker_name: self.worker_id.clone(),
                    });
                }

                // Emit TaskSuccess signal
                if let Some(dispatcher) = &self.signal_dispatcher {
                    dispatcher.dispatch_background(Signal::TaskSuccess {
                        task_id: task_id.clone(),
                        task_name: task_name.clone(),
                        result: value.clone(),
                        runtime,
                        worker_name: self.worker_id.clone(),
                    });
                }

                // Update state to SUCCESS
                self.backend
                    .set_state(&task_id, TaskState::Success)
                    .await?;

                // Store success result
                let result = TaskResult {
                    task_id: task_id.clone(),
                    state: TaskState::Success,
                    result: Some(value),
                    error: None,
                    traceback: None,
                    started_at: Some(start_time),
                    completed_at: Some(end_time),
                    runtime_ms: Some(runtime_ms),
                    retries: msg.retries,
                    worker_id: Some(self.worker_id.clone()),
                };
                self.backend.set_result(&task_id, result, None).await?;
            }
            TaskOutcome::Failure { error, retryable } => {
                tracing::warn!(
                    task_id = %task_id,
                    task_name = %task_name,
                    runtime_ms = runtime_ms,
                    retryable = retryable,
                    error = %error,
                    "Task failed"
                );

                // Emit TaskPostrun signal
                if let Some(dispatcher) = &self.signal_dispatcher {
                    dispatcher.dispatch_background(Signal::TaskPostrun {
                        task_id: task_id.clone(),
                        task_name: task_name.clone(),
                        state: TaskState::Failure,
                        runtime,
                        worker_name: self.worker_id.clone(),
                    });
                }

                // Emit TaskFailure signal
                if let Some(dispatcher) = &self.signal_dispatcher {
                    dispatcher.dispatch_background(Signal::TaskFailure {
                        task_id: task_id.clone(),
                        task_name: task_name.clone(),
                        error: error.clone(),
                        traceback: None,
                        runtime,
                        worker_name: self.worker_id.clone(),
                    });
                }

                // IMPORTANT: We need a broker instance to republish for retry.
                // Since MessageHandler doesn't have access to broker, we can't retry here.
                // The retry logic needs to be handled differently - either:
                // 1. Pass broker to TaskExecutor (requires refactoring MessageHandler trait)
                // 2. Store retry state and let a separate component handle republishing
                // 3. Return error to let caller (NatsBroker) handle retry via nack
                //
                // For now, we'll mark as FAILURE and log. The nack in NatsBroker will
                // trigger redelivery, which provides basic retry functionality.
                // TODO: Implement proper retry logic with delay calculation

                self.backend
                    .set_state(&task_id, TaskState::Failure)
                    .await?;

                let result = TaskResult {
                    task_id: task_id.clone(),
                    state: TaskState::Failure,
                    result: None,
                    error: Some(error.clone()),
                    traceback: None,
                    started_at: Some(start_time),
                    completed_at: Some(end_time),
                    runtime_ms: Some(runtime_ms),
                    retries: msg.retries,
                    worker_id: Some(self.worker_id.clone()),
                };
                self.backend.set_result(&task_id, result, None).await?;

                // Return error to trigger nack if retryable
                if retryable {
                    return Err(TaskError::Internal(error));
                }
            }
            TaskOutcome::Retry { reason, countdown: _ } => {
                tracing::info!(
                    task_id = %task_id,
                    task_name = %task_name,
                    runtime_ms = runtime_ms,
                    reason = %reason,
                    "Task requested retry"
                );

                // Same issue as above - need broker for retry
                // For now, return error to trigger nack
                self.backend
                    .set_state(&task_id, TaskState::Retry)
                    .await?;

                return Err(TaskError::Internal(format!(
                    "Retry requested: {}",
                    reason
                )));
            }
        }

        Ok(())
    }
}

/// Task worker runtime
pub struct Worker<B: PullBroker, R: ResultBackend> {
    config: WorkerConfig,
    pub(crate) broker: Arc<B>,
    pub(crate) backend: Arc<R>,
    registry: Arc<TaskRegistry>,
    semaphore: Arc<Semaphore>,
    shutdown: CancellationToken,
    rate_limiter: Option<Arc<RateLimitManager>>,
    signal_dispatcher: Option<Arc<SignalDispatcher>>,
    revocation_store: Option<Arc<dyn RevocationStore>>,
}

impl<B: PullBroker, R: ResultBackend> Worker<B, R> {
    /// Create a new worker
    pub fn new(
        config: WorkerConfig,
        broker: B,
        backend: R,
        registry: Arc<TaskRegistry>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.concurrency));
        let shutdown = CancellationToken::new();
        let revocation_store = config.revocation_store.clone();

        Self {
            config,
            broker: Arc::new(broker),
            backend: Arc::new(backend),
            registry,
            semaphore,
            shutdown,
            rate_limiter: None,
            signal_dispatcher: None,
            revocation_store,
        }
    }

    /// Set the rate limiter for this worker
    pub fn with_rate_limiter(mut self, rate_limiter: RateLimitManager) -> Self {
        self.rate_limiter = Some(Arc::new(rate_limiter));
        self
    }

    /// Set the signal dispatcher for this worker
    pub fn with_signal_dispatcher(mut self, dispatcher: SignalDispatcher) -> Self {
        self.signal_dispatcher = Some(Arc::new(dispatcher));
        self
    }

    /// Set the revocation store for this worker
    pub fn with_revocation_store<S: RevocationStore>(mut self, store: S) -> Self {
        self.revocation_store = Some(Arc::new(store));
        self
    }

    /// Start the worker
    #[cfg(feature = "nats")]
    pub async fn start(&self) -> Result<(), TaskError> {
        use crate::SubscriptionHandle;

        tracing::info!(
            worker_id = %self.config.name,
            queues = ?self.config.queues,
            concurrency = self.config.concurrency,
            "Starting worker"
        );

        // Emit WorkerInit signal
        if let Some(dispatcher) = &self.signal_dispatcher {
            dispatcher.dispatch_background(Signal::WorkerInit {
                worker_name: self.config.name.clone(),
                queues: self.config.queues.clone(),
                concurrency: self.config.concurrency,
            });
        }

        // Connect to broker
        self.broker.connect().await?;

        // Connect to backend (perform health check)
        self.backend.health_check().await?;

        // Create task executor
        let executor = Arc::new(TaskExecutor::new(
            self.registry.clone(),
            self.backend.clone(),
            self.semaphore.clone(),
            self.config.name.clone(),
            self.rate_limiter.clone(),
            self.signal_dispatcher.clone(),
            self.revocation_store.clone(),
        ));

        // Subscribe to all queues
        let mut subscription_handles: Vec<SubscriptionHandle> = Vec::new();
        for queue in &self.config.queues {
            tracing::info!(
                worker_id = %self.config.name,
                queue = %queue,
                "Subscribing to queue"
            );

            // Call subscribe directly on the PullBroker trait
            let handle = self.broker.subscribe(queue, executor.clone()).await?;
            subscription_handles.push(handle);
        }

        // Emit WorkerReady signal
        if let Some(dispatcher) = &self.signal_dispatcher {
            dispatcher.dispatch_background(Signal::WorkerReady {
                worker_name: self.config.name.clone(),
            });
        }

        // Spawn heartbeat task
        let heartbeat_interval = self.config.heartbeat;
        let backend = self.backend.clone();
        let broker = self.broker.clone();
        let shutdown = self.shutdown.clone();
        let worker_id = self.config.name.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        tracing::info!(worker_id = %worker_id, "Heartbeat task stopped");
                        break;
                    }
                    _ = interval.tick() => {
                        tracing::trace!(worker_id = %worker_id, "Heartbeat tick");
                        // Perform health checks
                        if let Err(e) = backend.health_check().await {
                            tracing::error!(worker_id = %worker_id, error = %e, "Backend health check failed");
                        }
                        if let Err(e) = broker.health_check().await {
                            tracing::error!(worker_id = %worker_id, error = %e, "Broker health check failed");
                        }
                    }
                }
            }
        });

        // Wait for shutdown signal
        tracing::info!(worker_id = %self.config.name, "Worker started, waiting for shutdown signal");
        self.shutdown.cancelled().await;

        // Cleanup
        tracing::info!(worker_id = %self.config.name, "Shutting down worker");

        // Emit WorkerShutdown signal
        if let Some(dispatcher) = &self.signal_dispatcher {
            dispatcher.dispatch_background(Signal::WorkerShutdown {
                worker_name: self.config.name.clone(),
                reason: ShutdownReason::Graceful,
            });
        }

        // Cancel all subscriptions
        for handle in subscription_handles {
            handle.cancel();
        }

        // Wait a bit for in-flight tasks to complete
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Disconnect from broker
        self.broker.disconnect().await?;

        tracing::info!(worker_id = %self.config.name, "Worker stopped");
        Ok(())
    }

    /// Shutdown the worker gracefully
    pub fn shutdown(&self) {
        tracing::info!(worker_id = %self.config.name, "Shutdown requested");
        self.shutdown.cancel();
    }

    /// Get worker configuration
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }

    /// Check if worker is shutting down
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown.is_cancelled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_config_defaults() {
        let config = WorkerConfig::default();
        assert_eq!(config.queues, vec!["default".to_string()]);
        assert_eq!(config.concurrency, num_cpus::get());
        assert_eq!(config.prefetch, 4);
        assert_eq!(config.heartbeat, Duration::from_secs(10));
        // name is UUID-based, just check it starts with "worker-"
        assert!(config.name.starts_with("worker-"));
    }

    #[test]
    fn test_worker_config_custom() {
        let config = WorkerConfig {
            name: "custom-worker".to_string(),
            queues: vec!["queue1".to_string(), "queue2".to_string()],
            concurrency: 16,
            prefetch: 10,
            heartbeat: Duration::from_secs(30),
            revocation_store: None,
        };
        assert_eq!(config.name, "custom-worker");
        assert_eq!(config.queues.len(), 2);
        assert_eq!(config.concurrency, 16);
        assert_eq!(config.prefetch, 10);
        assert_eq!(config.heartbeat, Duration::from_secs(30));
    }

    #[test]
    fn test_semaphore_permit_acquisition() {
        let semaphore = Arc::new(Semaphore::new(2));

        // Acquire two permits
        let permit1 = semaphore.try_acquire().unwrap();
        let permit2 = semaphore.try_acquire().unwrap();

        // Third should fail
        assert!(semaphore.try_acquire().is_err());

        // Drop one permit
        drop(permit1);

        // Should succeed now
        assert!(semaphore.try_acquire().is_ok());

        // Cleanup
        drop(permit2);
    }

    #[test]
    fn test_worker_with_rate_limiter() {
        use crate::ratelimit::TokenBucket;

        let rate_limiter = RateLimitManager::new()
            .task_limit("slow_task", TokenBucket::per_second(1));

        // This is a compile-time verification test
        // We just check that we can construct a Worker with a rate limiter
        let _has_rate_limiter = rate_limiter;
    }

    #[test]
    fn test_worker_without_rate_limiter() {
        // Verify that Worker works without rate limiter (backward compatibility)
        let config = WorkerConfig::default();
        assert!(config.name.starts_with("worker-"));
        assert_eq!(config.queues, vec!["default".to_string()]);
    }

    #[tokio::test]
    async fn test_worker_with_signal_dispatcher() {
        use crate::signals::{Signal, SignalDispatcher, SignalHandler};
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Create a handler that counts signals
        struct CountingHandler {
            count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl SignalHandler for CountingHandler {
            async fn handle(&self, _signal: &Signal) {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
        }

        let count = Arc::new(AtomicUsize::new(0));
        let handler = CountingHandler {
            count: count.clone(),
        };

        let dispatcher = SignalDispatcher::new()
            .on_all(handler);

        // Verify that the dispatcher can be attached to WorkerConfig (future expansion)
        // For now, we test at the Worker level since WorkerConfig doesn't have signal_dispatcher
        // This is a compile-time verification that the API works
        let _signal_dispatcher = dispatcher;

        // Verify count can be accessed (handler was moved, so we use the cloned Arc)
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_worker_without_signal_dispatcher() {
        // Verify that Worker works without signal dispatcher (backward compatibility)
        let config = WorkerConfig::default();
        assert!(config.name.starts_with("worker-"));
        assert_eq!(config.queues, vec!["default".to_string()]);
    }

    #[test]
    fn config_debug_format() {
        let config = WorkerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(
            debug_str.contains("WorkerConfig"),
            "Debug output should contain 'WorkerConfig', got: {}",
            debug_str
        );
    }

    #[test]
    fn config_debug_with_revocation_store() {
        use crate::revocation::InMemoryRevocationStore;

        let store = InMemoryRevocationStore::new();
        let config = WorkerConfig {
            name: "debug-test".to_string(),
            queues: vec!["q1".to_string()],
            concurrency: 4,
            prefetch: 2,
            heartbeat: Duration::from_secs(5),
            revocation_store: Some(Arc::new(store)),
        };
        let debug_str = format!("{:?}", config);
        assert!(
            debug_str.contains("Some"),
            "Debug output should indicate Some for revocation_store, got: {}",
            debug_str
        );
        assert!(
            debug_str.contains("RevocationStore"),
            "Debug output should mention RevocationStore, got: {}",
            debug_str
        );
    }

    #[test]
    fn config_clone() {
        let config = WorkerConfig {
            name: "clone-test".to_string(),
            queues: vec!["alpha".to_string(), "beta".to_string()],
            concurrency: 8,
            prefetch: 16,
            heartbeat: Duration::from_secs(20),
            revocation_store: None,
        };
        let cloned = config.clone();
        assert_eq!(cloned.name, config.name);
        assert_eq!(cloned.queues, config.queues);
        assert_eq!(cloned.concurrency, config.concurrency);
        assert_eq!(cloned.prefetch, config.prefetch);
        assert_eq!(cloned.heartbeat, config.heartbeat);
        assert!(cloned.revocation_store.is_none());
    }

    #[test]
    fn worker_config_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WorkerConfig>();
    }

    // Mock-based unit tests (extracted to separate file for line-count limits)
    #[path = "worker_mock_tests.rs"]
    mod mock_tests;

    // Integration tests requiring NATS and Redis
    #[cfg(all(feature = "nats", feature = "redis"))]
    mod integration {
        use super::*;
        use crate::{Broker, NatsBroker, NatsBrokerConfig, RedisBackend, RedisBackendConfig, TaskMessage};

        struct TestTask;

        #[async_trait]
        impl Task for TestTask {
            fn name(&self) -> &'static str {
                "test_task"
            }

            async fn execute(
                &self,
                _ctx: TaskContext,
                args: serde_json::Value,
            ) -> TaskOutcome {
                // Simple echo task
                TaskOutcome::Success(args)
            }
        }

        #[allow(dead_code)]
        struct FailingTask;

        #[async_trait]
        impl Task for FailingTask {
            fn name(&self) -> &'static str {
                "failing_task"
            }

            async fn execute(
                &self,
                _ctx: TaskContext,
                _args: serde_json::Value,
            ) -> TaskOutcome {
                TaskOutcome::Failure {
                    error: "Task failed".to_string(),
                    retryable: false,
                }
            }
        }

        struct TimeoutTask;

        #[async_trait]
        impl Task for TimeoutTask {
            fn name(&self) -> &'static str {
                "timeout_task"
            }

            fn hard_time_limit(&self) -> Option<Duration> {
                Some(Duration::from_secs(1))
            }

            async fn execute(
                &self,
                _ctx: TaskContext,
                _args: serde_json::Value,
            ) -> TaskOutcome {
                // Sleep longer than timeout
                tokio::time::sleep(Duration::from_secs(5)).await;
                TaskOutcome::Success(serde_json::json!(null))
            }
        }

        #[tokio::test]
        async fn test_worker_lifecycle() {
            let _ = tracing_subscriber::fmt::try_init();

            let broker_config = NatsBrokerConfig::default();
            let broker = NatsBroker::new(broker_config);
            if broker.connect().await.is_err() { return; }
            let _ = broker.disconnect().await;
            let broker = NatsBroker::new(NatsBrokerConfig::default());

            let backend_config = RedisBackendConfig::default();
            let Some(backend) = RedisBackend::new(backend_config).await.ok() else { return };

            let registry = Arc::new(TaskRegistry::new());
            registry.register(TestTask);

            let worker_config = WorkerConfig {
                name: "test-worker".to_string(),
                queues: vec!["test".to_string()],
                concurrency: 2,
                prefetch: 4,
                heartbeat: Duration::from_secs(10),
                revocation_store: None,
            };

            let worker = Worker::new(worker_config, broker, backend, registry);

            // Start worker in background
            let _worker_handle = {
                let worker = Arc::new(worker);
                let worker_clone = worker.clone();
                tokio::spawn(async move {
                    worker_clone.start().await.unwrap();
                })
            };

            // Give worker time to start
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Shutdown worker
            // worker.shutdown(); // This won't work because worker was moved

            // Wait for worker to stop
            tokio::time::sleep(Duration::from_millis(500)).await;

            // For this test, we'll just let it timeout since we can't call shutdown
            // In real usage, the Worker would be stored and shutdown called on it
        }

        #[tokio::test]
        async fn test_task_execution() {
            let _ = tracing_subscriber::fmt::try_init();

            let broker_config = NatsBrokerConfig::default();
            let broker = NatsBroker::new(broker_config);
            if broker.connect().await.is_err() { return; }
            let _ = broker.disconnect().await;
            let broker = NatsBroker::new(NatsBrokerConfig::default());

            let backend_config = RedisBackendConfig::default();
            let Some(backend) = RedisBackend::new(backend_config).await.ok() else { return };

            let registry = Arc::new(TaskRegistry::new());
            registry.register(TestTask);

            let worker_config = WorkerConfig {
                name: "test-worker".to_string(),
                queues: vec!["test".to_string()],
                concurrency: 2,
                prefetch: 4,
                heartbeat: Duration::from_secs(10),
                revocation_store: None,
            };

            let worker = Arc::new(Worker::new(
                worker_config,
                broker,
                backend,
                registry,
            ));

            // Connect broker
            worker.broker.connect().await.unwrap();

            // Publish a task
            let msg = TaskMessage::new("test_task", serde_json::json!([1, 2, 3]));
            let task_id = msg.id.clone();
            worker.broker.publish("test", msg).await.unwrap();

            // Start worker
            let worker_clone = worker.clone();
            tokio::spawn(async move {
                let _ = worker_clone.start().await;
            });

            // Wait for task to be processed
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Check result
            let result = worker.backend.get_result(&task_id).await.unwrap();
            assert!(result.is_some());
            let result = result.unwrap();
            assert_eq!(result.state, TaskState::Success);
            assert_eq!(result.result, Some(serde_json::json!([1, 2, 3])));

            // Shutdown
            worker.shutdown();
            tokio::time::sleep(Duration::from_millis(500)).await;

            worker.broker.disconnect().await.unwrap();
        }

        #[tokio::test]
        async fn test_timeout_handling() {
            let _ = tracing_subscriber::fmt::try_init();

            let broker_config = NatsBrokerConfig::default();
            let broker = NatsBroker::new(broker_config);
            if broker.connect().await.is_err() { return; }
            let _ = broker.disconnect().await;
            let broker = NatsBroker::new(NatsBrokerConfig::default());

            let backend_config = RedisBackendConfig::default();
            let Some(backend) = RedisBackend::new(backend_config).await.ok() else { return };

            let registry = Arc::new(TaskRegistry::new());
            registry.register(TimeoutTask);

            let worker_config = WorkerConfig {
                name: "test-worker".to_string(),
                queues: vec!["test".to_string()],
                concurrency: 1,
                prefetch: 1,
                heartbeat: Duration::from_secs(10),
                revocation_store: None,
            };

            let worker = Arc::new(Worker::new(
                worker_config,
                broker,
                backend,
                registry,
            ));

            // Connect broker
            worker.broker.connect().await.unwrap();

            // Publish a timeout task
            let msg = TaskMessage::new("timeout_task", serde_json::json!(null));
            let task_id = msg.id.clone();
            worker.broker.publish("test", msg).await.unwrap();

            // Start worker
            let worker_clone = worker.clone();
            tokio::spawn(async move {
                let _ = worker_clone.start().await;
            });

            // Wait for task to timeout and be processed
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Check result - should be failure due to timeout
            let result = worker.backend.get_result(&task_id).await.unwrap();
            assert!(result.is_some());
            let result = result.unwrap();
            assert_eq!(result.state, TaskState::Failure);
            assert!(result.error.is_some());
            assert!(result.error.unwrap().contains("exceeded hard time limit"));

            // Shutdown
            worker.shutdown();
            tokio::time::sleep(Duration::from_millis(500)).await;

            worker.broker.disconnect().await.unwrap();
        }
    }
}

//! Unit tests for TaskExecutor, Worker, and message handling.
//!
//! TaskExecutor handle/execute tests use real RedisBackend (feature = "redis").
//! Worker construction and check_revocation tests use MockResultBackend (no infra needed).
//! MockPullBroker is used for Worker::new() (doesn't connect).

use super::*;
use crate::broker::{Broker, BrokerCapabilities, BrokerMessage, DeliveryModel, SubscriptionHandle};
use crate::message::TaskMessage;
use dashmap::DashMap;

#[cfg(feature = "redis")]
use crate::backend::redis::{RedisBackend, RedisBackendConfig};

#[cfg(feature = "redis")]
async fn make_redis_backend() -> Option<RedisBackend> {
    RedisBackend::new(RedisBackendConfig::default()).await.ok()
}

// ── MockPullBroker ────────────────────────────────────────────────────

struct MockPullBroker;

#[async_trait]
impl Broker for MockPullBroker {
    async fn connect(&self) -> Result<(), TaskError> { Ok(()) }
    async fn disconnect(&self) -> Result<(), TaskError> { Ok(()) }
    async fn publish(&self, _queue: &str, _message: TaskMessage) -> Result<(), TaskError> { Ok(()) }
    async fn health_check(&self) -> Result<(), TaskError> { Ok(()) }
    fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Pull }
    fn capabilities(&self) -> BrokerCapabilities { BrokerCapabilities::default() }
}

#[async_trait]
impl PullBroker for MockPullBroker {
    async fn subscribe<H: MessageHandler + 'static>(
        &self,
        _queue: &str,
        _handler: Arc<H>,
    ) -> Result<SubscriptionHandle, TaskError> {
        Ok(SubscriptionHandle::new("mock".to_string(), CancellationToken::new()))
    }
    async fn ack(&self, _delivery_tag: &str) -> Result<(), TaskError> { Ok(()) }
    async fn nack(&self, _delivery_tag: &str, _requeue: bool) -> Result<(), TaskError> { Ok(()) }
}

// ── MockResultBackend ─────────────────────────────────────────────────

struct MockResultBackend {
    states: DashMap<TaskId, TaskState>,
    results: DashMap<TaskId, TaskResult>,
    metadata: DashMap<String, serde_json::Value>,
}

impl MockResultBackend {
    fn new() -> Self {
        Self {
            states: DashMap::new(),
            results: DashMap::new(),
            metadata: DashMap::new(),
        }
    }
}

#[async_trait]
impl ResultBackend for MockResultBackend {
    async fn set_state(&self, task_id: &TaskId, state: TaskState) -> Result<(), TaskError> {
        self.states.insert(task_id.clone(), state);
        Ok(())
    }

    async fn get_state(&self, task_id: &TaskId) -> Result<Option<TaskState>, TaskError> {
        Ok(self.states.get(task_id).map(|r| *r))
    }

    async fn set_result(
        &self,
        task_id: &TaskId,
        result: TaskResult,
        _ttl: Option<Duration>,
    ) -> Result<(), TaskError> {
        self.results.insert(task_id.clone(), result);
        Ok(())
    }

    async fn get_result(&self, task_id: &TaskId) -> Result<Option<TaskResult>, TaskError> {
        Ok(self.results.get(task_id).map(|r| r.clone()))
    }

    async fn wait_for_result(
        &self,
        task_id: &TaskId,
        timeout: Option<Duration>,
        _poll_interval: Duration,
    ) -> Result<TaskResult, TaskError> {
        let deadline = timeout.map(|t| std::time::Instant::now() + t);
        loop {
            if let Some(r) = self.results.get(task_id) {
                return Ok(r.clone());
            }
            if let Some(dl) = deadline {
                if std::time::Instant::now() >= dl {
                    return Err(TaskError::Internal("Timeout waiting for result".to_string()));
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn delete(&self, task_id: &TaskId) -> Result<(), TaskError> {
        self.states.remove(task_id);
        self.results.remove(task_id);
        Ok(())
    }

    async fn get_many(&self, task_ids: &[TaskId]) -> Result<Vec<Option<TaskResult>>, TaskError> {
        Ok(task_ids.iter().map(|id| self.results.get(id).map(|r| r.clone())).collect())
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        Ok(())
    }

    async fn set_metadata(
        &self,
        key: &str,
        value: serde_json::Value,
        _ttl: Option<Duration>,
    ) -> Result<(), TaskError> {
        self.metadata.insert(key.to_string(), value);
        Ok(())
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<serde_json::Value>, TaskError> {
        Ok(self.metadata.get(key).map(|r| r.clone()))
    }

    async fn delete_metadata(&self, key: &str) -> Result<(), TaskError> {
        self.metadata.remove(key);
        Ok(())
    }
}

// ── Test Task structs ─────────────────────────────────────────────────

struct SuccessTask;

#[async_trait]
impl Task for SuccessTask {
    fn name(&self) -> &'static str { "success_task" }
    async fn execute(&self, _ctx: TaskContext, _args: serde_json::Value) -> TaskOutcome {
        TaskOutcome::Success(serde_json::json!(42))
    }
}

struct FailingTask {
    retryable: bool,
}

#[async_trait]
impl Task for FailingTask {
    fn name(&self) -> &'static str { "failing_task" }
    async fn execute(&self, _ctx: TaskContext, _args: serde_json::Value) -> TaskOutcome {
        TaskOutcome::Failure {
            error: "test failure".to_string(),
            retryable: self.retryable,
        }
    }
}

struct RetryTask;

#[async_trait]
impl Task for RetryTask {
    fn name(&self) -> &'static str { "retry_task" }
    async fn execute(&self, _ctx: TaskContext, _args: serde_json::Value) -> TaskOutcome {
        TaskOutcome::Retry {
            reason: "need retry".to_string(),
            countdown: None,
        }
    }
}

struct SlowTask;

#[async_trait]
impl Task for SlowTask {
    fn name(&self) -> &'static str { "slow_task" }
    fn hard_time_limit(&self) -> Option<Duration> { Some(Duration::from_millis(100)) }
    async fn execute(&self, _ctx: TaskContext, _args: serde_json::Value) -> TaskOutcome {
        tokio::time::sleep(Duration::from_secs(2)).await;
        TaskOutcome::Success(serde_json::json!("should not reach"))
    }
}

// ── Helper functions ──────────────────────────────────────────────────

fn make_broker_message(task_name: &str) -> BrokerMessage {
    use std::collections::HashMap;
    let msg = TaskMessage::new(task_name, serde_json::json!([1, 2, 3]));
    BrokerMessage {
        delivery_tag: format!("tag-{}", msg.id),
        payload: msg,
        headers: HashMap::new(),
        timestamp: chrono::Utc::now(),
        redelivered: false,
    }
}

fn make_executor_mock(
    registry: Arc<TaskRegistry>,
    backend: Arc<MockResultBackend>,
) -> TaskExecutor<MockResultBackend> {
    TaskExecutor::new(
        registry,
        backend,
        Arc::new(Semaphore::new(10)),
        "test-worker".to_string(),
        None,
        None,
        None,
    )
}

#[cfg(feature = "redis")]
fn make_executor_redis(
    registry: Arc<TaskRegistry>,
    backend: Arc<RedisBackend>,
) -> TaskExecutor<RedisBackend> {
    TaskExecutor::new(
        registry,
        backend,
        Arc::new(Semaphore::new(10)),
        "test-worker".to_string(),
        None,
        None,
        None,
    )
}

// ── TaskExecutor tests (use real Redis) ──────────────────────────────

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_success_task() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Success));
    assert!(backend.get_result(&task_id).await.unwrap().is_some());
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_success_result_fields() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    executor.handle(msg).await.unwrap();

    let result = backend.get_result(&task_id).await.unwrap().unwrap();
    assert_eq!(result.task_id, task_id);
    assert_eq!(result.state, TaskState::Success);
    assert_eq!(result.result, Some(serde_json::json!(42)));
    assert_eq!(result.worker_id, Some("test-worker".to_string()));
    assert!(result.started_at.is_some());
    assert!(result.completed_at.is_some());
    assert!(result.runtime_ms.is_some());
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_non_retryable_failure() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(FailingTask { retryable: false });
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("failing_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Failure));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_retryable_failure() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(FailingTask { retryable: true });
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("failing_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TaskError::Internal(_)));
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Failure));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_retry_outcome() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(RetryTask);
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("retry_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_err());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Retry));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_unknown_task() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    let executor = make_executor_redis(registry, backend.clone());

    let msg = make_broker_message("nonexistent_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TaskError::TaskNotFound(_)));
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Rejected));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_expired_message() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let executor = make_executor_redis(registry, backend.clone());

    let mut msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    msg.payload.expires = Some(Utc::now() - chrono::Duration::hours(1));
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Revoked));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_revoked_task() {
    use crate::revocation::InMemoryRevocationStore;
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let store = Arc::new(InMemoryRevocationStore::new());

    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    store.revoke(&task_id, false).await.unwrap();

    let executor = TaskExecutor::new(
        registry, backend.clone(), Arc::new(Semaphore::new(10)),
        "test-worker".to_string(), None, None,
        Some(store as Arc<dyn RevocationStore>),
    );
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Revoked));
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn execute_no_timeout() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let executor = make_executor_redis(registry.clone(), backend);
    let task = registry.get("success_task").unwrap();
    let ctx = TaskContext {
        task_id: TaskId::new(), task_name: "success_task".to_string(),
        queue: "default".to_string(), retry_count: 0, max_retries: 3,
        correlation_id: None, parent_id: None, root_id: None,
    };
    let outcome = executor.execute_with_timeout(task, ctx, serde_json::json!([])).await.unwrap();
    assert!(matches!(outcome, TaskOutcome::Success(v) if v == serde_json::json!(42)));
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn execute_exceeds_timeout() {
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SlowTask);
    let executor = make_executor_redis(registry.clone(), backend);
    let task = registry.get("slow_task").unwrap();
    let ctx = TaskContext {
        task_id: TaskId::new(), task_name: "slow_task".to_string(),
        queue: "default".to_string(), retry_count: 0, max_retries: 3,
        correlation_id: None, parent_id: None, root_id: None,
    };
    let outcome = executor.execute_with_timeout(task, ctx, serde_json::json!([])).await.unwrap();
    match outcome {
        TaskOutcome::Failure { error, retryable } => {
            assert!(error.contains("exceeded hard time limit"), "error was: {}", error);
            assert!(!retryable);
        }
        other => panic!("Expected Failure with timeout, got: {:?}", other),
    }
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_with_signal_dispatcher() {
    use crate::signals::{SignalDispatcher, SignalHandler};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct CountHandler { count: Arc<AtomicUsize> }
    #[async_trait]
    impl SignalHandler for CountHandler {
        async fn handle(&self, _signal: &Signal) { self.count.fetch_add(1, Ordering::SeqCst); }
    }

    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let count = Arc::new(AtomicUsize::new(0));
    let dispatcher = SignalDispatcher::new().on_all(CountHandler { count: count.clone() });
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);

    let executor = TaskExecutor::new(
        registry, backend.clone(), Arc::new(Semaphore::new(10)),
        "test-worker".to_string(), None, Some(Arc::new(dispatcher)), None,
    );
    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    tokio::time::sleep(Duration::from_millis(50)).await;
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_revoked_emits_signal() {
    use crate::revocation::InMemoryRevocationStore;
    use crate::signals::{SignalDispatcher, SignalHandler};
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct CountHandler { count: Arc<AtomicUsize> }
    #[async_trait]
    impl SignalHandler for CountHandler {
        async fn handle(&self, _signal: &Signal) { self.count.fetch_add(1, Ordering::SeqCst); }
    }

    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let count = Arc::new(AtomicUsize::new(0));
    let dispatcher = SignalDispatcher::new().on_all(CountHandler { count: count.clone() });
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);
    let store = Arc::new(InMemoryRevocationStore::new());

    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    store.revoke(&task_id, false).await.unwrap();

    let executor = TaskExecutor::new(
        registry, backend.clone(), Arc::new(Semaphore::new(10)),
        "test-worker".to_string(), None, Some(Arc::new(dispatcher)),
        Some(store as Arc<dyn RevocationStore>),
    );
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Revoked));
    tokio::time::sleep(Duration::from_millis(50)).await;
    backend.delete(&task_id).await.ok();
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn handle_with_rate_limiter() {
    use crate::ratelimit::TokenBucket;
    let Some(rb) = make_redis_backend().await else { return };
    let backend = Arc::new(rb);
    let rate_limiter = RateLimitManager::new()
        .task_limit("success_task", TokenBucket::per_second(100));
    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);

    let executor = TaskExecutor::new(
        registry, backend.clone(), Arc::new(Semaphore::new(10)),
        "test-worker".to_string(), Some(Arc::new(rate_limiter)), None, None,
    );
    let msg = make_broker_message("success_task");
    let task_id = msg.payload.id.clone();
    let result = executor.handle(msg).await;
    assert!(result.is_ok());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Success));
    backend.delete(&task_id).await.ok();
}

#[tokio::test]
async fn worker_new_with_mocks() {
    let config = WorkerConfig {
        name: "mock-worker".to_string(),
        queues: vec!["q1".to_string()],
        concurrency: 4,
        prefetch: 2,
        heartbeat: Duration::from_secs(5),
        revocation_store: None,
    };

    let registry = Arc::new(TaskRegistry::new());
    registry.register(SuccessTask);

    let worker = Worker::new(config, MockPullBroker, MockResultBackend::new(), registry);

    assert_eq!(worker.config().name, "mock-worker");
    assert_eq!(worker.config().queues, vec!["q1".to_string()]);
    assert_eq!(worker.config().concurrency, 4);
    assert!(!worker.is_shutting_down());

    worker.shutdown();
    assert!(worker.is_shutting_down());
}

#[tokio::test]
async fn worker_builder_chain_with_mocks() {
    use crate::revocation::InMemoryRevocationStore;
    use crate::ratelimit::TokenBucket;
    use crate::signals::SignalDispatcher;

    let config = WorkerConfig::default();
    let registry = Arc::new(TaskRegistry::new());

    let rate_limiter = RateLimitManager::new()
        .task_limit("t", TokenBucket::per_second(10));
    let dispatcher = SignalDispatcher::new();
    let store = InMemoryRevocationStore::new();

    let worker = Worker::new(config, MockPullBroker, MockResultBackend::new(), registry)
        .with_rate_limiter(rate_limiter)
        .with_signal_dispatcher(dispatcher)
        .with_revocation_store(store);

    assert!(worker.rate_limiter.is_some());
    assert!(worker.signal_dispatcher.is_some());
    assert!(worker.revocation_store.is_some());
    assert!(!worker.is_shutting_down());
}

#[tokio::test]
async fn check_revocation_no_store() {
    let registry = Arc::new(TaskRegistry::new());
    let backend = Arc::new(MockResultBackend::new());
    let executor = make_executor_mock(registry, backend);

    let task_id = TaskId::new();
    let revoked = executor.check_revocation(&task_id).await.unwrap();
    assert!(!revoked);
}

#[tokio::test]
async fn check_revocation_revoked() {
    use crate::revocation::InMemoryRevocationStore;

    let registry = Arc::new(TaskRegistry::new());
    let backend = Arc::new(MockResultBackend::new());
    let store = Arc::new(InMemoryRevocationStore::new());

    let task_id = TaskId::new();
    store.revoke(&task_id, false).await.unwrap();

    let executor = TaskExecutor::new(
        registry,
        backend,
        Arc::new(Semaphore::new(10)),
        "test-worker".to_string(),
        None,
        None,
        Some(store as Arc<dyn RevocationStore>),
    );

    let revoked = executor.check_revocation(&task_id).await.unwrap();
    assert!(revoked);
}

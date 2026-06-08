//! Celery-style handle for tracking a single task result.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::{ResultBackend, TaskError, TaskId, TaskResult, TaskState};

/// Handle returned to callers after enqueuing a task.
///
/// This mirrors Celery's `AsyncResult`: it is a lightweight task-id handle that
/// can query any queuekit [`ResultBackend`] for state or wait for the final
/// value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncResult {
    pub task_id: TaskId,
}

impl AsyncResult {
    /// Create a result handle from a task id.
    pub fn new(task_id: TaskId) -> Self {
        Self { task_id }
    }

    /// Parse a task id string and create a result handle.
    pub fn from_string(task_id: &str) -> Result<Self, TaskError> {
        Ok(Self::new(TaskId::from_string(task_id)?))
    }

    /// Alias for Celery-style `id`.
    pub fn id(&self) -> &TaskId {
        &self.task_id
    }

    /// Return the current task state, falling back to stored result metadata.
    pub async fn state<R: ResultBackend>(
        &self,
        backend: &R,
    ) -> Result<Option<TaskState>, TaskError> {
        if let Some(state) = backend.get_state(&self.task_id).await? {
            return Ok(Some(state));
        }
        Ok(backend
            .get_result(&self.task_id)
            .await?
            .map(|result| result.state))
    }

    /// Return the full stored result if present.
    pub async fn result<R: ResultBackend>(
        &self,
        backend: &R,
    ) -> Result<Option<TaskResult>, TaskError> {
        backend.get_result(&self.task_id).await
    }

    /// Check whether this task is in a terminal state.
    pub async fn ready<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        Ok(self
            .state(backend)
            .await?
            .is_some_and(|state| state.is_terminal()))
    }

    /// Check whether this task completed successfully.
    pub async fn successful<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        Ok(matches!(
            self.state(backend).await?,
            Some(TaskState::Success)
        ))
    }

    /// Check whether this task failed permanently.
    pub async fn failed<R: ResultBackend>(&self, backend: &R) -> Result<bool, TaskError> {
        Ok(matches!(
            self.state(backend).await?,
            Some(TaskState::Failure)
        ))
    }

    /// Wait for the task to finish and return its successful JSON value.
    pub async fn get<R: ResultBackend>(
        &self,
        backend: &R,
        timeout: Option<Duration>,
    ) -> Result<serde_json::Value, TaskError> {
        let poll_interval = Duration::from_millis(100);
        let result = backend
            .wait_for_result(&self.task_id, timeout, poll_interval)
            .await?;
        result_value(result)
    }

    /// Blocking form of [`Self::state`].
    pub fn state_blocking<R>(&self, backend: Arc<R>) -> Result<Option<TaskState>, TaskError>
    where
        R: ResultBackend,
    {
        let result = self.clone();
        run_async_result_blocking(async move { result.state(backend.as_ref()).await })
    }

    /// Blocking form of [`Self::ready`].
    pub fn ready_blocking<R>(&self, backend: Arc<R>) -> Result<bool, TaskError>
    where
        R: ResultBackend,
    {
        let result = self.clone();
        run_async_result_blocking(async move { result.ready(backend.as_ref()).await })
    }

    /// Blocking form of [`Self::successful`].
    pub fn successful_blocking<R>(&self, backend: Arc<R>) -> Result<bool, TaskError>
    where
        R: ResultBackend,
    {
        let result = self.clone();
        run_async_result_blocking(async move { result.successful(backend.as_ref()).await })
    }

    /// Blocking form of [`Self::failed`].
    pub fn failed_blocking<R>(&self, backend: Arc<R>) -> Result<bool, TaskError>
    where
        R: ResultBackend,
    {
        let result = self.clone();
        run_async_result_blocking(async move { result.failed(backend.as_ref()).await })
    }

    /// Blocking form of [`Self::get`].
    pub fn get_blocking<R>(
        &self,
        backend: Arc<R>,
        timeout: Option<Duration>,
    ) -> Result<serde_json::Value, TaskError>
    where
        R: ResultBackend,
    {
        let result = self.clone();
        run_async_result_blocking(async move { result.get(backend.as_ref(), timeout).await })
    }
}

fn result_value(result: TaskResult) -> Result<serde_json::Value, TaskError> {
    match result.state {
        TaskState::Success => result
            .result
            .ok_or_else(|| TaskError::Internal("Success state but no result".to_string())),
        TaskState::Failure => Err(TaskError::Internal(
            result.error.unwrap_or_else(|| "Task failed".to_string()),
        )),
        TaskState::Revoked => Err(TaskError::Revoked(result.task_id.to_string())),
        TaskState::Rejected => Err(TaskError::Internal(format!(
            "Task {} was rejected",
            result.task_id
        ))),
        other => Err(TaskError::Internal(format!(
            "Unexpected task state: {:?}",
            other
        ))),
    }
}

fn run_async_result_blocking<Fut, T>(future: Fut) -> Result<T, TaskError>
where
    Fut: Future<Output = Result<T, TaskError>> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                TaskError::Internal(format!("failed to create async result runtime: {err}"))
            })?;
        runtime.block_on(future)
    })
    .join()
    .map_err(|_| TaskError::Internal("async result blocking wait panicked".to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    #[derive(Default)]
    struct MemoryBackend {
        states: RwLock<HashMap<TaskId, TaskState>>,
        results: RwLock<HashMap<TaskId, TaskResult>>,
        metadata: RwLock<HashMap<String, serde_json::Value>>,
    }

    #[async_trait]
    impl ResultBackend for MemoryBackend {
        async fn set_state(&self, task_id: &TaskId, state: TaskState) -> Result<(), TaskError> {
            self.states.write().await.insert(task_id.clone(), state);
            Ok(())
        }

        async fn get_state(&self, task_id: &TaskId) -> Result<Option<TaskState>, TaskError> {
            Ok(self.states.read().await.get(task_id).copied())
        }

        async fn set_result(
            &self,
            task_id: &TaskId,
            result: TaskResult,
            _ttl: Option<Duration>,
        ) -> Result<(), TaskError> {
            self.results.write().await.insert(task_id.clone(), result);
            Ok(())
        }

        async fn get_result(&self, task_id: &TaskId) -> Result<Option<TaskResult>, TaskError> {
            Ok(self.results.read().await.get(task_id).cloned())
        }

        async fn wait_for_result(
            &self,
            task_id: &TaskId,
            _timeout: Option<Duration>,
            _poll_interval: Duration,
        ) -> Result<TaskResult, TaskError> {
            self.get_result(task_id)
                .await?
                .ok_or_else(|| TaskError::TaskNotFound(task_id.to_string()))
        }

        async fn delete(&self, task_id: &TaskId) -> Result<(), TaskError> {
            self.states.write().await.remove(task_id);
            self.results.write().await.remove(task_id);
            Ok(())
        }

        async fn get_many(
            &self,
            task_ids: &[TaskId],
        ) -> Result<Vec<Option<TaskResult>>, TaskError> {
            let results = self.results.read().await;
            Ok(task_ids.iter().map(|id| results.get(id).cloned()).collect())
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
            self.metadata.write().await.insert(key.to_string(), value);
            Ok(())
        }

        async fn get_metadata(&self, key: &str) -> Result<Option<serde_json::Value>, TaskError> {
            Ok(self.metadata.read().await.get(key).cloned())
        }

        async fn delete_metadata(&self, key: &str) -> Result<(), TaskError> {
            self.metadata.write().await.remove(key);
            Ok(())
        }
    }

    #[tokio::test]
    async fn async_result_reports_state_and_success() {
        let backend = MemoryBackend::default();
        let task_id = TaskId::new();
        let handle = AsyncResult::new(task_id.clone());

        assert_eq!(handle.id(), &task_id);
        assert!(!handle.ready(&backend).await.unwrap());

        backend
            .set_result(
                &task_id,
                TaskResult::success(task_id.clone(), serde_json::json!({"ok": true})),
                None,
            )
            .await
            .unwrap();

        assert_eq!(
            handle.state(&backend).await.unwrap(),
            Some(TaskState::Success)
        );
        assert!(handle.ready(&backend).await.unwrap());
        assert!(handle.successful(&backend).await.unwrap());
        assert!(!handle.failed(&backend).await.unwrap());
        assert_eq!(
            handle.get(&backend, None).await.unwrap(),
            serde_json::json!({"ok": true})
        );
    }

    #[tokio::test]
    async fn async_result_failed_tracks_failure_state() {
        let backend = MemoryBackend::default();
        let task_id = TaskId::new();
        let handle = AsyncResult::new(task_id.clone());

        backend
            .set_result(
                &task_id,
                TaskResult::failure(task_id.clone(), "boom".to_string()),
                None,
            )
            .await
            .unwrap();

        assert!(handle.ready(&backend).await.unwrap());
        assert!(!handle.successful(&backend).await.unwrap());
        assert!(handle.failed(&backend).await.unwrap());
        let err = handle.get(&backend, None).await.unwrap_err();
        assert!(err.to_string().contains("boom"));
    }

    #[test]
    fn async_result_blocking_methods_use_backend() {
        let backend = Arc::new(MemoryBackend::default());
        let task_id = TaskId::new();
        let handle = AsyncResult::new(task_id.clone());
        let task_result = TaskResult::success(task_id.clone(), serde_json::json!(42));

        run_async_result_blocking({
            let backend = backend.clone();
            async move { backend.set_result(&task_id, task_result, None).await }
        })
        .unwrap();

        assert_eq!(
            handle.state_blocking(backend.clone()).unwrap(),
            Some(TaskState::Success)
        );
        assert!(handle.ready_blocking(backend.clone()).unwrap());
        assert!(handle.successful_blocking(backend.clone()).unwrap());
        assert!(!handle.failed_blocking(backend.clone()).unwrap());
        assert_eq!(
            handle.get_blocking(backend, None).unwrap(),
            serde_json::json!(42)
        );
    }

    #[test]
    fn async_result_from_string_parses_task_id() {
        let task_id = TaskId::new();
        let handle = AsyncResult::from_string(&task_id.to_string()).unwrap();
        assert_eq!(handle.id(), &task_id);
    }
}

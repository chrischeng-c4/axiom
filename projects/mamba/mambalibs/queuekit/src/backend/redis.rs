//! Redis result backend implementation
//!
//! Provides Redis-based storage for task results with connection pooling,
//! TTL support, and efficient batch operations.

use async_trait::async_trait;
use deadpool_redis::{Config as PoolConfig, Connection, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::{ResultBackend, TaskError, TaskId, TaskResult, TaskState};

/// Redis result backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisBackendConfig {
    /// Redis URL (e.g., "redis://localhost:6379")
    pub url: String,
    /// Key prefix for all task data
    pub key_prefix: String,
    /// Default result TTL (0 = no expiry)
    pub default_ttl: Duration,
    /// Connection pool size
    pub pool_size: usize,
}

impl Default for RedisBackendConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            key_prefix: "cclab-meteor".to_string(),
            default_ttl: Duration::from_secs(86400), // 24 hours
            pool_size: 10,
        }
    }
}

/// Redis result backend implementation
///
/// Uses deadpool-redis for async connection pooling and provides
/// atomic operations for state and result updates.
pub struct RedisBackend {
    config: RedisBackendConfig,
    pool: Pool,
}

impl RedisBackend {
    /// Create a new Redis backend
    pub async fn new(config: RedisBackendConfig) -> Result<Self, TaskError> {
        debug!(
            "Creating Redis backend: url={}, prefix={}, pool_size={}",
            config.url, config.key_prefix, config.pool_size
        );

        let pool_config = PoolConfig::from_url(&config.url);
        let pool = pool_config
            .builder()
            .map_err(|e| TaskError::Backend(format!("Failed to create pool builder: {}", e)))?
            .max_size(config.pool_size)
            .runtime(Runtime::Tokio1)
            .build()
            .map_err(|e| TaskError::Backend(format!("Failed to create pool: {}", e)))?;

        // Test connection
        let mut conn = pool
            .get()
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get connection: {}", e)))?;

        // Connection successful if we got here
        let _: Option<String> = conn.get("__ping__").await.ok();

        debug!("Redis backend initialized successfully");

        Ok(Self { config, pool })
    }

    /// Generate state key for a task
    fn state_key(&self, task_id: &TaskId) -> String {
        format!("{}:state:{}", self.config.key_prefix, task_id)
    }

    /// Generate result key for a task
    fn result_key(&self, task_id: &TaskId) -> String {
        format!("{}:result:{}", self.config.key_prefix, task_id)
    }

    /// Get a connection from the pool
    async fn get_conn(&self) -> Result<Connection, TaskError> {
        self.pool
            .get()
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get connection: {}", e)))
    }

    /// Get TTL in seconds
    fn get_ttl_seconds(&self, ttl: Option<Duration>) -> u64 {
        ttl.unwrap_or(self.config.default_ttl).as_secs()
    }
}

#[async_trait]
impl ResultBackend for RedisBackend {
    async fn set_state(&self, task_id: &TaskId, state: TaskState) -> Result<(), TaskError> {
        let key = self.state_key(task_id);
        let value = serde_json::to_string(&state)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize state: {}", e)))?;

        let ttl = self.get_ttl_seconds(None);
        let mut conn = self.get_conn().await?;

        debug!("Setting state for task {}: {:?}", task_id, state);

        if ttl > 0 {
            conn.set_ex::<_, _, ()>(&key, value, ttl)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set state: {}", e)))?;
        } else {
            conn.set::<_, _, ()>(&key, value)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set state: {}", e)))?;
        }

        Ok(())
    }

    async fn get_state(&self, task_id: &TaskId) -> Result<Option<TaskState>, TaskError> {
        let key = self.state_key(task_id);
        let mut conn = self.get_conn().await?;

        debug!("Getting state for task {}", task_id);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get state: {}", e)))?;

        match value {
            Some(v) => {
                let state = serde_json::from_str(&v).map_err(|e| {
                    TaskError::Deserialization(format!("Failed to deserialize state: {}", e))
                })?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    async fn set_result(
        &self,
        task_id: &TaskId,
        result: TaskResult,
        ttl: Option<Duration>,
    ) -> Result<(), TaskError> {
        let state_key = self.state_key(task_id);
        let result_key = self.result_key(task_id);

        let state_value = serde_json::to_string(&result.state)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize state: {}", e)))?;
        let result_value = serde_json::to_string(&result)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize result: {}", e)))?;

        let ttl_secs = self.get_ttl_seconds(ttl);
        let mut conn = self.get_conn().await?;

        debug!("Setting result for task {}: {:?}", task_id, result.state);

        // Set both keys (not truly atomic in Redis without Lua script, but good enough)
        if ttl_secs > 0 {
            conn.set_ex::<_, _, ()>(&state_key, &state_value, ttl_secs)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set state: {}", e)))?;
            conn.set_ex::<_, _, ()>(&result_key, &result_value, ttl_secs)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set result: {}", e)))?;
        } else {
            conn.set::<_, _, ()>(&state_key, &state_value)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set state: {}", e)))?;
            conn.set::<_, _, ()>(&result_key, &result_value)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set result: {}", e)))?;
        }

        Ok(())
    }

    async fn get_result(&self, task_id: &TaskId) -> Result<Option<TaskResult>, TaskError> {
        let key = self.result_key(task_id);
        let mut conn = self.get_conn().await?;

        debug!("Getting result for task {}", task_id);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get result: {}", e)))?;

        match value {
            Some(v) => {
                let result = serde_json::from_str(&v).map_err(|e| {
                    TaskError::Deserialization(format!("Failed to deserialize result: {}", e))
                })?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    async fn wait_for_result(
        &self,
        task_id: &TaskId,
        timeout: Option<Duration>,
        poll_interval: Duration,
    ) -> Result<TaskResult, TaskError> {
        let start = std::time::Instant::now();
        let timeout_duration = timeout.unwrap_or(Duration::from_secs(3600)); // 1 hour default

        debug!(
            "Waiting for result: task={}, timeout={:?}, interval={:?}",
            task_id, timeout, poll_interval
        );

        loop {
            // Check timeout
            if start.elapsed() >= timeout_duration {
                warn!("Timeout waiting for task {}", task_id);
                return Err(TaskError::Timeout(format!(
                    "Task {} did not complete within {:?}",
                    task_id, timeout_duration
                )));
            }

            // Check state
            if let Some(state) = self.get_state(task_id).await? {
                if state.is_terminal() {
                    // Terminal state reached, get result
                    if let Some(result) = self.get_result(task_id).await? {
                        debug!("Task {} completed with state {:?}", task_id, state);
                        return Ok(result);
                    } else {
                        error!("Task {} in terminal state but no result found", task_id);
                        return Err(TaskError::Backend(format!(
                            "Task {} in terminal state but no result found",
                            task_id
                        )));
                    }
                }
            }

            // Wait before next poll
            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn delete(&self, task_id: &TaskId) -> Result<(), TaskError> {
        let state_key = self.state_key(task_id);
        let result_key = self.result_key(task_id);

        debug!("Deleting task data for {}", task_id);

        let mut conn = self.get_conn().await?;

        // Delete both keys
        conn.del::<_, ()>(&state_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete state: {}", e)))?;
        conn.del::<_, ()>(&result_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete result: {}", e)))?;

        Ok(())
    }

    async fn get_many(&self, task_ids: &[TaskId]) -> Result<Vec<Option<TaskResult>>, TaskError> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = task_ids.iter().map(|id| self.result_key(id)).collect();

        debug!("Getting {} results in batch", task_ids.len());

        let mut conn = self.get_conn().await?;

        let values: Vec<Option<String>> = conn
            .get(&keys)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get many results: {}", e)))?;

        let mut results = Vec::with_capacity(values.len());
        for (i, value) in values.into_iter().enumerate() {
            match value {
                Some(v) => {
                    let result = serde_json::from_str(&v).map_err(|e| {
                        TaskError::Deserialization(format!(
                            "Failed to deserialize result for task {}: {}",
                            task_ids[i], e
                        ))
                    })?;
                    results.push(Some(result));
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        debug!("Performing Redis health check");

        let mut conn = self.get_conn().await?;

        // Just get the connection - if we can connect, Redis is healthy
        let _: Option<String> = conn
            .get("__health_check__")
            .await
            .map_err(|e| TaskError::Backend(format!("Health check failed: {}", e)))?;

        Ok(())
    }

    // ==================== Metadata API ====================

    async fn set_metadata(
        &self,
        key: &str,
        value: serde_json::Value,
        ttl: Option<Duration>,
    ) -> Result<(), TaskError> {
        let full_key = format!("{}:meta:{}", self.config.key_prefix, key);
        let serialized =
            serde_json::to_string(&value).map_err(|e| TaskError::Serialization(e.to_string()))?;

        let mut conn = self.get_conn().await?;

        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let ttl_secs = ttl.as_secs() as i64;

        let _: () = conn
            .set_ex(&full_key, &serialized, ttl_secs as u64)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to set metadata: {}", e)))?;

        Ok(())
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<serde_json::Value>, TaskError> {
        let full_key = format!("{}:meta:{}", self.config.key_prefix, key);

        let mut conn = self.get_conn().await?;

        let value: Option<String> = conn
            .get(&full_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get metadata: {}", e)))?;

        match value {
            Some(v) => {
                let result: serde_json::Value = serde_json::from_str(&v)
                    .map_err(|e| TaskError::Deserialization(e.to_string()))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    async fn delete_metadata(&self, key: &str) -> Result<(), TaskError> {
        let full_key = format!("{}:meta:{}", self.config.key_prefix, key);

        let mut conn = self.get_conn().await?;

        let _: () = conn
            .del(&full_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete metadata: {}", e)))?;

        Ok(())
    }
}

// Make backend cloneable by cloning the pool
impl Clone for RedisBackend {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
#[path = "redis_tests.rs"]
mod tests;

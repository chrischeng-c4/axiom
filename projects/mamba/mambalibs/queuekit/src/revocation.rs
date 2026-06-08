//! Task revocation system
//!
//! Provides mechanisms to revoke tasks (prevent execution or terminate running tasks).
//! Supports both in-memory and distributed (Redis) revocation stores.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::{TaskError, TaskId};

/// Revoked task record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RevokedTask {
    /// Unique task identifier
    pub task_id: TaskId,
    /// Timestamp when the task was revoked
    pub revoked_at: DateTime<Utc>,
    /// Whether to terminate a running task (vs just preventing future execution)
    pub terminate: bool,
    /// Optional expiration time (auto-cleanup after this time)
    pub expires_at: Option<DateTime<Utc>>,
}

impl RevokedTask {
    /// Create a new revoked task record
    pub fn new(task_id: TaskId, terminate: bool, ttl: Option<Duration>) -> Self {
        let revoked_at = Utc::now();
        let expires_at = ttl.map(|d| revoked_at + chrono::Duration::from_std(d).unwrap());

        Self {
            task_id,
            revoked_at,
            terminate,
            expires_at,
        }
    }

    /// Check if this revocation has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// Trait for revocation storage backends
#[async_trait]
pub trait RevocationStore: Send + Sync + 'static {
    /// Check if a task is revoked
    async fn is_revoked(&self, task_id: &TaskId) -> Result<bool, TaskError>;

    /// Revoke a task by ID
    async fn revoke(&self, task_id: &TaskId, terminate: bool) -> Result<(), TaskError>;

    /// Revoke multiple tasks
    async fn revoke_many(&self, task_ids: &[TaskId], terminate: bool) -> Result<(), TaskError>;

    /// Get all revoked task IDs (for sync across workers)
    async fn get_revoked(&self) -> Result<Vec<TaskId>, TaskError>;

    /// Clear expired revocations (cleanup)
    /// Returns the number of revocations removed
    async fn cleanup(&self) -> Result<usize, TaskError>;
}

/// In-memory revocation store (thread-safe, non-distributed)
pub struct InMemoryRevocationStore {
    /// Map of task_id -> revoked task
    revoked: Arc<DashMap<TaskId, RevokedTask>>,
    /// Default TTL for revocations
    default_ttl: Option<Duration>,
}

impl InMemoryRevocationStore {
    /// Create a new in-memory revocation store
    pub fn new() -> Self {
        Self {
            revoked: Arc::new(DashMap::new()),
            default_ttl: None,
        }
    }

    /// Create with a default TTL for all revocations
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            revoked: Arc::new(DashMap::new()),
            default_ttl: Some(ttl),
        }
    }

    /// Get the number of revoked tasks (including expired)
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }
}

impl Default for InMemoryRevocationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RevocationStore for InMemoryRevocationStore {
    async fn is_revoked(&self, task_id: &TaskId) -> Result<bool, TaskError> {
        if let Some(entry) = self.revoked.get(task_id) {
            // Check if expired
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.revoked.remove(task_id);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn revoke(&self, task_id: &TaskId, terminate: bool) -> Result<(), TaskError> {
        let revoked_task = RevokedTask::new(task_id.clone(), terminate, self.default_ttl);
        self.revoked.insert(task_id.clone(), revoked_task);

        tracing::info!(
            task_id = %task_id,
            terminate = terminate,
            "Task revoked"
        );

        Ok(())
    }

    async fn revoke_many(&self, task_ids: &[TaskId], terminate: bool) -> Result<(), TaskError> {
        for task_id in task_ids {
            self.revoke(task_id, terminate).await?;
        }
        Ok(())
    }

    async fn get_revoked(&self) -> Result<Vec<TaskId>, TaskError> {
        let mut revoked_ids = Vec::new();

        for entry in self.revoked.iter() {
            if !entry.value().is_expired() {
                revoked_ids.push(entry.key().clone());
            }
        }

        Ok(revoked_ids)
    }

    async fn cleanup(&self) -> Result<usize, TaskError> {
        let mut removed = 0;

        // Collect expired task IDs
        let expired_ids: Vec<TaskId> = self
            .revoked
            .iter()
            .filter(|entry| entry.value().is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        // Remove expired entries
        for task_id in expired_ids {
            self.revoked.remove(&task_id);
            removed += 1;
        }

        if removed > 0 {
            tracing::debug!(removed = removed, "Cleaned up expired revocations");
        }

        Ok(removed)
    }
}

/// Redis-based revocation store (distributed across workers)
#[cfg(feature = "redis")]
pub struct RedisRevocationStore {
    pool: deadpool_redis::Pool,
    key_prefix: String,
    default_ttl: Option<Duration>,
}

#[cfg(feature = "redis")]
impl RedisRevocationStore {
    /// Create a new Redis revocation store
    pub fn new(pool: deadpool_redis::Pool) -> Self {
        Self {
            pool,
            key_prefix: "revoked_tasks".to_string(),
            default_ttl: None,
        }
    }

    /// Create with a custom key prefix
    pub fn with_prefix(pool: deadpool_redis::Pool, key_prefix: String) -> Self {
        Self {
            pool,
            key_prefix,
            default_ttl: None,
        }
    }

    /// Create with a default TTL
    pub fn with_ttl(pool: deadpool_redis::Pool, ttl: Duration) -> Self {
        Self {
            pool,
            key_prefix: "revoked_tasks".to_string(),
            default_ttl: Some(ttl),
        }
    }

    /// Get the Redis key for a task ID
    fn task_key(&self, task_id: &TaskId) -> String {
        format!("{}:{}", self.key_prefix, task_id)
    }

    /// Get the Redis set key for all revoked tasks
    fn set_key(&self) -> String {
        format!("{}:set", self.key_prefix)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl RevocationStore for RedisRevocationStore {
    async fn is_revoked(&self, task_id: &TaskId) -> Result<bool, TaskError> {
        use redis::AsyncCommands;

        let mut conn =
            self.pool.get().await.map_err(|e| {
                TaskError::Backend(format!("Failed to get Redis connection: {}", e))
            })?;

        let key = self.task_key(task_id);
        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| TaskError::Backend(format!("Redis EXISTS failed: {}", e)))?;

        Ok(exists)
    }

    async fn revoke(&self, task_id: &TaskId, terminate: bool) -> Result<(), TaskError> {
        use redis::AsyncCommands;

        let mut conn =
            self.pool.get().await.map_err(|e| {
                TaskError::Backend(format!("Failed to get Redis connection: {}", e))
            })?;

        let revoked_task = RevokedTask::new(task_id.clone(), terminate, self.default_ttl);
        let serialized = serde_json::to_string(&revoked_task).map_err(|e| {
            TaskError::Serialization(format!("Failed to serialize RevokedTask: {}", e))
        })?;

        let key = self.task_key(task_id);
        let set_key = self.set_key();

        // Store in hash and add to set
        let _: () = conn
            .set(&key, &serialized)
            .await
            .map_err(|e| TaskError::Backend(format!("Redis SET failed: {}", e)))?;

        let _: () = conn
            .sadd(&set_key, task_id.to_string())
            .await
            .map_err(|e| TaskError::Backend(format!("Redis SADD failed: {}", e)))?;

        // Set TTL if specified
        if let Some(ttl) = self.default_ttl {
            let ttl_secs = ttl.as_secs() as i64;
            let _: () = conn
                .expire(&key, ttl_secs)
                .await
                .map_err(|e| TaskError::Backend(format!("Redis EXPIRE failed: {}", e)))?;
        }

        tracing::info!(
            task_id = %task_id,
            terminate = terminate,
            "Task revoked (Redis)"
        );

        Ok(())
    }

    async fn revoke_many(&self, task_ids: &[TaskId], terminate: bool) -> Result<(), TaskError> {
        // Use pipelining for better performance
        for task_id in task_ids {
            self.revoke(task_id, terminate).await?;
        }
        Ok(())
    }

    async fn get_revoked(&self) -> Result<Vec<TaskId>, TaskError> {
        use redis::AsyncCommands;

        let mut conn =
            self.pool.get().await.map_err(|e| {
                TaskError::Backend(format!("Failed to get Redis connection: {}", e))
            })?;

        let set_key = self.set_key();
        let task_id_strings: Vec<String> = conn
            .smembers(&set_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Redis SMEMBERS failed: {}", e)))?;

        let mut task_ids = Vec::new();
        for id_str in task_id_strings {
            match TaskId::from_string(&id_str) {
                Ok(task_id) => task_ids.push(task_id),
                Err(e) => {
                    tracing::warn!(
                        task_id_str = %id_str,
                        error = %e,
                        "Invalid task ID in revocation set, skipping"
                    );
                }
            }
        }

        Ok(task_ids)
    }

    async fn cleanup(&self) -> Result<usize, TaskError> {
        use redis::AsyncCommands;

        let mut conn =
            self.pool.get().await.map_err(|e| {
                TaskError::Backend(format!("Failed to get Redis connection: {}", e))
            })?;

        let set_key = self.set_key();
        let task_id_strings: Vec<String> = conn
            .smembers(&set_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Redis SMEMBERS failed: {}", e)))?;

        let mut removed = 0;

        for id_str in task_id_strings {
            let task_id = match TaskId::from_string(&id_str) {
                Ok(id) => id,
                Err(_) => continue,
            };

            let key = self.task_key(&task_id);
            let exists: bool = conn
                .exists(&key)
                .await
                .map_err(|e| TaskError::Backend(format!("Redis EXISTS failed: {}", e)))?;

            // If key doesn't exist (expired via TTL), remove from set
            if !exists {
                let _: () = conn
                    .srem(&set_key, &id_str)
                    .await
                    .map_err(|e| TaskError::Backend(format!("Redis SREM failed: {}", e)))?;
                removed += 1;
            }
        }

        if removed > 0 {
            tracing::debug!(removed = removed, "Cleaned up expired revocations (Redis)");
        }

        Ok(removed)
    }
}

/// Revocation request builder
#[derive(Debug, Clone)]
pub struct RevokeRequest {
    /// Task IDs to revoke
    pub task_ids: Vec<TaskId>,
    /// Whether to terminate running tasks
    pub terminate: bool,
    /// Optional signal to send (e.g., "SIGTERM", "SIGKILL")
    pub signal: Option<String>,
}

impl RevokeRequest {
    /// Create a new revoke request
    pub fn new() -> Self {
        Self {
            task_ids: Vec::new(),
            terminate: false,
            signal: None,
        }
    }

    /// Add a task ID to revoke
    pub fn task_id(mut self, task_id: TaskId) -> Self {
        self.task_ids.push(task_id);
        self
    }

    /// Add multiple task IDs
    pub fn task_ids(mut self, task_ids: Vec<TaskId>) -> Self {
        self.task_ids.extend(task_ids);
        self
    }

    /// Set whether to terminate running tasks
    pub fn terminate(mut self, terminate: bool) -> Self {
        self.terminate = terminate;
        self
    }

    /// Set the signal to send when terminating
    pub fn signal(mut self, signal: String) -> Self {
        self.signal = Some(signal);
        self
    }

    /// Execute the revocation request
    pub async fn execute<S: RevocationStore>(self, store: &S) -> Result<(), TaskError> {
        if self.task_ids.is_empty() {
            return Ok(());
        }

        store.revoke_many(&self.task_ids, self.terminate).await?;

        if self.terminate {
            tracing::info!(
                count = self.task_ids.len(),
                signal = ?self.signal,
                "Revoked tasks with termination"
            );
        } else {
            tracing::info!(
                count = self.task_ids.len(),
                "Revoked tasks (no termination)"
            );
        }

        Ok(())
    }
}

impl Default for RevokeRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to revoke a single task
pub fn revoke(task_id: TaskId) -> RevokeRequest {
    RevokeRequest::new().task_id(task_id)
}

/// Helper function to revoke a task by name (requires task registry lookup)
pub fn revoke_by_name(task_name: &str) -> RevokeByNameRequest {
    RevokeByNameRequest {
        task_name: task_name.to_string(),
        terminate: false,
        signal: None,
    }
}

/// Request to revoke tasks by name
#[derive(Debug, Clone)]
pub struct RevokeByNameRequest {
    /// Task name to search for
    pub task_name: String,
    /// Whether to terminate running tasks
    pub terminate: bool,
    /// Optional signal to send
    pub signal: Option<String>,
}

impl RevokeByNameRequest {
    /// Set whether to terminate running tasks
    pub fn terminate(mut self, terminate: bool) -> Self {
        self.terminate = terminate;
        self
    }

    /// Set the signal to send when terminating
    pub fn signal(mut self, signal: String) -> Self {
        self.signal = Some(signal);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_revoke_single() {
        let store = InMemoryRevocationStore::new();
        let task_id = TaskId::new();

        // Initially not revoked
        assert!(!store.is_revoked(&task_id).await.unwrap());

        // Revoke the task
        store.revoke(&task_id, false).await.unwrap();

        // Now it should be revoked
        assert!(store.is_revoked(&task_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_many() {
        let store = InMemoryRevocationStore::new();
        let task_ids: Vec<TaskId> = (0..5).map(|_| TaskId::new()).collect();

        // Revoke all tasks
        store.revoke_many(&task_ids, true).await.unwrap();

        // All should be revoked
        for task_id in &task_ids {
            assert!(store.is_revoked(task_id).await.unwrap());
        }

        // Get all revoked task IDs
        let revoked = store.get_revoked().await.unwrap();
        assert_eq!(revoked.len(), 5);
    }

    #[tokio::test]
    async fn test_is_revoked() {
        let store = InMemoryRevocationStore::new();
        let task_id = TaskId::new();

        // Not revoked initially
        assert!(!store.is_revoked(&task_id).await.unwrap());

        // Revoke it
        store.revoke(&task_id, false).await.unwrap();

        // Should be revoked
        assert!(store.is_revoked(&task_id).await.unwrap());

        // Different task ID should not be revoked
        let other_id = TaskId::new();
        assert!(!store.is_revoked(&other_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let ttl = Duration::from_millis(100);
        let store = InMemoryRevocationStore::with_ttl(ttl);
        let task_id = TaskId::new();

        // Revoke the task
        store.revoke(&task_id, false).await.unwrap();
        assert!(store.is_revoked(&task_id).await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup should remove the expired entry
        let removed = store.cleanup().await.unwrap();
        assert_eq!(removed, 1);

        // Task should no longer be revoked
        assert!(!store.is_revoked(&task_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_revocation_expires() {
        let ttl = Duration::from_millis(100);
        let store = InMemoryRevocationStore::with_ttl(ttl);
        let task_id = TaskId::new();

        // Revoke the task
        store.revoke(&task_id, false).await.unwrap();
        assert!(store.is_revoked(&task_id).await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // is_revoked should return false and auto-cleanup
        assert!(!store.is_revoked(&task_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_request_builder() {
        let store = InMemoryRevocationStore::new();
        let task_id1 = TaskId::new();
        let task_id2 = TaskId::new();

        let request = RevokeRequest::new()
            .task_id(task_id1.clone())
            .task_id(task_id2.clone())
            .terminate(true)
            .signal("SIGTERM".to_string());

        request.execute(&store).await.unwrap();

        assert!(store.is_revoked(&task_id1).await.unwrap());
        assert!(store.is_revoked(&task_id2).await.unwrap());
    }

    #[tokio::test]
    async fn test_helper_functions() {
        let store = InMemoryRevocationStore::new();
        let task_id = TaskId::new();

        let request = revoke(task_id.clone());
        request.execute(&store).await.unwrap();

        assert!(store.is_revoked(&task_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_by_name() {
        let request = revoke_by_name("my_task")
            .terminate(true)
            .signal("SIGTERM".to_string());

        assert_eq!(request.task_name, "my_task");
        assert!(request.terminate);
        assert_eq!(request.signal, Some("SIGTERM".to_string()));
    }

    #[tokio::test]
    async fn test_store_len_and_empty() {
        let store = InMemoryRevocationStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        let task_id = TaskId::new();
        store.revoke(&task_id, false).await.unwrap();

        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }

    // ── RevokedTask ──────────────────────────────────────────────────

    #[test]
    fn revoked_task_new_no_ttl() {
        let task_id = TaskId::new();
        let rt = RevokedTask::new(task_id.clone(), false, None);
        assert_eq!(rt.task_id, task_id);
        assert!(!rt.terminate);
        assert!(rt.expires_at.is_none());
    }

    #[test]
    fn revoked_task_new_with_ttl() {
        let task_id = TaskId::new();
        let ttl = Duration::from_secs(60);
        let rt = RevokedTask::new(task_id.clone(), false, Some(ttl));
        assert!(rt.expires_at.is_some());
        assert!(rt.expires_at.unwrap() > rt.revoked_at);
    }

    #[test]
    fn revoked_task_new_terminate_flag() {
        let rt = RevokedTask::new(TaskId::new(), true, None);
        assert!(rt.terminate);
    }

    #[test]
    fn revoked_task_is_expired_none() {
        let rt = RevokedTask::new(TaskId::new(), false, None);
        assert!(!rt.is_expired());
    }

    #[test]
    fn revoked_task_is_expired_future() {
        let rt = RevokedTask::new(TaskId::new(), false, Some(Duration::from_secs(3600)));
        assert!(!rt.is_expired());
    }

    #[tokio::test]
    async fn revoked_task_is_expired_past() {
        let rt = RevokedTask::new(TaskId::new(), false, Some(Duration::from_millis(10)));
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(rt.is_expired());
    }

    #[test]
    fn revoked_task_serde_roundtrip() {
        let rt = RevokedTask::new(TaskId::new(), true, Some(Duration::from_secs(60)));
        let json = serde_json::to_string(&rt).unwrap();
        let deserialized: RevokedTask = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.task_id, rt.task_id);
        assert_eq!(deserialized.terminate, rt.terminate);
        assert_eq!(deserialized.expires_at, rt.expires_at);
        assert_eq!(deserialized.revoked_at, rt.revoked_at);
    }

    #[test]
    fn revoked_task_debug_impl() {
        let rt = RevokedTask::new(TaskId::new(), false, None);
        let debug_str = format!("{:?}", rt);
        assert!(debug_str.contains("RevokedTask"));
    }

    #[test]
    fn revoked_task_clone() {
        let rt = RevokedTask::new(TaskId::new(), true, Some(Duration::from_secs(30)));
        let cloned = rt.clone();
        assert_eq!(cloned.task_id, rt.task_id);
        assert_eq!(cloned.terminate, rt.terminate);
        assert_eq!(cloned.expires_at, rt.expires_at);
        assert_eq!(cloned.revoked_at, rt.revoked_at);
    }

    // ── InMemoryRevocationStore ──────────────────────────────────────

    #[test]
    fn store_new_empty() {
        let store = InMemoryRevocationStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn store_default_equals_new() {
        let from_new = InMemoryRevocationStore::new();
        let from_default = InMemoryRevocationStore::default();
        assert_eq!(from_new.len(), from_default.len());
        assert!(from_new.is_empty());
        assert!(from_default.is_empty());
        // Both should have no default_ttl
        assert!(from_new.default_ttl.is_none());
        assert!(from_default.default_ttl.is_none());
    }

    #[test]
    fn store_with_ttl_empty() {
        let store = InMemoryRevocationStore::with_ttl(Duration::from_secs(60));
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.default_ttl.is_some());
    }

    #[tokio::test]
    async fn is_revoked_auto_removes_expired_entry() {
        let store = InMemoryRevocationStore::with_ttl(Duration::from_millis(10));
        let task_id = TaskId::new();
        store.revoke(&task_id, false).await.unwrap();
        assert_eq!(store.len(), 1);

        tokio::time::sleep(Duration::from_millis(50)).await;

        // is_revoked should return false AND auto-remove the entry
        assert!(!store.is_revoked(&task_id).await.unwrap());
        assert_eq!(store.len(), 0);
    }

    #[tokio::test]
    async fn get_revoked_filters_expired() {
        let store = InMemoryRevocationStore::new();

        // Revoke task1 with a tiny TTL (manually insert)
        let task1 = TaskId::new();
        let rt1 = RevokedTask::new(task1.clone(), false, Some(Duration::from_millis(10)));
        store.revoked.insert(task1.clone(), rt1);

        // Revoke task2 with no TTL
        let task2 = TaskId::new();
        let rt2 = RevokedTask::new(task2.clone(), false, None);
        store.revoked.insert(task2.clone(), rt2);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let revoked = store.get_revoked().await.unwrap();
        assert_eq!(revoked.len(), 1);
        assert_eq!(revoked[0], task2);
    }

    #[tokio::test]
    async fn cleanup_no_expired_returns_zero() {
        let store = InMemoryRevocationStore::new();
        let task_id = TaskId::new();
        store.revoke(&task_id, false).await.unwrap();

        let removed = store.cleanup().await.unwrap();
        assert_eq!(removed, 0);
    }

    #[tokio::test]
    async fn cleanup_mixed_expired_and_live() {
        let store = InMemoryRevocationStore::new();

        // One expired entry (tiny TTL)
        let expired_id = TaskId::new();
        let rt_expired =
            RevokedTask::new(expired_id.clone(), false, Some(Duration::from_millis(10)));
        store.revoked.insert(expired_id.clone(), rt_expired);

        // One live entry (no TTL)
        let live_id = TaskId::new();
        let rt_live = RevokedTask::new(live_id.clone(), false, None);
        store.revoked.insert(live_id.clone(), rt_live);

        tokio::time::sleep(Duration::from_millis(50)).await;

        let removed = store.cleanup().await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.len(), 1);
        assert!(store.is_revoked(&live_id).await.unwrap());
        assert!(!store.is_revoked(&expired_id).await.unwrap());
    }

    #[tokio::test]
    async fn revoke_overwrites_existing() {
        let store = InMemoryRevocationStore::new();
        let task_id = TaskId::new();

        store.revoke(&task_id, false).await.unwrap();
        store.revoke(&task_id, true).await.unwrap();

        assert_eq!(store.len(), 1);
        // The second revoke should overwrite; task should still be revoked
        assert!(store.is_revoked(&task_id).await.unwrap());
    }

    #[tokio::test]
    async fn revoke_many_empty_slice() {
        let store = InMemoryRevocationStore::new();
        store.revoke_many(&[], false).await.unwrap();
        assert_eq!(store.len(), 0);
    }

    // ── RevokeRequest Builder ────────────────────────────────────────

    #[test]
    fn revoke_request_default() {
        let req = RevokeRequest::default();
        assert!(req.task_ids.is_empty());
        assert!(!req.terminate);
        assert!(req.signal.is_none());
    }

    #[test]
    fn revoke_request_new_equals_default() {
        let from_new = RevokeRequest::new();
        let from_default = RevokeRequest::default();
        assert_eq!(from_new.task_ids.len(), from_default.task_ids.len());
        assert_eq!(from_new.terminate, from_default.terminate);
        assert_eq!(from_new.signal, from_default.signal);
    }

    #[test]
    fn revoke_request_task_ids_batch() {
        let ids: Vec<TaskId> = (0..3).map(|_| TaskId::new()).collect();
        let req = RevokeRequest::new().task_ids(ids.clone());
        assert_eq!(req.task_ids.len(), 3);
        for (a, b) in req.task_ids.iter().zip(ids.iter()) {
            assert_eq!(a, b);
        }
    }

    #[tokio::test]
    async fn revoke_request_execute_empty() {
        let store = InMemoryRevocationStore::new();
        let req = RevokeRequest::new();
        req.execute(&store).await.unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn revoke_request_terminate_flag() {
        let req = RevokeRequest::new().terminate(true);
        assert!(req.terminate);
    }

    #[test]
    fn revoke_request_signal_builder() {
        let req = RevokeRequest::new().signal("SIGKILL".to_string());
        assert_eq!(req.signal, Some("SIGKILL".to_string()));
    }

    #[test]
    fn revoke_request_chained_builder() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        let req = RevokeRequest::new()
            .task_id(id1.clone())
            .task_id(id2.clone())
            .terminate(true)
            .signal("SIGTERM".to_string());

        assert_eq!(req.task_ids.len(), 2);
        assert_eq!(req.task_ids[0], id1);
        assert_eq!(req.task_ids[1], id2);
        assert!(req.terminate);
        assert_eq!(req.signal, Some("SIGTERM".to_string()));
    }

    // ── RevokeByNameRequest ──────────────────────────────────────────

    #[test]
    fn revoke_by_name_defaults() {
        let req = revoke_by_name("my_task");
        assert_eq!(req.task_name, "my_task");
        assert!(!req.terminate);
        assert!(req.signal.is_none());
    }

    #[test]
    fn revoke_by_name_terminate() {
        let req = revoke_by_name("t").terminate(true);
        assert!(req.terminate);
    }

    #[test]
    fn revoke_by_name_signal() {
        let req = revoke_by_name("t").signal("SIGKILL".to_string());
        assert_eq!(req.signal, Some("SIGKILL".to_string()));
    }

    #[test]
    fn revoke_by_name_debug_clone() {
        let req = revoke_by_name("task_a")
            .terminate(true)
            .signal("SIGTERM".to_string());
        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("RevokeByNameRequest"));
        let cloned = req.clone();
        assert_eq!(cloned.task_name, req.task_name);
        assert_eq!(cloned.terminate, req.terminate);
        assert_eq!(cloned.signal, req.signal);
    }

    // ── Helper Functions ─────────────────────────────────────────────

    #[test]
    fn revoke_helper_creates_single_id() {
        let id = TaskId::new();
        let req = revoke(id.clone());
        assert_eq!(req.task_ids.len(), 1);
        assert_eq!(req.task_ids[0], id);
    }

    // ── Store Utility ────────────────────────────────────────────────

    #[tokio::test]
    async fn store_len_after_multiple_revokes() {
        let store = InMemoryRevocationStore::new();
        let ids: Vec<TaskId> = (0..3).map(|_| TaskId::new()).collect();
        for id in &ids {
            store.revoke(id, false).await.unwrap();
        }
        assert_eq!(store.len(), 3);
    }

    // ── Trait + Thread Safety ────────────────────────────────────────

    #[test]
    fn in_memory_store_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<InMemoryRevocationStore>();
    }

    #[test]
    fn revoked_task_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RevokedTask>();
    }

    #[tokio::test]
    async fn concurrent_revoke_and_check() {
        let store = Arc::new(InMemoryRevocationStore::new());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let store = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let id = TaskId::new();
                store.revoke(&id, false).await.unwrap();
                id
            }));
        }

        let mut ids = Vec::new();
        for handle in handles {
            ids.push(handle.await.unwrap());
        }

        assert_eq!(store.len(), 10);
        for id in &ids {
            assert!(store.is_revoked(id).await.unwrap());
        }
    }

    #[tokio::test]
    async fn concurrent_revoke_same_id() {
        let store = Arc::new(InMemoryRevocationStore::new());
        let shared_id = TaskId::new();
        let mut handles = Vec::new();

        for _ in 0..5 {
            let store = Arc::clone(&store);
            let id = shared_id.clone();
            handles.push(tokio::spawn(async move {
                store.revoke(&id, false).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(store.len(), 1);
        assert!(store.is_revoked(&shared_id).await.unwrap());
    }
}

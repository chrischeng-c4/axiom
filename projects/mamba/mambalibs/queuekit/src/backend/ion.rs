//! Ion result backend implementation
//!
//! Uses cclab-ion as a high-performance, Rust-native result backend.

use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::Mutex;

use keep::client::KvClient;
use keep::KvValue;
use crate::{TaskError, TaskId, TaskResult, TaskState};
use super::ResultBackend;

/// Configuration for Ion backend
#[derive(Debug, Clone)]
pub struct IonBackendConfig {
    /// Connection URL (e.g., "127.0.0.1:16380" or "127.0.0.1:16380/namespace")
    pub url: String,
    /// Key prefix for task data
    pub key_prefix: String,
    /// Default TTL for results
    pub default_ttl: Option<Duration>,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for IonBackendConfig {
    fn default() -> Self {
        Self {
            url: "127.0.0.1:16380".to_string(),
            key_prefix: "meteor".to_string(),
            default_ttl: Some(Duration::from_secs(86400)), // 24 hours
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Ion-based result backend
///
/// High-performance result storage using cclab-ion KV store.
pub struct IonBackend {
    config: IonBackendConfig,
    client: Mutex<KvClient>,
}

impl IonBackend {
    /// Create a new Ion backend
    pub async fn new(config: IonBackendConfig) -> Result<Self, TaskError> {
        let client = KvClient::connect(&config.url)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to connect to Ion: {}", e)))?;

        Ok(Self {
            config,
            client: Mutex::new(client),
        })
    }

    /// Get the state key for a task
    fn state_key(&self, task_id: &TaskId) -> String {
        format!("{}:state:{}", self.config.key_prefix, task_id)
    }

    /// Get the result key for a task
    fn result_key(&self, task_id: &TaskId) -> String {
        format!("{}:result:{}", self.config.key_prefix, task_id)
    }
}

#[async_trait]
impl ResultBackend for IonBackend {
    async fn set_state(&self, task_id: &TaskId, state: TaskState) -> Result<(), TaskError> {
        let key = self.state_key(task_id);
        let value = serde_json::to_vec(&state)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;

        let mut client = self.client.lock().await;
        client
            .set(&key, KvValue::Bytes(value), self.config.default_ttl)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to set state in Ion: {}", e)))?;

        Ok(())
    }

    async fn get_state(&self, task_id: &TaskId) -> Result<Option<TaskState>, TaskError> {
        let key = self.state_key(task_id);

        let mut client = self.client.lock().await;
        match client.get(&key).await {
            Ok(Some(value)) => {
                let bytes = value_to_bytes(&value)?;
                let state: TaskState = serde_json::from_slice(&bytes)
                    .map_err(|e| TaskError::Deserialization(e.to_string()))?;
                Ok(Some(state))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(TaskError::Backend(format!("Failed to get state from Ion: {}", e))),
        }
    }

    async fn set_result(
        &self,
        task_id: &TaskId,
        result: TaskResult,
        ttl: Option<Duration>,
    ) -> Result<(), TaskError> {
        let key = self.result_key(task_id);
        let value = serde_json::to_vec(&result)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;

        let ttl = ttl.or(self.config.default_ttl);

        {
            let mut client = self.client.lock().await;
            client
                .set(&key, KvValue::Bytes(value), ttl)
                .await
                .map_err(|e| TaskError::Backend(format!("Failed to set result in Ion: {}", e)))?;
        }

        // Also update state
        self.set_state(task_id, result.state.clone()).await?;

        Ok(())
    }

    async fn get_result(&self, task_id: &TaskId) -> Result<Option<TaskResult>, TaskError> {
        let key = self.result_key(task_id);

        let mut client = self.client.lock().await;
        match client.get(&key).await {
            Ok(Some(value)) => {
                let bytes = value_to_bytes(&value)?;
                let result: TaskResult = serde_json::from_slice(&bytes)
                    .map_err(|e| TaskError::Deserialization(e.to_string()))?;
                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(TaskError::Backend(format!("Failed to get result from Ion: {}", e))),
        }
    }

    async fn wait_for_result(
        &self,
        task_id: &TaskId,
        timeout: Option<Duration>,
        poll_interval: Duration,
    ) -> Result<TaskResult, TaskError> {
        let start = std::time::Instant::now();

        loop {
            // Check for result
            if let Some(result) = self.get_result(task_id).await? {
                if result.state.is_terminal() {
                    return Ok(result);
                }
            }

            // Check timeout
            if let Some(timeout) = timeout {
                if start.elapsed() >= timeout {
                    return Err(TaskError::Timeout(format!(
                        "Timeout waiting for task {}",
                        task_id
                    )));
                }
            }

            // Wait before next poll
            tokio::time::sleep(poll_interval).await;
        }
    }

    async fn delete(&self, task_id: &TaskId) -> Result<(), TaskError> {
        let state_key = self.state_key(task_id);
        let result_key = self.result_key(task_id);

        let mut client = self.client.lock().await;
        client
            .delete(&state_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete state from Ion: {}", e)))?;

        client
            .delete(&result_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete result from Ion: {}", e)))?;

        Ok(())
    }

    async fn get_many(&self, task_ids: &[TaskId]) -> Result<Vec<Option<TaskResult>>, TaskError> {
        // Use batch mget for efficiency
        let keys: Vec<String> = task_ids.iter().map(|id| self.result_key(id)).collect();
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();

        let mut client = self.client.lock().await;
        let values = client
            .mget(&key_refs)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to get results from Ion: {}", e)))?;

        let mut results = Vec::with_capacity(task_ids.len());
        for value in values {
            match value {
                Some(v) => {
                    let bytes = value_to_bytes(&v)?;
                    let result: TaskResult = serde_json::from_slice(&bytes)
                        .map_err(|e| TaskError::Deserialization(e.to_string()))?;
                    results.push(Some(result));
                }
                None => results.push(None),
            }
        }

        Ok(results)
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        let mut client = self.client.lock().await;
        client
            .ping()
            .await
            .map_err(|e| TaskError::Backend(format!("Ion health check failed: {}", e)))?;
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
        let bytes = serde_json::to_vec(&value)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;

        let ttl = ttl.or(self.config.default_ttl);
        let mut client = self.client.lock().await;
        client
            .set(&full_key, KvValue::Bytes(bytes), ttl)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to set metadata in Ion: {}", e)))?;

        Ok(())
    }

    async fn get_metadata(&self, key: &str) -> Result<Option<serde_json::Value>, TaskError> {
        let full_key = format!("{}:meta:{}", self.config.key_prefix, key);

        let mut client = self.client.lock().await;
        match client.get(&full_key).await {
            Ok(Some(value)) => {
                let bytes = value_to_bytes(&value)?;
                let result: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| TaskError::Deserialization(e.to_string()))?;
                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(TaskError::Backend(format!(
                "Failed to get metadata from Ion: {}",
                e
            ))),
        }
    }

    async fn delete_metadata(&self, key: &str) -> Result<(), TaskError> {
        let full_key = format!("{}:meta:{}", self.config.key_prefix, key);

        let mut client = self.client.lock().await;
        client
            .delete(&full_key)
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete metadata from Ion: {}", e)))?;

        Ok(())
    }
}

/// Helper to extract bytes from KvValue
fn value_to_bytes(value: &KvValue) -> Result<Vec<u8>, TaskError> {
    match value {
        KvValue::Bytes(b) => Ok(b.clone()),
        KvValue::String(s) => Ok(s.as_bytes().to_vec()),
        _ => Err(TaskError::Deserialization(
            "Unexpected value type from Ion".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== I1: config_default_values ====================

    #[test]
    fn config_default_values() {
        let config = IonBackendConfig::default();
        assert_eq!(config.url, "127.0.0.1:16380");
        assert_eq!(config.key_prefix, "meteor");
        assert_eq!(config.default_ttl, Some(Duration::from_secs(86400)));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    // ==================== I2: key_format_state ====================

    #[test]
    fn key_format_state() {
        let config = IonBackendConfig::default();
        let task_id = TaskId::new();
        let state_key = format!("{}:state:{}", config.key_prefix, task_id);
        assert!(
            state_key.starts_with("meteor:state:"),
            "state key should start with 'meteor:state:', got: {}",
            state_key
        );
    }

    // ==================== I3: key_format_result ====================

    #[test]
    fn key_format_result() {
        let config = IonBackendConfig::default();
        let task_id = TaskId::new();
        let result_key = format!("{}:result:{}", config.key_prefix, task_id);
        assert!(
            result_key.starts_with("meteor:result:"),
            "result key should start with 'meteor:result:', got: {}",
            result_key
        );
    }

    // ==================== config_debug ====================

    #[test]
    fn config_debug() {
        let config = IonBackendConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("url"), "debug output should contain 'url'");
        assert!(debug_str.contains("key_prefix"), "debug output should contain 'key_prefix'");
        assert!(debug_str.contains("default_ttl"), "debug output should contain 'default_ttl'");
        assert!(
            debug_str.contains("connect_timeout"),
            "debug output should contain 'connect_timeout'"
        );
        assert!(
            debug_str.contains("request_timeout"),
            "debug output should contain 'request_timeout'"
        );
    }

    // ==================== config_clone ====================

    #[test]
    fn config_clone() {
        let config = IonBackendConfig {
            url: "10.0.0.1:9999".to_string(),
            key_prefix: "custom".to_string(),
            default_ttl: Some(Duration::from_secs(3600)),
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(60),
        };
        let cloned = config.clone();
        assert_eq!(cloned.url, config.url);
        assert_eq!(cloned.key_prefix, config.key_prefix);
        assert_eq!(cloned.default_ttl, config.default_ttl);
        assert_eq!(cloned.connect_timeout, config.connect_timeout);
        assert_eq!(cloned.request_timeout, config.request_timeout);
    }

    // ==================== I4: value_to_bytes_from_bytes ====================

    #[test]
    fn value_to_bytes_from_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0xFF];
        let kv = KvValue::Bytes(data.clone());
        let result = value_to_bytes(&kv).expect("should convert Bytes successfully");
        assert_eq!(result, data);
    }

    // ==================== I5: value_to_bytes_from_string ====================

    #[test]
    fn value_to_bytes_from_string() {
        let text = "hello world".to_string();
        let kv = KvValue::String(text.clone());
        let result = value_to_bytes(&kv).expect("should convert String successfully");
        assert_eq!(result, text.as_bytes().to_vec());
    }

    // ==================== I6: value_to_bytes_unexpected_type ====================

    #[test]
    fn value_to_bytes_unexpected_type() {
        let kv = KvValue::Int(42);
        let result = value_to_bytes(&kv);
        assert!(result.is_err(), "Int variant should produce an error");
        match result.unwrap_err() {
            TaskError::Deserialization(msg) => {
                assert!(
                    msg.contains("Unexpected value type"),
                    "error message should mention unexpected type, got: {}",
                    msg
                );
            }
            other => panic!("expected TaskError::Deserialization, got: {:?}", other),
        }
    }

    // ==================== Helper ====================

    async fn make_backend() -> IonBackend {
        IonBackend::new(IonBackendConfig::default()).await.unwrap()
    }

    // ==================== I7: set_get_state_round_trip ====================

    #[tokio::test]
    #[ignore]
    async fn set_get_state_round_trip() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        backend.set_state(&task_id, TaskState::Started).await.unwrap();
        let got = backend.get_state(&task_id).await.unwrap();
        assert_eq!(got, Some(TaskState::Started));
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I8: get_state_absent_returns_none ====================

    #[tokio::test]
    #[ignore]
    async fn get_state_absent_returns_none() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        assert_eq!(backend.get_state(&task_id).await.unwrap(), None);
    }

    // ==================== I9: set_state_overwrites ====================

    #[tokio::test]
    #[ignore]
    async fn set_state_overwrites() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        backend.set_state(&task_id, TaskState::Pending).await.unwrap();
        backend.set_state(&task_id, TaskState::Started).await.unwrap();
        assert_eq!(backend.get_state(&task_id).await.unwrap(), Some(TaskState::Started));
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I10: set_get_result_round_trip ====================

    #[tokio::test]
    #[ignore]
    async fn set_get_result_round_trip() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"key": "value"}));
        backend.set_result(&task_id, result.clone(), None).await.unwrap();
        let got = backend.get_result(&task_id).await.unwrap().unwrap();
        assert_eq!(got.task_id, task_id);
        assert_eq!(got.state, TaskState::Success);
        assert_eq!(got.result, Some(serde_json::json!({"key": "value"})));
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I11: get_result_absent_returns_none ====================

    #[tokio::test]
    #[ignore]
    async fn get_result_absent_returns_none() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        assert!(backend.get_result(&task_id).await.unwrap().is_none());
    }

    // ==================== I12: set_result_updates_state ====================

    #[tokio::test]
    #[ignore]
    async fn set_result_updates_state() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"done": true}));
        backend.set_result(&task_id, result, None).await.unwrap();
        // Ion's set_result calls self.set_state internally
        let state = backend.get_state(&task_id).await.unwrap();
        assert_eq!(state, Some(TaskState::Success));
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I13: set_result_custom_ttl ====================

    #[tokio::test]
    #[ignore]
    async fn set_result_custom_ttl() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"v": 1}));
        backend.set_result(&task_id, result, Some(Duration::from_secs(10))).await.unwrap();
        let got = backend.get_result(&task_id).await.unwrap();
        assert!(got.is_some());
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I14: set_result_nil_ttl_uses_default ====================

    #[tokio::test]
    #[ignore]
    async fn set_result_nil_ttl_uses_default() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"v": 2}));
        // ttl=None → uses config.default_ttl via `ttl.or(self.config.default_ttl)`
        backend.set_result(&task_id, result, None).await.unwrap();
        let got = backend.get_result(&task_id).await.unwrap();
        assert!(got.is_some());
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I15: wait_for_result_immediate_terminal ====================

    #[tokio::test]
    #[ignore]
    async fn wait_for_result_immediate_terminal() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"fast": true}));
        backend.set_result(&task_id, result, None).await.unwrap();
        let got = backend
            .wait_for_result(&task_id, Some(Duration::from_secs(5)), Duration::from_millis(50))
            .await
            .unwrap();
        assert_eq!(got.state, TaskState::Success);
        assert_eq!(got.task_id, task_id);
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I16: wait_for_result_timeout ====================

    #[tokio::test]
    #[ignore]
    async fn wait_for_result_timeout() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        // No result set → wait should timeout
        let err = backend
            .wait_for_result(&task_id, Some(Duration::from_millis(200)), Duration::from_millis(50))
            .await
            .unwrap_err();
        assert!(matches!(err, TaskError::Timeout(_)));
    }

    // ==================== I17: wait_for_result_polls_until_done ====================

    #[tokio::test]
    #[ignore]
    async fn wait_for_result_polls_until_done() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let task_id_clone = task_id.clone();
        // Spawn a separate backend to write the result after delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let writer = make_backend().await;
            let result = TaskResult::success(task_id_clone.clone(), serde_json::json!({"delayed": true}));
            writer.set_result(&task_id_clone, result, None).await.unwrap();
        });
        let got = backend
            .wait_for_result(&task_id, Some(Duration::from_secs(5)), Duration::from_millis(50))
            .await
            .unwrap();
        assert_eq!(got.state, TaskState::Success);
        assert_eq!(got.task_id, task_id);
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I18: wait_for_result_state_only_insufficient ====================

    #[tokio::test]
    #[ignore]
    async fn wait_for_result_state_only_insufficient() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        // Only set state (no result key) → Ion polls get_result, never finds it
        backend.set_state(&task_id, TaskState::Success).await.unwrap();
        let err = backend
            .wait_for_result(&task_id, Some(Duration::from_millis(200)), Duration::from_millis(50))
            .await
            .unwrap_err();
        assert!(matches!(err, TaskError::Timeout(_)));
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I19: delete_removes_both_keys ====================

    #[tokio::test]
    #[ignore]
    async fn delete_removes_both_keys() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        let result = TaskResult::success(task_id.clone(), serde_json::json!({"del": true}));
        backend.set_result(&task_id, result, None).await.unwrap();
        backend.delete(&task_id).await.unwrap();
        assert_eq!(backend.get_state(&task_id).await.unwrap(), None);
        assert!(backend.get_result(&task_id).await.unwrap().is_none());
    }

    // ==================== I20: delete_idempotent ====================

    #[tokio::test]
    #[ignore]
    async fn delete_idempotent() {
        let backend = make_backend().await;
        let task_id = TaskId::new();
        backend.delete(&task_id).await.unwrap();
    }

    // ==================== I21: get_many_mixed ====================

    #[tokio::test]
    #[ignore]
    async fn get_many_mixed() {
        let backend = make_backend().await;
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        let id3 = TaskId::new();
        let r1 = TaskResult::success(id1.clone(), serde_json::json!({"id": 1}));
        let r2 = TaskResult::success(id2.clone(), serde_json::json!({"id": 2}));
        backend.set_result(&id1, r1, None).await.unwrap();
        backend.set_result(&id2, r2, None).await.unwrap();
        let results = backend.get_many(&[id1.clone(), id2.clone(), id3]).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].is_some());
        assert!(results[1].is_some());
        assert!(results[2].is_none());
        assert_eq!(results[0].as_ref().unwrap().task_id, id1);
        assert_eq!(results[1].as_ref().unwrap().task_id, id2);
        backend.delete(&id1).await.unwrap();
        backend.delete(&id2).await.unwrap();
    }

    // ==================== I22: get_many_empty_input ====================

    #[tokio::test]
    #[ignore]
    async fn get_many_empty_input() {
        let backend = make_backend().await;
        let results = backend.get_many(&[]).await.unwrap();
        assert!(results.is_empty());
    }

    // ==================== I23: health_check_ping ====================

    #[tokio::test]
    #[ignore]
    async fn health_check_ping() {
        let backend = make_backend().await;
        backend.health_check().await.unwrap();
    }

    // ==================== I24: set_get_metadata_round_trip ====================

    #[tokio::test]
    #[ignore]
    async fn set_get_metadata_round_trip() {
        let backend = make_backend().await;
        let key = format!("test-meta-{}", uuid::Uuid::now_v7());
        let value = serde_json::json!({"chain_id": "abc", "step": 3});
        backend.set_metadata(&key, value.clone(), None).await.unwrap();
        let got = backend.get_metadata(&key).await.unwrap();
        assert_eq!(got, Some(value));
        backend.delete_metadata(&key).await.unwrap();
    }

    // ==================== I25: get_metadata_absent_returns_none ====================

    #[tokio::test]
    #[ignore]
    async fn get_metadata_absent_returns_none() {
        let backend = make_backend().await;
        let got = backend.get_metadata("nonexistent-key-xyz").await.unwrap();
        assert_eq!(got, None);
    }

    // ==================== I26: set_metadata_custom_ttl ====================

    #[tokio::test]
    #[ignore]
    async fn set_metadata_custom_ttl() {
        let backend = make_backend().await;
        let key = format!("test-meta-ttl-{}", uuid::Uuid::now_v7());
        let value = serde_json::json!({"ttl_test": true});
        backend.set_metadata(&key, value.clone(), Some(Duration::from_secs(10))).await.unwrap();
        let got = backend.get_metadata(&key).await.unwrap();
        assert_eq!(got, Some(value));
        backend.delete_metadata(&key).await.unwrap();
    }

    // ==================== I27: delete_metadata_removes_key ====================

    #[tokio::test]
    #[ignore]
    async fn delete_metadata_removes_key() {
        let backend = make_backend().await;
        let key = format!("test-meta-del-{}", uuid::Uuid::now_v7());
        backend.set_metadata(&key, serde_json::json!("val"), None).await.unwrap();
        backend.delete_metadata(&key).await.unwrap();
        let got = backend.get_metadata(&key).await.unwrap();
        assert_eq!(got, None);
    }

    // ==================== I28: delete_metadata_idempotent ====================

    #[tokio::test]
    #[ignore]
    async fn delete_metadata_idempotent() {
        let backend = make_backend().await;
        backend.delete_metadata("never-existed-key").await.unwrap();
    }
}

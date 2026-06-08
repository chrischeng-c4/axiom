//! Tests for RedisBackend
//!
//! Unit tests (R1-R4) run without Redis.
//! Integration tests (R5-R34) require a running Redis instance and skip gracefully if unavailable.

use super::*;

// ==================== R1: config_defaults ====================

#[test]
fn test_config_defaults() {
    let config = RedisBackendConfig::default();
    assert_eq!(config.url, "redis://localhost:6379");
    assert_eq!(config.key_prefix, "cclab-meteor");
    assert_eq!(config.default_ttl, Duration::from_secs(86400));
    assert_eq!(config.pool_size, 10);
}

#[test]
fn test_key_generation() {
    let config = RedisBackendConfig::default();
    let backend_result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { RedisBackend::new(config.clone()).await });

    // Skip if Redis not available (integration test environment)
    if backend_result.is_err() {
        return;
    }

    let backend = backend_result.unwrap();
    let task_id = TaskId::new();

    let state_key = backend.state_key(&task_id);
    let result_key = backend.result_key(&task_id);

    assert_eq!(state_key, format!("cclab-meteor:state:{}", task_id));
    assert_eq!(result_key, format!("cclab-meteor:result:{}", task_id));
}

#[test]
fn test_serialization() {
    let state = TaskState::Success;
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: TaskState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, deserialized);

    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"foo": "bar"}));
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TaskResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task_id, task_id);
    assert_eq!(deserialized.state, TaskState::Success);
}

// ==================== R2: config_serde_round_trip ====================

#[test]
fn config_serde_round_trip() {
    let config = RedisBackendConfig {
        url: "redis://myhost:1234".to_string(),
        key_prefix: "test-prefix".to_string(),
        default_ttl: Duration::from_secs(300),
        pool_size: 5,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RedisBackendConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.url, "redis://myhost:1234");
    assert_eq!(deserialized.key_prefix, "test-prefix");
    assert_eq!(deserialized.default_ttl, Duration::from_secs(300));
    assert_eq!(deserialized.pool_size, 5);
}

// ==================== R3: state_serialization_round_trip ====================

#[test]
fn state_serialization_round_trip() {
    let state = TaskState::Success;
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: TaskState = serde_json::from_str(&json).unwrap();
    assert_eq!(state, deserialized);
}

// ==================== R4: result_serialization_round_trip ====================

#[test]
fn result_serialization_round_trip() {
    let task_id = TaskId::new();
    let result =
        TaskResult::success(task_id.clone(), serde_json::json!({"key": "value", "num": 42}));
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: TaskResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task_id, task_id);
    assert_eq!(deserialized.state, TaskState::Success);
    assert_eq!(
        deserialized.result,
        Some(serde_json::json!({"key": "value", "num": 42}))
    );
}

#[test]
fn config_debug() {
    let config = RedisBackendConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(
        debug_str.contains("RedisBackendConfig") || debug_str.contains("url"),
        "Debug output should contain type name or field names, got: {}",
        debug_str
    );
}

#[test]
fn config_clone() {
    let config = RedisBackendConfig {
        url: "redis://clonehost:9999".to_string(),
        key_prefix: "clone-prefix".to_string(),
        default_ttl: Duration::from_secs(600),
        pool_size: 20,
    };
    let cloned = config.clone();
    assert_eq!(cloned.url, config.url);
    assert_eq!(cloned.key_prefix, config.key_prefix);
    assert_eq!(cloned.default_ttl, config.default_ttl);
    assert_eq!(cloned.pool_size, config.pool_size);
}

// ==================== Helper ====================

async fn make_backend() -> Option<RedisBackend> {
    RedisBackend::new(RedisBackendConfig::default()).await.ok()
}

// ===========================================================================
// Integration tests — require Redis running
// ===========================================================================

// ==================== R5: key_generation_state_format ====================

#[tokio::test]
async fn key_generation_state_format() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    assert_eq!(
        backend.state_key(&task_id),
        format!("cclab-meteor:state:{}", task_id)
    );
}

// ==================== R6: key_generation_result_format ====================

#[tokio::test]
async fn key_generation_result_format() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    assert_eq!(
        backend.result_key(&task_id),
        format!("cclab-meteor:result:{}", task_id)
    );
}

// ==================== R7: get_ttl_seconds_with_override ====================

#[tokio::test]
async fn get_ttl_seconds_with_override() {
    let Some(backend) = make_backend().await else { return };
    assert_eq!(backend.get_ttl_seconds(Some(Duration::from_secs(60))), 60);
}

// ==================== R8: get_ttl_seconds_default_fallback ====================

#[tokio::test]
async fn get_ttl_seconds_default_fallback() {
    let Some(backend) = make_backend().await else { return };
    assert_eq!(
        backend.get_ttl_seconds(None),
        backend.config.default_ttl.as_secs()
    );
}

// ==================== R9: set_get_state_round_trip ====================

#[tokio::test]
async fn test_set_get_state() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();

    backend.set_state(&task_id, TaskState::Started).await.unwrap();
    let retrieved = backend.get_state(&task_id).await.unwrap();
    assert_eq!(retrieved, Some(TaskState::Started));

    backend.delete(&task_id).await.unwrap();
}

// ==================== R10: get_state_absent_returns_none ====================

#[tokio::test]
async fn get_state_absent_returns_none() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    assert_eq!(backend.get_state(&task_id).await.unwrap(), None);
}

// ==================== R11: set_state_overwrites ====================

#[tokio::test]
async fn set_state_overwrites() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    backend.set_state(&task_id, TaskState::Pending).await.unwrap();
    backend.set_state(&task_id, TaskState::Started).await.unwrap();
    assert_eq!(
        backend.get_state(&task_id).await.unwrap(),
        Some(TaskState::Started)
    );
    backend.delete(&task_id).await.unwrap();
}

// ==================== R12: set_get_result_round_trip ====================

#[tokio::test]
async fn test_set_get_result() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"test": "data"}));

    backend
        .set_result(&task_id, result.clone(), None)
        .await
        .unwrap();

    let retrieved = backend.get_result(&task_id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.task_id, task_id);
    assert_eq!(retrieved.state, TaskState::Success);

    backend.delete(&task_id).await.unwrap();
}

// ==================== R13: get_result_absent_returns_none ====================

#[tokio::test]
async fn get_result_absent_returns_none() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    assert!(backend.get_result(&task_id).await.unwrap().is_none());
}

// ==================== R14: set_result_custom_ttl ====================

#[tokio::test]
async fn set_result_custom_ttl() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"v": 1}));
    backend
        .set_result(&task_id, result, Some(Duration::from_secs(10)))
        .await
        .unwrap();
    let retrieved = backend.get_result(&task_id).await.unwrap();
    assert!(retrieved.is_some());
    backend.delete(&task_id).await.unwrap();
}

// ==================== R15: set_result_zero_ttl_no_expiry ====================

#[tokio::test]
async fn set_result_zero_ttl_no_expiry() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"v": 2}));
    // Duration::ZERO -> ttl_secs == 0 -> SET without EX (no expiry)
    backend
        .set_result(&task_id, result, Some(Duration::ZERO))
        .await
        .unwrap();
    let retrieved = backend.get_result(&task_id).await.unwrap();
    assert!(retrieved.is_some());
    backend.delete(&task_id).await.unwrap();
}

// ==================== R16: set_result_writes_state_key ====================

#[tokio::test]
async fn set_result_writes_state_key() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"done": true}));
    backend.set_result(&task_id, result, None).await.unwrap();
    // set_result should also write the state key
    let state = backend.get_state(&task_id).await.unwrap();
    assert_eq!(state, Some(TaskState::Success));
    backend.delete(&task_id).await.unwrap();
}

// ==================== R17: wait_for_result_immediate_terminal ====================

#[tokio::test]
async fn wait_for_result_immediate_terminal() {
    let Some(backend) = make_backend().await else { return };
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

// ==================== R18: wait_for_result_polls_until_done ====================

#[tokio::test]
async fn test_wait_for_result_success() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();

    backend
        .set_state(&task_id, TaskState::Pending)
        .await
        .unwrap();

    let task_id_clone = task_id.clone();
    let backend_clone = backend.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = TaskResult::success(
            task_id_clone.clone(),
            serde_json::json!({"result": "done"}),
        );
        backend_clone
            .set_result(&task_id_clone, result, None)
            .await
            .unwrap();
    });

    let result = backend
        .wait_for_result(
            &task_id,
            Some(Duration::from_secs(5)),
            Duration::from_millis(50),
        )
        .await
        .unwrap();

    assert_eq!(result.state, TaskState::Success);
    assert_eq!(result.task_id, task_id);
    backend.delete(&task_id).await.unwrap();
}

// ==================== R19: wait_for_result_timeout ====================

#[tokio::test]
async fn test_wait_for_result_timeout() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();

    backend
        .set_state(&task_id, TaskState::Pending)
        .await
        .unwrap();

    let result = backend
        .wait_for_result(
            &task_id,
            Some(Duration::from_millis(200)),
            Duration::from_millis(50),
        )
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TaskError::Timeout(_)));
    backend.delete(&task_id).await.unwrap();
}

// ==================== R20: wait_for_result_default_timeout ====================

#[tokio::test]
async fn wait_for_result_default_timeout() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    backend
        .set_state(&task_id, TaskState::Pending)
        .await
        .unwrap();
    // timeout=None -> 3600s internally; cancel via tokio::select after 200ms
    let result = tokio::select! {
        r = backend.wait_for_result(&task_id, None, Duration::from_millis(50)) => r,
        _ = tokio::time::sleep(Duration::from_millis(200)) => {
            // Cancelled -- no panic, test passes
            backend.delete(&task_id).await.unwrap();
            return;
        }
    };
    // If we somehow got a result, it should be an error (task never completes)
    assert!(result.is_err());
    backend.delete(&task_id).await.unwrap();
}

// ==================== R21: wait_for_result_terminal_no_result_key ====================

#[tokio::test]
async fn wait_for_result_terminal_no_result_key() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    // Set state to terminal WITHOUT setting result
    backend
        .set_state(&task_id, TaskState::Success)
        .await
        .unwrap();
    let err = backend
        .wait_for_result(&task_id, Some(Duration::from_secs(5)), Duration::from_millis(50))
        .await
        .unwrap_err();
    match err {
        TaskError::Backend(msg) => assert!(
            msg.contains("no result found"),
            "expected 'no result found' in: {msg}"
        ),
        other => panic!("expected TaskError::Backend, got: {other:?}"),
    }
    backend.delete(&task_id).await.unwrap();
}

// ==================== R22: delete_removes_both_keys ====================

#[tokio::test]
async fn test_delete() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"test": "data"}));

    backend
        .set_result(&task_id, result, None)
        .await
        .unwrap();

    assert!(backend.get_result(&task_id).await.unwrap().is_some());
    backend.delete(&task_id).await.unwrap();
    assert!(backend.get_result(&task_id).await.unwrap().is_none());
    assert_eq!(backend.get_state(&task_id).await.unwrap(), None);
}

// ==================== R23: delete_idempotent ====================

#[tokio::test]
async fn delete_idempotent() {
    let Some(backend) = make_backend().await else { return };
    let task_id = TaskId::new();
    backend.delete(&task_id).await.unwrap();
}

// ==================== R24: get_many_mixed ====================

#[tokio::test]
async fn test_get_many() {
    let Some(backend) = make_backend().await else { return };
    let task_id1 = TaskId::new();
    let task_id2 = TaskId::new();
    let task_id3 = TaskId::new();

    let result1 = TaskResult::success(task_id1.clone(), serde_json::json!({"id": 1}));
    let result2 = TaskResult::success(task_id2.clone(), serde_json::json!({"id": 2}));

    backend.set_result(&task_id1, result1, None).await.unwrap();
    backend.set_result(&task_id2, result2, None).await.unwrap();

    let results = backend
        .get_many(&[task_id1.clone(), task_id2.clone(), task_id3.clone()])
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    assert!(results[0].is_some());
    assert!(results[1].is_some());
    assert!(results[2].is_none());
    assert_eq!(results[0].as_ref().unwrap().task_id, task_id1);
    assert_eq!(results[1].as_ref().unwrap().task_id, task_id2);

    backend.delete(&task_id1).await.unwrap();
    backend.delete(&task_id2).await.unwrap();
}

// ==================== R25: get_many_empty_input ====================

#[tokio::test]
async fn get_many_empty_input() {
    let Some(backend) = make_backend().await else { return };
    let results = backend.get_many(&[]).await.unwrap();
    assert!(results.is_empty());
}

// ==================== R26: get_many_all_absent ====================

#[tokio::test]
async fn get_many_all_absent() {
    let Some(backend) = make_backend().await else { return };
    let ids = [TaskId::new(), TaskId::new(), TaskId::new()];
    let results = backend.get_many(&ids).await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_none()));
}

// ==================== R27: health_check_ok ====================

#[tokio::test]
async fn test_health_check() {
    let Some(backend) = make_backend().await else { return };
    backend.health_check().await.unwrap();
}

// ==================== R28: set_get_metadata_round_trip ====================

#[tokio::test]
async fn set_get_metadata_round_trip() {
    let Some(backend) = make_backend().await else { return };
    let key = format!("test-meta-{}", uuid::Uuid::now_v7());
    let value = serde_json::json!({"chain_id": "abc", "step": 3});
    backend.set_metadata(&key, value.clone(), None).await.unwrap();
    let got = backend.get_metadata(&key).await.unwrap();
    assert_eq!(got, Some(value));
    backend.delete_metadata(&key).await.unwrap();
}

// ==================== R29: get_metadata_absent_returns_none ====================

#[tokio::test]
async fn get_metadata_absent_returns_none() {
    let Some(backend) = make_backend().await else { return };
    let got = backend.get_metadata("nonexistent-key-xyz").await.unwrap();
    assert_eq!(got, None);
}

// ==================== R30: set_metadata_custom_ttl ====================

#[tokio::test]
async fn set_metadata_custom_ttl() {
    let Some(backend) = make_backend().await else { return };
    let key = format!("test-meta-ttl-{}", uuid::Uuid::now_v7());
    let value = serde_json::json!({"ttl_test": true});
    backend
        .set_metadata(&key, value.clone(), Some(Duration::from_secs(10)))
        .await
        .unwrap();
    let got = backend.get_metadata(&key).await.unwrap();
    assert_eq!(got, Some(value));
    backend.delete_metadata(&key).await.unwrap();
}

// ==================== R31: delete_metadata_removes_key ====================

#[tokio::test]
async fn delete_metadata_removes_key() {
    let Some(backend) = make_backend().await else { return };
    let key = format!("test-meta-del-{}", uuid::Uuid::now_v7());
    backend
        .set_metadata(&key, serde_json::json!("val"), None)
        .await
        .unwrap();
    backend.delete_metadata(&key).await.unwrap();
    let got = backend.get_metadata(&key).await.unwrap();
    assert_eq!(got, None);
}

// ==================== R32: delete_metadata_idempotent ====================

#[tokio::test]
async fn delete_metadata_idempotent() {
    let Some(backend) = make_backend().await else { return };
    backend.delete_metadata("never-existed-key").await.unwrap();
}

// ==================== R33: metadata_key_format ====================

#[tokio::test]
async fn metadata_key_format() {
    let Some(backend) = make_backend().await else { return };
    let key = "workflow-123";
    let value = serde_json::json!({"status": "running"});
    backend.set_metadata(key, value.clone(), None).await.unwrap();
    let got = backend.get_metadata(key).await.unwrap();
    assert_eq!(got, Some(value));
    backend.delete_metadata(key).await.unwrap();
}

// ==================== R34: clone_shares_pool ====================

#[tokio::test]
async fn clone_shares_pool() {
    let Some(backend1) = make_backend().await else { return };
    let backend2 = backend1.clone();
    let task_id = TaskId::new();
    let result = TaskResult::success(task_id.clone(), serde_json::json!({"shared": true}));
    // Write from clone
    backend2.set_result(&task_id, result, None).await.unwrap();
    // Read from original
    let got = backend1.get_result(&task_id).await.unwrap();
    assert!(got.is_some());
    assert_eq!(got.unwrap().task_id, task_id);
    backend1.delete(&task_id).await.unwrap();
}

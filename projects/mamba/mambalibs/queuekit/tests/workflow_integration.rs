//! Integration tests for workflow primitives
//!
//! These tests verify chain, group, and chord execution flows.
//! They are marked as #[ignore] by default since they require running infrastructure.

use queuekit::{
    Broker, Chain, Chord, Group, NatsBroker, NatsBrokerConfig, RedisBackend, RedisBackendConfig,
    TaskSignature, TaskOptions,
};

/// Helper to create a test broker
async fn create_test_broker() -> NatsBroker {
    let config = NatsBrokerConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };
    let broker = NatsBroker::new(config);
    broker.connect().await.expect("Failed to connect to NATS");
    broker
}

/// Helper to create a test backend
async fn create_test_backend() -> RedisBackend {
    let config = RedisBackendConfig {
        url: "redis://localhost:6379".to_string(),
        ..Default::default()
    };
    RedisBackend::new(config).await.expect("Failed to create Redis backend")
}

#[tokio::test]
#[ignore] // Requires NATS running
async fn test_chain_apply_async() {
    let broker = create_test_broker().await;

    let tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
        TaskSignature::new("task3", serde_json::json!([3])),
    ];

    let chain = Chain::new(tasks);
    let result = chain.apply_async(&broker).await;

    assert!(result.is_ok());
    let async_result = result.unwrap();
    assert_eq!(async_result.chain_id, chain.id);

    broker.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires NATS running
async fn test_chain_with_custom_queue() {
    let broker = create_test_broker().await;

    let tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1]))
            .set_queue("priority"),
        TaskSignature::new("task2", serde_json::json!([2])),
    ];

    let options = TaskOptions::new().with_queue("default_queue");
    let chain = Chain::new(tasks).with_options(options);

    let result = chain.apply_async(&broker).await;
    assert!(result.is_ok());

    broker.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_chain_empty_tasks_error() {
    let config = NatsBrokerConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };
    let broker = NatsBroker::new(config);

    let chain = Chain::new(vec![]);
    let result = chain.apply_async(&broker).await;

    assert!(result.is_err());
    match result {
        Err(queuekit::TaskError::InvalidWorkflow(msg)) => {
            assert!(msg.contains("at least one task"));
        }
        _ => panic!("Expected InvalidWorkflow error"),
    }
}

#[tokio::test]
#[ignore] // Requires NATS running
async fn test_group_apply_async() {
    let broker = create_test_broker().await;

    let tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
        TaskSignature::new("task3", serde_json::json!([3])),
    ];

    let group = Group::new(tasks);
    let result = group.apply_async(&broker).await;

    assert!(result.is_ok());
    let group_result = result.unwrap();
    assert_eq!(group_result.task_ids.len(), 3);

    broker.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires NATS running
async fn test_group_with_options() {
    let broker = create_test_broker().await;

    let tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
    ];

    let options = TaskOptions::new().with_queue("bulk");
    let group = Group::new(tasks).with_options(options);

    let result = group.apply_async(&broker).await;
    assert!(result.is_ok());

    broker.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_group_empty_tasks_error() {
    let config = NatsBrokerConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };
    let broker = NatsBroker::new(config);

    let group = Group::new(vec![]);
    let result = group.apply_async(&broker).await;

    assert!(result.is_err());
    match result {
        Err(queuekit::TaskError::InvalidWorkflow(msg)) => {
            assert!(msg.contains("at least one task"));
        }
        _ => panic!("Expected InvalidWorkflow error"),
    }
}

#[tokio::test]
#[ignore] // Requires NATS and Redis running
async fn test_chord_apply_async() {
    let broker = create_test_broker().await;
    let backend = create_test_backend().await;

    let header_tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
    ];

    let header = Group::new(header_tasks);
    let callback = TaskSignature::new("combine", serde_json::json!([]));
    let chord = Chord::new(header, callback);

    let result = chord.apply_async(&broker, &backend).await;
    assert!(result.is_ok());

    let chord_result = result.unwrap();
    assert_eq!(chord_result.chord_id, chord.id);
    assert_eq!(chord_result.header_result.task_ids.len(), 2);

    broker.disconnect().await.unwrap();
}

#[tokio::test]
#[ignore] // Requires Redis running
async fn test_chord_empty_header_error() {
    let config = NatsBrokerConfig {
        url: "nats://localhost:4222".to_string(),
        ..Default::default()
    };
    let broker = NatsBroker::new(config);

    let backend_config = RedisBackendConfig {
        url: "redis://localhost:6379".to_string(),
        ..Default::default()
    };
    let backend = RedisBackend::new(backend_config).await.expect("Failed to create backend");

    let header = Group::new(vec![]);
    let callback = TaskSignature::new("callback", serde_json::json!([]));
    let chord = Chord::new(header, callback);

    let result = chord.apply_async(&broker, &backend).await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires NATS running
async fn test_chord_trigger_callback() {
    let broker = create_test_broker().await;

    let header_tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
    ];

    let header = Group::new(header_tasks);
    let callback = TaskSignature::new("combine", serde_json::json!([]));
    let chord = Chord::new(header, callback);

    let header_results = vec![
        serde_json::json!({"result": 1}),
        serde_json::json!({"result": 2}),
    ];

    let result = chord.trigger_callback(&broker, header_results).await;
    assert!(result.is_ok());

    broker.disconnect().await.unwrap();
}

#[test]
fn test_task_signature_builder() {
    let sig = TaskSignature::new("my_task", serde_json::json!([1, 2, 3]))
        .with_kwargs(serde_json::json!({"key": "value"}))
        .set_queue("priority")
        .set_countdown(60);

    assert_eq!(sig.task_name, "my_task");
    assert_eq!(sig.args, serde_json::json!([1, 2, 3]));
    assert_eq!(sig.kwargs, serde_json::json!({"key": "value"}));
    assert_eq!(sig.options.queue, Some("priority".to_string()));
    assert_eq!(sig.options.countdown, Some(60));
}

#[test]
fn test_task_signature_immutable() {
    let sig = TaskSignature::new("task", serde_json::json!([2, 3]))
        .immutable();

    assert!(sig.immutable);

    // When immutable, args should not be modified with previous result
    let new_args = sig.args_with_result(serde_json::json!(1));
    assert_eq!(new_args, serde_json::json!([2, 3]));
}

#[test]
fn test_task_signature_mutable_chain() {
    let sig = TaskSignature::new("task", serde_json::json!([2, 3]));

    assert!(!sig.immutable);

    // When mutable, previous result should be prepended
    let new_args = sig.args_with_result(serde_json::json!(1));
    assert_eq!(new_args, serde_json::json!([1, 2, 3]));
}

#[test]
fn test_chain_metadata() {
    let tasks = vec![
        TaskSignature::new("task1", serde_json::json!([1])),
        TaskSignature::new("task2", serde_json::json!([2])),
    ];

    let chain = Chain::new(tasks);
    let metadata = chain.get_metadata();

    assert!(metadata.is_ok());
    let (key, data) = metadata.unwrap();
    assert!(key.starts_with("chain:"));
    assert!(!data.is_empty());

    // Verify it can be deserialized
    let parsed: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert!(parsed.is_object());
}

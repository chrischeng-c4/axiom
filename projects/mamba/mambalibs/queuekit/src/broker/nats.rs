//! NATS JetStream broker implementation

use async_nats::jetstream::{
    self,
    consumer::{pull::Config as ConsumerConfig, AckPolicy, DeliverPolicy},
    stream::{Config as StreamConfig, RetentionPolicy},
};
use async_trait::async_trait;
use chrono::Utc;
use futures::StreamExt;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{
    broker::{BrokerMessage, MessageHandler, SubscriptionHandle},
    Broker, BrokerCapabilities, DelayedBroker, DeliveryModel, PullBroker, TaskError, TaskMessage,
};

/// NATS JetStream broker configuration
#[derive(Debug, Clone)]
pub struct NatsBrokerConfig {
    /// NATS server URL (e.g., "nats://localhost:4222")
    pub url: String,
    /// Stream name for task messages
    pub stream_name: String,
    /// Consumer durable name prefix
    pub consumer_prefix: String,
    /// Maximum number of pending messages per consumer
    pub max_pending: usize,
    /// Ack wait timeout
    pub ack_wait: Duration,
    /// Maximum delivery attempts before dead-letter
    pub max_deliver: i64,
}

impl Default for NatsBrokerConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            stream_name: "TASKS".to_string(),
            consumer_prefix: "task-worker".to_string(),
            max_pending: 1000,
            ack_wait: Duration::from_secs(30),
            max_deliver: 5,
        }
    }
}

/// NATS JetStream broker implementation
pub struct NatsBroker {
    config: NatsBrokerConfig,
    client: RwLock<Option<async_nats::Client>>,
    jetstream: RwLock<Option<jetstream::Context>>,
    stream: RwLock<Option<jetstream::stream::Stream>>,
}

impl NatsBroker {
    /// Create a new NATS broker with the given configuration
    pub fn new(config: NatsBrokerConfig) -> Self {
        Self {
            config,
            client: RwLock::new(None),
            jetstream: RwLock::new(None),
            stream: RwLock::new(None),
        }
    }

    /// Subscribe to a queue with a message handler (internal implementation)
    async fn subscribe_impl<H: MessageHandler + 'static>(
        &self,
        queue: &str,
        handler: Arc<H>,
    ) -> Result<SubscriptionHandle, TaskError> {
        let js = self
            .jetstream
            .read()
            .await
            .as_ref()
            .ok_or(TaskError::NotConnected)?
            .clone();

        // Create or get durable pull consumer
        let consumer_name = format!("{}-{}", self.config.consumer_prefix, queue);
        let subject_filter = format!("tasks.{}", queue);

        tracing::debug!(
            "Creating consumer '{}' for subject '{}'",
            consumer_name,
            subject_filter
        );

        let consumer_config = ConsumerConfig {
            durable_name: Some(consumer_name.clone()),
            ack_policy: AckPolicy::Explicit,
            deliver_policy: DeliverPolicy::All,
            filter_subject: subject_filter.clone(),
            ack_wait: self.config.ack_wait,
            max_deliver: self.config.max_deliver,
            ..Default::default()
        };

        let stream = js
            .get_stream(&self.config.stream_name)
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to get stream: {}", e)))?;

        let consumer = stream
            .get_or_create_consumer(&consumer_name, consumer_config)
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to create consumer: {}", e)))?;

        // Create cancellation token
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();

        // Spawn message processing loop
        let queue_owned = queue.to_string();
        tokio::spawn(async move {
            tracing::info!("Starting message loop for queue: {}", queue_owned);

            loop {
                tokio::select! {
                    _ = cancel_token_clone.cancelled() => {
                        tracing::info!("Message loop cancelled for queue: {}", queue_owned);
                        break;
                    }
                    result = consumer.batch().max_messages(10).expires(Duration::from_secs(5)).messages() => {
                        match result {
                            Ok(mut messages) => {
                                while let Some(Ok(nats_msg)) = messages.next().await {
                                    match Self::nats_to_broker_message(&nats_msg).await {
                                        Ok(broker_msg) => {
                                            match handler.handle(broker_msg).await {
                                                Ok(_) => {
                                                    if let Err(e) = nats_msg.ack().await {
                                                        tracing::error!("Failed to ack message: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("Handler error: {}", e);
                                                    // Nack with retry
                                                    if let Err(e) = nats_msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await {
                                                        tracing::error!("Failed to nack message: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to parse message: {}", e);
                                            // Ack to avoid infinite loop on malformed messages
                                            if let Err(e) = nats_msg.ack().await {
                                                tracing::error!("Failed to ack malformed message: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error fetching batch: {}", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }

            tracing::info!("Message loop ended for queue: {}", queue_owned);
        });

        Ok(SubscriptionHandle::new(queue.to_string(), cancel_token))
    }

    /// Convert NATS message to BrokerMessage
    async fn nats_to_broker_message(
        msg: &async_nats::jetstream::Message,
    ) -> Result<BrokerMessage, TaskError> {
        // Parse payload
        let payload: TaskMessage = serde_json::from_slice(&msg.payload)
            .map_err(|e| TaskError::Deserialization(format!("Failed to parse payload: {}", e)))?;

        // Extract headers
        let mut headers = HashMap::new();
        if let Some(nats_headers) = &msg.headers {
            for (key, values) in nats_headers.iter() {
                if let Some(value) = values.first() {
                    headers.insert(key.to_string(), value.to_string());
                }
            }
        }

        // Get delivery info
        let delivery_tag = msg
            .info()
            .map_err(|e| TaskError::Broker(format!("Failed to get message info: {}", e)))?
            .stream_sequence
            .to_string();

        let redelivered = msg
            .info()
            .map_err(|e| TaskError::Broker(format!("Failed to get message info: {}", e)))?
            .delivered
            > 1;

        Ok(BrokerMessage {
            delivery_tag,
            payload,
            headers,
            timestamp: Utc::now(),
            redelivered,
        })
    }
}

#[async_trait]
impl Broker for NatsBroker {
    async fn connect(&self) -> Result<(), TaskError> {
        tracing::info!("Connecting to NATS at {}", self.config.url);

        // Connect to NATS
        let client = async_nats::connect(&self.config.url)
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to connect to NATS: {}", e)))?;

        // Get JetStream context
        let jetstream = jetstream::new(client.clone());

        // Create or get stream
        let stream_config = StreamConfig {
            name: self.config.stream_name.clone(),
            subjects: vec!["tasks.>".to_string()],
            retention: RetentionPolicy::WorkQueue,
            ..Default::default()
        };

        let stream = jetstream
            .get_or_create_stream(stream_config)
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to create stream: {}", e)))?;

        tracing::info!("Connected to NATS stream: {}", self.config.stream_name);

        // Store client, jetstream, and stream
        *self.client.write().await = Some(client);
        *self.jetstream.write().await = Some(jetstream);
        *self.stream.write().await = Some(stream);

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), TaskError> {
        tracing::info!("Disconnecting from NATS");

        *self.stream.write().await = None;
        *self.jetstream.write().await = None;

        if let Some(client) = self.client.write().await.take() {
            // Flush any pending messages
            client
                .flush()
                .await
                .map_err(|e| TaskError::Broker(format!("Failed to flush: {}", e)))?;
        }

        tracing::info!("Disconnected from NATS");
        Ok(())
    }

    async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
        let js = self
            .jetstream
            .read()
            .await
            .as_ref()
            .ok_or(TaskError::NotConnected)?
            .clone();

        let subject = format!("tasks.{}", queue);
        let payload = serde_json::to_vec(&message)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Create headers
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("task-id", message.id.to_string().as_str());
        headers.insert("task-name", message.task_name.as_str());
        if let Some(ref correlation_id) = message.correlation_id {
            headers.insert("correlation-id", correlation_id.as_str());
        }

        tracing::debug!(
            "Publishing message to subject '{}': task_id={}, task_name={}",
            subject,
            message.id,
            message.task_name
        );

        // Publish with headers
        js.publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to publish: {}", e)))?
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to await ack: {}", e)))?;

        Ok(())
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        let js_guard = self.jetstream.read().await;
        let js = js_guard.as_ref().ok_or(TaskError::NotConnected)?;

        // Check if we can access the stream (verifies connection is alive)
        js.get_stream(&self.config.stream_name)
            .await
            .map_err(|e| TaskError::Broker(format!("Health check failed: {}", e)))?;

        Ok(())
    }

    fn delivery_model(&self) -> DeliveryModel {
        DeliveryModel::Pull
    }

    fn capabilities(&self) -> BrokerCapabilities {
        BrokerCapabilities {
            delayed_tasks: true,
            dead_letter: true,
            priority: false,
            batching: true,
            max_delay: None, // NATS doesn't have a hard limit
        }
    }
}

#[async_trait]
impl PullBroker for NatsBroker {
    async fn subscribe<H: MessageHandler + 'static>(
        &self,
        queue: &str,
        handler: Arc<H>,
    ) -> Result<SubscriptionHandle, TaskError> {
        // Delegate to the internal subscribe implementation
        self.subscribe_impl(queue, handler).await
    }

    async fn ack(&self, delivery_tag: &str) -> Result<(), TaskError> {
        // NATS JetStream handles ack/nack internally via the message object
        // This method is provided for trait compatibility but is not used
        // in practice - acknowledgments are handled in the subscribe loop
        tracing::warn!(
            "Direct ack called for delivery_tag={}, but NATS handles ack internally",
            delivery_tag
        );
        Err(TaskError::Internal(
            "NATS JetStream handles acknowledgments internally via message objects. \
             Ack/nack are automatically handled in the subscribe message loop."
                .to_string(),
        ))
    }

    async fn nack(&self, delivery_tag: &str, requeue: bool) -> Result<(), TaskError> {
        // NATS JetStream handles ack/nack internally via the message object
        // This method is provided for trait compatibility but is not used
        // in practice - acknowledgments are handled in the subscribe loop
        tracing::warn!(
            "Direct nack called for delivery_tag={}, requeue={}, but NATS handles nack internally",
            delivery_tag,
            requeue
        );
        Err(TaskError::Internal(
            "NATS JetStream handles acknowledgments internally via message objects. \
             Ack/nack are automatically handled in the subscribe message loop."
                .to_string(),
        ))
    }
}

#[async_trait]
impl DelayedBroker for NatsBroker {
    async fn publish_delayed(
        &self,
        queue: &str,
        message: TaskMessage,
        delay: Duration,
    ) -> Result<(), TaskError> {
        let js = self
            .jetstream
            .read()
            .await
            .as_ref()
            .ok_or(TaskError::NotConnected)?
            .clone();

        // Calculate ETA
        let eta = Utc::now()
            + chrono::Duration::from_std(delay).map_err(|e| {
                TaskError::Internal(format!(
                    "Failed to convert delay to chrono::Duration: {}",
                    e
                ))
            })?;

        let subject = format!("tasks.scheduled.{}", queue);
        let payload = serde_json::to_vec(&message)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Create headers with ETA and target queue
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("task-id", message.id.to_string().as_str());
        headers.insert("task-name", message.task_name.as_str());
        headers.insert("eta", eta.to_rfc3339().as_str());
        headers.insert("target-queue", queue);
        if let Some(ref correlation_id) = message.correlation_id {
            headers.insert("correlation-id", correlation_id.as_str());
        }

        tracing::debug!(
            "Publishing delayed message to subject '{}': task_id={}, task_name={}, eta={}",
            subject,
            message.id,
            message.task_name,
            eta
        );

        // Publish with headers
        js.publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to publish delayed: {}", e)))?
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to await ack: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = NatsBrokerConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.stream_name, "TASKS");
        assert_eq!(config.consumer_prefix, "task-worker");
        assert_eq!(config.max_pending, 1000);
        assert_eq!(config.ack_wait, Duration::from_secs(30));
        assert_eq!(config.max_deliver, 5);
    }

    #[test]
    fn test_message_serialization() {
        let message = TaskMessage::new("test_task", serde_json::json!([1, 2, 3]))
            .with_correlation_id("test-correlation-id");

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: TaskMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(message.task_name, deserialized.task_name);
        assert_eq!(message.correlation_id, deserialized.correlation_id);
    }

    #[test]
    fn test_subscription_handle() {
        let token = CancellationToken::new();
        let handle = SubscriptionHandle::new("test-queue".to_string(), token.clone());

        assert_eq!(handle.queue, "test-queue");
        assert!(!token.is_cancelled());

        handle.cancel();
        assert!(token.is_cancelled());
    }

    // Integration tests — skip if NATS unavailable
    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_connect_disconnect() {
        let config = NatsBrokerConfig::default();
        let broker = NatsBroker::new(config);
        if broker.connect().await.is_err() {
            return;
        }
        broker.health_check().await.unwrap();
        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_publish() {
        let config = NatsBrokerConfig::default();
        let broker = NatsBroker::new(config);
        if broker.connect().await.is_err() {
            return;
        }

        let message = TaskMessage::new("test_task", serde_json::json!([1, 2, 3]));
        broker.publish("test", message).await.unwrap();

        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_publish_delayed() {
        let config = NatsBrokerConfig::default();
        let broker = NatsBroker::new(config);
        if broker.connect().await.is_err() {
            return;
        }

        let message = TaskMessage::new("delayed_task", serde_json::json!([1, 2, 3]));
        broker
            .publish_delayed("test", message, Duration::from_secs(10))
            .await
            .unwrap();

        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_subscribe() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct TestHandler {
            count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl MessageHandler for TestHandler {
            async fn handle(&self, _message: BrokerMessage) -> Result<(), TaskError> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let config = NatsBrokerConfig::default();
        let broker = Arc::new(NatsBroker::new(config));

        if broker.connect().await.is_err() {
            return;
        }

        let count = Arc::new(AtomicUsize::new(0));
        let handler = Arc::new(TestHandler {
            count: count.clone(),
        });

        let handle = broker.subscribe("test", handler).await.unwrap();

        // Publish a message
        let message = TaskMessage::new("test_task", serde_json::json!([1, 2, 3]));
        broker.publish("test", message).await.unwrap();

        // Wait for processing
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check that message was received
        assert!(count.load(Ordering::SeqCst) > 0);

        // Cancel subscription
        handle.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        broker.disconnect().await.unwrap();
    }

    #[test]
    fn test_pull_broker_trait_implemented() {
        // This test verifies that NatsBroker implements the PullBroker trait
        fn assert_is_pull_broker<T: PullBroker>(_: &T) {}

        let config = NatsBrokerConfig::default();
        let broker = NatsBroker::new(config);

        assert_is_pull_broker(&broker);
    }

    #[tokio::test]
    async fn test_ack_nack_not_supported() {
        let config = NatsBrokerConfig::default();
        let broker = NatsBroker::new(config);

        // These methods should return errors indicating they are not supported
        // since NATS handles ack/nack internally
        let ack_result = broker.ack("test-delivery-tag").await;
        assert!(ack_result.is_err());
        assert!(matches!(ack_result.unwrap_err(), TaskError::Internal(_)));

        let nack_result = broker.nack("test-delivery-tag", true).await;
        assert!(nack_result.is_err());
        assert!(matches!(nack_result.unwrap_err(), TaskError::Internal(_)));
    }

    // =======================================================================
    // Additional integration tests — real NATS, skip if unavailable
    // =======================================================================

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_health_check() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());
        if broker.connect().await.is_err() {
            return;
        }

        let result = broker.health_check().await;
        assert!(
            result.is_ok(),
            "health_check should succeed on a connected broker"
        );

        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_capabilities() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());
        if broker.connect().await.is_err() {
            return;
        }

        let caps = broker.capabilities();
        assert!(
            caps.delayed_tasks,
            "NATS broker should support delayed tasks"
        );
        assert!(caps.dead_letter, "NATS broker should support dead letter");
        assert!(!caps.priority, "NATS broker should not support priority");
        assert!(caps.batching, "NATS broker should support batching");
        assert!(
            caps.max_delay.is_none(),
            "NATS broker should have no max delay limit"
        );

        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_delivery_model() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());
        if broker.connect().await.is_err() {
            return;
        }

        assert_eq!(
            broker.delivery_model(),
            DeliveryModel::Pull,
            "NATS broker delivery model should be Pull"
        );

        broker.disconnect().await.unwrap();
    }

    #[test]
    fn test_config_custom() {
        let config = NatsBrokerConfig {
            url: "nats://custom-host:5222".to_string(),
            stream_name: "MY_STREAM".to_string(),
            consumer_prefix: "my-worker".to_string(),
            max_pending: 500,
            ack_wait: Duration::from_secs(60),
            max_deliver: 10,
        };

        assert_eq!(config.url, "nats://custom-host:5222");
        assert_eq!(config.stream_name, "MY_STREAM");
        assert_eq!(config.consumer_prefix, "my-worker");
        assert_eq!(config.max_pending, 500);
        assert_eq!(config.ack_wait, Duration::from_secs(60));
        assert_eq!(config.max_deliver, 10);

        // Verify Clone works and preserves all fields
        let cloned = config.clone();
        assert_eq!(cloned.url, config.url);
        assert_eq!(cloned.stream_name, config.stream_name);
        assert_eq!(cloned.consumer_prefix, config.consumer_prefix);
        assert_eq!(cloned.max_pending, config.max_pending);
        assert_eq!(cloned.ack_wait, config.ack_wait);
        assert_eq!(cloned.max_deliver, config.max_deliver);
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_disconnect_without_connect() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());

        // Disconnect on an unconnected broker should not panic.
        // It may succeed (no-op) or return Ok since there is nothing to flush.
        let result = broker.disconnect().await;
        assert!(
            result.is_ok(),
            "disconnect on unconnected broker should not panic or error"
        );
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_publish_disconnect_reconnect() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());
        if broker.connect().await.is_err() {
            return;
        }

        // Publish first message
        let msg1 = TaskMessage::new("reconnect_task_1", serde_json::json!({"phase": "before"}));
        broker
            .publish("reconnect-test", msg1)
            .await
            .expect("first publish should succeed");

        // Disconnect
        broker
            .disconnect()
            .await
            .expect("disconnect should succeed");

        // Reconnect
        broker.connect().await.expect("reconnect should succeed");

        // Publish second message after reconnect
        let msg2 = TaskMessage::new("reconnect_task_2", serde_json::json!({"phase": "after"}));
        broker
            .publish("reconnect-test", msg2)
            .await
            .expect("publish after reconnect should succeed");

        broker.disconnect().await.unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "nats")]
    async fn test_ack_nack_with_connection() {
        let broker = NatsBroker::new(NatsBrokerConfig::default());
        if broker.connect().await.is_err() {
            return;
        }

        // NATS handles ack/nack internally via message objects, so direct
        // ack/nack calls with dummy delivery tags should return Internal errors
        // even when connected.
        let ack_result = broker.ack("dummy-delivery-tag-123").await;
        assert!(ack_result.is_err(), "direct ack should return error");
        assert!(
            matches!(ack_result.unwrap_err(), TaskError::Internal(_)),
            "ack error should be TaskError::Internal"
        );

        let nack_result = broker.nack("dummy-delivery-tag-456", false).await;
        assert!(nack_result.is_err(), "direct nack should return error");
        assert!(
            matches!(nack_result.unwrap_err(), TaskError::Internal(_)),
            "nack error should be TaskError::Internal"
        );

        let nack_requeue_result = broker.nack("dummy-delivery-tag-789", true).await;
        assert!(
            nack_requeue_result.is_err(),
            "direct nack with requeue should return error"
        );
        assert!(
            matches!(nack_requeue_result.unwrap_err(), TaskError::Internal(_)),
            "nack requeue error should be TaskError::Internal"
        );

        broker.disconnect().await.unwrap();
    }
}

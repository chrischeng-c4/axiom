//! Google Cloud Pub/Sub pull-based broker implementation

use crate::{
    broker::{Broker, BrokerCapabilities, BrokerMessage, DeliveryModel, MessageHandler, PullBroker, SubscriptionHandle},
    error::TaskError,
    message::TaskMessage,
};
use async_trait::async_trait;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::client::{Client, ClientConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// Google Cloud Pub/Sub pull-based broker configuration
#[derive(Debug, Clone)]
pub struct PubSubPullConfig {
    /// GCP project ID (None = auto-detect from environment)
    pub project_id: Option<String>,
    /// Subscription name for consuming messages
    pub subscription_name: String,
    /// Topic name for publishing messages
    pub topic_name: String,
    /// Acknowledgment deadline
    pub ack_deadline: Duration,
    /// Maximum outstanding messages
    pub max_outstanding_messages: i32,
}

impl Default for PubSubPullConfig {
    fn default() -> Self {
        Self {
            project_id: None,
            subscription_name: "task-worker".to_string(),
            topic_name: "tasks".to_string(),
            ack_deadline: Duration::from_secs(30),
            max_outstanding_messages: 1000,
        }
    }
}

/// Google Cloud Pub/Sub pull-based broker
pub struct PubSubPullBroker {
    config: PubSubPullConfig,
    client: RwLock<Option<Client>>,
}

impl PubSubPullBroker {
    /// Create a new Pub/Sub pull broker with the given configuration
    pub fn new(config: PubSubPullConfig) -> Self {
        Self {
            config,
            client: RwLock::new(None),
        }
    }
}

#[async_trait]
impl Broker for PubSubPullBroker {
    async fn connect(&self) -> Result<(), TaskError> {
        tracing::info!("Connecting to Google Cloud Pub/Sub");

        // Create client config with authentication
        let client_config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| TaskError::Connection(format!("Failed to authenticate: {}", e)))?;

        // Create client
        let client = Client::new(client_config)
            .await
            .map_err(|e| TaskError::Connection(format!("Failed to create client: {}", e)))?;

        *self.client.write().await = Some(client);

        tracing::info!("Connected to Google Cloud Pub/Sub");
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), TaskError> {
        tracing::info!("Disconnecting from Google Cloud Pub/Sub");
        *self.client.write().await = None;
        tracing::info!("Disconnected from Google Cloud Pub/Sub");
        Ok(())
    }

    async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
        let client = self.client.read().await;
        let client = client
            .as_ref()
            .ok_or(TaskError::NotConnected)?;

        let topic = client.topic(&self.config.topic_name);
        let publisher = topic.new_publisher(None);

        // Serialize message
        let data = serde_json::to_vec(&message)
            .map_err(|e| TaskError::Serialization(format!("Failed to serialize message: {}", e)))?;

        // Create attributes for message routing
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("queue".to_string(), queue.to_string());
        attributes.insert("task_id".to_string(), message.id.to_string());
        attributes.insert("task_name".to_string(), message.task_name.clone());
        if let Some(ref correlation_id) = message.correlation_id {
            attributes.insert("correlation_id".to_string(), correlation_id.clone());
        }

        // Create Pub/Sub message
        let pubsub_msg = PubsubMessage {
            data,
            attributes,
            ..Default::default()
        };

        tracing::debug!(
            "Publishing message to topic '{}': task_id={}, task_name={}, queue={}",
            self.config.topic_name,
            message.id,
            message.task_name,
            queue
        );

        // Publish and await result
        let awaiter = publisher
            .publish(pubsub_msg)
            .await;

        awaiter
            .get()
            .await
            .map_err(|e| TaskError::Broker(format!("Failed to publish message: {}", e)))?;

        Ok(())
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        let client = self.client.read().await;
        if client.is_some() {
            Ok(())
        } else {
            Err(TaskError::NotConnected)
        }
    }

    fn delivery_model(&self) -> DeliveryModel {
        DeliveryModel::Pull
    }

    fn capabilities(&self) -> BrokerCapabilities {
        BrokerCapabilities {
            delayed_tasks: false, // Pub/Sub doesn't have native delay support
            dead_letter: true,
            priority: false,
            batching: true,
            max_delay: None,
        }
    }
}

#[async_trait]
impl PullBroker for PubSubPullBroker {
    async fn subscribe<H: MessageHandler + 'static>(
        &self,
        queue: &str,
        handler: Arc<H>,
    ) -> Result<SubscriptionHandle, TaskError> {
        let client = self.client.read().await;
        let client = client
            .as_ref()
            .ok_or(TaskError::NotConnected)?;

        let subscription = client.subscription(&self.config.subscription_name);
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();
        let queue_name = queue.to_string();
        let queue_name_clone = queue_name.clone();

        tracing::info!(
            "Subscribing to subscription '{}' for queue '{}'",
            self.config.subscription_name,
            queue
        );

        // Spawn subscription task
        tokio::spawn(async move {
            tracing::info!("Starting message loop for queue: {}", queue_name);

            let result = subscription
                .receive(
                    move |msg, _cancel| {
                        let handler = handler.clone();
                        let queue_filter = queue_name_clone.clone();
                        async move {
                            // Check if message is for this queue
                            if let Some(msg_queue) = msg.message.attributes.get("queue") {
                                if msg_queue != &queue_filter {
                                    // Not for this queue, nack and let another consumer handle it
                                    tracing::debug!(
                                        "Message not for this queue (expected: {}, got: {}), nacking",
                                        queue_filter,
                                        msg_queue
                                    );
                                    let _ = msg.nack().await;
                                    return;
                                }
                            } else {
                                // No queue attribute, nack
                                tracing::warn!("Message missing queue attribute, nacking");
                                let _ = msg.nack().await;
                                return;
                            }

                            // Parse message payload
                            match serde_json::from_slice::<TaskMessage>(&msg.message.data) {
                                Ok(task_msg) => {
                                    let broker_msg = BrokerMessage {
                                        delivery_tag: msg.message.message_id.clone(),
                                        payload: task_msg,
                                        headers: msg.message.attributes.clone(),
                                        timestamp: chrono::Utc::now(),
                                        redelivered: msg.delivery_attempt().unwrap_or(0) > 1,
                                    };

                                    // Handle message
                                    match handler.handle(broker_msg).await {
                                        Ok(_) => {
                                            if let Err(e) = msg.ack().await {
                                                tracing::error!("Failed to ack message: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Handler error: {}", e);
                                            if let Err(e) = msg.nack().await {
                                                tracing::error!("Failed to nack message: {}", e);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to parse message payload: {}", e);
                                    // Ack malformed messages to avoid infinite loop
                                    if let Err(e) = msg.ack().await {
                                        tracing::error!("Failed to ack malformed message: {}", e);
                                    }
                                }
                            }
                        }
                    },
                    cancel_clone,
                    None,
                )
                .await;

            if let Err(e) = result {
                tracing::error!("Subscription error for queue {}: {}", queue_name, e);
            }

            tracing::info!("Message loop ended for queue: {}", queue_name);
        });

        Ok(SubscriptionHandle::new(queue.to_string(), cancel_token))
    }

    async fn ack(&self, _delivery_tag: &str) -> Result<(), TaskError> {
        // Pub/Sub handles ack via message object, not delivery tag
        Err(TaskError::Internal(
            "Google Cloud Pub/Sub handles acknowledgments internally via message objects. \
             Ack/nack are automatically handled in the subscribe message loop."
                .to_string(),
        ))
    }

    async fn nack(&self, _delivery_tag: &str, _requeue: bool) -> Result<(), TaskError> {
        // Pub/Sub handles nack via message object, not delivery tag
        Err(TaskError::Internal(
            "Google Cloud Pub/Sub handles acknowledgments internally via message objects. \
             Ack/nack are automatically handled in the subscribe message loop."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = PubSubPullConfig::default();
        assert_eq!(config.project_id, None);
        assert_eq!(config.subscription_name, "task-worker");
        assert_eq!(config.topic_name, "tasks");
        assert_eq!(config.ack_deadline, Duration::from_secs(30));
        assert_eq!(config.max_outstanding_messages, 1000);
    }

    #[test]
    fn test_pull_broker_trait_implemented() {
        // Verify that PubSubPullBroker implements the PullBroker trait
        fn assert_is_pull_broker<T: PullBroker>(_: &T) {}

        let config = PubSubPullConfig::default();
        let broker = PubSubPullBroker::new(config);

        assert_is_pull_broker(&broker);
    }

    #[tokio::test]
    async fn test_ack_nack_not_supported() {
        let config = PubSubPullConfig::default();
        let broker = PubSubPullBroker::new(config);

        // These methods should return errors indicating they are not supported
        // since Pub/Sub handles ack/nack internally
        let ack_result = broker.ack("test-delivery-tag").await;
        assert!(ack_result.is_err());
        assert!(matches!(ack_result.unwrap_err(), TaskError::Internal(_)));

        let nack_result = broker.nack("test-delivery-tag", true).await;
        assert!(nack_result.is_err());
        assert!(matches!(nack_result.unwrap_err(), TaskError::Internal(_)));
    }

    #[test]
    fn test_broker_capabilities() {
        let config = PubSubPullConfig::default();
        let broker = PubSubPullBroker::new(config);

        let caps = broker.capabilities();
        assert!(!caps.delayed_tasks);
        assert!(caps.dead_letter);
        assert!(!caps.priority);
        assert!(caps.batching);
        assert_eq!(caps.max_delay, None);
    }

    #[test]
    fn test_delivery_model() {
        let config = PubSubPullConfig::default();
        let broker = PubSubPullBroker::new(config);

        assert_eq!(broker.delivery_model(), DeliveryModel::Pull);
    }
}

// To run integration tests:
// 1. Start Pub/Sub emulator:
//    docker run -p 8085:8085 gcr.io/google.com/cloudsdktool/cloud-sdk:latest \
//      gcloud beta emulators pubsub start --host-port=0.0.0.0:8085
//
// 2. Set environment variable:
//    export PUBSUB_EMULATOR_HOST=localhost:8085
//
// 3. Run tests:
//    cargo test -p cclab-meteor --features pubsub -- --ignored

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Helper to check if emulator is available
    fn emulator_available() -> bool {
        std::env::var("PUBSUB_EMULATOR_HOST").is_ok()
    }

    #[tokio::test]
    #[ignore] // Run with: PUBSUB_EMULATOR_HOST=localhost:8085 cargo test --features pubsub -- --ignored
    async fn test_connect_disconnect() {
        if !emulator_available() {
            eprintln!("Skipping: PUBSUB_EMULATOR_HOST not set");
            return;
        }

        let config = PubSubPullConfig {
            project_id: Some("test-project".to_string()),
            topic_name: "test-topic".to_string(),
            subscription_name: "test-sub".to_string(),
            ..Default::default()
        };

        let broker = PubSubPullBroker::new(config);

        // Connect
        broker.connect().await.expect("Failed to connect");

        // Health check should pass
        broker.health_check().await.expect("Health check failed");

        // Disconnect
        broker.disconnect().await.expect("Failed to disconnect");
    }

    #[tokio::test]
    #[ignore]
    async fn test_publish() {
        if !emulator_available() {
            return;
        }

        let config = PubSubPullConfig {
            project_id: Some("test-project".to_string()),
            topic_name: "test-topic".to_string(),
            subscription_name: "test-sub".to_string(),
            ..Default::default()
        };

        let broker = PubSubPullBroker::new(config);
        broker.connect().await.expect("Failed to connect");

        // Create a test message
        let message = TaskMessage::new("test_task", serde_json::json!({"key": "value"}));

        // Publish should succeed
        broker.publish("default", message).await.expect("Failed to publish");

        broker.disconnect().await.ok();
    }

    #[tokio::test]
    #[ignore]
    async fn test_subscribe_and_receive() {
        if !emulator_available() {
            return;
        }

        let config = PubSubPullConfig {
            project_id: Some("test-project".to_string()),
            topic_name: "test-topic-recv".to_string(),
            subscription_name: "test-sub-recv".to_string(),
            ..Default::default()
        };

        let broker = Arc::new(PubSubPullBroker::new(config));
        broker.connect().await.expect("Failed to connect");

        // Counter to track received messages
        let received = Arc::new(AtomicUsize::new(0));
        let received_clone = received.clone();

        // Create handler
        struct TestHandler {
            count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl MessageHandler for TestHandler {
            async fn handle(&self, _msg: BrokerMessage) -> Result<(), TaskError> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler { count: received_clone });

        // Subscribe
        let handle = broker.subscribe("default", handler).await.expect("Failed to subscribe");

        // Publish a message
        let message = TaskMessage::new("test_task", serde_json::json!({"test": true}));
        broker.publish("default", message).await.expect("Failed to publish");

        // Wait for message to be received
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cancel subscription
        handle.cancel();

        // Verify message was received
        assert!(received.load(Ordering::SeqCst) >= 1, "Expected at least 1 message");

        broker.disconnect().await.ok();
    }
}

//! Message broker implementations
//!
//! Provides traits and implementations for task message brokers.

pub mod config;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::{TaskError, TaskMessage};

/// Delivery model for broker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryModel {
    /// Worker pulls messages from broker (NATS, Pub/Sub pull)
    Pull,
    /// Broker pushes messages to worker via HTTP (Cloud Tasks, Pub/Sub push)
    Push,
}

/// Broker feature capabilities
#[derive(Debug, Clone, Default)]
pub struct BrokerCapabilities {
    /// Supports native delayed/scheduled tasks
    pub delayed_tasks: bool,
    /// Supports dead-letter queues
    pub dead_letter: bool,
    /// Supports message priority
    pub priority: bool,
    /// Supports message batching
    pub batching: bool,
    /// Maximum delay duration (if delayed_tasks is true)
    pub max_delay: Option<Duration>,
}

/// Trait for message broker implementations
#[async_trait]
pub trait Broker: Send + Sync + 'static {
    /// Connect to the broker
    async fn connect(&self) -> Result<(), TaskError>;

    /// Disconnect from the broker
    async fn disconnect(&self) -> Result<(), TaskError>;

    /// Publish a task message to a queue
    async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError>;

    /// Check if broker is healthy
    async fn health_check(&self) -> Result<(), TaskError>;

    /// Get the delivery model of this broker
    fn delivery_model(&self) -> DeliveryModel;

    /// Get the capabilities of this broker
    fn capabilities(&self) -> BrokerCapabilities;
}

/// Trait for pull-based brokers (worker fetches messages)
#[async_trait]
pub trait PullBroker: Broker {
    /// Subscribe to a queue and receive messages
    async fn subscribe<H: MessageHandler + 'static>(
        &self,
        queue: &str,
        handler: Arc<H>,
    ) -> Result<SubscriptionHandle, TaskError>;

    /// Acknowledge a message
    async fn ack(&self, delivery_tag: &str) -> Result<(), TaskError>;

    /// Negative acknowledge (requeue or DLQ)
    async fn nack(&self, delivery_tag: &str, requeue: bool) -> Result<(), TaskError>;
}

/// Trait for push-based brokers (broker sends HTTP to worker)
pub trait PushBroker: Broker {
    /// Parse an incoming HTTP request into a BrokerMessage
    fn parse_push_request(&self, headers: &HashMap<String, String>, body: &[u8])
        -> Result<BrokerMessage, TaskError>;

    /// Get HTTP status code for successful ack
    fn ack_status_code(&self) -> u16 { 200 }

    /// Get HTTP status code for nack (retry)
    fn nack_status_code(&self) -> u16 { 500 }

    /// Get the expected endpoint path pattern
    fn endpoint_path(&self) -> &str;
}

/// Trait for brokers with native delayed task support
#[async_trait]
pub trait DelayedBroker: Broker {
    /// Publish with native delay support
    async fn publish_delayed(
        &self,
        queue: &str,
        message: TaskMessage,
        delay: Duration,
    ) -> Result<(), TaskError>;

    /// Publish at specific time
    async fn publish_at(
        &self,
        queue: &str,
        message: TaskMessage,
        eta: DateTime<Utc>,
    ) -> Result<(), TaskError> {
        let now = Utc::now();
        if eta <= now {
            // Execute immediately
            self.publish(queue, message).await
        } else {
            let delay = (eta - now).to_std().unwrap_or(Duration::ZERO);
            self.publish_delayed(queue, message, delay).await
        }
    }
}

/// Message received from the broker
#[derive(Debug, Clone)]
pub struct BrokerMessage {
    /// Delivery tag for acknowledgment
    pub delivery_tag: String,
    /// Task message payload
    pub payload: TaskMessage,
    /// Message headers
    pub headers: HashMap<String, String>,
    /// Timestamp when message was received
    pub timestamp: DateTime<Utc>,
    /// Whether this is a redelivery
    pub redelivered: bool,
}

/// Handler for incoming messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle(&self, message: BrokerMessage) -> Result<(), TaskError>;
}

/// Handle for managing subscriptions
pub struct SubscriptionHandle {
    /// Queue name
    pub queue: String,
    /// Cancellation token
    cancel_token: CancellationToken,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    pub fn new(queue: String, cancel_token: CancellationToken) -> Self {
        Self { queue, cancel_token }
    }

    /// Cancel the subscription
    pub fn cancel(&self) {
        tracing::info!("Cancelling subscription for queue: {}", self.queue);
        self.cancel_token.cancel();
    }
}

// NATS JetStream broker implementation
#[cfg(feature = "nats")]
pub mod nats;

#[cfg(feature = "nats")]
pub use nats::{NatsBroker, NatsBrokerConfig};

// Google Cloud Pub/Sub broker implementation
#[cfg(feature = "pubsub")]
pub mod pubsub;

#[cfg(feature = "pubsub")]
pub use pubsub::{PubSubPullBroker, PubSubPullConfig};

#[cfg(feature = "pubsub-push")]
pub use pubsub::{PubSubPushBroker, PubSubPushConfig};

// Cloud Tasks broker implementation
#[cfg(feature = "cloudtasks")]
pub mod cloudtasks;

#[cfg(feature = "cloudtasks")]
pub use cloudtasks::{CloudTasksBroker, CloudTasksConfig};

// Unified broker configuration
pub use config::BrokerConfig;

#[cfg(any(feature = "nats", feature = "pubsub"))]
pub use config::BrokerInstance;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;
    use chrono::Utc;
    use crate::{TaskError, TaskMessage};
    use tokio_util::sync::CancellationToken;

    // -----------------------------------------------------------------------
    // R6: DeliveryModel enum
    // -----------------------------------------------------------------------

    #[test]
    fn test_delivery_model_variants() {
        let pull = DeliveryModel::Pull;
        let push = DeliveryModel::Push;
        assert_ne!(pull, push);
        assert_eq!(pull, DeliveryModel::Pull);
        assert_eq!(push, DeliveryModel::Push);
    }

    #[test]
    fn test_delivery_model_clone_copy() {
        let model = DeliveryModel::Push;
        let cloned = model.clone();
        let copied = model; // Copy
        assert_eq!(model, cloned);
        assert_eq!(model, copied);
    }

    #[test]
    fn test_delivery_model_debug() {
        let pull = DeliveryModel::Pull;
        let push = DeliveryModel::Push;
        assert_eq!(format!("{:?}", pull), "Pull");
        assert_eq!(format!("{:?}", push), "Push");
    }

    // -----------------------------------------------------------------------
    // R5: BrokerCapabilities struct
    // -----------------------------------------------------------------------

    #[test]
    fn test_broker_capabilities_default() {
        let caps = BrokerCapabilities::default();
        assert!(!caps.delayed_tasks);
        assert!(!caps.dead_letter);
        assert!(!caps.priority);
        assert!(!caps.batching);
        assert!(caps.max_delay.is_none());
    }

    #[test]
    fn test_broker_capabilities_custom() {
        let caps = BrokerCapabilities {
            delayed_tasks: true,
            dead_letter: true,
            priority: false,
            batching: false,
            max_delay: Some(Duration::from_secs(30 * 24 * 60 * 60)),
        };
        assert!(caps.delayed_tasks);
        assert!(caps.dead_letter);
        assert!(!caps.priority);
        assert!(!caps.batching);
        assert_eq!(caps.max_delay, Some(Duration::from_secs(2_592_000)));
    }

    #[test]
    fn test_broker_capabilities_clone() {
        let caps = BrokerCapabilities {
            delayed_tasks: true,
            dead_letter: false,
            priority: true,
            batching: false,
            max_delay: Some(Duration::from_secs(3600)),
        };
        let cloned = caps.clone();
        assert_eq!(cloned.delayed_tasks, caps.delayed_tasks);
        assert_eq!(cloned.dead_letter, caps.dead_letter);
        assert_eq!(cloned.priority, caps.priority);
        assert_eq!(cloned.batching, caps.batching);
        assert_eq!(cloned.max_delay, caps.max_delay);
    }

    // -----------------------------------------------------------------------
    // R7: BrokerMessage struct
    // -----------------------------------------------------------------------

    #[test]
    fn test_broker_message_construction() {
        let msg = TaskMessage::new("test_task", serde_json::json!(["arg1"]));
        let mut headers = HashMap::new();
        headers.insert("x-cloudtasks-taskname".to_string(), "task-123".to_string());
        let now = Utc::now();

        let broker_msg = BrokerMessage {
            delivery_tag: "task-123".to_string(),
            payload: msg,
            headers: headers.clone(),
            timestamp: now,
            redelivered: false,
        };

        assert_eq!(broker_msg.delivery_tag, "task-123");
        assert_eq!(broker_msg.payload.task_name, "test_task");
        assert_eq!(broker_msg.headers.get("x-cloudtasks-taskname").unwrap(), "task-123");
        assert_eq!(broker_msg.timestamp, now);
        assert!(!broker_msg.redelivered);
    }

    #[test]
    fn test_broker_message_redelivered() {
        let msg = TaskMessage::new("retry_task", serde_json::json!([]));
        let broker_msg = BrokerMessage {
            delivery_tag: "tag-456".to_string(),
            payload: msg,
            headers: HashMap::new(),
            timestamp: Utc::now(),
            redelivered: true,
        };

        assert!(broker_msg.redelivered);
    }

    #[test]
    fn test_broker_message_clone() {
        let msg = TaskMessage::new("clone_task", serde_json::json!([1, 2, 3]));
        let broker_msg = BrokerMessage {
            delivery_tag: "tag-clone".to_string(),
            payload: msg,
            headers: HashMap::new(),
            timestamp: Utc::now(),
            redelivered: false,
        };

        let cloned = broker_msg.clone();
        assert_eq!(cloned.delivery_tag, broker_msg.delivery_tag);
        assert_eq!(cloned.payload.task_name, broker_msg.payload.task_name);
        assert_eq!(cloned.redelivered, broker_msg.redelivered);
    }

    // -----------------------------------------------------------------------
    // SubscriptionHandle
    // -----------------------------------------------------------------------

    #[test]
    fn test_subscription_handle_creation() {
        let token = CancellationToken::new();
        let handle = SubscriptionHandle::new("my-queue".to_string(), token.clone());
        assert_eq!(handle.queue, "my-queue");
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_subscription_handle_cancel() {
        let token = CancellationToken::new();
        let handle = SubscriptionHandle::new("cancel-queue".to_string(), token.clone());
        assert!(!token.is_cancelled());

        handle.cancel();
        assert!(token.is_cancelled());
    }

    // -----------------------------------------------------------------------
    // S2: PushBroker default ack/nack status codes (R3)
    // -----------------------------------------------------------------------

    /// Mock PushBroker to test default trait method implementations
    struct MockPushBroker;

    #[async_trait]
    impl Broker for MockPushBroker {
        async fn connect(&self) -> Result<(), TaskError> { Ok(()) }
        async fn disconnect(&self) -> Result<(), TaskError> { Ok(()) }
        async fn publish(&self, _queue: &str, _message: TaskMessage) -> Result<(), TaskError> { Ok(()) }
        async fn health_check(&self) -> Result<(), TaskError> { Ok(()) }
        fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Push }
        fn capabilities(&self) -> BrokerCapabilities { BrokerCapabilities::default() }
    }

    impl PushBroker for MockPushBroker {
        fn parse_push_request(
            &self,
            _headers: &HashMap<String, String>,
            body: &[u8],
        ) -> Result<BrokerMessage, TaskError> {
            let payload: TaskMessage = serde_json::from_slice(body)
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(BrokerMessage {
                delivery_tag: "mock-tag".to_string(),
                payload,
                headers: HashMap::new(),
                timestamp: Utc::now(),
                redelivered: false,
            })
        }

        fn endpoint_path(&self) -> &str {
            "/mock/push/{queue}"
        }
    }

    #[test]
    fn test_push_broker_default_ack_status_code() {
        let broker = MockPushBroker;
        assert_eq!(broker.ack_status_code(), 200);
    }

    #[test]
    fn test_push_broker_default_nack_status_code() {
        let broker = MockPushBroker;
        assert_eq!(broker.nack_status_code(), 500);
    }

    #[test]
    fn test_push_broker_endpoint_path() {
        let broker = MockPushBroker;
        assert_eq!(broker.endpoint_path(), "/mock/push/{queue}");
    }

    #[test]
    fn test_push_broker_parse_push_request() {
        let broker = MockPushBroker;
        let msg = TaskMessage::new("push_task", serde_json::json!(["a", "b"]));
        let body = serde_json::to_vec(&msg).unwrap();

        let result = broker.parse_push_request(&HashMap::new(), &body);
        assert!(result.is_ok());
        let broker_msg = result.unwrap();
        assert_eq!(broker_msg.delivery_tag, "mock-tag");
        assert_eq!(broker_msg.payload.task_name, "push_task");
        assert!(!broker_msg.redelivered);
    }

    #[test]
    fn test_push_broker_parse_invalid_body() {
        let broker = MockPushBroker;
        let invalid_body = b"not valid json";

        let result = broker.parse_push_request(&HashMap::new(), invalid_body);
        assert!(result.is_err());
        match result.unwrap_err() {
            TaskError::Deserialization(_) => {} // expected
            other => panic!("Expected Deserialization error, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // S4: DelayedBroker default publish_at (R4)
    // -----------------------------------------------------------------------

    /// Mock DelayedBroker to test default publish_at implementation
    struct MockDelayedBroker {
        /// Track which method was called
        published_immediate: std::sync::Arc<std::sync::atomic::AtomicBool>,
        published_delayed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    impl MockDelayedBroker {
        fn new() -> Self {
            Self {
                published_immediate: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                published_delayed: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }
    }

    #[async_trait]
    impl Broker for MockDelayedBroker {
        async fn connect(&self) -> Result<(), TaskError> { Ok(()) }
        async fn disconnect(&self) -> Result<(), TaskError> { Ok(()) }
        async fn publish(&self, _queue: &str, _message: TaskMessage) -> Result<(), TaskError> {
            self.published_immediate.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        async fn health_check(&self) -> Result<(), TaskError> { Ok(()) }
        fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Push }
        fn capabilities(&self) -> BrokerCapabilities {
            BrokerCapabilities {
                delayed_tasks: true,
                ..Default::default()
            }
        }
    }

    #[async_trait]
    impl DelayedBroker for MockDelayedBroker {
        async fn publish_delayed(
            &self,
            _queue: &str,
            _message: TaskMessage,
            _delay: Duration,
        ) -> Result<(), TaskError> {
            self.published_delayed.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        // Uses default publish_at implementation
    }

    #[tokio::test]
    async fn test_delayed_broker_publish_at_past_eta_calls_publish() {
        let broker = MockDelayedBroker::new();
        let past_eta = Utc::now() - chrono::Duration::hours(1);
        let msg = TaskMessage::new("past_task", serde_json::json!([]));

        let result = broker.publish_at("test-queue", msg, past_eta).await;
        assert!(result.is_ok());
        assert!(broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
            "Past ETA should call publish() immediately");
        assert!(!broker.published_delayed.load(std::sync::atomic::Ordering::SeqCst),
            "Past ETA should NOT call publish_delayed()");
    }

    #[tokio::test]
    async fn test_delayed_broker_publish_at_future_eta_calls_publish_delayed() {
        let broker = MockDelayedBroker::new();
        let future_eta = Utc::now() + chrono::Duration::hours(1);
        let msg = TaskMessage::new("future_task", serde_json::json!([]));

        let result = broker.publish_at("test-queue", msg, future_eta).await;
        assert!(result.is_ok());
        assert!(!broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
            "Future ETA should NOT call publish() immediately");
        assert!(broker.published_delayed.load(std::sync::atomic::Ordering::SeqCst),
            "Future ETA should call publish_delayed()");
    }

    #[tokio::test]
    async fn test_delayed_broker_publish_at_now_calls_publish() {
        let broker = MockDelayedBroker::new();
        // ETA exactly now (or slightly in the past by the time it executes)
        let now_eta = Utc::now();
        let msg = TaskMessage::new("now_task", serde_json::json!([]));

        let result = broker.publish_at("test-queue", msg, now_eta).await;
        assert!(result.is_ok());
        // eta <= now should trigger immediate publish
        assert!(broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
            "ETA at now should call publish() immediately");
    }

    // -----------------------------------------------------------------------
    // S6: CloudTasksBroker capabilities and delivery model (feature-gated)
    // -----------------------------------------------------------------------

    #[cfg(feature = "cloudtasks")]
    mod cloudtasks_tests {
        use super::*;
        use crate::broker::cloudtasks::{CloudTasksBroker, CloudTasksConfig};

        fn test_config() -> CloudTasksConfig {
            CloudTasksConfig {
                project_id: "test-project".to_string(),
                location: "us-central1".to_string(),
                worker_url: "https://worker.example.com".to_string(),
                service_account_email: None,
                oidc_audience: None,
                default_queue: "default".to_string(),
                dispatch_deadline: Duration::from_secs(600),
                max_retry_count: None,
                credentials_path: None,
            }
        }

        #[test]
        fn test_cloudtasks_delivery_model_is_push() {
            let broker = CloudTasksBroker::new(test_config());
            assert_eq!(broker.delivery_model(), DeliveryModel::Push);
        }

        #[test]
        fn test_cloudtasks_capabilities() {
            let broker = CloudTasksBroker::new(test_config());
            let caps = broker.capabilities();
            assert!(caps.delayed_tasks, "Cloud Tasks supports delayed tasks");
            assert!(caps.dead_letter, "Cloud Tasks supports dead letter");
            assert!(!caps.priority, "Cloud Tasks does not support priority");
            assert!(!caps.batching, "Cloud Tasks does not support batching");
            assert_eq!(
                caps.max_delay,
                Some(Duration::from_secs(30 * 24 * 60 * 60)),
                "Cloud Tasks max delay should be 30 days"
            );
        }

        // S2: PushBroker ack/nack status codes for CloudTasksBroker
        #[test]
        fn test_cloudtasks_ack_status_code() {
            let broker = CloudTasksBroker::new(test_config());
            assert_eq!(broker.ack_status_code(), 200);
        }

        #[test]
        fn test_cloudtasks_nack_status_code() {
            let broker = CloudTasksBroker::new(test_config());
            assert_eq!(broker.nack_status_code(), 500);
        }

        #[test]
        fn test_cloudtasks_endpoint_path() {
            let broker = CloudTasksBroker::new(test_config());
            assert_eq!(broker.endpoint_path(), "/meteor/push/{queue}");
        }

        // S1: Parse push request with Cloud Tasks headers
        #[test]
        fn test_cloudtasks_parse_push_request_with_task_header() {
            let broker = CloudTasksBroker::new(test_config());
            let msg = TaskMessage::new("cloud_task", serde_json::json!({"key": "value"}));
            let body = serde_json::to_vec(&msg).unwrap();

            let mut headers = HashMap::new();
            headers.insert("x-cloudtasks-taskname".to_string(), "projects/test/locations/us/queues/q/tasks/t123".to_string());
            headers.insert("x-cloudtasks-taskretrycount".to_string(), "0".to_string());

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_ok());
            let broker_msg = result.unwrap();
            assert_eq!(broker_msg.delivery_tag, "projects/test/locations/us/queues/q/tasks/t123");
            assert_eq!(broker_msg.payload.task_name, "cloud_task");
            assert!(!broker_msg.redelivered, "retry count 0 should not be redelivered");
        }

        // S1: Redelivery detection via retry count header
        #[test]
        fn test_cloudtasks_parse_push_request_redelivered() {
            let broker = CloudTasksBroker::new(test_config());
            let msg = TaskMessage::new("retry_task", serde_json::json!([]));
            let body = serde_json::to_vec(&msg).unwrap();

            let mut headers = HashMap::new();
            headers.insert("x-cloudtasks-taskname".to_string(), "task-retry-1".to_string());
            headers.insert("x-cloudtasks-taskretrycount".to_string(), "3".to_string());

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_ok());
            let broker_msg = result.unwrap();
            assert!(broker_msg.redelivered, "retry count > 0 should be redelivered");
        }

        // S1: Missing task name header falls back to task ID
        #[test]
        fn test_cloudtasks_parse_push_request_no_taskname_header() {
            let broker = CloudTasksBroker::new(test_config());
            let msg = TaskMessage::new("fallback_task", serde_json::json!([]));
            let task_id_str = msg.id.to_string();
            let body = serde_json::to_vec(&msg).unwrap();

            let headers = HashMap::new(); // no Cloud Tasks headers

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_ok());
            let broker_msg = result.unwrap();
            assert_eq!(broker_msg.delivery_tag, task_id_str,
                "Missing x-cloudtasks-taskname should fall back to task ID");
        }

        // Parse invalid JSON body
        #[test]
        fn test_cloudtasks_parse_push_request_invalid_json() {
            let broker = CloudTasksBroker::new(test_config());
            let headers = HashMap::new();
            let invalid_body = b"this is not json";

            let result = broker.parse_push_request(&headers, invalid_body);
            assert!(result.is_err());
            match result.unwrap_err() {
                TaskError::Deserialization(_) => {} // expected
                other => panic!("Expected Deserialization error, got: {:?}", other),
            }
        }

        // OIDC auth header required when service_account_email is set
        #[test]
        fn test_cloudtasks_parse_push_request_missing_auth() {
            let mut config = test_config();
            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
            let broker = CloudTasksBroker::new(config);

            let msg = TaskMessage::new("auth_task", serde_json::json!([]));
            let body = serde_json::to_vec(&msg).unwrap();
            let headers = HashMap::new(); // no Authorization header

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_err());
            match result.unwrap_err() {
                TaskError::Authentication(msg) => {
                    assert!(msg.contains("Authorization"), "Error should mention Authorization header");
                }
                other => panic!("Expected Authentication error, got: {:?}", other),
            }
        }

        // OIDC auth header with invalid format
        #[test]
        fn test_cloudtasks_parse_push_request_invalid_auth_format() {
            let mut config = test_config();
            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
            let broker = CloudTasksBroker::new(config);

            let msg = TaskMessage::new("auth_task", serde_json::json!([]));
            let body = serde_json::to_vec(&msg).unwrap();
            let mut headers = HashMap::new();
            headers.insert("authorization".to_string(), "Basic dXNlcjpwYXNz".to_string());

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_err());
            match result.unwrap_err() {
                TaskError::Authentication(msg) => {
                    assert!(msg.contains("Invalid"), "Error should mention invalid format");
                }
                other => panic!("Expected Authentication error, got: {:?}", other),
            }
        }

        // Successful OIDC auth when Bearer token is present with valid claims
        #[test]
        fn test_cloudtasks_parse_push_request_valid_bearer_token() {
            let mut config = test_config();
            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
            let broker = CloudTasksBroker::new(config);

            let msg = TaskMessage::new("auth_ok_task", serde_json::json!([]));
            let body = serde_json::to_vec(&msg).unwrap();

            // Build a valid JWT with proper base64url-encoded claims
            // Audience must match effective_audience (worker_url since oidc_audience is None)
            let jwt_token = make_test_jwt(
                "sa@project.iam.gserviceaccount.com",
                "https://worker.example.com",
                chrono::Utc::now().timestamp() + 3600,
            );

            let mut headers = HashMap::new();
            headers.insert("authorization".to_string(), format!("Bearer {}", jwt_token));
            headers.insert("x-cloudtasks-taskname".to_string(), "auth-task-1".to_string());

            let result = broker.parse_push_request(&headers, &body);
            assert!(result.is_ok(), "Valid Bearer token should pass auth check");
            assert_eq!(result.unwrap().delivery_tag, "auth-task-1");
        }

        /// Build a fake JWT with properly base64url-encoded claims for testing
        fn make_test_jwt(email: &str, audience: &str, exp: i64) -> String {
            use base64::Engine;
            let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(r#"{"alg":"RS256","typ":"JWT"}"#);
            let payload_json = serde_json::json!({
                "email": email,
                "aud": audience,
                "exp": exp,
                "iss": "accounts.google.com",
            });
            let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(payload_json.to_string().as_bytes());
            format!("{}.{}.fake_sig", header, payload)
        }

        // CloudTasksConfig default values
        #[test]
        fn test_cloudtasks_config_default() {
            let config = CloudTasksConfig::default();
            assert!(config.project_id.is_empty());
            assert_eq!(config.location, "us-central1");
            assert!(config.worker_url.is_empty());
            assert!(config.service_account_email.is_none());
            assert!(config.oidc_audience.is_none());
            assert_eq!(config.default_queue, "default");
            assert_eq!(config.dispatch_deadline, Duration::from_secs(600));
            assert!(config.max_retry_count.is_none());
            assert!(config.credentials_path.is_none());
        }

        // Connect validation: empty project_id
        #[tokio::test]
        async fn test_cloudtasks_connect_empty_project_id() {
            let config = CloudTasksConfig {
                project_id: String::new(),
                worker_url: "https://example.com".to_string(),
                ..Default::default()
            };
            let broker = CloudTasksBroker::new(config);
            let result = broker.connect().await;
            assert!(result.is_err());
            match result.unwrap_err() {
                TaskError::Configuration(msg) => {
                    assert!(msg.contains("project_id"));
                }
                other => panic!("Expected Configuration error, got: {:?}", other),
            }
        }

        // Connect validation: empty worker_url
        #[tokio::test]
        async fn test_cloudtasks_connect_empty_worker_url() {
            let config = CloudTasksConfig {
                project_id: "test-project".to_string(),
                worker_url: String::new(),
                ..Default::default()
            };
            let broker = CloudTasksBroker::new(config);
            let result = broker.connect().await;
            assert!(result.is_err());
            match result.unwrap_err() {
                TaskError::Configuration(msg) => {
                    assert!(msg.contains("worker_url"));
                }
                other => panic!("Expected Configuration error, got: {:?}", other),
            }
        }

        // Disconnect clears client
        #[tokio::test]
        async fn test_cloudtasks_disconnect() {
            let broker = CloudTasksBroker::new(test_config());
            // connect first
            broker.connect().await.unwrap();
            // then disconnect
            let result = broker.disconnect().await;
            assert!(result.is_ok());
        }
    }
}

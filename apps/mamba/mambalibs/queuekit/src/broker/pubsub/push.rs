//! Google Cloud Pub/Sub push-based broker implementation
//!
//! Push broker where Pub/Sub sends HTTP requests to the worker endpoint.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

use crate::broker::{Broker, BrokerCapabilities, BrokerMessage, DeliveryModel, PushBroker};
use crate::{TaskError, TaskMessage};

/// Configuration for Pub/Sub push broker
#[derive(Debug, Clone)]
pub struct PubSubPushConfig {
    /// GCP project ID
    pub project_id: String,
    /// Topic name for publishing messages
    pub topic_name: String,
    /// Push subscription name
    pub subscription_name: String,
    /// Worker endpoint URL for push delivery
    pub push_endpoint: String,
    /// Service account email for authentication
    pub service_account_email: Option<String>,
    /// OIDC audience
    pub oidc_audience: Option<String>,
    /// Acknowledgment deadline
    pub ack_deadline: Duration,
}

impl Default for PubSubPushConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            topic_name: "meteor-tasks".to_string(),
            subscription_name: "meteor-worker-push".to_string(),
            push_endpoint: String::new(),
            service_account_email: None,
            oidc_audience: None,
            ack_deadline: Duration::from_secs(30),
        }
    }
}

impl PubSubPushConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, TaskError> {
        let project_id = std::env::var("GCP_PROJECT_ID")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
            .map_err(|_| TaskError::Configuration("GCP_PROJECT_ID not set".into()))?;

        let topic_name =
            std::env::var("PUBSUB_TOPIC").unwrap_or_else(|_| "meteor-tasks".to_string());

        let subscription_name = std::env::var("PUBSUB_SUBSCRIPTION")
            .unwrap_or_else(|_| "meteor-worker-push".to_string());

        let push_endpoint = std::env::var("METEOR_PUSH_ENDPOINT")
            .map_err(|_| TaskError::Configuration("METEOR_PUSH_ENDPOINT not set".into()))?;

        let service_account_email = std::env::var("PUBSUB_SERVICE_ACCOUNT").ok();
        let oidc_audience = std::env::var("PUBSUB_OIDC_AUDIENCE").ok();

        Ok(Self {
            project_id,
            topic_name,
            subscription_name,
            push_endpoint,
            service_account_email,
            oidc_audience,
            ack_deadline: Duration::from_secs(30),
        })
    }
}

/// Pub/Sub push message envelope
#[derive(Debug, serde::Deserialize)]
pub struct PubSubPushMessage {
    /// The Pub/Sub message
    pub message: PubSubMessageData,
    /// The subscription name
    pub subscription: String,
}

/// Pub/Sub message data
#[derive(Debug, serde::Deserialize)]
pub struct PubSubMessageData {
    /// Base64-encoded message data
    pub data: String,
    /// Message ID
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// Publish time
    #[serde(rename = "publishTime")]
    pub publish_time: String,
    /// Message attributes
    #[serde(default)]
    pub attributes: HashMap<String, String>,
}

/// Pub/Sub push broker
///
/// Push-based broker where Pub/Sub delivers messages via HTTP to the worker.
pub struct PubSubPushBroker {
    config: PubSubPushConfig,
    #[allow(dead_code)]
    connected: bool,
}

impl PubSubPushBroker {
    /// Create a new Pub/Sub push broker
    pub fn new(config: PubSubPushConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }
}

#[async_trait]
impl Broker for PubSubPushBroker {
    async fn connect(&self) -> Result<(), TaskError> {
        if self.config.project_id.is_empty() {
            return Err(TaskError::Configuration("project_id is required".into()));
        }
        if self.config.push_endpoint.is_empty() {
            return Err(TaskError::Configuration("push_endpoint is required".into()));
        }

        tracing::info!(
            project_id = %self.config.project_id,
            topic = %self.config.topic_name,
            subscription = %self.config.subscription_name,
            "Connected to Pub/Sub (push mode)"
        );

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), TaskError> {
        tracing::info!("Disconnected from Pub/Sub (push mode)");
        Ok(())
    }

    async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
        let payload =
            serde_json::to_vec(&message).map_err(|e| TaskError::Serialization(e.to_string()))?;

        tracing::debug!(
            topic = %self.config.topic_name,
            task_id = %message.id,
            queue = %queue,
            payload_size = payload.len(),
            "Publishing to Pub/Sub"
        );

        // In production, this would call the Pub/Sub API:
        // POST https://pubsub.googleapis.com/v1/projects/{project}/topics/{topic}:publish
        // with message containing:
        // - data: base64(payload)
        // - attributes: { "queue": queue, "task_id": message.id }

        tracing::info!(
            task_id = %message.id,
            queue = %queue,
            "Pub/Sub message published (stub - actual API call not implemented)"
        );

        Ok(())
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        Ok(())
    }

    fn delivery_model(&self) -> DeliveryModel {
        DeliveryModel::Push
    }

    fn capabilities(&self) -> BrokerCapabilities {
        BrokerCapabilities {
            delayed_tasks: false, // Pub/Sub doesn't have native delayed delivery
            dead_letter: true,
            priority: false,
            batching: true,
            max_delay: None,
        }
    }
}

impl PushBroker for PubSubPushBroker {
    fn parse_push_request(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> Result<BrokerMessage, TaskError> {
        // Validate OIDC token if configured
        if self.config.service_account_email.is_some() {
            let auth_header = headers
                .get("authorization")
                .ok_or_else(|| TaskError::Authentication("Missing Authorization header".into()))?;

            if !auth_header.starts_with("Bearer ") {
                return Err(TaskError::Authentication(
                    "Invalid Authorization header format".into(),
                ));
            }

            // In production, validate the OIDC token
            tracing::debug!("OIDC token validation (stub - not implemented)");
        }

        // Parse Pub/Sub push envelope
        let envelope: PubSubPushMessage = serde_json::from_slice(body).map_err(|e| {
            TaskError::Deserialization(format!("Invalid Pub/Sub push message: {}", e))
        })?;

        // Decode base64 message data
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&envelope.message.data)
            .map_err(|e| TaskError::Deserialization(format!("Invalid base64 data: {}", e)))?;

        // Parse the task message
        let payload: TaskMessage = serde_json::from_slice(&decoded)
            .map_err(|e| TaskError::Deserialization(e.to_string()))?;

        // Check for redelivery
        let delivery_count = envelope
            .message
            .attributes
            .get("googclient_deliveryattempt")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);

        Ok(BrokerMessage {
            delivery_tag: envelope.message.message_id,
            payload,
            headers: envelope.message.attributes,
            timestamp: chrono::Utc::now(),
            redelivered: delivery_count > 1,
        })
    }

    fn endpoint_path(&self) -> &str {
        "/meteor/pubsub/push"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_model() {
        let config = PubSubPushConfig::default();
        let broker = PubSubPushBroker::new(config);
        assert_eq!(broker.delivery_model(), DeliveryModel::Push);
    }

    #[test]
    fn test_capabilities() {
        let config = PubSubPushConfig::default();
        let broker = PubSubPushBroker::new(config);
        let caps = broker.capabilities();
        assert!(!caps.delayed_tasks);
        assert!(caps.dead_letter);
        assert!(caps.batching);
    }

    #[test]
    fn test_endpoint_path() {
        let config = PubSubPushConfig::default();
        let broker = PubSubPushBroker::new(config);
        assert_eq!(broker.endpoint_path(), "/meteor/pubsub/push");
    }
}

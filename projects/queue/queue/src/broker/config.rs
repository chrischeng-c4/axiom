//! Broker configuration for runtime selection.
//!
//! # Example
//! ```rust,ignore
//! use cclab_queue::BrokerConfig;
//!
//! // From environment
//! let config = BrokerConfig::from_env()?;
//!
//! // Or explicit configuration
//! let config = BrokerConfig::Nats(NatsBrokerConfig::default());
//!
//! // Create broker instance
//! let broker = config.into_broker();
//! ```

use crate::error::TaskError;

#[cfg(feature = "nats")]
use super::nats::{NatsBroker, NatsBrokerConfig};

#[cfg(feature = "pubsub")]
use super::pubsub::{PubSubPullBroker, PubSubPullConfig};

/// Unified broker configuration enum.
///
/// Supports runtime selection of broker implementation based on
/// compile-time features.
#[derive(Debug, Clone)]
pub enum BrokerConfig {
    /// NATS JetStream broker (pull-based)
    #[cfg(feature = "nats")]
    Nats(NatsBrokerConfig),

    /// Google Cloud Pub/Sub pull broker
    #[cfg(feature = "pubsub")]
    PubSub(PubSubPullConfig),
}

impl BrokerConfig {
    /// Create broker configuration from environment variables.
    ///
    /// Checks `BROKER_TYPE` environment variable:
    /// - `nats` -> Uses NATS_URL, etc.
    /// - `pubsub` -> Uses PUBSUB_PROJECT_ID, PUBSUB_TOPIC, PUBSUB_SUBSCRIPTION
    ///
    /// If `BROKER_TYPE` is not set, defaults to NATS if available.
    pub fn from_env() -> Result<Self, TaskError> {
        let broker_type = std::env::var("BROKER_TYPE")
            .unwrap_or_else(|_| "nats".to_string());

        match broker_type.to_lowercase().as_str() {
            #[cfg(feature = "nats")]
            "nats" => {
                let url = std::env::var("NATS_URL")
                    .unwrap_or_else(|_| "nats://localhost:4222".to_string());
                let stream_name = std::env::var("NATS_STREAM")
                    .unwrap_or_else(|_| "TASKS".to_string());

                Ok(BrokerConfig::Nats(NatsBrokerConfig {
                    url,
                    stream_name,
                    ..Default::default()
                }))
            }

            #[cfg(feature = "pubsub")]
            "pubsub" | "gcp" | "google" => {
                let project_id = std::env::var("PUBSUB_PROJECT_ID").ok()
                    .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok());
                let topic_name = std::env::var("PUBSUB_TOPIC")
                    .unwrap_or_else(|_| "tasks".to_string());
                let subscription_name = std::env::var("PUBSUB_SUBSCRIPTION")
                    .unwrap_or_else(|_| "task-worker".to_string());

                Ok(BrokerConfig::PubSub(PubSubPullConfig {
                    project_id,
                    topic_name,
                    subscription_name,
                    ..Default::default()
                }))
            }

            other => Err(TaskError::Configuration(format!(
                "Unknown broker type: '{}'. Available types: {}",
                other,
                Self::available_types().join(", ")
            ))),
        }
    }

    /// Returns list of available broker types based on compiled features.
    #[allow(clippy::vec_init_then_push)] // Conditional compilation requires this pattern
    pub fn available_types() -> Vec<&'static str> {
        let mut types = vec![];

        #[cfg(feature = "nats")]
        types.push("nats");

        #[cfg(feature = "pubsub")]
        types.push("pubsub");

        types
    }

    /// Get the broker type as a string.
    pub fn broker_type(&self) -> &'static str {
        match self {
            #[cfg(feature = "nats")]
            BrokerConfig::Nats(_) => "nats",

            #[cfg(feature = "pubsub")]
            BrokerConfig::PubSub(_) => "pubsub",
        }
    }

    /// Create a concrete broker from this configuration.
    ///
    /// Returns a `BrokerInstance` enum that wraps the concrete broker type.
    /// Since `PullBroker` is not object-safe (due to generic methods),
    /// we return an enum instead of a trait object.
    #[cfg(any(feature = "nats", feature = "pubsub"))]
    pub fn into_broker(self) -> BrokerInstance {
        match self {
            #[cfg(feature = "nats")]
            BrokerConfig::Nats(config) => {
                BrokerInstance::Nats(Box::new(NatsBroker::new(config)))
            }

            #[cfg(feature = "pubsub")]
            BrokerConfig::PubSub(config) => {
                BrokerInstance::PubSub(Box::new(PubSubPullBroker::new(config)))
            }
        }
    }
}

/// Concrete broker instance.
///
/// Since `PullBroker` is not object-safe (has generic methods),
/// we use an enum to hold concrete broker types.
///
/// Large variants are boxed to reduce total enum size.
#[cfg(any(feature = "nats", feature = "pubsub"))]
pub enum BrokerInstance {
    #[cfg(feature = "nats")]
    Nats(Box<NatsBroker>),

    #[cfg(feature = "pubsub")]
    PubSub(Box<PubSubPullBroker>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_types() {
        let types = BrokerConfig::available_types();
        assert!(!types.is_empty());

        #[cfg(feature = "nats")]
        assert!(types.contains(&"nats"));

        #[cfg(feature = "pubsub")]
        assert!(types.contains(&"pubsub"));
    }

    #[test]
    fn test_broker_type() {
        #[cfg(feature = "nats")]
        {
            let config = BrokerConfig::Nats(NatsBrokerConfig::default());
            assert_eq!(config.broker_type(), "nats");
        }

        #[cfg(feature = "pubsub")]
        {
            let config = BrokerConfig::PubSub(PubSubPullConfig::default());
            assert_eq!(config.broker_type(), "pubsub");
        }
    }

    #[test]
    fn test_from_env_default() {
        // Save and clear BROKER_TYPE to test default
        let original = std::env::var("BROKER_TYPE").ok();
        std::env::remove_var("BROKER_TYPE");

        let config = BrokerConfig::from_env();

        #[cfg(feature = "nats")]
        assert!(config.is_ok(), "Expected successful config with default NATS broker");

        // Restore original value
        if let Some(val) = original {
            std::env::set_var("BROKER_TYPE", val);
        }
    }


    #[test]
    fn test_from_env_nats() {
        #[cfg(feature = "nats")]
        {
            // Save original values
            let original_type = std::env::var("BROKER_TYPE").ok();
            let original_url = std::env::var("NATS_URL").ok();
            let original_stream = std::env::var("NATS_STREAM").ok();

            std::env::set_var("BROKER_TYPE", "nats");
            std::env::set_var("NATS_URL", "nats://example.com:4222");
            std::env::set_var("NATS_STREAM", "MY_STREAM");

            let config = BrokerConfig::from_env().unwrap();
            match config {
                BrokerConfig::Nats(nats_config) => {
                    assert_eq!(nats_config.url, "nats://example.com:4222");
                    assert_eq!(nats_config.stream_name, "MY_STREAM");
                }
                #[cfg(feature = "pubsub")]
                _ => panic!("Expected NATS config"),
            }

            // Restore original values
            match original_type {
                Some(val) => std::env::set_var("BROKER_TYPE", val),
                None => std::env::remove_var("BROKER_TYPE"),
            }
            match original_url {
                Some(val) => std::env::set_var("NATS_URL", val),
                None => std::env::remove_var("NATS_URL"),
            }
            match original_stream {
                Some(val) => std::env::set_var("NATS_STREAM", val),
                None => std::env::remove_var("NATS_STREAM"),
            }
        }
    }

    #[test]
    fn test_from_env_pubsub() {
        #[cfg(feature = "pubsub")]
        {
            // Save original values
            let original_type = std::env::var("BROKER_TYPE").ok();
            let original_project = std::env::var("PUBSUB_PROJECT_ID").ok();
            let original_topic = std::env::var("PUBSUB_TOPIC").ok();
            let original_sub = std::env::var("PUBSUB_SUBSCRIPTION").ok();

            std::env::set_var("BROKER_TYPE", "pubsub");
            std::env::set_var("PUBSUB_PROJECT_ID", "my-project");
            std::env::set_var("PUBSUB_TOPIC", "my-topic");
            std::env::set_var("PUBSUB_SUBSCRIPTION", "my-sub");

            let config = BrokerConfig::from_env().unwrap();
            match config {
                BrokerConfig::PubSub(pubsub_config) => {
                    assert_eq!(pubsub_config.project_id, Some("my-project".to_string()));
                    assert_eq!(pubsub_config.topic_name, "my-topic");
                    assert_eq!(pubsub_config.subscription_name, "my-sub");
                }
                #[cfg(feature = "nats")]
                _ => panic!("Expected Pub/Sub config"),
            }

            // Restore original values
            match original_type {
                Some(val) => std::env::set_var("BROKER_TYPE", val),
                None => std::env::remove_var("BROKER_TYPE"),
            }
            match original_project {
                Some(val) => std::env::set_var("PUBSUB_PROJECT_ID", val),
                None => std::env::remove_var("PUBSUB_PROJECT_ID"),
            }
            match original_topic {
                Some(val) => std::env::set_var("PUBSUB_TOPIC", val),
                None => std::env::remove_var("PUBSUB_TOPIC"),
            }
            match original_sub {
                Some(val) => std::env::set_var("PUBSUB_SUBSCRIPTION", val),
                None => std::env::remove_var("PUBSUB_SUBSCRIPTION"),
            }
        }
    }

    #[test]
    fn test_from_env_unknown_type() {
        // Save original value
        let original = std::env::var("BROKER_TYPE").ok();

        std::env::set_var("BROKER_TYPE", "unknown");

        let result = BrokerConfig::from_env();
        assert!(result.is_err());

        if let Err(TaskError::Configuration(msg)) = result {
            assert!(msg.contains("Unknown broker type"));
            assert!(msg.contains("unknown"));
        } else {
            panic!("Expected Configuration error");
        }

        // Restore original value
        match original {
            Some(val) => std::env::set_var("BROKER_TYPE", val),
            None => std::env::remove_var("BROKER_TYPE"),
        }
    }
}

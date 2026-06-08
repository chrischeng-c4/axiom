//! Delayed task scheduler
//!
//! Polls scheduled messages and republishes them when ETA is reached.

#[cfg(feature = "nats")]
use std::sync::Arc;
#[cfg(feature = "nats")]
use std::time::Duration;
#[cfg(feature = "nats")]
use chrono::{DateTime, Utc};
#[cfg(feature = "nats")]
use tokio_util::sync::CancellationToken;

#[cfg(feature = "nats")]
use crate::{BrokerMessage, MessageHandler, NatsBroker, PullBroker, TaskError};
#[cfg(feature = "nats")]
use crate::Broker;

/// Configuration for delayed task scheduler
#[cfg(feature = "nats")]
#[derive(Debug, Clone)]
pub struct DelayedTaskConfig {
    /// Poll interval for checking scheduled tasks
    pub poll_interval: Duration,
    /// Batch size for fetching scheduled messages
    pub batch_size: usize,
}

#[cfg(feature = "nats")]
impl Default for DelayedTaskConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
            batch_size: 100,
        }
    }
}

/// Scheduler for delayed tasks
///
/// Polls the `tasks.scheduled.*` subjects and republishes
/// messages to their target queues when ETA is reached.
#[cfg(feature = "nats")]
pub struct DelayedTaskScheduler {
    #[allow(dead_code)]
    config: DelayedTaskConfig,
    broker: Arc<NatsBroker>,
    shutdown: CancellationToken,
}

#[cfg(feature = "nats")]
impl DelayedTaskScheduler {
    /// Create a new delayed task scheduler
    pub fn new(broker: Arc<NatsBroker>, config: DelayedTaskConfig) -> Self {
        Self {
            config,
            broker,
            shutdown: CancellationToken::new(),
        }
    }

    /// Start the scheduler (spawns background task)
    pub async fn start(&self) -> Result<(), TaskError> {
        let broker = self.broker.clone();
        let shutdown = self.shutdown.clone();

        // Create handler for scheduled messages
        struct ScheduledMessageHandler {
            broker: Arc<NatsBroker>,
        }

        #[async_trait::async_trait]
        impl MessageHandler for ScheduledMessageHandler {
            async fn handle(&self, message: BrokerMessage) -> Result<(), TaskError> {
                // Check if ETA header exists and if it's time to execute
                if let Some(eta_str) = message.headers.get("eta") {
                    let eta = DateTime::parse_from_rfc3339(eta_str)
                        .map_err(|e| TaskError::Internal(format!("Invalid ETA: {}", e)))?
                        .with_timezone(&Utc);

                    if eta <= Utc::now() {
                        // ETA reached, republish to target queue
                        if let Some(target_queue) = message.headers.get("target-queue") {
                            tracing::debug!(
                                "Republishing scheduled task {} to queue {}",
                                message.payload.id,
                                target_queue
                            );

                            self.broker.publish(target_queue, message.payload.clone()).await?;
                        } else {
                            tracing::error!("Scheduled message missing target-queue header");
                            return Err(TaskError::Internal("Missing target-queue header".to_string()));
                        }
                    } else {
                        // Not ready yet, nack to requeue
                        tracing::trace!("Task {} not ready yet (ETA: {})", message.payload.id, eta);
                        return Err(TaskError::Internal("Not ready yet".to_string()));
                    }
                } else {
                    tracing::error!("Scheduled message missing ETA header");
                    return Err(TaskError::Internal("Missing ETA header".to_string()));
                }

                Ok(())
            }
        }

        let handler = Arc::new(ScheduledMessageHandler {
            broker: broker.clone(),
        });

        // Subscribe to all scheduled queues
        let subscription = broker.subscribe("scheduled.*", handler).await?;

        // Spawn monitoring task
        tokio::spawn(async move {
            shutdown.cancelled().await;
            tracing::info!("Delayed task scheduler shutting down");
            subscription.cancel();
        });

        Ok(())
    }

    /// Shutdown the scheduler
    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }
}

#[cfg(all(test, feature = "nats"))]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // DelayedTaskConfig tests (T1–T7)
    // -----------------------------------------------------------------------

    /// T1: Default config has poll_interval==1s and batch_size==100
    #[test]
    fn config_defaults() {
        let config = DelayedTaskConfig::default();
        assert_eq!(config.poll_interval, Duration::from_secs(1));
        assert_eq!(config.batch_size, 100);
    }

    /// T2: Custom field values are preserved after construction
    #[test]
    fn config_custom_values() {
        let config = DelayedTaskConfig {
            poll_interval: Duration::from_millis(500),
            batch_size: 42,
        };
        assert_eq!(config.poll_interval, Duration::from_millis(500));
        assert_eq!(config.batch_size, 42);
    }

    /// T3: Debug impl includes "poll_interval"
    #[test]
    fn config_debug_impl() {
        let config = DelayedTaskConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(
            debug_str.contains("poll_interval"),
            "Debug output should contain 'poll_interval', got: {}",
            debug_str
        );
        assert!(
            debug_str.contains("batch_size"),
            "Debug output should contain 'batch_size', got: {}",
            debug_str
        );
    }

    /// T4: Clone produces an identical copy
    #[test]
    fn config_clone() {
        let config = DelayedTaskConfig {
            poll_interval: Duration::from_secs(5),
            batch_size: 200,
        };
        let cloned = config.clone();
        assert_eq!(cloned.poll_interval, config.poll_interval);
        assert_eq!(cloned.batch_size, config.batch_size);
    }

    /// T5: batch_size of 0 is constructible (no validation in constructor)
    #[test]
    fn config_zero_batch_size() {
        let config = DelayedTaskConfig {
            batch_size: 0,
            ..Default::default()
        };
        assert_eq!(config.batch_size, 0);
    }

    /// T6: Duration::ZERO is constructible for poll_interval
    #[test]
    fn config_zero_poll_interval() {
        let config = DelayedTaskConfig {
            poll_interval: Duration::ZERO,
            ..Default::default()
        };
        assert_eq!(config.poll_interval, Duration::ZERO);
    }

    /// T7: usize::MAX is constructible for batch_size
    #[test]
    fn config_large_batch_size() {
        let config = DelayedTaskConfig {
            batch_size: usize::MAX,
            ..Default::default()
        };
        assert_eq!(config.batch_size, usize::MAX);
    }

    // -----------------------------------------------------------------------
    // DelayedTaskScheduler construction tests (T8–T11)
    //
    // NatsBroker::new() does NOT require a live connection (it initializes
    // internal fields to None), so we can construct DelayedTaskScheduler
    // in unit tests without a running NATS server.
    // -----------------------------------------------------------------------

    /// Helper: build a scheduler with default config and an unconnected broker.
    fn make_scheduler() -> DelayedTaskScheduler {
        let broker = Arc::new(NatsBroker::new(Default::default()));
        DelayedTaskScheduler::new(broker, DelayedTaskConfig::default())
    }

    /// T8: Scheduler is constructible with default config
    #[test]
    fn scheduler_new_with_default_config() {
        let _scheduler = make_scheduler();
        // Construction succeeds — no panic
    }

    /// T9: Scheduler is constructible with custom config
    #[test]
    fn scheduler_new_with_custom_config() {
        let broker = Arc::new(NatsBroker::new(Default::default()));
        let config = DelayedTaskConfig {
            poll_interval: Duration::from_millis(250),
            batch_size: 10,
        };
        let _scheduler = DelayedTaskScheduler::new(broker, config);
    }

    /// T10: shutdown() can be called immediately without prior start()
    #[test]
    fn scheduler_shutdown_without_start() {
        let scheduler = make_scheduler();
        // Should not panic — CancellationToken::cancel() is idempotent
        scheduler.shutdown();
    }

    /// T11: shutdown() is idempotent (can be called multiple times)
    #[test]
    fn scheduler_shutdown_idempotent() {
        let scheduler = make_scheduler();
        scheduler.shutdown();
        scheduler.shutdown();
        // No panic on repeated cancel
    }

    // -----------------------------------------------------------------------
    // Handler logic tests (T12–T21): SKIPPED
    //
    // The message handler (`ScheduledMessageHandler`) is defined as a local
    // struct inside `DelayedTaskScheduler::start()`. It is not accessible
    // from outside that function, so it cannot be unit-tested in isolation.
    //
    // To enable unit testing of the handler logic, it would need to be
    // extracted into a standalone (pub(crate) or pub) struct or free
    // function. Until that refactoring occurs, handler-level tests
    // require integration tests with a live NATS broker.
    // -----------------------------------------------------------------------
}

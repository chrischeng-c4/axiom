//! cclab-meteor: High-performance distributed task queue
//!
//! A Rust-native replacement for Celery.

pub mod error;
pub mod handoff;
pub mod message;
pub mod ratelimit;
pub mod result;
pub mod retry;
pub mod revocation;
pub mod routing;
pub mod signals;
pub mod state;
pub mod task;

pub mod backend;
pub mod broker;
pub mod executor;
pub mod scheduler;
pub mod worker;
pub mod workflow;

// Optional: Metrics and tracing
#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "tracing-otel")]
pub mod tracing_support;

// Optional: JSON Schema generation
#[cfg(feature = "schema")]
pub mod schema;

// Re-exports
pub use error::TaskError;
pub use handoff::{
    parse_http_background_handoffs, publish_http_background_handoffs,
    publish_http_background_handoffs_blocking, publish_routed_handoffs,
    publish_routed_handoffs_blocking, route_http_background_handoffs, PublishedTaskMessage,
    RoutedTaskMessage, TaskHandoff,
};
pub use message::TaskMessage;
pub use ratelimit::{
    RateLimitConfig, RateLimitManager, RateLimitResult, RateLimiter, SlidingWindow, TokenBucket,
};
pub use result::AsyncResult;
pub use retry::RetryPolicy;
pub use revocation::{
    revoke, revoke_by_name, InMemoryRevocationStore, RevocationStore, RevokeRequest, RevokedTask,
};
pub use routing::{PatternType, Route, Router, RouterConfig, RoutesConfig};
pub use signals::{ShutdownReason, Signal, SignalDispatcher, SignalHandler};
pub use state::{TaskResult, TaskState};
pub use task::{Task, TaskContext, TaskId, TaskOutcome, TaskRegistry};

#[cfg(feature = "redis")]
pub use revocation::RedisRevocationStore;

// Broker re-exports
pub use broker::{
    Broker, BrokerCapabilities, BrokerConfig, BrokerMessage, DelayedBroker, DeliveryModel,
    MessageHandler, PullBroker, PushBroker, SubscriptionHandle,
};

#[cfg(any(feature = "nats", feature = "pubsub"))]
pub use broker::BrokerInstance;

#[cfg(feature = "nats")]
pub use broker::{NatsBroker, NatsBrokerConfig};

#[cfg(feature = "pubsub")]
pub use broker::{PubSubPullBroker, PubSubPullConfig};

#[cfg(feature = "pubsub-push")]
pub use broker::{PubSubPushBroker, PubSubPushConfig};

#[cfg(feature = "cloudtasks")]
pub use broker::{CloudTasksBroker, CloudTasksConfig};

// Backend re-exports
pub use backend::ResultBackend;

#[cfg(feature = "redis")]
pub use backend::{RedisBackend, RedisBackendConfig};

#[cfg(feature = "ion")]
pub use backend::{IonBackend, IonBackendConfig};

// Worker re-exports
pub use worker::{Worker, WorkerConfig};

// Scheduler re-exports
pub use scheduler::{MemorySchedulerBackend, SchedulerBackend, TaskScheduleState};

#[cfg(all(feature = "scheduler", feature = "nats"))]
pub use scheduler::{DelayedTaskConfig, DelayedTaskScheduler};

#[cfg(feature = "scheduler")]
pub use scheduler::{IonSchedulerBackend, PeriodicSchedule, PeriodicScheduler, PeriodicTask};

#[cfg(feature = "cloud-scheduler")]
pub use scheduler::{CloudSchedulerBackend, CloudSchedulerConfig};

pub use scheduler::periodic::PeriodicSchedulerConfig;

#[cfg(feature = "push-receiver")]
pub use scheduler::{PushReceiver, PushReceiverConfig};

// Workflow re-exports
pub use workflow::{
    chunks, starmap, xmap, AsyncChainResult, AsyncChordResult, Chain, ChainMeta, Chord, ChordMeta,
    Chunks, Group, GroupResult, Map, Starmap, TaskOptions, TaskSignature, WorkflowEngine,
};

// K8s executor re-exports
#[cfg(feature = "k8s")]
pub use executor::{K8sJobExecutor, K8sJobExecutorConfig, OffloadedJobInfo};

// Metrics re-exports
#[cfg(feature = "metrics")]
pub use metrics::{gather_metrics, TaskMetrics, METRICS};

// Tracing re-exports
#[cfg(feature = "tracing-otel")]
pub use tracing_support::{create_task_span, init_tracing, shutdown_tracing, TaskSpanAttributes};

/// Result type for task operations
pub type Result<T> = std::result::Result<T, TaskError>;

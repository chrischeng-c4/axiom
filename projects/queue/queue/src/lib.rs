//! cclab-meteor: High-performance distributed task queue
//!
//! A Rust-native replacement for Celery with PyO3 bindings.

pub mod error;
pub mod state;
pub mod retry;
pub mod message;
pub mod task;
pub mod routing;
pub mod ratelimit;
pub mod signals;
pub mod revocation;

pub mod broker;
pub mod backend;
pub mod worker;
pub mod scheduler;
pub mod workflow;
pub mod executor;

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
pub use state::{TaskState, TaskResult};
pub use retry::RetryPolicy;
pub use message::TaskMessage;
pub use task::{Task, TaskId, TaskContext, TaskOutcome, TaskRegistry};
pub use routing::{Router, RouterConfig, Route, PatternType, RoutesConfig};
pub use ratelimit::{
    RateLimiter, RateLimitConfig, RateLimitResult, RateLimitManager,
    TokenBucket, SlidingWindow,
};
pub use signals::{Signal, SignalHandler, SignalDispatcher, ShutdownReason};
pub use revocation::{RevocationStore, InMemoryRevocationStore, RevokedTask, RevokeRequest, revoke, revoke_by_name};

#[cfg(feature = "redis")]
pub use revocation::RedisRevocationStore;

// Broker re-exports
pub use broker::{
    Broker, DeliveryModel, BrokerCapabilities, PullBroker, PushBroker, DelayedBroker,
    BrokerMessage, MessageHandler, SubscriptionHandle, BrokerConfig,
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
    TaskSignature, TaskOptions,
    Chain, AsyncChainResult,
    Group, GroupResult,
    Chord, AsyncChordResult,
    ChainMeta, ChordMeta,
    Map, Starmap, Chunks,
    xmap, starmap, chunks,
    WorkflowEngine,
};

// K8s executor re-exports
#[cfg(feature = "k8s")]
pub use executor::{K8sJobExecutor, K8sJobExecutorConfig, OffloadedJobInfo};

// Metrics re-exports
#[cfg(feature = "metrics")]
pub use metrics::{TaskMetrics, METRICS, gather_metrics};

// Tracing re-exports
#[cfg(feature = "tracing-otel")]
pub use tracing_support::{TaskSpanAttributes, init_tracing, shutdown_tracing, create_task_span};

/// Result type for task operations
pub type Result<T> = std::result::Result<T, TaskError>;

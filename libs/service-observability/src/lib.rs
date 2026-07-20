// HANDWRITE-BEGIN gap="missing-generator:logic:37a91ad9" tracker="1868" reason="Export typed configuration, tracing, metric provider, and lifecycle metrics."
//! Protocol-neutral service observability composition.
//!
//! This crate owns configuration, stable identity, structured logging,
//! optional OTLP export, metric-provider semantics, and lifecycle connection
//! counters. It also exposes safe, portable process-resource samples for
//! service evidence. Protocol adapters such as HTTP remain in their protocol
//! crates.

pub mod config;
pub mod jsonl;
pub mod logging;
pub mod metrics;
pub mod process;

pub use config::{LogFormat, ObservabilityConfig, ServiceIdentity};
pub use jsonl::{
    collector_compatible, service_log_schema_v1, ServiceJsonFormatter, ServiceLogEventV1,
    ServiceLogIdentityV1, MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_BYTES, MAX_ATTRIBUTE_VALUE_BYTES,
    MAX_EVENT_BYTES, MAX_REQUEST_ID_BYTES, SERVICE_LOG_SCHEMA_V1,
};
#[cfg(feature = "otlp")]
pub use logging::extract_trace_context;
pub use logging::{
    init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode,
};
pub use metrics::{LifecycleMetrics, MetricsProvider};
pub use process::{process_usage, ProcessUsage};
// HANDWRITE-END

// HANDWRITE-BEGIN gap="missing-generator:logic:37a91ad9" tracker="pending-tracker" reason="Export typed configuration, tracing, metric provider, and lifecycle metrics."
//! Protocol-neutral service observability composition.
//!
//! This crate owns configuration, stable identity, structured logging,
//! optional OTLP export, metric-provider semantics, and lifecycle connection
//! counters. Protocol adapters such as HTTP remain in their protocol crates.

pub mod config;
pub mod logging;
pub mod metrics;

pub use config::{LogFormat, ObservabilityConfig, ServiceIdentity};
#[cfg(feature = "otlp")]
pub use logging::extract_trace_context;
pub use logging::{
    init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode,
};
pub use metrics::{LifecycleMetrics, MetricsProvider};
// HANDWRITE-END

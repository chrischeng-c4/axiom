// CODEGEN-BEGIN
//! Compatibility adapter to protocol-neutral `service-observability`.

use crate::config::HttpConfig;

pub use service_observability::{OtelFallback, TracingMode};

/// Resolve the shared trace mode from HTTP configuration.
pub fn tracing_mode(
    config: &HttpConfig,
    identity: &service_observability::ServiceIdentity,
) -> TracingMode {
    service_observability::tracing_mode(&config.observability_config(), identity)
}

/// Install tracing using the compatibility default identity.
pub fn init_tracing(config: &HttpConfig) -> anyhow::Result<()> {
    service_observability::init_tracing(&config.observability_config())
}

/// Install tracing with application-owned stable identity.
pub fn init_tracing_with_identity(
    config: &HttpConfig,
    identity: &service_observability::ServiceIdentity,
) -> anyhow::Result<()> {
    service_observability::init_tracing_with_identity(&config.observability_config(), identity)
}

#[cfg(feature = "otlp")]
pub use service_observability::extract_trace_context;
// CODEGEN-END

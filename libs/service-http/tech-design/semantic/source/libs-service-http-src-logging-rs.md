---
id: libs-service-http-src-logging-rs
summary: Source unit for the service-http observability compatibility adapter.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "HTTP callers retain their existing API while service-observability owns implementation."
fill_sections: [overview, source, changes]
---

# service-http observability compatibility adapter

## Overview
<!-- type: overview lang: markdown -->

`service-http` projects `HttpConfig` into the protocol-neutral
`ObservabilityConfig` and delegates all logging and exporter decisions to
`service-observability`.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/logging.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: "Delegates protocol-neutral observability implementation while preserving the service-http API."
```

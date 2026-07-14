---
id: libs-service-http-src-logging-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/logging.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/logging.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/logging.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TracingMode` | libs/service-http/src/logging.rs | enum | pub | 14 | pub enum TracingMode { |
| `OtelFallback` | libs/service-http/src/logging.rs | enum | pub | 29 | pub enum OtelFallback { |
| `tracing_mode` | libs/service-http/src/logging.rs | function | pub | 38 | pub fn tracing_mode(cfg: &HttpConfig, identity: &ServiceIdentity) -> TracingMode { |
| `init_tracing` | libs/service-http/src/logging.rs | function | pub | 79 | pub fn init_tracing(cfg: &HttpConfig) -> anyhow::Result<()> { |
| `init_tracing_with_identity` | libs/service-http/src/logging.rs | function | pub | 86 | pub fn init_tracing_with_identity(cfg: &HttpConfig, identity: &ServiceIdentity) -> anyhow::Result<()> { |
| `extract_trace_context` | libs/service-http/src/logging.rs | function | pub | 162 | pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> opentelemetry::Context { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Tracing init: one shared `tracing-subscriber` registry built from
//! [`HttpConfig`].
//!
//! `RUST_LOG` wins; otherwise the filter falls back to `cfg.log_level`. The fmt
//! layer is `pretty` or `json` per `cfg.log_format`. This is the prefix-agnostic
//! version of the `init_tracing` each service binary hand-rolls today (lumen's
//! `init_tracing`, keep's inline `fmt().with_env_filter(...)`).

use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::config::{HttpConfig, LogFormat, ServiceIdentity};

/// The trace-export outcome selected from service configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TracingMode {
    /// No exporter was requested; install the standard formatter only.
    LoggingOnly,
    /// Export using this endpoint and service identity.
    Otel {
        endpoint: String,
        identity: ServiceIdentity,
    },
    /// Export was requested but cannot be initialized; keep logging available.
    OtelUnavailable {
        endpoint: String,
        reason: OtelFallback,
    },
}

/// A redacted reason for falling back to logging-only tracing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtelFallback {
    /// The binary was built without the optional exporter feature.
    FeatureDisabled,
    /// The endpoint is not an absolute HTTP(S) URI with an authority.
    InvalidEndpoint,
}

/// Resolve the non-global trace-export mode before a service installs its one
/// global subscriber. Keeping this pure makes the fallback contract testable.
pub fn tracing_mode(cfg: &HttpConfig, identity: &ServiceIdentity) -> TracingMode {
    let Some(endpoint) = cfg.otlp_endpoint.as_deref() else {
        return TracingMode::LoggingOnly;
    };
    if !valid_otlp_endpoint(endpoint) {
        return TracingMode::OtelUnavailable {
            endpoint: endpoint.to_string(),
            reason: OtelFallback::InvalidEndpoint,
        };
    }

    #[cfg(feature = "otlp")]
    {
        TracingMode::Otel {
            endpoint: endpoint.to_string(),
            identity: identity.clone(),
        }
    }
    #[cfg(not(feature = "otlp"))]
    {
        let _ = identity;
        TracingMode::OtelUnavailable {
            endpoint: endpoint.to_string(),
            reason: OtelFallback::FeatureDisabled,
        }
    }
}

fn valid_otlp_endpoint(endpoint: &str) -> bool {
    let Ok(uri) = endpoint.parse::<axum::http::Uri>() else {
        return false;
    };
    matches!(uri.scheme_str(), Some("http") | Some("https")) && uri.authority().is_some()
}

/// Install the global tracing subscriber from `cfg`.
///
/// Filter precedence: `RUST_LOG` (via `EnvFilter::try_from_default_env`) →
/// otherwise `cfg.log_level`. The fmt layer is JSON or pretty per
/// `cfg.log_format`. OTLP is opt-in through the `otlp` feature and an endpoint;
/// malformed configuration retains the formatter and emits only a redacted
/// fallback reason.
pub fn init_tracing(cfg: &HttpConfig) -> anyhow::Result<()> {
    let identity = ServiceIdentity::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;
    init_tracing_with_identity(cfg, &identity)
}

/// Install the global tracing subscriber using application-owned stable
/// identity for optional OTLP resource attributes.
pub fn init_tracing_with_identity(
    cfg: &HttpConfig,
    identity: &ServiceIdentity,
) -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(cfg.log_level.clone()));
    let fmt_layer = match cfg.log_format {
        LogFormat::Pretty => tracing_subscriber::fmt::layer().boxed(),
        LogFormat::Json => tracing_subscriber::fmt::layer().json().boxed(),
    };

    match tracing_mode(cfg, identity) {
        TracingMode::LoggingOnly => tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}")),
        TracingMode::OtelUnavailable { reason, .. } => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}"))?;
            tracing::warn!(reason = ?reason, "OTLP tracing unavailable; emitting structured logs only");
            Ok(())
        }
        #[cfg(feature = "otlp")]
        TracingMode::Otel { endpoint, identity } => match build_otel_tracer(&endpoint, &identity) {
            Ok(tracer) => {
                install_trace_context_propagator();
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .with(tracing_opentelemetry::layer().with_tracer(tracer))
                    .try_init()
                    .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}"))
            }
            Err(_error) => {
                tracing_subscriber::registry()
                    .with(filter)
                    .with(fmt_layer)
                    .try_init()
                    .map_err(|e| anyhow::anyhow!("install tracing subscriber: {e}"))?;
                tracing::warn!(reason = "exporter-construction", "OTLP tracer construction failed; emitting structured logs only");
                Ok(())
            }
        },
        #[cfg(not(feature = "otlp"))]
        TracingMode::Otel { .. } => unreachable!("OTLP mode requires the otlp feature"),
    }
}

#[cfg(feature = "otlp")]
fn build_otel_tracer(
    endpoint: &str,
    identity: &ServiceIdentity,
) -> std::result::Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint.to_string());
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new("service.name", identity.name().to_string()),
                opentelemetry::KeyValue::new("service.version", identity.version().to_string()),
            ]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(Into::into)
}

#[cfg(feature = "otlp")]
fn install_trace_context_propagator() {
    use std::sync::Once;
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );
    });
}

/// Extract W3C trace context from HTTP request headers. Invalid or absent
/// headers yield an empty context, which safely creates a root request span.
#[cfg(feature = "otlp")]
pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> opentelemetry::Context {
    use opentelemetry::propagation::Extractor;
    install_trace_context_propagator();

    struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);
    impl Extractor for HeaderExtractor<'_> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|value| value.to_str().ok())
        }

        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(axum::http::HeaderName::as_str).collect()
        }
    }

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(headers))
    })
}
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
    description: |
      Provides an optional shared OTLP exporter with stable resource attributes,
      redacted logging-only fallback, and W3C context extraction.
```

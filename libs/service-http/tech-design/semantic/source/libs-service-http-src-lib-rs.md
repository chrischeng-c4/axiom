---
id: libs-service-http-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/lib.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `config` | libs/service-http/src/lib.rs | module | pub | 74 | pub mod config; |
| `error` | libs/service-http/src/lib.rs | module | pub | 75 | pub mod error; |
| `logging` | libs/service-http/src/lib.rs | module | pub | 76 | pub mod logging; |
| `metrics` | libs/service-http/src/lib.rs | module | pub | 77 | pub mod metrics; |
| `probes` | libs/service-http/src/lib.rs | module | pub | 78 | pub mod probes; |
| `readiness` | libs/service-http/src/lib.rs | module | pub | 79 | pub mod readiness; |
| `signal` | libs/service-http/src/lib.rs | module | pub | 80 | pub mod signal; |
| `transport` | libs/service-http/src/lib.rs | module | pub | 81 | pub mod transport; |
| `HttpConfig` | libs/service-http/src/lib.rs | re-export | pub | 83 | pub use config::{HttpConfig, LogFormat, ServiceIdentity}; |
| `LogFormat` | libs/service-http/src/lib.rs | re-export | pub | 83 | pub use config::{HttpConfig, LogFormat, ServiceIdentity}; |
| `ServiceIdentity` | libs/service-http/src/lib.rs | re-export | pub | 83 | pub use config::{HttpConfig, LogFormat, ServiceIdentity}; |
| `ApiErr` | libs/service-http/src/lib.rs | re-export | pub | 84 | pub use error::{ApiErr, ErrorEnvelope}; |
| `ErrorEnvelope` | libs/service-http/src/lib.rs | re-export | pub | 84 | pub use error::{ApiErr, ErrorEnvelope}; |
| `extract_trace_context` | libs/service-http/src/lib.rs | re-export | pub | 86 | pub use logging::extract_trace_context; |
| `init_tracing` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use logging::{init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode}; |
| `init_tracing_with_identity` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use logging::{init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode}; |
| `tracing_mode` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use logging::{init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode}; |
| `OtelFallback` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use logging::{init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode}; |
| `TracingMode` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use logging::{init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode}; |
| `MetricsProvider` | libs/service-http/src/lib.rs | re-export | pub | 86 | pub use metrics::MetricsProvider; |
| `standard_probe_routes` | libs/service-http/src/lib.rs | re-export | pub | 87 | pub use probes::standard_probe_routes; |
| `ReadinessHook` | libs/service-http/src/lib.rs | re-export | pub | 88 | pub use readiness::ReadinessHook; |
| `shutdown_with_drain` | libs/service-http/src/lib.rs | re-export | pub | 89 | pub use signal::{shutdown_with_drain, wait_shutdown_signal}; |
| `wait_shutdown_signal` | libs/service-http/src/lib.rs | re-export | pub | 89 | pub use signal::{shutdown_with_drain, wait_shutdown_signal}; |
| `serve` | libs/service-http/src/lib.rs | re-export | pub | 92 | pub use transport::{serve, trace_layer, PropagatingMakeSpan}; |
| `trace_layer` | libs/service-http/src/lib.rs | re-export | pub | 92 | pub use transport::{serve, trace_layer, PropagatingMakeSpan}; |
| `PropagatingMakeSpan` | libs/service-http/src/lib.rs | re-export | pub | 92 | pub use transport::{serve, trace_layer, PropagatingMakeSpan}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `service-http` — shared HTTP-service scaffolding for the ecosystem's
//! k8s-native services.
//!
//! lumen, keep, relay, and loom compose the same HTTP policy shell: the
//! standard probe/admin endpoints (`/healthz` `/readyz` `/metrics`
//! `/openapi.json` `/docs`), observability compatibility adapters, lifecycle
//! readiness/shutdown re-exports, runtime delegation, and the
//! `{"error", "message"}` HTTP error envelope
//! ([`error`]) each service renders for its error responses. This crate is
//! the one place that HTTP shape lives. Protocol-neutral logging, tracing,
//! metric-provider, and lifecycle metric ownership belongs to
//! `service-observability`. This crate operationalizes the CONTRIBUTING "standard
//! endpoints" convention: every service exposes the same probe surface,
//! with the same auth-exempt / no-body-limit treatment.
//!
//! ## Composition
//!
//! It composes, it does not replace: [`transport::serve`] delegates listener
//! ownership to `server-http`; [`probes::standard_probe_routes`] returns an `axum::Router`
//! a service `.merge`s its own (auth'd, body-limited) data plane onto.
//!
//! ## What a service wires
//!
//! ```ignore
//! use std::sync::Arc;
//! use std::time::Duration;
//! use service_http::{
//!     HttpConfig, LogFormat, MetricsProvider, ReadinessHook,
//!     init_tracing, serve, shutdown_with_drain, standard_probe_routes, trace_layer,
//! };
//!
//! # async fn run(cfg: HttpConfig, readiness: Arc<R>, data_plane: axum::Router) -> anyhow::Result<()>
//! # where R: ReadinessHook + 'static {
//! init_tracing(&cfg)?;
//!
//! let app = standard_probe_routes(readiness.clone(), None, my_service::openapi)
//!     .merge(data_plane)
//!     .layer(trace_layer());
//!
//! let listener = tokio::net::TcpListener::bind(cfg.bind_addr()).await?;
//! let grace = Duration::from_secs(cfg.grace_secs);
//! serve(
//!     listener,
//!     app,
//!     shutdown_with_drain(move || readiness.start_drain(), grace),
//! )
//! .await;
//! # Ok(()) }
//! # fn openapi() -> utoipa::openapi::OpenApi { unimplemented!() }
//! ```
//!
//! ## Scope
//!
//! Auth and backup are deliberately out of scope (separate follow-ups); a
//! service keeps owning those on its data plane. OTLP trace export is optional
//! behind the `otlp` feature; a service supplies its own stable identity.
//!
//! [`error::ErrorEnvelope`]'s derived `utoipa::ToSchema` is named
//! `ErrorEnvelope` in a service's generated OpenAPI document. A service that
//! already published a different schema name for this shape (e.g. lumen's
//! established `ApiError`) keeps its own local doc-only struct of the same
//! `{error, message}` shape for that name/description rather than aliasing
//! this one — `#[derive(ToSchema)]` fixes both the schema name and its
//! `description` (sourced from the struct's doc comment) at the type's own
//! definition site, so neither a `pub type` alias nor a `#[schema(as =
//! ...)]` override on this shared struct can reproduce a *different*
//! consuming service's pre-existing name and doc-comment-derived
//! description without baking that service's spec-path text into this
//! generic crate. [`error::ApiErr`] (the runtime status/kind/message
//! wrapper — it carries no `ToSchema`) has no such constraint and is meant
//! to be adopted directly.

pub mod admission;
pub mod config;
pub mod error;
pub mod logging;
pub mod metrics;
pub mod probes;
pub mod readiness;
pub mod signal;
pub mod transport;

pub use admission::{
    admission_middleware, AdmissionController, AdmissionDecision, AdmissionEvent, AdmissionInput,
    AdmissionMiddleware, AdmissionObserver, AdmissionOutcome, AdmissionPolicy,
    AdmissionPolicyError, NoopAdmissionObserver,
};
pub use config::{HttpConfig, LogFormat, ServiceIdentity};
pub use error::{ApiErr, ErrorEnvelope};
#[cfg(feature = "otlp")]
pub use logging::extract_trace_context;
pub use logging::{
    init_tracing, init_tracing_with_identity, tracing_mode, OtelFallback, TracingMode,
};
pub use metrics::MetricsProvider;
pub use probes::{standard_probe_routes, standard_probe_routes_canonical_json};
pub use readiness::ReadinessHook;
pub use service_observability::LifecycleMetrics;
pub use signal::{shutdown_with_drain, wait_shutdown_signal};
pub use transport::{serve, trace_layer, PropagatingMakeSpan};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Re-exports the optional OTLP configuration, initialization and W3C span
      propagation surfaces so Lumen and Tape consume one public contract.
```

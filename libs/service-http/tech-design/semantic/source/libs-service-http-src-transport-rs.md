---
id: libs-service-http-src-transport-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/transport.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/transport.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/transport.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PropagatingMakeSpan` | libs/service-http/src/transport.rs | struct | pub | 13 | pub struct PropagatingMakeSpan; |
| `serve` | libs/service-http/src/transport.rs | function | pub | 42 | pub async fn serve( |
| `trace_layer` | libs/service-http/src/transport.rs | function | pub | 67 | pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, PropagatingMakeSpan> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! HTTP transport: the h2c serve loop + the standard request-tracing layer.
//!
//! [`serve`] composes [`transport_h2c::serve`] (HTTP/1.1 + HTTP/2 cleartext on one port —
//! the in-cluster default `axum::serve` can't do) rather than re-implementing
//! the accept loop. [`trace_layer`] is the one INFO-level span-per-request layer
//! lumen/keep both attach; a service `.layer(...)`s it onto its router.

use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, MakeSpan, TraceLayer};

/// Request span maker that preserves standard request fields and, in an
/// OTLP-enabled build, attaches a valid propagated W3C parent context.
#[derive(Debug, Clone, Copy)]
pub struct PropagatingMakeSpan;

impl<B> MakeSpan<B> for PropagatingMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let mut default = DefaultMakeSpan::new().level(tracing::Level::INFO);
        let span = default.make_span(request);
        #[cfg(feature = "otlp")]
        {
            use opentelemetry::trace::TraceContextExt as _;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let parent = crate::logging::extract_trace_context(request.headers());
            if parent.span().span_context().is_valid() {
                span.set_parent(parent);
            }
        }
        span
    }
}

/// Serve `app` (HTTP/1.1 + h2c on one port) on `listener`, stopping when
/// `shutdown` resolves (e.g. [`crate::signal::shutdown_with_drain`]).
///
/// Thin delegation to [`transport_h2c::serve`] — the shared transport — so a service does
/// not hand-roll the hyper-util auto-builder accept loop. In-flight connections
/// get a bounded grace period after `shutdown` resolves before the process
/// exits.
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()>,
) {
    transport_h2c::serve(listener, app, shutdown).await;
}

/// The standard request-tracing layer: one INFO-level span per HTTP request.
///
/// INFO so the default `info` `EnvFilter` keeps it, and so the spans the OTLP
/// layer (when wired) would export are produced. Attach it to the **outer**
/// router so it spans probe and data-plane requests alike:
///
/// ```ignore
/// let app = service_http::standard_probe_routes(readiness, metrics, openapi)
///     .merge(data_plane)
///     .layer(service_http::trace_layer())
///     .with_state(state);
/// ```
///
/// Returns the concrete `TraceLayer` so callers `.layer()` it directly. For a
/// different classifier/make-span, build `TraceLayer::new_for_http()` inline
/// instead.
pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, PropagatingMakeSpan> {
    TraceLayer::new_for_http().make_span_with(PropagatingMakeSpan)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/transport.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Replaces the default span factory with a shared factory that preserves a
      valid W3C parent when the optional OTLP feature is enabled.
```

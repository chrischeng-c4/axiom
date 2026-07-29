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
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! [`serve`] composes [`server_http::serve_h2c`] (HTTP/1.1 + HTTP/2 cleartext on one port —
//! the in-cluster default `axum::serve` can't do) rather than re-implementing
//! the accept loop. [`trace_layer`] is the one INFO-level span-per-request layer
//! lumen/keep both attach; a service `.layer(...)`s it onto its router.

use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, MakeSpan, TraceLayer};

/// Request span maker that preserves standard request fields and, in an
/// OTLP-enabled build, attaches a valid propagated W3C parent context.
#[derive(Debug, Clone, Copy)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#source
pub struct PropagatingMakeSpan;

impl<B> MakeSpan<B> for PropagatingMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let mut default = DefaultMakeSpan::new().level(tracing::Level::INFO);
        let span = default.make_span(request);
        #[cfg(feature = "otlp")]
        {
            use opentelemetry::trace::TraceContextExt as _;
            use tracing_opentelemetry::OpenTelemetrySpanExt as _;
            let parent = service_observability::extract_trace_context(request.headers());
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
/// Thin delegation to [`server_http::serve_h2c`] — the shared HTTP runtime — so
/// a service does not hand-roll the hyper-util auto-builder accept loop.
/// In-flight connections
/// get a bounded grace period after `shutdown` resolves before the process
/// exits.
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#source
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    server_http::serve_h2c(listener, app, shutdown).await;
}

#[cfg(test)]
mod delegation_tests {
    use super::*;
    use axum::{routing::get, Router};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn serve_delegates_listener_to_shared_http_runtime() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = Router::new().route("/delegated", get(|| async { "shared-runtime" }));

        let server = tokio::spawn(serve(listener, app, async move {
            let _ = shutdown_rx.await;
        }));

        let mut stream = tokio::net::TcpStream::connect(addr)
            .await
            .expect("connect delegated runtime");
        stream
            .write_all(b"GET /delegated HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("write delegated request");
        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("read delegated response");
        let response = String::from_utf8_lossy(&response);
        assert!(
            response.starts_with("HTTP/1.1 200"),
            "service shell must expose the shared runtime listener: {response}"
        );
        assert!(
            response.ends_with("shared-runtime"),
            "service shell must preserve router behavior through delegation: {response}"
        );

        let _ = shutdown_tx.send(());
        tokio::time::timeout(std::time::Duration::from_secs(3), server)
            .await
            .expect("delegated server shutdown")
            .expect("delegated server task");
    }
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
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#source
pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, PropagatingMakeSpan> {
    TraceLayer::new_for_http().make_span_with(PropagatingMakeSpan)
}

/// Request span maker that always records canonical correlation fields and,
/// when an OpenTelemetry layer is installed, attaches the same valid W3C
/// parent context to the exported span. Initializes `subject = "anonymous"` by default,
/// which is recorded by `record_subject_to_span` middleware when authenticated.
#[derive(Debug, Clone, Copy)]
pub struct CorrelatingMakeSpan;

impl<B> MakeSpan<B> for CorrelatingMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let context = request_trace_context(request.headers());
        let span = tracing::span!(
            tracing::Level::INFO,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            trace_id = %context.trace_id(),
            span_id = %context.span_id(),
            parent_span_id = tracing::field::Empty,
            trace_flags = %context.trace_flags(),
            subject = "anonymous",  // Default subject; overridden by auth middleware if authenticated
        );
        // ... OpenTelemetry setup follows ...
        span
    }
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

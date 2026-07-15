// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-transport-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END
// SPEC-MANAGED: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#logic
// CODEGEN-BEGIN
pub fn configure() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?
    if todo!("decision: Is OTLP requested with a valid absolute HTTP(S) endpoint and compiled exporter support?") /* branch */ {
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-logging
        // TODO: Implement process step: Install one RUST_LOG-first pretty or JSON subscriber
        todo!("process: Install one RUST_LOG-first pretty or JSON subscriber");
    } else if todo!("decision branch: {}", "branch") { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-exporter
        // TODO: Implement process step: Attach stable service.name and service.version resources and W3C propagator
        todo!("process: Attach stable service.name and service.version resources and W3C propagator");
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
        // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
        todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
        todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    } else { /* branch */
        // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-fallback
        // TODO: Implement process step: Install logging-only subscriber and emit a redacted fallback reason
        todo!("process: Install logging-only subscriber and emit a redacted fallback reason");
    }
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-provider
    // TODO: Implement process step: MetricsProvider returns canonical Prometheus exposition bytes
    todo!("process: MetricsProvider returns canonical Prometheus exposition bytes");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-connection
    // TODO: Implement process step: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters
    todo!("process: LifecycleMetrics implements ConnectionMetrics using metrics-prometheus counters");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    todo!("terminal: Raw TCP and future protocol runtimes consume service-observability directly");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_config
    // TODO: Implement process step: service-http HttpConfig projects only its observability fields into ObservabilityConfig
    todo!("process: service-http HttpConfig projects only its observability fields into ObservabilityConfig");
    // SPEC-REF: libs/service-http/tech-design/interfaces/rest/extract-protocol-neutral-service-observability-integration.md#shared-service-observability-contract-http_adapter
    // TODO: Implement process step: service-http extracts request headers and serves provider bytes without owning protocol-neutral state
    todo!("process: service-http extracts request headers and serves provider bytes without owning protocol-neutral state");
    todo!("terminal: Existing service-http names remain additive compatibility re-exports");
    // Terminal: compatible -> Existing service-http names remain additive compatibility re-exports
    // Terminal: non_http -> Raw TCP and future protocol runtimes consume service-observability directly
}
// CODEGEN-END

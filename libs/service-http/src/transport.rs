//! HTTP transport: the h2c serve loop + the standard request-tracing layer.
//!
//! [`serve`] composes [`h2c::serve`] (HTTP/1.1 + HTTP/2 cleartext on one port —
//! the in-cluster default `axum::serve` can't do) rather than re-implementing
//! the accept loop. [`trace_layer`] is the one INFO-level span-per-request layer
//! lumen/keep both attach; a service `.layer(...)`s it onto its router.

use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

/// Serve `app` (HTTP/1.1 + h2c on one port) on `listener`, stopping when
/// `shutdown` resolves (e.g. [`crate::signal::shutdown_with_drain`]).
///
/// Thin delegation to [`h2c::serve`] — the shared transport — so a service does
/// not hand-roll the hyper-util auto-builder accept loop. In-flight connections
/// get a bounded grace period after `shutdown` resolves before the process
/// exits.
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()>,
) {
    h2c::serve(listener, app, shutdown).await;
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
pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, DefaultMakeSpan> {
    TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
}

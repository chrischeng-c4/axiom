//! Shared HTTP runtime above `server-core` and `tcp-server`.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! This crate is intentionally below `service-http`: it provides HTTP serving
//! primitives for both production services and tool/dev servers. Service
//! archetype policy such as `/healthz`, `/readyz`, `/metrics`, OpenAPI, and
//! docs remains in `service-http`.

use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub use server_core as core;
pub use tcp_server as tcp;

pub use h2c::ServerOptions as H2cServerOptions;

/// Serve HTTP/1.1 + h2c on one listener.
/// @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
pub async fn serve_h2c(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()>,
) {
    h2c::serve(listener, app, shutdown).await;
}

/// Serve HTTP/1.1 + h2c with tunable HTTP/2 stream and drain settings.
pub async fn serve_h2c_with_options(
    listener: TcpListener,
    app: axum::Router,
    options: H2cServerOptions,
    shutdown: impl std::future::Future<Output = ()>,
) {
    h2c::serve_with_options(listener, app, options, shutdown).await;
}

/// Standard INFO-level request tracing layer for HTTP runtimes.
pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, DefaultMakeSpan> {
    TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
}

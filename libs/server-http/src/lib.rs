// HANDWRITE-BEGIN gap="missing-generator:server-http-runtime" tracker="#1776" reason="Shared HTTP listener composition needs a deterministic Rust generator."
//! Shared HTTP runtime above `server-lifecycle` and `server-tcp`.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! This crate is intentionally below `service-http`: it provides HTTP serving
//! primitives for both production services and tool/dev servers. Service
//! archetype policy such as `/healthz`, `/readyz`, `/metrics`, OpenAPI, and
//! docs remains in `service-http`.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use server_lifecycle::{
    BindConfig, ConnectionBudget, ConnectionMetrics, DrainController, LifecycleController,
    NoopConnectionMetrics,
};
use server_tcp::TcpSocketOptions;
use tokio::net::TcpListener;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub mod tls;

pub use server_lifecycle as core;
pub use server_tcp as tcp;
pub use tls::{
    config_source, serve_tls, ServerConfigSource, TlsListenerMetrics, TlsListenerSnapshot,
    TlsServerOptions,
};

/// HTTP listener/runtime options owned by `server-http`.
#[derive(Clone)]
pub struct HttpServerOptions {
    pub max_concurrent_streams: u32,
    pub drain_timeout: Duration,
    pub connection_budget: Option<ConnectionBudget>,
    pub drain: DrainController,
    pub socket: TcpSocketOptions,
    pub connection_metrics: Arc<dyn ConnectionMetrics>,
}

impl Default for HttpServerOptions {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 4096,
            drain_timeout: Duration::from_secs(5),
            connection_budget: None,
            drain: DrainController::new(),
            socket: TcpSocketOptions::default(),
            connection_metrics: Arc::new(NoopConnectionMetrics),
        }
    }
}

impl fmt::Debug for HttpServerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpServerOptions")
            .field("max_concurrent_streams", &self.max_concurrent_streams)
            .field("drain_timeout", &self.drain_timeout)
            .field("connection_budget", &self.connection_budget)
            .field("drain", &self.drain)
            .field("socket", &self.socket)
            .field("connection_metrics", &"dyn ConnectionMetrics")
            .finish()
    }
}

/// Backwards-compatible name for existing service runtime plans.
pub type H2cServerOptions = HttpServerOptions;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HttpServerReport {
    pub accepted: u64,
    pub rejected: u64,
    pub completed: u64,
    pub failed: u64,
    pub timed_out: u64,
    pub unfinished: u64,
    pub streams_completed: u64,
    pub streams_admitted: u64,
    pub streams_active_at_drain: u64,
    pub streams_refused: u64,
    pub streams_timed_out: u64,
    pub streams_ambiguous: u64,
    pub accept_errors: u64,
    pub deadline_missing: bool,
}

/// Production HTTP composition. The supplied lifecycle owns listener drain,
/// per-connection subscriptions, and the shutdown-time absolute deadline.
pub async fn serve_h2c_with_lifecycle(
    listener: TcpListener,
    app: axum::Router,
    options: HttpServerOptions,
    lifecycle: LifecycleController,
) -> HttpServerReport {
    let local_addr = listener
        .local_addr()
        .unwrap_or_else(|_| BindConfig::default().socket_addr());
    let mut tcp_config = server_tcp::TcpServerConfig::new(BindConfig {
        host: local_addr.ip(),
        port: local_addr.port(),
    })
    .with_socket_options(options.socket)
    .with_drain(DrainController::from_lifecycle(lifecycle.clone()))
    .with_connection_metrics(options.connection_metrics);
    if let Some(budget) = options.connection_budget {
        tcp_config = tcp_config.with_connection_budget(budget);
    }
    let connection_options = transport_h2c::ConnectionOptions {
        max_concurrent_streams: options.max_concurrent_streams,
    };
    let report = server_tcp::serve_with_report(
        listener,
        tcp_config,
        move |stream, cx: server_tcp::ConnectionContext| {
            let app = app.clone();
            async move {
                let connection = transport_h2c::serve_connection_with_lifecycle(
                    stream,
                    app,
                    connection_options,
                    cx.lifecycle,
                )
                .await;
                let terminal = match connection.terminal {
                    transport_h2c::ConnectionTerminal::DeadlineExceeded => {
                        server_tcp::TcpConnectionTerminal::TimedOut
                    }
                    transport_h2c::ConnectionTerminal::Failed => {
                        server_tcp::TcpConnectionTerminal::Failed
                    }
                    transport_h2c::ConnectionTerminal::PeerClosed
                    | transport_h2c::ConnectionTerminal::Drained => {
                        server_tcp::TcpConnectionTerminal::Completed
                    }
                };
                server_tcp::TcpConnectionResult {
                    terminal,
                    streams_admitted: connection.admitted as u64,
                    streams_active_at_drain: connection.active_at_drain as u64,
                    streams_completed: connection.completed as u64,
                    streams_refused: connection.refused as u64,
                    streams_timed_out: connection.timed_out as u64,
                    streams_ambiguous: connection.ambiguous as u64,
                }
            }
        },
        lifecycle,
    )
    .await;
    HttpServerReport {
        accepted: report.accepted,
        rejected: report.rejected,
        completed: report.completed,
        failed: report.failed,
        timed_out: report.timed_out,
        unfinished: report.unfinished,
        streams_completed: report.streams_completed,
        streams_admitted: report.streams_admitted,
        streams_active_at_drain: report.streams_active_at_drain,
        streams_refused: report.streams_refused,
        streams_timed_out: report.streams_timed_out,
        streams_ambiguous: report.streams_ambiguous,
        accept_errors: report.accept_errors,
        deadline_missing: report.deadline_missing,
    }
}

/// Serve HTTP/1.1 + h2c on one listener.
///
/// Legacy/test-dev adapter; production composition should use
/// [`serve_h2c_with_lifecycle`] so one caller-supplied lifecycle owns drain.
/// @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
pub async fn serve_h2c(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    serve_h2c_with_options(listener, app, HttpServerOptions::default(), shutdown).await;
}

/// Serve HTTP/1.1 + h2c with tunable HTTP/2 stream and drain settings.
///
/// Legacy/test-dev adapter retained for source compatibility. Production
/// callers should use [`serve_h2c_with_lifecycle`].
pub async fn serve_h2c_with_options(
    listener: TcpListener,
    app: axum::Router,
    options: HttpServerOptions,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) {
    let local_addr = listener
        .local_addr()
        .unwrap_or_else(|_| BindConfig::default().socket_addr());
    let mut tcp_config = server_tcp::TcpServerConfig::new(BindConfig {
        host: local_addr.ip(),
        port: local_addr.port(),
    })
    .with_socket_options(options.socket)
    .with_drain(options.drain)
    .with_drain_timeout(options.drain_timeout)
    .with_connection_metrics(options.connection_metrics);
    if let Some(budget) = options.connection_budget {
        tcp_config = tcp_config.with_connection_budget(budget);
    }

    let connection_options = transport_h2c::ConnectionOptions {
        max_concurrent_streams: options.max_concurrent_streams,
    };
    server_tcp::serve(
        listener,
        tcp_config,
        move |stream, cx| {
            let app = app.clone();
            async move {
                let _ = cx;
                transport_h2c::serve_connection_with_options(stream, app, connection_options)
                    .await
                    .map_err(|error| anyhow::anyhow!(error.to_string()))
            }
        },
        shutdown,
    )
    .await;
}

/// Standard INFO-level request tracing layer for HTTP runtimes.
pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, DefaultMakeSpan> {
    TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn serves_http1_and_h2c_on_one_listener_with_tunable_options() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("local addr");
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let app = Router::new().route("/healthz", get(|| async { "ok" }));
        let options = HttpServerOptions {
            max_concurrent_streams: 17,
            drain_timeout: Duration::from_secs(1),
            ..Default::default()
        };

        let server = tokio::spawn(serve_h2c_with_options(listener, app, options, async move {
            let _ = shutdown_rx.await;
        }));

        let mut http1 = tokio::net::TcpStream::connect(addr)
            .await
            .expect("connect HTTP/1.1");
        http1
            .write_all(b"GET /healthz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("write HTTP/1.1 request");
        let mut http1_response = Vec::new();
        http1
            .read_to_end(&mut http1_response)
            .await
            .expect("read HTTP/1.1 response");
        let http1_response = String::from_utf8_lossy(&http1_response);
        assert!(
            http1_response.starts_with("HTTP/1.1 200"),
            "plain HTTP/1.1 must share the listener: {http1_response}"
        );
        assert!(
            http1_response.ends_with("ok"),
            "plain HTTP/1.1 must dispatch the router body: {http1_response}"
        );

        let client = transport_h2c::h2c_client().expect("h2c client");
        let response = tokio::time::timeout(
            Duration::from_secs(3),
            client.get(format!("http://{addr}/healthz")).send(),
        )
        .await
        .expect("h2c response timeout")
        .expect("h2c response");
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(response.text().await.expect("body"), "ok");
        drop(client);

        let _ = shutdown_tx.send(());
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .expect("server shutdown")
            .expect("server task");
    }
}
// HANDWRITE-END

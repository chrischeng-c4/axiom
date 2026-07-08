//! Server-side h2c transport (behind the `server` feature): serve **HTTP/1.1
//! @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//! and HTTP/2 cleartext (h2c, prior-knowledge) on one socket** via hyper-util's
//! auto builder, with connection-level graceful shutdown.
//!
//! `axum::serve` speaks HTTP/1 only; this is the ecosystem's drop-in replacement
//! so a service actually accepts h2c (the in-cluster default) alongside HTTP/1.1
//! on a single port. The client side of the same transport lives in this crate's
//! `h2c_client` / `H2cPool` / `H2cManager`.

use std::time::Duration;

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_util::server::graceful::GracefulShutdown;
use tokio::net::TcpListener;
use tower::ServiceExt;

/// @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerOptions {
    pub max_concurrent_streams: u32,
    pub drain_timeout: Duration,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 4096,
            drain_timeout: Duration::from_secs(5),
        }
    }
}

/// Accept loop serving HTTP/1.1 + h2c on `listener`, dispatching every request
/// through the axum `app`. `shutdown` resolves to stop accepting (e.g. on
/// SIGTERM after the readiness-drain window); in-flight connections then get a
/// bounded grace period before the process is allowed to exit.
pub async fn serve(
    listener: TcpListener,
    app: axum::Router,
    shutdown: impl std::future::Future<Output = ()>,
) {
    serve_with_options(listener, app, ServerOptions::default(), shutdown).await;
}

/// Like [`serve`], with tunable HTTP/2 stream and drain settings.
pub async fn serve_with_options(
    listener: TcpListener,
    app: axum::Router,
    options: ServerOptions,
    shutdown: impl std::future::Future<Output = ()>,
) {
    // @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
    let mut builder = auto::Builder::new(TokioExecutor::new());
    // Lift the per-connection concurrent-stream ceiling: clients open
    // ~ln(concurrency) connections and multiplex many streams over each (see
    // this crate's connection-count heuristic). The hyper default (~200) caused
    // stream starvation / hangs at few-connections + high-concurrency. Flow-
    // control windows stay at hyper defaults — on a low-RTT link the workload is
    // CPU-bound (framing + JSON), not window-bound, so enlarging them is a
    // WAN-only tuning with no local benefit.
    builder
        .http2()
        .max_concurrent_streams(options.max_concurrent_streams);

    let graceful = GracefulShutdown::new();
    let mut shutdown = std::pin::pin!(shutdown);

    loop {
        tokio::select! {
            accept = listener.accept() => {
                let (stream, _peer) = match accept {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!(error = %e, "accept failed");
                        continue;
                    }
                };
                let io = TokioIo::new(stream);
                let app = app.clone();
                // axum's Router is Service<Request<Incoming>>; oneshot drives one request.
                let svc = service_fn(move |req| app.clone().oneshot(req));
                let conn = builder.serve_connection_with_upgrades(io, svc);
                let conn = graceful.watch(conn.into_owned());
                tokio::spawn(async move {
                    if let Err(e) = conn.await {
                        tracing::debug!(error = %e, "connection closed with error");
                    }
                });
            }
            _ = &mut shutdown => {
                tracing::info!("no longer accepting connections");
                break;
            }
        }
    }
    drop(listener);

    // Bound the in-flight wait so a stuck client can't block process exit.
    tokio::select! {
        _ = graceful.shutdown() => tracing::info!("all connections drained"),
        _ = tokio::time::sleep(options.drain_timeout) => {
            tracing::warn!("drain timeout — forcing shutdown")
        }
    }
}

// SPEC-MANAGED: libs/transport-h2c/tech-design/semantic/source/libs-transport-h2c-src-server-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Server-side h2c transport (behind the `server` feature): serve one accepted
//! stream as **HTTP/1.1 or HTTP/2 cleartext (h2c, prior-knowledge)** via
//! hyper-util's auto builder.
//! @spec apps/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! `axum::serve` speaks HTTP/1 only; this is the ecosystem's drop-in replacement
//! so a service actually accepts h2c (the in-cluster default) alongside HTTP/1.1
//! on a single port. The client side of the same transport lives in this crate's
//! `h2c_client` / `H2cPool` / `H2cManager`.

use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tower::ServiceExt;

/// Per-connection h2c tuning owned by the transport layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionOptions {
    pub max_concurrent_streams: u32,
}

pub type ConnectionError = Box<dyn std::error::Error + Send + Sync>;

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 4096,
        }
    }
}

/// Serve one accepted stream as HTTP/1.1 or h2c and dispatch every request
/// through the axum `app`. Listener admission, shutdown, and task supervision
/// belong to `server-http`/`server-tcp`.
/// @spec libs/transport-h2c/tech-design/semantic/source/libs-transport-h2c-src-server-rs.md#source
pub async fn serve_connection(stream: TcpStream, app: axum::Router) -> Result<(), ConnectionError> {
    serve_io(stream, app).await
}

/// Like [`serve_connection`], with a tunable HTTP/2 stream limit.
pub async fn serve_connection_with_options(
    stream: TcpStream,
    app: axum::Router,
    options: ConnectionOptions,
) -> Result<(), ConnectionError> {
    serve_io_with_options(stream, app, options).await
}

/// Serve one arbitrary Tokio byte stream as HTTP/1.1 or HTTP/2. This is the
/// transport seam used by authenticated peer ports after rustls completes its
/// handshake; cleartext callers continue to use [`serve_connection`].
pub async fn serve_io<I>(stream: I, app: axum::Router) -> Result<(), ConnectionError>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    serve_io_with_options(stream, app, ConnectionOptions::default()).await
}

pub async fn serve_io_with_options<I>(
    stream: I,
    app: axum::Router,
    options: ConnectionOptions,
) -> Result<(), ConnectionError>
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
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

    let io = TokioIo::new(stream);
    // axum's Router is Service<Request<Incoming>>; oneshot drives one request.
    let svc = service_fn(move |req| app.clone().oneshot(req));
    builder.serve_connection_with_upgrades(io, svc).await
}
// CODEGEN-END

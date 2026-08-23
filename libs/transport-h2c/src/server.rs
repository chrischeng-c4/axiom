// CODEGEN-BEGIN
//! Server-side h2c transport (behind the `server` feature): serve one accepted
//! stream as **HTTP/1.1 or HTTP/2 cleartext (h2c, prior-knowledge)** via
//! hyper-util's auto builder.
//!
//! `axum::serve` speaks HTTP/1 only; this is the ecosystem's drop-in replacement
//! so a service actually accepts h2c (the in-cluster default) alongside HTTP/1.1
//! on a single port. The client side of the same transport lives in this crate's
//! `h2c_client` / `H2cPool` / `H2cManager`.

use http::{Method, Request, Response, StatusCode, Version};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use server_lifecycle::{LifecycleSubscription, ShutdownDeadline};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tower::ServiceExt;

/// Per-connection h2c tuning owned by the transport layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionOptions {
    pub max_concurrent_streams: u32,
}

pub type ConnectionError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Undetermined,
    Http1,
    Http2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionTerminal {
    PeerClosed,
    Drained,
    DeadlineExceeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionReport {
    pub protocol: ConnectionProtocol,
    pub admitted: usize,
    pub active_at_drain: usize,
    pub completed: usize,
    pub refused: usize,
    pub timed_out: usize,
    pub ambiguous: usize,
    pub terminal: ConnectionTerminal,
    pub error: Option<String>,
}

struct Accounting {
    admission: Mutex<AdmissionState>,
    protocol: Mutex<ConnectionProtocol>,
    admitted: AtomicUsize,
    active: AtomicUsize,
    active_at_drain: AtomicUsize,
    completed: AtomicUsize,
    refused: AtomicUsize,
    timed_out: AtomicUsize,
    ambiguous: AtomicUsize,
}

struct AdmissionState {
    open: bool,
    active: usize,
    mutations: usize,
}

impl Accounting {
    fn new() -> Self {
        Self {
            admission: Mutex::new(AdmissionState {
                open: true,
                active: 0,
                mutations: 0,
            }),
            protocol: Mutex::new(ConnectionProtocol::Undetermined),
            admitted: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            active_at_drain: AtomicUsize::new(0),
            completed: AtomicUsize::new(0),
            refused: AtomicUsize::new(0),
            timed_out: AtomicUsize::new(0),
            ambiguous: AtomicUsize::new(0),
        }
    }

    fn protocol(&self, version: Version) {
        let mut protocol = self.protocol.lock().unwrap();
        *protocol = if version == Version::HTTP_2 {
            ConnectionProtocol::Http2
        } else {
            ConnectionProtocol::Http1
        };
    }

    fn begin_drain(&self) {
        let mut admission = self.admission.lock().unwrap();
        if admission.open {
            admission.open = false;
            self.active_at_drain
                .store(admission.active, Ordering::Release);
        }
    }

    fn drain_started(&self) -> bool {
        !self.admission.lock().unwrap().open
    }

    fn admit(self: &Arc<Self>, version: Version, method: &Method) -> Option<RequestGuard> {
        self.protocol(version);
        let mutation = !is_safe_method(method);
        let mut admission = self.admission.lock().unwrap();
        if !admission.open {
            self.refused.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        admission.active += 1;
        if mutation {
            admission.mutations += 1;
        }
        self.admitted.fetch_add(1, Ordering::Relaxed);
        self.active.fetch_add(1, Ordering::AcqRel);
        drop(admission);
        Some(RequestGuard {
            accounting: Arc::clone(self),
            mutation,
            done: false,
        })
    }

    fn mark_deadline(&self) {
        let admission = self.admission.lock().unwrap();
        let active = admission.active;
        self.timed_out.fetch_add(active, Ordering::Relaxed);
    }

    fn report(&self, terminal: ConnectionTerminal, error: Option<String>) -> ConnectionReport {
        ConnectionReport {
            protocol: *self.protocol.lock().unwrap(),
            admitted: self.admitted.load(Ordering::Acquire),
            active_at_drain: self.active_at_drain.load(Ordering::Acquire),
            completed: self.completed.load(Ordering::Acquire),
            refused: self.refused.load(Ordering::Acquire),
            timed_out: self.timed_out.load(Ordering::Acquire),
            ambiguous: self.ambiguous.load(Ordering::Acquire),
            terminal,
            error,
        }
    }
}

struct RequestGuard {
    accounting: Arc<Accounting>,
    mutation: bool,
    done: bool,
}
impl RequestGuard {
    fn complete(mut self) {
        self.done = true;
        let mut admission = self.accounting.admission.lock().unwrap();
        admission.active = admission.active.saturating_sub(1);
        if self.mutation {
            admission.mutations = admission.mutations.saturating_sub(1);
        }
        drop(admission);
        self.accounting.active.fetch_sub(1, Ordering::AcqRel);
        self.accounting.completed.fetch_add(1, Ordering::Relaxed);
    }
}
impl Drop for RequestGuard {
    fn drop(&mut self) {
        if !self.done {
            let mut admission = self.accounting.admission.lock().unwrap();
            admission.active = admission.active.saturating_sub(1);
            if self.mutation {
                admission.mutations = admission.mutations.saturating_sub(1);
            }
            drop(admission);
            self.accounting.active.fetch_sub(1, Ordering::AcqRel);
            if self.mutation {
                self.accounting.ambiguous.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

fn is_safe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

fn observe_drain(accounting: &Accounting, lifecycle: &Mutex<LifecycleSubscription>) -> bool {
    if lifecycle
        .lock()
        .unwrap()
        .observation()
        .phase
        .is_draining_or_later()
    {
        accounting.begin_drain();
        return true;
    }
    false
}

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

pub async fn serve_connection_with_drain(
    stream: TcpStream,
    app: axum::Router,
    options: ConnectionOptions,
    lifecycle: LifecycleSubscription,
    deadline: ShutdownDeadline,
) -> ConnectionReport {
    serve_io_with_drain(stream, app, options, lifecycle, deadline).await
}

/// Serve immediately while the lifecycle is Serving, then drain against the
/// absolute deadline published by that same lifecycle subscription.
pub async fn serve_connection_with_lifecycle(
    stream: TcpStream,
    app: axum::Router,
    options: ConnectionOptions,
    lifecycle: LifecycleSubscription,
) -> ConnectionReport {
    serve_io_with_lifecycle(stream, app, options, lifecycle).await
}

pub async fn serve_io_with_lifecycle<I>(
    stream: I,
    app: axum::Router,
    options: ConnectionOptions,
    lifecycle: LifecycleSubscription,
) -> ConnectionReport
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    serve_io_with_deadline_source(stream, app, options, lifecycle, None).await
}

pub async fn serve_io_with_drain<I>(
    stream: I,
    app: axum::Router,
    options: ConnectionOptions,
    lifecycle: LifecycleSubscription,
    deadline: ShutdownDeadline,
) -> ConnectionReport
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    serve_io_with_deadline_source(stream, app, options, lifecycle, Some(deadline)).await
}

async fn serve_io_with_deadline_source<I>(
    stream: I,
    app: axum::Router,
    options: ConnectionOptions,
    mut lifecycle: LifecycleSubscription,
    fixed_deadline: Option<ShutdownDeadline>,
) -> ConnectionReport
where
    I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let accounting = Arc::new(Accounting::new());
    let service_lifecycle = Arc::new(Mutex::new(lifecycle.clone()));
    let (terminal, error) = {
        let mut builder = auto::Builder::new(TokioExecutor::new());
        builder
            .http2()
            .max_concurrent_streams(options.max_concurrent_streams);
        let io = TokioIo::new(stream);
        let shared = Arc::clone(&accounting);
        let service_lifecycle = Arc::clone(&service_lifecycle);
        let svc = service_fn(move |req: Request<hyper::body::Incoming>| {
            let app = app.clone();
            let shared = Arc::clone(&shared);
            let service_lifecycle = Arc::clone(&service_lifecycle);
            async move {
                let version = req.version();
                let method = req.method().clone();
                observe_drain(&shared, &service_lifecycle);
                let Some(guard) = shared.admit(version, &method) else {
                    let response = Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .header("connection", "close")
                        .body(axum::body::Body::empty())
                        .expect("static response");
                    return Ok::<_, std::convert::Infallible>(response);
                };
                let mut response = app.oneshot(req).await;
                let draining = observe_drain(&shared, &service_lifecycle);
                if draining && version != Version::HTTP_2 {
                    if let Ok(response) = response.as_mut() {
                        response.headers_mut().insert(
                            http::header::CONNECTION,
                            http::HeaderValue::from_static("close"),
                        );
                    }
                }
                guard.complete();
                response
            }
        });
        let connection = builder.serve_connection_with_upgrades(io, svc);
        tokio::pin!(connection);
        loop {
            if lifecycle.observation().phase.is_draining_or_later() {
                accounting.begin_drain();
                connection.as_mut().graceful_shutdown();
                let Some(deadline) = fixed_deadline.or_else(|| lifecycle.shutdown_deadline())
                else {
                    break (
                        ConnectionTerminal::Failed,
                        Some("lifecycle entered draining without shutdown deadline".into()),
                    );
                };
                let usable = deadline.usable_remaining();
                if usable.is_zero() {
                    accounting.mark_deadline();
                    break (ConnectionTerminal::DeadlineExceeded, None);
                }
                match tokio::time::timeout(usable, &mut connection).await {
                    Ok(Ok(())) => break (ConnectionTerminal::Drained, None),
                    Ok(Err(error)) => break (ConnectionTerminal::Failed, Some(error.to_string())),
                    Err(_) => {
                        accounting.mark_deadline();
                        break (ConnectionTerminal::DeadlineExceeded, None);
                    }
                }
            }
            tokio::select! {
                result = &mut connection => break match result { Ok(()) => (if accounting.drain_started() { ConnectionTerminal::Drained } else { ConnectionTerminal::PeerClosed }, None), Err(error) => (ConnectionTerminal::Failed, Some(error.to_string())) },
                _ = lifecycle.changed() => {}
            }
        }
    };
    accounting.report(terminal, error)
}
// CODEGEN-END

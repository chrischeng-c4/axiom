//! `H2cManager` — a self-managing pool of frame-level h2c connections to one
//! authority.
//!
//! Where [`H2cPool`](crate::H2cPool) is a fixed-size, blindly round-robin set of
//! reqwest clients, `H2cManager` actively manages the underlying `h2`
//! connections:
//!
//! - **least-loaded dispatch** — each request goes to the healthy connection
//!   with the fewest in-flight streams (not blind round-robin).
//! - **adaptive sizing** — grows a new connection when the least-loaded one is
//!   saturated, up to the `ln(concurrency)`/cores cap; a supervisor shrinks
//!   connections that sit idle past `idle_timeout` (down to `min_connections`).
//! - **health / failover** — a supervisor PINGs each connection for liveness;
//!   the connection driver flags GOAWAY / I/O death; dead connections are
//!   evicted and replenished to `min_connections`. A request that loses its
//!   connection is retried once on a fresh one.
//! - **metrics** — [`H2cManager::stats`] snapshots connection count, health,
//!   in-flight streams, and lifetime request/error totals.
//!
//! Cheap to [`Clone`] (shares one `Arc` of state); clone freely across tasks.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use bytes::Bytes;
use http::{Method, Request, Response};
use tokio::sync::RwLock;
use tokio::time::MissedTickBehavior;

use crate::conn::{ConnConfig, ManagedConn};
use crate::error::{H2cError, Result};
use crate::{cpu_parallelism, recommended_h2c_connections};

/// Configuration for an [`H2cManager`].
#[derive(Clone, Debug)]
pub struct ManagerConfig {
    /// Connections kept warm at all times (opened eagerly at connect).
    pub min_connections: usize,
    /// Hard ceiling on connections (adaptive growth stops here).
    pub max_connections: usize,
    /// Grow a new connection when the least-loaded healthy one has at least this
    /// many in-flight streams (and we're under `max_connections`).
    pub grow_threshold: usize,
    /// Deadline for a single TCP connect + handshake.
    pub connect_timeout: Duration,
    /// Per-request deadline (`None` disables it).
    pub request_timeout: Option<Duration>,
    /// Supervisor cadence: liveness ping + prune/shrink/replenish sweep.
    pub ping_interval: Duration,
    /// Shrink a connection idle longer than this (above `min_connections`).
    pub idle_timeout: Duration,
    /// h2 per-stream receive window.
    pub stream_window: u32,
    /// h2 whole-connection receive window.
    pub conn_window: u32,
    /// h2 max frame size.
    pub max_frame: u32,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: cpu_parallelism().max(1),
            grow_threshold: 32,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Some(Duration::from_secs(30)),
            ping_interval: Duration::from_secs(15),
            idle_timeout: Duration::from_secs(60),
            stream_window: 1024 * 1024,   // 1 MiB
            conn_window: 4 * 1024 * 1024, // 4 MiB
            max_frame: 16 * 1024,         // 16 KiB
        }
    }
}

impl ManagerConfig {
    /// Cap `max_connections` by the `ln(concurrency)`/cores heuristic for a
    /// target peak concurrency.
    pub fn for_concurrency(concurrency: usize) -> Self {
        let mut c = Self::default();
        c.max_connections = recommended_h2c_connections(concurrency).max(c.min_connections);
        c
    }

    fn conn_config(&self) -> ConnConfig {
        ConnConfig {
            stream_window: self.stream_window,
            conn_window: self.conn_window,
            max_frame: self.max_frame,
        }
    }
}

/// Aggregate snapshot of an [`H2cManager`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManagerStats {
    /// Connections currently in the pool (healthy + not-yet-pruned dead).
    pub connections: usize,
    /// Of those, how many are healthy.
    pub healthy: usize,
    /// In-flight streams summed across connections.
    pub in_flight: usize,
    /// Lifetime requests started (including on since-evicted connections).
    pub total_requests: u64,
    /// Lifetime errors (including on since-evicted connections).
    pub total_errors: u64,
}

struct Inner {
    authority: String,
    cfg: ManagerConfig,
    conns: RwLock<Vec<Arc<ManagedConn>>>,
    /// Connections-plus-pending-connects, used to enforce `max_connections`
    /// without holding the conns write lock across a (slow) connect. Kept equal
    /// to `conns.len()` once connects settle.
    slots: AtomicUsize,
    next_id: AtomicUsize,
    shutdown: AtomicBool,
    // Totals from connections that have been evicted, so lifetime stats survive.
    retired_requests: AtomicU64,
    retired_errors: AtomicU64,
}

/// A self-managing pool of frame-level h2c connections to one authority.
///
/// ```no_run
/// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
/// // Warm a managed pool to a keep node and drive requests through it.
/// let mgr = h2c::H2cManager::connect("keep:7117").await?;
/// let resp = mgr.get("/healthz").await?;
/// assert!(resp.status().is_success());
/// // It grows/shrinks/heals connections on its own; snapshot the live state:
/// let s = mgr.stats().await;
/// println!("{}/{} healthy, {} in-flight", s.healthy, s.connections, s.in_flight);
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct H2cManager {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for H2cManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H2cManager")
            .field("authority", &self.inner.authority)
            .field("max_connections", &self.inner.cfg.max_connections)
            .finish_non_exhaustive()
    }
}

impl H2cManager {
    /// Connect a manager to `endpoint` (`host:port` or `http://host:port`),
    /// opening `min_connections` eagerly and starting the supervisor.
    pub async fn connect(endpoint: &str) -> Result<Self> {
        Self::with_config(endpoint, ManagerConfig::default()).await
    }

    /// Like [`connect`](Self::connect) with an explicit [`ManagerConfig`].
    pub async fn with_config(endpoint: &str, cfg: ManagerConfig) -> Result<Self> {
        let authority = authority_of(endpoint);
        let inner = Arc::new(Inner {
            authority,
            cfg,
            conns: RwLock::new(Vec::new()),
            slots: AtomicUsize::new(0),
            next_id: AtomicUsize::new(0),
            shutdown: AtomicBool::new(false),
            retired_requests: AtomicU64::new(0),
            retired_errors: AtomicU64::new(0),
        });
        let mgr = H2cManager { inner };

        // Open the warm minimum eagerly so the first request is fast.
        for _ in 0..mgr.inner.cfg.min_connections.max(1) {
            mgr.grow_one().await?;
        }

        // Supervisor holds a Weak so it stops when every handle is dropped.
        let weak = Arc::downgrade(&mgr.inner);
        tokio::spawn(supervise(weak));
        Ok(mgr)
    }

    /// The `host:port` this manager dials.
    pub fn authority(&self) -> &str {
        &self.inner.authority
    }

    /// GET `path` (e.g. `/healthz`).
    pub async fn get(&self, path: &str) -> Result<Response<Bytes>> {
        self.request(self.build(Method::GET, path, Bytes::new())?)
            .await
    }

    /// PUT `path` with `body`.
    pub async fn put(&self, path: &str, body: Bytes) -> Result<Response<Bytes>> {
        self.request(self.build(Method::PUT, path, body)?).await
    }

    /// POST `path` with `body`.
    pub async fn post(&self, path: &str, body: Bytes) -> Result<Response<Bytes>> {
        self.request(self.build(Method::POST, path, body)?).await
    }

    /// Send a fully-built request. Dispatches to the least-loaded healthy
    /// connection; on a lost connection, retries once on a fresh one.
    pub async fn request(&self, req: Request<Bytes>) -> Result<Response<Bytes>> {
        let timeout = self.inner.cfg.request_timeout;
        let mut last_err: Option<H2cError> = None;
        for attempt in 0..2u8 {
            let lease = self.acquire().await?;
            lease.conn.touch();
            let res = match timeout {
                Some(t) => {
                    match tokio::time::timeout(t, lease.conn.send(dup_request(&req))).await {
                        Ok(r) => r,
                        Err(_) => return Err(H2cError::Timeout(t)),
                    }
                }
                None => lease.conn.send(dup_request(&req)).await,
            };
            drop(lease); // release the in-flight slot before retry / return
            match res {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let retryable = e.is_connection_lost();
                    last_err = Some(e);
                    if attempt == 0 && retryable {
                        continue; // fresh connection on the next loop
                    }
                    break;
                }
            }
        }
        Err(last_err.unwrap_or_else(|| H2cError::NoConnection(self.inner.authority.clone())))
    }

    /// Snapshot connection / health / in-flight / lifetime totals.
    pub async fn stats(&self) -> ManagerStats {
        let conns = self.inner.conns.read().await;
        let mut s = ManagerStats {
            connections: conns.len(),
            ..Default::default()
        };
        for c in conns.iter() {
            if c.is_healthy() {
                s.healthy += 1;
            }
            s.in_flight += c.in_flight();
            s.total_requests += c.total();
            s.total_errors += c.errors();
        }
        s.total_requests += self.inner.retired_requests.load(Ordering::Relaxed);
        s.total_errors += self.inner.retired_errors.load(Ordering::Relaxed);
        s
    }

    /// Stop the supervisor and drop all connections (draining in-flight streams
    /// as their futures complete). Subsequent requests fail with `Shutdown`.
    pub async fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        let drained: Vec<Arc<ManagedConn>> = {
            let mut conns = self.inner.conns.write().await;
            for c in conns.iter() {
                self.inner
                    .retired_requests
                    .fetch_add(c.total(), Ordering::Relaxed);
                self.inner
                    .retired_errors
                    .fetch_add(c.errors(), Ordering::Relaxed);
            }
            std::mem::take(&mut conns)
        };
        drop(drained);
    }

    fn build(&self, method: Method, path: &str, body: Bytes) -> Result<Request<Bytes>> {
        let uri = format!("http://{}{}", self.inner.authority, path);
        Ok(Request::builder().method(method).uri(uri).body(body)?)
    }

    /// Lease the least-loaded healthy connection (reserving an in-flight slot on
    /// it), growing a new connection when the best is saturated (and under
    /// `max_connections`) or none is healthy.
    async fn acquire(&self) -> Result<Lease> {
        if self.inner.shutdown.load(Ordering::Acquire) {
            return Err(H2cError::Shutdown);
        }
        let cfg = &self.inner.cfg;
        let (best, total) = {
            let conns = self.inner.conns.read().await;
            let best = conns
                .iter()
                .filter(|c| c.is_healthy())
                .min_by_key(|c| c.in_flight())
                .cloned();
            (best, conns.len())
        };

        let should_grow = match &best {
            None => true, // no healthy connection
            Some(c) => c.in_flight() >= cfg.grow_threshold && total < cfg.max_connections,
        };
        let chosen = if should_grow {
            match self.grow_one().await {
                Ok(c) => c,
                Err(e) => match best {
                    Some(b) => {
                        tracing::debug!(error = %e, "grow failed; using least-loaded existing conn");
                        b
                    }
                    None => return Err(e),
                },
            }
        } else {
            best.expect("best is Some when should_grow is false")
        };
        // Reserve the slot now so concurrent acquirers see this connection's load
        // rise — that's what makes adaptive growth track real demand.
        chosen.reserve();
        Ok(Lease { conn: chosen })
    }

    /// Open one new connection and add it to the pool (respecting the cap).
    async fn grow_one(&self) -> Result<Arc<ManagedConn>> {
        connect_tracked(&self.inner).await
    }
}

/// Open one connection and track it in the pool, enforcing `max_connections`
/// via the `slots` reservation so no orphan (connected-but-untracked) socket is
/// ever created. Shared by on-demand growth and supervisor replenishment.
async fn connect_tracked(inner: &Arc<Inner>) -> Result<Arc<ManagedConn>> {
    let cfg = &inner.cfg;
    // Claim a slot up front; back out if it would breach the cap.
    let prev = inner.slots.fetch_add(1, Ordering::AcqRel);
    if prev >= cfg.max_connections {
        inner.slots.fetch_sub(1, Ordering::AcqRel);
        return Err(H2cError::NoConnection(format!(
            "{} at max_connections ({})",
            inner.authority, cfg.max_connections
        )));
    }
    let id = inner.next_id.fetch_add(1, Ordering::Relaxed);
    let connect = ManagedConn::connect(id, &inner.authority, cfg.conn_config());
    let conn = match tokio::time::timeout(cfg.connect_timeout, connect).await {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            inner.slots.fetch_sub(1, Ordering::AcqRel);
            return Err(e);
        }
        Err(_) => {
            inner.slots.fetch_sub(1, Ordering::AcqRel);
            return Err(H2cError::Timeout(cfg.connect_timeout));
        }
    };
    inner.conns.write().await.push(conn.clone());
    Ok(conn)
}

/// Retire a connection's lifetime totals and free its slot (on evict / shrink).
fn retire(inner: &Inner, c: &ManagedConn) {
    inner
        .retired_requests
        .fetch_add(c.total(), Ordering::Relaxed);
    inner
        .retired_errors
        .fetch_add(c.errors(), Ordering::Relaxed);
    inner.slots.fetch_sub(1, Ordering::AcqRel);
}

/// An in-flight reservation on a connection. Holds the connection alive for the
/// request and releases the reserved slot on drop (including on cancellation).
struct Lease {
    conn: Arc<ManagedConn>,
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.conn.release();
    }
}

/// Background supervisor: ping for liveness, evict dead, shrink idle, replenish
/// to `min_connections`. Exits when the manager is fully dropped or shut down.
async fn supervise(weak: Weak<Inner>) {
    let interval = match weak.upgrade() {
        Some(inner) => inner.cfg.ping_interval,
        None => return,
    };
    let mut tick = tokio::time::interval(interval);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    tick.tick().await; // consume the immediate first fire

    loop {
        tick.tick().await;
        let Some(inner) = weak.upgrade() else { break };
        if inner.shutdown.load(Ordering::Acquire) {
            break;
        }

        // 1. Liveness-ping healthy connections; flag the dead ones.
        let snapshot: Vec<Arc<ManagedConn>> = inner.conns.read().await.clone();
        for c in &snapshot {
            if c.is_healthy() {
                if let Err(e) = c.ping().await {
                    tracing::debug!(conn = c.id, error = %e, "liveness ping failed; evicting");
                    c.mark_dead();
                }
            }
        }

        // 2. Evict dead + shrink one idle connection above the minimum.
        {
            let mut conns = inner.conns.write().await;
            let min = inner.cfg.min_connections;
            conns.retain(|c| {
                let keep = c.is_healthy();
                if !keep {
                    retire(&inner, c);
                }
                keep
            });
            if conns.len() > min {
                if let Some(pos) = conns
                    .iter()
                    .position(|c| c.in_flight() == 0 && c.idle() >= inner.cfg.idle_timeout)
                {
                    let c = conns.remove(pos);
                    retire(&inner, &c);
                    tracing::debug!(conn = c.id, "shrinking idle h2c connection");
                }
            }
        }

        // 3. Replenish to the warm minimum.
        let deficit = inner
            .cfg
            .min_connections
            .saturating_sub(inner.conns.read().await.len());
        for _ in 0..deficit {
            if let Err(e) = connect_tracked(&inner).await {
                tracing::debug!(error = %e, "replenish connect failed");
                break;
            }
        }
        // `inner` dropped here, before the next idle tick, so the Weak can die.
    }
}

/// Normalize an endpoint to a bare `host:port` authority for `TcpStream` +
/// the `:authority` pseudo-header (strip a leading `http://`, trailing `/`).
fn authority_of(endpoint: &str) -> String {
    endpoint
        .strip_prefix("http://")
        .unwrap_or(endpoint)
        .trim_end_matches('/')
        .to_string()
}

/// Clone a request (method / uri / version / headers / body) for a retry.
/// Extensions are dropped — the manager never sets any.
fn dup_request(req: &Request<Bytes>) -> Request<Bytes> {
    let mut builder = Request::builder()
        .method(req.method().clone())
        .uri(req.uri().clone())
        .version(req.version());
    if let Some(headers) = builder.headers_mut() {
        *headers = req.headers().clone();
    }
    builder
        .body(req.body().clone())
        .expect("rebuilding a validated request cannot fail")
}

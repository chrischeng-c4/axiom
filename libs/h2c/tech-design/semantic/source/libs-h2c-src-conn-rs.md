---
id: libs-h2c-src-conn-rs
summary: Lossless rust-source-unit coverage for `libs/h2c/src/conn.rs`.
capability_refs:
  - id: http2-cleartext-client-helpers
    role: primary
    claim: http2-cleartext-client-helpers-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the H2c library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/h2c/src/conn.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/h2c/src/conn.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ConnConfig` | libs/h2c/src/conn.rs | struct | pub | 25 | pub(crate) struct ConnConfig { |
| `ManagedConn` | libs/h2c/src/conn.rs | struct | pub | 32 | pub(crate) struct ManagedConn { |
| `connect` | libs/h2c/src/conn.rs | function | pub | 56 | pub(crate) async fn connect(id: usize, authority: &str, cfg: ConnConfig) -> Result<Arc<Self>> { |
| `is_healthy` | libs/h2c/src/conn.rs | function | pub | 104 | pub(crate) fn is_healthy(&self) -> bool { |
| `mark_dead` | libs/h2c/src/conn.rs | function | pub | 108 | pub(crate) fn mark_dead(&self) { |
| `in_flight` | libs/h2c/src/conn.rs | function | pub | 112 | pub(crate) fn in_flight(&self) -> usize { |
| `reserve` | libs/h2c/src/conn.rs | function | pub | 118 | pub(crate) fn reserve(&self) { |
| `release` | libs/h2c/src/conn.rs | function | pub | 123 | pub(crate) fn release(&self) { |
| `total` | libs/h2c/src/conn.rs | function | pub | 127 | pub(crate) fn total(&self) -> u64 { |
| `errors` | libs/h2c/src/conn.rs | function | pub | 131 | pub(crate) fn errors(&self) -> u64 { |
| `touch` | libs/h2c/src/conn.rs | function | pub | 136 | pub(crate) fn touch(&self) { |
| `idle` | libs/h2c/src/conn.rs | function | pub | 142 | pub(crate) fn idle(&self) -> Duration { |
| `send` | libs/h2c/src/conn.rs | function | pub | 150 | pub(crate) async fn send(&self, req: Request<Bytes>) -> Result<Response<Bytes>> { |
| `ping` | libs/h2c/src/conn.rs | function | pub | 192 | pub(crate) async fn ping(&self) -> Result<Duration> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `ManagedConn` — one frame-level h2c connection and its live health/stats.
//!
//! Each connection owns a single TCP socket carrying an HTTP/2 cleartext
//! (prior-knowledge) session. The `h2` connection driver runs on its own task;
//! the cloneable [`h2::client::SendRequest`] handle multiplexes many concurrent
//! streams over the one socket. We track in-flight streams, totals, and a
//! liveness bit the driver flips when the connection ends (clean, errored, or
//! GOAWAY) — the signals reqwest hides.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bytes::{BufMut, Bytes, BytesMut};
use h2::client::SendRequest;
use h2::{Ping, PingPong, SendStream};
use http::{Request, Response};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::error::{H2cError, Result};

/// h2 handshake tunables for one connection.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ConnConfig {
    pub stream_window: u32,
    pub conn_window: u32,
    pub max_frame: u32,
}

/// One managed frame-level h2c connection. Cheap to share via `Arc`.
pub(crate) struct ManagedConn {
    pub(crate) id: usize,
    /// The multiplexing handle. `SendRequest` is `Clone`; we clone it per
    /// request so concurrent sends don't contend on a lock.
    send: SendRequest<Bytes>,
    /// Liveness probe, taken once from the handshake. Behind a mutex because
    /// only the supervisor pings, one at a time.
    ping: Mutex<Option<PingPong>>,
    /// Flipped to `false` by the connection driver when the connection closes
    /// (clean / errored / GOAWAY), or by [`mark_dead`](Self::mark_dead). The
    /// authoritative liveness bit.
    healthy: Arc<AtomicBool>,
    in_flight: AtomicUsize,
    total: AtomicU64,
    errors: AtomicU64,
    created: Instant,
    /// Millis-since-`created` of the last request start, for idle shrink.
    last_used_ms: AtomicU64,
}

impl ManagedConn {
    /// Open a new connection: TCP connect → h2c prior-knowledge handshake →
    /// spawn the connection driver. The driver owns the socket I/O; when it
    /// ends, the connection is marked unhealthy so the manager evicts it.
    pub(crate) async fn connect(id: usize, authority: &str, cfg: ConnConfig) -> Result<Arc<Self>> {
        let tcp = TcpStream::connect(authority)
            .await
            .map_err(|source| H2cError::Connect {
                authority: authority.to_string(),
                source,
            })?;
        let _ = tcp.set_nodelay(true);

        let mut builder = h2::client::Builder::new();
        builder
            .initial_window_size(cfg.stream_window)
            .initial_connection_window_size(cfg.conn_window)
            .max_frame_size(cfg.max_frame);
        let (send, mut connection) = builder.handshake::<_, Bytes>(tcp).await?;

        // The ping/pong handle lives on the connection — take it before the
        // connection future is moved into its driver task.
        let ping = connection.ping_pong();

        // Drive the connection on its own task; flag death when it ends.
        let healthy = Arc::new(AtomicBool::new(true));
        let driver_flag = healthy.clone();
        tokio::spawn(async move {
            match connection.await {
                Ok(()) => tracing::debug!(conn = id, "h2c connection closed"),
                Err(e) => tracing::debug!(
                    conn = id,
                    error = %e,
                    go_away = e.is_go_away(),
                    "h2c connection ended"
                ),
            }
            driver_flag.store(false, Ordering::Release);
        });
        Ok(Arc::new(Self {
            id,
            send,
            ping: Mutex::new(ping),
            healthy,
            in_flight: AtomicUsize::new(0),
            total: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            created: Instant::now(),
            last_used_ms: AtomicU64::new(0),
        }))
    }

    pub(crate) fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Acquire)
    }

    pub(crate) fn mark_dead(&self) {
        self.healthy.store(false, Ordering::Release);
    }

    pub(crate) fn in_flight(&self) -> usize {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Reserve an in-flight slot (the manager calls this at dispatch time so a
    /// burst of concurrent dispatches sees load rise and grows the pool).
    pub(crate) fn reserve(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Release a reserved slot — even if the request future was cancelled.
    pub(crate) fn release(&self) {
        self.in_flight.fetch_sub(1, Ordering::Relaxed);
    }

    pub(crate) fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub(crate) fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    /// Mark this connection as just-used (resets its idle clock).
    pub(crate) fn touch(&self) {
        self.last_used_ms
            .store(self.created.elapsed().as_millis() as u64, Ordering::Relaxed);
    }

    /// How long since the last request started on this connection.
    pub(crate) fn idle(&self) -> Duration {
        let now = self.created.elapsed().as_millis() as u64;
        Duration::from_millis(now.saturating_sub(self.last_used_ms.load(Ordering::Relaxed)))
    }

    /// Round-trip one request over this connection. The in-flight slot is
    /// reserved/released by the manager's lease (around dispatch + retry); here
    /// we only record totals/errors and flag a lost connection.
    pub(crate) async fn send(&self, req: Request<Bytes>) -> Result<Response<Bytes>> {
        self.total.fetch_add(1, Ordering::Relaxed);
        match self.send_inner(req).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                self.errors.fetch_add(1, Ordering::Relaxed);
                if e.is_connection_lost() {
                    self.mark_dead();
                }
                Err(e)
            }
        }
    }

    async fn send_inner(&self, req: Request<Bytes>) -> Result<Response<Bytes>> {
        let (parts, body) = req.into_parts();
        let head = Request::from_parts(parts, ());
        let eos = body.is_empty();

        // `ready()` is the back-pressure point: it resolves once the connection
        // can open another stream (under the peer's MAX_CONCURRENT_STREAMS).
        let mut send = self.send.clone().ready().await?;
        let (resp_fut, mut stream) = send.send_request(head, eos)?;

        if !eos {
            send_body(&mut stream, body).await?;
        }

        let resp = resp_fut.await?;
        let (parts, mut recv) = resp.into_parts();
        let mut buf = BytesMut::new();
        while let Some(chunk) = recv.data().await {
            let chunk = chunk?;
            // Release flow-control capacity so the peer keeps sending.
            let _ = recv.flow_control().release_capacity(chunk.len());
            buf.put_slice(&chunk);
        }
        Ok(Response::from_parts(parts, buf.freeze()))
    }

    /// Send a PING and await the PONG — a positive liveness probe that also
    /// measures RTT. An error means the connection is dead.
    pub(crate) async fn ping(&self) -> Result<Duration> {
        let mut guard = self.ping.lock().await;
        let Some(pp) = guard.as_mut() else {
            // No ping handle — fall back to the driver's liveness bit.
            return if self.is_healthy() {
                Ok(Duration::ZERO)
            } else {
                Err(H2cError::NoConnection("ping".into()))
            };
        };
        let t = Instant::now();
        pp.ping(Ping::opaque()).await?;
        Ok(t.elapsed())
    }
}

/// Send a request body over a `SendStream`, respecting HTTP/2 flow control:
/// reserve capacity, wait for the window, send a bounded chunk, repeat.
async fn send_body(stream: &mut SendStream<Bytes>, mut body: Bytes) -> Result<()> {
    while !body.is_empty() {
        stream.reserve_capacity(body.len());
        let cap = std::future::poll_fn(|cx| stream.poll_capacity(cx))
            .await
            .transpose()? // Option<Result<usize>> → Result<Option<usize>>
            .unwrap_or(0);
        if cap == 0 {
            // The stream/connection closed before the body was fully sent.
            return Err(H2cError::NoConnection("send body: stream closed".into()));
        }
        let n = cap.min(body.len());
        let chunk = body.split_to(n);
        let end = body.is_empty();
        stream.send_data(chunk, end)?;
    }
    Ok(())
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/h2c/src/conn.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/h2c/src/conn.rs` captured during libs codegen standardization.
```

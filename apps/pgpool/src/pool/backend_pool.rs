// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! `BackendPool`: the shared backend-connection pool used by both pool
//! modes (Logic section). Capacity (R1) is tracked with a `tokio::sync::Semaphore`
//! so every physical connection — idle or active — holds exactly one
//! permit for its whole life; a lease's `CapacityGuard` returns that permit
//! if the lease is ever dropped without an explicit `release()` call, so
//! capacity can never leak (R5) even though `release()` itself always runs
//! first in the normal path (the guard's cleanup is a no-op once `release()`
//! has already removed the id from `outstanding`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;

use crate::pool::types::{
    BackendConnectionId, BackendPoolStats, LeaseDisposition, PoolConfig, PoolError,
};
use crate::wire::{
    BackendMessage, FrameReader, FrontendMessage, Query, Role, WireCodecConfig, WireMessage,
};

/// One leased physical backend connection, handed out by
/// [`BackendPool::acquire`]/[`BackendPool::acquire_fresh`].
#[derive(Debug)]
pub struct BackendLease {
    pub id: BackendConnectionId,
    /// `true` when this connection needs startup+auth relay before it can
    /// carry query traffic (a brand-new connect, from either
    /// `acquire_fresh()` or `acquire()`'s fresh-connect branch); `false`
    /// when an already-authenticated idle connection was reused and only
    /// post-auth traffic should be relayed.
    pub fresh: bool,
    pub stream: TcpStream,
    /// Never read directly — held only for its `Drop` side effect (R5): if
    /// this lease is ever dropped without an explicit `release()` call, the
    /// guard returns the permit so capacity can never leak.
    #[allow(dead_code)]
    capacity_guard: CapacityGuard,
}

struct CapacityGuard {
    inner: Arc<PoolInner>,
    id: BackendConnectionId,
}

impl std::fmt::Debug for CapacityGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapacityGuard")
            .field("id", &self.id)
            .finish()
    }
}

impl Drop for CapacityGuard {
    fn drop(&mut self) {
        // Idempotent: a normal `release()` call already removed this id from
        // `outstanding` (parking the permit in `idle` or dropping it for
        // good). If it is still present here, the lease was dropped without
        // an explicit release (R5) and the permit needs to be returned so
        // its capacity slot is not leaked.
        let removed = {
            let mut state = self.inner.state.lock().expect("pool state lock");
            state.outstanding.remove(&self.id)
        };
        if removed.is_some() {
            self.inner.notify.notify_waiters();
        }
    }
}

#[derive(Debug)]
struct PoolState {
    idle: Vec<(BackendConnectionId, TcpStream, OwnedSemaphorePermit)>,
    outstanding: HashMap<BackendConnectionId, OwnedSemaphorePermit>,
    next_id: u64,
}

#[derive(Debug)]
struct PoolInner {
    permits: Arc<Semaphore>,
    state: Mutex<PoolState>,
    notify: Notify,
}

/// Shared backend-connection pool: idle-reuse-preferring acquire with a
/// non-blocking liveness peek, always-fresh acquire for the one-time
/// startup+auth handshake, and a disposition-driven release (Logic
/// section). Cheaply `Clone`-able (an `Arc`-backed handle) so both
/// `SessionProxyConfig` and `TransactionProxyConfig` can share one pool.
#[derive(Debug, Clone)]
pub struct BackendPool {
    config: PoolConfig,
    inner: Arc<PoolInner>,
}

impl BackendPool {
    pub fn new(config: PoolConfig) -> Self {
        let max = config.max_backend_connections;
        Self {
            config,
            inner: Arc::new(PoolInner {
                permits: Arc::new(Semaphore::new(max)),
                state: Mutex::new(PoolState {
                    idle: Vec::new(),
                    outstanding: HashMap::new(),
                    next_id: 0,
                }),
                notify: Notify::new(),
            }),
        }
    }

    /// Idle-reuse-preferring acquire (Logic section `acquire_txn_backend`):
    /// pops idle connections (dropping any that fail their non-blocking
    /// liveness peek and retrying — R1a/R1b), else fresh-connects if
    /// capacity remains, else waits up to `acquire_timeout` for a slot to
    /// free before `PoolError::Saturated` (R3a/R3b).
    pub async fn acquire(&self) -> Result<BackendLease, PoolError> {
        self.acquire_internal(true).await
    }

    /// Always a brand-new connect (Logic section `txn_admit_handshake` /
    /// session-mode's `connect_backend`), bounded by `backend_connect_timeout`;
    /// if the pool is momentarily saturated this waits up to
    /// `acquire_timeout` for a capacity slot first, exactly like `acquire()`,
    /// but never reuses an idle connection.
    pub async fn acquire_fresh(&self) -> Result<BackendLease, PoolError> {
        self.acquire_internal(false).await
    }

    async fn acquire_internal(&self, allow_idle_reuse: bool) -> Result<BackendLease, PoolError> {
        let deadline = Instant::now() + self.config.acquire_timeout;
        loop {
            if allow_idle_reuse {
                if let Some(lease) = self.try_take_idle().await {
                    return Ok(lease);
                }
            }

            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                return self.connect_fresh(permit).await;
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(self.saturated());
            }

            let notified = self.inner.notify.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();

            if tokio::time::timeout(remaining, notified).await.is_err() {
                return Err(self.saturated());
            }
            // Notified (or spuriously woken): loop back and recheck.
        }
    }

    fn saturated(&self) -> PoolError {
        PoolError::Saturated {
            max: self.config.max_backend_connections,
            waited: self.config.acquire_timeout,
        }
    }

    /// Returns a leased connection per `disposition`:
    /// [`LeaseDisposition::ReturnToIdle`] resets it with `DISCARD ALL` and
    /// parks it in the idle set on success (R1c), closing it instead if the
    /// reset fails/EOFs/times out (R1d); [`LeaseDisposition::Close`] tears
    /// it down immediately. Either way the connection's capacity slot is
    /// freed for reuse via the semaphore permit tracked for `id`.
    pub async fn release(
        &self,
        id: BackendConnectionId,
        stream: TcpStream,
        disposition: LeaseDisposition,
    ) {
        let permit = {
            let mut state = self.inner.state.lock().expect("pool state lock");
            state.outstanding.remove(&id)
        };
        let Some(permit) = permit else {
            // Already handled (double-release, or the lease's
            // `CapacityGuard` already fired); nothing left to account for
            // beyond not leaking this socket.
            drop(stream);
            return;
        };

        match disposition {
            LeaseDisposition::Close => {
                let mut stream = stream;
                let _ = stream.shutdown().await;
                drop(stream);
                drop(permit);
                self.inner.notify.notify_waiters();
            }
            LeaseDisposition::ReturnToIdle => {
                match reset_connection(
                    stream,
                    &self.config.wire,
                    self.config.backend_connect_timeout,
                )
                .await
                {
                    Ok(stream) => {
                        let mut state = self.inner.state.lock().expect("pool state lock");
                        state.idle.push((id, stream, permit));
                        drop(state);
                        self.inner.notify.notify_waiters();
                    }
                    Err(mut stream) => {
                        let _ = stream.shutdown().await;
                        drop(stream);
                        drop(permit);
                        self.inner.notify.notify_waiters();
                    }
                }
            }
        }
    }

    pub fn stats(&self) -> BackendPoolStats {
        let state = self.inner.state.lock().expect("pool state lock");
        BackendPoolStats {
            backend_active: state.outstanding.len(),
            backend_idle: state.idle.len(),
        }
    }

    async fn try_take_idle(&self) -> Option<BackendLease> {
        loop {
            let candidate = {
                let mut state = self.inner.state.lock().expect("pool state lock");
                state.idle.pop()
            };
            let (id, stream, permit) = candidate?;
            if liveness_check(&stream).await {
                let mut state = self.inner.state.lock().expect("pool state lock");
                state.outstanding.insert(id, permit);
                drop(state);
                return Some(BackendLease {
                    id,
                    fresh: false,
                    stream,
                    capacity_guard: CapacityGuard {
                        inner: Arc::clone(&self.inner),
                        id,
                    },
                });
            }
            // Dead idle connection (R1b): dropping `permit` frees its
            // capacity slot for a future fresh-connect or waiter.
            drop(stream);
            drop(permit);
            self.inner.notify.notify_waiters();
        }
    }

    async fn connect_fresh(&self, permit: OwnedSemaphorePermit) -> Result<BackendLease, PoolError> {
        let addr = format!(
            "{}:{}",
            self.config.endpoint.host, self.config.endpoint.port
        );
        let connect = TcpStream::connect(&addr);
        let stream = match tokio::time::timeout(self.config.backend_connect_timeout, connect).await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(error)) => {
                drop(permit);
                self.inner.notify.notify_waiters();
                return Err(PoolError::BackendUnreachable(error.to_string()));
            }
            Err(_) => {
                drop(permit);
                self.inner.notify.notify_waiters();
                return Err(PoolError::BackendUnreachable(format!(
                    "backend connect to {addr} timed out after {:?}",
                    self.config.backend_connect_timeout
                )));
            }
        };

        let id = {
            let mut state = self.inner.state.lock().expect("pool state lock");
            let id = BackendConnectionId(state.next_id);
            state.next_id += 1;
            state.outstanding.insert(id, permit);
            id
        };
        Ok(BackendLease {
            id,
            fresh: true,
            stream,
            capacity_guard: CapacityGuard {
                inner: Arc::clone(&self.inner),
                id,
            },
        })
    }
}

/// Non-blocking liveness peek (R1): an idle connection with no pending read
/// readiness is presumed alive (the expected steady state for an idle,
/// already-authenticated backend); a clean EOF or read error marks it dead
/// so `acquire()` drops and retries (R1a/R1b).
async fn liveness_check(stream: &TcpStream) -> bool {
    match tokio::time::timeout(Duration::from_millis(0), stream.readable()).await {
        Err(_) => true,
        Ok(Err(_)) => false,
        Ok(Ok(())) => {
            let mut probe = [0_u8; 1];
            match stream.try_read(&mut probe) {
                Ok(0) => false,
                Ok(_) => true,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => true,
                Err(_) => false,
            }
        }
    }
}

/// Sends `DISCARD ALL` (pgpool acting as its own client toward the backend)
/// and waits for `ReadyForQuery`, bounded by `reset_timeout`; returns the
/// stream on success or hands it back on failure so the caller can close it
/// (R1c/R1d).
async fn reset_connection(
    mut stream: TcpStream,
    wire: &WireCodecConfig,
    reset_timeout: Duration,
) -> Result<TcpStream, TcpStream> {
    let mut buf = BytesMut::new();
    FrontendMessage::Query(Query {
        sql: "DISCARD ALL".to_string(),
    })
    .encode(&mut buf);
    if stream.write_all(&buf).await.is_err() {
        return Err(stream);
    }

    let mut reader = FrameReader::new(Role::Backend, wire);
    let deadline = Instant::now() + reset_timeout;
    loop {
        match reader.next_frame() {
            Ok(Some(WireMessage::Backend(BackendMessage::ReadyForQuery(_)))) => return Ok(stream),
            Ok(Some(WireMessage::Backend(BackendMessage::ErrorResponse(_)))) => return Err(stream),
            Ok(Some(WireMessage::Backend(_))) => continue,
            Ok(Some(WireMessage::Frontend(_))) => {
                unreachable!("backend-role reader only emits Backend frames")
            }
            Ok(None) => {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    return Err(stream);
                }
                let mut probe = [0_u8; 4096];
                match tokio::time::timeout(remaining, stream.read(&mut probe)).await {
                    Ok(Ok(0)) => return Err(stream),
                    Ok(Ok(n)) => reader.feed(&probe[..n]),
                    Ok(Err(_)) => return Err(stream),
                    Err(_) => return Err(stream),
                }
            }
            Err(_) => return Err(stream),
        }
    }
}
// </HANDWRITE>

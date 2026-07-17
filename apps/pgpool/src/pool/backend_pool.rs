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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::Instant;

use crate::pool::telemetry::{
    TransactionPhase, TransactionPhaseOutcome, TransactionPhaseTelemetry,
    TransactionPhaseTelemetrySnapshot,
};
use crate::pool::types::{
    BackendConnectionId, BackendPoolStats, LeaseDisposition, PoolConfig, PoolError,
};
use crate::pool::{
    ReserveLeaseClient, ReserveLeaseDemand, ReserveLeasePolicy, ReserveLeaseRuntimeConfig,
};
use crate::wire::{
    BackendMessage, FrameReader, FrontendMessage, Role, StartupMessage, WireCodecConfig,
    WireMessage,
};

const MAX_STARTUP_REPLAYS: usize = 64;
/// Byte-exact PostgreSQL simple Query frame for `DISCARD ALL`: tag `Q`,
/// 16-byte length (including its four-byte length field), SQL, and NUL.
const DISCARD_ALL_QUERY_FRAME: &[u8] = b"Q\0\0\0\x10DISCARD ALL\0";

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

/// Startup admission result for transaction pooling. A replay admission has
/// already authenticated the frontend at the protocol layer and therefore
/// intentionally holds no physical backend lease.
#[derive(Debug)]
pub enum StartupAdmission {
    Replay(Vec<BackendMessage>),
    Fresh(BackendLease),
}

#[derive(Debug, Clone)]
struct StartupReplay {
    startup: StartupMessage,
    messages: Vec<BackendMessage>,
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
    startup_replays: Vec<StartupReplay>,
    next_id: u64,
}

#[derive(Debug)]
struct PoolInner {
    permits: Arc<Semaphore>,
    state: Mutex<PoolState>,
    notify: Notify,
    telemetry: Option<Arc<TransactionPhaseTelemetry>>,
    reserve: Option<ReserveRuntime>,
    reactor_stats: ReactorStats,
}

#[derive(Debug)]
struct ReserveRuntime {
    config: ReserveLeaseRuntimeConfig,
    client: ReserveLeaseClient,
}

#[derive(Debug)]
struct ReactorStats {
    enabled: AtomicBool,
    active: AtomicUsize,
    idle: AtomicUsize,
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in backend_pool.rs is hand-written pending codegen support">
/// Shared backend-connection pool: idle-reuse-preferring acquire with a
/// non-blocking liveness peek, always-fresh acquire for the one-time
/// startup+auth handshake, and a disposition-driven release (Logic
/// section). Cheaply `Clone`-able (an `Arc`-backed handle) so both
/// `SessionProxyConfig` and `TransactionProxyConfig` can share one pool.
#[derive(Debug, Clone)]
pub struct BackendPool {
    config: PoolConfig,
    inner: Arc<PoolInner>,
    backend_application_name: Option<Arc<str>>,
}
// </HANDWRITE>

impl BackendPool {
    pub fn new(config: PoolConfig) -> Self {
        Self::new_inner(config, TransactionPhaseTelemetry::from_environment(), None)
    }

    /// Creates a pool with bounded phase telemetry for deterministic tests and
    /// explicitly diagnostic callers.  Ordinary construction remains off by
    /// default unless `PGPOOL_TRANSACTION_PHASE_TELEMETRY` enables it.
    pub fn new_with_transaction_phase_telemetry(config: PoolConfig) -> Self {
        Self::new_inner(config, Some(TransactionPhaseTelemetry::new()), None)
    }

    /// Builds a pool with an endpoint-local reserve cache. The cache may only
    /// ask a background worker for a grant after normal capacity has waited
    /// through the configured reserve timeout; it never performs remote I/O
    /// in `acquire` itself.
    pub fn new_with_reserve(config: PoolConfig, reserve: ReserveLeaseRuntimeConfig) -> Self {
        let client = ReserveLeaseClient::new(reserve.policy);
        Self::new_inner(
            config,
            TransactionPhaseTelemetry::from_environment(),
            Some(ReserveRuntime {
                config: reserve,
                client,
            }),
        )
    }

    fn new_inner(
        config: PoolConfig,
        telemetry: Option<Arc<TransactionPhaseTelemetry>>,
        reserve: Option<ReserveRuntime>,
    ) -> Self {
        let max = config.max_backend_connections;
        Self {
            config,
            backend_application_name: None,
            inner: Arc::new(PoolInner {
                permits: Arc::new(Semaphore::new(max)),
                state: Mutex::new(PoolState {
                    idle: Vec::new(),
                    outstanding: HashMap::new(),
                    startup_replays: Vec::new(),
                    next_id: 0,
                }),
                notify: Notify::new(),
                telemetry,
                reserve,
                reactor_stats: ReactorStats {
                    enabled: AtomicBool::new(false),
                    active: AtomicUsize::new(0),
                    idle: AtomicUsize::new(0),
                },
            }),
        }
    }

    /// Attach the Deployment-scoped identity that pgpool writes into every
    /// backend StartupMessage. Tests and direct library users remain
    /// byte-for-byte pass-through until they opt in.
    pub fn with_backend_application_name(mut self, application_name: impl Into<String>) -> Self {
        let application_name = application_name.into();
        self.backend_application_name = (!application_name.is_empty())
            .then(|| Arc::<str>::from(application_name));
        self
    }

    /// Return the startup identity used for a physical backend connection.
    /// The same normalized value is used by handshake forwarding and replay
    /// cache keys, so client-supplied application names cannot split cache
    /// identity from what PostgreSQL observes.
    pub fn normalize_backend_startup(&self, startup: StartupMessage) -> StartupMessage {
        match &self.backend_application_name {
            Some(application_name) => startup.with_application_name(application_name.as_ref()),
            None => startup,
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

    /// @spec apps/pgpool/tech-design/logic/trust-startup-replay-for-capped-transaction-pooling.md#logic
    /// Admits a transaction client after its startup packet is known. Exact
    /// trust/no-challenge replies bypass a physical backend lease; otherwise
    /// this waits for a fresh connection while rechecking the cache after
    /// every pool notification so capped waiters can observe a just-published
    /// reply instead of timing out behind idle connections.
    pub async fn acquire_for_startup(
        &self,
        startup: &StartupMessage,
    ) -> Result<StartupAdmission, PoolError> {
        let deadline = Instant::now() + self.config.acquire_timeout;
        loop {
            if let Some(reply) = self.startup_replay(startup) {
                return Ok(StartupAdmission::Replay(reply));
            }

            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                return self
                    .connect_fresh(permit)
                    .await
                    .map(StartupAdmission::Fresh);
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
        }
    }

    /// Acquires a backend for a frontend that was established from a safe
    /// startup replay. Idle backends are already authenticated. If concurrency
    /// needs another physical connection, bootstrap it with the same startup
    /// locally before returning it so query traffic never reaches an
    /// unauthenticated PostgreSQL socket.
    pub async fn acquire_for_replayed_startup(
        &self,
        startup: &StartupMessage,
    ) -> Result<BackendLease, PoolError> {
        let deadline = Instant::now() + self.config.acquire_timeout;
        loop {
            if let Some(lease) = self.try_take_idle().await {
                return Ok(lease);
            }

            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                let lease = self.connect_fresh(permit).await?;
                return match bootstrap_no_challenge(
                    lease,
                    startup,
                    &self.config.wire,
                    self.config.backend_connect_timeout,
                )
                .await
                {
                    Ok(lease) => Ok(lease),
                    Err(lease) => {
                        drop(lease);
                        self.inner.notify.notify_waiters();
                        Err(PoolError::BackendUnreachable(
                            "backend did not accept replayed trust startup".to_string(),
                        ))
                    }
                };
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
        }
    }

    /// @spec apps/pgpool/tech-design/logic/trust-startup-replay-for-capped-transaction-pooling.md#logic
    /// Publishes one complete, challenge-free startup response. Existing
    /// entries are never overwritten, and the bounded cache only accepts
    /// exact startup identities.
    pub fn publish_startup_replay(&self, startup: StartupMessage, messages: Vec<BackendMessage>) {
        if messages.is_empty() {
            return;
        }

        let inserted = {
            let mut state = self.inner.state.lock().expect("pool state lock");
            if state
                .startup_replays
                .iter()
                .any(|entry| entry.startup == startup)
            {
                false
            } else if state.startup_replays.len() < MAX_STARTUP_REPLAYS {
                state
                    .startup_replays
                    .push(StartupReplay { startup, messages });
                true
            } else {
                false
            }
        };
        if inserted {
            self.inner.notify.notify_waiters();
        }
    }

    async fn acquire_internal(&self, allow_idle_reuse: bool) -> Result<BackendLease, PoolError> {
        let queue_wait_timeout = self
            .inner
            .reserve
            .as_ref()
            .map(|runtime| Duration::from_secs(runtime.config.policy.queue_wait_timeout_seconds))
            .unwrap_or(self.config.acquire_timeout);
        let deadline = Instant::now() + queue_wait_timeout;
        let reserve_deadline = self.inner.reserve.as_ref().map(|runtime| {
            Instant::now() + Duration::from_secs(runtime.config.policy.reserve_pool_timeout_seconds)
        });
        let mut reserve_demanded = false;
        loop {
            if allow_idle_reuse {
                if let Some(lease) = self.try_take_idle().await {
                    return Ok(lease);
                }
            }

            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                return self.connect_fresh(permit).await;
            }

            if !reserve_demanded && reserve_deadline.is_some_and(|at| Instant::now() >= at) {
                if let Some(runtime) = &self.inner.reserve {
                    runtime.client.queue_demand(ReserveLeaseDemand {
                        endpoint: runtime.config.endpoint.clone(),
                        pod: runtime.config.pod.clone(),
                        units: 1,
                    });
                    reserve_demanded = true;
                }
            }

            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(self.saturated_after(queue_wait_timeout));
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
        self.saturated_after(self.config.acquire_timeout)
    }

    fn saturated_after(&self, waited: Duration) -> PoolError {
        PoolError::Saturated {
            max: self.config.max_backend_connections,
            waited,
        }
    }

    fn startup_replay(&self, startup: &StartupMessage) -> Option<Vec<BackendMessage>> {
        let state = self.inner.state.lock().expect("pool state lock");
        state
            .startup_replays
            .iter()
            .find(|entry| &entry.startup == startup)
            .map(|entry| entry.messages.clone())
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
        let release_started = self.transaction_phase_started_at();

        let outcome = match disposition {
            LeaseDisposition::Close => {
                let mut stream = stream;
                let _ = stream.shutdown().await;
                drop(stream);
                drop(permit);
                self.inner.notify.notify_waiters();
                TransactionPhaseOutcome::Success
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
                        TransactionPhaseOutcome::Success
                    }
                    Err(mut stream) => {
                        let _ = stream.shutdown().await;
                        drop(stream);
                        drop(permit);
                        self.inner.notify.notify_waiters();
                        TransactionPhaseOutcome::Failure
                    }
                }
            }
        };
        self.record_transaction_phase_started(TransactionPhase::Release, outcome, release_started);
    }

    pub fn stats(&self) -> BackendPoolStats {
        let reserve = self
            .inner
            .reserve
            .as_ref()
            .map(|runtime| runtime.client.stats())
            .unwrap_or_default();
        if self.inner.reactor_stats.enabled.load(Ordering::Acquire) {
            return BackendPoolStats {
                backend_active: self.inner.reactor_stats.active.load(Ordering::Relaxed),
                backend_idle: self.inner.reactor_stats.idle.load(Ordering::Relaxed),
                reserve_queued: reserve.queued_units as usize,
                reserve_granted: reserve.granted_units as usize,
                reserve_spent: reserve.spent_units as usize,
            };
        }
        let state = self.inner.state.lock().expect("pool state lock");
        BackendPoolStats {
            backend_active: state.outstanding.len(),
            backend_idle: state.idle.len(),
            reserve_queued: reserve.queued_units as usize,
            reserve_granted: reserve.granted_units as usize,
            reserve_spent: reserve.spent_units as usize,
        }
    }

    /// Clone the reserve cache for the background control-plane worker. The
    /// transaction reactor and legacy handler only share its local snapshot.
    pub fn reserve_client(&self) -> Option<ReserveLeaseClient> {
        self.inner
            .reserve
            .as_ref()
            .map(|runtime| runtime.client.clone())
    }

    pub(crate) fn reserve_policy(&self) -> Option<ReserveLeasePolicy> {
        self.inner
            .reserve
            .as_ref()
            .map(|runtime| runtime.config.policy)
    }

    pub(crate) fn publish_reactor_stats(&self, active: usize, idle: usize) {
        self.inner
            .reactor_stats
            .active
            .store(active, Ordering::Relaxed);
        self.inner.reactor_stats.idle.store(idle, Ordering::Relaxed);
        self.inner
            .reactor_stats
            .enabled
            .store(true, Ordering::Release);
    }

    pub(crate) fn reactor_config(&self) -> PoolConfig {
        self.config.clone()
    }

    /// Returns the fixed-cardinality diagnostic snapshot only when the pool
    /// was explicitly started with transaction phase telemetry enabled.
    pub fn transaction_phase_telemetry(&self) -> Option<TransactionPhaseTelemetrySnapshot> {
        self.inner
            .telemetry
            .as_ref()
            .map(|telemetry| telemetry.snapshot())
    }

    pub(crate) fn record_transaction_phase(
        &self,
        phase: TransactionPhase,
        outcome: TransactionPhaseOutcome,
        elapsed: Duration,
    ) {
        if let Some(telemetry) = &self.inner.telemetry {
            telemetry.record(phase, outcome, elapsed);
        }
    }

    /// Captures an `Instant` only while the explicitly opt-in diagnostic
    /// telemetry is active, keeping the ordinary release path free of timing
    /// calls and atomic updates.
    pub(crate) fn transaction_phase_started_at(&self) -> Option<Instant> {
        self.inner.telemetry.as_ref().map(|_| Instant::now())
    }

    pub(crate) fn record_transaction_phase_started(
        &self,
        phase: TransactionPhase,
        outcome: TransactionPhaseOutcome,
        started_at: Option<Instant>,
    ) {
        if let Some(started_at) = started_at {
            self.record_transaction_phase(phase, outcome, started_at.elapsed());
        }
    }

    /// Number of bounded, exact trust/no-challenge startup replies currently
    /// available for transaction admission. This is intentionally read-only;
    /// cache entries are created only by a completed backend handshake.
    pub fn startup_replay_count(&self) -> usize {
        let state = self.inner.state.lock().expect("pool state lock");
        state.startup_replays.len()
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

        if let Err(error) = stream.set_nodelay(true) {
            drop(stream);
            drop(permit);
            self.inner.notify.notify_waiters();
            return Err(PoolError::BackendUnreachable(format!(
                "backend connect to {addr} could not enable TCP_NODELAY: {error}"
            )));
        }

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

/// Non-consuming liveness peek (R1): an idle connection with no pending read
/// readiness is presumed alive (the expected steady state for an idle,
/// already-authenticated backend); a clean EOF or read error marks it dead
/// so `acquire()` drops and retries (R1a/R1b). A readable protocol frame is
/// deliberately left queued for the normal relay -- consuming even one byte
/// here would desynchronize the PostgreSQL frame stream.
async fn liveness_check(stream: &TcpStream) -> bool {
    let mut probe = [0_u8; 1];
    match tokio::time::timeout(Duration::from_millis(0), stream.peek(&mut probe)).await {
        Err(_) => true,
        Ok(Err(_)) => false,
        Ok(Ok(0)) => false,
        Ok(Ok(_)) => true,
    }
}

/// Completes a backend-only startup for a frontend already admitted by a
/// cached no-challenge reply. No client frame is available here, so every
/// password/MD5/SASL challenge is a hard failure and the socket is discarded.
async fn bootstrap_no_challenge(
    mut lease: BackendLease,
    startup: &StartupMessage,
    wire: &WireCodecConfig,
    read_timeout: Duration,
) -> Result<BackendLease, BackendLease> {
    let mut outbound = BytesMut::new();
    FrontendMessage::Startup(startup.clone()).encode(&mut outbound);
    if lease.stream.write_all(&outbound).await.is_err() {
        return Err(lease);
    }

    let mut reader = FrameReader::new(Role::Backend, wire);
    let mut saw_authentication_ok = false;
    loop {
        match reader.next_frame() {
            Ok(Some(WireMessage::Backend(BackendMessage::AuthenticationOk(_)))) => {
                saw_authentication_ok = true;
            }
            Ok(Some(WireMessage::Backend(BackendMessage::ReadyForQuery(_))))
                if saw_authentication_ok =>
            {
                return Ok(lease);
            }
            Ok(Some(WireMessage::Backend(
                BackendMessage::AuthenticationCleartextPassword(_)
                | BackendMessage::AuthenticationMd5Password(_)
                | BackendMessage::AuthenticationSasl(_)
                | BackendMessage::AuthenticationSaslContinue(_)
                | BackendMessage::AuthenticationSaslFinal(_)
                | BackendMessage::ErrorResponse(_),
            )))
            | Ok(Some(WireMessage::Frontend(_)))
            | Err(_) => return Err(lease),
            Ok(Some(WireMessage::Backend(_))) => {}
            Ok(None) => {
                match tokio::time::timeout(read_timeout, reader.read_from(&mut lease.stream)).await
                {
                    Ok(Ok(0)) | Ok(Err(_)) | Err(_) => return Err(lease),
                    Ok(Ok(_)) => {}
                }
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
    if stream.write_all(DISCARD_ALL_QUERY_FRAME).await.is_err() {
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
                match tokio::time::timeout(remaining, reader.read_from(&mut stream)).await {
                    Ok(Ok(0)) => return Err(stream),
                    Ok(Ok(_)) => {}
                    Ok(Err(_)) => return Err(stream),
                    Err(_) => return Err(stream),
                }
            }
            Err(_) => return Err(stream),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::DISCARD_ALL_QUERY_FRAME;
    use crate::wire::{FrontendMessage, Query};

    #[test]
    fn discard_all_static_frame_matches_the_wire_encoder() {
        let mut encoded = BytesMut::new();
        FrontendMessage::Query(Query {
            sql: "DISCARD ALL".to_string(),
        })
        .encode(&mut encoded);

        assert_eq!(DISCARD_ALL_QUERY_FRAME, encoded.as_ref());
    }
}
// </HANDWRITE>

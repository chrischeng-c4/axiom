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

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Notify, OwnedSemaphorePermit, Semaphore};
use tokio::time::{sleep_until, Instant};

use crate::pool::types::{
    BackendConnectionId, BackendPoolStats, LeaseDisposition, PoolConfig, PoolError,
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
            self.inner
                .capacity_waiters
                .capacity_available(CapacityAvailability::Physical);
        }
    }
}

/// Pool-owned capacity admission. Frontends wait on a lightweight oneshot;
/// one background driver owns the only Tokio timer for the queue's earliest
/// deadline, avoiding a `Sleep` registration per saturated client.
#[derive(Debug)]
struct CapacityWaiters {
    state: Mutex<CapacityWaiterState>,
    changed: Arc<Notify>,
    driver_started: AtomicBool,
}

#[derive(Debug, Default)]
struct CapacityWaiterState {
    queue: VecDeque<CapacityWaiter>,
    in_flight_grants: HashMap<u64, CapacityAvailability>,
    next_id: u64,
}

#[derive(Debug)]
struct CapacityWaiter {
    id: u64,
    deadline: Instant,
    needs_fresh: bool,
    tx: oneshot::Sender<CapacityWaiterResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapacityWaiterResult {
    Granted(CapacityAvailability),
    Expired,
}

/// A reset-clean idle stream satisfies a reusable acquisition, while only a
/// dropped physical permit can satisfy a startup/session fresh connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapacityAvailability {
    Reusable,
    Physical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapacityWaitError {
    Expired,
    Closed,
}

#[derive(Debug)]
struct CapacityTicket {
    waiters: Arc<CapacityWaiters>,
    id: u64,
    deadline: Instant,
    rx: oneshot::Receiver<CapacityWaiterResult>,
    armed: bool,
}

#[derive(Debug)]
struct CapacityGrant {
    waiters: Arc<CapacityWaiters>,
    id: u64,
    availability: CapacityAvailability,
    active: bool,
}

impl CapacityWaiters {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(CapacityWaiterState::default()),
            changed: Arc::new(Notify::new()),
            driver_started: AtomicBool::new(false),
        })
    }

    /// New callers may use an immediately visible idle/backend permit only
    /// while no FIFO handoff is queued or in progress.
    fn may_try_immediate(&self) -> bool {
        let state = self.state.lock().expect("capacity waiter lock");
        state.queue.is_empty() && state.in_flight_grants.is_empty()
    }

    fn enqueue(self: &Arc<Self>, deadline: Instant, needs_fresh: bool) -> CapacityTicket {
        self.ensure_driver();
        let (tx, rx) = oneshot::channel();
        let id = {
            let mut state = self.state.lock().expect("capacity waiter lock");
            let id = state.next_id;
            state.next_id += 1;
            state.queue.push_back(CapacityWaiter {
                id,
                deadline,
                needs_fresh,
                tx,
            });
            id
        };
        self.changed.notify_one();
        CapacityTicket {
            waiters: Arc::clone(self),
            id,
            deadline,
            rx,
            armed: true,
        }
    }

    fn ensure_driver(self: &Arc<Self>) {
        if self
            .driver_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let weak = Arc::downgrade(self);
            tokio::spawn(async move {
                Self::drive_deadlines(weak).await;
            });
        }
    }

    async fn drive_deadlines(weak: std::sync::Weak<Self>) {
        loop {
            let Some(waiters) = weak.upgrade() else {
                return;
            };
            let changed = Arc::clone(&waiters.changed);
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let deadline = waiters.earliest_deadline();
            drop(waiters);

            match deadline {
                Some(deadline) if deadline <= Instant::now() => {
                    if let Some(waiters) = weak.upgrade() {
                        waiters.expire_due();
                    }
                }
                Some(deadline) => {
                    tokio::select! {
                        _ = sleep_until(deadline) => {
                            if let Some(waiters) = weak.upgrade() {
                                waiters.expire_due();
                            }
                        }
                        _ = &mut notified => {}
                    }
                }
                None => notified.await,
            }
        }
    }

    fn earliest_deadline(&self) -> Option<Instant> {
        let state = self.state.lock().expect("capacity waiter lock");
        state.queue.iter().map(|waiter| waiter.deadline).min()
    }

    fn expire_due(&self) {
        let now = Instant::now();
        let expired = {
            let mut state = self.state.lock().expect("capacity waiter lock");
            let mut expired = Vec::new();
            while let Some(waiter) = state.queue.front() {
                if waiter.deadline > now {
                    break;
                }
                expired.push(state.queue.pop_front().expect("front exists"));
            }
            expired
        };
        for waiter in expired {
            let _ = waiter.tx.send(CapacityWaiterResult::Expired);
        }
        self.changed.notify_one();
    }

    /// Returns true only when a resolved grant was abandoned before its
    /// caller committed a backend, which leaves one capacity slot to hand on.
    fn cancel(&self, id: u64) -> Option<CapacityAvailability> {
        let abandoned_grant = {
            let mut state = self.state.lock().expect("capacity waiter lock");
            if let Some(index) = state.queue.iter().position(|waiter| waiter.id == id) {
                state.queue.remove(index);
                None
            } else {
                state.in_flight_grants.remove(&id)
            }
        };
        self.changed.notify_one();
        abandoned_grant
    }

    /// Call only after an idle stream has been parked or a physical permit has
    /// been dropped. A failed receiver leaves the same slot available, so the
    /// next live FIFO waiter is granted immediately instead.
    fn capacity_available(&self, availability: CapacityAvailability) {
        self.expire_due();
        let grant = {
            let mut state = self.state.lock().expect("capacity waiter lock");
            let grant = state
                .queue
                .iter()
                .position(|waiter| {
                    availability == CapacityAvailability::Physical || !waiter.needs_fresh
                })
                .and_then(|index| state.queue.remove(index));
            if let Some(waiter) = grant.as_ref() {
                state.in_flight_grants.insert(waiter.id, availability);
            }
            grant
        };
        let Some(grant) = grant else {
            self.changed.notify_one();
            return;
        };
        if grant
            .tx
            .send(CapacityWaiterResult::Granted(availability))
            .is_err()
        {
            self.finish_grant(grant.id, None);
            self.capacity_available(availability);
        }
    }

    fn finish_grant(&self, id: u64, handoff_capacity: Option<CapacityAvailability>) {
        let removed = {
            let mut state = self.state.lock().expect("capacity waiter lock");
            state.in_flight_grants.remove(&id)
        };
        if removed.is_none() {
            return;
        }
        if let Some(availability) = handoff_capacity {
            self.capacity_available(availability);
        } else {
            self.changed.notify_one();
        }
    }
}

impl CapacityTicket {
    async fn wait(&mut self) -> Result<CapacityGrant, CapacityWaitError> {
        let result = (&mut self.rx).await;
        self.armed = false;
        match result {
            Ok(CapacityWaiterResult::Granted(availability)) if Instant::now() < self.deadline => {
                Ok(CapacityGrant {
                    waiters: Arc::clone(&self.waiters),
                    id: self.id,
                    availability,
                    active: true,
                })
            }
            Ok(CapacityWaiterResult::Granted(availability)) => {
                self.waiters.finish_grant(self.id, Some(availability));
                Err(CapacityWaitError::Expired)
            }
            Ok(CapacityWaiterResult::Expired) => Err(CapacityWaitError::Expired),
            Err(_) => Err(CapacityWaitError::Closed),
        }
    }
}

impl Drop for CapacityTicket {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Some(availability) = self.waiters.cancel(self.id) {
            self.waiters.capacity_available(availability);
        }
    }
}

impl CapacityGrant {
    fn consume(&mut self) {
        if self.active {
            self.active = false;
            self.waiters.finish_grant(self.id, None);
        }
    }

    fn handoff_capacity(&mut self) {
        if self.active {
            self.active = false;
            self.waiters.finish_grant(self.id, Some(self.availability));
        }
    }
}

impl Drop for CapacityGrant {
    fn drop(&mut self) {
        self.handoff_capacity();
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
    capacity_waiters: Arc<CapacityWaiters>,
    state: Mutex<PoolState>,
    replay_notify: Notify,
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
                capacity_waiters: CapacityWaiters::new(),
                state: Mutex::new(PoolState {
                    idle: Vec::new(),
                    outstanding: HashMap::new(),
                    startup_replays: Vec::new(),
                    next_id: 0,
                }),
                replay_notify: Notify::new(),
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

    /// @spec apps/pgpool/tech-design/logic/trust-startup-replay-for-capped-transaction-pooling.md#logic
    /// Admits a transaction client after its startup packet is known. Exact
    /// trust/no-challenge replies bypass a physical backend lease; otherwise
    /// this waits for a fresh connection through the FIFO capacity scheduler
    /// while observing the separate replay-cache broadcast so capped waiters
    /// can consume a just-published reply without a physical lease.
    pub async fn acquire_for_startup(
        &self,
        startup: &StartupMessage,
    ) -> Result<StartupAdmission, PoolError> {
        let deadline = Instant::now() + self.config.acquire_timeout;
        loop {
            if let Some(reply) = self.startup_replay(startup) {
                return Ok(StartupAdmission::Replay(reply));
            }

            if self.inner.capacity_waiters.may_try_immediate() {
                if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                    return self
                        .connect_fresh(permit)
                        .await
                        .map(StartupAdmission::Fresh);
                }
            }

            if Instant::now() >= deadline {
                return Err(self.saturated());
            }

            let replay_notified = self.inner.replay_notify.notified();
            tokio::pin!(replay_notified);
            replay_notified.as_mut().enable();
            if let Some(reply) = self.startup_replay(startup) {
                return Ok(StartupAdmission::Replay(reply));
            }

            let mut ticket = self.inner.capacity_waiters.enqueue(deadline, true);
            tokio::select! {
                result = ticket.wait() => {
                    let mut grant = result.map_err(|_| self.saturated())?;
                    if let Some(reply) = self.startup_replay(startup) {
                        grant.handoff_capacity();
                        return Ok(StartupAdmission::Replay(reply));
                    }
                    if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                        let result = self.connect_fresh(permit).await;
                        grant.consume();
                        return result.map(StartupAdmission::Fresh);
                    }
                    grant.handoff_capacity();
                }
                _ = &mut replay_notified => {}
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
            if self.inner.capacity_waiters.may_try_immediate() {
                if let Some(lease) = self.try_take_idle().await {
                    return Ok(lease);
                }
                if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                    return self.bootstrap_replayed_startup(permit, startup).await;
                }
            }

            if Instant::now() >= deadline {
                return Err(self.saturated());
            }

            let mut grant = self
                .inner
                .capacity_waiters
                .enqueue(deadline, false)
                .wait()
                .await
                .map_err(|_| self.saturated())?;
            if let Some(lease) = self.try_take_idle().await {
                grant.consume();
                return Ok(lease);
            }
            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                let result = self.bootstrap_replayed_startup(permit, startup).await;
                grant.consume();
                return result;
            }
            grant.handoff_capacity();
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
            self.inner.replay_notify.notify_waiters();
        }
    }

    async fn acquire_internal(&self, allow_idle_reuse: bool) -> Result<BackendLease, PoolError> {
        let deadline = Instant::now() + self.config.acquire_timeout;
        loop {
            if self.inner.capacity_waiters.may_try_immediate() {
                if allow_idle_reuse {
                    if let Some(lease) = self.try_take_idle().await {
                        return Ok(lease);
                    }
                }

                if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                    return self.connect_fresh(permit).await;
                }
            }

            if Instant::now() >= deadline {
                return Err(self.saturated());
            }

            let mut grant = self
                .inner
                .capacity_waiters
                .enqueue(deadline, !allow_idle_reuse)
                .wait()
                .await
                .map_err(|_| self.saturated())?;
            if allow_idle_reuse {
                if let Some(lease) = self.try_take_idle().await {
                    grant.consume();
                    return Ok(lease);
                }
            }
            if let Ok(permit) = Arc::clone(&self.inner.permits).try_acquire_owned() {
                let result = self.connect_fresh(permit).await;
                grant.consume();
                return result;
            }
            grant.handoff_capacity();
        }
    }

    async fn bootstrap_replayed_startup(
        &self,
        permit: OwnedSemaphorePermit,
        startup: &StartupMessage,
    ) -> Result<BackendLease, PoolError> {
        let lease = self.connect_fresh(permit).await?;
        match bootstrap_no_challenge(
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
                Err(PoolError::BackendUnreachable(
                    "backend did not accept replayed trust startup".to_string(),
                ))
            }
        }
    }

    fn saturated(&self) -> PoolError {
        PoolError::Saturated {
            max: self.config.max_backend_connections,
            waited: self.config.acquire_timeout,
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

        match disposition {
            LeaseDisposition::Close => {
                let mut stream = stream;
                let _ = stream.shutdown().await;
                drop(stream);
                drop(permit);
                self.inner
                    .capacity_waiters
                    .capacity_available(CapacityAvailability::Physical);
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
                        self.inner
                            .capacity_waiters
                            .capacity_available(CapacityAvailability::Reusable);
                    }
                    Err(mut stream) => {
                        let _ = stream.shutdown().await;
                        drop(stream);
                        drop(permit);
                        self.inner
                            .capacity_waiters
                            .capacity_available(CapacityAvailability::Physical);
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
                self.inner
                    .capacity_waiters
                    .capacity_available(CapacityAvailability::Physical);
                return Err(PoolError::BackendUnreachable(error.to_string()));
            }
            Err(_) => {
                drop(permit);
                self.inner
                    .capacity_waiters
                    .capacity_available(CapacityAvailability::Physical);
                return Err(PoolError::BackendUnreachable(format!(
                    "backend connect to {addr} timed out after {:?}",
                    self.config.backend_connect_timeout
                )));
            }
        };

        if let Err(error) = stream.set_nodelay(true) {
            drop(stream);
            drop(permit);
            self.inner
                .capacity_waiters
                .capacity_available(CapacityAvailability::Physical);
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

    use super::{CapacityAvailability, CapacityWaiters, DISCARD_ALL_QUERY_FRAME};
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

    #[tokio::test]
    async fn capacity_waiters_grant_fifo_and_skip_cancelled_ticket() {
        let waiters = CapacityWaiters::new();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let mut first = waiters.enqueue(deadline, false);
        let second = waiters.enqueue(deadline, false);
        let mut third = waiters.enqueue(deadline, false);
        drop(second);

        waiters.capacity_available(CapacityAvailability::Physical);
        let mut first_grant = first.wait().await.expect("first ticket is granted");
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), third.wait())
                .await
                .is_err(),
            "one available slot must not grant a second waiter"
        );

        first_grant.consume();
        waiters.capacity_available(CapacityAvailability::Physical);
        let mut third_grant = third.wait().await.expect("cancelled ticket is skipped");
        third_grant.consume();
    }

    #[tokio::test]
    async fn expired_capacity_ticket_never_consumes_a_grant() {
        let waiters = CapacityWaiters::new();
        let mut ticket = waiters.enqueue(tokio::time::Instant::now(), false);
        waiters.capacity_available(CapacityAvailability::Physical);
        assert!(
            ticket.wait().await.is_err(),
            "an expired ticket must not receive the available slot"
        );
        assert!(waiters.may_try_immediate());
    }

    #[tokio::test]
    async fn reusable_capacity_skips_fresh_waiter_until_physical_slot_frees() {
        let waiters = CapacityWaiters::new();
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let mut fresh = waiters.enqueue(deadline, true);
        let mut reusable = waiters.enqueue(deadline, false);

        waiters.capacity_available(CapacityAvailability::Reusable);
        let mut reusable_grant = reusable
            .wait()
            .await
            .expect("idle backend must wake reusable acquisition");
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), fresh.wait())
                .await
                .is_err(),
            "an idle backend cannot grant a fresh-only startup/session waiter"
        );
        reusable_grant.consume();

        waiters.capacity_available(CapacityAvailability::Physical);
        let mut fresh_grant = fresh
            .wait()
            .await
            .expect("dropped physical permit wakes fresh waiter");
        fresh_grant.consume();
    }
}
// </HANDWRITE>

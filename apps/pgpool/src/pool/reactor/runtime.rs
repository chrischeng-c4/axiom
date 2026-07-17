// SPEC-MANAGED: apps/pgpool/tech-design/logic/p0-dense-buffer-readiness-reactor.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-readiness-reactor" tracker="#1753" reason="The transaction reactor owns a socket-readiness state machine not yet expressible by the service generator.">
//! `mio` ingress for the transaction readiness owner.
//!
//! Accepted frontend sockets cross this boundary exactly once. The Tokio
//! accept task retains no per-transaction state after `handoff`; a single
//! reactor thread will own all subsequent frontend/backend readiness and the
//! accompanying [`super::state::ReactorState`].

use std::collections::VecDeque;
use std::io::{self, IoSlice, Write};
use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bytes::{Bytes, BytesMut};
use mio::net::TcpStream as MioTcpStream;
use mio::{Interest, Poll, Token, Waker};
use server_lifecycle::ConnectionPermit;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::oneshot;

use crate::pool::reactor::state::{BackendId, ClientId, ReactorState, TransactionAction};
use crate::pool::{BackendPool, PoolConfig};
use crate::wire::{
    BackendKeyData, BackendMessage, FrameReader, FrontendMessage, RelayFrame, RelayFrameKind, Role,
    StartupMessage, TransactionStatus, WireFrame, WireMessage,
};

const INGRESS_TOKEN: Token = Token(0);
const FIRST_SOCKET_TOKEN: usize = 1;
const DISCARD_ALL_QUERY_FRAME: &[u8] = b"Q\0\0\0\x10DISCARD ALL\0";

/// A frontend accepted by the shared Tokio listener but not yet registered
/// with the transaction readiness owner.
struct IncomingFrontend {
    stream: std::net::TcpStream,
    permit: ConnectionPermit,
    completion: oneshot::Sender<()>,
}

struct Ingress {
    pending: Mutex<VecDeque<IncomingFrontend>>,
    wake: Arc<Waker>,
    stopping: AtomicBool,
}

/// Cloneable, one-shot handoff handle owned by [`crate::pool::TransactionHandler`].
///
/// It intentionally exposes no acquire/release API: those operations belong
/// to the reactor's single owner, not the Tokio task that accepted a client.
#[derive(Clone)]
pub(crate) struct TransactionReactor {
    ingress: Arc<Ingress>,
}

impl std::fmt::Debug for TransactionReactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionReactor").finish_non_exhaustive()
    }
}

impl TransactionReactor {
// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="Resolve the backend endpoint before spawning the readiness thread, cache its SocketAddr, and recycle tokens after failed Mio registration.">
    /// Starts the dedicated readiness owner. Backend pool configuration and
    /// admin-stat publication are captured here; no transaction socket has
    /// crossed the boundary yet.
    pub(crate) fn start(pool: BackendPool) -> io::Result<Self> {
        let poll = Poll::new()?;
        let wake = Arc::new(Waker::new(poll.registry(), INGRESS_TOKEN)?);
        let ingress = Arc::new(Ingress {
            pending: Mutex::new(VecDeque::new()),
            wake,
            stopping: AtomicBool::new(false),
        });
        let thread_ingress = Arc::clone(&ingress);
        thread::Builder::new()
            .name("pgpool-transaction-reactor".to_string())
            .spawn(move || ReactorRuntime::new(poll, thread_ingress, pool).run())?;
        Ok(Self { ingress })
    }
// </HANDWRITE>

    /// Transfers the accepted socket and its frontend budget permit to the
    /// readiness owner. `TcpStream::into_std` preserves nonblocking mode; the
    /// reactor resets it explicitly before `mio` registration so this remains
    /// correct if Tokio's conversion semantics ever change.
    pub(crate) fn handoff(
        &self,
        stream: TokioTcpStream,
        permit: ConnectionPermit,
    ) -> io::Result<oneshot::Receiver<()>> {
        let stream = stream.into_std()?;
        stream.set_nonblocking(true)?;
        let (completion, done) = oneshot::channel();
        {
            let mut pending = self.ingress.pending.lock().expect("reactor ingress lock");
            pending.push_back(IncomingFrontend {
                stream,
                permit,
                completion,
            });
        }
        self.ingress.wake.wake()?;
        Ok(done)
    }
}

impl Drop for TransactionReactor {
    fn drop(&mut self) {
        // The runtime keeps one ingress Arc. `server_tcp::serve_arc` now
        // retains its handler through drain, so seeing only this final
        // handler plus the runtime means every frontend completion task has
        // finished and the owner can stop without truncating an in-flight
        // transaction.
        if Arc::strong_count(&self.ingress) == 2 {
            self.ingress.stopping.store(true, Ordering::Release);
            let _ = self.ingress.wake.wake();
        }
    }
}

struct ReactorRuntime {
    poll: Poll,
    ingress: Arc<Ingress>,
    pool: BackendPool,
    config: PoolConfig,
    events: mio::Events,
    ready_events: Vec<(Token, bool, bool)>,
    dirty_client_interests: Vec<Token>,
    dirty_backend_interests: Vec<Token>,
    next_token: usize,
    free_tokens: Vec<usize>,
    retired_tokens: Vec<usize>,
    next_deadline_epoch: u64,
    clients: TokenSlots<ReactorClient>,
    backends: TokenSlots<ReactorBackend>,
    deadlines: VecDeque<(Instant, usize, u64)>,
    state: ReactorState,
    startup_replays: Vec<StartupReplay>,
    startup_waiters: VecDeque<ClientId>,
    stats_dirty: bool,
}

impl ReactorRuntime {
    // <HANDWRITE gap="missing-generator:logic" tracker="#1891" reason="Use the Duration queue policy directly in reactor wait deadlines.">
    /// The reactor owns FIFO deadlines. When reserve leasing is configured,
    /// its queueWaitTimeout replaces the historical local acquire timeout;
    /// reserve grants are still consumed only from the background-updated
    /// pool cache and never by a readiness callback doing Kubernetes I/O.
    fn queue_wait_timeout(&self) -> Duration {
        self.pool
            .reserve_policy()
            .map(|policy| policy.queue_wait_timeout)
            .unwrap_or(self.config.acquire_timeout)
    }
    // </HANDWRITE>
}

struct TokenSlots<T> {
    slots: Vec<Option<T>>,
    len: usize,
}

impl<T> TokenSlots<T> {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
        }
    }

    fn get(&self, token: &Token) -> Option<&T> {
        self.slots.get(token.0).and_then(Option::as_ref)
    }

    fn get_mut(&mut self, token: &Token) -> Option<&mut T> {
        self.slots.get_mut(token.0).and_then(Option::as_mut)
    }

    fn contains_key(&self, token: &Token) -> bool {
        self.get(token).is_some()
    }

    fn insert(&mut self, token: Token, value: T) -> Option<T> {
        if self.slots.len() <= token.0 {
            self.slots.resize_with(token.0 + 1, || None);
        }
        let old = self.slots[token.0].replace(value);
        if old.is_none() {
            self.len += 1;
        }
        old
    }

    fn remove(&mut self, token: &Token) -> Option<T> {
        let value = self.slots.get_mut(token.0).and_then(Option::take);
        if value.is_some() {
            self.len -= 1;
        }
        value
    }

    fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(Option::as_ref)
    }

    fn len(&self) -> usize {
        self.len
    }
}

struct ReactorClient {
    stream: MioTcpStream,
    reader: FrameReader,
    _permit: ConnectionPermit,
    completion: oneshot::Sender<()>,
    mode: ClientMode,
    startup: Option<StartupMessage>,
    pending_first: Option<RelayFrame>,
    output: OutputQueue,
    wait_deadline: Option<Instant>,
    deadline_epoch: u64,
    interest_dirty: bool,
    registered: bool,
}

#[derive(Debug)]
enum ClientMode {
    Startup,
    StartupWaiting,
    Handshaking {
        backend: BackendId,
        awaiting_auth: bool,
    },
    Idle,
    Waiting,
    Active {
        backend: BackendId,
        pending_next: Option<RelayFrame>,
        /// A `ReadyForQuery(InTransaction)` is waiting for its next frontend
        /// frame. That frame belongs to this same backend immediately.
        backend_ready_for_frontend: bool,
    },
    Closing,
}

struct ReactorBackend {
    stream: MioTcpStream,
    reader: FrameReader,
    mode: BackendMode,
    output: OutputQueue,
    interest_dirty: bool,
    registered: bool,
}

#[derive(Debug)]
enum BackendMode {
    Connecting(ConnectPurpose),
    InitialHandshake {
        client: ClientId,
        replay: StartupCapture,
    },
    Bootstrap {
        client: ClientId,
        saw_auth_ok: bool,
    },
    Active {
        client: ClientId,
    },
    Resetting,
    Idle,
}

struct OutputQueue {
    chunks: VecDeque<Bytes>,
    front_offset: usize,
}

impl OutputQueue {
    fn new() -> Self {
        Self {
            chunks: VecDeque::new(),
            front_offset: 0,
        }
    }

    fn push(&mut self, bytes: Bytes) {
        if !bytes.is_empty() {
            self.chunks.push_back(bytes);
        }
    }

    fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    fn advance(&mut self, mut written: usize) {
        while written > 0 {
            let remaining = self
                .chunks
                .front()
                .map(|chunk| chunk.len() - self.front_offset)
                .expect("write cannot exceed queued bytes");
            if written < remaining {
                self.front_offset += written;
                return;
            }
            written -= remaining;
            self.chunks.pop_front();
            self.front_offset = 0;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BackendFrameTarget {
    Initial(ClientId),
    Bootstrap(ClientId),
    Active(ClientId),
    Resetting,
    Idle,
}

#[derive(Debug, Clone)]
enum ConnectPurpose {
    Initial { client: ClientId, startup: Bytes },
    Bootstrap { client: ClientId, startup: Bytes },
}

#[derive(Debug)]
struct StartupCapture {
    frames: Vec<Vec<u8>>,
    messages: Vec<BackendMessage>,
    saw_auth_ok: bool,
    replayable: bool,
}

impl Default for StartupCapture {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            messages: Vec::new(),
            saw_auth_ok: false,
            replayable: true,
        }
    }
}

#[derive(Debug, Clone)]
struct StartupReplay {
    startup: StartupMessage,
    frames: Vec<Vec<u8>>,
}

impl ReactorRuntime {
    fn new(poll: Poll, ingress: Arc<Ingress>, pool: BackendPool) -> Self {
        Self {
            poll,
            ingress,
            config: pool.reactor_config(),
            pool,
            events: mio::Events::with_capacity(256),
            ready_events: Vec::with_capacity(256),
            dirty_client_interests: Vec::with_capacity(256),
            dirty_backend_interests: Vec::with_capacity(64),
            next_token: FIRST_SOCKET_TOKEN,
            free_tokens: Vec::with_capacity(256),
            retired_tokens: Vec::with_capacity(256),
            next_deadline_epoch: 1,
            clients: TokenSlots::new(),
            backends: TokenSlots::new(),
            deadlines: VecDeque::new(),
            state: ReactorState::new(),
            startup_replays: Vec::new(),
            startup_waiters: VecDeque::new(),
            stats_dirty: false,
        }
    }

    fn run(mut self) {
        loop {
            let wait_timeout = self.next_wait_timeout();
            if self.poll.poll(&mut self.events, wait_timeout).is_err() {
                break;
            }
            self.ready_events.clear();
            self.ready_events.extend(
                self.events
                    .iter()
                    .map(|event| (event.token(), event.is_readable(), event.is_writable())),
            );
            for index in 0..self.ready_events.len() {
                let (token, readable, writable) = self.ready_events[index];
                self.handle_event(token, readable, writable);
            }
            self.expire_waiters();
            self.flush_interest_updates();
            self.flush_stats();
            // Tokens retired while handling this readiness snapshot cannot be
            // reused until every event in the snapshot has been consumed.
            self.free_tokens.append(&mut self.retired_tokens);
            if self.ingress.stopping.load(Ordering::Acquire) {
                break;
            }
        }
        // The full runtime begins publishing exact ownership counts as soon as
        // it registers frontend/backend sockets. Publishing zero here keeps an
        // already-stopped owner from leaving stale admin data behind.
        self.pool.publish_reactor_stats(0, 0);
    }

    fn drain_ingress(&mut self) {
        let pending: Vec<_> = {
            let mut pending = self.ingress.pending.lock().expect("reactor ingress lock");
            pending.drain(..).collect()
        };
        for incoming in pending {
            self.register_client(incoming);
        }
    }

    fn handle_event(&mut self, token: Token, readable: bool, writable: bool) {
        if token == INGRESS_TOKEN {
            self.drain_ingress();
            return;
        }
        if self.clients.contains_key(&token) {
            if writable {
                self.flush_client(token);
            }
            if readable && self.clients.contains_key(&token) {
                self.read_client(token);
            }
            return;
        }
        if self.backends.contains_key(&token) {
            if writable {
                self.flush_backend(token);
            }
            if readable && self.backends.contains_key(&token) {
                self.read_backend(token);
            }
        }
    }

    fn register_client(&mut self, incoming: IncomingFrontend) {
        let token = self.next_token();
        let id = ClientId(token.0 as u64);
        let mut stream = MioTcpStream::from_std(incoming.stream);
        if stream.set_nodelay(true).is_err()
            || self
                .poll
                .registry()
                .register(&mut stream, token, Interest::READABLE)
                .is_err()
        {
            return;
        }
        self.state.add_client(id);
        self.clients.insert(
            token,
            ReactorClient {
                stream,
                reader: FrameReader::new(Role::Frontend, &self.config.wire),
                _permit: incoming.permit,
                completion: incoming.completion,
                mode: ClientMode::Startup,
                startup: None,
                pending_first: None,
                output: OutputQueue::new(),
                wait_deadline: None,
                deadline_epoch: 0,
                interest_dirty: false,
                registered: true,
            },
        );
        self.publish_stats();
    }

    fn next_token(&mut self) -> Token {
        if let Some(token) = self.free_tokens.pop() {
            return Token(token);
        }
        let token = Token(self.next_token);
        self.next_token += 1;
        token
    }

    fn client_token(&self, client: ClientId) -> Option<Token> {
        let token = Token(client.0 as usize);
        self.clients.contains_key(&token).then_some(token)
    }

    fn backend_token(&self, backend: BackendId) -> Option<Token> {
        let token = Token(backend.0 as usize);
        self.backends.contains_key(&token).then_some(token)
    }

    fn publish_stats(&mut self) {
        self.stats_dirty = true;
    }

    fn flush_stats(&mut self) {
        if !self.stats_dirty {
            return;
        }
        let active = self
            .backends
            .values()
            .filter(|backend| !matches!(backend.mode, BackendMode::Idle | BackendMode::Resetting))
            .count();
        let idle = self
            .backends
            .values()
            .filter(|backend| matches!(backend.mode, BackendMode::Idle))
            .count();
        self.pool.publish_reactor_stats(active, idle);
        self.stats_dirty = false;
    }

    fn prune_deadlines(&mut self) {
        while let Some((deadline, token, epoch)) = self.deadlines.front().copied() {
            let valid = self.clients.get(&Token(token)).is_some_and(|client| {
                client.deadline_epoch == epoch && client.wait_deadline == Some(deadline)
            });
            if valid {
                break;
            }
            self.deadlines.pop_front();
        }
    }

    fn set_client_deadline(&mut self, token: Token, deadline: Option<Instant>) {
        let epoch = self.next_deadline_epoch;
        self.next_deadline_epoch = self.next_deadline_epoch.wrapping_add(1).max(1);
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        client.deadline_epoch = epoch;
        client.wait_deadline = deadline;
        if let Some(deadline) = deadline {
            // Every waiter uses the same acquire timeout, so insertion time
            // is deadline order. A FIFO deadline queue avoids heap work on
            // every short-lived transaction wait while retaining exact expiry.
            self.deadlines.push_back((deadline, token.0, epoch));
        }
    }

    fn next_wait_timeout(&mut self) -> Option<std::time::Duration> {
        self.prune_deadlines();
        let now = Instant::now();
        self.deadlines
            .front()
            .map(|(deadline, _, _)| deadline.saturating_duration_since(now))
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1879" reason="logic section in runtime.rs is hand-written pending codegen support">
    fn expire_waiters(&mut self) {
        let now = Instant::now();
        loop {
            self.prune_deadlines();
            let Some((deadline, token_index, _)) = self.deadlines.front().copied() else {
                break;
            };
            if deadline > now {
                break;
            }
            self.deadlines.pop_front();
            let token = Token(token_index);
            let epoch = self.next_deadline_epoch;
            self.next_deadline_epoch = self.next_deadline_epoch.wrapping_add(1).max(1);
            let expired = if let Some(client) = self.clients.get_mut(&token) {
                client.wait_deadline = None;
                client.deadline_epoch = epoch;
                client.mode = ClientMode::Closing;
                client.pending_first = None;
                true
            } else {
                false
            };
            if !expired {
                continue;
            }
            // The socket remains long enough to flush the rejection, but the
            // scheduler must forget it now. Otherwise a delayed clean-backend
            // action can resurrect this Closing client and relay its stale
            // first query after the 53300 error.
            let _ = self.state.remove_client(ClientId(token.0 as u64));
            let mut error = BytesMut::new();
            crate::pool::PoolRejectionReason::BackendPoolSaturated
                .synthesized_error_response()
                .encode(&mut error);
            self.queue_client_owned(token, error.freeze());
        }
    }
    // </HANDWRITE>

    fn queue_client(&mut self, token: Token, bytes: impl AsRef<[u8]>) {
        self.queue_client_owned(token, Bytes::copy_from_slice(bytes.as_ref()));
    }

    fn queue_client_owned(&mut self, token: Token, bytes: Bytes) {
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        client.output.push(bytes);
        self.update_client_interest(token);
    }

    /// Queues the transaction-mode protocol stopgap response before closing.
    /// `Closing` disables reads while the writable interest flushes the error
    /// to the frontend, so a rejected client observes ErrorResponse then EOF.
    fn reject_extended_query(&mut self, token: Token) {
        if let Some(client) = self.clients.get_mut(&token) {
            client.mode = ClientMode::Closing;
            client.wait_deadline = None;
        }
        let mut error = BytesMut::new();
        crate::pool::transaction::extended_query_rejection().encode(&mut error);
        self.queue_client_owned(token, error.freeze());
    }

    fn queue_backend(&mut self, token: Token, bytes: impl AsRef<[u8]>) {
        self.queue_backend_owned(token, Bytes::copy_from_slice(bytes.as_ref()));
    }

    fn queue_backend_owned(&mut self, token: Token, bytes: Bytes) {
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        backend.output.push(bytes);
        self.update_backend_interest(token);
    }

    fn flush_client(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) => flush_socket(&mut client.stream, &mut client.output),
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        let close_after_flush = matches!(
            self.clients.get(&token).map(|client| &client.mode),
            Some(ClientMode::Closing)
        ) && self
            .clients
            .get(&token)
            .is_some_and(|client| client.output.is_empty());
        if close_after_flush {
            self.close_client(token);
        } else {
            self.update_client_interest(token);
        }
    }

    fn flush_backend(&mut self, token: Token) {
        self.complete_connect(token);
        let close = match self.backends.get_mut(&token) {
            Some(backend) => flush_socket(&mut backend.stream, &mut backend.output),
            None => return,
        };
        if close {
            self.close_backend(token);
        } else {
            self.update_backend_interest(token);
        }
    }

    fn update_client_interest(&mut self, token: Token) {
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        if client.interest_dirty {
            return;
        }
        client.interest_dirty = true;
        self.dirty_client_interests.push(token);
    }

    fn update_backend_interest(&mut self, token: Token) {
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        if backend.interest_dirty {
            return;
        }
        backend.interest_dirty = true;
        self.dirty_backend_interests.push(token);
    }

    fn flush_interest_updates(&mut self) {
        while let Some(token) = self.dirty_client_interests.pop() {
            let Some(client) = self.clients.get_mut(&token) else {
                continue;
            };
            client.interest_dirty = false;
            self.apply_client_interest(token);
        }
        while let Some(token) = self.dirty_backend_interests.pop() {
            let Some(backend) = self.backends.get_mut(&token) else {
                continue;
            };
            backend.interest_dirty = false;
            self.apply_backend_interest(token);
        }
    }

    fn apply_client_interest(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) if !client.output.is_empty() => {
                flush_socket(&mut client.stream, &mut client.output)
            }
            Some(_) => false,
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        let close_after_flush = self.clients.get(&token).is_some_and(|client| {
            matches!(client.mode, ClientMode::Closing) && client.output.is_empty()
        });
        if close_after_flush {
            self.close_client(token);
            return;
        }
        let Some(client) = self.clients.get_mut(&token) else {
            return;
        };
        let want_read = client_can_read(&client.mode);
        let want_write = !client.output.is_empty();
        let interest = match (want_read, want_write) {
            (true, true) => Some(Interest::READABLE | Interest::WRITABLE),
            (true, false) => Some(Interest::READABLE),
            (false, true) => Some(Interest::WRITABLE),
            (false, false) => None,
        };
        match (client.registered, interest) {
            (true, Some(interest)) => {
                if self
                    .poll
                    .registry()
                    .reregister(&mut client.stream, token, interest)
                    .is_err()
                {
                    self.close_client(token);
                }
            }
            (false, Some(interest)) => {
                if self
                    .poll
                    .registry()
                    .register(&mut client.stream, token, interest)
                    .is_err()
                {
                    self.close_client(token);
                } else {
                    client.registered = true;
                }
            }
            (true, None) => {
                let _ = self.poll.registry().deregister(&mut client.stream);
                client.registered = false;
            }
            (false, None) => {}
        }
    }

    fn apply_backend_interest(&mut self, token: Token) {
        let close = match self.backends.get_mut(&token) {
            Some(backend)
                if !matches!(backend.mode, BackendMode::Connecting(_))
                    && !backend.output.is_empty() =>
            {
                flush_socket(&mut backend.stream, &mut backend.output)
            }
            Some(_) => false,
            None => return,
        };
        if close {
            self.close_backend(token);
            return;
        }
        let Some(backend) = self.backends.get_mut(&token) else {
            return;
        };
        let interest = if backend.output.is_empty() {
            Interest::READABLE
        } else {
            Interest::READABLE | Interest::WRITABLE
        };
        let result = if backend.registered {
            self.poll
                .registry()
                .reregister(&mut backend.stream, token, interest)
        } else {
            self.poll
                .registry()
                .register(&mut backend.stream, token, interest)
        };
        if result.is_err() {
            self.close_backend(token);
        } else {
            backend.registered = true;
        }
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1878" reason="logic section in runtime.rs is hand-written pending codegen support">
    fn read_client(&mut self, token: Token) {
        let close = match self.clients.get_mut(&token) {
            Some(client) => drain_socket(&mut client.stream, &mut client.reader),
            None => return,
        };
        if close {
            self.close_client(token);
            return;
        }
        self.drain_buffered_client_frames(token);
    }

    /// Re-drives complete frames that were already read from the socket before
    /// a transition made this client readable again. The caller must have
    /// changed the mode first; this helper never reads the socket itself, so it
    /// cannot bypass the wait/pending backpressure boundary.
    fn resume_buffered_client_frames(&mut self, token: Token) {
        if self
            .clients
            .get(&token)
            .is_some_and(|client| client_can_read(&client.mode))
        {
            self.drain_buffered_client_frames(token);
        }
    }

    fn drain_buffered_client_frames(&mut self, token: Token) {
        let id = ClientId(token.0 as u64);

        loop {
            let startup_phase = self
                .clients
                .get(&token)
                .is_some_and(|client| matches!(client.mode, ClientMode::Startup));
            if startup_phase {
                let frame = match self.clients.get_mut(&token) {
                    Some(client) => client.reader.next_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => self.handle_startup_frame(id, token, frame),
                    Ok(None) => break,
                    Err(_) => {
                        self.close_client(token);
                        return;
                    }
                }
            } else {
                let can_read = self
                    .clients
                    .get(&token)
                    .is_some_and(|client| client_can_read(&client.mode));
                if !can_read {
                    break;
                }
                let frame = match self.clients.get_mut(&token) {
                    Some(client) => client.reader.next_relay_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => {
                        self.handle_client_relay_frame(id, token, frame);
                        // A waiting/pipelined client is intentionally
                        // deregistered for reads to apply socket backpressure.
                        if !self
                            .clients
                            .get(&token)
                            .is_some_and(|client| client_can_read(&client.mode))
                        {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        self.close_client(token);
                        return;
                    }
                }
            }
            if !self.clients.contains_key(&token) {
                return;
            }
        }
        self.update_client_interest(token);
    }
    // </HANDWRITE>

    fn read_backend(&mut self, _token: Token) {
        let token = _token;
        self.complete_connect(token);
        let close = match self.backends.get_mut(&token) {
            Some(backend) => drain_socket(&mut backend.stream, &mut backend.reader),
            None => return,
        };
        if close {
            self.close_backend(token);
            return;
        }
        loop {
            let active_client = self
                .backends
                .get(&token)
                .and_then(|backend| match backend.mode {
                    BackendMode::Active { client } => Some(client),
                    _ => None,
                });
            if let Some(client) = active_client {
                let frame = match self.backends.get_mut(&token) {
                    Some(backend) => backend.reader.next_relay_frame_with_raw(),
                    None => return,
                };
                match frame {
                    Ok(Some(frame)) => self.handle_active_backend_frame(token, client, frame),
                    Ok(None) => break,
                    Err(_) => {
                        self.close_backend(token);
                        return;
                    }
                }
                if !self.backends.contains_key(&token) {
                    return;
                }
                continue;
            }
            let frame = match self.backends.get_mut(&token) {
                Some(backend) => backend.reader.next_frame_with_raw(),
                None => return,
            };
            match frame {
                Ok(Some(frame)) => self.handle_backend_frame(token, frame),
                Ok(None) => break,
                Err(_) => {
                    self.close_backend(token);
                    return;
                }
            }
            if !self.backends.contains_key(&token) {
                return;
            }
        }
        self.update_backend_interest(token);
    }

    fn complete_connect(&mut self, token: Token) {
        let purpose = match self.backends.get(&token).map(|backend| &backend.mode) {
            Some(BackendMode::Connecting(purpose)) => purpose.clone(),
            _ => return,
        };
        let error = self
            .backends
            .get_mut(&token)
            .and_then(|backend| backend.stream.take_error().ok())
            .flatten();
        if error.is_some() {
            self.close_backend(token);
            return;
        }
        let (mode, startup) = match purpose {
            ConnectPurpose::Initial { client, startup } => (
                BackendMode::InitialHandshake {
                    client,
                    replay: StartupCapture::default(),
                },
                startup,
            ),
            ConnectPurpose::Bootstrap { client, startup } => (
                BackendMode::Bootstrap {
                    client,
                    saw_auth_ok: false,
                },
                startup,
            ),
        };
        if let Some(backend) = self.backends.get_mut(&token) {
            if backend.stream.set_nodelay(true).is_err() {
                self.close_backend(token);
                return;
            }
            backend.mode = mode;
        }
        self.queue_backend_owned(token, startup);
        self.publish_stats();
    }

    fn handle_backend_frame(&mut self, token: Token, frame: WireFrame) {
        let mode = match self.backends.get(&token).map(|backend| &backend.mode) {
            Some(BackendMode::InitialHandshake { client, .. }) => {
                BackendFrameTarget::Initial(*client)
            }
            Some(BackendMode::Bootstrap { client, .. }) => BackendFrameTarget::Bootstrap(*client),
            Some(BackendMode::Active { client }) => BackendFrameTarget::Active(*client),
            Some(BackendMode::Resetting) => BackendFrameTarget::Resetting,
            Some(BackendMode::Idle) => BackendFrameTarget::Idle,
            Some(BackendMode::Connecting(_)) | None => return,
        };
        match mode {
            BackendFrameTarget::Initial(client) => {
                self.handle_initial_backend_frame(token, client, frame)
            }
            BackendFrameTarget::Bootstrap(client) => {
                self.handle_bootstrap_backend_frame(token, client, frame)
            }
            BackendFrameTarget::Active(client) => {
                let kind = match &frame.message {
                    WireMessage::Backend(BackendMessage::ReadyForQuery(ready)) => {
                        RelayFrameKind::BackendReady(ready.status)
                    }
                    _ => RelayFrameKind::Other,
                };
                self.handle_active_backend_frame(
                    token,
                    client,
                    RelayFrame {
                        kind,
                        bytes: frame.bytes,
                    },
                )
            }
            BackendFrameTarget::Resetting => self.handle_reset_backend_frame(token, frame),
            BackendFrameTarget::Idle => self.close_backend(token),
        }
    }

    fn handle_initial_backend_frame(&mut self, token: Token, client: ClientId, frame: WireFrame) {
        let raw = frame.bytes.to_vec();
        let Some(client_token) = self.client_token(client) else {
            self.close_backend(token);
            return;
        };
        self.queue_client(client_token, &raw);

        let is_ready = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ReadyForQuery(_))
        );
        let is_error = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ErrorResponse(_))
        );
        let requires_frontend_auth = matches!(
            &frame.message,
            WireMessage::Backend(
                BackendMessage::AuthenticationCleartextPassword(_)
                    | BackendMessage::AuthenticationMd5Password(_)
                    | BackendMessage::AuthenticationSasl(_)
                    | BackendMessage::AuthenticationSaslContinue(_)
            )
        );
        if let Some(backend) = self.backends.get_mut(&token) {
            if let BackendMode::InitialHandshake { replay, .. } = &mut backend.mode {
                match &frame.message {
                    WireMessage::Backend(BackendMessage::AuthenticationOk(_)) => {
                        replay.saw_auth_ok = true;
                        replay.frames.push(raw.clone());
                        replay.messages.push(BackendMessage::AuthenticationOk(
                            crate::wire::AuthenticationOk,
                        ));
                    }
                    WireMessage::Backend(BackendMessage::BackendKeyData(_))
                        if replay.saw_auth_ok =>
                    {
                        replay.frames.push(zero_backend_key_frame());
                        replay
                            .messages
                            .push(BackendMessage::BackendKeyData(BackendKeyData {
                                process_id: 0,
                                secret_key: 0,
                            }));
                    }
                    WireMessage::Backend(
                        BackendMessage::AuthenticationCleartextPassword(_)
                        | BackendMessage::AuthenticationMd5Password(_)
                        | BackendMessage::AuthenticationSasl(_)
                        | BackendMessage::AuthenticationSaslContinue(_),
                    ) => {
                        replay.replayable = false;
                        replay.frames.clear();
                        replay.messages.clear();
                    }
                    WireMessage::Backend(message) if replay.saw_auth_ok => {
                        replay.frames.push(raw.clone());
                        replay.messages.push(message.clone());
                    }
                    _ => {}
                }
            }
        }
        if is_error {
            self.close_backend(token);
            return;
        }
        if requires_frontend_auth {
            if let Some(client) = self.clients.get_mut(&client_token) {
                client.mode = ClientMode::Handshaking {
                    backend: BackendId(token.0 as u64),
                    awaiting_auth: true,
                };
            }
            self.resume_buffered_client_frames(client_token);
            return;
        }
        if !is_ready {
            return;
        }

        let replay = self
            .backends
            .get_mut(&token)
            .and_then(|backend| match &mut backend.mode {
                BackendMode::InitialHandshake { replay, .. } => Some(std::mem::take(replay)),
                _ => None,
            });
        let Some(replay) = replay else {
            self.close_backend(token);
            return;
        };
        let startup = self
            .clients
            .get(&client_token)
            .and_then(|entry| entry.startup.clone());
        let Some(startup) = startup else {
            self.close_backend(token);
            return;
        };
        if replay.replayable
            && replay.saw_auth_ok
            && self.startup_replays.len() < 64
            && !self
                .startup_replays
                .iter()
                .any(|entry| entry.startup == startup)
        {
            self.pool
                .publish_startup_replay(startup.clone(), replay.messages.clone());
            self.startup_replays.push(StartupReplay {
                startup,
                frames: replay.frames,
            });
        }
        if let Some(entry) = self.clients.get_mut(&client_token) {
            entry.mode = ClientMode::Idle;
            entry.wait_deadline = None;
        }
        if let Some(entry) = self.backends.get_mut(&token) {
            entry.mode = BackendMode::Resetting;
        }
        self.state.add_resetting_backend(BackendId(token.0 as u64));
        self.queue_backend(token, DISCARD_ALL_QUERY_FRAME);
        self.serve_startup_waiters();
        self.resume_buffered_client_frames(client_token);
    }

    fn handle_bootstrap_backend_frame(&mut self, token: Token, client: ClientId, frame: WireFrame) {
        let challenge_or_error = matches!(
            &frame.message,
            WireMessage::Backend(
                BackendMessage::AuthenticationCleartextPassword(_)
                    | BackendMessage::AuthenticationMd5Password(_)
                    | BackendMessage::AuthenticationSasl(_)
                    | BackendMessage::AuthenticationSaslContinue(_)
                    | BackendMessage::ErrorResponse(_)
            )
        );
        if challenge_or_error {
            self.close_backend(token);
            return;
        }
        if let Some(backend) = self.backends.get_mut(&token) {
            if let BackendMode::Bootstrap { saw_auth_ok, .. } = &mut backend.mode {
                if matches!(
                    &frame.message,
                    WireMessage::Backend(BackendMessage::AuthenticationOk(_))
                ) {
                    *saw_auth_ok = true;
                }
            }
        }
        let ready = matches!(
            &frame.message,
            WireMessage::Backend(BackendMessage::ReadyForQuery(_))
        );
        if !ready {
            return;
        }
        let ready_for_use = matches!(
            self.backends.get(&token).map(|backend| &backend.mode),
            Some(BackendMode::Bootstrap {
                saw_auth_ok: true,
                ..
            })
        );
        if !ready_for_use {
            self.close_backend(token);
            return;
        }
        if let Some(backend) = self.backends.get_mut(&token) {
            backend.mode = BackendMode::Idle;
        }
        let action = self.state.add_clean_backend(BackendId(token.0 as u64));
        self.drive_action(action);
        self.publish_stats();
        let _ = client;
    }

    fn handle_active_backend_frame(&mut self, token: Token, client: ClientId, frame: RelayFrame) {
        let Some(client_token) = self.client_token(client) else {
            self.close_backend(token);
            return;
        };
        let ready = match frame.kind {
            RelayFrameKind::BackendReady(status) => Some(status),
            _ => None,
        };
        self.queue_client_owned(client_token, frame.bytes);
        let Some(ready) = ready else {
            return;
        };
        let backend = BackendId(token.0 as u64);
        if ready == TransactionStatus::Idle {
            let pending = self
                .clients
                .get(&client_token)
                .and_then(|entry| match &entry.mode {
                    ClientMode::Active { pending_next, .. } => pending_next.clone(),
                    _ => None,
                });
            let has_pending = pending.is_some();
            let deadline = (pending.is_some()).then(|| Instant::now() + self.queue_wait_timeout());
            if let Some(entry) = self.clients.get_mut(&client_token) {
                if let Some(next) = pending {
                    entry.pending_first = Some(next);
                    entry.mode = ClientMode::Waiting;
                } else {
                    entry.mode = ClientMode::Idle;
                }
            }
            self.set_client_deadline(client_token, deadline);
            let action = self
                .state
                .transaction_ready_idle(client, backend, has_pending);
            self.drive_action(action);
            if has_pending {
                self.update_client_interest(client_token);
            } else {
                self.resume_buffered_client_frames(client_token);
            }
        } else if let Some(next) = self.clients.get_mut(&client_token).and_then(|entry| {
            if let ClientMode::Active { pending_next, .. } = &mut entry.mode {
                pending_next.take()
            } else {
                None
            }
        }) {
            self.queue_backend_owned(token, next.bytes);
            self.update_client_interest(client_token);
        } else if let Some(entry) = self.clients.get_mut(&client_token) {
            if let ClientMode::Active {
                backend_ready_for_frontend,
                ..
            } = &mut entry.mode
            {
                *backend_ready_for_frontend = true;
            }
            self.update_client_interest(client_token);
        }
    }

    fn handle_reset_backend_frame(&mut self, token: Token, frame: WireFrame) {
        match frame.message {
            WireMessage::Backend(BackendMessage::ReadyForQuery(ready))
                if ready.status == TransactionStatus::Idle =>
            {
                if let Some(backend) = self.backends.get_mut(&token) {
                    backend.mode = BackendMode::Idle;
                }
                let backend = BackendId(token.0 as u64);
                let action = self.state.reset_ready_idle(backend);
                self.drive_action(action);
                self.publish_stats();
            }
            WireMessage::Backend(BackendMessage::ErrorResponse(_)) => self.close_backend(token),
            _ => {}
        }
    }

    fn serve_startup_waiters(&mut self) {
        let waiting: Vec<_> = self.startup_waiters.drain(..).collect();
        for client in waiting {
            let Some(token) = self.client_token(client) else {
                continue;
            };
            if matches!(
                self.clients.get(&token).map(|entry| &entry.mode),
                Some(ClientMode::StartupWaiting)
            ) {
                self.admit_startup(client);
                self.resume_buffered_client_frames(token);
            }
        }
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in runtime.rs is hand-written pending codegen support">
    fn handle_startup_frame(&mut self, id: ClientId, token: Token, frame: WireFrame) {
        match frame.message {
            WireMessage::Frontend(FrontendMessage::Ssl(_)) => self.queue_client(token, b"N"),
            WireMessage::Frontend(FrontendMessage::Startup(startup)) => {
                let startup = self.pool.normalize_backend_startup(startup);
                if let Some(client) = self.clients.get_mut(&token) {
                    client.startup = Some(startup);
                }
                self.admit_startup(id);
            }
            _ => self.close_client(token),
        }
    }
    // </HANDWRITE>

    fn handle_client_relay_frame(&mut self, id: ClientId, token: Token, frame: RelayFrame) {
        if matches!(frame.kind, RelayFrameKind::FrontendTerminate) {
            self.close_client(token);
            return;
        }
        if frame.is_extended_query() {
            self.reject_extended_query(token);
            return;
        }
        let mode = self.clients.get(&token).map(|client| match &client.mode {
            ClientMode::Idle => (0_u8, None),
            ClientMode::Handshaking {
                backend,
                awaiting_auth: true,
            } => (1, Some(*backend)),
            ClientMode::Active {
                backend,
                pending_next: None,
                backend_ready_for_frontend: true,
                ..
            } => (2, Some(*backend)),
            ClientMode::Active {
                pending_next: None, ..
            } => (3, None),
            _ => (4, None),
        });
        match mode {
            Some((0, _)) => {
                let deadline = Instant::now() + self.queue_wait_timeout();
                if let Some(client) = self.clients.get_mut(&token) {
                    client.pending_first = Some(frame);
                    client.mode = ClientMode::Waiting;
                }
                self.set_client_deadline(token, Some(deadline));
                let action = self.state.request_backend(id);
                self.drive_action(action);
                if self.backends.len() < self.config.max_backend_connections
                    && self.state.should_open_normal_backend()
                    && matches!(
                        self.clients.get(&token).map(|client| &client.mode),
                        Some(ClientMode::Waiting)
                    )
                {
                    let _ = self.open_backend(ConnectPurpose::Bootstrap {
                        client: id,
                        startup: self.startup_bytes(id),
                    });
                }
            }
            Some((1, Some(backend))) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Handshaking { awaiting_auth, .. } = &mut client.mode {
                        *awaiting_auth = false;
                    }
                }
                if let Some(backend_token) = self.backend_token(backend) {
                    self.queue_backend_owned(backend_token, frame.bytes);
                } else {
                    self.close_client(token);
                }
            }
            Some((2, Some(backend))) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Active {
                        backend_ready_for_frontend,
                        ..
                    } = &mut client.mode
                    {
                        *backend_ready_for_frontend = false;
                    }
                }
                if let Some(backend_token) = self.backend_token(backend) {
                    self.queue_backend_owned(backend_token, frame.bytes);
                } else {
                    self.close_client(token);
                }
            }
            Some((3, _)) => {
                if let Some(client) = self.clients.get_mut(&token) {
                    if let ClientMode::Active { pending_next, .. } = &mut client.mode {
                        *pending_next = Some(frame);
                    }
                }
            }
            _ => self.close_client(token),
        }
    }

    fn startup_bytes(&self, client: ClientId) -> Bytes {
        let mut bytes = BytesMut::new();
        if let Some(token) = self.client_token(client) {
            if let Some(startup) = self
                .clients
                .get(&token)
                .and_then(|client| client.startup.clone())
            {
                FrontendMessage::Startup(startup).encode(&mut bytes);
            }
        }
        bytes.freeze()
    }

    fn admit_startup(&mut self, client: ClientId) {
        let Some(token) = self.client_token(client) else {
            return;
        };
        let startup = self
            .clients
            .get(&token)
            .and_then(|entry| entry.startup.clone());
        let Some(startup) = startup else {
            self.close_client(token);
            return;
        };
        if let Some(replay) = self
            .startup_replays
            .iter()
            .find(|entry| entry.startup == startup)
            .cloned()
        {
            for frame in replay.frames {
                self.queue_client_owned(token, Bytes::from(frame));
            }
            if let Some(entry) = self.clients.get_mut(&token) {
                entry.mode = ClientMode::Idle;
                entry.wait_deadline = None;
            }
            self.update_client_interest(token);
            return;
        }
        if self.backends.len() < self.config.max_backend_connections {
            if let Some(backend) = self.open_backend(ConnectPurpose::Initial {
                client,
                startup: self.startup_bytes(client),
            }) {
                if let Some(entry) = self.clients.get_mut(&token) {
                    entry.mode = ClientMode::Handshaking {
                        backend,
                        awaiting_auth: false,
                    };
                    entry.wait_deadline = None;
                }
                return;
            }
        }
        let deadline = Instant::now() + self.queue_wait_timeout();
        if let Some(entry) = self.clients.get_mut(&token) {
            entry.mode = ClientMode::StartupWaiting;
        }
        self.set_client_deadline(token, Some(deadline));
        self.startup_waiters.push_back(client);
        self.update_client_interest(token);
    }

    fn open_backend(&mut self, purpose: ConnectPurpose) -> Option<BackendId> {
        let address = (
            self.config.endpoint.host.as_str(),
            self.config.endpoint.port,
        )
            .to_socket_addrs()
            .ok()
            .and_then(|mut addresses| addresses.next())?;
        let mut stream = match MioTcpStream::connect(address) {
            Ok(stream) => stream,
            Err(_) => return None,
        };
        let token = self.next_token();
        if self
            .poll
            .registry()
            .register(&mut stream, token, Interest::READABLE | Interest::WRITABLE)
            .is_err()
        {
            return None;
        }
        let id = BackendId(token.0 as u64);
        self.backends.insert(
            token,
            ReactorBackend {
                stream,
                reader: FrameReader::new(Role::Backend, &self.config.wire),
                mode: BackendMode::Connecting(purpose),
                output: OutputQueue::new(),
                interest_dirty: false,
                registered: true,
            },
        );
        self.publish_stats();
        Some(id)
    }

    fn drive_action(&mut self, action: TransactionAction) {
        match action {
            TransactionAction::Assign { client, backend } => {
                let (Some(client_token), Some(backend_token)) =
                    (self.client_token(client), self.backend_token(backend))
                else {
                    return;
                };
                if self
                    .clients
                    .get(&client_token)
                    .is_some_and(|entry| matches!(entry.mode, ClientMode::Closing))
                {
                    // Expiry removes the scheduler entry before it can
                    // produce an Assign action. If an action was already
                    // prepared, discard it without dropping the queued
                    // ErrorResponse; returning the backend to the scheduler
                    // lets another live waiter use it.
                    if let Some(entry) = self.clients.get_mut(&client_token) {
                        entry.pending_first = None;
                    }
                    let _ = self.state.remove_backend(backend);
                    let retry = self.state.add_clean_backend(backend);
                    self.drive_action(retry);
                    return;
                }
                let first = self
                    .clients
                    .get_mut(&client_token)
                    .and_then(|entry| entry.pending_first.take());
                let Some(first) = first else {
                    self.close_backend(backend_token);
                    return;
                };
                if let Some(client_entry) = self.clients.get_mut(&client_token) {
                    client_entry.mode = ClientMode::Active {
                        backend,
                        pending_next: None,
                        backend_ready_for_frontend: false,
                    };
                    client_entry.wait_deadline = None;
                }
                if let Some(backend_entry) = self.backends.get_mut(&backend_token) {
                    backend_entry.mode = BackendMode::Active { client };
                }
                self.queue_backend_owned(backend_token, first.bytes);
                self.update_client_interest(client_token);
                self.resume_buffered_client_frames(client_token);
                self.publish_stats();
            }
            TransactionAction::Reset { backend } => {
                if let Some(token) = self.backend_token(backend) {
                    if let Some(entry) = self.backends.get_mut(&token) {
                        entry.mode = BackendMode::Resetting;
                    }
                    self.queue_backend(token, DISCARD_ALL_QUERY_FRAME);
                    self.publish_stats();
                }
            }
            TransactionAction::Queued { .. } | TransactionAction::Idle { .. } => {}
        }
    }

    fn close_client(&mut self, token: Token) {
        let Some(mut client) = self.clients.remove(&token) else {
            return;
        };
        let direct_backend = match client.mode {
            ClientMode::Handshaking { backend, .. } | ClientMode::Active { backend, .. } => {
                Some(backend)
            }
            _ => None,
        };
        if client.registered {
            let _ = self.poll.registry().deregister(&mut client.stream);
        }
        let id = ClientId(token.0 as u64);
        self.startup_waiters.retain(|waiting| *waiting != id);
        let _ = client.completion.send(());
        let state_backend = self.state.remove_client(id);
        if let Some(backend) = direct_backend.or(state_backend) {
            if let Some(backend_token) = self.backend_token(backend) {
                self.close_backend(backend_token);
            }
        }
        self.retired_tokens.push(token.0);
        self.publish_stats();
    }

    // <HANDWRITE gap="missing-generator:logic" tracker="#1880" reason="logic section in runtime.rs is hand-written pending codegen support">
    fn close_backend(&mut self, token: Token) {
        let Some(mut backend) = self.backends.remove(&token) else {
            return;
        };
        let direct_client = match &backend.mode {
            BackendMode::Connecting(ConnectPurpose::Initial { client, .. })
            | BackendMode::InitialHandshake { client, .. }
            | BackendMode::Active { client } => Some(*client),
            // A bootstrap is speculative capacity for a client that remains
            // Waiting in ReactorState. Auth-required backends cannot accept
            // it without a frontend password exchange, so discarding this
            // backend must not disconnect the healthy waiter.
            BackendMode::Connecting(ConnectPurpose::Bootstrap { .. })
            | BackendMode::Bootstrap { .. }
            | BackendMode::Resetting
            | BackendMode::Idle => None,
        };
        if backend.registered {
            let _ = self.poll.registry().deregister(&mut backend.stream);
        }
        let id = BackendId(token.0 as u64);
        let state_client = self.state.remove_backend(id);
        if let Some(client) = state_client.or(direct_client) {
            if let Some(client_token) = self.client_token(client) {
                self.close_client(client_token);
            }
        }
        self.retired_tokens.push(token.0);
        self.publish_stats();
    }
    // </HANDWRITE>
}

fn client_can_read(mode: &ClientMode) -> bool {
    matches!(
        mode,
        ClientMode::Startup
            | ClientMode::Idle
            | ClientMode::Handshaking {
                awaiting_auth: true,
                ..
            }
            | ClientMode::Active {
                pending_next: None,
                ..
            }
    )
}

/// Writes until the kernel would block. `true` means EOF/error and asks the
/// owner to close the complete client/backend state, not merely this socket.
fn flush_socket(stream: &mut MioTcpStream, output: &mut OutputQueue) -> bool {
    const MAX_IO_SLICES: usize = 16;
    while !output.is_empty() {
        let result = {
            let mut slices: [IoSlice<'_>; MAX_IO_SLICES] =
                std::array::from_fn(|_| IoSlice::new(&[]));
            let mut count = 0;
            for (index, chunk) in output.chunks.iter().take(MAX_IO_SLICES).enumerate() {
                let start = if index == 0 { output.front_offset } else { 0 };
                slices[index] = IoSlice::new(&chunk[start..]);
                count += 1;
            }
            stream.write_vectored(&slices[..count])
        };
        match result {
            Ok(0) => return true,
            Err(ref error) if error.kind() == io::ErrorKind::WriteZero => return true,
            Ok(written) => output.advance(written),
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(_) => return true,
        }
    }
    false
}

/// Drains a nonblocking socket directly into its persistent parser buffer.
/// `true` classifies EOF or an I/O failure as terminal; `WouldBlock` ends the
/// current readiness turn without allocating or copying through a temporary
/// aggregate buffer.
fn drain_socket(stream: &mut MioTcpStream, reader: &mut FrameReader) -> bool {
    loop {
        match reader.read_from_sync(stream) {
            Ok(0) => return true,
            Ok(_) => {}
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => return false,
            Err(_) => return true,
        }
    }
}

fn zero_backend_key_frame() -> Vec<u8> {
    let mut bytes = BytesMut::new();
    BackendMessage::BackendKeyData(BackendKeyData {
        process_id: 0,
        secret_key: 0,
    })
    .encode(&mut bytes);
    bytes.to_vec()
}
// </HANDWRITE>

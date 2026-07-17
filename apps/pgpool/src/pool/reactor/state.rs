// SPEC-MANAGED: apps/pgpool/tech-design/logic/p0-dense-buffer-readiness-reactor.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-readiness-reactor" tracker="#1753" reason="The transaction reactor owns a socket-readiness state machine not yet expressible by the service generator.">
//! Socket-independent ownership state for transaction pooling.
//!
//! `ReactorState` deliberately knows neither Tokio nor `mio`.  A runtime feeds
//! it accepted-client, backend-ready, and reset-complete events, then carries
//! out the returned action against actual sockets.  This keeps the important
//! invariant explicit: a backend that has reached `ReadyForQuery(Idle)` is
//! *resetting*, not reusable, until the reset's own `ReadyForQuery(Idle)`.

use std::collections::VecDeque;

/// Identity assigned by the readiness runtime to an accepted frontend socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ClientId(pub u64);

/// Identity assigned by the readiness runtime to a physical backend socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BackendId(pub u64);

trait DenseKey {
    fn index(self) -> usize;
}

impl DenseKey for ClientId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

impl DenseKey for BackendId {
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug)]
struct DenseSlots<T> {
    slots: Vec<Option<T>>,
}

impl<T> DenseSlots<T> {
    fn new() -> Self {
        Self { slots: Vec::new() }
    }

    fn get<K: DenseKey + Copy>(&self, key: &K) -> Option<&T> {
        self.slots.get(key.index()).and_then(Option::as_ref)
    }

    fn insert<K: DenseKey>(&mut self, key: K, value: T) -> Option<T> {
        let index = key.index();
        if self.slots.len() <= index {
            self.slots.resize_with(index + 1, || None);
        }
        self.slots[index].replace(value)
    }

    fn remove<K: DenseKey + Copy>(&mut self, key: &K) -> Option<T> {
        self.slots.get_mut(key.index()).and_then(Option::take)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientPhase {
    Idle,
    Waiting,
    Active(BackendId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendPhase {
    Idle,
    Active(ClientId),
    Resetting,
}

/// Concrete I/O work selected by a pure ownership transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransactionAction {
    /// Send the client's first or already-pipelined next transaction frame to
    /// this clean, newly assigned backend.
    Assign {
        client: ClientId,
        backend: BackendId,
    },
    /// Send `DISCARD ALL`; the backend stays unavailable until a separate
    /// [`Self::ResetComplete`] transition.
    Reset { backend: BackendId },
    /// No clean backend exists yet.  The runtime must retain normal socket
    /// backpressure; it must not route the frame to a resetting backend.
    Queued { client: ClientId },
    /// A backend has become clean but no frontend is waiting for it.
    Idle { backend: BackendId },
}

impl TransactionAction {
    /// Marker used only in test diagnostics; runtime actions remain explicit
    /// rather than encoded as nullable IDs.
    const fn queued(client: ClientId) -> Self {
        Self::Queued { client }
    }
}

/// Single-owner scheduling state for transaction pooling.
///
/// All methods require `&mut self`, so the eventual readiness loop owns the
/// queue and maps directly.  There is no pool mutex, semaphore, notification
/// herd, or deadline timer in this state machine's transaction transition.
#[derive(Debug)]
pub(crate) struct ReactorState {
    clients: DenseSlots<ClientPhase>,
    backends: DenseSlots<BackendPhase>,
    clean_idle: VecDeque<BackendId>,
    waiters: VecDeque<ClientId>,
    waiting_count: usize,
    resetting_count: usize,
}

impl ReactorState {
    pub(crate) fn new() -> Self {
        Self {
            clients: DenseSlots::new(),
            backends: DenseSlots::new(),
            clean_idle: VecDeque::new(),
            waiters: VecDeque::new(),
            waiting_count: 0,
            resetting_count: 0,
        }
    }

    pub(crate) fn add_client(&mut self, client: ClientId) {
        let previous = self.clients.insert(client, ClientPhase::Idle);
        assert!(previous.is_none(), "client identity must be unique");
    }

    /// Registers a completed startup/reset-clean backend.  The caller must
    /// only call this after wire validation has observed the relevant
    /// `ReadyForQuery(Idle)`.
    pub(crate) fn add_clean_backend(&mut self, backend: BackendId) -> TransactionAction {
        let previous = self.backends.insert(backend, BackendPhase::Idle);
        assert!(previous.is_none(), "backend identity must be unique");
        self.assign_or_park(backend)
    }

    /// Registers the mandatory reset that follows the first authenticated
    /// startup exchange. It participates in capacity decisions immediately,
    /// even though it cannot enter the clean-idle queue until reset completes.
    pub(crate) fn add_resetting_backend(&mut self, backend: BackendId) {
        let previous = self.backends.insert(backend, BackendPhase::Resetting);
        assert!(previous.is_none(), "backend identity must be unique");
        self.resetting_count += 1;
    }

    /// A frontend has supplied the first frame of a transaction.  The runtime
    /// holds that raw frame until an [`TransactionAction::Assign`] arrives;
    /// this method never forwards directly to an arbitrary busy backend.
    pub(crate) fn request_backend(&mut self, client: ClientId) -> TransactionAction {
        assert_eq!(self.clients.get(&client), Some(&ClientPhase::Idle));
        match self.clean_idle.pop_front() {
            Some(backend) => self.assign(client, backend),
            None => {
                self.clients.insert(client, ClientPhase::Waiting);
                self.waiters.push_back(client);
                self.waiting_count += 1;
                TransactionAction::queued(client)
            }
        }
    }

    /// The active backend has reached `ReadyForQuery(Idle)` for the client's
    /// transaction.  The backend immediately becomes `Resetting`; it cannot
    /// serve the caller's pipelined next frame or any other client yet.
    pub(crate) fn transaction_ready_idle(
        &mut self,
        client: ClientId,
        backend: BackendId,
        has_pipelined_next_frame: bool,
    ) -> TransactionAction {
        assert_eq!(
            self.clients.get(&client),
            Some(&ClientPhase::Active(backend))
        );
        assert_eq!(
            self.backends.get(&backend),
            Some(&BackendPhase::Active(client))
        );
        self.backends.insert(backend, BackendPhase::Resetting);
        self.resetting_count += 1;
        if has_pipelined_next_frame {
            self.clients.insert(client, ClientPhase::Waiting);
            self.waiters.push_back(client);
            self.waiting_count += 1;
        } else {
            self.clients.insert(client, ClientPhase::Idle);
        }
        TransactionAction::Reset { backend }
    }

    /// The reset query itself returned `ReadyForQuery(Idle)`.  The backend is
    /// now the only kind that may be assigned to a waiter.
    pub(crate) fn reset_ready_idle(&mut self, backend: BackendId) -> TransactionAction {
        assert_eq!(self.backends.get(&backend), Some(&BackendPhase::Resetting));
        self.backends.insert(backend, BackendPhase::Idle);
        self.resetting_count -= 1;
        self.assign_or_park(backend)
    }

    /// Removes a socket after EOF, frame failure, or I/O failure.  Stale queue
    /// IDs are harmless: assignment skips missing/non-waiting clients.
    pub(crate) fn remove_client(&mut self, client: ClientId) -> Option<BackendId> {
        self.waiters.retain(|id| *id != client);
        match self.clients.remove(&client) {
            Some(ClientPhase::Active(backend)) => {
                self.backends.remove(&backend);
                self.clean_idle.retain(|id| *id != backend);
                Some(backend)
            }
            Some(ClientPhase::Waiting) => {
                self.waiting_count -= 1;
                None
            }
            Some(ClientPhase::Idle) | None => None,
        }
    }

    pub(crate) fn remove_backend(&mut self, backend: BackendId) -> Option<ClientId> {
        self.clean_idle.retain(|id| *id != backend);
        match self.backends.remove(&backend) {
            Some(BackendPhase::Active(client)) => {
                self.clients.remove(&client);
                Some(client)
            }
            Some(BackendPhase::Resetting) => {
                self.resetting_count -= 1;
                None
            }
            Some(BackendPhase::Idle) | None => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn clean_idle_len(&self) -> usize {
        self.clean_idle.len()
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(crate) fn waiting_len(&self) -> usize {
        self.waiting_count
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(crate) fn resetting_len(&self) -> usize {
        self.resetting_count
    }

    /// A fresh normal backend is useful only when a live FIFO waiter remains
    /// after accounting for reset-in-flight backends that will shortly become
    /// clean. The readiness runtime uses this predicate before opening normal
    /// capacity; reserve demand begins only after its separate bounded wait.
    pub(crate) fn should_open_normal_backend(&self) -> bool {
        self.waiting_count > self.resetting_count
    }

    fn assign_or_park(&mut self, backend: BackendId) -> TransactionAction {
        while let Some(client) = self.waiters.pop_front() {
            if self.clients.get(&client) == Some(&ClientPhase::Waiting) {
                self.waiting_count -= 1;
                return self.assign(client, backend);
            }
        }
        self.clean_idle.push_back(backend);
        TransactionAction::Idle { backend }
    }

    fn assign(&mut self, client: ClientId, backend: BackendId) -> TransactionAction {
        assert_eq!(self.backends.get(&backend), Some(&BackendPhase::Idle));
        self.clients.insert(client, ClientPhase::Active(backend));
        self.backends.insert(backend, BackendPhase::Active(client));
        TransactionAction::Assign { client, backend }
    }
}

#[cfg(test)]
mod tests {
    use super::{BackendId, ClientId, ReactorState, TransactionAction};

    #[test]
    fn reset_boundary_blocks_a_pipelined_next_query_from_the_old_backend() {
        let client_a = ClientId(1);
        let client_b = ClientId(2);
        let backend = BackendId(1);
        let mut state = ReactorState::new();
        state.add_client(client_a);
        state.add_client(client_b);
        assert_eq!(
            state.add_clean_backend(backend),
            TransactionAction::Idle { backend }
        );
        assert_eq!(
            state.request_backend(client_a),
            TransactionAction::Assign {
                client: client_a,
                backend
            }
        );

        // Client A pipelined its next query. It becomes a waiter, but must
        // never be assigned until the old backend has completed DISCARD ALL.
        assert_eq!(
            state.transaction_ready_idle(client_a, backend, true),
            TransactionAction::Reset { backend }
        );
        assert_eq!(
            state.request_backend(client_b),
            TransactionAction::Queued { client: client_b }
        );
        assert_eq!(state.waiting_len(), 2);

        // FIFO picks A after a *reset* completion, never at the transaction
        // ReadyForQuery boundary. B remains queued for a later clean backend.
        assert_eq!(
            state.reset_ready_idle(backend),
            TransactionAction::Assign {
                client: client_a,
                backend
            }
        );
        assert_eq!(state.waiting_len(), 1);
        assert_eq!(state.clean_idle_len(), 0);
    }

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in state.rs is hand-written pending codegen support">
    #[test]
    fn disconnected_waiters_do_not_consume_a_clean_backend() {
        let client = ClientId(1);
        let backend = BackendId(1);
        let mut state = ReactorState::new();
        state.add_client(client);
        assert_eq!(
            state.request_backend(client),
            TransactionAction::Queued { client }
        );
        assert_eq!(state.remove_client(client), None);
        assert_eq!(
            state.add_clean_backend(backend),
            TransactionAction::Idle { backend }
        );
        assert_eq!(state.clean_idle_len(), 1);
        assert_eq!(state.waiting_len(), 0);
    }
// </HANDWRITE>

    #[test]
    fn initial_reset_counts_as_recoverable_capacity() {
        let backend = BackendId(1);
        let mut state = ReactorState::new();
        state.add_resetting_backend(backend);
        assert_eq!(state.resetting_len(), 1);
        assert_eq!(
            state.reset_ready_idle(backend),
            TransactionAction::Idle { backend }
        );
        assert_eq!(state.resetting_len(), 0);
        assert_eq!(state.clean_idle_len(), 1);
    }

    #[test]
    fn disconnected_waiter_identity_can_be_reused_without_a_stale_queue_entry() {
        let client = ClientId(1);
        let backend = BackendId(2);
        let mut state = ReactorState::new();
        state.add_client(client);
        assert_eq!(
            state.request_backend(client),
            TransactionAction::Queued { client }
        );
        assert_eq!(state.remove_client(client), None);

        state.add_client(client);
        assert_eq!(
            state.add_clean_backend(backend),
            TransactionAction::Idle { backend }
        );
        assert_eq!(state.clean_idle_len(), 1);
        assert_eq!(state.waiting_len(), 0);
    }
}
// </HANDWRITE>

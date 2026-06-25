//! Production driver that runs the shared [`RaftNode`] core for lumen.
//!
//! Mirrors relay's `raft_driver` **structurally** but with one critical
//! difference: it does **not** apply committed entries to the engine. lumen
//! already has exactly one applier — the [`crate::coordinator::WriteCoordinator`]
//! apply loop tailing the WAL — so this driver instead *surfaces* committed
//! entries as `(index, WalRecord)` into a buffer that [`crate::wal_raft::RaftWal`]
//! exposes through the `WalLog` seam. The raft log index **is** the WAL seq
//! (both 1-based), so no offset bridging is needed.
//!
//! Slice 1 (this file) is single-node: the sole voter elects itself within a few
//! ticks and commits immediately. The peer transport (h2c flush + inbound
//! Vote/Append RPCs + leader redirect) is Slice 2.

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

use crate::raft_core::{Membership, NodeId, RaftNode};
use crate::raft_store::RaftStore;
use crate::wal::WalRecord;

/// Logical tick interval (a sole voter elects after ~`ELECTION_MIN` ticks).
const TICK: Duration = Duration::from_millis(20);

/// Committed records surfaced from the raft log, index-ordered (1-based,
/// contiguous): position `i` holds the entry with index `i + 1`.
type Committed = Arc<StdMutex<Vec<(u64, WalRecord)>>>;

struct Shared {
    node: Mutex<RaftNode>,
    store: RaftStore,
    committed: Committed,
    commit_tx: watch::Sender<u64>,
}

impl Shared {
    /// Persist the node's hard state (best-effort; called while holding the lock,
    /// before any reply/heartbeat is flushed).
    fn persist(&self, node: &RaftNode) {
        let _ = self.store.save(&node.persisted());
    }

    /// Drain newly-committed entries into the surfaced buffer and bump the watch
    /// so subscribers wake. Called while holding the node lock, so `take_committed`
    /// (which advances `last_applied`) and the buffer push stay strictly ordered.
    fn surface(&self, node: &mut RaftNode) {
        let entries = node.take_committed();
        if entries.is_empty() {
            return;
        }
        let mut last = 0;
        {
            let mut buf = self
                .committed
                .lock()
                .expect("raft committed buffer poisoned");
            for e in entries {
                match WalRecord::decode(&e.command) {
                    Ok(rec) => {
                        buf.push((e.index, rec));
                        last = e.index;
                    }
                    Err(err) => {
                        tracing::error!(index = e.index, error = %err, "raft: undecodable committed entry");
                    }
                }
            }
        }
        if last > 0 {
            let _ = self.commit_tx.send(last);
        }
    }
}

/// Drives a single-shard Raft group for lumen and surfaces its committed log to
/// the WAL seam.
pub struct RaftDriver {
    shared: Arc<Shared>,
    tick: JoinHandle<()>,
}

impl Drop for RaftDriver {
    fn drop(&mut self) {
        self.tick.abort();
    }
}

impl RaftDriver {
    /// Build a driver for node `id`, recovering persisted state if present, and
    /// start its tick task. `peers` is empty for the single-node slice.
    pub fn spawn(
        id: NodeId,
        membership: Membership,
        _peers: HashMap<NodeId, String>,
        store: RaftStore,
    ) -> RaftDriver {
        let node = match store.load().ok().flatten() {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        let (commit_tx, _rx) = watch::channel(0u64);
        let shared = Arc::new(Shared {
            node: Mutex::new(node),
            store,
            committed: Arc::new(StdMutex::new(Vec::new())),
            commit_tx,
        });
        let s = Arc::clone(&shared);
        let tick = tokio::spawn(async move {
            loop {
                tokio::time::sleep(TICK).await;
                let mut n = s.node.lock().await;
                n.tick();
                s.persist(&n);
                s.surface(&mut n);
            }
        });
        RaftDriver { shared, tick }
    }

    /// Shared handle to the surfaced committed log (for `RaftWal::subscribe`).
    pub fn committed(&self) -> Committed {
        Arc::clone(&self.shared.committed)
    }

    /// A receiver that fires when the committed head advances.
    pub fn commit_watch(&self) -> watch::Receiver<u64> {
        self.shared.commit_tx.subscribe()
    }

    /// Highest committed (surfaced) seq, or 0.
    pub fn latest_committed(&self) -> u64 {
        self.shared
            .committed
            .lock()
            .ok()
            .and_then(|b| b.last().map(|(i, _)| *i))
            .unwrap_or(0)
    }

    /// Propose `cmd` on the leader and return its seq once committed. Single-node:
    /// waits a few ticks for self-election, then `propose` commits immediately.
    pub async fn propose_committed(&self, cmd: Vec<u8>) -> Result<u64> {
        let s = &self.shared;

        // Wait for leadership, then propose + persist + surface.
        let deadline = Instant::now() + Duration::from_secs(10);
        let index = loop {
            {
                let mut n = s.node.lock().await;
                if n.is_leader() {
                    let idx = n
                        .propose(cmd)
                        .ok_or_else(|| anyhow!("raft: lost leadership during propose"))?;
                    s.persist(&n);
                    s.surface(&mut n);
                    break idx;
                }
            }
            if Instant::now() >= deadline {
                bail!("raft: no leader elected (cluster not ready)");
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        };

        // Wait for the entry to commit (single-node: already committed above).
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            {
                let mut n = s.node.lock().await;
                s.surface(&mut n);
                if n.commit_index() >= index {
                    return Ok(index);
                }
            }
            if Instant::now() >= deadline {
                bail!("raft: commit timeout at index {index}");
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

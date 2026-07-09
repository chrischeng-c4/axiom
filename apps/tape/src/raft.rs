// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:a53592d8" tracker="pending-tracker" reason="TapeCommand::Append { topic, key, payload, timestamp_ms } / TapeCommand::CheckpointPut { topic, consumer, offset, updated_at_ms } (the replicated commands, both time fields resolved by the caller before proposing so every replica computes the identical value); TapeOutcome::{Appended(TapeEvent), Checkpoint(Result<ConsumerCheckpoint, TapeError>)} (local-only, claimed from an OutcomeWindow, never serialized over the wire); TapeStateMachine (apply = lock the shared Arc<Mutex<TapeJournal>> and call the unchanged journal.append / journal.put_checkpoint_at, stash the outcome, persist the fsynced applied-<node>.idx marker; snapshot/restore = whole-journal serde_json tagged with the applied index; applied_index recovered from the marker at construction); TapeRaft (single-group wrapper: RaftStore::open on {data_dir}/raft, RaftHost::spawn, router() passthrough, propose_append/propose_checkpoint = propose + claim outcome, from_topology(ClusterTopology) constructor, is_leader/leader/applied_index accessors, host_config(snapshot_every))."
//! raft-host-backed consensus for tape (#1327).
//!
//! `apps/tape`'s journal is wired as a [`raft_host::RaftStateMachine`] so HA
//! append/checkpoint-put go through the shared driver (propose -> commit ->
//! sole applier) instead of a hand-rolled one. tape is a **single-group**
//! adopter (like relay, unlike keep's host-per-shard): one [`RaftHost`]
//! replicates every topic's appends and every consumer's checkpoints; the
//! command is [`TapeCommand`] (append or checkpoint-put).
//!
//! Replication scope (deliberate): the whole [`crate::TapeJournal`] — append
//! AND checkpoint-put both propose through raft in replica/HA mode. Reads
//! (`replay` / `checkpoint_get`) stay node-local against the same shared
//! journal the state machine mutates.
//!
//! Restart honesty: the host cold-replays resident committed entries into the
//! state machine. tape's journal is pure-append (no delete-on-ack), so a naive
//! replay of an `Append` is merely wasteful, not lossy or duplicating (offsets
//! are assigned by journal length at apply time, and the marker below
//! prevents replaying an already-applied entry at all) -- but a naive replay
//! of a `CheckpointPut` could re-run an already-applied checkpoint against a
//! journal whose end offset changed, corrupting the stale/beyond-end
//! invariants. [`TapeStateMachine`] therefore persists its applied index to a
//! small fsynced marker file in the raft data dir (relay #1207's proven
//! recipe, not keep's derive-at-recovery) and skips entries at or below the
//! recovered floor.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::Router;
use raft_host::{
    ClusterTopology, FsyncPolicy, HostConfig, Index, Membership, NodeId, OutcomeWindow, RaftHost,
    RaftStateMachine, RaftStore, SnapshotPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ConsumerCheckpoint, TapeError, TapeEvent, TapeJournal};

/// How many applied entries between host snapshots (log compaction; arms
/// InstallSnapshot for a lagging/fresh replica).
pub const SNAPSHOT_EVERY: u64 = 1024;

/// A tape write replicated through raft -- the command bytes of one log
/// entry. Both time-dependent fields (`timestamp_ms` / `updated_at_ms`) are
/// resolved by the proposing handler BEFORE the command is encoded, never
/// inside [`TapeStateMachine::apply`], so every replica computes the
/// identical value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TapeCommand {
    Append {
        topic: String,
        key: Option<String>,
        payload: serde_json::Value,
        timestamp_ms: u64,
    },
    CheckpointPut {
        topic: String,
        consumer: String,
        offset: u64,
        updated_at_ms: u64,
    },
}

/// The local-only apply outcome, claimed from an [`OutcomeWindow`] by the
/// proposing handler. Never serialized over the wire -- only [`TapeCommand`]
/// crosses the raft log.
#[derive(Debug, Clone)]
pub enum TapeOutcome {
    Appended(TapeEvent),
    Checkpoint(Result<ConsumerCheckpoint, TapeError>),
}

/// Whole-journal snapshot tagged with the raft applied index. A full-state
/// snapshot (not a live/un-acked subset like relay's) is correct here because
/// tape's journal never trims history.
#[derive(Debug, Serialize, Deserialize)]
struct JournalSnapshot {
    up_to: Index,
    journal: TapeJournal,
}

/// tape's [`TapeJournal`] driven as a [`RaftStateMachine`].
///
/// `apply` calls the SAME validated [`TapeJournal::append`] /
/// [`TapeJournal::put_checkpoint_at`] methods the single-node path uses --
/// append-ordering, retention, and stale-checkpoint semantics are unchanged --
/// and stashes the outcome in an [`OutcomeWindow`] keyed by raft index so the
/// proposing handler can return the real domain result (read-your-write). The
/// applied index is mirrored to a fsynced marker file: the restart floor.
///
/// Restart durability (beyond the floor): unlike relay's engine, which
/// persists directly to its OWN disk segments independent of raft, tape's
/// journal here is purely in-memory -- the raft log resident on disk is not
/// itself replayed once an entry is at/below the recovered floor (that is the
/// whole point of the floor: it stops double-apply). So the floor alone would
/// silently lose journal content across a real process restart. To close
/// that gap without waiting on raft-host's own (peer-to-peer-only)
/// InstallSnapshot plumbing, [`Self::persist_marker`] also writes a
/// full-journal snapshot file (`snapshot-<node>.json`, sibling to the
/// marker), and [`Self::new`] restores it before recovering the floor -- a
/// restart therefore recovers the SAME journal content the previous run had,
/// not just its raft bookkeeping.
pub struct TapeStateMachine {
    journal: Arc<Mutex<TapeJournal>>,
    applied: AtomicU64,
    /// `applied-<node>.idx` in the raft data dir; `None` = floor is in-memory
    /// only (embedded/test single-process use) and no snapshot is persisted
    /// either.
    marker: Option<PathBuf>,
    outcomes: Mutex<OutcomeWindow<TapeOutcome>>,
}

/// The sibling snapshot file path for a given marker path
/// (`applied-<node>.idx` -> `snapshot-<node>.json`, same directory).
fn snapshot_path_for(marker: &Path) -> PathBuf {
    let name = marker
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("applied.idx");
    let name = name.replacen("applied-", "snapshot-", 1);
    let name = name
        .strip_suffix(".idx")
        .map(|stem| format!("{stem}.json"))
        .unwrap_or_else(|| format!("{name}.json"));
    marker.with_file_name(name)
}

impl TapeStateMachine {
    /// Wrap `journal` as the group's state machine. When `marker` is set,
    /// recovers a previous run's full journal content from the sibling
    /// snapshot file (if one was persisted) and, either way, the applied
    /// floor from the marker file itself (a snapshot always implies its own
    /// `up_to` floor, so this is really "prefer the snapshot's floor, fall
    /// back to the bare marker when no snapshot exists yet").
    pub fn new(journal: Arc<Mutex<TapeJournal>>, marker: Option<PathBuf>) -> Result<Arc<Self>> {
        let mut applied = 0u64;
        if let Some(path) = &marker {
            let snap_path = snapshot_path_for(path);
            match std::fs::read(&snap_path) {
                Ok(bytes) => {
                    let snap: JournalSnapshot = serde_json::from_slice(&bytes)
                        .with_context(|| format!("corrupt journal snapshot {}", snap_path.display()))?;
                    *journal.lock().expect("journal mutex poisoned") = snap.journal;
                    applied = snap.up_to;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("read journal snapshot"),
            }
            // The marker floor wins if it is ahead of the snapshot's `up_to`
            // (e.g. a snapshot write failed on a later apply than the last
            // successful one) -- never regress the floor.
            match std::fs::read_to_string(path) {
                Ok(s) => {
                    let marker_floor = s
                        .trim()
                        .parse::<u64>()
                        .with_context(|| format!("corrupt applied marker {}", path.display()))?;
                    applied = applied.max(marker_floor);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("read applied marker"),
            }
        }
        Ok(Arc::new(Self {
            journal,
            applied: AtomicU64::new(applied),
            marker,
            outcomes: Mutex::new(OutcomeWindow::default()),
        }))
    }

    /// Durably record the applied floor AND the full journal snapshot
    /// (atomic temp-write + fsync + rename, same recipe for both files).
    /// Best-effort: a persist failure degrades to at-least-once replay /
    /// stale-content recovery on the next restart (logged), it never stalls
    /// the apply loop.
    ///
    /// Writing the whole journal on every apply is a deliberate
    /// correctness-first tradeoff for this slice (proving no committed event
    /// loss matters more than write amplification here); throttling this to
    /// every `SNAPSHOT_EVERY` applies, mirroring raft-host's own log
    /// compaction cadence, is a reasonable follow-up once tape's journals are
    /// large enough for it to matter.
    fn persist_marker(&self, index: Index) {
        let Some(path) = &self.marker else { return };
        let write_marker = || -> std::io::Result<()> {
            let tmp = path.with_extension("tmp");
            {
                let mut f = std::fs::File::create(&tmp)?;
                f.write_all(index.to_string().as_bytes())?;
                f.sync_all()?;
            }
            std::fs::rename(&tmp, path)
        };
        if let Err(e) = write_marker() {
            tracing::warn!(index, error = %e, "raft: applied-marker persist failed (floor stale-low; entries at/below it may re-apply on restart)");
        }

        let snap_path = snapshot_path_for(path);
        let write_snapshot = || -> Result<()> {
            let journal = self.journal.lock().expect("journal mutex poisoned").clone();
            let bytes = serde_json::to_vec(&JournalSnapshot {
                up_to: index,
                journal,
            })?;
            let tmp = snap_path.with_extension("tmp");
            {
                let mut f = std::fs::File::create(&tmp)?;
                f.write_all(&bytes)?;
                f.sync_all()?;
            }
            std::fs::rename(&tmp, &snap_path)?;
            Ok(())
        };
        if let Err(e) = write_snapshot() {
            tracing::warn!(index, error = %e, "raft: journal-snapshot persist failed (restart may recover stale journal content)");
        }
    }

    /// Remove and return the apply outcome at `index` (the proposing handler
    /// claims it once; unclaimed outcomes age out).
    pub fn claim_outcome(&self, index: Index) -> Option<TapeOutcome> {
        self.outcomes.lock().expect("outcome window").claim(index)
    }

    /// The journal this state machine applies into.
    pub fn journal(&self) -> Arc<Mutex<TapeJournal>> {
        Arc::clone(&self.journal)
    }
}

impl RaftStateMachine for TapeStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        // Restart floor: entries at or below the recovered marker were
        // already applied by a previous run -- skip them so cold-replay never
        // double-applies an append or re-runs a checkpoint out of order.
        if index <= self.applied.load(Ordering::Acquire) && self.marker.is_some() {
            return Ok(());
        }
        match serde_json::from_slice::<TapeCommand>(command) {
            Ok(TapeCommand::Append {
                topic,
                key,
                payload,
                timestamp_ms,
            }) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let event = journal.append(topic, key, payload, Some(timestamp_ms));
                drop(journal);
                let mut w = self.outcomes.lock().expect("outcome window");
                w.insert(index, TapeOutcome::Appended(event));
                w.advance(index);
            }
            Ok(TapeCommand::CheckpointPut {
                topic,
                consumer,
                offset,
                updated_at_ms,
            }) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let result = journal.put_checkpoint_at(topic, consumer, offset, updated_at_ms);
                drop(journal);
                let mut w = self.outcomes.lock().expect("outcome window");
                w.insert(index, TapeOutcome::Checkpoint(result));
                w.advance(index);
            }
            Err(e) => {
                tracing::warn!(index, error = %e, "raft: undecodable command (entry no-ops)")
            }
        }
        self.applied.store(index, Ordering::Release);
        self.persist_marker(index);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let journal = self.journal.lock().expect("journal mutex poisoned").clone();
        Ok(serde_json::to_vec(&JournalSnapshot {
            up_to: self.applied_index(),
            journal,
        })?)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snap: JournalSnapshot = serde_json::from_slice(snapshot)?;
        *self.journal.lock().expect("journal mutex poisoned") = snap.journal;
        self.applied.store(snap.up_to, Ordering::Release);
        self.persist_marker(snap.up_to);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

/// One running raft group over a tape journal -- the single-group wrapper the
/// serve path (and tests) hold. Dropping it aborts the host's tick/pump tasks.
pub struct TapeRaft {
    host: RaftHost,
    sm: Arc<TapeStateMachine>,
}

impl TapeRaft {
    /// Spawn the group for node `node_id`, persisting raft hard state + the
    /// applied marker under `raft_dir` (created if needed). `peers` maps the
    /// other members to base URLs (empty => single-node).
    pub fn spawn(
        journal: Arc<Mutex<TapeJournal>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
    ) -> Result<TapeRaft> {
        std::fs::create_dir_all(raft_dir)?;
        let dir = raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?;
        let store = RaftStore::open(dir, node_id, FsyncPolicy::Always)?;
        let sm = TapeStateMachine::new(journal, Some(raft_dir.join(format!("applied-{node_id}.idx"))))?;
        let host = RaftHost::spawn(
            node_id,
            membership,
            peers,
            store,
            Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
            cfg,
        );
        Ok(TapeRaft { host, sm })
    }

    /// Spawn from a k8s-derived [`ClusterTopology`] (the auto-mode serve
    /// path); raft state lives under `{data_dir}/raft`.
    pub fn from_topology(
        journal: Arc<Mutex<TapeJournal>>,
        data_dir: &Path,
        topo: &ClusterTopology,
        cfg: HostConfig,
    ) -> Result<TapeRaft> {
        Self::spawn(
            journal,
            &data_dir.join("raft"),
            topo.node_id,
            topo.membership.clone(),
            topo.peers.clone(),
            cfg,
        )
    }

    /// The standard host tuning for tape: default timing + compaction every
    /// `snapshot_every` applied entries (tests pass a small threshold to arm
    /// InstallSnapshot quickly).
    pub fn host_config(snapshot_every: u64) -> HostConfig {
        HostConfig {
            snapshot: SnapshotPolicy::EveryEntries(snapshot_every),
            ..HostConfig::default()
        }
    }

    /// Peer raft RPCs + leader forward + `/raftz` -- merge onto the serve port
    /// OUTSIDE the bearer-auth data plane (cluster traffic, like probes).
    pub fn router(&self) -> Router {
        self.host.router()
    }

    /// Propose an append (leader-local or forwarded to the leader by the
    /// host) and claim the appended event once THIS node applied it
    /// (read-your-write). `None` means the outcome aged out of the window.
    pub async fn propose_append(
        &self,
        topic: String,
        key: Option<String>,
        payload: serde_json::Value,
        timestamp_ms: u64,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let cmd = TapeCommand::Append {
            topic,
            key,
            payload,
            timestamp_ms,
        };
        let index = self.host.propose(serde_json::to_vec(&cmd)?).await?;
        Ok((index, self.sm.claim_outcome(index)))
    }

    /// Propose a checkpoint-put and claim the outcome once THIS node applied
    /// it.
    pub async fn propose_checkpoint(
        &self,
        topic: String,
        consumer: String,
        offset: u64,
        updated_at_ms: u64,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let cmd = TapeCommand::CheckpointPut {
            topic,
            consumer,
            offset,
            updated_at_ms,
        };
        let index = self.host.propose(serde_json::to_vec(&cmd)?).await?;
        Ok((index, self.sm.claim_outcome(index)))
    }

    pub async fn is_leader(&self) -> bool {
        self.host.is_leader().await
    }

    pub async fn leader(&self) -> Option<NodeId> {
        self.host.leader().await
    }

    /// Highest raft index this node's journal has applied.
    pub fn applied_index(&self) -> Index {
        self.sm.applied_index()
    }

    /// The journal this group replicates into.
    pub fn journal(&self) -> Arc<Mutex<TapeJournal>> {
        self.sm.journal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journal() -> Arc<Mutex<TapeJournal>> {
        Arc::new(Mutex::new(TapeJournal::default()))
    }

    #[test]
    fn apply_append_stashes_claimable_outcome() {
        let sm = TapeStateMachine::new(journal(), None).unwrap();
        let cmd = TapeCommand::Append {
            topic: "orders".into(),
            key: None,
            payload: serde_json::json!({"n": 1}),
            timestamp_ms: 100,
        };
        sm.apply(1, &serde_json::to_vec(&cmd).unwrap()).unwrap();
        assert_eq!(sm.applied_index(), 1);
        let outcome = sm.claim_outcome(1).expect("outcome stashed");
        match outcome {
            TapeOutcome::Appended(event) => {
                assert_eq!(event.topic, "orders");
                assert_eq!(event.offset, 0);
                assert_eq!(event.timestamp_ms, 100);
            }
            _ => panic!("expected Appended outcome"),
        }
        // Claiming twice returns None.
        assert!(sm.claim_outcome(1).is_none());
    }

    #[test]
    fn apply_checkpoint_put_surfaces_stale_rejection() {
        let j = journal();
        j.lock()
            .unwrap()
            .append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let sm = TapeStateMachine::new(j, None).unwrap();
        let cmd = TapeCommand::CheckpointPut {
            topic: "orders".into(),
            consumer: "c1".into(),
            offset: 5,
            updated_at_ms: 200,
        };
        sm.apply(1, &serde_json::to_vec(&cmd).unwrap()).unwrap();
        match sm.claim_outcome(1).expect("outcome") {
            TapeOutcome::Checkpoint(Err(TapeError::CheckpointBeyondEnd { .. })) => {}
            other => panic!("expected CheckpointBeyondEnd, got {other:?}"),
        }
    }

    #[test]
    fn snapshot_restore_round_trips_whole_journal() {
        let j = journal();
        j.lock()
            .unwrap()
            .append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let sm = TapeStateMachine::new(j, None).unwrap();
        sm.applied.store(3, Ordering::Release);
        let bytes = sm.snapshot().unwrap();

        let fresh = TapeStateMachine::new(journal(), None).unwrap();
        fresh.restore(&bytes).unwrap();
        assert_eq!(fresh.applied_index(), 3);
        assert_eq!(fresh.journal().lock().unwrap().end_offset("orders"), 1);
    }

    #[test]
    fn applied_floor_recovered_from_marker_skips_stale_replay() {
        let dir = std::env::temp_dir().join(format!("tape-sm-marker-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("applied-0.idx");

        {
            let sm = TapeStateMachine::new(journal(), Some(marker.clone())).unwrap();
            let cmd = TapeCommand::Append {
                topic: "orders".into(),
                key: None,
                payload: serde_json::json!({"n": 1}),
                timestamp_ms: 100,
            };
            sm.apply(1, &serde_json::to_vec(&cmd).unwrap()).unwrap();
            assert_eq!(sm.applied_index(), 1);
        }

        // Fresh state machine over a fresh (empty) journal recovers BOTH the
        // floor AND the actual journal content from the sibling snapshot file
        // persist_marker wrote alongside the marker -- a real restart must
        // not lose committed content, only avoid double-applying it.
        let sm2 = TapeStateMachine::new(journal(), Some(marker.clone())).unwrap();
        assert_eq!(sm2.applied_index(), 1);
        assert_eq!(sm2.journal().lock().unwrap().end_offset("orders"), 1);

        let cmd = TapeCommand::Append {
            topic: "orders".into(),
            key: None,
            payload: serde_json::json!({"n": 1}),
            timestamp_ms: 100,
        };
        sm2.apply(1, &serde_json::to_vec(&cmd).unwrap()).unwrap();
        // Skipped: re-applying the same (already-recovered) index does not
        // double the entry.
        assert_eq!(sm2.journal().lock().unwrap().end_offset("orders"), 1);

        std::fs::remove_dir_all(&dir).ok();
    }
}
// HANDWRITE-END

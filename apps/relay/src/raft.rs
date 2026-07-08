// SPEC-MANAGED: apps/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:786bf09f" tracker="pending-tracker" reason="raft-host adoption surface (#544): PubCommand (the replicated multi-subject publish), RelayStateMachine (apply = idempotent engine publish + OutcomeWindow + fsynced applied-index marker; snapshot/restore = live-state dump/load), and RelayRaft (single-group RaftHost wrapper: store on {data_dir}/raft, router passthrough, propose-publish with outcome claim)."
//! raft-host-backed consensus for relay (#544).
//!
//! relay's engine is wired as a [`raft_host::RaftStateMachine`] so HA publishes
//! go through the shared driver (propose → commit → sole applier) instead of
//! the hand-rolled driver/store/topology stack this module replaces (#544).
//! relay is a **single-group** adopter (like lumen, unlike keep's
//! host-per-shard): one [`RaftHost`] replicates every subject's publishes; the
//! command is a multi-subject [`PubCommand`] — an upgrade over the old driver,
//! which was pinned to one subject per process.
//!
//! Replication scope (deliberate, unchanged from the old driver): **publishes
//! only**. Leases/acks/consume stay node-local, so a failover redelivers work
//! that was leased-but-unacked on the old leader (and acked work a follower
//! has not yet trimmed via a snapshot install) — at-least-once, fenced
//! per-node by lease epochs.
//!
//! Restart honesty: the host cold-replays resident committed entries into the
//! state machine. relay's engine is delete-on-ack with a bounded dedupe
//! window, so replaying an old committed publish whose entry was already
//! acked+trimmed would *resurrect* finished work. [`RelayStateMachine`]
//! therefore persists its applied index to a small fsynced marker file in the
//! raft data dir and skips entries at or below the recovered floor. With the
//! default `FsyncPolicy::Always` engine, the append is durable before the
//! marker advances, so the floor never runs ahead of engine state; a crash
//! between the two at worst re-applies one entry that is still in the dedupe
//! window (deduped) — at-least-once either way.

use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::Router;
use chrono::{DateTime, Utc};
use raft_host::{
    ClusterTopology, FsyncPolicy, HostConfig, Index, Membership, NodeId, OutcomeWindow, RaftHost,
    RaftStateMachine, RaftStore, SnapshotPolicy,
};
use serde::{Deserialize, Serialize};

use crate::engine::{Relay, SubjectLive};
use crate::types::{default_priority, AppendOutcome, Payload};

/// How many applied entries between host snapshots (log compaction; arms
/// InstallSnapshot for a lagging/fresh replica). Snapshots serialize only the
/// live (un-acked) backlog, so the capture cost tracks consumer lag, not
/// publish volume.
pub const SNAPSHOT_EVERY: u64 = 1024;

/// A publish replicated through raft — the command bytes of one log entry.
/// Multi-subject: the subject rides in the command (the old driver replicated
/// a single fixed subject per process).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubCommand {
    pub subject: String,
    pub message_id: String,
    pub payload: Payload,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default = "default_priority")]
    pub priority: u8,
    /// Work-queue visibility gate (delayed delivery), replicated verbatim.
    #[serde(default)]
    pub not_before: Option<DateTime<Utc>>,
}

/// Full state-machine snapshot: the live (un-acked) engine state as of raft
/// index `up_to`. Restore is an idempotent merge (message_id dedupe), which is
/// sound because raft only installs a snapshot on a replica whose applied
/// publish stream is a prefix of the leader's.
///
/// Public since #1209: the `GET /admin/backup` endpoint and the `relay backup`
/// artifact carry EXACTLY this serialization (via [`snapshot_bytes`] /
/// [`load_snapshot_bytes`]) — one snapshot format, shared with raft's
/// InstallSnapshot path, never a parallel backup format.
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSnapshot {
    /// Raft applied index the capture reflects (0 on a raft-less single node).
    pub up_to: Index,
    /// Per-`(subject, shard)` live (un-acked) backlog, deterministically
    /// ordered.
    pub subjects: Vec<SubjectLive>,
}

/// Serialize the ONE snapshot format: the engine's live (un-acked) state via
/// [`Relay::dump_live`] plus the applied raft index. This is what
/// [`RelayStateMachine::snapshot`] produces for raft InstallSnapshot and what
/// `GET /admin/backup` streams for `relay backup` (#1209).
pub fn snapshot_bytes(relay: &Relay, up_to: Index) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&EngineSnapshot {
        up_to,
        subjects: relay.dump_live()?,
    })?)
}

/// Load [`snapshot_bytes`] output into `relay` via the [`Relay::load_live`]
/// MERGE (idempotent per message_id; entries the target already holds dedupe,
/// surplus local entries stay as redelivery candidates — at-least-once).
/// Returns the snapshot's applied index (`up_to`) so callers — raft restore,
/// or a fresh-node restore from a backup artifact — can advance their floor.
pub fn load_snapshot_bytes(relay: &Relay, snapshot: &[u8]) -> Result<Index> {
    let snap: EngineSnapshot = serde_json::from_slice(snapshot)?;
    relay.load_live(snap.subjects)?;
    Ok(snap.up_to)
}

/// relay's [`Relay`] engine driven as a [`RaftStateMachine`].
///
/// `apply` publishes idempotently through the engine and stashes the outcome
/// (`{seq, deduped}`) in an [`OutcomeWindow`] keyed by raft index so the
/// proposing handler can return the engine outcome (read-your-write). The
/// applied index is mirrored to a fsynced marker file — the restart floor (see
/// the module docs for why relay, unlike keep, needs a durable floor).
pub struct RelayStateMachine {
    relay: Arc<Relay>,
    applied: AtomicU64,
    /// `applied-<node>.idx` in the raft data dir; `None` = floor is in-memory
    /// only (embedded/test single-process use).
    marker: Option<PathBuf>,
    outcomes: Mutex<OutcomeWindow<AppendOutcome>>,
}

impl RelayStateMachine {
    /// Wrap `relay` as the group's state machine, recovering the applied floor
    /// from `marker` if a previous run persisted one.
    pub fn new(relay: Arc<Relay>, marker: Option<PathBuf>) -> Result<Arc<Self>> {
        let applied = match &marker {
            Some(path) => match std::fs::read_to_string(path) {
                Ok(s) => s
                    .trim()
                    .parse::<u64>()
                    .with_context(|| format!("corrupt applied marker {}", path.display()))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
                Err(e) => return Err(e).context("read applied marker"),
            },
            None => 0,
        };
        Ok(Arc::new(Self {
            relay,
            applied: AtomicU64::new(applied),
            marker,
            outcomes: Mutex::new(OutcomeWindow::default()),
        }))
    }

    /// Durably record the applied floor (atomic temp-write + fsync + rename).
    /// Best-effort: a persist failure degrades to at-least-once on the next
    /// restart (logged), it never stalls the apply loop.
    fn persist_marker(&self, index: Index) {
        let Some(path) = &self.marker else { return };
        let write = || -> std::io::Result<()> {
            let tmp = path.with_extension("tmp");
            {
                let mut f = std::fs::File::create(&tmp)?;
                f.write_all(index.to_string().as_bytes())?;
                f.sync_all()?;
            }
            std::fs::rename(&tmp, path)
        };
        if let Err(e) = write() {
            tracing::warn!(index, error = %e, "raft: applied-marker persist failed (floor stale-low; at-least-once on restart)");
        }
    }

    /// Remove and return the engine outcome of the apply at `index` (the
    /// proposing handler claims it once; unclaimed outcomes age out).
    pub fn claim_outcome(&self, index: Index) -> Option<AppendOutcome> {
        self.outcomes.lock().expect("outcome window").claim(index)
    }

    /// The engine this state machine applies into.
    pub fn relay(&self) -> Arc<Relay> {
        Arc::clone(&self.relay)
    }
}

impl RaftStateMachine for RelayStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        // Decode and publish through the normal engine path (idempotent per
        // message_id). A bad decode / engine error no-ops the entry (logged)
        // but still advances the floor so the log keeps moving (sole applier).
        match serde_json::from_slice::<PubCommand>(command) {
            Ok(cmd) => {
                match self.relay.publish_at(
                    &cmd.subject,
                    &cmd.message_id,
                    cmd.payload,
                    cmd.headers,
                    cmd.not_before,
                    cmd.priority,
                    Utc::now(),
                ) {
                    Ok(outcome) => {
                        let mut w = self.outcomes.lock().expect("outcome window");
                        w.insert(index, outcome);
                        w.advance(index);
                    }
                    Err(e) => {
                        tracing::warn!(index, error = %e, "raft: apply publish error (entry no-ops)")
                    }
                }
            }
            Err(e) => tracing::warn!(index, error = %e, "raft: undecodable command (entry no-ops)"),
        }
        self.applied.store(index, Ordering::Release);
        self.persist_marker(index);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        snapshot_bytes(&self.relay, self.applied_index())
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let up_to = load_snapshot_bytes(&self.relay, snapshot)?;
        self.applied.store(up_to, Ordering::Release);
        self.persist_marker(up_to);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

/// One running raft group over a relay engine — the single-group wrapper the
/// serve path (and tests) hold. Dropping it aborts the host's tick/pump tasks.
pub struct RelayRaft {
    host: RaftHost,
    sm: Arc<RelayStateMachine>,
}

impl RelayRaft {
    /// Spawn the group for node `node_id`, persisting raft hard state + the
    /// applied marker under `raft_dir` (created if needed). `peers` maps the
    /// other members to base URLs (empty ⇒ single-node).
    pub fn spawn(
        relay: Arc<Relay>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
    ) -> Result<RelayRaft> {
        std::fs::create_dir_all(raft_dir)?;
        let dir = raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?;
        let store = RaftStore::open(dir, node_id, FsyncPolicy::Always)?;
        let sm =
            RelayStateMachine::new(relay, Some(raft_dir.join(format!("applied-{node_id}.idx"))))?;
        let host = RaftHost::spawn(
            node_id,
            membership,
            peers,
            store,
            Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
            cfg,
        );
        Ok(RelayRaft { host, sm })
    }

    /// Spawn from a k8s-derived [`ClusterTopology`] (the auto-mode serve
    /// path); raft state lives under `{data_dir}/raft`.
    pub fn from_topology(
        relay: Arc<Relay>,
        data_dir: &Path,
        topo: &ClusterTopology,
        cfg: HostConfig,
    ) -> Result<RelayRaft> {
        Self::spawn(
            relay,
            &data_dir.join("raft"),
            topo.node_id,
            topo.membership.clone(),
            topo.peers.clone(),
            cfg,
        )
    }

    /// The standard host tuning for relay: default timing + compaction every
    /// `snapshot_every` applied entries (tests pass a small threshold to arm
    /// InstallSnapshot quickly).
    pub fn host_config(snapshot_every: u64) -> HostConfig {
        HostConfig {
            snapshot: SnapshotPolicy::EveryEntries(snapshot_every),
            ..HostConfig::default()
        }
    }

    /// Peer raft RPCs + leader forward + `/raftz` — merge onto the serve port
    /// OUTSIDE the bearer-auth data plane (cluster traffic, like probes).
    pub fn router(&self) -> Router {
        self.host.router()
    }

    /// Propose a publish (leader-local or forwarded to the leader by the
    /// host) and claim the engine outcome once THIS node applied it
    /// (read-your-write). `None` outcome means it aged out of the window —
    /// the caller resolves idempotently through the engine.
    pub async fn publish(&self, cmd: &PubCommand) -> Result<(Index, Option<AppendOutcome>)> {
        let index = self.host.propose(serde_json::to_vec(cmd)?).await?;
        Ok((index, self.sm.claim_outcome(index)))
    }

    pub async fn is_leader(&self) -> bool {
        self.host.is_leader().await
    }

    pub async fn leader(&self) -> Option<NodeId> {
        self.host.leader().await
    }

    /// Highest raft index this node's engine has applied.
    pub fn applied_index(&self) -> Index {
        self.sm.applied_index()
    }

    /// The engine this group replicates into.
    pub fn relay(&self) -> Arc<Relay> {
        self.sm.relay()
    }
}
// HANDWRITE-END

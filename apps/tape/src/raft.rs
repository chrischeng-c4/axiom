// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:a53592d8" tracker="pending-tracker" reason="Tape's deterministic Raft commands/outcomes, whole-journal snapshots, shared RaftStore recovery, proposal dedupe, and single-group wrapper remain service-owned domain integration."
//! raft-runtime-backed consensus for tape (#1327).
//!
//! `apps/tape`'s journal is wired as a [`raft_runtime::RaftStateMachine`] so HA
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
//! Restart honesty: `RaftStore` persists the commit watermark with hard state,
//! so the host cold-replays every resident committed entry into a fresh state
//! machine before accepting new proposals. Host snapshots restore the whole
//! journal before log tailing. Old `applied-*.idx`/`snapshot-*.json` files are
//! read only as a migration path; new runs do not duplicate generic commit
//! persistence or per-apply fsyncs inside Tape.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::Router;
use raft_runtime::{
    ClusterTopology, FsyncPolicy, HostConfig, Index, Membership, NodeId, OutcomeWindow,
    PeerTransport, ProposalCache, RaftHost, RaftStateMachine, RaftStore, SnapshotPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    ConsumerCheckpoint, RetentionOutcome, RetentionPolicy, Subscription, SubscriptionAckError,
    SubscriptionError, TapeError, TapeEvent, TapeJournal,
};

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
        #[serde(default)]
        applied_at_ms: u64,
    },
    CheckpointPut {
        topic: String,
        consumer: String,
        offset: u64,
        updated_at_ms: u64,
    },
    SubscriptionCreate {
        topic: String,
        name: String,
    },
    SubscriptionDelete {
        topic: String,
        name: String,
    },
    SubscriptionAck {
        topic: String,
        name: String,
        offset: u64,
        updated_at_ms: u64,
    },
    RetentionPut {
        topic: String,
        policy: RetentionPolicy,
        now_ms: u64,
    },
}

/// The local-only apply outcome, claimed from an [`OutcomeWindow`] by the
/// proposing handler. Never serialized over the wire -- only [`TapeCommand`]
/// crosses the raft log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TapeOutcome {
    Appended(TapeEvent),
    Checkpoint(Result<ConsumerCheckpoint, TapeError>),
    SubscriptionCreated(Result<Subscription, SubscriptionError>),
    SubscriptionDeleted(Result<Subscription, SubscriptionError>),
    SubscriptionAcked(Result<ConsumerCheckpoint, SubscriptionAckError>),
    RetentionUpdated(RetentionOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ProposalId {
    node: NodeId,
    session: u64,
    sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TapeEnvelope {
    proposal_id: ProposalId,
    command: TapeCommand,
}

/// Whole-journal snapshot tagged with the raft applied index. A full-state
/// snapshot (not a live/un-acked subset like relay's) is correct here because
/// tape's journal never trims history.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JournalSnapshot {
    up_to: Index,
    journal: TapeJournal,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    completed_proposals: Vec<(ProposalId, TapeOutcome)>,
}

/// Serialize the SAME whole-journal [`JournalSnapshot`] shape
/// [`TapeStateMachine::snapshot`]/[`TapeStateMachine::restore`] round-trip,
/// callable without a live [`TapeStateMachine`] instance (single-node serving
/// has none). Backs `GET /admin/backup` (#1329): the exact bytes a backup
/// runner ships are the exact bytes a raft group would snapshot.
pub fn snapshot_bytes(journal: &Arc<Mutex<TapeJournal>>, up_to: Index) -> Result<Vec<u8>> {
    let journal = journal.lock().expect("journal mutex poisoned").clone();
    Ok(serde_json::to_vec(&JournalSnapshot {
        up_to,
        journal,
        completed_proposals: Vec::new(),
    })?)
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1812" reason="Tape delegates the generic marker and snapshot atomic-write mechanics to storage-durable while retaining JournalSnapshot serialization and recovery ordering.">
/// Prepare an empty replica PVC to recover from one exact backup object.
///
/// This is deliberately a cold-start-only operation: it refuses any existing
/// content in `data_dir`, decodes the same [`JournalSnapshot`] shape served by
/// `/admin/backup`, then writes the state-machine files that
/// [`TapeRaft::from_topology`] will consume. The snapshot is installed before
/// the Raft host opens its store, so normal Raft log/snapshot catch-up resumes
/// from `up_to` instead of replaying the seed as new appends. It is not a live
/// restore API and must never overwrite a running replica's durable state.
pub fn prepare_bootstrap_seed(data_dir: &Path, node_id: NodeId, bytes: &[u8]) -> Result<()> {
    let snapshot: JournalSnapshot =
        serde_json::from_slice(bytes).context("decode bootstrap JournalSnapshot JSON")?;

    if data_dir.exists() {
        let entries = std::fs::read_dir(data_dir)
            .with_context(|| format!("read bootstrap data dir {}", data_dir.display()))?;
        for entry in entries {
            let entry = entry?;
            // A freshly provisioned cloud PV mounts its ext4 filesystem root
            // directly at the data dir, so `lost+found` is present on every
            // real PVC (#2443). It is mkfs output, not raft state.
            if entry.file_name() == "lost+found" {
                continue;
            }
            anyhow::bail!(
                "bootstrap seed requires an empty data directory {}; refusing to replace existing raft state",
                data_dir.display()
            );
        }
    } else {
        std::fs::create_dir_all(data_dir)
            .with_context(|| format!("create bootstrap data dir {}", data_dir.display()))?;
    }

    let raft_dir = data_dir.join("raft");
    let store = RaftStore::open(
        raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?,
        node_id,
        FsyncPolicy::Always,
    )?;
    store.seed_snapshot(snapshot.up_to, 0, bytes.to_vec())?;
    Ok(())
}
// </HANDWRITE>

/// tape's [`TapeJournal`] driven as a [`RaftStateMachine`].
///
/// `apply` calls the SAME validated [`TapeJournal::append`] /
/// [`TapeJournal::put_checkpoint_at`] methods the single-node path uses --
/// append-ordering, retention, and stale-checkpoint semantics are unchanged --
/// and stashes the outcome in an [`OutcomeWindow`] keyed by raft index so the
/// proposing handler can return the real domain result (read-your-write). The
/// applied index is tracked in memory while shared Raft persistence owns new
/// restart recovery. The optional marker/sibling snapshot fields below exist
/// solely to adopt data produced by the pre-shared-persistence implementation.
pub struct TapeStateMachine {
    journal: Arc<Mutex<TapeJournal>>,
    applied: AtomicU64,
    /// Legacy `applied-<node>.idx` migration source. New runs do not write it.
    marker: Option<PathBuf>,
    outcomes: Mutex<OutcomeWindow<TapeOutcome>>,
    completed: Mutex<ProposalCache<ProposalId, TapeOutcome>>,
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
    /// Wrap `journal` as the group's state machine. `marker` names only the
    /// legacy migration files; current RaftStore log/snapshot recovery is the
    /// durable source for new data.
    pub fn new(journal: Arc<Mutex<TapeJournal>>, marker: Option<PathBuf>) -> Result<Arc<Self>> {
        let mut applied = 0u64;
        let mut recovered_completed = Vec::new();
        if let Some(path) = &marker {
            let snap_path = snapshot_path_for(path);
            match std::fs::read(&snap_path) {
                Ok(bytes) => {
                    let snap: JournalSnapshot =
                        serde_json::from_slice(&bytes).with_context(|| {
                            format!("corrupt journal snapshot {}", snap_path.display())
                        })?;
                    *journal.lock().expect("journal mutex poisoned") = snap.journal;
                    applied = snap.up_to;
                    // Installed below after `Self` is constructed.
                    recovered_completed = snap.completed_proposals;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("read journal snapshot"),
            }
            // The journal snapshot is authoritative. A marker without the
            // matching state must never advance the apply floor or committed
            // events would disappear after restart.
            match std::fs::read_to_string(path) {
                Ok(s) => {
                    let marker_floor = s
                        .trim()
                        .parse::<u64>()
                        .with_context(|| format!("corrupt applied marker {}", path.display()))?;
                    if marker_floor != applied {
                        tracing::warn!(
                            marker_floor,
                            snapshot_floor = applied,
                            "raft: applied marker disagrees with journal snapshot; replaying from snapshot floor"
                        );
                    }
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
            completed: Mutex::new({
                let mut cache = ProposalCache::default();
                cache.restore(recovered_completed);
                cache
            }),
        }))
    }

    /// Remove and return the apply outcome at `index` (the proposing handler
    /// claims it once; unclaimed outcomes age out).
    pub fn claim_outcome(&self, index: Index) -> Option<TapeOutcome> {
        self.outcomes.lock().expect("outcome window").claim(index)
    }

    /// Resolve a committed proposal by its stable id. Unlike the transient
    /// index window, this cache survives ambiguous transport retries and is
    /// snapshotted with the state machine.
    fn proposal_outcome(&self, id: &ProposalId) -> Option<TapeOutcome> {
        self.completed.lock().expect("completed proposals").get(id)
    }

    /// The journal this state machine applies into.
    pub fn journal(&self) -> Arc<Mutex<TapeJournal>> {
        Arc::clone(&self.journal)
    }
}

impl RaftStateMachine for TapeStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        // Legacy migration floor: entries represented by an imported app
        // snapshot were already applied. New stores start at zero and replay
        // their shared persisted commit range normally.
        if index <= self.applied.load(Ordering::Acquire) && self.marker.is_some() {
            return Ok(());
        }
        let (proposal_id, decoded) = match serde_json::from_slice::<TapeEnvelope>(command) {
            Ok(envelope) => (Some(envelope.proposal_id), Ok(envelope.command)),
            Err(envelope_error) => (
                None,
                serde_json::from_slice::<TapeCommand>(command).map_err(|_| envelope_error),
            ),
        };
        let cached = proposal_id
            .as_ref()
            .and_then(|id| self.completed.lock().expect("completed proposals").get(id));
        let outcome = match (cached, decoded) {
            (Some(outcome), _) => Some(outcome),
            (
                None,
                Ok(TapeCommand::Append {
                    topic,
                    key,
                    payload,
                    timestamp_ms,
                    applied_at_ms,
                }),
            ) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let applied_at_ms = if applied_at_ms == 0 {
                    timestamp_ms
                } else {
                    applied_at_ms
                };
                let event = journal.append_at(topic, key, payload, timestamp_ms, applied_at_ms);
                Some(TapeOutcome::Appended(event))
            }
            (
                None,
                Ok(TapeCommand::CheckpointPut {
                    topic,
                    consumer,
                    offset,
                    updated_at_ms,
                }),
            ) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let result = journal.put_checkpoint_at(topic, consumer, offset, updated_at_ms);
                Some(TapeOutcome::Checkpoint(result))
            }
            (None, Ok(TapeCommand::SubscriptionCreate { topic, name })) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                Some(TapeOutcome::SubscriptionCreated(
                    journal.create_subscription(topic, name),
                ))
            }
            (None, Ok(TapeCommand::SubscriptionDelete { topic, name })) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                Some(TapeOutcome::SubscriptionDeleted(
                    journal.delete_subscription(&topic, &name),
                ))
            }
            (
                None,
                Ok(TapeCommand::SubscriptionAck {
                    topic,
                    name,
                    offset,
                    updated_at_ms,
                }),
            ) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                let valid = journal
                    .require_pull_subscription(&topic, &name)
                    .map(|_| ())
                    .map_err(SubscriptionAckError::from);
                let result = match valid {
                    Ok(()) => journal
                        .put_checkpoint_at(topic, name, offset, updated_at_ms)
                        .map_err(SubscriptionAckError::from),
                    Err(error) => Err(error),
                };
                Some(TapeOutcome::SubscriptionAcked(result))
            }
            (
                None,
                Ok(TapeCommand::RetentionPut {
                    topic,
                    policy,
                    now_ms,
                }),
            ) => {
                let mut journal = self.journal.lock().expect("journal mutex poisoned");
                Some(TapeOutcome::RetentionUpdated(
                    journal.put_retention(topic, policy, now_ms),
                ))
            }
            (None, Err(e)) => {
                tracing::warn!(index, error = %e, "raft: undecodable command (entry no-ops)");
                None
            }
        };
        if let Some(outcome) = outcome {
            if let Some(id) = proposal_id {
                self.completed
                    .lock()
                    .expect("completed proposals")
                    .insert(id, outcome.clone());
            }
            let mut window = self.outcomes.lock().expect("outcome window");
            window.insert(index, outcome);
            window.advance(index);
        }
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let journal = self.journal.lock().expect("journal mutex poisoned").clone();
        Ok(serde_json::to_vec(&JournalSnapshot {
            up_to: self.applied_index(),
            journal,
            completed_proposals: self
                .completed
                .lock()
                .expect("completed proposals")
                .snapshot(),
        })?)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snap: JournalSnapshot = serde_json::from_slice(snapshot)?;
        *self.journal.lock().expect("journal mutex poisoned") = snap.journal;
        self.completed
            .lock()
            .expect("completed proposals")
            .restore(snap.completed_proposals);
        self.applied.store(snap.up_to, Ordering::Release);
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
    node_id: NodeId,
    session: u64,
    proposal_sequence: AtomicU64,
}

// <HANDWRITE gap="missing-generator:raft-transport-adapter" tracker="#1805" reason="raft-transport-adapter section in raft.rs is hand-written pending codegen support">
impl TapeRaft {
    /// Spawn the group for node `node_id`, persisting shared Raft hard state,
    /// commit watermark, log, and snapshots under `raft_dir`. `peers` maps the
    /// other members to base URLs (empty => single-node).
    pub fn spawn(
        journal: Arc<Mutex<TapeJournal>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
    ) -> Result<TapeRaft> {
        Self::spawn_inner(journal, raft_dir, node_id, membership, peers, cfg, None)
    }

    /// Spawn a group whose outgoing peer RPCs and incoming Raft listener use
    /// the shared mutually authenticated transport. The caller serves
    /// [`Self::router`] through the same transport on its dedicated port.
    pub fn spawn_with_peer_transport(
        journal: Arc<Mutex<TapeJournal>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
        peer_transport: PeerTransport,
    ) -> Result<TapeRaft> {
        Self::spawn_inner(
            journal,
            raft_dir,
            node_id,
            membership,
            peers,
            cfg,
            Some(peer_transport),
        )
    }

    fn spawn_inner(
        journal: Arc<Mutex<TapeJournal>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
        peer_transport: Option<PeerTransport>,
    ) -> Result<TapeRaft> {
        std::fs::create_dir_all(raft_dir)?;
        let dir = raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?;
        let store = RaftStore::open(dir, node_id, FsyncPolicy::Always)?;
        let sm = TapeStateMachine::new(
            journal,
            Some(raft_dir.join(format!("applied-{node_id}.idx"))),
        )?;
        let host = match peer_transport {
            Some(transport) => RaftHost::spawn_with_peer_transport(
                node_id,
                membership,
                peers,
                store,
                Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
                cfg,
                transport,
            ),
            None => RaftHost::spawn(
                node_id,
                membership,
                peers,
                store,
                Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
                cfg,
            ),
        };
        static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);
        let wall = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let session = wall
            ^ ((std::process::id() as u64) << 32)
            ^ NEXT_SESSION.fetch_add(1, Ordering::Relaxed);
        Ok(TapeRaft {
            host,
            sm,
            node_id,
            session,
            proposal_sequence: AtomicU64::new(1),
        })
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

    /// TLS-aware topology constructor. The topology must carry `https` peer
    /// URLs for the same dedicated listener the caller serves below.
    pub fn from_topology_with_peer_transport(
        journal: Arc<Mutex<TapeJournal>>,
        data_dir: &Path,
        topo: &ClusterTopology,
        cfg: HostConfig,
        peer_transport: PeerTransport,
    ) -> Result<TapeRaft> {
        Self::spawn_with_peer_transport(
            journal,
            &data_dir.join("raft"),
            topo.node_id,
            topo.membership.clone(),
            topo.peers.clone(),
            cfg,
            peer_transport,
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

    /// Drain the shared Raft host's in-flight peer RPCs before its h2 client
    /// and peer listener are torn down.
    pub async fn shutdown(&self) -> Result<()> {
        self.host.shutdown().await
    }

    /// Peer raft RPCs + leader forwarding + `/raftz`. The h2c compatibility
    /// path merges this onto the public app; mTLS serves it independently.
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
            applied_at_ms: crate::now_ms(),
        };
        let (index, proposal_id) = self.propose(cmd).await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
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
        let (index, proposal_id) = self.propose(cmd).await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
    }

    pub async fn propose_subscription_create(
        &self,
        topic: String,
        name: String,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let (index, proposal_id) = self
            .propose(TapeCommand::SubscriptionCreate { topic, name })
            .await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
    }

    pub async fn propose_subscription_delete(
        &self,
        topic: String,
        name: String,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let (index, proposal_id) = self
            .propose(TapeCommand::SubscriptionDelete { topic, name })
            .await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
    }

    pub async fn propose_subscription_ack(
        &self,
        topic: String,
        name: String,
        offset: u64,
        updated_at_ms: u64,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let (index, proposal_id) = self
            .propose(TapeCommand::SubscriptionAck {
                topic,
                name,
                offset,
                updated_at_ms,
            })
            .await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
    }

    pub async fn propose_retention(
        &self,
        topic: String,
        policy: RetentionPolicy,
        now_ms: u64,
    ) -> Result<(Index, Option<TapeOutcome>)> {
        let (index, proposal_id) = self
            .propose(TapeCommand::RetentionPut {
                topic,
                policy,
                now_ms,
            })
            .await?;
        let outcome = self
            .sm
            .proposal_outcome(&proposal_id)
            .or_else(|| self.sm.claim_outcome(index));
        Ok((index, outcome))
    }

    async fn propose(&self, command: TapeCommand) -> Result<(Index, ProposalId)> {
        let envelope = TapeEnvelope {
            proposal_id: ProposalId {
                node: self.node_id,
                session: self.session,
                sequence: self.proposal_sequence.fetch_add(1, Ordering::Relaxed),
            },
            command,
        };
        let proposal_id = envelope.proposal_id.clone();
        let index = self.host.propose(serde_json::to_vec(&envelope)?).await?;
        Ok((index, proposal_id))
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

    /// Capture the exact state-machine snapshot, including the bounded
    /// proposal-outcome cache needed for ambiguous retry idempotency.
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>> {
        self.sm.snapshot()
    }

    /// The journal this group replicates into.
    pub fn journal(&self) -> Arc<Mutex<TapeJournal>> {
        self.sm.journal()
    }
}
// </HANDWRITE>

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
            applied_at_ms: 100,
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
    fn legacy_snapshot_floor_is_still_read_during_migration() {
        let dir = std::env::temp_dir().join(format!("tape-sm-marker-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let marker = dir.join("applied-0.idx");
        let legacy = journal();
        legacy
            .lock()
            .unwrap()
            .append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let bytes = serde_json::to_vec(&JournalSnapshot {
            up_to: 1,
            journal: legacy.lock().unwrap().clone(),
            completed_proposals: Vec::new(),
        })
        .unwrap();
        std::fs::write(snapshot_path_for(&marker), bytes).unwrap();
        std::fs::write(&marker, b"1").unwrap();
        let sm2 = TapeStateMachine::new(journal(), Some(marker.clone())).unwrap();
        assert_eq!(sm2.applied_index(), 1);
        assert_eq!(sm2.journal().lock().unwrap().end_offset("orders"), 1);

        let cmd = TapeCommand::Append {
            topic: "orders".into(),
            key: None,
            payload: serde_json::json!({"n": 1}),
            timestamp_ms: 100,
            applied_at_ms: 100,
        };
        sm2.apply(1, &serde_json::to_vec(&cmd).unwrap()).unwrap();
        // Skipped: re-applying the same (already-recovered) index does not
        // double the entry.
        assert_eq!(sm2.journal().lock().unwrap().end_offset("orders"), 1);

        std::fs::remove_dir_all(&dir).ok();
    }
}
// HANDWRITE-END

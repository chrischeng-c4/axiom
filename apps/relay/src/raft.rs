// SPEC-MANAGED: apps/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:786bf09f" tracker="pending-tracker" reason="raft-runtime adoption surface (#544): PubCommand (the replicated multi-subject publish), RelayStateMachine (apply = idempotent engine publish + OutcomeWindow + fsynced applied-index marker; snapshot/restore = live-state dump/load), and RelayRaft (single-group RaftHost wrapper: store on {data_dir}/raft, router passthrough, propose-publish with outcome claim)."
//! raft-runtime-backed consensus for relay (#544).
//!
//! relay's engine is wired as a [`raft_runtime::RaftStateMachine`] so HA publishes
//! go through the shared driver (propose → commit → sole applier) instead of
//! the hand-rolled driver/store/topology stack this module replaces (#544).
//! relay is a **single-group** adopter (like lumen, unlike keep's
//! host-per-shard): one [`RaftHost`] replicates every subject's publishes; the
//! command is a multi-subject [`PubCommand`] — an upgrade over the old driver,
//! which was pinned to one subject per process.
//!
//! Replication scope covers every authoritative mutation: publish, lease grant,
//! ack/nack, heartbeat, expiry/reclaim, committed watermark, and DLQ transition.
//! Executors may send work only after the assignment commits; owner + epoch
//! fencing rejects stale or cross-replica outcomes. Delivery remains
//! at-least-once for ambiguous external effects and lease-expiry redelivery.
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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::Router;
use chrono::{DateTime, Utc};
use raft_runtime::{
    AppliedIndexStore, ClusterTopology, FsyncPolicy, HostConfig, Index, Membership, NodeId,
    OutcomeWindow, PeerTransport, ProposalCache, RaftHost, RaftStateMachine, RaftStore,
    SnapshotPolicy,
};
use serde::{Deserialize, Serialize};

use crate::engine::{Relay, SubjectLive};
use crate::types::{default_priority, AppendOutcome, CommittedOffset, Lease, Payload};

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
    /// Resolved by the proposer so every replica persists the same timestamp.
    #[serde(default = "unix_epoch")]
    pub appended_at: DateTime<Utc>,
}

fn unix_epoch() -> DateTime<Utc> {
    DateTime::from_timestamp(0, 0).expect("unix epoch")
}

/// Every authoritative Relay mutation replicated through the single Raft group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RelayCommand {
    Publish {
        command: PubCommand,
    },
    PublishBatch {
        subject: String,
        commands: Vec<PubCommand>,
        now: DateTime<Utc>,
    },
    Lease {
        subject: String,
        consumer_id: String,
        executor_node: NodeId,
        now: DateTime<Utc>,
    },
    LeaseBatch {
        subject: String,
        consumer_id: String,
        executor_node: NodeId,
        max: usize,
        now: DateTime<Utc>,
    },
    Ack {
        subject: String,
        lease_id: String,
        executor_node: NodeId,
        epoch: u64,
        now: DateTime<Utc>,
    },
    AckBatch {
        subject: String,
        acks: Vec<(String, u64)>,
        executor_node: NodeId,
        now: DateTime<Utc>,
    },
    Release {
        subject: String,
        lease_id: String,
        executor_node: NodeId,
        epoch: u64,
        now: DateTime<Utc>,
    },
    Heartbeat {
        subject: String,
        lease_id: String,
        executor_node: NodeId,
        epoch: u64,
        now: DateTime<Utc>,
    },
    Reconcile {
        now: DateTime<Utc>,
    },
}

/// Local apply result claimed by the node that proposed a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayOutcome {
    Published(AppendOutcome),
    PublishedBatch(Vec<AppendOutcome>),
    Leased(Option<Lease>),
    LeaseBatch(Vec<Lease>),
    Acked {
        accepted: bool,
        committed: Option<CommittedOffset>,
    },
    AckBatch {
        accepted: usize,
        committed: Option<CommittedOffset>,
    },
    Released(bool),
    Heartbeat(Option<DateTime<Utc>>),
    Reconciled(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ProposalId {
    node: NodeId,
    session: u64,
    sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelayEnvelope {
    proposal_id: ProposalId,
    command: RelayCommand,
}

/// Full state-machine snapshot: the exact live engine state as of raft index
/// `up_to`, including work-queue ownership and recent proposal outcomes.
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
    /// Recent replicated proposal outcomes. Raft snapshots retain these so an
    /// ambiguous forwarded retry after leader change cannot execute twice.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    completed_proposals: Vec<(ProposalId, RelayOutcome)>,
}

/// Serialize the ONE snapshot format: the engine's live (un-acked) state via
/// [`Relay::dump_live`] plus the applied raft index. This is what
/// [`RelayStateMachine::snapshot`] produces for raft InstallSnapshot and what
/// `GET /admin/backup` streams for `relay backup` (#1209).
pub fn snapshot_bytes(relay: &Relay, up_to: Index) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&EngineSnapshot {
        up_to,
        subjects: relay.dump_live()?,
        completed_proposals: Vec::new(),
    })?)
}

/// Load [`snapshot_bytes`] output into `relay` via exact state replacement.
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
    applied_store: Option<AppliedIndexStore>,
    outcomes: Mutex<OutcomeWindow<RelayOutcome>>,
    completed: Mutex<ProposalCache<ProposalId, RelayOutcome>>,
}

impl RelayStateMachine {
    /// Wrap `relay` as the group's state machine, recovering the applied floor
    /// from `marker` if a previous run persisted one.
    pub fn new(relay: Arc<Relay>, marker: Option<PathBuf>) -> Result<Arc<Self>> {
        let applied_store = marker.map(AppliedIndexStore::new);
        let applied = applied_store
            .as_ref()
            .map(AppliedIndexStore::load)
            .transpose()
            .context("read Relay applied floor")?
            .unwrap_or(0);
        Ok(Arc::new(Self {
            relay,
            applied: AtomicU64::new(applied),
            applied_store,
            outcomes: Mutex::new(OutcomeWindow::default()),
            completed: Mutex::new(ProposalCache::default()),
        }))
    }

    /// Durably record the applied floor (atomic temp-write + fsync + rename).
    /// Best-effort: a persist failure degrades to at-least-once on the next
    /// restart (logged), it never stalls the apply loop.
    fn persist_marker(&self, index: Index) {
        let Some(store) = &self.applied_store else {
            return;
        };
        if let Err(e) = store.store(index) {
            tracing::warn!(index, error = %e, "raft: applied-marker persist failed (floor stale-low; at-least-once on restart)");
        }
    }

    /// Remove and return the engine outcome of the apply at `index` (the
    /// proposing handler claims it once; unclaimed outcomes age out).
    pub fn claim_outcome(&self, index: Index) -> Option<RelayOutcome> {
        self.outcomes.lock().expect("outcome window").claim(index)
    }

    /// The engine this state machine applies into.
    pub fn relay(&self) -> Arc<Relay> {
        Arc::clone(&self.relay)
    }
}

impl RaftStateMachine for RelayStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        // New commands use the tagged RelayCommand envelope. A bare PubCommand
        // remains readable so an upgrade can replay resident pre-upgrade logs.
        let (proposal_id, decoded) = match serde_json::from_slice::<RelayEnvelope>(command) {
            Ok(envelope) => (Some(envelope.proposal_id), Ok(envelope.command)),
            Err(envelope_err) => (
                None,
                serde_json::from_slice::<RelayCommand>(command).or_else(|_| {
                    serde_json::from_slice::<PubCommand>(command)
                        .map(|command| RelayCommand::Publish { command })
                        .map_err(|_| envelope_err)
                }),
            ),
        };
        match decoded {
            Ok(command) => match proposal_id
                .as_ref()
                .and_then(|id| self.completed.lock().expect("completed proposals").get(id))
                .map(Ok)
                .unwrap_or_else(|| self.apply_command(command))
            {
                Ok(outcome) => {
                    if let Some(id) = proposal_id {
                        self.completed
                            .lock()
                            .expect("completed proposals")
                            .insert(id, outcome.clone());
                    }
                    let mut w = self.outcomes.lock().expect("outcome window");
                    w.insert(index, outcome);
                    w.advance(index);
                }
                Err(e) => {
                    tracing::warn!(index, error = %e, "raft: apply Relay command error (entry no-ops)")
                }
            },
            Err(e) => {
                tracing::warn!(index, error = %e, "raft: undecodable command (entry no-ops)")
            }
        }
        self.applied.store(index, Ordering::Release);
        self.persist_marker(index);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&EngineSnapshot {
            up_to: self.applied_index(),
            subjects: self.relay.dump_live()?,
            completed_proposals: self
                .completed
                .lock()
                .expect("completed proposals")
                .snapshot(),
        })?)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snap: EngineSnapshot = serde_json::from_slice(snapshot)?;
        self.relay.load_live(snap.subjects)?;
        self.completed
            .lock()
            .expect("completed proposals")
            .restore(snap.completed_proposals);
        self.applied.store(snap.up_to, Ordering::Release);
        self.persist_marker(snap.up_to);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

impl RelayStateMachine {
    fn apply_command(&self, command: RelayCommand) -> Result<RelayOutcome> {
        Ok(match command {
            RelayCommand::Publish { command } => RelayOutcome::Published(self.relay.publish_at(
                &command.subject,
                &command.message_id,
                command.payload,
                command.headers,
                command.not_before,
                command.priority,
                command.appended_at,
            )?),
            RelayCommand::PublishBatch {
                subject,
                commands,
                now,
            } => {
                let mut messages = Vec::with_capacity(commands.len());
                for command in commands {
                    anyhow::ensure!(
                        command.subject == subject,
                        "relay publish batch mixes subjects"
                    );
                    anyhow::ensure!(
                        command.not_before.is_none(),
                        "relay publish batch does not accept delayed messages"
                    );
                    messages.push((
                        command.message_id,
                        command.payload,
                        command.headers,
                        command.priority,
                    ));
                }
                RelayOutcome::PublishedBatch(self.relay.publish_batch(&subject, messages, now)?)
            }
            RelayCommand::Lease {
                subject,
                consumer_id,
                executor_node,
                now,
            } => RelayOutcome::Leased(self.relay.lease_on_node(
                &subject,
                &consumer_id,
                executor_node,
                now,
            )?),
            RelayCommand::LeaseBatch {
                subject,
                consumer_id,
                executor_node,
                max,
                now,
            } => RelayOutcome::LeaseBatch(self.relay.lease_batch_on_node(
                &subject,
                &consumer_id,
                executor_node,
                max,
                now,
            )?),
            RelayCommand::Ack {
                subject,
                lease_id,
                executor_node,
                epoch,
                now,
            } => RelayOutcome::Acked {
                accepted: self
                    .relay
                    .ack_on_node(&subject, &lease_id, executor_node, epoch, now)?,
                committed: self.relay.committed_offset(&subject)?,
            },
            RelayCommand::AckBatch {
                subject,
                acks,
                executor_node,
                now,
            } => {
                let mut accepted = 0;
                for (lease_id, epoch) in acks {
                    if self
                        .relay
                        .ack_on_node(&subject, &lease_id, executor_node, epoch, now)?
                    {
                        accepted += 1;
                    }
                }
                RelayOutcome::AckBatch {
                    accepted,
                    committed: self.relay.committed_offset(&subject)?,
                }
            }
            RelayCommand::Release {
                subject,
                lease_id,
                executor_node,
                epoch,
                now,
            } => RelayOutcome::Released(self.relay.release_on_node(
                &subject,
                &lease_id,
                executor_node,
                epoch,
                now,
            )?),
            RelayCommand::Heartbeat {
                subject,
                lease_id,
                executor_node,
                epoch,
                now,
            } => RelayOutcome::Heartbeat(self.relay.heartbeat_on_node(
                &subject,
                &lease_id,
                executor_node,
                epoch,
                now,
            )?),
            RelayCommand::Reconcile { now } => RelayOutcome::Reconciled(self.relay.reconcile(now)),
        })
    }
}

/// One running raft group over a relay engine — the single-group wrapper the
/// serve path (and tests) hold. Dropping it aborts the host's tick/pump tasks.
pub struct RelayRaft {
    host: RaftHost,
    sm: Arc<RelayStateMachine>,
    node_id: NodeId,
    session: u64,
    proposal_sequence: AtomicU64,
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
        Self::spawn_inner(relay, raft_dir, node_id, membership, peers, cfg, None)
    }

    pub fn spawn_with_peer_transport(
        relay: Arc<Relay>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
        transport: PeerTransport,
    ) -> Result<RelayRaft> {
        Self::spawn_inner(
            relay,
            raft_dir,
            node_id,
            membership,
            peers,
            cfg,
            Some(transport),
        )
    }

    fn spawn_inner(
        relay: Arc<Relay>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        cfg: HostConfig,
        transport: Option<PeerTransport>,
    ) -> Result<RelayRaft> {
        std::fs::create_dir_all(raft_dir)?;
        let dir = raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?;
        let store = RaftStore::open(dir, node_id, FsyncPolicy::Always)?;
        let sm =
            RelayStateMachine::new(relay, Some(raft_dir.join(format!("applied-{node_id}.idx"))))?;
        let host = match transport {
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
        Ok(RelayRaft {
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

    pub fn from_topology_with_peer_transport(
        relay: Arc<Relay>,
        data_dir: &Path,
        topo: &ClusterTopology,
        cfg: HostConfig,
        transport: PeerTransport,
    ) -> Result<RelayRaft> {
        Self::spawn_with_peer_transport(
            relay,
            &data_dir.join("raft"),
            topo.node_id,
            topo.membership.clone(),
            topo.peers.clone(),
            cfg,
            transport,
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

    /// Drain the shared Raft host's in-flight peer RPCs before its h2 client
    /// and peer listener are torn down.
    pub async fn shutdown(&self) -> Result<()> {
        self.host.shutdown().await
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
        let (index, outcome) = self
            .propose(&RelayCommand::Publish {
                command: cmd.clone(),
            })
            .await?;
        let published = match outcome {
            Some(RelayOutcome::Published(outcome)) => Some(outcome),
            None => None,
            Some(other) => anyhow::bail!("raft publish outcome mismatch: {other:?}"),
        };
        Ok((index, published))
    }

    pub async fn publish_batch(
        &self,
        subject: String,
        commands: Vec<PubCommand>,
        now: DateTime<Utc>,
    ) -> Result<(Index, Vec<AppendOutcome>)> {
        match self
            .propose(&RelayCommand::PublishBatch {
                subject,
                commands,
                now,
            })
            .await?
        {
            (index, Some(RelayOutcome::PublishedBatch(outcomes))) => Ok((index, outcomes)),
            (_, other) => anyhow::bail!("raft publish-batch outcome mismatch: {other:?}"),
        }
    }

    async fn propose(&self, command: &RelayCommand) -> Result<(Index, Option<RelayOutcome>)> {
        let envelope = RelayEnvelope {
            proposal_id: ProposalId {
                node: self.node_id,
                session: self.session,
                sequence: self.proposal_sequence.fetch_add(1, Ordering::Relaxed),
            },
            command: command.clone(),
        };
        let index = self.host.propose(serde_json::to_vec(&envelope)?).await?;
        Ok((index, self.sm.claim_outcome(index)))
    }

    pub async fn lease(
        &self,
        subject: String,
        consumer_id: String,
        now: DateTime<Utc>,
    ) -> Result<Option<Lease>> {
        match self
            .propose(&RelayCommand::Lease {
                subject,
                consumer_id,
                executor_node: self.node_id,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::Leased(lease)) => Ok(lease),
            other => anyhow::bail!("raft lease outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn lease_batch(
        &self,
        subject: String,
        consumer_id: String,
        max: usize,
        now: DateTime<Utc>,
    ) -> Result<Vec<Lease>> {
        match self
            .propose(&RelayCommand::LeaseBatch {
                subject,
                consumer_id,
                executor_node: self.node_id,
                max,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::LeaseBatch(leases)) => Ok(leases),
            other => anyhow::bail!("raft lease-batch outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn ack(
        &self,
        subject: String,
        lease_id: String,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Result<(bool, Option<CommittedOffset>)> {
        match self
            .propose(&RelayCommand::Ack {
                subject,
                lease_id,
                executor_node: self.node_id,
                epoch,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::Acked {
                accepted,
                committed,
            }) => Ok((accepted, committed)),
            other => anyhow::bail!("raft ack outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn ack_batch(
        &self,
        subject: String,
        acks: Vec<(String, u64)>,
        now: DateTime<Utc>,
    ) -> Result<(usize, Option<CommittedOffset>)> {
        match self
            .propose(&RelayCommand::AckBatch {
                subject,
                acks,
                executor_node: self.node_id,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::AckBatch {
                accepted,
                committed,
            }) => Ok((accepted, committed)),
            other => anyhow::bail!("raft ack-batch outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn release(
        &self,
        subject: String,
        lease_id: String,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        match self
            .propose(&RelayCommand::Release {
                subject,
                lease_id,
                executor_node: self.node_id,
                epoch,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::Released(released)) => Ok(released),
            other => anyhow::bail!("raft release outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn heartbeat(
        &self,
        subject: String,
        lease_id: String,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Result<Option<DateTime<Utc>>> {
        match self
            .propose(&RelayCommand::Heartbeat {
                subject,
                lease_id,
                executor_node: self.node_id,
                epoch,
                now,
            })
            .await?
            .1
        {
            Some(RelayOutcome::Heartbeat(expiry)) => Ok(expiry),
            other => anyhow::bail!("raft heartbeat outcome missing or mismatched: {other:?}"),
        }
    }

    pub async fn reconcile(&self, now: DateTime<Utc>) -> Result<usize> {
        match self.propose(&RelayCommand::Reconcile { now }).await?.1 {
            Some(RelayOutcome::Reconciled(count)) => Ok(count),
            other => anyhow::bail!("raft reconcile outcome missing or mismatched: {other:?}"),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
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

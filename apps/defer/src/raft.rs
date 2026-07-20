// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:defer-raft-scheduler" tracker="#766" reason="Raft-backed delayed-task state machine with durable snapshots and fenced dispatch ownership."
//! Raft-backed authoritative state for Defer.
//!
//! Every scheduler mutation is a committed command. In particular, an
//! executor may call an HTTP target only after [`DeferCommand::LeaseDue`] has
//! committed the executor node and fence epoch. Ack/nack therefore reject a
//! stale replica or an expired attempt. External HTTP effects remain
//! at-least-once: a crash after the target accepts but before ack commits can
//! cause a retry, while the stable attempt idempotency key lets a cooperating
//! target collapse that ambiguity.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::Router;
use chrono::{DateTime, Utc};
use raft_runtime::{
    ClusterTopology, FsyncPolicy as RaftFsyncPolicy, HostConfig, Index, Membership, NodeId,
    OutcomeWindow, PeerTransport, ProposalCache, RaftHost, RaftStateMachine, RaftStore,
    SnapshotPolicy,
};
use serde::{Deserialize, Serialize};

use crate::{
    AttemptSettlement, CreateTask, DeferScheduler, DispatchLease, NackOutcome, QueueControlState,
    QueuePolicy, QueueSnapshot, SchedulerError, SettlementOutcome,
};

pub const SNAPSHOT_EVERY: u64 = 1024;

/// Every authoritative scheduler transition. All clocks and executor ids are
/// resolved by the proposer so replicas apply identical bytes and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DeferCommand {
    ConfigureQueue {
        queue: String,
        policy: QueuePolicy,
    },
    UpdateQueuePolicy {
        queue: String,
        policy: QueuePolicy,
    },
    SetQueueControl {
        queue: String,
        control: QueueControlState,
    },
    CreateTask {
        queue: String,
        task: CreateTask,
    },
    CreateTasks {
        queue: String,
        tasks: Vec<CreateTask>,
    },
    LeaseDue {
        queue: String,
        executor_node: NodeId,
        now: DateTime<Utc>,
        requested: usize,
    },
    Ack {
        queue: String,
        attempt_id: String,
        executor_node: NodeId,
        epoch: u64,
        now: DateTime<Utc>,
    },
    Nack {
        queue: String,
        attempt_id: String,
        executor_node: NodeId,
        epoch: u64,
        now: DateTime<Utc>,
    },
    SettleBatch {
        queue: String,
        executor_node: NodeId,
        attempts: Vec<AttemptSettlement>,
    },
    ReclaimExpired {
        queue: String,
        now: DateTime<Utc>,
    },
    Cancel {
        queue: String,
        task_id: String,
    },
}

/// Read-your-write outcome retained by proposal id across snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeferOutcome {
    Queue(Result<QueueSnapshot, SchedulerError>),
    Created(Result<(), SchedulerError>),
    CreatedBatch(Result<usize, SchedulerError>),
    Leased(Result<Vec<DispatchLease>, SchedulerError>),
    Acked(Result<bool, SchedulerError>),
    Nacked(Result<Option<NackOutcome>, SchedulerError>),
    Settled(Result<Vec<SettlementOutcome>, SchedulerError>),
    Reclaimed(Result<Vec<String>, SchedulerError>),
    Canceled(Result<bool, SchedulerError>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct ProposalId {
    node: NodeId,
    session: u64,
    sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeferEnvelope {
    proposal_id: ProposalId,
    command: DeferCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SchedulerSnapshot {
    up_to: Index,
    scheduler: DeferScheduler,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    completed_proposals: Vec<(ProposalId, DeferOutcome)>,
}

/// Seed an empty PVC with the exact state-machine snapshot served by the
/// admin backup endpoint. This is cold-start only and refuses replacement of
/// any existing state.
pub fn prepare_bootstrap_seed(data_dir: &Path, node_id: NodeId, bytes: &[u8]) -> Result<()> {
    let snapshot: SchedulerSnapshot =
        serde_json::from_slice(bytes).context("decode Defer scheduler snapshot")?;
    if data_dir.exists() {
        let mut entries = std::fs::read_dir(data_dir)
            .with_context(|| format!("read bootstrap data dir {}", data_dir.display()))?;
        anyhow::ensure!(
            entries.next().transpose()?.is_none(),
            "bootstrap seed requires an empty data directory {}",
            data_dir.display()
        );
    } else {
        std::fs::create_dir_all(data_dir)?;
    }
    let raft_dir = data_dir.join("raft");
    let store = RaftStore::open(
        raft_dir
            .to_str()
            .context("raft data dir is not valid UTF-8")?,
        node_id,
        RaftFsyncPolicy::Always,
    )?;
    store.seed_snapshot(snapshot.up_to, 0, bytes.to_vec())?;
    Ok(())
}

fn snapshot_path_for(marker: &Path) -> PathBuf {
    let name = marker
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("applied.idx")
        .replacen("applied-", "snapshot-", 1);
    let name = name
        .strip_suffix(".idx")
        .map(|stem| format!("{stem}.json"))
        .unwrap_or_else(|| format!("{name}.json"));
    marker.with_file_name(name)
}

pub struct DeferStateMachine {
    scheduler: Arc<Mutex<DeferScheduler>>,
    applied: AtomicU64,
    marker: Option<PathBuf>,
    outcomes: Mutex<OutcomeWindow<DeferOutcome>>,
    completed: Mutex<ProposalCache<ProposalId, DeferOutcome>>,
}

impl DeferStateMachine {
    pub fn new(
        scheduler: Arc<Mutex<DeferScheduler>>,
        marker: Option<PathBuf>,
    ) -> Result<Arc<Self>> {
        let mut applied = 0;
        let mut recovered_completed = Vec::new();
        if let Some(path) = &marker {
            let snapshot_path = snapshot_path_for(path);
            match std::fs::read(&snapshot_path) {
                Ok(bytes) => {
                    let snapshot: SchedulerSnapshot =
                        serde_json::from_slice(&bytes).with_context(|| {
                            format!("corrupt scheduler snapshot {}", snapshot_path.display())
                        })?;
                    *scheduler.lock().expect("scheduler mutex poisoned") = snapshot.scheduler;
                    applied = snapshot.up_to;
                    recovered_completed = snapshot.completed_proposals;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("read scheduler snapshot"),
            }
            match std::fs::read_to_string(path) {
                Ok(value) => {
                    let marker_floor = value
                        .trim()
                        .parse::<u64>()
                        .with_context(|| format!("corrupt applied marker {}", path.display()))?;
                    if marker_floor != applied {
                        tracing::warn!(
                            marker_floor,
                            snapshot_floor = applied,
                            "defer raft marker disagrees with scheduler snapshot"
                        );
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("read applied marker"),
            }
        }
        let mut completed = ProposalCache::default();
        completed.restore(recovered_completed);
        Ok(Arc::new(Self {
            scheduler,
            applied: AtomicU64::new(applied),
            marker,
            outcomes: Mutex::new(OutcomeWindow::default()),
            completed: Mutex::new(completed),
        }))
    }

    pub fn scheduler(&self) -> Arc<Mutex<DeferScheduler>> {
        Arc::clone(&self.scheduler)
    }

    pub fn claim_outcome(&self, index: Index) -> Option<DeferOutcome> {
        self.outcomes.lock().expect("outcome window").claim(index)
    }

    fn apply_command(&self, command: DeferCommand) -> DeferOutcome {
        let mut scheduler = self.scheduler.lock().expect("scheduler mutex poisoned");
        match command {
            DeferCommand::ConfigureQueue { queue, policy } => {
                scheduler.configure_queue(&queue, policy);
                DeferOutcome::Queue(scheduler.queue_snapshot(&queue))
            }
            DeferCommand::UpdateQueuePolicy { queue, policy } => DeferOutcome::Queue(
                scheduler
                    .update_queue_policy(&queue, policy)
                    .and_then(|()| scheduler.queue_snapshot(&queue)),
            ),
            DeferCommand::SetQueueControl { queue, control } => {
                let changed = match control {
                    QueueControlState::Running => scheduler.resume_queue(&queue),
                    QueueControlState::Paused => scheduler.pause_queue(&queue),
                    QueueControlState::Disabled => scheduler.disable_queue(&queue),
                };
                DeferOutcome::Queue(changed.and_then(|()| scheduler.queue_snapshot(&queue)))
            }
            DeferCommand::CreateTask { queue, task } => {
                DeferOutcome::Created(scheduler.create_task(queue, task))
            }
            DeferCommand::CreateTasks { queue, tasks } => {
                let count = tasks.len();
                DeferOutcome::CreatedBatch(scheduler.create_tasks(queue, tasks).map(|()| count))
            }
            DeferCommand::LeaseDue {
                queue,
                executor_node,
                now,
                requested,
            } => DeferOutcome::Leased(scheduler.lease_due_on_node(
                &queue,
                executor_node,
                now,
                requested,
            )),
            DeferCommand::Ack {
                queue,
                attempt_id,
                executor_node,
                epoch,
                now,
            } => DeferOutcome::Acked(scheduler.ack_on_node(
                &queue,
                &attempt_id,
                executor_node,
                epoch,
                now,
            )),
            DeferCommand::Nack {
                queue,
                attempt_id,
                executor_node,
                epoch,
                now,
            } => DeferOutcome::Nacked(scheduler.nack_on_node(
                &queue,
                &attempt_id,
                executor_node,
                epoch,
                now,
            )),
            DeferCommand::SettleBatch {
                queue,
                executor_node,
                attempts,
            } => DeferOutcome::Settled(scheduler.settle_batch(&queue, executor_node, attempts)),
            DeferCommand::ReclaimExpired { queue, now } => {
                DeferOutcome::Reclaimed(scheduler.reclaim_expired(&queue, now))
            }
            DeferCommand::Cancel { queue, task_id } => {
                DeferOutcome::Canceled(scheduler.cancel(&queue, &task_id))
            }
        }
    }
}

impl RaftStateMachine for DeferStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        if index <= self.applied.load(Ordering::Acquire) && self.marker.is_some() {
            return Ok(());
        }
        let envelope: DeferEnvelope = serde_json::from_slice(command)?;
        let cached = self
            .completed
            .lock()
            .expect("completed proposals")
            .get(&envelope.proposal_id);
        let outcome = cached.unwrap_or_else(|| self.apply_command(envelope.command));
        self.completed
            .lock()
            .expect("completed proposals")
            .insert(envelope.proposal_id, outcome.clone());
        let mut outcomes = self.outcomes.lock().expect("outcome window");
        outcomes.insert(index, outcome);
        outcomes.advance(index);
        drop(outcomes);
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&SchedulerSnapshot {
            up_to: self.applied_index(),
            scheduler: self
                .scheduler
                .lock()
                .expect("scheduler mutex poisoned")
                .clone(),
            completed_proposals: self
                .completed
                .lock()
                .expect("completed proposals")
                .snapshot(),
        })?)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snapshot: SchedulerSnapshot = serde_json::from_slice(snapshot)?;
        *self.scheduler.lock().expect("scheduler mutex poisoned") = snapshot.scheduler;
        self.completed
            .lock()
            .expect("completed proposals")
            .restore(snapshot.completed_proposals);
        self.applied.store(snapshot.up_to, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

pub struct DeferRaft {
    host: RaftHost,
    sm: Arc<DeferStateMachine>,
    node_id: NodeId,
    session: u64,
    proposal_sequence: AtomicU64,
}

impl DeferRaft {
    pub fn spawn(
        scheduler: Arc<Mutex<DeferScheduler>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        config: HostConfig,
    ) -> Result<Self> {
        Self::spawn_inner(
            scheduler, raft_dir, node_id, membership, peers, config, None,
        )
    }

    pub fn spawn_with_peer_transport(
        scheduler: Arc<Mutex<DeferScheduler>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        config: HostConfig,
        transport: PeerTransport,
    ) -> Result<Self> {
        Self::spawn_inner(
            scheduler,
            raft_dir,
            node_id,
            membership,
            peers,
            config,
            Some(transport),
        )
    }

    fn spawn_inner(
        scheduler: Arc<Mutex<DeferScheduler>>,
        raft_dir: &Path,
        node_id: NodeId,
        membership: Membership,
        peers: HashMap<NodeId, String>,
        config: HostConfig,
        transport: Option<PeerTransport>,
    ) -> Result<Self> {
        std::fs::create_dir_all(raft_dir)?;
        let store = RaftStore::open(
            raft_dir
                .to_str()
                .context("raft data dir is not valid UTF-8")?,
            node_id,
            RaftFsyncPolicy::Always,
        )?;
        let sm = DeferStateMachine::new(
            scheduler,
            Some(raft_dir.join(format!("applied-{node_id}.idx"))),
        )?;
        let host = match transport {
            Some(transport) => RaftHost::spawn_with_peer_transport(
                node_id,
                membership,
                peers,
                store,
                Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
                config,
                transport,
            ),
            None => RaftHost::spawn(
                node_id,
                membership,
                peers,
                store,
                Arc::clone(&sm) as Arc<dyn RaftStateMachine>,
                config,
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
        Ok(Self {
            host,
            sm,
            node_id,
            session,
            proposal_sequence: AtomicU64::new(1),
        })
    }

    pub fn from_topology(
        scheduler: Arc<Mutex<DeferScheduler>>,
        data_dir: &Path,
        topology: &ClusterTopology,
        config: HostConfig,
    ) -> Result<Self> {
        Self::spawn(
            scheduler,
            &data_dir.join("raft"),
            topology.node_id,
            topology.membership.clone(),
            topology.peers.clone(),
            config,
        )
    }

    pub fn from_topology_with_peer_transport(
        scheduler: Arc<Mutex<DeferScheduler>>,
        data_dir: &Path,
        topology: &ClusterTopology,
        config: HostConfig,
        transport: PeerTransport,
    ) -> Result<Self> {
        Self::spawn_with_peer_transport(
            scheduler,
            &data_dir.join("raft"),
            topology.node_id,
            topology.membership.clone(),
            topology.peers.clone(),
            config,
            transport,
        )
    }

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

    pub fn router(&self) -> Router {
        self.host.router()
    }

    async fn propose(&self, command: DeferCommand) -> Result<DeferOutcome> {
        let envelope = DeferEnvelope {
            proposal_id: ProposalId {
                node: self.node_id,
                session: self.session,
                sequence: self.proposal_sequence.fetch_add(1, Ordering::Relaxed),
            },
            command,
        };
        let index = self.host.propose(serde_json::to_vec(&envelope)?).await?;
        self.sm
            .claim_outcome(index)
            .with_context(|| format!("defer outcome for raft index {index} aged out"))
    }

    pub async fn configure_queue(
        &self,
        queue: String,
        policy: QueuePolicy,
    ) -> Result<QueueSnapshot> {
        match self
            .propose(DeferCommand::ConfigureQueue { queue, policy })
            .await?
        {
            DeferOutcome::Queue(result) => Ok(result?),
            other => anyhow::bail!("configure queue outcome mismatch: {other:?}"),
        }
    }

    pub async fn update_queue_policy(
        &self,
        queue: String,
        policy: QueuePolicy,
    ) -> Result<QueueSnapshot> {
        match self
            .propose(DeferCommand::UpdateQueuePolicy { queue, policy })
            .await?
        {
            DeferOutcome::Queue(result) => Ok(result?),
            other => anyhow::bail!("update queue outcome mismatch: {other:?}"),
        }
    }

    pub async fn set_queue_control(
        &self,
        queue: String,
        control: QueueControlState,
    ) -> Result<QueueSnapshot> {
        match self
            .propose(DeferCommand::SetQueueControl { queue, control })
            .await?
        {
            DeferOutcome::Queue(result) => Ok(result?),
            other => anyhow::bail!("queue control outcome mismatch: {other:?}"),
        }
    }

    pub async fn create_task(&self, queue: String, task: CreateTask) -> Result<()> {
        match self
            .propose(DeferCommand::CreateTask { queue, task })
            .await?
        {
            DeferOutcome::Created(result) => Ok(result?),
            other => anyhow::bail!("create task outcome mismatch: {other:?}"),
        }
    }

    pub async fn create_tasks(&self, queue: String, tasks: Vec<CreateTask>) -> Result<usize> {
        match self
            .propose(DeferCommand::CreateTasks { queue, tasks })
            .await?
        {
            DeferOutcome::CreatedBatch(result) => Ok(result?),
            other => anyhow::bail!("create tasks outcome mismatch: {other:?}"),
        }
    }

    pub async fn lease_due(
        &self,
        queue: String,
        now: DateTime<Utc>,
        requested: usize,
    ) -> Result<Vec<DispatchLease>> {
        match self
            .propose(DeferCommand::LeaseDue {
                queue,
                executor_node: self.node_id,
                now,
                requested,
            })
            .await?
        {
            DeferOutcome::Leased(result) => Ok(result?),
            other => anyhow::bail!("lease outcome mismatch: {other:?}"),
        }
    }

    pub async fn ack(
        &self,
        queue: String,
        attempt_id: String,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Result<bool> {
        match self
            .propose(DeferCommand::Ack {
                queue,
                attempt_id,
                executor_node: self.node_id,
                epoch,
                now,
            })
            .await?
        {
            DeferOutcome::Acked(result) => Ok(result?),
            other => anyhow::bail!("ack outcome mismatch: {other:?}"),
        }
    }

    pub async fn nack(
        &self,
        queue: String,
        attempt_id: String,
        epoch: u64,
        now: DateTime<Utc>,
    ) -> Result<Option<NackOutcome>> {
        match self
            .propose(DeferCommand::Nack {
                queue,
                attempt_id,
                executor_node: self.node_id,
                epoch,
                now,
            })
            .await?
        {
            DeferOutcome::Nacked(result) => Ok(result?),
            other => anyhow::bail!("nack outcome mismatch: {other:?}"),
        }
    }

    pub async fn settle_batch(
        &self,
        queue: String,
        attempts: Vec<AttemptSettlement>,
    ) -> Result<Vec<SettlementOutcome>> {
        match self
            .propose(DeferCommand::SettleBatch {
                queue,
                executor_node: self.node_id,
                attempts,
            })
            .await?
        {
            DeferOutcome::Settled(result) => Ok(result?),
            other => anyhow::bail!("settle batch outcome mismatch: {other:?}"),
        }
    }

    pub async fn reclaim_expired(&self, queue: String, now: DateTime<Utc>) -> Result<Vec<String>> {
        match self
            .propose(DeferCommand::ReclaimExpired { queue, now })
            .await?
        {
            DeferOutcome::Reclaimed(result) => Ok(result?),
            other => anyhow::bail!("reclaim outcome mismatch: {other:?}"),
        }
    }

    pub async fn cancel(&self, queue: String, task_id: String) -> Result<bool> {
        match self
            .propose(DeferCommand::Cancel { queue, task_id })
            .await?
        {
            DeferOutcome::Canceled(result) => Ok(result?),
            other => anyhow::bail!("cancel outcome mismatch: {other:?}"),
        }
    }

    pub fn scheduler(&self) -> Arc<Mutex<DeferScheduler>> {
        self.sm.scheduler()
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

    pub fn applied_index(&self) -> Index {
        self.sm.applied_index()
    }

    pub fn snapshot_bytes(&self) -> Result<Vec<u8>> {
        self.sm.snapshot()
    }
}
// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Target;
    use chrono::TimeZone;

    fn task(id: &str, at: DateTime<Utc>) -> CreateTask {
        CreateTask {
            task_id: id.into(),
            target: Target {
                url: "http://example.test/dispatch".into(),
                method: "POST".into(),
                headers: Default::default(),
            },
            payload: serde_json::json!({"id": id}),
            schedule_at: at,
            priority: 10,
            max_attempts: 3,
        }
    }

    fn envelope(sequence: u64, command: DeferCommand) -> Vec<u8> {
        serde_json::to_vec(&DeferEnvelope {
            proposal_id: ProposalId {
                node: 1,
                session: 9,
                sequence,
            },
            command,
        })
        .unwrap()
    }

    #[test]
    fn snapshot_round_trip_preserves_fenced_attempt_and_dedupe() {
        let now = Utc.timestamp_millis_opt(1_000).unwrap();
        let scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
        let sm = DeferStateMachine::new(scheduler.clone(), None).unwrap();
        sm.apply(
            1,
            &envelope(
                1,
                DeferCommand::ConfigureQueue {
                    queue: "jobs".into(),
                    policy: QueuePolicy::default(),
                },
            ),
        )
        .unwrap();
        sm.apply(
            2,
            &envelope(
                2,
                DeferCommand::CreateTask {
                    queue: "jobs".into(),
                    task: task("one", now),
                },
            ),
        )
        .unwrap();
        let lease_command = DeferCommand::LeaseDue {
            queue: "jobs".into(),
            executor_node: 7,
            now,
            requested: 1,
        };
        let lease_bytes = envelope(3, lease_command);
        sm.apply(3, &lease_bytes).unwrap();
        let lease = match sm.claim_outcome(3).unwrap() {
            DeferOutcome::Leased(Ok(mut leases)) => leases.remove(0),
            other => panic!("{other:?}"),
        };

        let restored_scheduler = Arc::new(Mutex::new(DeferScheduler::new()));
        let restored = DeferStateMachine::new(restored_scheduler.clone(), None).unwrap();
        restored.restore(&sm.snapshot().unwrap()).unwrap();
        restored.apply(4, &lease_bytes).unwrap();
        let repeated = match restored.claim_outcome(4).unwrap() {
            DeferOutcome::Leased(Ok(mut leases)) => leases.remove(0),
            other => panic!("{other:?}"),
        };
        assert_eq!(lease.attempt_id, repeated.attempt_id);
        assert_eq!(lease.epoch, repeated.epoch);
        assert_eq!(
            restored_scheduler
                .lock()
                .unwrap()
                .queue_snapshot("jobs")
                .unwrap()
                .in_flight_count,
            1
        );
    }
}

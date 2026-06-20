//! openraft-backed consensus (HA phase C). **Single-node integration** behind
//! the `raft` feature: it wires keep's engine as a raft state machine so writes
//! go through the raft log (propose → commit → apply). Multi-node networking
//! over HTTP/2 + k8s discovery + having the raft log subsume the on-disk WAL are
//! the staged remainder (see HA.md). The default server is unaffected.
//!
//! Structure mirrors openraft's `raft-kv-memstore` example: an in-memory
//! `LogStore`, an engine-backed `StateMachineStore`, and a stub `Network`
//! (a single node never sends RPCs).

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::RangeBounds;
use std::sync::Arc;

use openraft::storage::{LogFlushed, LogState, RaftLogStorage, RaftStateMachine, Snapshot};
use openraft::{
    BasicNode, Entry, EntryPayload, LogId, OptionalSend, RaftLogReader, RaftSnapshotBuilder,
    SnapshotMeta, StorageError, StorageIOError, StoredMembership, Vote,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::engine::KvEngine;
use crate::persistence::format::WalOp;

/// App response for a committed command (the engine result isn't threaded back
/// for v1 — the command's own HTTP handler already has it).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Response {
    pub applied: bool,
}

openraft::declare_raft_types!(
    /// keep's raft type config: the app command is a logical mutation (`WalOp`,
    /// the same type the WAL + recovery already use).
    pub TypeConfig:
        D = WalOp,
        R = Response,
        Node = BasicNode,
);

type NodeId = <TypeConfig as openraft::RaftTypeConfig>::NodeId;

// ---------------------------------------------------------------------------
// Log store (in-memory; the durable raft log = WAL subsumption is staged)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct LogStoreInner {
    vote: Option<Vote<NodeId>>,
    log: BTreeMap<u64, Entry<TypeConfig>>,
    committed: Option<LogId<NodeId>>,
    last_purged: Option<LogId<NodeId>>,
}

#[derive(Clone, Default)]
pub struct LogStore {
    inner: Arc<Mutex<LogStoreInner>>,
}

impl RaftLogReader<TypeConfig> for LogStore {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<Entry<TypeConfig>>, StorageError<NodeId>> {
        let inner = self.inner.lock();
        Ok(inner.log.range(range).map(|(_, e)| e.clone()).collect())
    }
}

impl RaftLogStorage<TypeConfig> for LogStore {
    type LogReader = Self;

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, StorageError<NodeId>> {
        let inner = self.inner.lock();
        let last = inner.log.iter().next_back().map(|(_, e)| e.log_id);
        let last_purged = inner.last_purged;
        let last = last.or(last_purged);
        Ok(LogState {
            last_purged_log_id: last_purged,
            last_log_id: last,
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn save_vote(&mut self, vote: &Vote<NodeId>) -> Result<(), StorageError<NodeId>> {
        self.inner.lock().vote = Some(*vote);
        Ok(())
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<NodeId>>, StorageError<NodeId>> {
        Ok(self.inner.lock().vote)
    }

    async fn save_committed(
        &mut self,
        committed: Option<LogId<NodeId>>,
    ) -> Result<(), StorageError<NodeId>> {
        self.inner.lock().committed = committed;
        Ok(())
    }

    async fn read_committed(&mut self) -> Result<Option<LogId<NodeId>>, StorageError<NodeId>> {
        Ok(self.inner.lock().committed)
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: LogFlushed<TypeConfig>,
    ) -> Result<(), StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry<TypeConfig>> + OptionalSend,
    {
        {
            let mut inner = self.inner.lock();
            for e in entries {
                inner.log.insert(e.log_id.index, e);
            }
        }
        // In-memory: the entries are durable as soon as they're inserted.
        callback.log_io_completed(Ok(()));
        Ok(())
    }

    async fn truncate(&mut self, log_id: LogId<NodeId>) -> Result<(), StorageError<NodeId>> {
        let mut inner = self.inner.lock();
        inner.log.split_off(&log_id.index);
        Ok(())
    }

    async fn purge(&mut self, log_id: LogId<NodeId>) -> Result<(), StorageError<NodeId>> {
        let mut inner = self.inner.lock();
        inner.last_purged = Some(log_id);
        let keep = inner.log.split_off(&(log_id.index + 1));
        inner.log = keep;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// State machine (engine-backed)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSnapshot {
    meta: SnapshotMeta<NodeId, BasicNode>,
    data: Vec<u8>,
}

struct SmInner {
    last_applied: Option<LogId<NodeId>>,
    last_membership: StoredMembership<NodeId, BasicNode>,
    snapshot: Option<StoredSnapshot>,
    snapshot_idx: u64,
}

#[derive(Clone)]
pub struct StateMachineStore {
    engine: Arc<KvEngine>,
    inner: Arc<Mutex<SmInner>>,
}

impl StateMachineStore {
    pub fn new(engine: Arc<KvEngine>) -> Self {
        Self {
            engine,
            inner: Arc::new(Mutex::new(SmInner {
                last_applied: None,
                last_membership: StoredMembership::default(),
                snapshot: None,
                snapshot_idx: 0,
            })),
        }
    }
}

impl RaftSnapshotBuilder<TypeConfig> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<Snapshot<TypeConfig>, StorageError<NodeId>> {
        let (last_applied, last_membership) = {
            let g = self.inner.lock();
            (g.last_applied, g.last_membership.clone())
        };
        // Serialize the full engine contents (key→value; TTL dropped — see
        // KvEngine::dump_values).
        let dump = self.engine.dump_values();
        let data = serde_json::to_vec(&dump)
            .map_err(|e| StorageIOError::read_state_machine(&e))?;

        let snapshot_id = {
            let mut g = self.inner.lock();
            g.snapshot_idx += 1;
            format!(
                "{}-{}",
                last_applied.map(|l| l.index).unwrap_or(0),
                g.snapshot_idx
            )
        };
        let meta = SnapshotMeta {
            last_log_id: last_applied,
            last_membership,
            snapshot_id,
        };
        self.inner.lock().snapshot = Some(StoredSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        });
        Ok(Snapshot {
            meta,
            snapshot: Box::new(Cursor::new(data)),
        })
    }
}

impl RaftStateMachine<TypeConfig> for StateMachineStore {
    type SnapshotBuilder = Self;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogId<NodeId>>, StoredMembership<NodeId, BasicNode>), StorageError<NodeId>>
    {
        let g = self.inner.lock();
        Ok((g.last_applied, g.last_membership.clone()))
    }

    async fn apply<I>(&mut self, entries: I) -> Result<Vec<Response>, StorageError<NodeId>>
    where
        I: IntoIterator<Item = Entry<TypeConfig>> + OptionalSend,
    {
        let mut res = Vec::new();
        for entry in entries {
            {
                self.inner.lock().last_applied = Some(entry.log_id);
            }
            match entry.payload {
                EntryPayload::Blank => res.push(Response { applied: false }),
                EntryPayload::Normal(op) => {
                    // Reuse the exact WAL-replay apply path.
                    let _ = crate::persistence::recovery::RecoveryManager::apply_one(&self.engine, &op);
                    res.push(Response { applied: true });
                }
                EntryPayload::Membership(m) => {
                    self.inner.lock().last_membership =
                        StoredMembership::new(Some(entry.log_id), m);
                    res.push(Response { applied: false });
                }
            }
        }
        Ok(res)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<Cursor<Vec<u8>>>, StorageError<NodeId>> {
        Ok(Box::new(Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<NodeId, BasicNode>,
        snapshot: Box<Cursor<Vec<u8>>>,
    ) -> Result<(), StorageError<NodeId>> {
        let data = snapshot.into_inner();
        let dump: Vec<(String, crate::types::KvValue)> = serde_json::from_slice(&data)
            .map_err(|e| StorageIOError::read_snapshot(Some(meta.signature()), &e))?;
        self.engine.load_values(dump);
        let mut g = self.inner.lock();
        g.last_applied = meta.last_log_id;
        g.last_membership = meta.last_membership.clone();
        g.snapshot = Some(StoredSnapshot {
            meta: meta.clone(),
            data,
        });
        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<NodeId>> {
        Ok(self.inner.lock().snapshot.clone().map(|s| Snapshot {
            meta: s.meta,
            snapshot: Box::new(Cursor::new(s.data)),
        }))
    }
}

// ---------------------------------------------------------------------------
// Network (stub: a single node never sends RPCs; multi-node = HTTP/2, staged)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Network;

impl openraft::RaftNetworkFactory<TypeConfig> for Network {
    type Network = NetworkConn;
    async fn new_client(&mut self, target: NodeId, _node: &BasicNode) -> Self::Network {
        NetworkConn { _target: target }
    }
}

pub struct NetworkConn {
    _target: NodeId,
}

impl openraft::RaftNetwork<TypeConfig> for NetworkConn {
    async fn append_entries(
        &mut self,
        _rpc: openraft::raft::AppendEntriesRequest<TypeConfig>,
        _option: openraft::network::RPCOption,
    ) -> Result<
        openraft::raft::AppendEntriesResponse<NodeId>,
        openraft::error::RPCError<NodeId, BasicNode, openraft::error::RaftError<NodeId>>,
    > {
        Err(unreachable_rpc())
    }

    async fn install_snapshot(
        &mut self,
        _rpc: openraft::raft::InstallSnapshotRequest<TypeConfig>,
        _option: openraft::network::RPCOption,
    ) -> Result<
        openraft::raft::InstallSnapshotResponse<NodeId>,
        openraft::error::RPCError<
            NodeId,
            BasicNode,
            openraft::error::RaftError<NodeId, openraft::error::InstallSnapshotError>,
        >,
    > {
        Err(unreachable_rpc())
    }

    async fn vote(
        &mut self,
        _rpc: openraft::raft::VoteRequest<NodeId>,
        _option: openraft::network::RPCOption,
    ) -> Result<
        openraft::raft::VoteResponse<NodeId>,
        openraft::error::RPCError<NodeId, BasicNode, openraft::error::RaftError<NodeId>>,
    > {
        Err(unreachable_rpc())
    }
}

fn unreachable_rpc<E: std::error::Error + 'static>(
) -> openraft::error::RPCError<NodeId, BasicNode, E> {
    openraft::error::RPCError::Unreachable(openraft::error::Unreachable::new(
        &std::io::Error::new(std::io::ErrorKind::Other, "multi-node RPC not wired (staged)"),
    ))
}

// ---------------------------------------------------------------------------
// RaftKv: single-node bring-up + a write path through consensus
// ---------------------------------------------------------------------------

pub type KeepRaft = openraft::Raft<TypeConfig>;

/// A single-node raft fronting a keep engine. Writes go through the raft log.
pub struct RaftKv {
    pub raft: KeepRaft,
    pub engine: Arc<KvEngine>,
}

impl RaftKv {
    /// Build a single-node raft and initialize it as a one-member cluster.
    pub async fn single_node(node_id: NodeId, engine: Arc<KvEngine>) -> anyhow::Result<Self> {
        let config = Arc::new(
            openraft::Config {
                heartbeat_interval: 250,
                election_timeout_min: 500,
                election_timeout_max: 1000,
                ..Default::default()
            }
            .validate()?,
        );
        let raft = openraft::Raft::new(
            node_id,
            config,
            Network,
            LogStore::default(),
            StateMachineStore::new(engine.clone()),
        )
        .await?;

        let mut members = BTreeMap::new();
        members.insert(node_id, BasicNode::default());
        raft.initialize(members).await?;
        // A single-node cluster elects itself; wait so the first write doesn't
        // race leadership.
        raft.wait(Some(std::time::Duration::from_secs(5)))
            .state(openraft::ServerState::Leader, "single-node leader")
            .await?;
        Ok(Self { raft, engine })
    }

    /// Propose a mutation through raft; resolves once committed + applied.
    pub async fn write(&self, op: WalOp) -> anyhow::Result<Response> {
        let r = self.raft.client_write(op).await?;
        Ok(r.data)
    }
}

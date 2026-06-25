//! lumen's consensus surface — re-exports the shared `raftcore` engine.
//!
//! The step-driven Raft core lives in `libs/raftcore` (serde-only) so relay,
//! keep and lumen share one verified implementation. lumen supplies the glue:
//! [`crate::raft_driver`] (h2c transport + surface-committed-to-WAL),
//! [`crate::raft_store`] (durable hard state) and [`crate::wal_raft`] (the
//! `WalLog` seam). This module exists to keep the engine surface separate from
//! lumen's existing cluster-state DTOs in [`crate::raft`].

pub use raftcore::{
    auto_membership, AppendReq, AppendResp, Index, InstallSnapshotReq, InstallSnapshotResp,
    Membership, NodeId, Outgoing, PersistedState, RaftEntry, RaftMsg, RaftNode, RaftTransport,
    Role, Term, VoteReq, VoteResp,
};

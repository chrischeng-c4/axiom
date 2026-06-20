// SPEC-MANAGED: projects/relay/tech-design/logic/single-shard-raft-consensus-core-self-contained-rsm-auto-voter-l.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:786bf09f" tracker="pending-tracker" reason="Single-shard Raft consensus core. The step-driven engine was extracted to the shared `raftcore` crate (libs/raftcore, serde-only) so relay and keep share one verified implementation; relay re-exports it and supplies the glue (raft_driver h2c transport, raft_store persistence, raft_config k8s)."
//! Single-shard Raft consensus core for relay.
//!
//! The step-driven engine now lives in the shared [`raftcore`] crate
//! (`libs/raftcore`, serde-only) so relay and keep share one verified
//! implementation. relay re-exports it here and supplies the glue:
//! [`crate::raft_driver`] (h2c transport + apply to the relay engine),
//! [`crate::raft_store`] (durable persistence) and [`crate::raft_config`]
//! (k8s identity / peers).
//!
//! @spec projects/relay/tech-design/logic/single-shard-raft-consensus-core-self-contained-rsm-auto-voter-l.md#logic

pub use raftcore::{
    auto_membership, AppendReq, AppendResp, Index, InstallSnapshotReq, InstallSnapshotResp,
    Membership, NodeId, Outgoing, PersistedState, RaftEntry, RaftMsg, RaftNode, RaftTransport,
    Role, Term, VoteReq, VoteResp,
};
// HANDWRITE-END

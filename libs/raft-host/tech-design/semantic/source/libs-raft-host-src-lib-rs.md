---
id: libs-raft-host-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/lib.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `cluster` | libs/raft-host/src/lib.rs | module | pub | 13 | pub mod cluster; |
| `llm` | libs/raft-host/src/lib.rs | module | pub | 16 | pub mod llm; |
| `parse_peer_overrides` | libs/raft-host/src/lib.rs | re-export | pub | 23 | pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology}; |
| `peer_ordinal` | libs/raft-host/src/lib.rs | re-export | pub | 23 | pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology}; |
| `replica_mode` | libs/raft-host/src/lib.rs | re-export | pub | 23 | pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology}; |
| `ClusterDims` | libs/raft-host/src/lib.rs | re-export | pub | 23 | pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology}; |
| `ClusterTopology` | libs/raft-host/src/lib.rs | re-export | pub | 23 | pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology}; |
| `HostConfig` | libs/raft-host/src/lib.rs | re-export | pub | 24 | pub use config::{HostConfig, SnapshotPolicy}; |
| `SnapshotPolicy` | libs/raft-host/src/lib.rs | re-export | pub | 24 | pub use config::{HostConfig, SnapshotPolicy}; |
| `RaftHost` | libs/raft-host/src/lib.rs | re-export | pub | 25 | pub use host::RaftHost; |
| `OutcomeWindow` | libs/raft-host/src/lib.rs | re-export | pub | 26 | pub use outcome_window::{OutcomeWindow, DEFAULT_CAPACITY as OUTCOME_WINDOW_DEFAULT_CAPACITY}; |
| `OUTCOME_WINDOW_DEFAULT_CAPACITY` | libs/raft-host/src/lib.rs | re-export | pub | 26 | pub use outcome_window::{OutcomeWindow, DEFAULT_CAPACITY as OUTCOME_WINDOW_DEFAULT_CAPACITY}; |
| `ReadConsistency` | libs/raft-host/src/lib.rs | re-export | pub | 27 | pub use read_consistency::{ReadConsistency, READ_CONSISTENCY_HEADER}; |
| `READ_CONSISTENCY_HEADER` | libs/raft-host/src/lib.rs | re-export | pub | 27 | pub use read_consistency::{ReadConsistency, READ_CONSISTENCY_HEADER}; |
| `Command` | libs/raft-host/src/lib.rs | re-export | pub | 28 | pub use state_machine::{Command, RaftStateMachine}; |
| `RaftStateMachine` | libs/raft-host/src/lib.rs | re-export | pub | 28 | pub use state_machine::{Command, RaftStateMachine}; |
| `FsyncPolicy` | libs/raft-host/src/lib.rs | re-export | pub | 29 | pub use store::{FsyncPolicy, RaftStore}; |
| `RaftStore` | libs/raft-host/src/lib.rs | re-export | pub | 29 | pub use store::{FsyncPolicy, RaftStore}; |
| `ClusterStateView` | libs/raft-host/src/lib.rs | re-export | pub | 30 | pub use view::{ClusterStateView, PeerAddr, RaftRole}; |
| `PeerAddr` | libs/raft-host/src/lib.rs | re-export | pub | 30 | pub use view::{ClusterStateView, PeerAddr, RaftRole}; |
| `RaftRole` | libs/raft-host/src/lib.rs | re-export | pub | 30 | pub use view::{ClusterStateView, PeerAddr, RaftRole}; |
| `auto_membership` | libs/raft-host/src/lib.rs | re-export | pub | 33 | pub use raft_core::{auto_membership, Index, Membership, NodeId, Term}; |
| `Index` | libs/raft-host/src/lib.rs | re-export | pub | 33 | pub use raft_core::{auto_membership, Index, Membership, NodeId, Term}; |
| `Membership` | libs/raft-host/src/lib.rs | re-export | pub | 33 | pub use raft_core::{auto_membership, Index, Membership, NodeId, Term}; |
| `NodeId` | libs/raft-host/src/lib.rs | re-export | pub | 33 | pub use raft_core::{auto_membership, Index, Membership, NodeId, Term}; |
| `Term` | libs/raft-host/src/lib.rs | re-export | pub | 33 | pub use raft_core::{auto_membership, Index, Membership, NodeId, Term}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `raft-host` — the ecosystem's shared raft driver.
//!
//! `libs/raft-core` is the step-driven consensus core; this crate is the **host**
//! that drives it for a [`RaftStateMachine`]: a tick/pump loop, the h2c peer
//! transport (Vote / Append / InstallSnapshot), the single apply loop, snapshot
//! + log compaction, a read-your-write [`RaftHost::propose`], and a peer
//! [`RaftHost::router`] to merge into the service's h2c port.
//!
//! Every raft_core service (lumen, keep, relay, loom) supplies a
//! [`RaftStateMachine`] (`apply`/`snapshot`/`restore`/`applied_index`) and gets
//! HA + the backup layer for free, instead of hand-rolling a driver.

pub mod cluster;
mod config;
mod host;
pub mod llm;
mod outcome_window;
mod read_consistency;
mod state_machine;
mod store;
mod view;

pub use cluster::{parse_peer_overrides, peer_ordinal, replica_mode, ClusterDims, ClusterTopology};
pub use config::{HostConfig, SnapshotPolicy};
pub use host::RaftHost;
pub use outcome_window::{OutcomeWindow, DEFAULT_CAPACITY as OUTCOME_WINDOW_DEFAULT_CAPACITY};
pub use read_consistency::{ReadConsistency, READ_CONSISTENCY_HEADER};
pub use state_machine::{Command, RaftStateMachine};
pub use store::{FsyncPolicy, RaftStore};
pub use view::{ClusterStateView, PeerAddr, RaftRole};

// Re-export the raft_core surface a host consumer needs (membership, ids).
pub use raft_core::{auto_membership, Index, Membership, NodeId, Term};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    /// A trivial state machine: applies `u64` commands into a log, tracks the
    /// applied index, snapshots/restores the whole log.
    struct CounterSm {
        log: Mutex<Vec<(Index, u64)>>,
        applied: AtomicU64,
    }
    impl CounterSm {
        fn new() -> Arc<Self> {
            Arc::new(CounterSm {
                log: Mutex::new(Vec::new()),
                applied: AtomicU64::new(0),
            })
        }
    }
    impl RaftStateMachine for CounterSm {
        fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
            let v = u64::from_le_bytes(command.try_into().unwrap_or([0; 8]));
            self.log.lock().unwrap().push((index, v));
            self.applied.store(index, Ordering::Release);
            Ok(())
        }
        fn snapshot(&self) -> anyhow::Result<Vec<u8>> {
            Ok(serde_json::to_vec(&*self.log.lock().unwrap())?)
        }
        fn restore(&self, snapshot: &[u8]) -> anyhow::Result<()> {
            let log: Vec<(Index, u64)> = serde_json::from_slice(snapshot)?;
            let last = log.last().map(|(i, _)| *i).unwrap_or(0);
            *self.log.lock().unwrap() = log;
            self.applied.store(last, Ordering::Release);
            Ok(())
        }
        fn applied_index(&self) -> Index {
            self.applied.load(Ordering::Acquire)
        }
    }

    fn store(dir: &std::path::Path, id: NodeId) -> RaftStore {
        RaftStore::open(dir.to_str().unwrap(), id, FsyncPolicy::Os).unwrap()
    }

    #[tokio::test]
    async fn single_node_propose_applies_read_your_write() {
        let tmp = std::env::temp_dir().join(format!("raft-host-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let sm = CounterSm::new();
        let host = RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            std::collections::HashMap::new(),
            store(&tmp, 0),
            sm.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        );
        // propose returns only after the SM has applied the entry (RYW).
        for v in 1..=3u64 {
            let idx = host.propose(v.to_le_bytes().to_vec()).await.unwrap();
            assert_eq!(idx, v);
            assert!(sm.applied_index() >= idx, "applied before propose returned");
        }
        let log = sm.log.lock().unwrap().clone();
        assert_eq!(log, vec![(1, 1), (2, 2), (3, 3)]);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn restart_replays_committed_log_into_a_fresh_sm() {
        let tmp = std::env::temp_dir().join(format!("raft-host-replay-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        // Use Always fsync so the log is durable across the "restart".
        let mk = |sm: Arc<dyn RaftStateMachine>| {
            RaftHost::spawn(
                0,
                Membership {
                    voters: vec![0],
                    learners: vec![],
                },
                std::collections::HashMap::new(),
                RaftStore::open(tmp.to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
                sm,
                HostConfig::default(),
            )
        };
        {
            let sm = CounterSm::new();
            let host = mk(sm.clone());
            host.propose(7u64.to_le_bytes().to_vec()).await.unwrap();
            host.propose(8u64.to_le_bytes().to_vec()).await.unwrap();
            assert_eq!(sm.applied_index(), 2);
        } // host dropped → tasks aborted (simulated restart)
          // A fresh SM cold-starts from the persisted raft log. Standard raft only
          // re-commits prior-term entries once a *current-term* entry commits, so
          // the backlog (7, 8) is replayed together with the first post-restart
          // write (9). (Services whose SM persists its own state — lumen's RDB/AOF
          // — recover without this; a memory-only SM needs the new-term commit.)
        let sm2 = CounterSm::new();
        let host2 = mk(sm2.clone());
        let idx = host2.propose(9u64.to_le_bytes().to_vec()).await.unwrap();
        assert_eq!(idx, 3);
        assert_eq!(sm2.applied_index(), 3);
        assert_eq!(
            sm2.log.lock().unwrap().clone(),
            vec![(1, 7), (2, 8), (3, 9)],
            "the backlog replays with the first new-term commit"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/lib.rs` captured during libs codegen standardization.
```

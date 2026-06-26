//! `raft-host` — the ecosystem's shared raft driver.
//!
//! `libs/raftcore` is the step-driven consensus core; this crate is the **host**
//! that drives it for a [`RaftStateMachine`]: a tick/pump loop, the h2c peer
//! transport (Vote / Append / InstallSnapshot), the single apply loop, snapshot
//! + log compaction, a read-your-write [`RaftHost::propose`], and a peer
//! [`RaftHost::router`] to merge into the service's h2c port.
//!
//! Every raftcore service (lumen, keep, relay, loom) supplies a
//! [`RaftStateMachine`] (`apply`/`snapshot`/`restore`/`applied_index`) and gets
//! HA + the backup layer for free, instead of hand-rolling a driver.

mod config;
mod host;
mod state_machine;
mod store;

pub use config::{HostConfig, SnapshotPolicy};
pub use host::RaftHost;
pub use state_machine::{Command, RaftStateMachine};
pub use store::{FsyncPolicy, RaftStore};

// Re-export the raftcore surface a host consumer needs (membership, ids).
pub use raftcore::{auto_membership, Index, Membership, NodeId, Term};

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

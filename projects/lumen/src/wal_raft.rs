//! `RaftWal` — a [`WalLog`] backed by lumen-owned raft replication.
//!
//! The seam that makes raft invisible to the rest of lumen: `publish` proposes
//! the record through [`crate::raft_driver::RaftDriver`] and returns once it
//! commits; `subscribe` tails the driver's surfaced committed log; `latest_seq`
//! is the committed head. The raft log index is the WAL seq directly (both
//! 1-based) — a cleaner mapping than `RelayWal`'s `seq + 1`.
//!
//! The `RaftWal` owns the `Arc<RaftDriver>`, so the driver's tick task lives for
//! as long as the WAL (i.e. the server) does.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::raft_driver::RaftDriver;
use crate::wal::{WalLog, WalRecord, WalStream};

/// A `WalLog` whose ordering + durability come from `libs/raftcore`.
pub struct RaftWal {
    driver: Arc<RaftDriver>,
}

impl RaftWal {
    pub fn new(driver: Arc<RaftDriver>) -> Self {
        Self { driver }
    }
}

#[async_trait]
impl WalLog for RaftWal {
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let cmd = record.encode()?;
        self.driver.propose_committed(cmd).await
    }

    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let committed = self.driver.committed();
        let rx = self.driver.commit_watch();
        // MemWal-style unfold: deliver the next seq after `delivered` from the
        // resident committed buffer (1-based, contiguous), else await a commit.
        let stream = futures::stream::unfold(
            (from_seq, rx, committed),
            |(delivered, mut rx, committed)| async move {
                loop {
                    let next = {
                        let buf = match committed.lock() {
                            Ok(b) => b,
                            Err(_) => return None,
                        };
                        let want = delivered + 1;
                        buf.get((want.saturating_sub(1)) as usize).cloned()
                    };
                    if let Some((seq, rec)) = next {
                        return Some((Ok((seq, rec)), (seq, rx, committed)));
                    }
                    if rx.changed().await.is_err() {
                        return None;
                    }
                }
            },
        );
        Ok(Box::pin(stream))
    }

    async fn latest_seq(&self) -> Result<u64> {
        Ok(self.driver.latest_committed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_entry::RaftLogEntry;
    use crate::raft_core::Membership;
    use crate::raft_driver::RaftDriver;
    use crate::raft_store::{FsyncPolicy, RaftStore};
    use crate::types::CreateCollectionRequest;
    use futures::StreamExt;
    use std::collections::{BTreeMap, HashMap};

    fn create_record(coll: &str) -> WalRecord {
        WalRecord::new(RaftLogEntry::CreateCollection {
            collection_id: coll.into(),
            req: CreateCollectionRequest {
                fields: BTreeMap::new(),
            },
        })
    }

    fn single_node_wal(dir: &str) -> RaftWal {
        let store = RaftStore::open(dir, 0, FsyncPolicy::Os).unwrap();
        let membership = Membership {
            voters: vec![0],
            learners: vec![],
        };
        let driver = Arc::new(RaftDriver::spawn(0, membership, HashMap::new(), store));
        RaftWal::new(driver)
    }

    #[tokio::test]
    async fn single_node_publish_then_subscribe_round_trips() {
        let tmp = std::env::temp_dir().join(format!("lumen-raftwal-{}", std::process::id()));
        let dir = tmp.to_string_lossy().to_string();
        let wal = single_node_wal(&dir);

        // publish three records; raft index == seq, 1-based, in order.
        let s1 = wal.publish(create_record("a")).await.unwrap();
        let s2 = wal.publish(create_record("b")).await.unwrap();
        let s3 = wal.publish(create_record("c")).await.unwrap();
        assert_eq!((s1, s2, s3), (1, 2, 3));
        assert_eq!(wal.latest_seq().await.unwrap(), 3);

        // subscribe(0) replays the committed backlog in order.
        let mut stream = wal.subscribe(0).await.unwrap();
        for expect in 1..=3u64 {
            let (seq, _rec) = stream.next().await.unwrap().unwrap();
            assert_eq!(seq, expect);
        }

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn subscribe_from_offset_skips_backlog_and_tails() {
        let tmp = std::env::temp_dir().join(format!("lumen-raftwal-tail-{}", std::process::id()));
        let dir = tmp.to_string_lossy().to_string();
        let wal = single_node_wal(&dir);

        wal.publish(create_record("a")).await.unwrap();
        wal.publish(create_record("b")).await.unwrap();
        let mut stream = wal.subscribe(1).await.unwrap(); // skip seq 1
        let (seq, _) = stream.next().await.unwrap().unwrap();
        assert_eq!(seq, 2);

        // a later append tails live.
        let s3 = wal.publish(create_record("c")).await.unwrap();
        assert_eq!(s3, 3);
        let (seq, _) = stream.next().await.unwrap().unwrap();
        assert_eq!(seq, 3);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

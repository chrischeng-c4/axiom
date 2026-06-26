//! raft_core-backed consensus (HA phase C). Only compiles/runs with
//! `--features raft`. Proves a write goes through the Raft log (propose →
//! commit → apply) onto the engine, survives a snapshot/compaction, and that
//! per-shard groups route writes by key.

#![cfg(feature = "raft")]

use std::sync::Arc;

use keep::persistence::format::WalOp;
use keep::raft::{RaftKv, ShardedRaft};
use keep::{ClusterConfig, KvEngine, KvKey, KvValue};

#[tokio::test]
async fn single_group_commits_applies_and_snapshots() {
    let engine = Arc::new(KvEngine::with_shards(4));
    let raft = RaftKv::single_node(1, engine.clone()).await.unwrap();

    // A write proposed through Raft is committed and applied to the engine.
    let resp = raft
        .write(WalOp::Set {
            key: "rk".to_string(),
            value: KvValue::String("via-raft".to_string()),
            ttl: None,
        })
        .await
        .unwrap();
    assert!(resp.applied, "command should report applied");
    assert_eq!(
        engine.get(&KvKey::new("rk").unwrap()),
        Some(KvValue::String("via-raft".to_string())),
        "raft-committed write must be applied to the engine"
    );

    raft.write(WalOp::Delete {
        key: "rk".to_string(),
    })
    .await
    .unwrap();
    assert_eq!(engine.get(&KvKey::new("rk").unwrap()), None);

    // A committed op then a snapshot/compaction; engine state is preserved.
    raft.write(WalOp::Set {
        key: "k2".to_string(),
        value: KvValue::String("v2".to_string()),
        ttl: None,
    })
    .await
    .unwrap();
    raft.snapshot().await.unwrap();
    assert_eq!(
        engine.get(&KvKey::new("k2").unwrap()),
        Some(KvValue::String("v2".to_string())),
        "engine survives compaction"
    );
}

#[tokio::test]
async fn per_shard_groups_route_writes_by_key() {
    let engine = Arc::new(KvEngine::with_shards(4));
    // A single node that owns every shard, but as one Raft group per shard.
    let cluster = ClusterConfig::new(0, 1, 8, vec![]);
    let sharded = ShardedRaft::new(cluster, engine.clone()).await.unwrap();
    assert!(sharded.group_count() >= 1, "one Raft group per owned shard");

    for i in 0..20 {
        let key = format!("key-{i}");
        sharded
            .write(
                &key,
                WalOp::Set {
                    key: key.clone(),
                    value: KvValue::String(format!("v{i}")),
                    ttl: None,
                },
            )
            .await
            .unwrap();
    }
    // Every routed write committed through its shard's group and hit the engine.
    for i in 0..20 {
        let key = format!("key-{i}");
        assert_eq!(
            engine.get(&KvKey::new(key).unwrap()),
            Some(KvValue::String(format!("v{i}"))),
            "key-{i} replicated + applied via its shard group"
        );
    }
}

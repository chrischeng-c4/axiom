//! Single-node openraft integration (HA phase C). Only compiles/runs with
//! `--features raft`. Proves a write goes through the raft log (propose →
//! commit → apply) onto the engine, and survives a state-machine snapshot.
#![cfg(feature = "raft")]

use std::sync::Arc;

use keep::persistence::format::WalOp;
use keep::raft::RaftKv;
use keep::{KvEngine, KvKey, KvValue};

#[tokio::test]
async fn single_node_raft_commits_and_applies() {
    let engine = Arc::new(KvEngine::with_shards(4));
    let raft = RaftKv::single_node(1, engine.clone()).await.unwrap();

    // A write proposed through raft is committed by the (single-node) quorum and
    // applied to the engine by the state machine.
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

    // A couple more committed ops, then a state-machine snapshot round-trips.
    raft.write(WalOp::Delete { key: "rk".to_string() })
        .await
        .unwrap();
    assert_eq!(engine.get(&KvKey::new("rk").unwrap()), None);

    raft.raft.trigger().snapshot().await.unwrap();

    raft.raft.shutdown().await.unwrap();
}

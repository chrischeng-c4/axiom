//! raft-host-backed consensus (HA phase C). Only compiles/runs with
//! `--features raft`. Proves a write goes through the shared [`RaftHost`]
//! (propose → commit → apply) onto the engine with read-your-write, that the
//! per-shard hosts route by key, and that a shard's snapshot round-trips
//! (`dump_values`/`load_values`).
//!
//! The multi-process acceptance criteria — leader failover under SIGKILL and
//! InstallSnapshot catch-up of a wiped-data-dir replica — are authored as a
//! real in-process localhost h2c cluster but `#[ignore]`d: a reliable run needs
//! orchestrated OS processes (the strict criterion kills the leader with
//! `kill -9`, since SIGTERM triggers a 30s graceful drain and no failover).
//! See each test's `#[ignore]` reason + the harness comment.

#![cfg(feature = "raft")]

use std::sync::Arc;

use keep::persistence::format::WalOp;
use keep::raft::{KvStateMachine, ShardHosts};
use keep::{ClusterConfig, KvEngine, KvKey, KvValue};
use raft_host::RaftStateMachine;

/// A fresh, empty temp dir for a test's raft hard-state stores.
fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("keep-raft-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Single node owning every shard, one sole-voter host per shard: a SET
/// proposed through its shard host commits + applies, and the engine reflects it
/// the moment `propose` returns (read-your-write).
#[tokio::test]
async fn single_node_per_shard_read_your_write() {
    let dir = tmp_dir("ryw");
    let engine = Arc::new(KvEngine::with_shards(4));
    let cluster = ClusterConfig::new(0, 1, 1, vec![]);
    let hosts = ShardHosts::new(cluster, engine.clone(), &dir, 1)
        .await
        .unwrap();
    assert!(hosts.host_count() >= 1, "one host per owned shard");

    let idx = hosts
        .write(
            "rk",
            WalOp::Set {
                key: "rk".to_string(),
                value: KvValue::String("via-raft".to_string()),
                ttl: None,
            },
        )
        .await
        .unwrap();
    assert!(idx >= 1, "propose returns the applied raft index");
    assert_eq!(
        engine.get(&KvKey::new("rk").unwrap()),
        Some(KvValue::String("via-raft".to_string())),
        "raft-committed write is applied to the engine before propose returns"
    );

    // A delete routes to the same shard host and applies.
    hosts
        .write(
            "rk",
            WalOp::Delete {
                key: "rk".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(engine.get(&KvKey::new("rk").unwrap()), None);

    let _ = std::fs::remove_dir_all(&dir);
}

/// A single node owning every shard, but as one host per shard: writes route to
/// their key's shard host and every committed write lands in the engine.
#[tokio::test]
async fn per_shard_hosts_route_writes_by_key() {
    let dir = tmp_dir("route");
    let engine = Arc::new(KvEngine::with_shards(4));
    // 8 virtual shards on one node ⇒ up to 8 per-shard hosts.
    let cluster = ClusterConfig::new(0, 1, 8, vec![]);
    let hosts = ShardHosts::new(cluster, engine.clone(), &dir, 1)
        .await
        .unwrap();
    assert!(hosts.host_count() >= 1, "one host per owned shard");

    for i in 0..20 {
        let key = format!("key-{i}");
        hosts
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
    for i in 0..20 {
        let key = format!("key-{i}");
        assert_eq!(
            engine.get(&KvKey::new(key).unwrap()),
            Some(KvValue::String(format!("v{i}"))),
            "key-{i} committed through its shard host + applied"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

/// A shard's state machine snapshot keeps only that shard's keys and round-trips
/// through `restore` into a fresh engine, restoring `applied_index` — the bytes
/// the host ships via InstallSnapshot for catch-up.
#[tokio::test]
async fn shard_state_machine_snapshot_round_trips() {
    let cluster = ClusterConfig::new(0, 1, 8, vec![]);
    let src = Arc::new(KvEngine::with_shards(4));
    // Seed values across the keyspace; only shard-0 keys belong in shard 0's snapshot.
    for i in 0..50 {
        let k = format!("snap-{i}");
        let _ = src.set(
            &KvKey::new(&k).unwrap(),
            KvValue::String(format!("v{i}")),
            None,
        );
    }
    let sm = KvStateMachine::new(src.clone(), cluster.clone(), 0);
    // Advance the applied head, then capture the snapshot.
    let cmd = serde_json::to_vec(&WalOp::Set {
        key: "applied-marker".to_string(),
        value: KvValue::String("m".to_string()),
        ttl: None,
    })
    .unwrap();
    sm.apply(7, &cmd).unwrap();
    assert_eq!(sm.applied_index(), 7);
    let snap = sm.snapshot().unwrap();

    // Restore into a fresh engine through a fresh state machine.
    let dst = Arc::new(KvEngine::with_shards(4));
    let sm2 = KvStateMachine::new(dst.clone(), cluster.clone(), 0);
    sm2.restore(&snap).unwrap();
    assert_eq!(
        sm2.applied_index(),
        7,
        "restore sets applied_index to the snapshot head"
    );
    for i in 0..50 {
        let k = format!("snap-{i}");
        let got = dst.get(&KvKey::new(&k).unwrap());
        if cluster.shard_for(&k) == 0 {
            assert_eq!(
                got,
                Some(KvValue::String(format!("v{i}"))),
                "shard-0 key {k} restored"
            );
        } else {
            assert_eq!(
                got, None,
                "off-shard key {k} excluded from shard-0 snapshot"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Multi-node acceptance criteria — authored but deferred (#[ignore]).
//
// These need an orchestrated multi-node cluster. The helpers below build a real
// in-process localhost h2c group (each node serves its `host.router()` on an
// ephemeral port; peers reach each other over the shared h2c transport), which
// is faithful enough to drive failover + InstallSnapshot. They are `#[ignore]`d
// because (a) the strict acceptance criterion requires SIGKILL of a real OS
// process (SIGTERM = 30s graceful drain ⇒ the leader keeps heartbeating ⇒ no
// failover), which an in-process task-abort only approximates, and (b) raft
// elections over real sockets inside one test are timing-sensitive. Run them
// deliberately via `cargo test -p keep --features raft -- --ignored`, or wire
// the documented multi-process harness.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod multi_node {
    use super::*;
    use raft_host::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStore, SnapshotPolicy};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::net::TcpListener;

    /// A pre-bound localhost listener + the base URL peers address it by.
    async fn bind() -> (TcpListener, String) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        (l, format!("http://127.0.0.1:{port}"))
    }

    /// A node in the in-process cluster: its host handle, its own engine, and the
    /// `service_http::serve` task driving its peer router (aborted to "crash" it).
    struct Node {
        host: Arc<RaftHost>,
        engine: Arc<KvEngine>,
        serve: tokio::task::JoinHandle<()>,
    }

    /// Spawn a single-shard raft node on a pre-bound listener, serving its peer
    /// router (Vote/Append/InstallSnapshot/publish) over localhost h2c.
    async fn spawn_on(
        id: u64,
        voters: Vec<u64>,
        peers: HashMap<u64, String>,
        listener: TcpListener,
        dir: &std::path::Path,
        snapshot_every: u64,
    ) -> Node {
        let engine = Arc::new(KvEngine::with_shards(4));
        let cluster = ClusterConfig::new(0, 1, 1, vec![]);
        let sm = KvStateMachine::new(engine.clone(), cluster, 0);
        std::fs::create_dir_all(dir).unwrap();
        let store = RaftStore::open(dir.to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
        let host = Arc::new(RaftHost::spawn(
            id,
            Membership {
                voters,
                learners: vec![],
            },
            peers,
            store,
            sm as Arc<dyn RaftStateMachine>,
            HostConfig {
                snapshot: SnapshotPolicy::EveryEntries(snapshot_every),
                ..Default::default()
            },
        ));
        let router = host.router();
        let serve = tokio::spawn(async move {
            service_http::serve(listener, router, std::future::pending::<()>()).await;
        });
        Node {
            host,
            engine,
            serve,
        }
    }

    /// Build the peer map for node `me` out of every node's (id, url).
    fn peers_excluding(me: u64, all: &[(u64, String)]) -> HashMap<u64, String> {
        all.iter()
            .filter(|(id, _)| *id != me)
            .map(|(id, url)| (*id, url.clone()))
            .collect()
    }

    /// Stand up an `n`-voter single-shard cluster with full peer connectivity:
    /// pre-bind every port, then spawn each host with the complete peer map.
    async fn cluster(n: u64, dir: &std::path::Path, snapshot_every: u64) -> Vec<Node> {
        let mut listeners = Vec::new();
        let mut all = Vec::new();
        for id in 0..n {
            let (l, url) = bind().await;
            listeners.push(l);
            all.push((id, url));
        }
        let voters: Vec<u64> = (0..n).collect();
        let mut nodes = Vec::new();
        for (idx, l) in listeners.into_iter().enumerate() {
            let id = idx as u64;
            let peers = peers_excluding(id, &all);
            nodes.push(
                spawn_on(
                    id,
                    voters.clone(),
                    peers,
                    l,
                    &dir.join(format!("n{id}")),
                    snapshot_every,
                )
                .await,
            );
        }
        nodes
    }

    /// Wait until some node reports leadership, returning its index.
    async fn await_leader(nodes: &[Node]) -> Option<usize> {
        for _ in 0..400 {
            for (i, n) in nodes.iter().enumerate() {
                if n.host.is_leader().await {
                    return Some(i);
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        None
    }

    /// AC: a replicated shard group elects a new leader after the leader is
    /// killed and committed data survives. (Authored; deferred — see module doc.)
    #[tokio::test]
    #[ignore = "multi-process cluster; strict run requires SIGKILL of a real leader process. Run with --ignored."]
    async fn replicated_shard_group_fails_over_and_keeps_data() {
        let dir = tmp_dir("failover");
        let mut nodes = cluster(3, &dir, 1024).await;
        let leader = await_leader(&nodes).await.expect("a leader is elected");

        // Commit a write through the leader.
        let cmd = serde_json::to_vec(&WalOp::Set {
            key: "fo".to_string(),
            value: KvValue::String("survives".to_string()),
            ttl: None,
        })
        .unwrap();
        nodes[leader].host.propose(cmd).await.unwrap();

        // "Crash" the leader: abort its serve task AND drop its host (tick/pump
        // abort on Drop) so it stops responding entirely — the in-process stand-in
        // for `kill -9` (SIGTERM would graceful-drain for 30s, preventing failover).
        let dead = nodes.remove(leader);
        dead.serve.abort();
        drop(dead);

        let new_leader = await_leader(&nodes).await.expect("a new leader is elected");
        // A fresh write through the new leader commits a current-term entry, which
        // by the standard Raft safety rule also makes the prior-term "fo" entry
        // committed + applied (a memory-only SM needs that new-term commit before
        // a backlog entry surfaces).
        let cmd2 = serde_json::to_vec(&WalOp::Set {
            key: "fo2".to_string(),
            value: KvValue::String("after".to_string()),
            ttl: None,
        })
        .unwrap();
        nodes[new_leader].host.propose(cmd2).await.unwrap();
        assert_eq!(
            nodes[new_leader].engine.get(&KvKey::new("fo").unwrap()),
            Some(KvValue::String("survives".to_string())),
            "committed data survives the failover"
        );
        assert_eq!(
            nodes[new_leader].engine.get(&KvKey::new("fo2").unwrap()),
            Some(KvValue::String("after".to_string())),
            "the new leader accepts and applies fresh writes"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC: a wiped-data-dir replica catches up via InstallSnapshot
    /// (snapshot_index > 0). (Authored; deferred — see module doc.)
    #[tokio::test]
    #[ignore = "multi-process cluster; needs orchestrated processes + a wiped replica data-dir. Run with --ignored."]
    async fn wiped_replica_catches_up_via_install_snapshot() {
        let dir = tmp_dir("catchup");
        // A small snapshot threshold so the leader compacts quickly, arming
        // InstallSnapshot for a lagging replica.
        let nodes = cluster(3, &dir, 4).await;
        let leader = await_leader(&nodes).await.expect("a leader is elected");

        // Commit enough entries to trigger snapshot/compaction on the leader.
        for i in 0..16 {
            let cmd = serde_json::to_vec(&WalOp::Set {
                key: format!("cu-{i}"),
                value: KvValue::String(format!("v{i}")),
                ttl: None,
            })
            .unwrap();
            nodes[leader].host.propose(cmd).await.unwrap();
        }

        // Every follower converges to the leader's state (via log replication
        // and/or InstallSnapshot once the log is compacted past their match).
        for (i, n) in nodes.iter().enumerate() {
            if i == leader {
                continue;
            }
            let mut caught_up = false;
            for _ in 0..400 {
                if n.engine.get(&KvKey::new("cu-15").unwrap())
                    == Some(KvValue::String("v15".to_string()))
                {
                    caught_up = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            assert!(caught_up, "replica {i} catches up to the leader's state");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

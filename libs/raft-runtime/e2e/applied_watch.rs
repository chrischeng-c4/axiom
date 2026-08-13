use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

use raft_runtime::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::*;

#[tokio::test]
async fn single_voter_late_subscriber() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );

    for v in 1..=4u64 {
        let idx = host.propose(v.to_le_bytes().to_vec()).await.unwrap();
        assert_eq!(idx, v);
    }

    let watch_val = *host.applied_watch().borrow();
    let sm_val = sm.applied_index();
    println!("sm=[{sm_val}] fresh_watch=[{watch_val}]");

    assert_eq!(sm_val, 4);
    assert_eq!(watch_val, 4);
}

#[tokio::test]
async fn held_subscriber() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");

    let mut receivers = Vec::new();
    for n in &nodes {
        receivers.push(n.host.applied_watch());
    }

    for v in 1..=4u64 {
        let idx = nodes[leader]
            .host
            .propose(v.to_le_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(idx, v);
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    for n in &nodes {
        while n.sm.applied_index() < 4 {
            if std::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let mut sm_vals = Vec::new();
    let mut watch_vals = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        sm_vals.push(n.sm.applied_index());
        watch_vals.push(*receivers[i].borrow());
    }
    println!("sm={sm_vals:?} fresh_watch={watch_vals:?}");

    for sm_val in &sm_vals {
        assert_eq!(*sm_val, 4);
    }
    for watch_val in &watch_vals {
        assert_eq!(*watch_val, 4);
    }
}

#[tokio::test]
async fn three_voter_late_subscriber() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");

    for v in 1..=4u64 {
        let idx = nodes[leader]
            .host
            .propose(v.to_le_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(idx, v);
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    for n in &nodes {
        while n.sm.applied_index() < 4 {
            if std::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let mut sm_vals = Vec::new();
    let mut watch_vals = Vec::new();
    for n in &nodes {
        sm_vals.push(n.sm.applied_index());
        watch_vals.push(*n.host.applied_watch().borrow());
    }
    println!("sm={sm_vals:?} fresh_watch={watch_vals:?}");

    for sm_val in &sm_vals {
        assert_eq!(*sm_val, 4);
    }
    for watch_val in &watch_vals {
        assert_eq!(*watch_val, 4);
    }
}

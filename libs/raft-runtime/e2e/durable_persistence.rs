use std::io::ErrorKind;
use std::sync::Arc;
use raft_runtime::{FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::{cluster, TestSm};

fn to_command(n: u64) -> Vec<u8> {
    n.to_le_bytes().to_vec()
}

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

#[tokio::test]
async fn test_1_propose_refusal_and_restart() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    let host = node.host.clone();
    
    let idx1 = host.propose(to_command(1)).await.unwrap();
    assert_eq!(idx1, 1);
    
    host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    
    let err = host.propose(to_command(2)).await.unwrap_err();
    assert_eq!(node.sm.applied_index(), 1);
    
    // Propose 3 (Measurement 2)
    let err2 = host.propose(to_command(3)).await.unwrap_err();
    assert_eq!(err.to_string(), err2.to_string());
    assert_eq!(node.sm.applied_index(), 1);

    let cluster::Node { host, sm: _sm, url: _url, _serve, _dir } = node;
    let dir_path = _dir.into_path();
    drop(host);

    let sm2 = TestSm::new();
    let store2 = RaftStore::open(dir_path.to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let _host2 = RaftHost::spawn(
        0,
        Membership { voters: vec![0], learners: vec![] },
        std::collections::HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );
    assert_eq!(sm2.applied_index(), 1);
    std::fs::remove_dir_all(dir_path).unwrap();
}

#[tokio::test]
async fn test_2_request_vote_refusal() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    
    node.host.propose(to_command(1)).await.unwrap();
    node.host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    let _ = node.host.propose(to_command(2)).await;

    let client = h2c_client();
    let vote_resp: serde_json::Value = client
        .post(&format!("{}/raft/request-vote", node.url))
        .json(&serde_json::json!({
            "from": 1,
            "req": {
                "term": 2,
                "candidate": 1,
                "last_log_index": 2,
                "last_log_term": 2,
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert_eq!(vote_resp.get("granted").unwrap().as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_3_append_entries_refusal() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    
    node.host.propose(to_command(1)).await.unwrap();
    node.host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    let _ = node.host.propose(to_command(2)).await;

    let client = h2c_client();
    let append_resp: serde_json::Value = client
        .post(&format!("{}/raft/append-entries", node.url))
        .json(&serde_json::json!({
            "from": 1,
            "req": {
                "term": 2,
                "leader": 1,
                "prev_log_index": 2,
                "prev_log_term": 2,
                "entries": [],
                "leader_commit": 2,
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    assert_eq!(append_resp.get("success").unwrap().as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_4_error_value_and_raftz() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    let host = node.host.clone();
    
    host.propose(to_command(1)).await.unwrap();
    host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    let err = host.propose(to_command(2)).await.unwrap_err();
    
    let err_str = err.to_string();
    assert!(err_str.contains("StorageFull"));
    assert!(err_str.contains("save"));
    assert!(err_str.contains("raft-0.state"));
    
    let client = h2c_client();
    let raftz_resp: serde_json::Value = client
        .get(&format!("{}/raftz", node.url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    let durability_error = raftz_resp.get("durability_error").unwrap().as_str().unwrap();
    assert!(durability_error.contains("StorageFull"));
}

#[tokio::test]
async fn test_5_healthy_host() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    let host = node.host.clone();
    
    host.propose(to_command(1)).await.unwrap();
    host.propose(to_command(2)).await.unwrap();
    host.propose(to_command(3)).await.unwrap();
    assert_eq!(node.sm.applied_index(), 3);

    let cluster::Node { host, sm: _sm, url: _url, _serve, _dir } = node;
    let dir_path = _dir.into_path();
    drop(host);

    let sm2 = TestSm::new();
    let store2 = RaftStore::open(dir_path.to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let _host2 = RaftHost::spawn(
        0,
        Membership { voters: vec![0], learners: vec![] },
        std::collections::HashMap::new(),
        store2,
        sm2.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );
    assert_eq!(sm2.applied_index(), 3);
    std::fs::remove_dir_all(dir_path).unwrap();
}

#[tokio::test]
async fn test_6_install_snapshot_refusal() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    
    node.host.propose(to_command(1)).await.unwrap();
    node.host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    let _ = node.host.propose(to_command(2)).await;

    let client = h2c_client();
    let snap_resp: serde_json::Value = client
        .post(&format!("{}/raft/install-snapshot", node.url))
        .json(&serde_json::json!({
            "from": 1,
            "req": {
                "term": 2,
                "leader": 1,
                "snapshot_index": 2,
                "snapshot_term": 2,
                "data": [],
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
    // An install snapshot on a latched host fails and returns a dummy response like index 0
    assert_eq!(snap_resp.get("snapshot_index").unwrap().as_u64().unwrap(), 0);
}

#[tokio::test]
async fn test_7_peer_requests_healthy_vs_latched() {
    let mut nodes = cluster(1).await;
    let node = nodes.pop().unwrap();
    
    // Propose one command so the log reaches index 1 at term 1.
    node.host.propose(to_command(1)).await.unwrap();

    let client = h2c_client();

    // 1. Healthy host accepts request_vote
    let vote_req = serde_json::json!({
        "from": 1,
        "req": {
            "term": 2,
            "candidate": 1,
            "last_log_index": 1,
            "last_log_term": 1,
        }
    });
    let vote_resp: serde_json::Value = client
        .post(&format!("{}/raft/request-vote", node.url))
        .json(&vote_req)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(vote_resp.get("granted").unwrap().as_bool().unwrap(), true);

    // 2. Healthy host accepts append_entries
    let append_req = serde_json::json!({
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "prev_log_index": 1,
            "prev_log_term": 1,
            "entries": [],
            "leader_commit": 1,
        }
    });
    let append_resp: serde_json::Value = client
        .post(&format!("{}/raft/append-entries", node.url))
        .json(&append_req)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(append_resp.get("success").unwrap().as_bool().unwrap(), true);

    // 3. Healthy host accepts install_snapshot
    let snap_req = serde_json::json!({
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "snapshot_index": 1,
            "snapshot_term": 1,
            "data": [],
        }
    });
    let snap_resp: serde_json::Value = client
        .post(&format!("{}/raft/install-snapshot", node.url))
        .json(&snap_req)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(snap_resp.get("snapshot_index").unwrap().as_u64().unwrap(), 1);

    // Now latch the host!
    node.host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);
    // propose to trigger the save failure
    let _ = node.host.propose(to_command(2)).await;

    // 1b. Latched host refuses request_vote (with term 3 to avoid stale term rejection if it wasn't latched)
    let vote_req_latched = serde_json::json!({
        "from": 1,
        "req": {
            "term": 3,
            "candidate": 1,
            "last_log_index": 1,
            "last_log_term": 1,
        }
    });
    let vote_resp_latched: serde_json::Value = client
        .post(&format!("{}/raft/request-vote", node.url))
        .json(&vote_req_latched)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(vote_resp_latched.get("granted").unwrap().as_bool().unwrap(), false);

    // 2b. Latched host refuses append_entries
    let append_req_latched = serde_json::json!({
        "from": 1,
        "req": {
            "term": 3,
            "leader": 1,
            "prev_log_index": 1,
            "prev_log_term": 1,
            "entries": [],
            "leader_commit": 1,
        }
    });
    let append_resp_latched: serde_json::Value = client
        .post(&format!("{}/raft/append-entries", node.url))
        .json(&append_req_latched)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(append_resp_latched.get("success").unwrap().as_bool().unwrap(), false);

    // 3b. Latched host refuses install_snapshot
    let snap_req_latched = serde_json::json!({
        "from": 1,
        "req": {
            "term": 3,
            "leader": 1,
            "snapshot_index": 1,
            "snapshot_term": 1,
            "data": [],
        }
    });
    let snap_resp_latched: serde_json::Value = client
        .post(&format!("{}/raft/install-snapshot", node.url))
        .json(&snap_req_latched)
        .send().await.unwrap().json().await.unwrap();
    assert_eq!(snap_resp_latched.get("snapshot_index").unwrap().as_u64().unwrap(), 0);
}

#[tokio::test]
async fn test_8_latched_follower_refuses_proposal() {
    use cluster::await_leader;
    let nodes = cluster(3).await;
    let leader_idx = await_leader(&nodes).await.unwrap();
    let follower_idx = (leader_idx + 1) % 3;
    let other_follower_idx = (leader_idx + 2) % 3;

    let leader = &nodes[leader_idx];
    let follower = &nodes[follower_idx];

    // Latch the follower
    follower.host.store().inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    // Trigger the save failure on the follower by proposing on the leader.
    // The leader sends append_entries, the follower receives it and tries to persist.
    leader.host.propose(to_command(1)).await.unwrap();

    let client = h2c_client();
    for _ in 0..50 {
        let raftz_resp: serde_json::Value = client
            .get(&format!("{}/raftz", follower.url))
            .send().await.unwrap().json().await.unwrap();
        if !raftz_resp.get("durability_error").unwrap().is_null() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Now observe the latched follower's propose returns an error (refused).
    let err = follower.host.propose(to_command(2)).await.unwrap_err();
    assert!(err.to_string().contains("StorageFull"));

    // The leader still accepts and applies commands.
    let _idx = leader.host.propose(to_command(3)).await.unwrap();
    
    // Wait for the healthy follower to apply it to prove the cluster is still healthy.
    for _ in 0..50 {
        if nodes[other_follower_idx].sm.applied_index() >= 2 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(nodes[other_follower_idx].sm.applied_index() >= 2);
}

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStatus, RaftStore,
    LEGACY_GROUP_ID,
};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::*;

#[tokio::test]
async fn failed_state_machine_restore_keeps_the_old_snapshot_and_log() {
    let (listener, url) = bind().await;
    let data = tempfile::tempdir().unwrap();
    let state_machine = TestSm::new();
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
        state_machine.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let router = host.router();
    let server = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = router.clone();
            tokio::spawn(async move {
                let _ = transport_h2c::server::serve_connection(stream, router).await;
            });
        }
    });

    host.propose(vec![1]).await.unwrap();
    state_machine.fail_restore.store(true, Ordering::Release);
    let response: serde_json::Value = transport_h2c::h2c_client_with(None, None)
        .unwrap()
        .post(format!("{url}/raft/install-snapshot"))
        .json(&serde_json::json!({
            "group_id": LEGACY_GROUP_ID,
            "from": 1,
            "req": {
                "term": 2,
                "leader": 1,
                "snapshot_index": 2,
                "snapshot_term": 2,
                "data": [9, 9, 9]
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["snapshot_index"], 0);
    assert_eq!(response["accepted"], false);
    assert_eq!(state_machine.restore_attempts.load(Ordering::Acquire), 1);

    let persisted = host.store().load().unwrap().unwrap();
    assert_eq!(persisted.snapshot_index, 0);
    assert_eq!(persisted.log.len(), 1);
    assert_eq!(persisted.log[0].index, 1);
    server.abort();
}

#[tokio::test]
async fn equal_index_snapshot_retry_requires_the_same_identity() {
    let (listener, url) = bind().await;
    let data = tempfile::tempdir().unwrap();
    let state_machine = TestSm::new();
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
        state_machine.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let router = host.router();
    let server = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = router.clone();
            tokio::spawn(async move {
                let _ = transport_h2c::server::serve_connection(stream, router).await;
            });
        }
    });

    for value in 1_u8..=4 {
        host.propose(vec![value]).await.unwrap();
    }
    host.snapshot_and_compact_through(4).await.unwrap();
    let before = host.store().load().unwrap().unwrap();
    assert_eq!(before.snapshot_index, 4);

    let client = transport_h2c::h2c_client_with(None, None).unwrap();
    let identical: serde_json::Value = client
        .post(format!("{url}/raft/install-snapshot"))
        .json(&serde_json::json!({
            "group_id": LEGACY_GROUP_ID,
            "from": 1,
            "req": {
                "term": before.term,
                "leader": 1,
                "snapshot_index": before.snapshot_index,
                "snapshot_term": before.snapshot_term,
                "data": before.snapshot
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(identical["accepted"], true);
    assert_eq!(identical["snapshot_index"], 4);

    let divergent: serde_json::Value = client
        .post(format!("{url}/raft/install-snapshot"))
        .json(&serde_json::json!({
            "group_id": LEGACY_GROUP_ID,
            "from": 1,
            "req": {
                "term": before.term,
                "leader": 1,
                "snapshot_index": before.snapshot_index,
                "snapshot_term": before.snapshot_term,
                "data": [9, 9, 9]
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(divergent["accepted"], false);
    assert_eq!(divergent["snapshot_index"], 4);
    assert_eq!(state_machine.restore_attempts.load(Ordering::Acquire), 0);

    let after = host.store().load().unwrap().unwrap();
    assert_eq!(after.snapshot_index, before.snapshot_index);
    assert_eq!(after.snapshot_term, before.snapshot_term);
    assert_eq!(after.snapshot, before.snapshot);
    server.abort();
}

#[tokio::test]
async fn raft_save_failure_happens_before_state_machine_restore() {
    let (listener, url) = bind().await;
    let data = tempfile::tempdir().unwrap();
    let state_machine = TestSm::new();
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: Vec::new(),
        },
        HashMap::new(),
        RaftStore::open(data.path().to_str().unwrap(), 0, FsyncPolicy::Always).unwrap(),
        state_machine.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let router = host.router();
    let server = tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = router.clone();
            tokio::spawn(async move {
                let _ = transport_h2c::server::serve_connection(stream, router).await;
            });
        }
    });

    host.propose(vec![1]).await.unwrap();
    let before = host.store().load().unwrap().unwrap();
    host.store()
        .inject_next_save_failure_with_kind(std::io::ErrorKind::StorageFull);
    let response: serde_json::Value = transport_h2c::h2c_client_with(None, None)
        .unwrap()
        .post(format!("{url}/raft/install-snapshot"))
        .json(&serde_json::json!({
            "group_id": LEGACY_GROUP_ID,
            "from": 1,
            "req": {
                "term": before.term + 1,
                "leader": 1,
                "snapshot_index": 2,
                "snapshot_term": before.term + 1,
                "data": [8, 8, 8]
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["accepted"], false);
    assert_eq!(state_machine.applied.load(Ordering::Acquire), 1);
    assert_eq!(state_machine.restore_attempts.load(Ordering::Acquire), 1);
    assert_eq!(host.store().load().unwrap().unwrap(), before);
    server.abort();
}

#[tokio::test]
async fn coordinated_compaction_reaches_every_voter_and_keeps_the_suffix() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=10 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < 10 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    assert_eq!(
        nodes[leader]
            .host
            .snapshot_and_compact_through(6)
            .await
            .unwrap(),
        6
    );

    let client = transport_h2c::h2c_client_with(None, None).unwrap();
    for node in &nodes {
        let status: RaftStatus = client
            .get(format!("{}/raftz", node.url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(status.snapshot_index, 6);
        assert_eq!(status.last_index, 10);
    }
}

#[tokio::test]
async fn coordinated_compaction_reports_an_already_installed_prefix_as_a_noop() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=4 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < 4 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    let installed = nodes[leader]
        .host
        .snapshot_and_compact_through_outcome(4)
        .await
        .unwrap();
    assert!(installed.installed);
    assert_eq!(installed.snapshot_index, 4);

    let noop = nodes[leader]
        .host
        .snapshot_and_compact_through_outcome(4)
        .await
        .unwrap();
    assert!(!noop.installed);
    assert_eq!(noop.snapshot_index, 4);
}

#[tokio::test]
async fn quorum_compaction_bounds_the_log_while_one_voter_is_offline() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    let offline = (0..nodes.len()).find(|index| *index != leader).unwrap();
    nodes[offline]
        .sm
        .snapshot_capable
        .store(false, Ordering::Release);
    for value in 1_u8..=8 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    let live = (0..nodes.len())
        .find(|index| *index != leader && *index != offline)
        .unwrap();
    while nodes[live].sm.applied.load(Ordering::Acquire) < 8 {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let all_voter_error = nodes[leader]
        .host
        .snapshot_and_compact_through_outcome(8)
        .await
        .expect_err("an offline voter must block the all-voter barrier");
    assert!(all_voter_error.to_string().contains("voter"));

    let compacted = nodes[leader]
        .host
        .snapshot_and_compact_through_quorum_outcome(8)
        .await
        .unwrap();
    assert!(compacted.installed);
    assert_eq!(compacted.snapshot_index, 8);
    let persisted = nodes[leader].host.store().load().unwrap().unwrap();
    assert_eq!(persisted.snapshot_index, 8);
    assert!(persisted.log.is_empty());
}

#[tokio::test]
async fn quorum_checkpoint_does_not_wait_for_a_blackholed_voter() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=8 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < 8 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    let blackholed = (0..nodes.len()).find(|index| *index != leader).unwrap();
    let (listener, blackhole_url) = bind().await;
    let blackhole = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let _stream = stream;
                std::future::pending::<()>().await;
            });
        }
    });
    nodes[leader]
        .host
        .upsert_peer(blackholed as u64, blackhole_url)
        .await;

    let compacted = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        nodes[leader]
            .host
            .snapshot_and_compact_through_quorum_outcome(8),
    )
    .await
    .expect("a live quorum must not wait for the snapshot RPC timeout")
    .unwrap();
    assert!(compacted.installed);
    assert_eq!(compacted.snapshot_index, 8);
    blackhole.abort();
}

#[tokio::test]
async fn all_voter_applied_barrier_refuses_a_lagging_replica() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    let index = nodes[leader].host.propose(vec![1]).await.unwrap();
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < index {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    nodes[leader]
        .host
        .require_applied_index_on_all_voters(index)
        .await
        .unwrap();

    let lagging = (0..nodes.len())
        .find(|candidate| *candidate != leader)
        .unwrap();
    nodes[lagging].sm.applied.store(0, Ordering::Release);
    let error = nodes[leader]
        .host
        .require_applied_index_on_all_voters(index)
        .await
        .expect_err("a lagging voter must block a destructive product transition");
    assert!(error.to_string().contains("applied index 0"));
}

#[tokio::test]
async fn coordinated_compaction_waits_for_every_voter_snapshot_capability() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=6 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < 6 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    let old_voter = (0..nodes.len()).find(|index| *index != leader).unwrap();
    nodes[old_voter]
        .sm
        .snapshot_capable
        .store(false, Ordering::Release);

    let preflight_error = nodes[leader]
        .host
        .require_snapshot_capability_on_all_voters()
        .await
        .expect_err("mixed snapshot decoders must block mutations before snapshot");
    assert!(preflight_error.to_string().contains("snapshot capability"));

    let error = nodes[leader]
        .host
        .snapshot_and_compact_through(4)
        .await
        .expect_err("mixed snapshot decoders must block coordinated compaction");
    assert!(error.to_string().contains("snapshot capability"));

    let client = transport_h2c::h2c_client_with(None, None).unwrap();
    for node in &nodes {
        let status: RaftStatus = client
            .get(format!("{}/raftz", node.url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(status.snapshot_index, 0);
        assert_eq!(status.last_index, 6);
    }
}

#[tokio::test]
async fn capability_disappearance_between_probe_and_install_fails_before_restore() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=6 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    for node in &nodes {
        while node.sm.applied.load(Ordering::Acquire) < 6 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    let changing_voter = (0..nodes.len()).find(|index| *index != leader).unwrap();
    nodes[changing_voter]
        .sm
        .snapshot_capability_calls
        .store(0, Ordering::Release);
    nodes[changing_voter]
        .sm
        .drop_capability_after_first_probe
        .store(true, Ordering::Release);

    let error = nodes[leader]
        .host
        .snapshot_and_compact_through(4)
        .await
        .expect_err("capability must still be present on the install request");
    assert!(error.to_string().contains("capable snapshot"));
    assert_eq!(
        nodes[changing_voter]
            .sm
            .restore_attempts
            .load(Ordering::Acquire),
        0,
        "a changed voter must reject the checkpoint before restore"
    );

    let client = transport_h2c::h2c_client_with(None, None).unwrap();
    for (index, node) in nodes.iter().enumerate() {
        let status: RaftStatus = client
            .get(format!("{}/raftz", node.url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if index == leader || index == changing_voter {
            assert_eq!(
                status.snapshot_index, 0,
                "the leader must not compact and the changed voter must not restore"
            );
        } else {
            assert!(
                status.snapshot_index <= 4,
                "a concurrently installed peer snapshot must not exceed the requested prefix"
            );
        }
        assert_eq!(status.last_index, 6);
    }
}

#[tokio::test]
async fn higher_term_snapshot_refusal_steps_the_old_leader_down_immediately() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    for value in 1_u8..=4 {
        nodes[leader].host.propose(vec![value]).await.unwrap();
    }
    let newer_voter = (0..nodes.len()).find(|index| *index != leader).unwrap();
    let client = transport_h2c::h2c_client_with(None, None).unwrap();
    client
        .post(format!("{}/raft/request-vote", nodes[newer_voter].url))
        .json(&serde_json::json!({
            "group_id": LEGACY_GROUP_ID,
            "from": newer_voter as u64,
            "req": {
                "term": 100,
                "candidate": newer_voter as u64,
                "last_log_index": 4,
                "last_log_term": 1
            }
        }))
        .send()
        .await
        .unwrap();

    let error = nodes[leader]
        .host
        .snapshot_and_compact_through_outcome(4)
        .await
        .expect_err("a higher-term voter must refuse the old leader snapshot");
    assert!(error.to_string().contains("advanced the term"));
    let old_leader: RaftStatus = client
        .get(format!("{}/raftz", nodes[leader].url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!old_leader.is_leader);
    assert!(old_leader.term >= 100);
}

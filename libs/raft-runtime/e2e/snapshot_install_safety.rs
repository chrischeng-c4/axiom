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

/// Shutdown must account for a snapshot request started by the public
/// coordinator. Holding only its reply leaves Raft heartbeats and leadership
/// handoff available, so an unfinished drain cannot hide behind another phase.
#[tokio::test]
async fn shutdown_waits_for_coordinated_snapshot_reply_before_safe_peer_close() {
    use axum::{extract::Request, middleware::Next};
    use raft_runtime::{LeadershipHandoff, ShutdownPhase};
    use server_lifecycle::ShutdownDeadline;
    use std::time::Duration;
    use tokio::{sync::watch, task::JoinSet};

    let mut nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");
    let applied = nodes[leader].host.propose(vec![1]).await.unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        for node in &nodes {
            while node.sm.applied.load(Ordering::Acquire) < applied {
                tokio::task::yield_now().await;
            }
        }
    })
    .await
    .expect("both voters apply the proposal before snapshot and shutdown");

    let held_voter = (0..nodes.len()).find(|index| *index != leader).unwrap();
    let other_voter = (0..nodes.len())
        .find(|index| *index != leader && *index != held_voter)
        .unwrap();
    let (entered_tx, mut entered_rx) = watch::channel(None);
    let (release_tx, release_rx) = watch::channel(false);
    let router = nodes[held_voter]
        .host
        .router()
        .layer(axum::middleware::from_fn(
            move |request: Request, next: Next| {
                let entered_tx = entered_tx.clone();
                let mut release_rx = release_rx.clone();
                async move {
                    let hold_reply = request.uri().path() == "/raft/install-snapshot-capable";
                    let response = next.run(request).await;
                    if hold_reply {
                        entered_tx.send_replace(Some(response.status()));
                        while !*release_rx.borrow() {
                            if release_rx.changed().await.is_err() {
                                break;
                            }
                        }
                    }
                    response
                }
            },
        ));
    let (listener, held_reply_url) = bind().await;
    // Dropping either JoinSet cancels its tasks if a setup assertion fails.
    // Dropping release_tx also opens the reply gate by closing its watch.
    let mut servers = JoinSet::new();
    servers.spawn(async move {
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { break };
                    let router = router.clone();
                    connections.spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, router).await;
                    });
                }
                Some(_) = connections.join_next(), if !connections.is_empty() => {}
            }
        }
    });
    nodes[leader]
        .host
        .upsert_peer(held_voter as u64, held_reply_url)
        .await;

    let mut snapshots = JoinSet::new();
    let host = Arc::clone(&nodes[leader].host);
    snapshots.spawn(async move { host.snapshot_and_compact_through_outcome(applied).await });
    tokio::time::timeout(Duration::from_secs(3), async {
        while entered_rx.borrow().is_none() {
            entered_rx
                .changed()
                .await
                .expect("the reply gate remains alive");
        }
    })
    .await
    .expect("the coordinator reaches the actual voter snapshot handler");
    assert_eq!(
        *entered_rx.borrow(),
        Some(axum::http::StatusCode::OK),
        "the gate holds a successful snapshot reply"
    );
    assert_eq!(nodes[held_voter].host.snapshot_index().await, applied);
    assert!(
        snapshots.try_join_next().is_none(),
        "the held reply keeps the coordinator pending"
    );
    assert_eq!(
        nodes[other_voter].sm.applied.load(Ordering::Acquire),
        applied
    );
    tokio::time::timeout(Duration::from_secs(3), async {
        while nodes[other_voter].host.snapshot_index().await < applied {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the other voter accepts its snapshot before leadership handoff");

    let shutdown = tokio::time::timeout(
        Duration::from_secs(3),
        nodes[leader].host.shutdown_within(
            ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap(),
        ),
    )
    .await;
    let snapshot_finished_at_shutdown = snapshots.try_join_next().is_some();

    // Complete or cancel every task before asserting the shutdown result.
    release_tx.send_replace(true);
    if !snapshot_finished_at_shutdown {
        let _ = tokio::time::timeout(Duration::from_secs(2), snapshots.join_next()).await;
    }
    snapshots.abort_all();
    while snapshots.join_next().await.is_some() {}
    nodes[leader]
        .host
        .upsert_peer(held_voter as u64, nodes[held_voter].url.clone())
        .await;
    servers.abort_all();
    while servers.join_next().await.is_some() {}
    let mut shutdowns = JoinSet::new();
    for node in &nodes {
        let host = Arc::clone(&node.host);
        shutdowns.spawn(async move {
            let _ = tokio::time::timeout(Duration::from_secs(3), host.shutdown()).await;
        });
    }
    while shutdowns.join_next().await.is_some() {}
    for node in &mut nodes {
        node._serve.abort();
        let _ = (&mut node._serve).await;
    }

    let report = shutdown.expect("shutdown returns within its bounded drain deadline");
    assert!(
        matches!(report.handoff, LeadershipHandoff::Transferred { .. }),
        "another caught-up voter remains available for leadership handoff: {report:?}"
    );
    assert!(
        report.incomplete_phase.is_none()
            || report.incomplete_phase == Some(ShutdownPhase::PeerRpcDrain),
        "shutdown must reach the peer drain phase: {report:?}"
    );
    assert!(
        snapshot_finished_at_shutdown || !report.peer_listener_close_safe,
        "shutdown reported peer_listener_close_safe=true while a coordinated snapshot reply was held and its caller remained pending: {report:?}"
    );
}

#[tokio::test]
async fn coordinated_snapshot_work_stays_open_for_final_flush_but_closes_at_shutdown() {
    use server_lifecycle::ShutdownDeadline;
    use std::time::Duration;

    let mut nodes = cluster(1).await;
    let host = Arc::clone(&nodes[0].host);
    let applied = host.propose(vec![1]).await.unwrap();
    host.quiesce_proposals();

    // A product can stop writes before it archives and compacts its final batch.
    host.require_snapshot_capability_on_all_voters()
        .await
        .unwrap();
    host.require_applied_index_on_all_voters(applied)
        .await
        .unwrap();
    assert_eq!(host.snapshot_and_compact().await.unwrap(), applied);

    let report = host
        .shutdown_within(
            ShutdownDeadline::from_now(Duration::from_secs(2), Duration::ZERO).unwrap(),
        )
        .await;
    nodes[0]._serve.abort();
    let _ = (&mut nodes[0]._serve).await;
    assert!(report.peer_listener_close_safe, "{report:?}");

    let outcomes = [
        host.require_snapshot_capability_on_all_voters().await,
        host.require_applied_index_on_all_voters(applied).await,
        host.snapshot_and_compact().await.map(|_| ()),
        host.snapshot_and_compact_through_quorum_outcome(applied)
            .await
            .map(|_| ()),
    ];
    for outcome in outcomes {
        assert!(
            outcome.is_err(),
            "coordinated snapshot work must be refused after safe peer close"
        );
    }
    assert_eq!(host.snapshot_index().await, applied);
}

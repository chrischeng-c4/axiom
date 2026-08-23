//! Handing leadership to an eligible caught-up voter before shutdown stops the host (#3664).

use std::sync::Arc;
use std::time::{Duration, Instant};

use tempfile::TempDir;

use raft_runtime::{
    FsyncPolicy, HostConfig, LeadershipHandoff, Membership, RaftHost, RaftStateMachine,
    RaftStatus, RaftStore, SnapshotPolicy,
};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster, peers_excluding, Node, TestSm};

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

async fn status(client: &reqwest::Client, url: &str) -> RaftStatus {
    client
        .get(format!("{url}/raftz"))
        .send()
        .await
        .expect("a host serves its own status")
        .json()
        .await
        .expect("the status is the published shape")
}

/// Settle all followers so that their logs match the leader's and acknowledgements
/// have been received by the leader before measuring handoff timing.
async fn settle_cluster(nodes: &[Node], leader: usize) {
    let client = h2c_client();
    for i in 0..5u8 {
        nodes[leader]
            .host
            .propose(vec![i])
            .await
            .expect("the leader accepts a proposal");
    }

    let leader_last = status(&client, &nodes[leader].url).await.last_index;
    assert!(leader_last >= 5);

    let caught_up = Instant::now() + Duration::from_secs(10);
    for (i, node) in nodes.iter().enumerate() {
        if i == leader {
            continue;
        }
        loop {
            let s = status(&client, &node.url).await;
            if s.last_index == leader_last && s.commit_index == leader_last {
                break;
            }
            assert!(
                Instant::now() < caught_up,
                "follower {i} never caught up with leader"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    // Allow heartbeats / peer acks to settle leader's match_index.
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// `handoff_leadership` on a three-voter leader reports `Transferred { target }`
/// with that named target holding the group afterwards.
#[tokio::test]
async fn a_three_voter_leader_hands_off_leadership_to_an_eligible_voter() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = Instant::now() + Duration::from_secs(5);
    let outcome = loop {
        let outcome = nodes[leader].host.handoff_leadership().await;
        match outcome {
            LeadershipHandoff::NoCaughtUpVoter { .. } => {
                assert!(
                    Instant::now() < deadline,
                    "the leader never recorded any peer as caught up"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            other => break other,
        }
    };

    let target = match outcome {
        LeadershipHandoff::Transferred { target } => target,
        other => panic!("expected LeadershipHandoff::Transferred, got {other:?}"),
    };
    assert_ne!(
        target, leader as u64,
        "the transferred target must not be the leader itself"
    );
    assert!(
        target < 3,
        "the transferred target must be one of the cluster voters"
    );

    // The host that hands off stays leader and keeps heartbeating, so followers'
    // election timers never expire. The target's leadership can only have been
    // delivered by TimeoutNow. The wait is a generous liveness bound.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if nodes[target as usize].host.is_leader().await {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "leadership never arrived at the named target {target}"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert_eq!(
        nodes[target as usize].host.leader().await,
        Some(target),
        "target node must report itself as leader"
    );
    assert!(
        !nodes[leader].host.is_leader().await,
        "original leader must no longer be leader"
    );
}

/// `shutdown()` alone moves leadership to another live node well inside the
/// election floor without calling `handoff_leadership` directly.
#[tokio::test]
async fn shutdown_alone_moves_leadership_to_another_live_node_within_delivery_budget() {
    // We configure an explicit HostConfig so the spontaneous election floor
    // (in tick units) and the handoff budget stretch together under load.
    let cfg = HostConfig {
        tick: Duration::from_millis(60),
        pump: Duration::from_millis(5),
        rpc_timeout: Duration::from_millis(200),
        propose_timeout: Duration::from_secs(10),
        snapshot: SnapshotPolicy::Disabled,
    };

    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..3u64 {
        let (l, url) = bind().await;
        listeners.push(l);
        all.push((id, url));
    }
    let voters: Vec<u64> = (0..3).collect();
    let mut nodes = Vec::new();
    for (idx, listener) in listeners.into_iter().enumerate() {
        let id = idx as u64;
        let peers = peers_excluding(id, &all);
        let sm = TestSm::new();
        let dir = TempDir::new().unwrap();
        let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
        let host = Arc::new(RaftHost::spawn(
            id,
            Membership {
                voters: voters.clone(),
                learners: vec![],
            },
            peers,
            store,
            sm.clone() as Arc<dyn RaftStateMachine>,
            cfg,
        ));
        let router = host.router();
        let serve = tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let r = router.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        });
        let url = all[idx].1.clone();
        nodes.push(Node {
            host,
            sm,
            url,
            _serve: serve,
            _dir: dir,
        });
    }

    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let live_peers: Vec<usize> = (0..3).filter(|&i| i != leader).collect();

    let start = Instant::now();
    nodes[leader]
        .host
        .shutdown()
        .await
        .expect("shutdown must return Ok");

    // In raft-core:
    // - ELECTION_MIN = 50 ticks (the minimum election timeout floor for any node).
    // - HEARTBEAT_TIMEOUT = 3 ticks (leader heartbeats every 3 ticks).
    // When the leader stops heartbeating, followers' election timers were reset at most
    // HEARTBEAT_TIMEOUT ticks ago. Therefore, the earliest any survivor follower can time out
    // and start a spontaneous re-election is (ELECTION_MIN - HEARTBEAT_TIMEOUT) = 47 ticks.
    //
    // A delivered handoff (TimeoutNow) is sent during shutdown() Phase 2 and processed
    // within a few HostConfig::pump intervals (5ms). shutdown() itself has a deadline
    // of 2 * HostConfig::rpc_timeout (400ms).
    //
    // Measuring arrival against a tick-derived budget (35 ticks) strictly below the
    // 47-tick election floor ensures that:
    // 1. A delivered handoff is accepted even with observation/load delay (Probe A).
    // 2. An undelivered handoff (Probe B) is rejected because survivors will not re-elect
    //    until after the 47-tick floor.
    const ELECTION_MIN_TICKS: u64 = 50;
    const HEARTBEAT_TIMEOUT_TICKS: u64 = 3;
    let election_floor_ticks = ELECTION_MIN_TICKS - HEARTBEAT_TIMEOUT_TICKS;
    let handoff_budget_ticks = 35u64;
    assert!(handoff_budget_ticks < election_floor_ticks);
    let handoff_budget = cfg.tick * handoff_budget_ticks as u32;

    let mut arrived = None;
    let mut new_leader = None;
    while start.elapsed() < handoff_budget {
        for &p in &live_peers {
            if nodes[p].host.is_leader().await {
                arrived = Some(start.elapsed());
                new_leader = Some(p);
                break;
            }
        }
        if arrived.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let _elapsed = arrived.expect("leadership must move to a live peer within handoff budget");
    let new_leader_id = new_leader.expect("a live peer became leader") as u64;
    assert_eq!(
        nodes[new_leader.unwrap()].host.leader().await,
        Some(new_leader_id),
        "new leader must report itself as leader"
    );
}

/// A single-voter host reports `SoleVoter` and `shutdown()` returns `Ok`.
#[tokio::test]
async fn single_voter_host_reports_sole_voter_and_shutdown_succeeds() {
    let nodes = cluster(1).await;
    let leader = await_leader(&nodes)
        .await
        .expect("single node elects leader");
    assert_eq!(leader, 0);

    let outcome = nodes[0].host.handoff_leadership().await;
    assert_eq!(
        outcome,
        LeadershipHandoff::SoleVoter,
        "single voter host must report SoleVoter"
    );

    nodes[0]
        .host
        .shutdown()
        .await
        .expect("shutdown must return Ok for single voter host");
}

/// A follower reports `NotLeader` and `shutdown()` returns `Ok`.
#[tokio::test]
async fn follower_reports_not_leader_and_shutdown_succeeds() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;

    let outcome = nodes[follower].host.handoff_leadership().await;
    assert_eq!(
        outcome,
        LeadershipHandoff::NotLeader,
        "follower must report NotLeader"
    );

    nodes[follower]
        .host
        .shutdown()
        .await
        .expect("shutdown must return Ok for follower");
}

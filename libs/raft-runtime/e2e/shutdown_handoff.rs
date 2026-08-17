//! Handing leadership to an eligible caught-up voter before shutdown stops the host (#3664).

use std::time::{Duration, Instant};

use raft_runtime::{LeadershipHandoff, RaftStatus};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, cluster, Node};

/// The shortest election timeout any node in these rows can have: `ELECTION_MIN` (50)
/// ticks of `HostConfig::default().tick` (20ms).
const ELECTION_TIMEOUT_FLOOR: Duration = Duration::from_millis(1000);

/// How long the handoff row waits for leadership to arrive. Comfortably above
/// what a loopback handoff costs — one 5ms pump and two h2c round trips — and
/// still below the floor above, so an arrival inside it is not a group that
/// re-elected.
const DELIVERY_BUDGET: Duration = Duration::from_millis(800);

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

    let start = Instant::now();
    let mut arrived = None;
    while start.elapsed() < DELIVERY_BUDGET {
        if nodes[target as usize].host.is_leader().await {
            arrived = Some(start.elapsed());
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let elapsed = arrived.expect("leadership must arrive at the named target within DELIVERY_BUDGET");
    assert!(
        elapsed < ELECTION_TIMEOUT_FLOOR,
        "leadership arrival took {elapsed:?}, which must be strictly below {ELECTION_TIMEOUT_FLOOR:?}"
    );
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
    let nodes = cluster(3).await;
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

    let mut arrived = None;
    let mut new_leader = None;
    while start.elapsed() < DELIVERY_BUDGET {
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
    let elapsed = arrived.expect("leadership must move to a live peer within DELIVERY_BUDGET");
    assert!(
        elapsed < ELECTION_TIMEOUT_FLOOR,
        "leadership moved in {elapsed:?}, which must be strictly below {ELECTION_TIMEOUT_FLOOR:?}"
    );
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

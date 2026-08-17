//! Member admission reachable on a running host and delivered to peers (#3650).
//!
//! # What was missing
//!
//! When a host is spawned, its peer address map and RPC lanes were fixed.
//! Adding a learner that did not exist at spawn failed to deliver any RPCs
//! because the leader had no address or lane for the new node.
//!
//! These rows test:
//! 1. Admitting a new member whose address is provided after spawn, observing
//!    replication and configuration adoption on the new member's own /raftz.
//! 2. Refusing admission with `AdmissionRefused::Unroutable` when no address
//!    has been registered for the target, leaving the group's configuration
//!    unchanged.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::{
    AdmissionRefused, FsyncPolicy, HostConfig, Membership, MembershipPhase, RaftHost,
    RaftStateMachine, RaftStatus, RaftStore,
};
use tempfile::TempDir;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster, Node, TestSm};

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

async fn poll_status_until<F>(
    client: &reqwest::Client,
    url: &str,
    predicate: F,
    timeout: Duration,
    desc: &str,
) -> RaftStatus
where
    F: Fn(&RaftStatus) -> bool,
{
    let deadline = Instant::now() + timeout;
    loop {
        let s = status(client, url).await;
        if predicate(&s) {
            return s;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for condition: {desc} at {url}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn spawn_standalone_node(id: u64) -> Node {
    let (listener, url) = bind().await;
    let sm = TestSm::new();
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
    let host = Arc::new(RaftHost::spawn(
        id,
        Membership {
            voters: vec![],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
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
    Node {
        host,
        sm,
        url,
        _serve: serve,
        _dir: dir,
    }
}

/// Three voters elect a leader; a fourth node is brought up that no member has an
/// address for. Every member is told the fourth node's address via `upsert_peer`.
/// The leader admits it as a learner, and the fourth node's own `/raftz` reports
/// itself as a learner.
#[tokio::test]
async fn a_running_host_admits_a_new_learner_observed_on_the_new_node_itself() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();

    let new_node_id = 3u64;
    let new_node = spawn_standalone_node(new_node_id).await;

    // Node 3's initial status has empty membership.
    let initial_status = status(&client, &new_node.url).await;
    assert!(initial_status.learners.is_empty());
    assert!(initial_status.committed_voters.is_empty());

    // Register node 3's address on all existing cluster members.
    for n in &nodes {
        n.host.upsert_peer(new_node_id, new_node.url.clone()).await;
    }

    // Admit node 3 as a learner on the leader.
    let idx = nodes[leader]
        .host
        .add_learner(new_node_id)
        .await
        .expect("admitting a routable learner succeeds");
    assert!(idx > 0);

    // Observe on the NEW NODE's own /raftz that it appears as a learner.
    let new_node_status = poll_status_until(
        &client,
        &new_node.url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.learners.contains(&new_node_id)
        },
        Duration::from_secs(5),
        "newly admitted node reports itself as learner in its own /raftz",
    )
    .await;

    assert_eq!(new_node_status.membership_phase, MembershipPhase::Stable);
    assert_eq!(new_node_status.role, "Learner");
    assert_eq!(new_node_status.learners, vec![new_node_id]);
    assert_eq!(new_node_status.committed_voters, vec![0, 1, 2]);

    // Also verify a bystander follower's /raftz reflects the new learner.
    let bystander = (leader + 1) % 3;
    let bystander_status = poll_status_until(
        &client,
        &nodes[bystander].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.learners.contains(&new_node_id)
        },
        Duration::from_secs(5),
        "bystander node reports new node as learner",
    )
    .await;

    assert_eq!(bystander_status.learners, vec![new_node_id]);
}

/// Admitting an unroutable node (no address ever supplied via `upsert_peer`)
/// returns `AdmissionRefused::Unroutable` and leaves the group's configuration
/// completely unchanged.
#[tokio::test]
async fn admitting_an_unaddressed_node_is_refused_as_unroutable_without_proposing_config() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let bystander = (leader + 1) % 3;
    let client = h2c_client();

    let initial_status = status(&client, &nodes[bystander].url).await;
    assert_eq!(initial_status.committed_voters, vec![0, 1, 2]);
    assert!(initial_status.learners.is_empty());

    let unaddressed_id = 999u64;

    // Call add_learner on the leader for an unaddressed node id.
    match nodes[leader].host.add_learner(unaddressed_id).await {
        Err(AdmissionRefused::Unroutable { target }) => {
            assert_eq!(target, unaddressed_id);
        }
        other => panic!("expected Err(AdmissionRefused::Unroutable), got {other:?}"),
    }

    // Propose a barrier command and wait for the bystander to apply it, ensuring
    // any configuration entry proposed before it has settled.
    let barrier_index = nodes[leader]
        .host
        .propose(b"command-after-refusal".to_vec())
        .await
        .expect("propose works after refusal");

    let post_status = poll_status_until(
        &client,
        &nodes[bystander].url,
        |s| s.applied_index >= barrier_index,
        Duration::from_secs(5),
        "bystander node applies barrier entry",
    )
    .await;
    assert_eq!(post_status.committed_voters, vec![0, 1, 2]);
    assert!(
        post_status.learners.is_empty(),
        "unroutable admission must not propose or commit any learner"
    );
}

/// Calling `add_learner` on a follower returns `NotLeaderOrTransferInFlight`.
#[tokio::test]
async fn a_host_that_is_not_the_leader_refuses_admission() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;

    nodes[follower]
        .host
        .upsert_peer(42, "http://127.0.0.1:1234".to_string())
        .await;

    match nodes[follower].host.add_learner(42).await {
        Err(AdmissionRefused::NotLeaderOrTransferInFlight) => {}
        other => {
            panic!("expected Err(AdmissionRefused::NotLeaderOrTransferInFlight), got {other:?}")
        }
    }
}

/// `forget_peer` removes an address so subsequent admission is refused as unroutable.
#[tokio::test]
async fn forget_peer_removes_address_causing_subsequent_admission_to_be_refused() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");

    nodes[leader]
        .host
        .upsert_peer(55, "http://127.0.0.1:5555".to_string())
        .await;

    nodes[leader].host.forget_peer(55).await;

    match nodes[leader].host.add_learner(55).await {
        Err(AdmissionRefused::Unroutable { target }) => {
            assert_eq!(target, 55);
        }
        other => panic!("expected Err(AdmissionRefused::Unroutable), got {other:?}"),
    }
}

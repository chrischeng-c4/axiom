//! Idempotent host shutdown and single-flight execution (#3683).

use std::time::{Duration, Instant};

use raft_runtime::{LeadershipHandoff, RaftStatus, ShutdownCaller};
use server_lifecycle::ShutdownDeadline;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, cluster, Node};

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
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// A repeat `shutdown_within` on an already shut-down host emits no new messages
/// on the wire and returns the terminal report with `ShutdownCaller::Joined`.
#[tokio::test]
async fn repeat_shutdown_after_completion_emits_no_new_messages_and_returns_joined_report() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let r1 = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        r1.caller,
        ShutdownCaller::Executed,
        "first shutdown caller must execute the phases"
    );
    assert_eq!(r1.phases.len(), 4);
    assert!(
        r1.peer_listener_close_safe,
        "first report must indicate peer listener close safe"
    );

    let target = match r1.handoff {
        LeadershipHandoff::Transferred { target } => target,
        ref other => panic!("expected LeadershipHandoff::Transferred, got {other:?}"),
    };

    let client = h2c_client();
    let before = status(&client, &nodes[leader].url)
        .await
        .undeliverable_never_addressed;

    // Withdraw the target peer address after the first shutdown completed,
    // so background loops are already dead and only a repeat attempt could emit.
    nodes[leader].host.forget_peer(target).await;

    let r2 = nodes[leader].host.shutdown_within(deadline).await;

    let after = status(&client, &nodes[leader].url)
        .await
        .undeliverable_never_addressed;
    assert_eq!(
        after - before,
        0,
        "repeat shutdown must not emit any new messages"
    );

    assert_eq!(
        r2.caller,
        ShutdownCaller::Joined,
        "repeat caller must be reported as Joined"
    );
    assert_eq!(
        r2.phases, r1.phases,
        "repeat caller must observe identical phases"
    );
    assert_eq!(
        r2.handoff, r1.handoff,
        "repeat caller must observe identical handoff"
    );
    assert_eq!(
        r2.incomplete_phase, r1.incomplete_phase,
        "repeat caller must observe identical incomplete_phase"
    );
    assert_eq!(
        r2.peer_listener_close_safe, r1.peer_listener_close_safe,
        "repeat caller must observe identical peer_listener_close_safe"
    );
    assert_eq!(
        r2.storage_failure, r1.storage_failure,
        "repeat caller must observe identical storage_failure"
    );
    assert!(
        r2.peer_listener_close_safe,
        "joined caller on completed shutdown must report peer_listener_close_safe true"
    );
}

/// Two callers racing `shutdown_within` from scratch execute the sequence
/// exactly once, agree on identical phases, and divide into Executed and Joined.
#[tokio::test]
async fn concurrent_shutdown_from_scratch_executes_once_and_returns_identical_phases() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let (r_a, r_b) = tokio::join!(
        nodes[leader].host.shutdown_within(deadline),
        nodes[leader].host.shutdown_within(deadline),
    );

    assert_eq!(
        r_a.phases, r_b.phases,
        "concurrent callers must observe identical phases"
    );
    assert_eq!(
        r_a.handoff, r_b.handoff,
        "concurrent callers must observe identical handoff"
    );
    assert_eq!(
        r_a.incomplete_phase, r_b.incomplete_phase,
        "concurrent callers must observe identical incomplete_phase"
    );
    assert_eq!(
        r_a.peer_listener_close_safe, r_b.peer_listener_close_safe,
        "concurrent callers must observe identical peer_listener_close_safe"
    );
    assert_eq!(
        r_a.storage_failure, r_b.storage_failure,
        "concurrent callers must observe identical storage_failure"
    );
    assert!(
        r_a.peer_listener_close_safe,
        "peer_listener_close_safe must be true for both callers"
    );
    assert!(
        r_b.peer_listener_close_safe,
        "peer_listener_close_safe must be true for both callers"
    );

    let roles = (r_a.caller, r_b.caller);
    assert!(
        roles == (ShutdownCaller::Executed, ShutdownCaller::Joined)
            || roles == (ShutdownCaller::Joined, ShutdownCaller::Executed),
        "exactly one caller must be Executed and the other Joined, got {roles:?}"
    );
}

/// Two concurrent callers calling `shutdown_within` on an already shut-down host
/// emit no new messages on the wire and both receive the joined report.
#[tokio::test]
async fn concurrent_repeat_shutdown_after_completion_emits_no_new_messages() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let r1 = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        r1.caller,
        ShutdownCaller::Executed,
        "initial caller must execute shutdown"
    );
    assert!(
        r1.peer_listener_close_safe,
        "initial shutdown report must indicate peer listener close safe"
    );

    let target = match r1.handoff {
        LeadershipHandoff::Transferred { target } => target,
        ref other => panic!("expected LeadershipHandoff::Transferred, got {other:?}"),
    };

    let client = h2c_client();
    let before = status(&client, &nodes[leader].url)
        .await
        .undeliverable_never_addressed;

    // Withdraw the target peer address after the first shutdown completed.
    nodes[leader].host.forget_peer(target).await;

    let (r2, r3) = tokio::join!(
        nodes[leader].host.shutdown_within(deadline),
        nodes[leader].host.shutdown_within(deadline),
    );

    let after = status(&client, &nodes[leader].url)
        .await
        .undeliverable_never_addressed;
    assert_eq!(
        after - before,
        0,
        "concurrent repeat shutdown calls must not emit any new messages"
    );

    assert_eq!(
        r2.caller,
        ShutdownCaller::Joined,
        "first concurrent repeat caller must be Joined"
    );
    assert_eq!(
        r3.caller,
        ShutdownCaller::Joined,
        "second concurrent repeat caller must be Joined"
    );
    assert_eq!(
        r2.phases, r1.phases,
        "first repeat caller must observe identical phases"
    );
    assert_eq!(
        r3.phases, r1.phases,
        "second repeat caller must observe identical phases"
    );
    assert_eq!(
        r2.handoff, r1.handoff,
        "first repeat caller must observe identical handoff"
    );
    assert_eq!(
        r3.handoff, r1.handoff,
        "second repeat caller must observe identical handoff"
    );
    assert_eq!(
        r2.incomplete_phase, r1.incomplete_phase,
        "first repeat caller must observe identical incomplete_phase"
    );
    assert_eq!(
        r3.incomplete_phase, r1.incomplete_phase,
        "second repeat caller must observe identical incomplete_phase"
    );
    assert_eq!(
        r2.peer_listener_close_safe, r1.peer_listener_close_safe,
        "first repeat caller must observe identical peer_listener_close_safe"
    );
    assert_eq!(
        r3.peer_listener_close_safe, r1.peer_listener_close_safe,
        "second repeat caller must observe identical peer_listener_close_safe"
    );
    assert_eq!(
        r2.storage_failure, r1.storage_failure,
        "first repeat caller must observe identical storage_failure"
    );
    assert_eq!(
        r3.storage_failure, r1.storage_failure,
        "second repeat caller must observe identical storage_failure"
    );
    assert!(
        r2.peer_listener_close_safe,
        "peer_listener_close_safe must be true for repeat caller"
    );
    assert!(
        r3.peer_listener_close_safe,
        "peer_listener_close_safe must be true for repeat caller"
    );
}

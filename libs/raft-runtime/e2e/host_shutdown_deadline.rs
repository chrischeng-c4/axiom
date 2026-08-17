//! Bounding RaftHost shutdown by caller-supplied ShutdownDeadline and reporting
//! terminal phase outcomes (#3672).

use std::io::ErrorKind;
use std::time::{Duration, Instant};

use raft_runtime::{
    LeadershipHandoff, PhaseStatus, ProposalOutcome, RaftStatus, ShutdownPhase,
};
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

/// A three-voter cluster's leader shut down under a generous deadline records
/// all four phases `Completed` in the fixed order with `peer_listener_close_safe`
/// true and `incomplete_phase` none.
#[tokio::test]
async fn three_voter_leader_generous_deadline_completes_all_phases_in_order() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let report = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        report.phases.len(),
        4,
        "all four phases must be recorded in order"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "first phase must be Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::Completed,
        "Quiesce must complete"
    );
    assert_eq!(
        report.phases[1].phase,
        ShutdownPhase::LeadershipHandoff,
        "second phase must be LeadershipHandoff"
    );
    assert_eq!(
        report.phases[1].status,
        PhaseStatus::Completed,
        "LeadershipHandoff must complete"
    );
    assert_eq!(
        report.phases[2].phase,
        ShutdownPhase::BackgroundTasks,
        "third phase must be BackgroundTasks"
    );
    assert_eq!(
        report.phases[2].status,
        PhaseStatus::Completed,
        "BackgroundTasks must complete"
    );
    assert_eq!(
        report.phases[3].phase,
        ShutdownPhase::PeerRpcDrain,
        "fourth phase must be PeerRpcDrain"
    );
    assert_eq!(
        report.phases[3].status,
        PhaseStatus::Completed,
        "PeerRpcDrain must complete"
    );

    assert_eq!(
        report.incomplete_phase, None,
        "incomplete_phase must be None on clean shutdown"
    );
    assert!(
        report.peer_listener_close_safe,
        "peer_listener_close_safe must be true when PeerRpcDrain completes"
    );
    assert!(
        report.storage_failure.is_none(),
        "storage_failure must be None on healthy host"
    );
    assert!(
        matches!(report.handoff, LeadershipHandoff::Transferred { .. }),
        "leadership handoff must transfer to a peer"
    );
}

/// A deadline whose total equals its reserve — so usable_remaining is zero —
/// stops at the first phase, names it as incomplete, and leaves peer_listener_close_safe false.
#[tokio::test]
async fn zero_usable_budget_stops_at_first_phase_and_names_it_incomplete() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(10), Duration::from_secs(10))
        .expect("valid deadline where total equals reserve");
    assert_eq!(
        deadline.usable_remaining(),
        Duration::ZERO,
        "usable_remaining must be zero"
    );

    let report = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        report.phases.len(),
        1,
        "exactly one phase record must be present on zero usable budget"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "phase must be Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::DeadlineExpired,
        "first phase status must be DeadlineExpired"
    );
    assert_eq!(
        report.incomplete_phase,
        Some(ShutdownPhase::Quiesce),
        "incomplete_phase must name ShutdownPhase::Quiesce"
    );
    assert!(
        !report.peer_listener_close_safe,
        "peer_listener_close_safe must be false"
    );
}

/// A host whose store has an injected save failure reports storage_failure carrying the
/// injected ErrorKind and PhaseStatus::StorageFailed, and legacy shutdown() on a separate
/// latched host returns Err naming the failure.
#[tokio::test]
async fn latched_storage_failure_reported_in_shutdown_within_and_legacy_shutdown_returns_err() {
    // Host 1: shutdown_within on latched host
    let nodes1 = cluster(1).await;
    nodes1[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    // Latch the failure via a proposal
    let outcome1 = nodes1[0]
        .host
        .propose_outcome(b"cmd-save-failure-1".to_vec())
        .await;
    assert!(
        matches!(outcome1, ProposalOutcome::DurabilityFailure { .. }),
        "proposal must fail and latch storage failure"
    );

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let report = nodes1[0].host.shutdown_within(deadline).await;

    assert!(
        report.storage_failure.is_some(),
        "report must carry storage failure"
    );
    let sf = report.storage_failure.unwrap();
    assert_eq!(
        sf.kind,
        ErrorKind::StorageFull,
        "storage failure kind must match injected ErrorKind"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "first phase is Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::StorageFailed,
        "phase where failure was observed must be StorageFailed"
    );

    // Host 2: legacy shutdown on a separate latched host
    let nodes2 = cluster(1).await;
    nodes2[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    let outcome2 = nodes2[0]
        .host
        .propose_outcome(b"cmd-save-failure-2".to_vec())
        .await;
    assert!(
        matches!(outcome2, ProposalOutcome::DurabilityFailure { .. }),
        "proposal must fail and latch storage failure on second host"
    );

    let res = nodes2[0].host.shutdown().await;
    assert!(
        res.is_err(),
        "legacy shutdown() must return Err on host with latched storage failure"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("durable storage failed"),
        "error message must name durability failure, got: {err_msg}"
    );
}

/// A healthy three-voter leader shut down through legacy shutdown() returns Ok(())
/// with the group's durable state intact afterwards.
#[tokio::test]
async fn healthy_three_voter_leader_legacy_shutdown_returns_ok_with_state_intact() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let res = nodes[leader].host.shutdown().await;
    assert!(
        res.is_ok(),
        "legacy shutdown on healthy leader must return Ok(())"
    );

    // Verify durable state intact
    assert!(
        nodes[leader].host.store().path().exists(),
        "store path must remain intact after shutdown"
    );
}

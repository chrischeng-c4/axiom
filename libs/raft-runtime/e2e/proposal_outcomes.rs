//! Proposal outcome typing and classification tests (#3663, #3665).
//!
//! # What was missing
//!
//! Every failure of `RaftHost::propose` was an `anyhow::Error` distinguished only
//! by its message string. Twelve terminal failure arms collapsed into it, erasing
//! the critical distinction between commands safe to re-route (never appended)
//! and commands with allocated indices that must not be blindly retried.
//!
//! These rows test:
//! 1. A single-voter proposal succeeds and returns `Completed` with the applied index.
//! 2. A quiesced host returns `RejectedBeforeAdmission` while `propose` returns an error
//!    containing `raft: proposal admission closed`.
//! 3. A host in a group with no leader elected returns `RejectedBeforeAdmission`.
//! 4. A leader with an injected save failure returns `DurabilityFailure` with `Some(index)`.
//! 5. A latched host with prior durability failure returns `DurabilityFailure` with `None`.
//! 6. A follower forwarding to a healthy leader returns `Completed` after its own apply.
//! 7. A follower forwarding to a quiesced leader returns `RejectedBeforeAdmission`.
//! 8. A forwarded request that times out returns an `Ambiguous` routing-timeout outcome.
//! 9. Negative control: sequential proposals on a healthy host strictly increase indices.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use axum::{http::StatusCode, routing::post, Json, Router};
use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, ProposalOutcome, RaftHost, RaftStateMachine, RaftStore,
};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, await_leader_with_tick, bind, cluster, peers_excluding, Node, TestSm};

async fn custom_timeout_cluster(n: u64, timeout: Duration) -> (Vec<Node>, Duration) {
    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..n {
        let (l, url) = bind().await;
        listeners.push(l);
        all.push((id, url));
    }
    let voters: Vec<u64> = (0..n).collect();
    let mut cfg = HostConfig::default();
    cfg.propose_timeout = timeout;
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
    (nodes, cfg.tick)
}

async fn publish_response_stub(status: StatusCode, body: serde_json::Value) -> String {
    let (listener, url) = bind().await;
    let router = Router::new().route(
        "/raft/publish",
        post(move || {
            let body = body.clone();
            async move { (status, Json(body)) }
        }),
    );
    tokio::spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("the publish-response stub accepts a forwarded request");
        transport_h2c::server::serve_connection(stream, router)
            .await
            .expect("the publish-response stub serves the forwarded request");
    });
    url
}

/// An applied single-voter proposal reports Completed with the index the state machine applied.
#[tokio::test]
async fn single_voter_completed_outcome_matches_applied_index() {
    let nodes = cluster(1).await;
    let outcome = nodes[0]
        .host
        .propose_outcome(b"single-voter-cmd".to_vec())
        .await;

    match outcome {
        ProposalOutcome::Completed { index } => {
            assert!(index >= 1, "completed index must be >= 1, got {index}");
            assert_eq!(
                nodes[0].sm.applied_index(),
                index,
                "state machine applied index must equal outcome index"
            );
        }
        other => panic!("expected ProposalOutcome::Completed, got: {other:?}"),
    }
}

/// A quiesced host returns RejectedBeforeAdmission, and propose() on that same host
/// returns an Err containing "proposal admission closed".
#[tokio::test]
async fn quiesced_host_rejects_before_admission_and_propose_preserves_error_string() {
    let nodes = cluster(1).await;
    let quiesced = nodes[0].host.quiesce_proposals();
    assert!(
        quiesced,
        "the first quiesce_proposals call must return true"
    );

    let outcome = nodes[0]
        .host
        .propose_outcome(b"cmd-after-quiesce".to_vec())
        .await;
    match outcome {
        ProposalOutcome::RejectedBeforeAdmission { ref reason } => {
            assert!(
                reason.contains("proposal admission closed"),
                "reason must contain 'proposal admission closed', got: {reason}"
            );
        }
        other => panic!("expected ProposalOutcome::RejectedBeforeAdmission, got: {other:?}"),
    }

    let res = nodes[0].host.propose(b"cmd-propose".to_vec()).await;
    assert!(res.is_err(), "propose() on quiesced host must return Err");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("proposal admission closed"),
        "error message must contain 'proposal admission closed', got: {err_msg}"
    );
}

/// A host that never wins an election reports RejectedBeforeAdmission rather than
/// Ambiguous, because nothing was ever sent.
#[tokio::test]
async fn no_leader_elected_rejects_before_admission() {
    let sm = TestSm::new();
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let mut cfg = HostConfig::default();
    cfg.propose_timeout = Duration::from_millis(150);
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        cfg,
    );

    let outcome = host.propose_outcome(b"cmd-no-leader".to_vec()).await;
    match outcome {
        ProposalOutcome::RejectedBeforeAdmission { reason } => {
            assert!(
                reason.contains("no leader elected"),
                "reason must contain 'no leader elected', got: {reason}"
            );
        }
        other => panic!("expected ProposalOutcome::RejectedBeforeAdmission, got: {other:?}"),
    }
}

/// A single-voter leader with an injected save failure reports DurabilityFailure
/// carrying Some(index).
#[tokio::test]
async fn injected_save_failure_reports_durability_failure_with_allocated_index() {
    let nodes = cluster(1).await;
    nodes[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    let outcome = nodes[0]
        .host
        .propose_outcome(b"cmd-save-failure".to_vec())
        .await;
    match outcome {
        ProposalOutcome::DurabilityFailure { index, failure } => {
            assert!(
                index.is_some(),
                "first durability failure must have allocated index Some(i)"
            );
            let idx = index.unwrap();
            assert!(idx >= 1, "allocated index must be >= 1, got {idx}");
            assert_eq!(failure.kind, ErrorKind::StorageFull);
        }
        other => {
            panic!("expected ProposalOutcome::DurabilityFailure with Some(index), got: {other:?}")
        }
    }
}

/// A latched host with prior durability failure reports DurabilityFailure carrying None,
/// because the latched check runs before any append.
#[tokio::test]
async fn latched_durability_failure_reports_none_index() {
    let nodes = cluster(1).await;
    nodes[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    // First proposal triggers save failure and latches failure state.
    let outcome1 = nodes[0]
        .host
        .propose_outcome(b"cmd-first-save-failure".to_vec())
        .await;
    match outcome1 {
        ProposalOutcome::DurabilityFailure { index, .. } => {
            assert!(index.is_some(), "first proposal must have allocated index");
        }
        other => panic!("expected first proposal to fail with DurabilityFailure, got: {other:?}"),
    }

    // Second proposal on the same host without re-arming injector.
    let outcome2 = nodes[0]
        .host
        .propose_outcome(b"cmd-latched-proposal".to_vec())
        .await;
    match outcome2 {
        ProposalOutcome::DurabilityFailure { index, failure } => {
            assert!(
                index.is_none(),
                "latched durability failure must have index None, got {index:?}"
            );
            assert_eq!(failure.kind, ErrorKind::StorageFull);
        }
        other => panic!("expected ProposalOutcome::DurabilityFailure with None, got: {other:?}"),
    }
}

/// A node known to be a follower forwards to the live leader and returns the
/// completed index only after that follower applies the command itself.
#[tokio::test]
async fn follower_forwarding_to_live_leader_completes_after_local_apply() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;
    assert!(
        !nodes[follower].host.is_leader().await,
        "selected node must be a follower before it forwards"
    );

    let outcome = nodes[follower]
        .host
        .propose_outcome(b"forwarded-cmd-to-live-leader".to_vec())
        .await;

    match outcome {
        ProposalOutcome::Completed { index } => {
            assert!(index >= 1, "forwarded completed index must be positive");
            assert_eq!(
                nodes[follower].sm.applied_index(),
                index,
                "the forwarding follower must apply the completed index"
            );
            assert_eq!(
                nodes[leader].sm.applied_index(),
                index,
                "the original leader must commit the forwarded command"
            );
        }
        other => panic!("expected forwarded ProposalOutcome::Completed, got: {other:?}"),
    }
}

/// A follower forwarding to a quiesced leader preserves the leader's typed
/// rejection instead of treating a known pre-append refusal as ambiguous.
#[tokio::test]
async fn follower_forwarding_to_quiesced_leader_reports_rejected_before_admission() {
    let (nodes, tick) = custom_timeout_cluster(3, Duration::from_millis(300)).await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");

    let quiesced = nodes[leader].host.quiesce_proposals();
    assert!(
        quiesced,
        "the first quiesce_proposals call must return true"
    );

    let follower = (leader + 1) % 3;
    let outcome = nodes[follower]
        .host
        .propose_outcome(b"forwarded-cmd-to-quiesced-leader".to_vec())
        .await;

    match outcome {
        ProposalOutcome::RejectedBeforeAdmission { reason } => {
            assert!(
                reason.contains("proposal admission closed"),
                "forwarded admission rejection must preserve its reason, got: {reason}"
            );
        }
        other => panic!("expected ProposalOutcome::RejectedBeforeAdmission, got: {other:?}"),
    }
}

/// A registered leader address that accepts h2c but never responds makes the
/// forwarding deadline expire. This is distinct from both a typed admission
/// rejection and a malformed immediate reply.
#[tokio::test]
async fn follower_forwarding_transport_timeout_is_ambiguous_with_routing_timeout_reason() {
    let (nodes, tick) = custom_timeout_cluster(3, Duration::from_millis(300)).await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;
    assert!(
        !nodes[follower].host.is_leader().await,
        "selected node must be a follower before it forwards"
    );
    let (listener, stalled_url) = bind().await;
    nodes[follower]
        .host
        .upsert_peer(leader as u64, stalled_url)
        .await;

    let stalled_request = tokio::spawn(async move {
        let (mut stream, _) = tokio::time::timeout(Duration::from_secs(2), listener.accept())
            .await
            .expect("the follower sends a forwarded request before its deadline")
            .expect("the stalled listener accepts the forwarded request");
        let mut preface = [0_u8; 24];
        tokio::time::timeout(Duration::from_secs(2), stream.read_exact(&mut preface))
            .await
            .expect("the forwarding client writes the h2c preface")
            .expect("the forwarding client writes a complete h2c preface");
        assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        tokio::time::sleep(Duration::from_millis(600)).await;
    });

    let outcome = tokio::time::timeout(
        Duration::from_secs(2),
        nodes[follower]
            .host
            .propose_outcome(b"forwarded-cmd-to-stalled-leader".to_vec()),
    )
    .await
    .expect("the forwarding proposal respects its deadline");
    tokio::time::timeout(Duration::from_secs(2), stalled_request)
        .await
        .expect("the stalled listener completes")
        .expect("the stalled listener does not panic");

    match outcome {
        ProposalOutcome::Ambiguous {
            index: None,
            reason,
        } => {
            assert!(
                reason.contains("proposal routing timed out"),
                "transport timeout must use the routing-timeout outcome, got: {reason}"
            );
        }
        other => panic!("expected routing-timeout ProposalOutcome::Ambiguous, got: {other:?}"),
    }
}

/// A 503 is typed only when it contains both the exact outcome discriminator
/// and a string refusal reason. A malformed admission body remains ambiguous.
#[tokio::test]
async fn follower_forwarding_malformed_admission_body_remains_ambiguous() {
    let (nodes, tick) = custom_timeout_cluster(3, Duration::from_millis(300)).await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;
    let malformed_url = publish_response_stub(
        StatusCode::SERVICE_UNAVAILABLE,
        serde_json::json!({ "outcome": "rejected_before_admission" }),
    )
    .await;
    nodes[follower]
        .host
        .upsert_peer(leader as u64, malformed_url)
        .await;

    let outcome = nodes[follower]
        .host
        .propose_outcome(b"forwarded-cmd-to-malformed-admission-reply".to_vec())
        .await;

    match outcome {
        ProposalOutcome::Ambiguous { index: None, .. } => {}
        other => panic!("expected malformed forwarded refusal to remain Ambiguous, got: {other:?}"),
    }
}

/// Negative control: sequential successful proposals on a healthy host strictly increase
/// indices and never return Ambiguous, RejectedBeforeAdmission, or DurabilityFailure.
#[tokio::test]
async fn sequential_successful_proposals_increase_indices() {
    let nodes = cluster(1).await;
    let mut prev_index = 0;

    for i in 1..=4u8 {
        let outcome = nodes[0]
            .host
            .propose_outcome(vec![i, i.wrapping_add(1), 42])
            .await;
        match outcome {
            ProposalOutcome::Completed { index } => {
                assert!(
                    index > prev_index,
                    "indices must strictly increase: {index} > {prev_index}"
                );
                prev_index = index;
                assert_eq!(
                    nodes[0].sm.applied_index(),
                    index,
                    "state machine applied index must equal completed index"
                );
            }
            ProposalOutcome::Ambiguous { .. } => {
                panic!("proposal {i} must not return Ambiguous");
            }
            ProposalOutcome::RejectedBeforeAdmission { .. } => {
                panic!("proposal {i} must not return RejectedBeforeAdmission");
            }
            ProposalOutcome::DurabilityFailure { .. } => {
                panic!("proposal {i} must not return DurabilityFailure");
            }
        }
    }
}

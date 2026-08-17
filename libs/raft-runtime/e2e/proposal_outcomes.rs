//! Proposal outcome typing and classification tests (#3663).
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
//! 6. A follower forwarding to a quiesced leader returns `Ambiguous` with `None`.
//! 7. Negative control: sequential proposals on a healthy host strictly increase indices.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;

use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, ProposalOutcome, RaftHost, RaftStateMachine, RaftStore,
};
use tempfile::TempDir;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster, peers_excluding, Node, TestSm};

async fn custom_timeout_cluster(n: u64, timeout: Duration) -> Vec<Node> {
    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..n {
        let (l, url) = bind().await;
        listeners.push(l);
        all.push((id, url));
    }
    let voters: Vec<u64> = (0..n).collect();
    let mut nodes = Vec::new();
    for (idx, listener) in listeners.into_iter().enumerate() {
        let id = idx as u64;
        let peers = peers_excluding(id, &all);
        let sm = TestSm::new();
        let dir = TempDir::new().unwrap();
        let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
        let mut cfg = HostConfig::default();
        cfg.propose_timeout = timeout;
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
    nodes
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
    assert!(quiesced, "the first quiesce_proposals call must return true");

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
        other => panic!("expected ProposalOutcome::DurabilityFailure with Some(index), got: {other:?}"),
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

/// A follower forwarding to a quiesced leader reports Ambiguous with no index.
#[tokio::test]
async fn follower_forwarding_to_quiesced_leader_reports_ambiguous_without_index() {
    let nodes = custom_timeout_cluster(3, Duration::from_millis(300)).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");

    let quiesced = nodes[leader].host.quiesce_proposals();
    assert!(quiesced, "the first quiesce_proposals call must return true");

    let follower = (leader + 1) % 3;
    let outcome = nodes[follower]
        .host
        .propose_outcome(b"forwarded-cmd-to-quiesced-leader".to_vec())
        .await;

    match outcome {
        ProposalOutcome::Ambiguous { index, .. } => {
            assert!(
                index.is_none(),
                "forwarding to quiesced leader must report Ambiguous with index None, got: {index:?}"
            );
        }
        other => panic!("expected ProposalOutcome::Ambiguous with index None, got: {other:?}"),
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

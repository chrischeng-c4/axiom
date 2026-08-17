//! Proposal admission lifecycle boundary tests (#3657).
//!
//! # What was missing
//!
//! `RaftHost` had no way to stop accepting proposals before shutdown.
//! `shutdown()` aborted the tick and pump tasks and drained tracked peer RPCs,
//! but proposals arriving while that ran were still admitted and appended.
//!
//! These rows test:
//! 1. `quiesce_proposals()` on a leader causes subsequent local proposals on that
//!    leader to return an `Err` containing `proposal admission closed`, and `/raftz`
//!    reports `proposal_admission_closed` true with `lifecycle_generation` 1.
//! 2. Forwarded proposals sent directly to the quiesced leader's `/raft/publish`
//!    over h2c return `503 Service Unavailable` with a JSON error containing
//!    `proposal admission closed`.
//! 3. Eight concurrent `quiesce_proposals()` calls on a shared host collapse into
//!    exactly one transition (one returning true, seven returning false), and `/raftz`
//!    reports `lifecycle_generation` 1.
//! 4. Negative control: an unquiesced cluster admits 8 proposals successfully,
//!    replicates them to all nodes, and all 3 nodes report `proposal_admission_closed`
//!    false and `lifecycle_generation` 0.

use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::{RaftStateMachine, RaftStatus};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, cluster};

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

/// A quiesced leader refuses fresh local proposals with an error containing
/// "proposal admission closed", and reports generation 1 on /raftz.
#[tokio::test]
async fn local_proposal_refused_after_quiesce() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();

    let quiesced = nodes[leader].host.quiesce_proposals();
    assert!(quiesced, "the first quiesce_proposals call must return true");

    let res = nodes[leader].host.propose(b"command-after-quiesce".to_vec()).await;
    assert!(res.is_err(), "proposal on quiesced leader must return Err");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("proposal admission closed"),
        "error message must contain 'proposal admission closed', got: {err_msg}"
    );

    let s = status(&client, &nodes[leader].url).await;
    assert!(
        s.proposal_admission_closed,
        "proposal_admission_closed must be true on quiesced leader"
    );
    assert_eq!(
        s.lifecycle_generation, 1,
        "lifecycle_generation must be exactly 1 on quiesced leader"
    );
}

/// A quiesced leader refuses forwarded proposals on /raft/publish with 503
/// Service Unavailable and a JSON error containing "proposal admission closed".
#[tokio::test]
async fn forwarded_publish_refused_after_quiesce() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();
    let group_id = nodes[leader].host.group_id().0.clone();

    let quiesced = nodes[leader].host.quiesce_proposals();
    assert!(quiesced, "the first quiesce_proposals call must return true");

    let resp = client
        .post(format!("{}/raft/publish", nodes[leader].url))
        .json(&serde_json::json!({
            "group_id": group_id,
            "command": b"forwarded-cmd-after-quiesce".to_vec(),
        }))
        .send()
        .await
        .expect("publish request sends");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "quiesced leader must answer /raft/publish with 503 Service Unavailable"
    );
    let body: serde_json::Value = resp.json().await.expect("response body is valid json");
    let error_text = body
        .get("error")
        .and_then(|e| e.as_str())
        .expect("response json contains string error field");
    assert!(
        error_text.contains("proposal admission closed"),
        "error field must contain 'proposal admission closed', got: {error_text}"
    );
}

/// Eight concurrent callers racing quiesce_proposals on one host observe exactly
/// one successful transition (true) and seven no-ops (false), and /raftz reports
/// generation 1.
#[tokio::test]
async fn concurrent_quiesce_collapses_to_single_transition() {
    let nodes = cluster(1).await;
    let client = h2c_client();
    let host = Arc::clone(&nodes[0].host);

    let mut handles = Vec::with_capacity(8);
    for _ in 0..8 {
        let h = Arc::clone(&host);
        handles.push(tokio::spawn(async move { h.quiesce_proposals() }));
    }

    let mut results = Vec::with_capacity(8);
    for handle in handles {
        results.push(handle.await.expect("quiesce task completes"));
    }

    let true_count = results.iter().filter(|&&r| r).count();
    let false_count = results.iter().filter(|&&r| !r).count();
    assert_eq!(
        true_count, 1,
        "exactly one concurrent caller must observe true"
    );
    assert_eq!(
        false_count, 7,
        "all other concurrent callers must observe false"
    );

    let s = status(&client, &nodes[0].url).await;
    assert!(
        s.proposal_admission_closed,
        "proposal_admission_closed must be true"
    );
    assert_eq!(
        s.lifecycle_generation, 1,
        "lifecycle_generation must be exactly 1"
    );
}

/// Negative control: a cluster that is never quiesced admits proposals normally
/// and all nodes report proposal_admission_closed false with lifecycle_generation 0.
#[tokio::test]
async fn unquiesced_cluster_admits_proposals_and_reports_generation_zero() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();

    for i in 0..8u8 {
        let res = nodes[leader]
            .host
            .propose(vec![i, i.wrapping_add(1), 42])
            .await;
        assert!(
            res.is_ok(),
            "unquiesced leader must accept proposal {i}: {:?}",
            res.err()
        );
    }

    // Await replication to all nodes.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut all_applied = true;
        for node in &nodes {
            if node.sm.applied_index() < 8 {
                all_applied = false;
                break;
            }
        }
        if all_applied {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for 8 proposals to replicate to all nodes"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    for (idx, node) in nodes.iter().enumerate() {
        let s = status(&client, &node.url).await;
        assert!(
            !s.proposal_admission_closed,
            "node {idx} must report proposal_admission_closed = false"
        );
        assert_eq!(
            s.lifecycle_generation, 0,
            "node {idx} must report lifecycle_generation = 0"
        );
    }
}

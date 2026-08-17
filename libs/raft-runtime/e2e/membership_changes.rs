//! Membership changes reachable on a running host and observed on peers (#3646).
//!
//! # What was missing
//!
//! `libs/raft-core` contains promotion, demotion and removal logic, validated
//! by in-process tests. However, `RaftHost` exposed no membership-mutation
//! methods, and no accessor was provided for the locked node.
//!
//! These rows test `promote_learner`, `demote_voter`, and `remove_member` on a
//! running host across h2c transport, asserting on peer status endpoints.

use std::time::{Duration, Instant};

use raft_runtime::{
    DemotionRefused, MembershipPhase, PromotionRefused, RaftStatus, RemovalRefused,
};

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

/// A running host demotes a voter to learner, observes the change on the demoted
/// peer's own status and a bystander follower's status, and then promotes the
/// learner back to voter, observing restoration on both peers.
#[tokio::test]
async fn a_running_host_demotes_and_promotes_back_observed_on_peers() {
    let nodes = cluster(4).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a four-voter cluster elects a leader");
    let target = (leader + 1) % 4;
    let bystander = (leader + 2) % 4;
    let client = h2c_client();

    let target_id = target as u64;

    // Demote voter `target`.
    nodes[leader]
        .host
        .demote_voter(target_id)
        .await
        .expect("demoting a voter from a 4-voter group succeeds");

    // Poll demoted node's status to Stable with target as learner.
    let demoted_status = poll_status_until(
        &client,
        &nodes[target].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.learners.contains(&target_id)
                && !s.committed_voters.contains(&target_id)
        },
        Duration::from_secs(5),
        "demoted node reports Stable phase with itself as learner",
    )
    .await;

    assert_eq!(demoted_status.membership_phase, MembershipPhase::Stable);
    assert_eq!(demoted_status.incoming_voters, None);
    assert_eq!(demoted_status.committed_voters.len(), 3);
    assert!(!demoted_status.committed_voters.contains(&target_id));
    assert_eq!(demoted_status.learners, vec![target_id]);
    assert_eq!(demoted_status.role, "Learner");

    // Poll bystander follower's status to Stable with target as learner.
    let bystander_status = poll_status_until(
        &client,
        &nodes[bystander].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.learners.contains(&target_id)
                && !s.committed_voters.contains(&target_id)
        },
        Duration::from_secs(5),
        "bystander node reports Stable phase with demoted node as learner",
    )
    .await;

    assert_eq!(bystander_status.membership_phase, MembershipPhase::Stable);
    assert_eq!(bystander_status.incoming_voters, None);
    assert_eq!(
        bystander_status.committed_voters,
        demoted_status.committed_voters
    );
    assert_eq!(bystander_status.learners, vec![target_id]);

    // Now promote the learner back to voter.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match nodes[leader].host.promote_learner(target_id).await {
            Ok(_) => break,
            Err(PromotionRefused::NotCaughtUp { .. }) => {
                assert!(
                    Instant::now() < deadline,
                    "the leader never recorded the learner as caught up"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            Err(other) => panic!(
                "a leader promoting a caught-up learner of its own group must accept; got {other:?}"
            ),
        }
    }

    // Poll promoted node's status to Stable with 4 voters and no learners.
    let repromoted_status = poll_status_until(
        &client,
        &nodes[target].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.committed_voters.len() == 4
                && s.learners.is_empty()
        },
        Duration::from_secs(5),
        "promoted node reports Stable phase with 4 voters and 0 learners",
    )
    .await;

    assert_eq!(repromoted_status.membership_phase, MembershipPhase::Stable);
    assert_eq!(repromoted_status.incoming_voters, None);
    assert_eq!(repromoted_status.committed_voters.len(), 4);
    assert!(repromoted_status.committed_voters.contains(&target_id));
    assert!(repromoted_status.learners.is_empty());

    // Poll bystander follower's status to Stable with 4 voters and no learners.
    let repromoted_bystander = poll_status_until(
        &client,
        &nodes[bystander].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.committed_voters.len() == 4
                && s.learners.is_empty()
        },
        Duration::from_secs(5),
        "bystander node reports Stable phase with 4 voters and 0 learners",
    )
    .await;

    assert_eq!(
        repromoted_bystander.membership_phase,
        MembershipPhase::Stable
    );
    assert_eq!(repromoted_bystander.incoming_voters, None);
    assert_eq!(
        repromoted_bystander.committed_voters,
        repromoted_status.committed_voters
    );
    assert!(repromoted_bystander.learners.is_empty());
}

/// A running host removes a member from a 4-voter group, and a bystander
/// follower reports 3 committed voters in the Stable phase.
#[tokio::test]
async fn a_running_host_removes_a_member_observed_on_bystander() {
    let nodes = cluster(4).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a four-voter cluster elects a leader");
    let target = (leader + 1) % 4;
    let bystander = (leader + 2) % 4;
    let client = h2c_client();

    let target_id = target as u64;

    nodes[leader]
        .host
        .remove_member(target_id)
        .await
        .expect("removing a member from a 4-voter group succeeds");

    let bystander_status = poll_status_until(
        &client,
        &nodes[bystander].url,
        |s| {
            s.membership_phase == MembershipPhase::Stable
                && s.committed_voters.len() == 3
                && !s.committed_voters.contains(&target_id)
        },
        Duration::from_secs(5),
        "bystander reports 3 committed voters in Stable phase",
    )
    .await;

    assert_eq!(bystander_status.membership_phase, MembershipPhase::Stable);
    assert_eq!(bystander_status.incoming_voters, None);
    assert_eq!(bystander_status.committed_voters.len(), 3);
    assert!(!bystander_status.committed_voters.contains(&target_id));
    assert!(bystander_status.learners.is_empty());
}

/// Calling promote_learner, demote_voter, or remove_member on a follower returns
/// the core's NotLeader refusal, matched by variant.
#[tokio::test]
async fn a_host_that_is_not_the_leader_returns_core_refusals() {
    let nodes = cluster(4).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a four-voter cluster elects a leader");
    let follower = (leader + 1) % 4;
    let other = (leader + 2) % 4;
    let other_id = other as u64;

    match nodes[follower].host.promote_learner(other_id).await {
        Err(PromotionRefused::NotLeader) => {}
        other => panic!("expected Err(PromotionRefused::NotLeader), got {other:?}"),
    }

    match nodes[follower].host.demote_voter(other_id).await {
        Err(DemotionRefused::NotLeader) => {}
        other => panic!("expected Err(DemotionRefused::NotLeader), got {other:?}"),
    }

    match nodes[follower].host.remove_member(other_id).await {
        Err(RemovalRefused::NotLeader) => {}
        other => panic!("expected Err(RemovalRefused::NotLeader), got {other:?}"),
    }
}

//! Starting a leadership handoff from a running host, over the wire (#3586).
//!
//! # What was missing
//!
//! `raft_core::RaftNode::transfer_leadership` has been correct since #3571, and
//! `libs/raft-core/e2e/leadership_transfer.rs` measures it thoroughly — on an
//! in-process bus, against a node the row holds directly. Nothing that runs a
//! real host could reach it: `RaftHost` exposes no transfer entry point and
//! hands out no accessor for the locked node, so the operation existed and was
//! unreachable. #3571's own sweep records the consequence: deleting the
//! `RaftMsg::TimeoutNow` arm of the host's `send_request`, which drops the
//! message into the catch-all instead of posting it, left both crates green.
//!
//! These rows buy the wire. They run hosts that talk h2c to each other, so the
//! leader has to serialize the handoff, post it, and have the receiver act on
//! it before any of them passes.
//!
//! # Why three hosts rather than the two the wire needs
//!
//! Two hosts are enough to cross the wire, but in a two-node group "leadership
//! arrived at the node I named" and "leadership arrived at the only other node"
//! are the same sentence, and an implementation that ignores the target
//! argument satisfies both. The third host makes the name load-bearing: the row
//! names one of two eligible followers and asserts the other one did not end up
//! holding the group.
//!
//! Three hosts also make the catch-up wait real. Quorum in a three-voter group
//! is two, so a proposal commits without the named target having acknowledged
//! anything; the row has to wait on that target's own log rather than infer it
//! from the leader having committed.
//!
//! # Why the arrival is bounded in wall-clock, and why the registry row is not
//!
//! `libs/raft-core/src/lib.rs:45` sets `ELECTION_MIN` to 50 ticks and
//! `libs/raft-runtime/src/config.rs:40` makes a tick 20ms, so no node can
//! campaign on its own account inside one second of its clock being reset — and
//! a leader that is heartbeating keeps resetting it. Bounding the arrival well
//! inside that is what separates a handoff that was delivered from a group that
//! re-elected; the bound is deliberately far above what a loopback handoff
//! costs, since the heartbeats already rule the timeout out on their own.
//!
//! That reasoning does not transfer to the registry row, and measuring it by
//! the clock there would be wrong twice over. A `#[tokio::test]` runs on a
//! current-thread runtime, so a synchronous call on the test's own thread — a
//! TLS provider being installed the first time an h2c client is built, for one —
//! stops the host's tick task for as long as it runs, and elapsed wall-clock
//! stops meaning elapsed ticks. That row therefore asks for a term no
//! accumulation of spontaneous elections could have reached in its lifetime,
//! and asserts the exact value that arrives. The clock is not consulted.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tempfile::TempDir;

use raft_runtime::{
    group::GroupId, FsyncPolicy, HostConfig, Membership, RaftHost, RaftRegistry, RaftStateMachine,
    RaftStatus, RaftStore, TransferRefused,
};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster, TestSm};

/// The shortest election timeout any node in these rows can have: `ELECTION_MIN`
/// ticks of `HostConfig::default().tick`. Ticks are driven by a sleep, so a
/// loaded machine only ever makes this longer.
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

async fn group_statuses(client: &reqwest::Client, url: &str) -> BTreeMap<String, RaftStatus> {
    client
        .get(format!("{url}/raftz"))
        .send()
        .await
        .expect("a registry serves the status of every group it holds")
        .json()
        .await
        .expect("the multi-group status is the published shape")
}

/// The item. A running host hands the group to a peer it names, the message
/// crosses h2c, and the peer is holding the group afterwards.
#[tokio::test]
async fn a_running_host_hands_leadership_to_the_peer_it_names() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let target = (leader + 1) % 3;
    let bystander = (leader + 2) % 3;
    let client = h2c_client();

    for i in 0..5u8 {
        nodes[leader]
            .host
            .propose(vec![i])
            .await
            .expect("the leader accepts a proposal");
    }

    // Quorum here is two, so the entries above are committed without the named
    // target having acknowledged one of them. Wait on that target's own log.
    let leader_last = status(&client, &nodes[leader].url).await.last_index;
    assert!(
        leader_last >= 5,
        "the leader must hold the entries it proposed, or the catch-up wait \
         below is satisfied by an empty log"
    );
    let caught_up = Instant::now() + Duration::from_secs(10);
    loop {
        let s = status(&client, &nodes[target].url).await;
        if s.last_index == leader_last && s.commit_index == leader_last {
            break;
        }
        assert!(
            Instant::now() < caught_up,
            "the named target never caught up with the leader, so the handoff \
             below would be refused for a reason this row is not about"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // The leader's per-peer match index is on no public surface, so the one
    // refusal that is purely an acknowledgement still being in flight is
    // retried. Every other refusal fails the row where it stands.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match nodes[leader].host.transfer_leadership(target as u64).await {
            Ok(()) => break,
            Err(TransferRefused::NotCaughtUp { .. }) => {
                assert!(
                    Instant::now() < deadline,
                    "the leader never recorded the target as caught up, though \
                     the target's own log says it is"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            Err(other) => panic!(
                "a leader handing off to a caught-up voter of its own group must \
                 accept; got {other:?}"
            ),
        }
    }

    let handed_off = Instant::now();
    let mut arrived = None;
    while handed_off.elapsed() < DELIVERY_BUDGET {
        if nodes[target].host.is_leader().await {
            arrived = Some(handed_off.elapsed());
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let arrived = arrived.unwrap_or_else(|| {
        panic!(
            "leadership must arrive at the named peer within {DELIVERY_BUDGET:?}; \
             a host that queues the handoff and never posts it leaves the group \
             exactly where it was"
        )
    });
    assert!(
        arrived < ELECTION_TIMEOUT_FLOOR,
        "the handoff landed in {arrived:?}, at or past {ELECTION_TIMEOUT_FLOOR:?}, \
         the shortest election timeout in this group; past that the row cannot \
         tell a delivered message from an election that was due anyway"
    );
    assert!(
        !nodes[leader].host.is_leader().await,
        "the host that handed off must not still consider itself leader"
    );
    assert!(
        !nodes[bystander].host.is_leader().await,
        "leadership must arrive at the peer that was named; a group that ended \
         up led by the third node was not handed off, it re-elected"
    );
    assert_eq!(
        nodes[target].host.leader().await,
        Some(target as u64),
        "the new leader must name itself as the group's leader"
    );
}

/// The refusal a host returns is `raft-core`'s own, reachable from a consumer
/// that depends on `raft-runtime` alone.
///
/// This row does not stand in for the one above: a refusal never leaves the
/// process, so nothing here observes the wire.
#[tokio::test]
async fn a_host_that_is_not_the_leader_returns_the_cores_own_refusal() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let follower = (leader + 1) % 3;
    let other = (leader + 2) % 3;

    match nodes[follower].host.transfer_leadership(other as u64).await {
        Err(TransferRefused::NotLeader) => {}
        Err(other) => panic!(
            "a follower must be told it is not the leader, not that its target \
             is behind or is not a voter; got {other:?}"
        ),
        Ok(()) => panic!("a host that is not the leader has no leadership to hand off"),
    }
}

/// The registry's own `/raft/timeout-now` route: it reaches the group that was
/// addressed, and answers `404` for one it does not hold.
///
/// The only landed row touching this endpoint serves `host.router()`, which has
/// its own route and its own guard, so the registry's demux was never observed.
/// #3571's sweep removed the registry route entirely and nothing turned red.
///
/// The group has two voters and one host, so the node it names can campaign but
/// can never win. That leaves it a candidate, which is a state no node at rest
/// reaches — unlike "leader", which a single-voter group would reach on its own.
///
/// The handoff is sent from a term well above the node's, and the row asserts
/// the exact term that comes back. A node that campaigned on its own account
/// instead would be one term higher, not eleven, so the assertion separates the
/// two by value and needs no claim about how long anything took.
#[tokio::test]
async fn the_registry_routes_a_handoff_to_the_group_it_names() {
    // Built before the host, because building it installs a TLS provider on
    // this test's own thread, and a current-thread runtime cannot tick the host
    // while that runs.
    let client = h2c_client();
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let registry = RaftRegistry::new();
    registry
        .register(Arc::clone(&host))
        .expect("the group is registered once");

    let (l, url) = bind().await;
    let _serve = tokio::spawn({
        let r = registry.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let before = group_statuses(&client, &url).await;
    let alpha = before
        .get("alpha")
        .expect("the registry reports the group it holds");
    assert_eq!(
        alpha.role, "Follower",
        "the node must be at rest before it is asked to campaign, or the role \
         read back below was already what this row is looking for"
    );
    let sender_term = alpha.term + 10;
    let campaigning_term = sender_term + 1;

    let resp = client
        .post(format!("{url}/raft/timeout-now"))
        .json(&serde_json::json!({
            "group_id": "alpha",
            "from": 1,
            "req": { "term": sender_term, "leader": 1 },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "the registry must carry a handoff to the group it was addressed to"
    );

    let after = group_statuses(&client, &url).await;
    let alpha = after
        .get("alpha")
        .expect("the registry still reports the group it holds");
    assert_eq!(
        alpha.role, "Candidate",
        "the addressed group's node must campaign; it holds one of two votes, \
         so it stays a candidate and this is not a state it reaches by resting"
    );
    assert_eq!(
        alpha.term, campaigning_term,
        "the node must stand one term above the one it was asked at, which it \
         can only reach by having read the message; ten terms of spontaneous \
         elections is not something a node reaches while this row runs"
    );

    let unknown = client
        .post(format!("{url}/raft/timeout-now"))
        .json(&serde_json::json!({
            "group_id": "beta",
            "from": 1,
            "req": { "term": campaigning_term + 100, "leader": 1 },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        unknown.status(),
        reqwest::StatusCode::NOT_FOUND,
        "a group the registry does not hold must be answered as not found; a \
         single host's own router answers 400 for the same request, so a row \
         accepting either status does not show which router served it"
    );
    assert_eq!(
        group_statuses(&client, &url)
            .await
            .get("alpha")
            .map(|s| s.term),
        Some(campaigning_term),
        "a handoff addressed to an unheld group must not reach the group the \
         registry does hold; it was sent from a hundred terms further on, so a \
         demux that fell through to the only registered host moves this value"
    );
}

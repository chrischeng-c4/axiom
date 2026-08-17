//! Undeliverable outbound Raft message observability (#3651).
//!
//! # What was missing
//!
//! When an outbound Raft message could not be delivered because the peer had no
//! registered address, or because its address was withdrawn while a send was in
//! flight, the message was silently discarded.
//!
//! These rows test:
//! 1. Discarding outbound messages to unaddressed peers increments the
//!    `undeliverable_never_addressed` counter on `/raftz`, while the
//!    `undeliverable_withdrawn_address` counter stays at zero.
//! 2. Discarding in-flight outbound messages whose peer address was withdrawn via
//!    `forget_peer` increments the `undeliverable_withdrawn_address` counter on
//!    `/raftz`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStatus, RaftStore,
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

async fn spawn_node_with_unaddressed_peers(id: u64, voters: Vec<u64>) -> Node {
    let (listener, url) = bind().await;
    let sm = TestSm::new();
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
    let host = Arc::new(RaftHost::spawn(
        id,
        Membership {
            voters,
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

/// A host whose configuration names peers it has no address for discards
/// campaign messages and increments the never-addressed counter on /raftz,
/// while the withdrawn-address counter remains zero.
#[tokio::test]
async fn unaddressed_peer_messages_increment_never_addressed_counter() {
    let client = h2c_client();
    let node = spawn_node_with_unaddressed_peers(0, vec![0, 1, 2]).await;

    let s = poll_status_until(
        &client,
        &node.url,
        |s| s.undeliverable_never_addressed > 0 && s.undeliverable_withdrawn_address == 0,
        Duration::from_secs(5),
        "never-addressed counter rises while withdrawn-address stays at zero",
    )
    .await;

    assert!(s.undeliverable_never_addressed > 0);
    assert_eq!(s.undeliverable_withdrawn_address, 0);
}

/// Withdrawing a peer address while a send to it is stalled in flight causes
/// the drained lane request to be discarded at send_request, incrementing
/// the withdrawn-address counter on /raftz.
#[tokio::test]
async fn withdrawn_peer_address_during_in_flight_send_increments_withdrawn_counter() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();

    let victim = ((leader + 1) % 3) as u64;
    let (_stall_listener, stall_url) = bind().await;

    // Repoint victim peer to a listener that never accepts connections.
    nodes[leader].host.upsert_peer(victim, stall_url).await;

    // Issue a proposal on the leader to trigger AppendEntries outbound to victim.
    let propose_host = Arc::clone(&nodes[leader].host);
    tokio::spawn(async move {
        let _ = propose_host.propose(b"payload-stalled-send".to_vec()).await;
    });

    // 120ms into the stalled send, withdraw the address with forget_peer.
    tokio::time::sleep(Duration::from_millis(120)).await;
    nodes[leader].host.forget_peer(victim).await;

    // Poll /raftz on leader until withdrawn-address counter rises above zero.
    let s = poll_status_until(
        &client,
        &nodes[leader].url,
        |s| s.undeliverable_withdrawn_address > 0,
        Duration::from_secs(5),
        "withdrawn-address counter on leader rises above zero",
    )
    .await;

    assert!(s.undeliverable_withdrawn_address > 0);
}

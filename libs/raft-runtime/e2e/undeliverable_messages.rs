//! Undeliverable outbound Raft message observability (#3656).
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
//! 3. Successful delivery, a retryable transport loss to a still-registered peer,
//!    and repeated status reads do not increment either discard counter.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::{
    FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStatus, RaftStore,
};
use tempfile::TempDir;
use tokio::io::AsyncReadExt;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader_with_tick, bind, peers_excluding, Node, TestSm};

async fn cluster_with_long_rpc_timeout() -> (Vec<Node>, Duration) {
    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..3 {
        let (listener, url) = bind().await;
        listeners.push(listener);
        all.push((id, url));
    }
    let voters: Vec<u64> = (0..3).collect();
    let cfg = HostConfig {
        rpc_timeout: Duration::from_secs(30),
        ..HostConfig::default()
    };
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
        nodes.push(Node {
            host,
            sm,
            url: all[idx].1.clone(),
            _serve: serve,
            _dir: dir,
        });
    }
    (nodes, cfg.tick)
}

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

/// A delivered proposal and observing the published status twice do not discard
/// any message, so neither terminal-discard counter changes.
#[tokio::test]
async fn successful_delivery_and_repeated_status_reads_do_not_increment_discard_counters() {
    let (nodes, tick) = cluster_with_long_rpc_timeout().await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();
    let before = status(&client, &nodes[leader].url).await;

    nodes[leader]
        .host
        .propose(b"successful-delivery".to_vec())
        .await
        .expect("a healthy three-voter cluster delivers and commits the proposal");

    let after_delivery = status(&client, &nodes[leader].url).await;
    let after_duplicate_observation = status(&client, &nodes[leader].url).await;
    assert_eq!(
        (
            after_delivery.undeliverable_never_addressed,
            after_delivery.undeliverable_withdrawn_address,
        ),
        (
            before.undeliverable_never_addressed,
            before.undeliverable_withdrawn_address,
        ),
        "successful delivery must not count as terminal discard"
    );
    assert_eq!(
        (
            after_duplicate_observation.undeliverable_never_addressed,
            after_duplicate_observation.undeliverable_withdrawn_address,
        ),
        (
            after_delivery.undeliverable_never_addressed,
            after_delivery.undeliverable_withdrawn_address,
        ),
        "repeated status observation must not mutate discard counters"
    );
}

/// Losing a live h2c connection before the peer replies is a retryable
/// transport loss, not a terminal address discard while that peer remains
/// registered.
#[tokio::test]
async fn retryable_transport_loss_to_registered_peer_does_not_increment_discard_counters() {
    let (nodes, tick) = cluster_with_long_rpc_timeout().await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();
    let victim = ((leader + 1) % 3) as u64;
    let (loss_listener, loss_url) = bind().await;
    nodes[leader].host.upsert_peer(victim, loss_url).await;

    let loss_observed = tokio::spawn(async move {
        let (mut stream, _) = tokio::time::timeout(Duration::from_secs(5), loss_listener.accept())
            .await
            .expect("the registered peer receives an h2c connection")
            .expect("the transport-loss listener accepts the connection");
        let mut preface = [0_u8; 24];
        tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut preface))
            .await
            .expect("the connection sends the HTTP/2 preface")
            .expect("the connection yields a complete HTTP/2 preface");
        assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
        // Dropping the stream before a reply makes this a transport loss while
        // the leader still retains the victim's address.
    });

    let before = status(&client, &nodes[leader].url).await;
    tokio::time::timeout(
        Duration::from_secs(5),
        nodes[leader]
            .host
            .propose(b"retryable-transport-loss".to_vec()),
    )
    .await
    .expect("the healthy voter still lets the proposal finish within five seconds")
    .expect("the healthy voter commits the proposal despite the lost peer connection");
    tokio::time::timeout(Duration::from_secs(5), loss_observed)
        .await
        .expect("the test observes the lost registered-peer connection")
        .expect("the transport-loss observer does not panic");
    tokio::time::sleep(Duration::from_millis(50)).await;

    let after = status(&client, &nodes[leader].url).await;
    assert_eq!(
        (
            after.undeliverable_never_addressed,
            after.undeliverable_withdrawn_address,
        ),
        (
            before.undeliverable_never_addressed,
            before.undeliverable_withdrawn_address,
        ),
        "retryable transport loss to a registered peer must not count as terminal discard"
    );
}

/// Withdrawing a peer address while a send to it is stalled in flight causes
/// the drained lane request to be discarded at send_request, incrementing
/// the withdrawn-address counter on /raftz.
#[tokio::test]
async fn withdrawn_peer_address_during_in_flight_send_increments_withdrawn_counter() {
    let (nodes, tick) = cluster_with_long_rpc_timeout().await;
    let leader = await_leader_with_tick(&nodes, tick)
        .await
        .expect("a three-voter cluster elects a leader");
    let client = h2c_client();

    let victim = ((leader + 1) % 3) as u64;
    let healthy_voter = (0usize..3)
        .find(|id| *id != leader && *id as u64 != victim)
        .expect("a three-node cluster has a healthy voter");
    let (_stall_listener, stall_url) = bind().await;

    // Repoint victim peer to a local listener that lets us hold its first
    // outbound request open. This is the first outbound request, not the first
    // proposal, because it may be a heartbeat.
    nodes[leader].host.upsert_peer(victim, stall_url).await;

    // Start the first proposal, then accept and hold the victim lane's first
    // outbound TCP connection. The exact HTTP/2 preface proves this is h2c.
    let propose_host = Arc::clone(&nodes[leader].host);
    let first_proposal =
        tokio::spawn(async move { propose_host.propose(b"payload-first-lane".to_vec()).await });
    let (mut held_stream, _) =
        tokio::time::timeout(Duration::from_secs(5), _stall_listener.accept())
            .await
            .expect("the leader opens the victim connection within five seconds")
            .expect("the victim listener accepts the outbound connection");
    let mut preface = [0_u8; 24];
    tokio::time::timeout(Duration::from_secs(5), held_stream.read_exact(&mut preface))
        .await
        .expect("the victim connection sends the HTTP/2 preface within five seconds")
        .expect("the victim connection yields a complete HTTP/2 preface");
    assert_eq!(&preface, b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");

    // The first proposal must commit through the other voter while the victim
    // lane worker remains held by the accepted stream.
    let first_result = tokio::time::timeout(Duration::from_secs(5), first_proposal)
        .await
        .expect("the first proposal completes within five seconds")
        .expect("the first proposal task does not panic")
        .expect("the first proposal commits through the healthy voter");
    poll_status_until(
        &client,
        &nodes[healthy_voter].url,
        |s| s.commit_index >= first_result,
        Duration::from_secs(5),
        "healthy voter commits the first proposal",
    )
    .await;

    // A second synchronous flush leaves a victim message pending behind the
    // held first lane worker, while the healthy voter completes the proposal.
    tokio::time::timeout(
        Duration::from_secs(5),
        nodes[leader].host.propose(b"payload-second-lane".to_vec()),
    )
    .await
    .expect("the second proposal completes within five seconds")
    .expect("the second proposal commits through the healthy voter");

    let baseline = status(&client, &nodes[leader].url)
        .await
        .undeliverable_withdrawn_address;
    nodes[leader].host.forget_peer(victim).await;
    drop(held_stream);

    // Poll the leader's real /raftz until the withdrawn counter rises above the
    // baseline. Do not assume a fixed count or a zero never-addressed count.
    let s = poll_status_until(
        &client,
        &nodes[leader].url,
        |s| s.undeliverable_withdrawn_address > baseline,
        Duration::from_secs(5),
        "withdrawn-address counter on leader rises above its baseline",
    )
    .await;

    assert!(s.undeliverable_withdrawn_address > baseline);
}

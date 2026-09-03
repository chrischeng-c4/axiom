//! A follower answers `421` on `/raft/publish` before body extraction (#4006).
//!
//! `apps/tape/e2e/raft_cluster.rs:312` already states this promise for one
//! request shape, buried in the middle of a nine-step election/failover case.
//! This file is the promise on its own, over the request shapes that separate
//! *leadership is checked first* from *leadership is checked once the body
//! parsed*:
//!
//! * no `Content-Type` at all — axum's `Json` extractor answers `415` before
//!   any handler body runs;
//! * `Content-Type: application/json` carrying bytes that are not JSON, and
//!   bytes that are JSON of the wrong shape — the same extractor answers
//!   `400`/`422`, again before any handler body runs.
//!
//! All three are `421` plus a leader hint under the contract. The reason is
//! not tidiness: a client told `415` or `422` has been handed a problem it can
//! act on, so it fixes the body and retries *the same node*, forever, because
//! the body was never why the node refused. A client told `421 {"error":
//! "not-leader", "leader": <id>}` retargets and makes progress. Ordering the
//! leadership check ahead of extraction is what makes the answer the one the
//! caller can use, so the ordering is the observable behaviour and not an
//! implementation detail.
//!
//! The hint is asserted, not just the status. A `421` with no leader in it
//! costs the caller the same extra round trip that the misdirected write did,
//! and the route already returns `{"error": "not-leader", "leader": <node
//! id>}` on the path where the body happens to parse — so a fix that answers
//! `421` earlier and drops the hint would trade one defect for another.
//! `TapeRaft::leader()` (`apps/tape/src/raft.rs:793`) is the value; on a
//! settled group a follower names the elected leader.
//!
//! `leader_publish_still_accepts_forwarded_proposals` is this file's own
//! negative control. `/raft/publish` is also where the shared host forwards a
//! follower's own proposal (`libs/raft-runtime/src/host.rs:1452`), so a
//! blanket `421` on the route would satisfy every row above while silently
//! breaking every follower-originated append. That case is green against the
//! current tree and has to stay green.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_runtime::Membership;
use tape::raft::TapeRaft;
use tape::TapeJournal;

/// h2's in-process client pool can tear down live streams while a second
/// cluster in the same process is shutting down, so the cases here run one at
/// a time. `raft_cluster.rs:21` carries the same lock for the same reason.
static PUBLISH_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Node {
    raft: Arc<TapeRaft>,
    serve: tokio::task::JoinHandle<()>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

/// An in-process group serving the real peer router over real h2c listeners.
/// Every listener is bound before any node starts, so each node's peer map is
/// complete and the election is not racing a socket.
struct Cluster {
    urls: HashMap<u64, String>,
    nodes: Vec<Node>,
    _dirs: Vec<tempfile::TempDir>,
    client: reqwest::Client,
}

impl Cluster {
    async fn start(n: u64) -> Cluster {
        let mut listeners = Vec::new();
        let mut urls = HashMap::new();
        for id in 0..n {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            urls.insert(id, format!("http://{}", listener.local_addr().unwrap()));
            listeners.push(listener);
        }
        let membership = Membership {
            voters: (0..n).collect(),
            learners: vec![],
        };
        let dirs: Vec<tempfile::TempDir> =
            (0..n).map(|_| tempfile::tempdir().unwrap()).collect();

        let mut nodes = Vec::new();
        for (id, listener) in listeners.into_iter().enumerate() {
            let journal = Arc::new(Mutex::new(TapeJournal::default()));
            let peers: HashMap<u64, String> = urls
                .iter()
                .filter(|(peer, _)| **peer != id as u64)
                .map(|(peer, url)| (*peer, url.clone()))
                .collect();
            let raft = Arc::new(
                TapeRaft::spawn(
                    journal,
                    &dirs[id].path().join("raft"),
                    id as u64,
                    membership.clone(),
                    peers,
                    // No compaction in reach of these cases; the snapshot path
                    // is `raft_cluster.rs`'s business.
                    TapeRaft::host_config(1024),
                )
                .unwrap(),
            );
            let app = raft.router();
            let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
            let serve = tokio::spawn(async move {
                let _ = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        let _ = shutdown_rx.await;
                    })
                    .await;
            });
            nodes.push(Node {
                raft,
                serve,
                shutdown: Some(shutdown),
            });
        }

        Cluster {
            urls,
            nodes,
            _dirs: dirs,
            client: reqwest::Client::builder()
                .http2_prior_knowledge()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
        }
    }

    /// The elected leader, sampled until one node has claimed it three times
    /// running. A single sample can catch a candidate mid-term.
    async fn wait_leader(&self) -> usize {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut stable: Option<usize> = None;
        let mut samples = 0;
        loop {
            let mut seen = None;
            for (index, node) in self.nodes.iter().enumerate() {
                if node.raft.is_leader().await {
                    seen = Some(index);
                    break;
                }
            }
            match seen {
                Some(index) if stable == Some(index) => samples += 1,
                Some(index) => {
                    stable = Some(index);
                    samples = 1;
                }
                None => {
                    stable = None;
                    samples = 0;
                }
            }
            if samples >= 3 {
                return stable.expect("three stable samples name a leader");
            }
            assert!(Instant::now() < deadline, "no leader elected");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    fn follower_of(&self, leader: usize) -> usize {
        (0..self.nodes.len())
            .find(|index| *index != leader)
            .expect("a group larger than one has a follower")
    }

    /// A node that has not yet learned who won the election would answer a
    /// null hint for a reason that has nothing to do with this contract, so
    /// the cases wait for its own view to name the leader before asking it.
    async fn wait_follower_names_leader(&self, follower: usize, leader: u64) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if self.nodes[follower].raft.leader().await == Some(leader) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "follower {follower} never learned that {leader} is the leader"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    fn publish_url(&self, node: usize) -> String {
        format!("{}/raft/publish", self.urls[&(node as u64)])
    }

    async fn shutdown(mut self) {
        for node in &self.nodes {
            node.raft.shutdown().await.unwrap();
        }
        for node in &mut self.nodes {
            if let Some(shutdown) = node.shutdown.take() {
                let _ = shutdown.send(());
            }
        }
        for node in self.nodes.drain(..) {
            let _ = node.serve.await;
        }
        // Dropping the client schedules its pooled h2 drivers to close. Give
        // them one bounded turn before this test's runtime is torn down.
        drop(self.client);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Assert the misdirect answer on `response`, including the hint that makes it
/// actionable. `shape` names the request so a failure says which one.
async fn assert_misdirected_to(response: reqwest::Response, leader: u64, shape: &str) {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::MISDIRECTED_REQUEST,
        "{shape}: a follower must refuse the write for being the wrong node \
         before it judges the request body; body was {body:?}"
    );
    let hint: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|error| panic!("{shape}: 421 body is not JSON ({error}): {body:?}"));
    assert_eq!(hint["error"], "not-leader", "{shape}: 421 body was {body:?}");
    assert_eq!(
        hint["leader"].as_u64(),
        Some(leader),
        "{shape}: the 421 must name the leader to retarget, body was {body:?}"
    );
}

/// The answer this contract generalises, pinned as it stands today.
///
/// A follower whose body the extractor *did* accept already answers `421` with
/// the hint, and that is the answer the two cases below require for the bodies
/// it never gets to look at. Pinning it here does two things: it shows the
/// shape those cases assert is the shape this route already produces, and it
/// refuses a fix that answers `421` sooner while dropping the leader out of
/// the body. Green against the current tree; it is here to stay green.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_publish_names_the_leader_when_the_body_parses() {
    let _guard = PUBLISH_TEST_LOCK.lock().await;
    let cluster = Cluster::start(3).await;
    let leader = cluster.wait_leader().await;
    let follower = cluster.follower_of(leader);
    cluster
        .wait_follower_names_leader(follower, leader as u64)
        .await;

    // The envelope shape the host itself forwards (`host.rs:1457`): the group
    // this host was spawned into, plus opaque command bytes. What the bytes
    // decode to is not reachable from a follower and does not matter here.
    let command = serde_json::to_vec(&tape::raft::TapeCommand::Append {
        topic: "orders".to_string(),
        key: None,
        payload: serde_json::json!({ "n": 99 }),
        timestamp_ms: 100,
        applied_at_ms: 100,
    })
    .unwrap();
    let response = cluster
        .client
        .post(cluster.publish_url(follower))
        .header("content-type", "application/json")
        .body(
            serde_json::to_vec(&serde_json::json!({
                "group_id": raft_runtime::LEGACY_GROUP_ID,
                "command": command,
            }))
            .unwrap(),
        )
        .send()
        .await
        .unwrap();
    assert_misdirected_to(response, leader as u64, "well-formed envelope").await;

    cluster.shutdown().await;
}

/// A publish POST with no `Content-Type` is misdirected, not unsupported.
///
/// This is the shape `raft_cluster.rs:316` sends: the serialized `TapeCommand`
/// bytes with no media type declared. Against the current tree the `Json`
/// extractor claims the request first and answers `415`.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_publish_answers_421_before_media_type_negotiation() {
    let _guard = PUBLISH_TEST_LOCK.lock().await;
    let cluster = Cluster::start(3).await;
    let leader = cluster.wait_leader().await;
    let follower = cluster.follower_of(leader);
    cluster
        .wait_follower_names_leader(follower, leader as u64)
        .await;

    let response = cluster
        .client
        .post(cluster.publish_url(follower))
        .body(
            serde_json::to_vec(&tape::raft::TapeCommand::Append {
                topic: "orders".to_string(),
                key: None,
                payload: serde_json::json!({ "n": 99 }),
                timestamp_ms: 100,
                applied_at_ms: 100,
            })
            .unwrap(),
        )
        .send()
        .await
        .unwrap();
    assert_misdirected_to(response, leader as u64, "no content-type").await;

    cluster.shutdown().await;
}

/// A publish POST the extractor would reject is misdirected too.
///
/// Two bodies, both `application/json`: one that is not JSON at all, and one
/// that is JSON of the wrong shape. They cover the two rejections the `Json`
/// extractor can raise, and neither can be reached at all once leadership is
/// checked first — which is the whole ordering claim.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn follower_publish_answers_421_before_json_deserialization() {
    let _guard = PUBLISH_TEST_LOCK.lock().await;
    let cluster = Cluster::start(3).await;
    let leader = cluster.wait_leader().await;
    let follower = cluster.follower_of(leader);
    cluster
        .wait_follower_names_leader(follower, leader as u64)
        .await;

    let bodies: [(&str, Vec<u8>); 2] = [
        ("truncated json", br#"{"group_id":"#.to_vec()),
        (
            "well-formed json of the wrong shape",
            serde_json::to_vec(&serde_json::json!({
                "group_id": 7,
                "command": "not a byte array",
            }))
            .unwrap(),
        ),
    ];
    for (shape, body) in bodies {
        let response = cluster
            .client
            .post(cluster.publish_url(follower))
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_misdirected_to(response, leader as u64, shape).await;
    }

    cluster.shutdown().await;
}

/// The route still serves the host's own leader forwarding.
///
/// `/raft/publish` is where a follower's proposal is sent when the host routes
/// it to the leader, so answering `421` unconditionally would pass every case
/// above and break every append that did not start on the leader. This case is
/// green against the current tree; it is here to stay green.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn leader_publish_still_accepts_forwarded_proposals() {
    let _guard = PUBLISH_TEST_LOCK.lock().await;
    let cluster = Cluster::start(3).await;
    let leader = cluster.wait_leader().await;
    let follower = cluster.follower_of(leader);

    let (_, outcome) = cluster.nodes[follower]
        .raft
        .propose_append(
            "orders".to_string(),
            None,
            serde_json::json!({ "n": 7 }),
            100,
        )
        .await
        .unwrap();
    match outcome.expect("the follower's own proposal resolved") {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 7),
        other => panic!("the leader must accept the forwarded append, got {other:?}"),
    }

    // Read-your-write on the forwarding node: the follower applied it too.
    let applied = cluster.nodes[follower]
        .raft
        .journal()
        .lock()
        .unwrap()
        .end_offset("orders");
    assert_eq!(applied, 1, "the forwarding follower applied its own append");

    cluster.shutdown().await;
}

// HANDWRITE-BEGIN gap="missing-generator:unit-test:0ca51618" tracker="pending-tracker" reason="An in-process 3-node TapeRaft group over real h2c listeners (relay's tests/raft_cluster.rs shape, adapted to tape's Append/CheckpointPut commands): exactly one leader; a leader append is applied and readable on every node's journal; a follower append is forwarded to the leader by the host; a direct follower POST to the host's peer route answers 421 not-leader; killing (aborting) the leader's task re-elects a survivor with no committed loss; a small SnapshotPolicy threshold compacts the leader's raft log so a late-started fresh node catches up via InstallSnapshot instead of full log replay."
//! raft-runtime cluster integration (#1327): 3 `TapeRaft` nodes over real h2c.
//!
//! Mirrors relay's `tests/raft_cluster.rs` (#544) shape: election, a leader
//! append applied on every node's journal, a follower append forwarded by
//! the host, a direct follower peer-route POST answering 421, leader kill
//! re-electing with no committed loss, a recovered node catching up before a
//! second leader loss, and the snapshot/compaction path — a fresh node catches
//! up via InstallSnapshot instead of full log replay.
//!
//! #4167 pins the shared publish preflight through Tape's unwrapped
//! [`TapeRaft::router`]: a follower gives its leader hint before malformed,
//! missing-media-type, or valid local-group input is decoded, while a
//! well-formed foreign group is rejected without a hint.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_runtime::{Membership, LEGACY_GROUP_ID};
use tape::raft::TapeRaft;
use tape::TapeJournal;

// h2's in-process client pool can otherwise tear down active streams while
// three independent nine-node test tasks are shutting down concurrently.
static CLUSTER_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Node {
    raft: Arc<TapeRaft>,
    serve: tokio::task::JoinHandle<()>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

/// An in-process group: every node's listener + URL is bound up-front so peer
/// maps are complete, but nodes start individually (the snapshot test starts
/// one late).
struct Cluster {
    urls: HashMap<u64, String>,
    listeners: Vec<Option<tokio::net::TcpListener>>,
    membership: Membership,
    nodes: Vec<Option<Node>>,
    _dirs: Vec<tempfile::TempDir>,
    client: reqwest::Client,
}

impl Cluster {
    async fn prepare(n: u64) -> Cluster {
        let mut listeners = Vec::new();
        let mut urls = HashMap::new();
        for id in 0..n {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            urls.insert(id, format!("http://{}", l.local_addr().unwrap()));
            listeners.push(Some(l));
        }
        Cluster {
            urls,
            listeners,
            membership: Membership {
                voters: (0..n).collect(),
                learners: vec![],
            },
            nodes: (0..n).map(|_| None).collect(),
            _dirs: (0..n).map(|_| tempfile::tempdir().unwrap()).collect(),
            client: reqwest::Client::builder()
                .http2_prior_knowledge()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap(),
        }
    }

    /// Spawn node `id` (fresh in-memory journal, its own raft dir) and serve
    /// its peer router on the pre-bound listener.
    fn start_node(&mut self, id: usize, snapshot_every: u64) {
        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let peers: HashMap<u64, String> = self
            .urls
            .iter()
            .filter(|(k, _)| **k != id as u64)
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        let raft = Arc::new(
            TapeRaft::spawn(
                journal,
                &self._dirs[id].path().join("raft"),
                id as u64,
                self.membership.clone(),
                peers,
                TapeRaft::host_config(snapshot_every),
            )
            .unwrap(),
        );
        let app = raft.router();
        let listener = match self.listeners[id].take() {
            Some(listener) => listener,
            None => {
                // A deliberately late node must be connection-refused before
                // it joins, not TCP-connected to a bound-but-unserved socket:
                // timing out an h2 handshake is not a real absent-pod shape.
                let address = self.urls[&(id as u64)]
                    .strip_prefix("http://")
                    .unwrap()
                    .parse::<std::net::SocketAddr>()
                    .unwrap();
                let listener = std::net::TcpListener::bind(address).unwrap();
                listener.set_nonblocking(true).unwrap();
                tokio::net::TcpListener::from_std(listener).unwrap()
            }
        };
        let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
        let serve = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
        self.nodes[id] = Some(Node {
            raft,
            serve,
            shutdown: Some(shutdown),
        });
    }

    fn start_all(&mut self, snapshot_every: u64) {
        for id in 0..self.nodes.len() {
            self.start_node(id, snapshot_every);
        }
    }

    fn leave_listener_unbound_until_start(&mut self, id: usize) {
        drop(self.listeners[id].take());
    }

    fn raft(&self, id: usize) -> &Arc<TapeRaft> {
        &self.nodes[id].as_ref().expect("node running").raft
    }

    /// Abort the node's serve loop and drop its host (tick/pump abort on
    /// drop) -- an in-process stand-in for a killed node; `raft_failover.rs`
    /// covers the real `kill -9` subprocess case.
    async fn kill(&mut self, id: usize) {
        if let Some(mut n) = self.nodes[id].take() {
            // Deliberately abrupt for the failover case. The separate
            // subprocess test covers an actual SIGKILL.
            n.shutdown.take();
            n.serve.abort();
            let _ = n.serve.await;
            n.raft.shutdown().await.unwrap();
        }
    }

    async fn shutdown(mut self) {
        for node in self.nodes.iter().flatten() {
            node.raft.shutdown().await.unwrap();
        }
        for node in &mut self.nodes {
            if let Some(mut node) = node.take() {
                if let Some(shutdown) = node.shutdown.take() {
                    let _ = shutdown.send(());
                }
                let _ = node.serve.await;
            }
        }
        // Dropping reqwest clients schedules their pooled h2 connection
        // drivers to close. Give those drivers one bounded turn before this
        // test's Tokio runtime is torn down; otherwise h2 debug assertions can
        // observe bookkeeping streams while the runtime is aborting them.
        drop(self.client);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    fn live(&self) -> impl Iterator<Item = (usize, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| n.as_ref().map(|n| (i, n)))
    }

    async fn leader(&self) -> Option<usize> {
        for (i, n) in self.live() {
            if n.raft.is_leader().await {
                return Some(i);
            }
        }
        None
    }

    async fn wait_leader(&self) -> usize {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut stable = None;
        let mut samples = 0;
        loop {
            if let Some(i) = self.leader().await {
                if stable == Some(i) {
                    samples += 1;
                } else {
                    stable = Some(i);
                    samples = 1;
                }
                if samples >= 3 {
                    return i;
                }
            } else {
                stable = None;
                samples = 0;
            }
            assert!(Instant::now() < deadline, "no leader elected");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Wait until every LIVE node's journal holds at least `want` events for
    /// topic "orders".
    async fn wait_converged(&self, want: u64) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let ok = self
                .live()
                .all(|(_, n)| n.raft.journal().lock().unwrap().end_offset("orders") >= want);
            if ok {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "journals did not converge to {want}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    fn payload_ns(&self, id: usize) -> Vec<i64> {
        let r = self.raft(id).journal();
        let journal = r.lock().unwrap();
        journal
            .replay("orders", None, None, None)
            .into_iter()
            .map(|e| e.payload["n"].as_i64().unwrap())
            .collect()
    }

    /// A follower may expose a matching end offset while its last committed
    /// apply is still crossing the host's asynchronous peer-pump boundary.
    /// Stability proof therefore waits for the actual semantic event set, not
    /// only its count.
    async fn wait_payloads(&self, want: &[i64]) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let converged = self.live().all(|(id, _)| {
                let got = self.payload_ns(id);
                want.iter().all(|expected| got.contains(expected))
            });
            if converged {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "journals did not converge to payloads {want:?}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

async fn propose(raft: &TapeRaft, n: i64) -> tape::raft::TapeOutcome {
    let (_, outcome) = raft
        .propose_append(
            "orders".to_string(),
            None,
            serde_json::json!({ "n": n }),
            100,
        )
        .await
        .unwrap();
    outcome.expect("leader/follower claims its own apply outcome")
}

/// The peer route accepts the shared raft-runtime envelope, whose command is
/// serialized Tape domain data. Keeping this construction in the consumer
/// test proves Tape does not insert an application-owned publish middleware.
fn publish_envelope(group_id: &str, n: i64) -> serde_json::Value {
    serde_json::json!({
        "group_id": group_id,
        "command": serde_json::to_vec(&tape::raft::TapeCommand::Append {
            topic: "orders".to_string(),
            key: None,
            payload: serde_json::json!({ "n": n }),
            timestamp_ms: 100,
            applied_at_ms: 100,
        })
        .expect("the Tape raft command serializes"),
    })
}

/// A follower's redirect is a JSON leader hint, not merely a status code.
/// The exact node id proves that the request crossed the live Tape raft router.
async fn assert_follower_leader_hint(
    response: reqwest::Response,
    expected_leader: usize,
    request_shape: &str,
) {
    let status = response.status();
    let body = response
        .text()
        .await
        .expect("the follower response body is readable");
    assert_eq!(
        status,
        reqwest::StatusCode::MISDIRECTED_REQUEST,
        "follower must preflight {request_shape} as 421; body: {body}",
    );
    let hint: serde_json::Value =
        serde_json::from_str(&body).expect("the follower response is a JSON leader hint");
    assert_eq!(
        hint,
        serde_json::json!({
            "error": "not-leader",
            "leader": expected_leader as u64,
        }),
        "follower must return one stable leader hint for {request_shape}",
    );
}

/// Exactly one leader; a leader append applies on every node's journal; a
/// follower append is forwarded by the host; a direct follower peer-route
/// POST answers 421 not-leader; killing the leader re-elects with no
/// committed loss; the stopped node restarts from durable state, catches up,
/// and the group keeps accepting appends after a second leader loss.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_node_group_elects_replicates_forwards_and_fails_over() {
    let _guard = CLUSTER_TEST_LOCK.lock().await;
    let mut c = Cluster::prepare(3).await;
    c.start_all(1024);

    // Exactly one leader.
    let leader = c.wait_leader().await;
    let mut leaders = 0;
    for (_, n) in c.live() {
        if n.raft.is_leader().await {
            leaders += 1;
        }
    }
    assert_eq!(leaders, 1, "exactly one leader");

    let follower = c.live().map(|(i, _)| i).find(|i| *i != leader).unwrap();

    // The follower chooses its route before it relies on an HTTP content type
    // or a decodable envelope. Each direct request stays on Tape's real h2c
    // peer listener rather than a mock or application wrapper.
    let follower_publish = format!("{}/raft/publish", c.urls[&(follower as u64)]);
    let malformed = c
        .client
        .post(&follower_publish)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(format!(
            r#"{{"group_id":{},"command":["#,
            serde_json::to_string(LEGACY_GROUP_ID).expect("the group id serializes"),
        ))
        .send()
        .await
        .expect("the malformed follower request reaches Tape");
    assert_follower_leader_hint(malformed, leader, "malformed JSON").await;

    let missing_content_type = c
        .client
        .post(&follower_publish)
        .body(
            serde_json::to_vec(&publish_envelope(LEGACY_GROUP_ID, 97))
                .expect("the missing-content-type envelope serializes"),
        )
        .send()
        .await
        .expect("the content-type-free follower request reaches Tape");
    assert_follower_leader_hint(
        missing_content_type,
        leader,
        "valid JSON without Content-Type",
    )
    .await;

    let valid = c
        .client
        .post(&follower_publish)
        .json(&publish_envelope(LEGACY_GROUP_ID, 98))
        .send()
        .await
        .expect("the valid follower request reaches Tape");
    assert_follower_leader_hint(valid, leader, "valid local-group publish").await;

    // A well-formed request for another group is input hardening, not a
    // redirect: it must not reveal this group leader to a foreign caller.
    let foreign_group = format!("{LEGACY_GROUP_ID}-foreign");
    let wrong_group = c
        .client
        .post(&follower_publish)
        .json(&publish_envelope(&foreign_group, 99))
        .send()
        .await
        .expect("the foreign-group follower request reaches Tape");
    let wrong_group_status = wrong_group.status();
    let wrong_group_body = wrong_group
        .text()
        .await
        .expect("the foreign-group response body is readable");
    assert_eq!(
        wrong_group_status,
        reqwest::StatusCode::BAD_REQUEST,
        "a well-formed foreign group must be rejected before follower routing; body: {wrong_group_body}",
    );
    assert!(
        !wrong_group_body.contains("leader"),
        "a foreign group response must not disclose a leader hint; body: {wrong_group_body}",
    );
    assert!(
        c.live().all(|(id, _)| c.payload_ns(id).is_empty()),
        "direct follower publish attempts must not commit an event",
    );

    // A valid peer publish to the elected leader commits through the same
    // Tape router and becomes visible on every member.
    let leader_publish = c
        .client
        .post(format!("{}/raft/publish", c.urls[&(leader as u64)]))
        .json(&publish_envelope(LEGACY_GROUP_ID, 1))
        .send()
        .await
        .expect("the leader publish reaches Tape");
    let leader_publish_status = leader_publish.status();
    let leader_publish_body: serde_json::Value = leader_publish
        .json()
        .await
        .expect("the leader publish response is JSON");
    assert_eq!(
        leader_publish_status,
        reqwest::StatusCode::OK,
        "the elected leader must commit a direct publish; body: {leader_publish_body}",
    );
    assert!(
        leader_publish_body
            .get("seq")
            .and_then(|seq| seq.as_u64())
            .is_some(),
        "the leader publish response must acknowledge its committed sequence: {leader_publish_body}",
    );
    c.wait_converged(1).await;
    c.wait_payloads(&[1]).await;

    // Follower append: the HOST forwards it to the leader and resolves once
    // the follower's own journal applied it.
    let out = propose(c.raft(follower), 2).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 2),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_converged(2).await;

    // Kill the leader -> survivors re-elect and keep the committed entries.
    c.kill(leader).await;
    let new_leader = c.wait_leader().await;
    assert_ne!(new_leader, leader);
    let out = propose(c.raft(new_leader), 3).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 3),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_converged(3).await;
    c.wait_payloads(&[1, 2, 3]).await;

    // Restart the original leader on the same durable raft directory. It must
    // recover/catch up before another leader is removed.
    c.start_node(leader, 1024);
    c.wait_payloads(&[1, 2, 3]).await;

    let second_leader = c.wait_leader().await;
    c.kill(second_leader).await;
    let third_leader = c.wait_leader().await;
    assert_ne!(third_leader, second_leader);
    let out = propose(c.raft(third_leader), 4).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 4),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_payloads(&[1, 2, 3, 4]).await;

    for (i, _) in c.live() {
        let got = c.payload_ns(i);
        for want in [1, 2, 3, 4] {
            assert!(got.contains(&want), "node {i} holds n={want}");
        }
    }
    c.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pull_subscription_metadata_and_explicit_ack_converge() {
    let _guard = CLUSTER_TEST_LOCK.lock().await;
    let mut c = Cluster::prepare(3).await;
    c.start_all(1024);
    let leader = c.wait_leader().await;
    let follower = c.live().map(|(id, _)| id).find(|id| *id != leader).unwrap();

    let (_, created) = c
        .raft(leader)
        .propose_subscription_create("orders".into(), "audit".into())
        .await
        .unwrap();
    assert!(matches!(
        created,
        Some(tape::raft::TapeOutcome::SubscriptionCreated(Ok(_)))
    ));
    propose(c.raft(leader), 1).await;
    propose(c.raft(leader), 2).await;
    c.wait_converged(2).await;

    for (_, node) in c.live() {
        let journal = node.raft.journal();
        let journal = journal.lock().unwrap();
        let batch = journal
            .pull_subscription("orders", "audit", Some(2))
            .unwrap();
        assert_eq!(batch.cursor, 0);
        assert_eq!(batch.next_offset, 2);
        assert_eq!(batch.events.len(), 2);
    }

    let (_, acked) = c
        .raft(follower)
        .propose_subscription_ack("orders".into(), "audit".into(), 2, 200)
        .await
        .unwrap();
    assert!(matches!(
        acked,
        Some(tape::raft::TapeOutcome::SubscriptionAcked(Ok(_)))
    ));
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if c.live().all(|(_, node)| {
            node.raft
                .journal()
                .lock()
                .unwrap()
                .checkpoint("orders", "audit")
                .is_some_and(|checkpoint| checkpoint.offset == 2)
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "subscription ack did not converge"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let (_, retained) = c
        .raft(leader)
        .propose_retention(
            "orders".into(),
            tape::RetentionPolicy {
                min_offset: Some(2),
                max_age_seconds: None,
                protected_consumers: vec!["audit".into()],
            },
            300,
        )
        .await
        .unwrap();
    assert!(matches!(
        retained,
        Some(tape::raft::TapeOutcome::RetentionUpdated(_))
    ));
    let appended = propose(c.raft(follower), 3).await;
    match appended {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.offset, 2),
        other => panic!("expected append after retention, got {other:?}"),
    }
    c.shutdown().await;
}

/// With a small SnapshotPolicy threshold the leader compacts its raft log, so
/// a node that starts late (empty journal, empty raft state) cannot be
/// caught up by AppendEntries alone -- it restores via InstallSnapshot (the
/// whole-journal dump) and then tails the remaining entries.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fresh_node_catches_up_via_install_snapshot() {
    let _guard = CLUSTER_TEST_LOCK.lock().await;
    let mut c = Cluster::prepare(3).await;
    // Node 2 stays down; 2 of 3 voters still form a quorum.
    c.leave_listener_unbound_until_start(2);
    c.start_node(0, 8);
    c.start_node(1, 8);
    let leader = c.wait_leader().await;

    for i in 0..20 {
        propose(c.raft(leader), i).await;
    }
    c.wait_converged(20).await;

    // Late joiner: fresh journal + fresh raft dir. The leader's log floor is
    // above index 1 (compaction every 8 applies), so catch-up REQUIRES an
    // InstallSnapshot round before AppendEntries can tail the rest.
    c.start_node(2, 8);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if c.raft(2).journal().lock().unwrap().end_offset("orders") >= 20 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "fresh node did not catch up via InstallSnapshot"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        c.raft(2).applied_index() >= 20,
        "snapshot restore set the applied floor"
    );
    let got = c.payload_ns(2);
    for i in 0..20 {
        assert!(got.contains(&i), "fresh node holds n={i}");
    }
    c.shutdown().await;
}
// HANDWRITE-END

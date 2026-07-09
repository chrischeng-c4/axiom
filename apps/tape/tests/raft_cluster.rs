// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:0ca51618" tracker="pending-tracker" reason="An in-process 3-node TapeRaft group over real h2c listeners (relay's tests/raft_cluster.rs shape, adapted to tape's Append/CheckpointPut commands): exactly one leader; a leader append is applied and readable on every node's journal; a follower append is forwarded to the leader by the host; a direct follower POST to the host's peer route answers 421 not-leader; killing (aborting) the leader's task re-elects a survivor with no committed loss; a small SnapshotPolicy threshold compacts the leader's raft log so a late-started fresh node catches up via InstallSnapshot instead of full log replay."
//! raft-host cluster integration (#1327): 3 `TapeRaft` nodes over real h2c.
//!
//! Mirrors relay's `tests/raft_cluster.rs` (#544) shape: election, a leader
//! append applied on every node's journal, a follower append forwarded by
//! the host, a direct follower peer-route POST answering 421, leader kill
//! re-electing with no committed loss, and the snapshot/compaction path — a
//! fresh node catches up via InstallSnapshot instead of full log replay.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_host::Membership;
use tape::raft::TapeRaft;
use tape::TapeJournal;

struct Node {
    raft: Arc<TapeRaft>,
    serve: tokio::task::JoinHandle<()>,
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
        let listener = self.listeners[id].take().expect("listener unused");
        let serve = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        self.nodes[id] = Some(Node { raft, serve });
    }

    fn start_all(&mut self, snapshot_every: u64) {
        for id in 0..self.nodes.len() {
            self.start_node(id, snapshot_every);
        }
    }

    fn raft(&self, id: usize) -> &Arc<TapeRaft> {
        &self.nodes[id].as_ref().expect("node running").raft
    }

    /// Abort the node's serve loop and drop its host (tick/pump abort on
    /// drop) -- an in-process stand-in for a killed node; `raft_failover.rs`
    /// covers the real `kill -9` subprocess case.
    fn kill(&mut self, id: usize) {
        if let Some(n) = self.nodes[id].take() {
            n.serve.abort();
        }
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
        loop {
            if let Some(i) = self.leader().await {
                return i;
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

/// Exactly one leader; a leader append applies on every node's journal; a
/// follower append is forwarded by the host; a direct follower peer-route
/// POST answers 421 not-leader; killing the leader re-elects with no
/// committed loss and the group keeps accepting appends.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn three_node_group_elects_replicates_forwards_and_fails_over() {
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

    // Leader append: applied on ALL journals, outcome claimed (read-your-write).
    let out = propose(c.raft(leader), 1).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 1),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_converged(1).await;

    // Follower append: the HOST forwards it to the leader and resolves once
    // the follower's own journal applied it.
    let follower = c.live().map(|(i, _)| i).find(|i| *i != leader).unwrap();
    let out = propose(c.raft(follower), 2).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 2),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_converged(2).await;

    // A direct POST to a follower's raft peer route answers 421 + leader hint.
    let resp = c
        .client
        .post(format!("{}/raft/publish", c.urls[&(follower as u64)]))
        .body(serde_json::to_vec(&tape::raft::TapeCommand::Append {
            topic: "orders".to_string(),
            key: None,
            payload: serde_json::json!({ "n": 99 }),
            timestamp_ms: 100,
        })
        .unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::MISDIRECTED_REQUEST);

    // Kill the leader -> survivors re-elect and keep the committed entries.
    c.kill(leader);
    let new_leader = c.wait_leader().await;
    assert_ne!(new_leader, leader);
    let out = propose(c.raft(new_leader), 3).await;
    match out {
        tape::raft::TapeOutcome::Appended(event) => assert_eq!(event.payload["n"], 3),
        _ => panic!("expected Appended outcome"),
    }
    c.wait_converged(3).await;

    for (i, _) in c.live() {
        let got = c.payload_ns(i);
        for want in [1, 2, 3] {
            assert!(got.contains(&want), "node {i} holds n={want}");
        }
    }
}

/// With a small SnapshotPolicy threshold the leader compacts its raft log, so
/// a node that starts late (empty journal, empty raft state) cannot be
/// caught up by AppendEntries alone -- it restores via InstallSnapshot (the
/// whole-journal dump) and then tails the remaining entries.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fresh_node_catches_up_via_install_snapshot() {
    let mut c = Cluster::prepare(3).await;
    // Node 2 stays down; 2 of 3 voters still form a quorum.
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
}
// HANDWRITE-END

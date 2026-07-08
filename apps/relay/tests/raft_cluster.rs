// SPEC-MANAGED: apps/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:e331737f" tracker="pending-tracker" reason="raft-host adoption cluster tests (#544): an in-process 3-node RelayRaft group over real h2c listeners — election, leader publish applied on every engine, follower publish forwarded by the host, direct follower /raft/publish answers 421, leader kill re-elects with no committed loss; and the snapshot path — a small SnapshotPolicy threshold compacts the leader log so a late-started fresh node catches up via InstallSnapshot."
//! raft-host cluster integration (#544): 3 `RelayRaft` nodes over real h2c.
//!
//! Replaces the old hand-rolled-driver test: the group now runs the shared
//! [`raft_host::RaftHost`] (relay supplies only [`relay::RelayStateMachine`]),
//! publishes are multi-subject [`PubCommand`]s, a follower publish is
//! *forwarded* to the leader by the host (the old driver only redirected), and
//! the snapshot/compaction path is real — a fresh node catches up via
//! InstallSnapshot instead of full log replay.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_host::Membership;
use relay::{PubCommand, Relay, RelayCoreConfig, RelayRaft};

fn cmd(id: &str) -> PubCommand {
    PubCommand {
        subject: "s".to_string(),
        message_id: id.to_string(),
        payload: serde_json::json!({ "m": id }),
        headers: Default::default(),
        priority: relay::DEFAULT_PRIORITY,
        not_before: None,
    }
}

struct Node {
    raft: Arc<RelayRaft>,
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

    /// Spawn node `id` (RAM engine, its own raft dir) and serve its peer
    /// router on the pre-bound listener.
    fn start_node(&mut self, id: usize, snapshot_every: u64) {
        let engine = Arc::new(Relay::new(RelayCoreConfig::in_memory()));
        let peers: HashMap<u64, String> = self
            .urls
            .iter()
            .filter(|(k, _)| **k != id as u64)
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        let raft = Arc::new(
            RelayRaft::spawn(
                engine,
                &self._dirs[id].path().join("raft"),
                id as u64,
                self.membership.clone(),
                peers,
                RelayRaft::host_config(snapshot_every),
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

    fn raft(&self, id: usize) -> &Arc<RelayRaft> {
        &self.nodes[id].as_ref().expect("node running").raft
    }

    /// Abort the node's serve loop and drop its host (tick/pump abort on drop).
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

    /// Wait until every LIVE node's engine holds at least `want` entries.
    async fn wait_converged(&self, want: u64) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let ok = self
                .live()
                .all(|(_, n)| n.raft.relay().log_len("s").unwrap() >= want);
            if ok {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "engines did not converge to {want}"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    fn message_ids(&self, id: usize) -> Vec<String> {
        let r = self.raft(id).relay();
        let len = r.log_len("s").unwrap();
        (0..len)
            .filter_map(|seq| r.entry("s", 0, seq).unwrap().map(|e| e.message_id))
            .collect()
    }
}

/// AC2 + failover: exactly one leader; a leader publish applies on every
/// node's engine; a follower publish is forwarded by the host; a direct
/// follower `/raft/publish` answers 421 not-leader; the raft path stays
/// idempotent per message_id; killing the leader re-elects with no committed
/// loss and the group keeps accepting publishes.
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

    // Leader publish: applied on ALL engines, outcome claimed (read-your-write).
    let (_, outcome) = c.raft(leader).publish(&cmd("a")).await.unwrap();
    let outcome = outcome.expect("leader claims its apply outcome");
    assert!(!outcome.deduped);
    c.wait_converged(1).await;

    // The raft path is idempotent per message_id: re-publishing "a" dedupes.
    let (_, dup) = c.raft(leader).publish(&cmd("a")).await.unwrap();
    assert!(dup.expect("outcome").deduped, "duplicate publish dedupes");

    // Follower publish: the HOST forwards it to the leader and resolves once
    // the follower's own engine applied it.
    let follower = c.live().map(|(i, _)| i).find(|i| *i != leader).unwrap();
    let (_, fw) = c.raft(follower).publish(&cmd("b")).await.unwrap();
    assert!(!fw.expect("follower claims its apply outcome").deduped);
    c.wait_converged(2).await;

    // A direct POST to a follower's /raft/publish answers 421 + leader hint.
    let resp = c
        .client
        .post(format!("{}/raft/publish", c.urls[&(follower as u64)]))
        .body(serde_json::to_vec(&cmd("x")).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::MISDIRECTED_REQUEST);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "not-leader");

    // Kill the leader -> survivors re-elect and keep the committed entries.
    c.kill(leader);
    let new_leader = c.wait_leader().await;
    assert_ne!(new_leader, leader);
    let (_, out) = c.raft(new_leader).publish(&cmd("c")).await.unwrap();
    assert!(!out.expect("outcome").deduped);
    c.wait_converged(3).await;

    for (i, _) in c.live() {
        let got = c.message_ids(i);
        for want in ["a", "b", "c"] {
            assert!(got.contains(&want.to_string()), "node {i} holds '{want}'");
        }
    }
}

/// AC4: with a small SnapshotPolicy threshold the leader compacts its raft
/// log, so a node that starts late (empty engine, empty raft state) cannot be
/// caught up by AppendEntries alone — it restores via InstallSnapshot (the
/// live un-acked engine dump) and then tails the remaining entries.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn fresh_node_catches_up_via_install_snapshot() {
    let mut c = Cluster::prepare(3).await;
    // Node 2 stays down; 2 of 3 voters still form a quorum.
    c.start_node(0, 8);
    c.start_node(1, 8);
    let leader = c.wait_leader().await;

    for i in 0..20 {
        let (_, out) = c
            .raft(leader)
            .publish(&cmd(&format!("m{i}")))
            .await
            .unwrap();
        assert!(!out.expect("outcome").deduped);
    }
    c.wait_converged(20).await;

    // Late joiner: fresh engine + fresh raft dir. The leader's log floor is
    // above index 1 (compaction every 8 applies), so catch-up REQUIRES an
    // InstallSnapshot round before AppendEntries can tail the rest.
    c.start_node(2, 8);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if c.raft(2).relay().log_len("s").unwrap() >= 20 {
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
    let got = c.message_ids(2);
    for i in 0..20 {
        assert!(got.contains(&format!("m{i}")), "fresh node holds m{i}");
    }
}
// HANDWRITE-END

// SPEC-MANAGED: apps/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:e331737f" tracker="pending-tracker" reason="raft-runtime adoption cluster tests (#544): an in-process 3-node RelayRaft group over real h2c listeners — election, leader publish applied on every engine, follower publish forwarded by the host, direct follower /raft/publish answers 421, leader kill re-elects with no committed loss; and the snapshot path — a small SnapshotPolicy threshold compacts the leader log so a late-started fresh node catches up via InstallSnapshot."
//! raft-runtime cluster integration (#544): 3 `RelayRaft` nodes over real h2c.
//!
//! Replaces the old hand-rolled-driver test: the group now runs the shared
//! [`raft_runtime::RaftHost`] (relay supplies only [`relay::RelayStateMachine`]),
//! publishes are multi-subject [`PubCommand`]s, a follower publish is
//! *forwarded* to the leader by the host (the old driver only redirected), and
//! a recovered node catches up before a second leader loss, and the
//! snapshot/compaction path is real — a fresh node catches up via
//! InstallSnapshot instead of full log replay.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use raft_runtime::Membership;
use relay::{PubCommand, Relay, RelayCoreConfig, RelayRaft};

static CLUSTER_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn cmd(id: &str) -> PubCommand {
    PubCommand {
        subject: "s".to_string(),
        message_id: id.to_string(),
        payload: serde_json::json!({ "m": id }),
        headers: Default::default(),
        priority: relay::DEFAULT_PRIORITY,
        not_before: None,
        appended_at: chrono::Utc::now(),
    }
}

struct Node {
    raft: Arc<RelayRaft>,
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

    /// Spawn node `id` (durable engine + raft state in its node directory) and
    /// serve its peer router on the pre-bound listener.
    fn start_node(&mut self, id: usize, snapshot_every: u64) {
        let engine = Arc::new(Relay::new(RelayCoreConfig {
            data_dir: self._dirs[id]
                .path()
                .join("relay")
                .to_string_lossy()
                .into_owned(),
            ..RelayCoreConfig::default()
        }));
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
        let listener = match self.listeners[id].take() {
            Some(listener) => listener,
            None => {
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

    fn raft(&self, id: usize) -> &Arc<RelayRaft> {
        &self.nodes[id].as_ref().expect("node running").raft
    }

    /// Abort the node's serve loop and drop its host (tick/pump abort on drop).
    async fn kill(&mut self, id: usize) {
        if let Some(mut n) = self.nodes[id].take() {
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
    c.kill(leader).await;
    let new_leader = c.wait_leader().await;
    assert_ne!(new_leader, leader);
    let (_, out) = c.raft(new_leader).publish(&cmd("c")).await.unwrap();
    assert!(!out.expect("outcome").deduped);
    c.wait_converged(3).await;

    // Recover the first leader from its durable raft directory, prove that it
    // catches up, then remove the current leader and commit through the next
    // elected primary.
    c.start_node(leader, 1024);
    c.wait_converged(3).await;
    for want in ["a", "b", "c"] {
        assert!(
            c.message_ids(leader).contains(&want.to_string()),
            "recovered node holds '{want}'"
        );
    }
    let second_leader = c.wait_leader().await;
    c.kill(second_leader).await;
    let third_leader = c.wait_leader().await;
    assert_ne!(third_leader, second_leader);
    let (_, out) = c.raft(third_leader).publish(&cmd("d")).await.unwrap();
    assert!(!out.expect("outcome").deduped);
    c.wait_converged(4).await;

    for (i, _) in c.live() {
        let got = c.message_ids(i);
        for want in ["a", "b", "c", "d"] {
            assert!(got.contains(&want.to_string()), "node {i} holds '{want}'");
        }
    }
    c.shutdown().await;
}

/// Relay's effectful delivery lifecycle is authoritative Raft state: a lease
/// granted through one follower is visible everywhere, cannot be leased again,
/// and can only be completed by the executor replica holding its fencing token.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn lease_and_ack_are_committed_and_executor_fenced() {
    let _guard = CLUSTER_TEST_LOCK.lock().await;
    let mut c = Cluster::prepare(3).await;
    c.start_all(1024);
    let leader = c.wait_leader().await;
    let executor = c.live().map(|(i, _)| i).find(|i| *i != leader).unwrap();
    let other = c.live().map(|(i, _)| i).find(|i| *i != executor).unwrap();

    c.raft(leader).publish(&cmd("owned")).await.unwrap();
    c.wait_converged(1).await;
    let lease = c
        .raft(executor)
        .lease("s".into(), "worker-a".into(), chrono::Utc::now())
        .await
        .unwrap()
        .expect("committed lease");
    assert_eq!(lease.executor_node, executor as u64);

    // The next committed lease command observes the first one on every state
    // machine, so no second replica can assign the same work.
    let conflicting = c
        .raft(other)
        .lease("s".into(), "worker-b".into(), chrono::Utc::now())
        .await
        .unwrap();
    assert!(conflicting.is_none(), "no cross-replica dual lease");

    // Correct epoch but wrong executor is fenced by committed owner identity.
    let (accepted, _) = c
        .raft(other)
        .ack(
            "s".into(),
            lease.lease_id.clone(),
            lease.epoch,
            chrono::Utc::now(),
        )
        .await
        .unwrap();
    assert!(!accepted, "non-owner executor cannot acknowledge");

    let (accepted, committed) = c
        .raft(executor)
        .ack("s".into(), lease.lease_id, lease.epoch, chrono::Utc::now())
        .await
        .unwrap();
    assert!(accepted);
    assert_eq!(committed.expect("watermark").committed_seq, 0);

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if c.live().all(|(_, n)| {
            n.raft
                .relay()
                .committed_offset("s")
                .unwrap()
                .is_some_and(|offset| offset.committed_seq == 0)
        }) {
            break;
        }
        assert!(Instant::now() < deadline, "committed ack did not converge");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    c.shutdown().await;
}

/// AC4: with a small SnapshotPolicy threshold the leader compacts its raft
/// log, so a node that starts late (empty engine, empty raft state) cannot be
/// caught up by AppendEntries alone — it restores via InstallSnapshot (the
/// live un-acked engine dump) and then tails the remaining entries.
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
    c.shutdown().await;
}
// HANDWRITE-END

// SPEC-MANAGED: projects/relay/tech-design/logic/h2c-raft-transport-driver-producer-redirect-to-leader.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:e331737f" tracker="pending-tracker" reason="Real-h2c integration: 3 relay-raft servers on ephemeral ports with wired peers. Assert one leader elected, a publish to the leader converges on all three engines, a publish to a follower returns NotLeader+hint, and killing the leader re-elects with no committed loss + accepts new publishes."
//! Raft driver over real h2c (#138): 3 relay-raft servers elect a leader,
//! a publish to the leader converges on every engine, a publish to a follower is
//! redirected, and killing the leader re-elects without losing committed data.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use relay::{auto_membership, FsyncPolicy, RaftDriver, RaftStore, Relay, RelayCoreConfig};

struct Node {
    driver: RaftDriver,
    serve: tokio::task::JoinHandle<()>,
    url: String,
    killed: bool,
}

struct Cluster {
    nodes: Vec<Node>,
    _dirs: Vec<tempfile::TempDir>,
    client: reqwest::Client,
}

impl Cluster {
    async fn start(n: u64) -> Cluster {
        // Bind first so peer URLs are known before the drivers start.
        let mut listeners = Vec::new();
        let mut urls = HashMap::new();
        for id in 0..n {
            let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            urls.insert(id, format!("http://{}", l.local_addr().unwrap()));
            listeners.push(l);
        }
        let membership = auto_membership(n);
        let mut nodes = Vec::new();
        let mut dirs = Vec::new();
        for (id, listener) in listeners.into_iter().enumerate() {
            let id = id as u64;
            let dir = tempfile::tempdir().unwrap();
            let store =
                RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Always).unwrap();
            let peers: HashMap<u64, String> = urls
                .iter()
                .filter(|(k, _)| **k != id)
                .map(|(k, v)| (*k, v.clone()))
                .collect();
            let relay = Arc::new(Relay::new(RelayCoreConfig::in_memory()));
            let driver = RaftDriver::spawn(id, membership.clone(), "s", peers, relay, store);
            let app = driver.router();
            let serve = tokio::spawn(async move {
                let _ = axum::serve(listener, app).await;
            });
            nodes.push(Node {
                driver,
                serve,
                url: urls[&id].clone(),
                killed: false,
            });
            dirs.push(dir);
        }
        let client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        Cluster {
            nodes,
            _dirs: dirs,
            client,
        }
    }

    async fn leader(&self) -> Option<usize> {
        for (i, n) in self.nodes.iter().enumerate() {
            if !n.killed && n.driver.is_leader().await {
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

    async fn publish(&self, idx: usize, mid: &str) -> reqwest::StatusCode {
        self.client
            .post(format!("{}/v1/s/publish", self.nodes[idx].url))
            .json(&serde_json::json!({ "message_id": mid, "payload": { "m": mid } }))
            .send()
            .await
            .unwrap()
            .status()
    }

    fn first_follower(&self, leader: usize) -> usize {
        (0..self.nodes.len())
            .find(|i| *i != leader && !self.nodes[*i].killed)
            .unwrap()
    }

    async fn wait_converged(&self, want: u64) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let ok = self
                .nodes
                .iter()
                .filter(|n| !n.killed)
                .all(|n| n.driver.relay().log_len("s").unwrap() >= want);
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

    fn kill(&mut self, idx: usize) {
        self.nodes[idx].driver.stop();
        self.nodes[idx].serve.abort();
        self.nodes[idx].killed = true;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn raft_cluster_elects_replicates_redirects_and_fails_over() {
    let mut c = Cluster::start(3).await;

    // Exactly one leader.
    let leader = c.wait_leader().await;
    let mut leaders = 0;
    for n in &c.nodes {
        if n.driver.is_leader().await {
            leaders += 1;
        }
    }
    assert_eq!(leaders, 1, "exactly one leader");

    // Publish to the leader replicates to every engine.
    assert_eq!(c.publish(leader, "a").await, reqwest::StatusCode::OK);
    c.wait_converged(1).await;

    // Publish to a follower is redirected with a leader hint.
    let follower = c.first_follower(leader);
    let resp = c
        .client
        .post(format!("{}/v1/s/publish", c.nodes[follower].url))
        .json(&serde_json::json!({ "message_id": "x", "payload": {} }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::MISDIRECTED_REQUEST);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "not-leader");
    assert!(body["leader"].is_number(), "carries a leader hint");

    // Kill the leader -> survivors re-elect and keep the committed entry.
    c.kill(leader);
    let new_leader = c.wait_leader().await;
    assert_ne!(new_leader, leader);

    // New publishes commit on the survivors; the old "a" is still there.
    assert_eq!(c.publish(new_leader, "b").await, reqwest::StatusCode::OK);
    c.wait_converged(2).await;

    for n in c.nodes.iter().filter(|n| !n.killed) {
        let got: Vec<String> = {
            let r = n.driver.relay();
            r.subscribe("s", "check", 0).unwrap();
            r.poll("s", "check")
                .unwrap()
                .into_iter()
                .map(|e| e.message_id)
                .collect()
        };
        assert!(got.contains(&"a".to_string()), "no committed loss");
        assert!(got.contains(&"b".to_string()), "new entry replicated");
    }
}
// HANDWRITE-END

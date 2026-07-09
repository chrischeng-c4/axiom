// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:2f55cb8a" tracker="pending-tracker" reason="Live 3-node kill -9 failover test: spawns three real `tape` OS subprocesses (REPLICAS_PER_SHARD=3, TAPE_PEERS local override, distinct --data-dir/--bind per node), waits for a leader, appends events through it, SIGKILLs (not SIGTERM) the leader's process, waits for the survivors to re-elect and keep accepting appends, then asserts every previously committed event is still replayable on every surviving node -- proving no committed event loss across a real process crash, not just an in-process task abort."
//! Live 3-node `kill -9` failover (#1327): three real `tape` OS processes,
//! not in-process task aborts (`tests/raft_cluster.rs` covers the
//! in-process shape). This is the requirement's load-bearing proof: a real
//! process crash of the raft leader must not lose any event a client was
//! already told was committed.

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Node {
    child: Child,
    bind: String,
}

impl Node {
    fn base_url(&self) -> String {
        format!("http://{}", self.bind)
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // Best-effort cleanup for whichever nodes the test didn't already
        // kill -9 itself.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Bind an ephemeral port and immediately release it for the child process
/// to rebind -- a small, accepted race in this style of subprocess test.
fn free_addr() -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    format!("{}", l.local_addr().unwrap())
}

fn spawn_node(id: u32, bind: &str, data_dir: &std::path::Path, peers_csv: &str) -> Node {
    let child = Command::new(env!("CARGO_BIN_EXE_tape"))
        .arg("serve")
        .arg("--bind")
        .arg(bind)
        .arg("--data-dir")
        .arg(data_dir)
        .env("REPLICAS_PER_SHARD", "3")
        .env("SHARD_COUNT", "1")
        .env("VOTER_COUNT", "3")
        .env("POD_NAME", format!("tape-{id}"))
        .env("TAPE_PEER_SERVICE", "tape")
        .env("TAPE_PEERS", peers_csv)
        .env("TAPE_AUTH", "off")
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn tape serve subprocess");
    Node {
        child,
        bind: bind.to_string(),
    }
}

async fn wait_healthy(client: &reqwest::Client, base: &str, deadline: Instant) {
    loop {
        if let Ok(resp) = client.get(format!("{base}/healthz")).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        assert!(Instant::now() < deadline, "{base} never became healthy");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn is_leader(client: &reqwest::Client, base: &str) -> bool {
    let Ok(resp) = client.get(format!("{base}/raftz")).send().await else {
        return false;
    };
    let Ok(body) = resp.json::<serde_json::Value>().await else {
        return false;
    };
    body["is_leader"].as_bool().unwrap_or(false)
}

async fn wait_leader<'a>(client: &reqwest::Client, bases: &[&'a str], deadline: Instant) -> &'a str {
    loop {
        for base in bases {
            if is_leader(client, base).await {
                return base;
            }
        }
        assert!(Instant::now() < deadline, "no leader elected among {bases:?}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn append(client: &reqwest::Client, base: &str, n: i64) -> reqwest::Response {
    client
        .post(format!("{base}/topics/orders/append"))
        .json(&serde_json::json!({ "payload": { "n": n } }))
        .send()
        .await
        .unwrap()
}

async fn replayed_ns(client: &reqwest::Client, base: &str) -> Vec<i64> {
    let resp = client
        .get(format!("{base}/topics/orders/replay"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "replay failed on {base}");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["payload"]["n"].as_i64().unwrap())
        .collect()
}

/// Kill the leader with SIGKILL (`kill -9`, not a graceful SIGTERM the
/// process could drain/checkpoint against) via the real `kill` system
/// command -- proving the durable applied-marker + journal-snapshot recovery
/// path (not a graceful-shutdown code path) is what protects committed data.
fn kill_9(pid: u32) {
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .expect("run kill -9");
    assert!(status.success(), "kill -9 {pid} failed to run");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn kill_9_leader_survivors_reelect_with_no_committed_event_loss() {
    let binds: Vec<String> = (0..3).map(|_| free_addr()).collect();
    let peers_csv = binds.join(",");
    let dirs: Vec<tempfile::TempDir> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();

    let mut nodes: Vec<Node> = (0..3u32)
        .map(|i| spawn_node(i, &binds[i as usize], dirs[i as usize].path(), &peers_csv))
        .collect();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(20);
    for n in &nodes {
        wait_healthy(&client, &n.base_url(), deadline).await;
    }

    let base_urls: Vec<String> = nodes.iter().map(Node::base_url).collect();
    let base_refs: Vec<&str> = base_urls.iter().map(String::as_str).collect();
    let leader_deadline = Instant::now() + Duration::from_secs(15);
    let leader_url = wait_leader(&client, &base_refs, leader_deadline)
        .await
        .to_string();

    // Commit two events through the leader before the crash.
    for n in [1, 2] {
        let resp = append(&client, &leader_url, n).await;
        assert!(resp.status().is_success(), "append {n} failed pre-crash");
    }

    // Find the leader's node so we can SIGKILL its actual OS process (not an
    // in-process task abort).
    let leader_idx = nodes
        .iter()
        .position(|n| n.base_url() == leader_url)
        .expect("leader is one of our nodes");
    let leader_pid = nodes[leader_idx].child.id();
    kill_9(leader_pid);
    // Reap so the OS doesn't hand back a lingering zombie status; the
    // process is already dead from SIGKILL, this just collects the exit.
    let _ = nodes[leader_idx].child.wait();

    let survivor_urls: Vec<String> = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != leader_idx)
        .map(|(_, n)| n.base_url())
        .collect();
    let survivor_refs: Vec<&str> = survivor_urls.iter().map(String::as_str).collect();

    // Survivors still form a quorum (2 of 3 voters) and re-elect.
    let reelect_deadline = Instant::now() + Duration::from_secs(15);
    let new_leader = wait_leader(&client, &survivor_refs, reelect_deadline)
        .await
        .to_string();
    assert_ne!(new_leader, leader_url, "a survivor took over leadership");

    // The new leader keeps accepting appends post-crash.
    let resp = append(&client, &new_leader, 3).await;
    assert!(resp.status().is_success(), "append 3 failed post-crash");

    // No committed event loss: every survivor eventually replays all three
    // events, including the two committed before the crash.
    let converge_deadline = Instant::now() + Duration::from_secs(15);
    for base in &survivor_urls {
        loop {
            let got = replayed_ns(&client, base).await;
            let has_all = [1, 2, 3].iter().all(|n| got.contains(n));
            if has_all {
                break;
            }
            assert!(
                Instant::now() < converge_deadline,
                "{base} never converged to all pre- and post-crash events, got {got:?}"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    // Clean up survivor processes; `Node::drop` also best-effort kills, but
    // reaping explicitly here avoids the test process outliving them.
    for (i, mut n) in nodes.into_iter().enumerate() {
        if i != leader_idx {
            let _ = n.child.kill();
            let _ = n.child.wait();
        }
    }
}
// HANDWRITE-END

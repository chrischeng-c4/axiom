// HANDWRITE-BEGIN gap="missing-generator:unit-test:2f55cb8a" tracker="pending-tracker" reason="Live 3-node kill -9 failover test: spawns three real `tape` OS subprocesses (REPLICAS_PER_SHARD=3, TAPE_PEERS local override, distinct --data-dir/--bind per node), waits for a leader, appends events through it, SIGKILLs (not SIGTERM) the leader's process, waits for the survivors to re-elect and keep accepting appends, then asserts every previously committed event is still replayable on every surviving node -- proving no committed event loss across a real process crash, not just an in-process task abort."
//! Live 3-node `kill -9` failover (#1327): three real `tape` OS processes,
//! not in-process task aborts (`e2e/raft_cluster.rs` covers the
//! in-process shape). This is the requirement's load-bearing proof: a real
//! process crash of the raft leader must not lose any event a client was
//! already told was committed.

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use futures::{StreamExt, TryStreamExt};

// Each observed state transition receives this budget when its wait starts.
// Never pass an earlier phase's deadline into a later health, election, or
// replication wait: that would make a slow earlier phase hide the later one.
const PHASE_BUDGET: Duration = Duration::from_secs(12);
const REQUEST_BUDGET: Duration = Duration::from_secs(12);
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const REPLAY_POLL_INTERVAL: Duration = Duration::from_millis(200);

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
    let child_log = std::env::var("TAPE_CHILD_RUST_LOG").unwrap_or_else(|_| "warn".into());
    let stderr = if std::env::var_os("TAPE_TEST_LOG").is_some() {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
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
        .env("RUST_LOG", child_log)
        .stdout(Stdio::null())
        .stderr(stderr)
        .spawn()
        .expect("spawn tape serve subprocess");
    Node {
        child,
        bind: bind.to_string(),
    }
}

fn phase_deadline() -> Instant {
    Instant::now() + PHASE_BUDGET
}

async fn wait_healthy(client: &reqwest::Client, base: &str, phase: &str) {
    let deadline = phase_deadline();
    loop {
        let last_status = match client.get(format!("{base}/healthz")).send().await {
            Ok(resp) if resp.status().is_success() => return,
            Ok(resp) => resp.status().to_string(),
            Err(error) => format!("request error: {error}"),
        };
        assert!(
            Instant::now() < deadline,
            "{phase}: {base} never became healthy; last_status={}",
            last_status,
        );
        tokio::time::sleep(POLL_INTERVAL).await;
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

/// One observation rejects a split-brain sample (more than one leader). The
/// caller requires three identical whole-cluster observations before it uses
/// a leader as a routing target.
async fn observed_leader<'a>(client: &reqwest::Client, bases: &[&'a str]) -> Option<&'a str> {
    let mut leader = None;
    for base in bases {
        if is_leader(client, base).await {
            if leader.is_some() {
                return None;
            }
            leader = Some(*base);
        }
    }
    leader
}

async fn wait_leader<'a>(client: &reqwest::Client, bases: &[&'a str], phase: &str) -> &'a str {
    let deadline = phase_deadline();
    let mut stable = None;
    let mut samples = 0;
    loop {
        match observed_leader(client, bases).await {
            Some(base) if stable == Some(base) => samples += 1,
            Some(base) => {
                stable = Some(base);
                samples = 1;
            }
            None => {
                stable = None;
                samples = 0;
            }
        }
        if samples >= 3 {
            return stable.expect("a stable leader has a base URL");
        }
        assert!(
            Instant::now() < deadline,
            "{phase}: no stable leader after {samples} consecutive samples; last_observed={stable:?}; candidates={bases:?}",
        );
        tokio::time::sleep(POLL_INTERVAL).await;
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

async fn replayed_ns(client: &reqwest::Client, base: &str) -> Result<Vec<i64>, String> {
    let resp = client
        .get(format!("{base}/topics/orders/replay"))
        .send()
        .await
        .map_err(|error| format!("replay request error: {error}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("replay status: {status}"));
    }
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|error| format!("replay JSON error: {error}"))?;
    let events = body["events"]
        .as_array()
        .ok_or_else(|| "replay response has no events array".to_string())?;
    events
        .iter()
        .map(|event| {
            event["payload"]["n"]
                .as_i64()
                .ok_or_else(|| "replay event has no integer payload.n".to_string())
        })
        .collect()
}

fn event_diagnostic(events: &[i64]) -> String {
    format!(
        "observed_count={}, first={:?}, last={:?}",
        events.len(),
        events.first(),
        events.last(),
    )
}

/// Poll one replica until it holds every expected event. A replica starts
/// with its own budget, so an earlier replica's catch-up cannot spend time
/// from a later one.
async fn wait_replayed(client: &reqwest::Client, base: &str, expected: &[i64], phase: &str) {
    let deadline = phase_deadline();
    let mut observed = Vec::new();
    loop {
        let last_error = match replayed_ns(client, base).await {
            Ok(events) => {
                let has_all = expected.iter().all(|event| events.contains(event));
                observed = events;
                if has_all {
                    return;
                }
                None
            }
            Err(error) => Some(error),
        };
        assert!(
            Instant::now() < deadline,
            "{phase}: {base} did not converge; expected_count={}, {}, last_error={}",
            expected.len(),
            event_diagnostic(&observed),
            last_error.as_deref().unwrap_or("none"),
        );
        tokio::time::sleep(REPLAY_POLL_INTERVAL).await;
    }
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
        .timeout(REQUEST_BUDGET)
        .build()
        .unwrap();

    for n in &nodes {
        let base = n.base_url();
        wait_healthy(&client, &base, "pre-crash health").await;
    }

    let base_urls: Vec<String> = nodes.iter().map(Node::base_url).collect();
    let base_refs: Vec<&str> = base_urls.iter().map(String::as_str).collect();
    let leader_url = wait_leader(&client, &base_refs, "initial election")
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
    let new_leader = wait_leader(&client, &survivor_refs, "post-kill re-election")
        .await
        .to_string();
    assert_ne!(new_leader, leader_url, "a survivor took over leadership");

    // The new leader keeps accepting appends post-crash.
    let resp = append(&client, &new_leader, 3).await;
    assert!(resp.status().is_success(), "append 3 failed post-crash");

    // No committed event loss: every survivor eventually replays all three
    // events, including the two committed before the crash.
    for base in &survivor_urls {
        wait_replayed(
            &client,
            base,
            &[1, 2, 3],
            "post-kill replication convergence",
        )
        .await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_ingress_across_all_replicas_commits_without_raft_timeouts() {
    const EVENTS: usize = 256;
    const CONCURRENCY: usize = 64;

    let binds: Vec<String> = (0..3).map(|_| free_addr()).collect();
    let peers_csv = binds.join(",");
    let dirs: Vec<tempfile::TempDir> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let nodes: Vec<Node> = (0..3u32)
        .map(|i| spawn_node(i, &binds[i as usize], dirs[i as usize].path(), &peers_csv))
        .collect();
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(REQUEST_BUDGET)
        .build()
        .unwrap();

    for node in &nodes {
        let base = node.base_url();
        wait_healthy(&client, &base, "concurrent-ingress health").await;
    }
    let base_urls: Vec<String> = nodes.iter().map(Node::base_url).collect();
    let base_refs: Vec<&str> = base_urls.iter().map(String::as_str).collect();
    wait_leader(&client, &base_refs, "concurrent-ingress election").await;

    let writes = futures::stream::iter(0..EVENTS)
        .map(|n| {
            let client = client.clone();
            let base = base_urls[n % base_urls.len()].clone();
            async move {
                let response = append(&client, &base, n as i64).await;
                let status = response.status();
                anyhow::ensure!(status.is_success(), "append {n} via {base}: {status}");
                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(CONCURRENCY)
        .try_collect::<Vec<_>>()
        .await;
    if let Err(error) = writes {
        for base in &base_urls {
            match replayed_ns(&client, base).await {
                Ok(events) => eprintln!(
                    "phase=concurrent-ingress append failure base={base} {}",
                    event_diagnostic(&events),
                ),
                Err(replay_error) => eprintln!(
                    "phase=concurrent-ingress append failure base={base} {} replay_error={replay_error}",
                    event_diagnostic(&[]),
                ),
            };
        }
        panic!("phase=concurrent-ingress append failure: {error:#}");
    }

    let expected: Vec<i64> = (0..EVENTS).map(|event| event as i64).collect();
    for base in &base_urls {
        wait_replayed(
            &client,
            base,
            &expected,
            "concurrent-ingress replication convergence",
        )
        .await;
    }
}
// HANDWRITE-END

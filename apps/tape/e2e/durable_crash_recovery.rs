// HANDWRITE-BEGIN gap="missing-generator:unit-test:3052-durable-crash-recovery" tracker="#3052" reason="Real-process acceptance coverage for WI #3052's rebuilt single-node durable write path that genuinely needs a real `tape` OS subprocess rather than an in-process router: AC4 (SIGKILL-then-restart recovers every acknowledged append and never a duplicate) and AC3 (RSS/logical-payload ratio on the durable path stays within an order of magnitude of the in-memory path's 1.6x)."
//! Real-subprocess acceptance tests for WI #3052's single-node durable write
//! path. Spawns the actual `tape` binary (`env!("CARGO_BIN_EXE_tape")`) with
//! `serve --data-dir <dir>` -- the same single-node WAL wiring
//! `tape.rs::serve_main` uses in production (no `REPLICAS_PER_SHARD`, so
//! `resolve_journal_store` picks the WAL arm, not Raft or the legacy
//! whole-file store). See `e2e/durable_write_path.rs` for the other three
//! criteria (AC2, AC6, AC7), which run in-process instead.
//!
//! Skipped here (covered elsewhere, per the accepted dispatch): AC1
//! (throughput scaling -- another agent's dispatch) and AC5 (torn-tail
//! recovery -- already proved in `wal.rs`'s own unit tests).

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;

struct Node {
    child: Child,
    bind: String,
}

impl Node {
    fn base_url(&self) -> String {
        format!("http://{}", self.bind)
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // Best-effort cleanup; the test's own SIGKILL already reaps the
        // first node explicitly.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn free_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    format!("{}", listener.local_addr().unwrap())
}

/// Spawn a real single-node `tape serve --data-dir <dir>` process. No
/// `REPLICAS_PER_SHARD` env is set, so `resolve_journal_store` resolves to
/// the WAL group-commit arm (`tape.rs`'s `serve_main`), the exact path under
/// test.
fn spawn_single_node(bind: &str, data_dir: &std::path::Path) -> Node {
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
        .arg("--grace-secs")
        .arg("0")
        .env("TAPE_AUTH", "off")
        .stdout(Stdio::null())
        .stderr(stderr)
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
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Real `kill -9` (not a graceful SIGTERM the process could drain/checkpoint
/// against) via the `kill` system command -- proving the WAL/coordinator
/// recovery path, not a graceful-shutdown code path, protects committed data.
fn kill_9(pid: u32) {
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .expect("run kill -9");
    assert!(status.success(), "kill -9 {pid} failed to run");
}

const PRE_KILL_EVENTS: usize = 40;
const FINAL_BATCH: usize = 24;

/// AC4: a `SIGKILL`-then-restart on the SAME `--data-dir` recovers every
/// acknowledged append (exact payload) and never a duplicate, and never an
/// event for a request that returned an explicit error. A final-batch
/// request whose response never arrived (killed mid-flight) is intentionally
/// left unasserted either way -- see the comment at the bottom of this test.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn sigkill_then_restart_recovers_every_acked_append_and_never_duplicates() {
    let dir = tempfile::tempdir().unwrap();
    let bind1 = free_addr();
    let mut node1 = spawn_single_node(&bind1, dir.path());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    wait_healthy(
        &client,
        &node1.base_url(),
        Instant::now() + Duration::from_secs(20),
    )
    .await;

    let topic = "orders";

    // Sequential pre-kill appends: every one of these is fully round-tripped
    // (request sent, 200 received) before the crash, so every one of them
    // MUST survive recovery.
    let mut acked: Vec<(u64, serde_json::Value)> = Vec::new();
    for n in 0..PRE_KILL_EVENTS {
        let payload = json!({ "phase": "pre", "n": n });
        let resp = client
            .post(format!("{}/topics/{topic}/append", node1.base_url()))
            .json(&json!({ "payload": payload }))
            .send()
            .await
            .expect("pre-kill append must round-trip");
        assert!(resp.status().is_success(), "pre-kill append {n} failed");
        let event: serde_json::Value = resp.json().await.unwrap();
        acked.push((event["offset"].as_u64().unwrap(), payload));
    }

    // Final batch: fire all of it concurrently, then SIGKILL with no
    // synchronization wait -- the point is a crash while requests are
    // genuinely in flight, not one that waits for them to finish first.
    let base = node1.base_url();
    let handles: Vec<_> = (0..FINAL_BATCH)
        .map(|n| {
            let client = client.clone();
            let base = base.clone();
            let payload = json!({ "phase": "final", "n": n });
            tokio::spawn(async move {
                let result = client
                    .post(format!("{base}/topics/{topic}/append"))
                    .json(&json!({ "payload": payload }))
                    .send()
                    .await;
                (payload, result)
            })
        })
        .collect();

    let pid = node1.pid();
    kill_9(pid);
    let _ = node1.child.wait();

    let mut confirmed_final: Vec<(u64, serde_json::Value)> = Vec::new();
    let mut errored_final: Vec<serde_json::Value> = Vec::new();
    let mut undetermined = 0usize;
    for handle in handles {
        let (payload, result) = handle.await.unwrap();
        match result {
            Ok(resp) if resp.status().is_success() => {
                let event: serde_json::Value = resp.json().await.unwrap();
                confirmed_final.push((event["offset"].as_u64().unwrap(), payload));
            }
            Ok(_resp) => {
                // Explicit non-2xx: must never have been durably committed
                // (`AppState::apply_mutation` fails closed on a durability
                // error -- nothing is applied before the error is returned).
                errored_final.push(payload);
            }
            Err(_) => {
                // The connection never completed (killed mid-flight):
                // genuinely undetermined whether the server durably wrote it
                // before dying.
                undetermined += 1;
            }
        }
    }

    // Restart the SAME --data-dir on a fresh bind.
    let bind2 = free_addr();
    let node2 = spawn_single_node(&bind2, dir.path());
    wait_healthy(
        &client,
        &node2.base_url(),
        Instant::now() + Duration::from_secs(20),
    )
    .await;

    let resp = client
        .get(format!("{}/topics/{topic}/replay", node2.base_url()))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    let events: Vec<serde_json::Value> = body["events"].as_array().unwrap().clone();

    // (1) Every pre-kill acked offset survives recovery with its exact
    // payload.
    for (offset, payload) in &acked {
        let found = events
            .iter()
            .find(|event| event["offset"].as_u64() == Some(*offset))
            .unwrap_or_else(|| {
                panic!("AC4 FAILED: acked pre-kill offset {offset} is missing after recovery")
            });
        assert_eq!(
            &found["payload"], payload,
            "AC4 FAILED: offset {offset} payload changed across recovery"
        );
    }

    // (2) Every CONFIRMED final-batch success (response fully round-tripped
    // before the kill) also survives recovery with its exact payload.
    for (offset, payload) in &confirmed_final {
        let found = events
            .iter()
            .find(|event| event["offset"].as_u64() == Some(*offset))
            .unwrap_or_else(|| {
                panic!(
                    "AC4 FAILED: confirmed final-batch offset {offset} is missing after recovery"
                )
            });
        assert_eq!(&found["payload"], payload);
    }

    // (3) Never a duplicate: a WAL replay bug duplicates an APPENDED FRAME,
    // which manifests as the SAME payload landing at two different offsets
    // (each replayed Append allocates a fresh offset) -- so the invariant is
    // "no payload value appears twice", not "no offset appears twice" (a
    // single in-memory journal can't have two events at one offset by
    // construction anyway).
    let mut seen_payloads = std::collections::HashSet::new();
    for event in &events {
        let key = event["payload"].to_string();
        assert!(
            seen_payloads.insert(key.clone()),
            "AC4 FAILED: payload {key} appears more than once after recovery -- \
             the WAL replayed an appended frame twice"
        );
    }

    // (4) Never an event for a request that returned an explicit error.
    for payload in &errored_final {
        assert!(
            !events.iter().any(|event| &event["payload"] == payload),
            "AC4 FAILED: an explicitly-errored append must never be durably committed: {payload}"
        );
    }

    eprintln!(
        "AC4: pre_kill_acked={} final_batch_confirmed={} final_batch_errored={} \
         final_batch_undetermined_in_flight={undetermined} recovered_event_count={}",
        acked.len(),
        confirmed_final.len(),
        errored_final.len(),
        events.len()
    );

    // The undetermined middle ground -- a final-batch request whose response
    // never arrived because the kill hit while it was still in flight -- is
    // intentionally NOT asserted either way above: the client cannot know
    // whether the server durably applied it before dying, and both "present
    // after recovery" and "absent after recovery" are correct semantics for
    // a crash landing exactly at that boundary. What must never happen (and
    // is asserted above) is a duplicate, or a commit for a request that came
    // back with an explicit error.

    drop(node2);
}

fn process_rss_bytes(pid: u32) -> u64 {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .expect("run ps to sample RSS");
    assert!(output.status.success(), "ps failed for pid {pid}");
    let stdout = String::from_utf8(output.stdout).expect("ps output is utf-8");
    let rss_kib: u64 = stdout
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("ps rss output {stdout:?} not numeric: {e}"));
    rss_kib.saturating_mul(1024)
}

const RSS_EVENTS: usize = 300;
const RSS_PAYLOAD_BYTES: usize = 24_000; // ~7.2 MiB logical total.

/// AC3: RSS / logical-payload ratio on the durable (`--data-dir`, WAL) path
/// stays within an order of magnitude of the in-memory path's 1.6x baseline
/// (i.e. `< 16x`), against the prior durable-path measurement of 43.4x.
///
/// Measured with `ps -o rss= -p <pid>` -- the same external, OS-level
/// technique the original 43.4x figure was measured with -- on
/// **macOS only** (this process's `target_os`); a Linux re-measurement is
/// tracked separately and is not this dispatch's job.
#[tokio::test]
async fn rss_over_logical_payload_stays_within_an_order_of_magnitude_of_1_6x() {
    let dir = tempfile::tempdir().unwrap();
    let bind = free_addr();
    let node = spawn_single_node(&bind, dir.path());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    wait_healthy(
        &client,
        &node.base_url(),
        Instant::now() + Duration::from_secs(20),
    )
    .await;

    // Let the process settle past its one-time startup allocations before
    // sampling the idle baseline.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let idle_rss = process_rss_bytes(node.pid());

    let payload = "x".repeat(RSS_PAYLOAD_BYTES);
    for n in 0..RSS_EVENTS {
        let resp = client
            .post(format!("{}/topics/rss-load/append", node.base_url()))
            .json(&json!({ "payload": payload }))
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success(), "append {n} failed");
    }

    let logical_bytes = (RSS_EVENTS * RSS_PAYLOAD_BYTES) as f64;
    let post_rss = process_rss_bytes(node.pid()) as f64;
    let ratio = post_rss / logical_bytes;
    let delta_ratio = (post_rss - idle_rss as f64).max(0.0) / logical_bytes;

    eprintln!(
        "AC3 (measured on {os}, RSS via `ps -o rss=`): idle_rss={idle_rss}B \
         post_workload_rss={post_rss}B logical_payload={logical_bytes}B \
         ratio={ratio:.2}x idle_subtracted_ratio={delta_ratio:.2}x \
         (order-of-magnitude gate: <16x; reference baselines: 1.6x in-memory, 43.4x prior durable)",
        os = std::env::consts::OS,
        idle_rss = idle_rss,
        post_rss = post_rss as u64,
        logical_bytes = logical_bytes as u64,
    );

    assert!(
        ratio < 16.0,
        "AC3 FAILED: durable-path RSS/logical-payload ratio {ratio:.2}x exceeds the \
         order-of-magnitude-of-1.6x gate (<16x); idle_rss={idle_rss}B post_rss={post_rss}B \
         logical_bytes={logical_bytes}B"
    );
}
// HANDWRITE-END

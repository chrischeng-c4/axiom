//! Black-box contract for the segment-backpressure counter on local admission.
//!
//! The child raises its HTTP body allowance only for this one fixture. It sends
//! one valid 100 MiB ASCII Keyword body, which is below that child-only body
//! allowance but whose raw record plus two embedded-WAL transport copies is
//! larger than the documented 256 MiB pending-change budget. The coordinator
//! must refuse it before `MemWal` receives a sequence. The counter must record
//! that real pre-commit capacity refusal exactly once.
//!
//! A later small valid write proves the service stays usable and does not add a
//! counter sample. A malformed JSON body proves the external byte parser stays
//! closed and does not look like a capacity refusal. The isolated child owns
//! the process-wide budget and body-limit environment. Its parent owns the
//! temporary directory and kills and reaps it only for cleanup.
//!
//! # Facets
//!
//! - Behavior: segment_backpressure_metric.rs:296-334 requires the valid
//!   oversized request to return `429`, retain the old WAL/applied/document
//!   state, and increase `lumen_segment_backpressure_total` exactly once.
//!   Assertions at :336-368 require one later small write to succeed without
//!   changing the counter. These exercise
//!   apps/lumen/src/coordinator.rs:1015-1088,
//!   apps/lumen/src/coordinator.rs:875-912, and
//!   apps/lumen/src/metrics.rs:296-297.
//! - Security: segment_backpressure_metric.rs:370-405 sends malformed external
//!   JSON and requires `400`, no `Retry-After`, no WAL/applied/document change,
//!   and no backpressure increment. This exercises the HTTP-byte boundary in
//!   apps/lumen/src/api.rs:1554-1589 and its closed capacity mapping at
//!   apps/lumen/src/api.rs:3269-3273.
//! - Performance: apps/lumen/docs/indexing.md:264-276 says, verbatim,
//!   "Pending active, frozen, and reserved changes have a 256 MiB budget."
//!   segment_backpressure_metric.rs:201-212 proves the one-item body exceeds
//!   one third of that budget, so the local raw record plus two transport
//!   copies cannot be admitted. The `429` assertion at :296-300 carries that
//!   documented limit. The 60-second request and 120-second child bounds are
//!   cleanup bounds, not product latency promises.
//!
//! # Root negative control
//!
//! Remove the production call that increments the counter from the
//! pre-publication capacity refusal branch in
//! `apps/lumen/src/coordinator.rs`. The assertion at :330-334 must fail because
//! the metric remains at its baseline. Restore every changed source file by
//! SHA-256.
//!
//! Target gate: cargo test -p lumen --test segment_backpressure_metric -- --nocapture.
//! Full declared behavior gate: cargo test -p lumen.

use std::fs::{self, File};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::{header::RETRY_AFTER, StatusCode};
use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::{WriteCoordinator, WriteSink};
use lumen::storage::Engine;
use lumen::wal::{MemWal, SharedWal, WalLog};

const COLLECTION: &str = "segment-backpressure-metric";
const FIELD: &str = "keyword";
const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
const OVERSIZED_KEYWORD_BYTES: usize = 100 * 1024 * 1024;
const CHILD_BODY_LIMIT_BYTES: usize = OVERSIZED_KEYWORD_BYTES + 2 * 1024 * 1024;
const REQUEST_CLEANUP_BOUND: Duration = Duration::from_secs(60);
const CHILD_CLEANUP_BOUND: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const CHILD_MODE_ENV: &str = "LUMEN_SEGMENT_BACKPRESSURE_METRIC_CHILD";
const CHILD_HANDSHAKE_ENV: &str = "LUMEN_SEGMENT_BACKPRESSURE_METRIC_HANDSHAKE";
const CHILD_CASE: &str = "precommit-capacity-refusal";
const TEST_NAME: &str = "precommit_capacity_refusal_increments_segment_backpressure_once";

/// Owns the child on every parent unwind path. A stuck fixture cannot leave a
/// process-wide change budget or body-limit environment alive for another test.
struct ChildCleanup(Option<Child>);

impl Drop for ChildCleanup {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(Some(_))) {
            let _ = child.kill();
        }
        let _ = child.wait();
    }
}

fn child_enters() -> bool {
    if std::env::var(CHILD_MODE_ENV).ok().as_deref() != Some(CHILD_CASE) {
        return false;
    }
    let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
        .expect("segment-backpressure child needs a handshake path");
    fs::write(handshake, CHILD_CASE).expect("record exact segment-backpressure child entry");
    true
}

async fn run_isolated_child() {
    let child_root = tempfile::tempdir().expect("segment-backpressure child workspace");
    let child_tmp = child_root.path().join("child-tmp");
    fs::create_dir(&child_tmp).expect("create parent-owned segment-backpressure child tempdir");
    let handshake = child_root.path().join("entered-case");
    let stdout_path = child_root.path().join("child.stdout");
    let stderr_path = child_root.path().join("child.stderr");
    let executable = std::env::current_exe().expect("current segment-backpressure test executable");
    let stdout = File::create(&stdout_path).expect("create segment-backpressure child stdout");
    let stderr = File::create(&stderr_path).expect("create segment-backpressure child stderr");
    let child = Command::new(executable)
        .env(CHILD_MODE_ENV, CHILD_CASE)
        .env(CHILD_HANDSHAKE_ENV, &handshake)
        .env("LUMEN_BODY_LIMIT_BYTES", CHILD_BODY_LIMIT_BYTES.to_string())
        .env("TMPDIR", &child_tmp)
        .env("TEMP", &child_tmp)
        .env("TMP", &child_tmp)
        .arg(TEST_NAME)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--test-threads=1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("spawn isolated segment-backpressure child");
    let mut child = ChildCleanup(Some(child));
    let deadline = Instant::now() + CHILD_CLEANUP_BOUND;
    loop {
        match child
            .0
            .as_mut()
            .expect("segment-backpressure child remains owned until exit")
            .try_wait()
        {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => tokio::time::sleep(POLL_INTERVAL).await,
            Ok(None) => {
                let mut raw = child
                    .0
                    .take()
                    .expect("timed-out segment-backpressure child remains owned");
                let _ = raw.kill();
                let status = raw
                    .wait()
                    .expect("wait for killed segment-backpressure child");
                let stdout = fs::read_to_string(&stdout_path)
                    .expect("read killed segment-backpressure child stdout");
                let stderr = fs::read_to_string(&stderr_path)
                    .expect("read killed segment-backpressure child stderr");
                panic!(
                    "segment-backpressure child exceeded cleanup bound {CHILD_CLEANUP_BOUND:?}; status={status}; stdout={stdout}; stderr={stderr}",
                );
            }
            Err(error) => panic!("poll isolated segment-backpressure child: {error}"),
        }
    }
    let status = child
        .0
        .take()
        .expect("exited segment-backpressure child remains owned")
        .wait()
        .expect("wait for exited segment-backpressure child");
    let stdout = fs::read_to_string(&stdout_path).expect("read segment-backpressure child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read segment-backpressure child stderr");
    let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
        panic!(
            "segment-backpressure child did not enter exact {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}",
        )
    });
    assert_eq!(
        entered, CHILD_CASE,
        "segment-backpressure child must enter the intended capacity body",
    );
    assert!(
        status.success(),
        "isolated segment-backpressure child failed: status={status}; stdout={stdout}; stderr={stderr}",
    );
}

fn oversized_index_body() -> Vec<u8> {
    let mut value = vec![b'x'; OVERSIZED_KEYWORD_BYTES];
    let marker = b"segment-backpressure-valid-keyword:";
    value[..marker.len()].copy_from_slice(marker);
    let request = json!({
        "items": [{
            "external_id": "capacity-refused",
            "field": FIELD,
            "value": String::from_utf8(value).expect("ASCII capacity fixture Keyword"),
        }]
    });
    assert_eq!(
        request["items"].as_array().map(Vec::len),
        Some(1),
        "the capacity fixture must contain exactly one valid index item",
    );
    let body = serde_json::to_vec(&request).expect("serialize valid capacity fixture request");
    drop(request);
    assert!(
        body.len() < CHILD_BODY_LIMIT_BYTES,
        "the real capacity request must stay below its child-only HTTP allowance: {} >= {}",
        body.len(),
        CHILD_BODY_LIMIT_BYTES,
    );
    assert!(
        body.len() > PENDING_HARD_LIMIT_BYTES / 3,
        "the one-item body must make raw plus two embedded-WAL copies exceed the 256 MiB budget: {} <= {}",
        body.len(),
        PENDING_HARD_LIMIT_BYTES / 3,
    );
    body
}

async fn indexed_total(server: &TestServer) -> u64 {
    let response = server
        .post(&format!("/collections/{COLLECTION}/search"))
        .json(&json!({
            "query": { "exists": { "field": FIELD } },
            "limit": 10,
        }))
        .await;
    response.assert_status_ok();
    response.json::<Value>()["total"]
        .as_u64()
        .expect("keyword exists response must carry a total")
}

fn metric_u64(metrics: &str, name: &str) -> u64 {
    let values = metrics
        .lines()
        .filter_map(|line| {
            let (metric, value) = line.split_once(|character: char| character.is_whitespace())?;
            (metric == name).then_some(value.trim())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        values.len(),
        1,
        "public /metrics must publish exactly one {name} sample: {metrics}",
    );
    values[0]
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned counter: {}", values[0]))
}

async fn segment_backpressure_total(server: &TestServer) -> u64 {
    let response = server.get("/metrics").await;
    response.assert_status_ok();
    metric_u64(&response.text(), "lumen_segment_backpressure_total")
}

async fn precommit_capacity_refusal_body() {
    let engine = Arc::new(Engine::new());
    let wal = Arc::new(MemWal::new());
    let shared_wal: SharedWal = wal.clone();
    let writer = WriteCoordinator::start(shared_wal, engine.clone());
    let sink: Arc<dyn WriteSink> = writer.clone();
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), sink);
    let server = TestServer::new(router(state)).expect("segment-backpressure HTTP server");

    server
        .put(&format!("/collections/{COLLECTION}"))
        .json(&json!({ "fields": { FIELD: { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let applied_before = writer.applied_seq();
    let wal_before = wal
        .latest_seq()
        .await
        .expect("read MemWal before capacity refusal");
    assert_eq!(
        indexed_total(&server).await,
        0,
        "the new collection must start without Keyword documents",
    );
    let backpressure_before = segment_backpressure_total(&server).await;
    assert_eq!(
        backpressure_before, 0,
        "a new Engine must start with zero real segment-backpressure admissions",
    );

    let response = tokio::time::timeout(
        REQUEST_CLEANUP_BOUND,
        server
            .post(&format!("/collections/{COLLECTION}/index"))
            .bytes(oversized_index_body().into())
            .content_type("application/json"),
    )
    .await
    .expect("valid pre-commit capacity refusal must complete within test cleanup bound");
    let retry_after = response
        .maybe_header(RETRY_AFTER)
        .and_then(|value| value.to_str().ok().map(ToOwned::to_owned));
    assert_eq!(
        response.status_code(),
        StatusCode::TOO_MANY_REQUESTS,
        "a valid one-item record over the local pending budget must return HTTP 429 before WAL publication",
    );
    let envelope = response.json::<Value>();
    assert_eq!(
        envelope["error"],
        "pending_change_capacity",
        "the valid oversized body must reach pre-commit capacity refusal, not a validation error",
    );
    assert_eq!(
        retry_after.as_deref(),
        Some("1"),
        "a pre-commit capacity refusal must expose Retry-After: 1",
    );
    assert_eq!(
        wal.latest_seq()
            .await
            .expect("read MemWal after capacity refusal"),
        wal_before,
        "a pre-commit capacity refusal must not allocate a WAL sequence",
    );
    assert_eq!(
        writer.applied_seq(),
        applied_before,
        "a pre-commit capacity refusal must not advance the applied sequence",
    );
    assert_eq!(
        indexed_total(&server).await,
        0,
        "a pre-commit capacity refusal must not index a document",
    );
    let backpressure_after_refusal = segment_backpressure_total(&server).await;
    assert_eq!(
        backpressure_after_refusal,
        backpressure_before + 1,
        "one real pre-commit capacity refusal must increment lumen_segment_backpressure_total exactly once",
    );

    server
        .post(&format!("/collections/{COLLECTION}/index"))
        .json(&json!({
            "items": [{
                "external_id": "small-after-capacity-refusal",
                "field": FIELD,
                "value": "small",
            }]
        }))
        .await
        .assert_status_ok();
    assert_eq!(
        wal.latest_seq()
            .await
            .expect("read MemWal after accepted small write"),
        wal_before + 1,
        "only the accepted small write may allocate the next WAL sequence",
    );
    assert_eq!(
        writer.applied_seq(),
        applied_before + 1,
        "only the accepted small write may advance the applied sequence",
    );
    assert_eq!(
        indexed_total(&server).await,
        1,
        "the accepted small write must remain query-visible after capacity refusal",
    );
    assert_eq!(
        segment_backpressure_total(&server).await,
        backpressure_after_refusal,
        "an accepted small write must not increment segment backpressure",
    );

    let malformed = server
        .post(&format!("/collections/{COLLECTION}/index"))
        .bytes(b"{".to_vec().into())
        .content_type("application/json")
        .await;
    assert_eq!(
        malformed.status_code(),
        StatusCode::BAD_REQUEST,
        "malformed external JSON must be rejected with HTTP 400 before coordinator admission",
    );
    assert!(
        malformed.maybe_header(RETRY_AFTER).is_none(),
        "malformed external JSON must not advertise a capacity retry",
    );
    assert_eq!(
        wal.latest_seq()
            .await
            .expect("read MemWal after malformed request"),
        wal_before + 1,
        "malformed external JSON must not allocate a WAL sequence",
    );
    assert_eq!(
        writer.applied_seq(),
        applied_before + 1,
        "malformed external JSON must not advance the applied sequence",
    );
    assert_eq!(
        indexed_total(&server).await,
        1,
        "malformed external JSON must not mutate indexed documents",
    );
    assert_eq!(
        segment_backpressure_total(&server).await,
        backpressure_after_refusal,
        "malformed external JSON must not increment segment backpressure",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn precommit_capacity_refusal_increments_segment_backpressure_once() {
    if child_enters() {
        precommit_capacity_refusal_body().await;
    } else {
        run_isolated_child().await;
    }
}

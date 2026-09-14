//! # Facets
//!
//! - Behavior: nonsegment_budget_progress.rs:41-61, :78, :92, :107, :112,
//!   :129, and :139 require legal large index requests to make progress
//!   through CBOR and no-data-dir serving without a permanent capacity
//!   refusal, retain the existing admin checkpoint response, and preserve a
//!   real CBOR cold start. Change points: apps/lumen/src/bin/lumen.rs:3518-3533
//!   chooses CBOR/no-data-dir persistence and apps/lumen/src/coordinator.rs
//!   owns capacity before a submitted record applies.
//! - Security: the changed request-capacity path accepts caller input after
//!   the existing body-limit boundary. Every large request in
//!   support/serve_budget_support.rs:336-342 is asserted below 8 MiB; the
//!   closed oversized-body outcome is already pinned by
//!   apps/lumen/e2e/http_body_limit_e2e.rs:67-119. No new parser, path, or
//!   credential input is opened by the spill/progress change points.
//! - Performance: apps/lumen/ROADMAP.md:60-70 promises, verbatim, "Pending
//!   active, frozen, and reserved changes have a 256 MiB total budget" and
//!   "At 128 MiB, the runtime requests an early checkpoint."
//!   nonsegment_budget_progress.rs:64 and
//!   support/serve_budget_support.rs:411, :415, and :419, invoked at :93 and
//!   :130, assert public accounting stays within that budget after distinct
//!   input exceeds it. These are budget checks, not a latency target.
//!
//! Gate: cargo test -p lumen --test nonsegment_budget_progress -- --nocapture.

#[path = "support/serve_budget_support.rs"]
mod serve_budget_support;

use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use serve_budget_support::{
    assert_keyword_hit, assert_pending_metrics, create_keyword_collection, index_large_keyword,
    raw_payload_bytes, stats_documents, wait_for_cbor_snapshot, LumenProcess, ServeMode,
    LARGE_VALUE_COUNT, PENDING_HARD_BYTES, RETRY_DEADLINE,
};

fn submit_all_large_rows_with_bounded_retry(process: &LumenProcess) {
    let deadline = Instant::now() + RETRY_DEADLINE;
    for ordinal in 0..LARGE_VALUE_COUNT {
        loop {
            let response = index_large_keyword(process, ordinal);
            match response.status {
                200 => break,
                429 => {
                    assert_eq!(
                        response.retry_after.as_deref(),
                        Some("1"),
                        "a temporary capacity refusal must expose the documented Retry-After: 1 header",
                    );
                    assert!(
                        Instant::now() < deadline,
                        "legal write {ordinal} remained permanently refused at the fixed capacity boundary: {}",
                        response.body,
                    );
                    thread::sleep(Duration::from_secs(1));
                }
                status => panic!(
                    "legal, under-8-MiB large index request {ordinal} returned unexpected HTTP {status}: {}",
                    response.body
                ),
            }
        }
    }
    assert!(
        raw_payload_bytes() > PENDING_HARD_BYTES as usize,
        "fixture must exceed 256 MiB of distinct raw caller input after all rows complete",
    );
}

fn assert_noop_checkpoint(process: &LumenProcess, mode: &str) {
    let response = process.post_json("/admin/checkpoint", &json!({}));
    assert_eq!(
        response.status, 200,
        "{mode} checkpoint endpoint must remain callable: {}",
        response.body
    );
    assert_eq!(
        response.body["persisted"], false,
        "{mode} must retain the existing public no-op checkpoint response: {}",
        response.body
    );
}

#[test]
fn cbor_server_crosses_capacity_with_retries_and_restores_latest_rdb() {
    let root = tempfile::tempdir().expect("CBOR budget root");
    // Leave time for the fixture to cross the early budget threshold before
    // either periodic writer fires. The final CBOR snapshot still uses the
    // production timer and is verified before the cold start.
    const SNAPSHOT_SECS: u64 = 60;
    let mut process = LumenProcess::spawn(Some(root.path()), ServeMode::Cbor, SNAPSHOT_SECS);
    process.wait_until_ready(SNAPSHOT_SECS);
    create_keyword_collection(&process);
    assert_noop_checkpoint(&process, "CBOR");

    submit_all_large_rows_with_bounded_retry(&process);
    assert_pending_metrics(&process.get_text("/metrics"));
    assert_eq!(
        stats_documents(&process),
        LARGE_VALUE_COUNT as u64,
        "all accepted CBOR writes must remain live",
    );
    for ordinal in [0, LARGE_VALUE_COUNT / 2, LARGE_VALUE_COUNT - 1] {
        assert_keyword_hit(&process, ordinal);
    }
    assert_noop_checkpoint(&process, "CBOR");

    // This reads the production rdb-*.lrb layout and restores it into a fresh
    // Engine before stopping the only live server. It prevents a stale timer
    // snapshot from becoming a false cold-start green.
    wait_for_cbor_snapshot(root.path(), LARGE_VALUE_COUNT as u64);
    process.stop_and_logs();

    let mut cold = LumenProcess::spawn(Some(root.path()), ServeMode::Cbor, SNAPSHOT_SECS);
    cold.wait_until_ready(SNAPSHOT_SECS);
    assert_eq!(
        stats_documents(&cold),
        LARGE_VALUE_COUNT as u64,
        "CBOR cold start must retain every capacity-progressed write",
    );
    for ordinal in [0, LARGE_VALUE_COUNT / 2, LARGE_VALUE_COUNT - 1] {
        assert_keyword_hit(&cold, ordinal);
    }
}

#[test]
fn memory_only_server_crosses_capacity_without_permanent_429_and_keeps_checkpoint_noop() {
    let mut process = LumenProcess::spawn(None, ServeMode::MemoryOnly, 300);
    process.wait_until_ready(300);
    create_keyword_collection(&process);
    assert_noop_checkpoint(&process, "no-data-dir");

    submit_all_large_rows_with_bounded_retry(&process);
    assert_pending_metrics(&process.get_text("/metrics"));
    assert_eq!(
        stats_documents(&process),
        LARGE_VALUE_COUNT as u64,
        "no-data-dir serving must retain every accepted write while it remains live",
    );
    for ordinal in [0, LARGE_VALUE_COUNT / 2, LARGE_VALUE_COUNT - 1] {
        assert_keyword_hit(&process, ordinal);
    }
    assert_noop_checkpoint(&process, "no-data-dir");
}

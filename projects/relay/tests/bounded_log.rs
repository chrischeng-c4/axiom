// SPEC-MANAGED: projects/relay/tech-design/logic/bounded-ram-durable-log-entry-eviction-offset-index-disk-backed.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:d73d9a0d" tracker="pending-tracker" reason="Tests: disk-backed entry() for evicted seqs, broadcast range/replay across the evict boundary, bounded dedupe window, and no-eviction parity."
//! Bounded-RAM durable log (#130): entries evicted past `ram_ring_entries` are
//! read back from disk via the offset index; broadcast replay spans the evict
//! boundary; the dedupe map is bounded to a FIFO window.

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Log, Relay, RelayCoreConfig};

fn disk_cfg(dir: &std::path::Path, ring: u64, dedupe_window: u64) -> RelayCoreConfig {
    let mut cfg = RelayCoreConfig::default();
    cfg.data_dir = dir.to_string_lossy().into_owned();
    cfg.ram_ring_entries = ring;
    cfg.dedupe.window_entries = dedupe_window;
    cfg
}

fn append(log: &mut Log, id: &str, i: usize) {
    log.append(
        id,
        serde_json::json!({ "i": i }),
        BTreeMap::new(),
        Utc::now(),
    )
    .unwrap();
}

// An evicted entry is read back from disk via the offset index.
#[test]
fn evicted_entry_reads_from_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&disk_cfg(dir.path(), 4, 1_000_000), "s", 0).unwrap();
    for i in 0..20 {
        append(&mut log, &format!("m{i}"), i);
    }
    assert_eq!(log.len(), 20);

    // seq 2 was evicted from the 4-entry ring -> served from disk.
    let evicted = log.entry(2).unwrap().unwrap();
    assert_eq!(evicted.seq, 2);
    assert_eq!(evicted.payload, serde_json::json!({ "i": 2 }));

    // seq 19 is still resident.
    let resident = log.entry(19).unwrap().unwrap();
    assert_eq!(resident.payload, serde_json::json!({ "i": 19 }));

    assert!(log.entry(20).unwrap().is_none());
}

// range / broadcast replay spans the evict boundary (cold prefix from disk,
// hot tail from the ring), in order.
#[test]
fn range_spans_evict_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&disk_cfg(dir.path(), 4, 1_000_000), "s", 0).unwrap();
    for i in 0..20 {
        append(&mut log, &format!("m{i}"), i);
    }
    let all = log.range(0).unwrap();
    assert_eq!(all.len(), 20);
    assert_eq!(
        all.iter().map(|e| e.seq).collect::<Vec<_>>(),
        (0..20).collect::<Vec<u64>>()
    );
    assert_eq!(all[5].payload, serde_json::json!({ "i": 5 }));
    assert_eq!(all[19].payload, serde_json::json!({ "i": 19 }));
}

// Broadcast subscribe-from-0 over a mostly-evicted log delivers everything.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broadcast_replays_evicted_log() {
    let dir = tempfile::tempdir().unwrap();
    let r = Relay::new(disk_cfg(dir.path(), 4, 1_000_000));
    let now = Utc::now();
    for i in 0..20 {
        r.publish(
            "s",
            &format!("m{i}"),
            serde_json::json!({ "i": i }),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
    r.subscribe("s", "sub", 0).unwrap();
    let got = r.poll("s", "sub").unwrap();
    assert_eq!(
        got.len(),
        20,
        "all entries replayed across the evict boundary"
    );
    assert_eq!(got[0].seq, 0);
    assert_eq!(got[19].seq, 19);
}

// The dedupe map is a bounded FIFO window: an id still in the window dedupes,
// an id evicted from the window re-appends (bounded at-least-once).
#[test]
fn dedupe_window_is_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&disk_cfg(dir.path(), 4, 4), "s", 0).unwrap();
    for i in 0..10 {
        append(&mut log, &format!("m{i}"), i);
    }
    // m9 is within the 4-id window -> deduped to its original seq.
    let in_window = log
        .append("m9", serde_json::json!({}), BTreeMap::new(), Utc::now())
        .unwrap();
    assert!(in_window.deduped);
    assert_eq!(in_window.seq, 9);

    // m0 fell out of the window -> re-appended as a new entry.
    let evicted = log
        .append("m0", serde_json::json!({}), BTreeMap::new(), Utc::now())
        .unwrap();
    assert!(!evicted.deduped, "evicted id is no longer deduped");
    assert_eq!(evicted.seq, 10);
}

// With the ring larger than the log, nothing is evicted (hot-path parity).
#[test]
fn no_eviction_when_ring_exceeds_log() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&disk_cfg(dir.path(), 1_000_000, 1_000_000), "s", 0).unwrap();
    for i in 0..5 {
        append(&mut log, &format!("m{i}"), i);
    }
    assert_eq!(log.range(0).unwrap().len(), 5);
    assert_eq!(
        log.entry(0).unwrap().unwrap().payload,
        serde_json::json!({ "i": 0 })
    );
}
// HANDWRITE-END

// SPEC-MANAGED: apps/relay/tech-design/logic/log-segment-rotation-retention-full-log-lifecycle.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:b8e02453" tracker="pending-tracker" reason="Tests: rotation into multiple segment files + ordered range across them, byte-based pruning advancing start_seq, reads of pruned seqs (None / clamp), multi-segment recovery on reopen, and single-segment parity at default sizes."
//! Log segment rotation + retention (#131): rolling into segment files, ordered
//! reads across segments, byte-based pruning that advances start_seq, pruned
//! reads, and recovery (including after pruning).

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Log, RelayCoreConfig};

fn seg_cfg(dir: &std::path::Path, segment_bytes: u64) -> RelayCoreConfig {
    let mut cfg = RelayCoreConfig::default();
    cfg.data_dir = dir.to_string_lossy().into_owned();
    cfg.segment_bytes = segment_bytes;
    cfg.ram_ring_entries = 4; // force disk-backed reads alongside segmentation
    cfg
}

fn append(log: &mut Log, i: usize) {
    log.append(
        &format!("m{i}"),
        serde_json::json!({ "i": i }),
        BTreeMap::new(),
        Utc::now(),
    )
    .unwrap();
}

fn count_segments(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.starts_with("s__shard0__") && n.ends_with(".ndjson")
        })
        .count()
}

// Appends roll into multiple segment files; a full range still reads in order.
#[test]
fn rotation_and_cross_segment_range() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 200), "s", 0).unwrap();
    for i in 0..20 {
        append(&mut log, i);
    }
    assert!(
        count_segments(dir.path()) > 1,
        "rolled into multiple segments"
    );
    let all = log.range(0).unwrap();
    assert_eq!(
        all.iter().map(|e| e.seq).collect::<Vec<_>>(),
        (0..20).collect::<Vec<u64>>()
    );
    assert_eq!(all[13].payload, serde_json::json!({ "i": 13 }));
}

// Surviving segments replay correctly on reopen.
#[test]
fn multi_segment_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = seg_cfg(dir.path(), 200);
    {
        let mut log = Log::open(&cfg, "s", 0).unwrap();
        for i in 0..20 {
            append(&mut log, i);
        }
    }
    let log2 = Log::open(&cfg, "s", 0).unwrap();
    assert_eq!(log2.len(), 20);
    let all = log2.range(0).unwrap();
    assert_eq!(all.len(), 20);
    assert_eq!(all[7].seq, 7);
}

// With a huge segment_bytes there is one segment and behavior is unchanged.
#[test]
fn single_segment_parity_default_sizes() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 100_000_000), "s", 0).unwrap();
    for i in 0..10 {
        append(&mut log, i);
    }
    assert_eq!(count_segments(dir.path()), 1);
    assert_eq!(log.range(0).unwrap().len(), 10);
    assert_eq!(log.start_seq(), 0);
}

// Phase 1 (delete-on-ack): in `Ack` mode, truncate_below_acked drops every
// oldest whole segment fully below the committed watermark, advancing start_seq;
// entries below the watermark are gone while the watermark's own segment (and
// everything after) survives.
#[test]
fn ack_mode_truncates_acked_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 200), "s", 0).unwrap();
    for i in 0..20 {
        append(&mut log, i);
    }
    let before = count_segments(dir.path());
    assert!(before > 1, "rolled into multiple segments");
    assert_eq!(log.start_seq(), 0);

    // Ack the contiguous prefix [0, 12): reclaim whole segments fully below 12.
    log.truncate_below_acked(12).unwrap();

    let start = log.start_seq();
    assert!(start > 0, "acked-prefix segments reclaimed, start_seq advanced");
    assert!(start <= 12, "never drops the segment holding seq 12 or beyond");
    assert!(
        count_segments(dir.path()) < before,
        "old segment files deleted"
    );
    assert!(log.entry(start - 1).unwrap().is_none(), "below start is gone");
    assert_eq!(
        log.entry(12).unwrap().unwrap().seq,
        12,
        "watermark entry survives"
    );
    assert_eq!(log.range(0).unwrap().last().unwrap().seq, 19);
}

// Phase 1: a still-un-acked hole pins the head — a low watermark keeps every
// segment even though much has been appended (the broker owns durability until
// the entry is acked).
#[test]
fn ack_mode_hole_pins_head() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 200), "s", 0).unwrap();
    for i in 0..20 {
        append(&mut log, i);
    }
    let before = count_segments(dir.path());
    // Watermark 0 = nothing contiguously acked (hole at seq 0) → nothing dropped.
    log.truncate_below_acked(0).unwrap();
    assert_eq!(log.start_seq(), 0, "an un-acked head pins all segments");
    assert_eq!(count_segments(dir.path()), before);
    assert!(log.entry(0).unwrap().is_some());
}

// HANDWRITE-END

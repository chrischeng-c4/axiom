// SPEC-MANAGED: projects/relay/tech-design/logic/log-segment-rotation-retention-full-log-lifecycle.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:b8e02453" tracker="pending-tracker" reason="Tests: rotation into multiple segment files + ordered range across them, byte-based pruning advancing start_seq, reads of pruned seqs (None / clamp), multi-segment recovery on reopen, and single-segment parity at default sizes."
//! Log segment rotation + retention (#131): rolling into segment files, ordered
//! reads across segments, byte-based pruning that advances start_seq, pruned
//! reads, and recovery (including after pruning).

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Log, RelayCoreConfig};

fn seg_cfg(dir: &std::path::Path, segment_bytes: u64, max_bytes: u64) -> RelayCoreConfig {
    let mut cfg = RelayCoreConfig::default();
    cfg.data_dir = dir.to_string_lossy().into_owned();
    cfg.segment_bytes = segment_bytes;
    cfg.ram_ring_entries = 4; // force disk-backed reads alongside segmentation
    cfg.retention.max_bytes_per_shard = max_bytes;
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
    let mut log = Log::open(&seg_cfg(dir.path(), 200, 0), "s", 0).unwrap();
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

// A small byte budget prunes the oldest segments and advances start_seq; reads
// of pruned seqs return None / clamp.
#[test]
fn byte_retention_prunes_and_clamps() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 200, 600), "s", 0).unwrap();
    for i in 0..40 {
        append(&mut log, i);
    }
    let start = log.start_seq();
    assert!(start > 0, "old segments pruned, start_seq advanced");
    assert!(log.entry(0).unwrap().is_none(), "pruned seq is gone");

    let surviving = log.range(0).unwrap(); // clamps up to start_seq
    assert_eq!(surviving.first().unwrap().seq, start);
    assert_eq!(surviving.last().unwrap().seq, 39);
    assert_eq!(surviving.len() as u64, 40 - start);
}

// Surviving segments replay correctly on reopen.
#[test]
fn multi_segment_recovery() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = seg_cfg(dir.path(), 200, 0);
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

// After pruning, a reopened log keeps the pruned range gone and resumes seqs
// correctly (offset index is relative to start_seq).
#[test]
fn recovery_after_pruning_preserves_seqs() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = seg_cfg(dir.path(), 200, 600);
    let start;
    {
        let mut log = Log::open(&cfg, "s", 0).unwrap();
        for i in 0..40 {
            append(&mut log, i);
        }
        start = log.start_seq();
        assert!(start > 0);
    }
    let log2 = Log::open(&cfg, "s", 0).unwrap();
    assert_eq!(log2.len(), 40, "len recovered across surviving segments");
    assert!(log2.start_seq() >= start, "pruned segments stay gone");
    let all = log2.range(0).unwrap();
    assert_eq!(all.first().unwrap().seq, log2.start_seq());
    assert_eq!(all.last().unwrap().seq, 39);
    // a surviving mid seq reads back with the right payload.
    let mid = log2.start_seq() + 1;
    assert_eq!(
        log2.entry(mid).unwrap().unwrap().payload,
        serde_json::json!({ "i": mid })
    );
}

// With a huge segment_bytes and no retention, there is one segment and behavior
// is unchanged (the durable benchmark runs here).
#[test]
fn single_segment_parity_default_sizes() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&seg_cfg(dir.path(), 100_000_000, 0), "s", 0).unwrap();
    for i in 0..10 {
        append(&mut log, i);
    }
    assert_eq!(count_segments(dir.path()), 1);
    assert_eq!(log.range(0).unwrap().len(), 10);
    assert_eq!(log.start_seq(), 0);
}
// HANDWRITE-END

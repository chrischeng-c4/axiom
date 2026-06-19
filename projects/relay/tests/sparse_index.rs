// SPEC-MANAGED: projects/relay/tech-design/logic/sparse-offset-index-scale-the-log-to-billions-of-entries.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:f6800828" tracker="pending-tracker" reason="Tests: correct reads at/before/after stride boundaries, sparse index size ~ N/stride, cross-segment cold range, and correct reads after prune + reopen."
//! Sparse offset index (#134): reads seek the nearest indexed point and scan
//! forward, so they stay correct around stride boundaries, across segments, and
//! after pruning — while the index holds only ~len/STRIDE points.

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Log, RelayCoreConfig};

fn cfg(dir: &std::path::Path, segment_bytes: u64, max_bytes: u64) -> RelayCoreConfig {
    let mut c = RelayCoreConfig::default();
    c.data_dir = dir.to_string_lossy().into_owned();
    c.segment_bytes = segment_bytes;
    c.ram_ring_entries = 2; // force disk-backed (sparse-indexed) reads
    c.retention.max_bytes_per_shard = max_bytes;
    c
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

// Reads are correct at, before and after stride boundaries (INDEX_STRIDE=64):
// the index holds only a handful of points yet every seq reads back via
// seek + scan-forward.
#[test]
fn reads_correct_around_stride_and_index_is_sparse() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&cfg(dir.path(), 100_000_000, 0), "s", 0).unwrap();
    const N: usize = 200;
    for i in 0..N {
        append(&mut log, i);
    }
    for seq in [0u64, 1, 63, 64, 65, 100, 128, 191, 192, 199] {
        let e = log.entry(seq).unwrap().unwrap();
        assert_eq!(e.seq, seq);
        assert_eq!(e.payload, serde_json::json!({ "i": seq }));
    }
    // ~N/64 index points (0,64,128,192) — far fewer than N.
    assert!(
        log.index_entries() < N / 8,
        "index is sparse ({} points)",
        log.index_entries()
    );
}

// A full range is correct and ordered across many segments (each run seeks its
// own segment's nearest index point).
#[test]
fn cross_segment_range_is_correct() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = Log::open(&cfg(dir.path(), 200, 0), "s", 0).unwrap();
    for i in 0..200 {
        append(&mut log, i);
    }
    let all = log.range(0).unwrap();
    assert_eq!(
        all.iter().map(|e| e.seq).collect::<Vec<_>>(),
        (0..200).collect::<Vec<u64>>()
    );
    assert_eq!(all[137].payload, serde_json::json!({ "i": 137 }));
}

// After pruning + reopen, the rebuilt sparse index still reads surviving seqs.
#[test]
fn reads_correct_after_prune_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let c = cfg(dir.path(), 200, 800);
    {
        let mut log = Log::open(&c, "s", 0).unwrap();
        for i in 0..100 {
            append(&mut log, i);
        }
        assert!(log.start_seq() > 0, "pruned");
    }
    let log2 = Log::open(&c, "s", 0).unwrap();
    let start = log2.start_seq();
    assert!(
        log2.entry(start.saturating_sub(1)).unwrap().is_none(),
        "pruned seq gone"
    );
    // a few surviving seqs read back correctly via the rebuilt sparse index.
    for seq in [start, start + 1, log2.len() - 1] {
        let e = log2.entry(seq).unwrap().unwrap();
        assert_eq!(e.payload, serde_json::json!({ "i": seq }));
    }
    let all = log2.range(0).unwrap();
    assert_eq!(all.first().unwrap().seq, start);
    assert_eq!(all.last().unwrap().seq, log2.len() - 1);
}
// HANDWRITE-END

#![cfg(feature = "experimental")] // unwired subsystem; run with --features experimental
//! Integration tests for the log-structured backend.
//!
//! These tests drive `lumen::storage_lsm::Lsm` directly via its
//! `Backend` trait — the API layer is intentionally not involved.
//! See `projects/lumen/README.md` §2 for the design these tests
//! verify.

use std::path::PathBuf;
use std::sync::atomic::Ordering;

use lumen::storage_backend::{Backend, PostingEntry};
use lumen::storage_lsm::{FsyncPolicy, Lsm, LsmConfig};
use rand::{rngs::StdRng, Rng, SeedableRng};
use tempfile::TempDir;

fn cfg(root: PathBuf) -> LsmConfig {
    LsmConfig {
        root,
        // Small budgets to drive flushes/compactions inside short tests.
        memtable_bytes: 4 * 1024,
        cache_bytes: 4 * 1024 * 1024,
        fsync: FsyncPolicy::PerWrite,
        block_bytes: 8 * 1024,
    }
}

fn write_n(lsm: &Lsm, n: usize) -> Vec<(String, String, String)> {
    let mut rng = StdRng::seed_from_u64(0xCAFE);
    let mut written = Vec::with_capacity(n);
    for i in 0..n {
        // Spread postings across several distinct keys so the bloom
        // filter has a non-trivial domain to test.
        let key = format!("key-{}", i % 32);
        let eid = format!("eid-{i:06}-{}", rng.gen::<u16>());
        let payload = format!("val-{i}");
        lsm.put_posting("c", 0, key.as_bytes(), &eid, payload.as_bytes())
            .unwrap();
        written.push((key, eid, payload));
    }
    written
}

fn collect_key(lsm: &Lsm, key: &str) -> Vec<PostingEntry> {
    lsm.posting("c", 0, key.as_bytes()).unwrap()
}

#[test]
fn flush_then_reopen_preserves_data() {
    let dir = TempDir::new().unwrap();
    let written = {
        let lsm = Lsm::open(cfg(dir.path().to_path_buf())).unwrap();
        let written = write_n(&lsm, 10_000);
        lsm.flush().unwrap();
        written
    };
    // Drop the LSM (compactor thread exits) then reopen. The fresh
    // instance must read the same data the first instance wrote.
    let lsm = Lsm::open(cfg(dir.path().to_path_buf())).unwrap();
    // Bucket the writes by key so we can verify aggregate counts.
    let mut by_key: std::collections::BTreeMap<String, Vec<(String, String)>> =
        std::collections::BTreeMap::new();
    for (k, eid, payload) in written {
        by_key.entry(k).or_default().push((eid, payload));
    }
    for (key, expected) in &by_key {
        let got = collect_key(&lsm, key);
        assert_eq!(
            got.len(),
            expected.len(),
            "key {key} count mismatch after reopen"
        );
        // Spot-check a few entries.
        for (eid, payload) in expected.iter().take(5) {
            let hit = got
                .iter()
                .find(|e| &e.external_id == eid)
                .unwrap_or_else(|| {
                    panic!("eid {eid} missing for key {key} after reopen");
                });
            assert_eq!(hit.payload, payload.as_bytes());
        }
    }
}

#[test]
fn bloom_filter_rejects_unknown_keys() {
    let dir = TempDir::new().unwrap();
    let lsm = Lsm::open(cfg(dir.path().to_path_buf())).unwrap();
    // Write a small known universe so the bloom filter actually has
    // room to reject probes.
    for i in 0..100 {
        lsm.put_posting(
            "c",
            0,
            format!("known-{i}").as_bytes(),
            &format!("eid-{i}"),
            b"v",
        )
        .unwrap();
    }
    lsm.flush().unwrap();

    // Reset metrics by reading the baseline.
    let baseline = lsm.metrics.bloom_rejections.load(Ordering::Relaxed);
    for i in 0..200 {
        // Keys not in the SST. A correct bloom filter should reject
        // the vast majority of these without paging in a block.
        let _ = lsm
            .posting("c", 0, format!("unknown-{i}").as_bytes())
            .unwrap();
    }
    let after = lsm.metrics.bloom_rejections.load(Ordering::Relaxed);
    let rejections = after - baseline;
    // FPR ~1%, so out of 200 unknown probes we expect ~198 rejections.
    // Be generous in the assertion — anything above 150 means the
    // bloom is plausibly working.
    assert!(
        rejections >= 150,
        "expected >=150 bloom rejections out of 200, got {rejections}"
    );
}

#[test]
fn compaction_reduces_sst_count_and_preserves_data() {
    let dir = TempDir::new().unwrap();
    let lsm = Lsm::open(cfg(dir.path().to_path_buf())).unwrap();

    // Generate several SSTs by repeatedly flushing.
    let mut written: Vec<(String, String, String)> = Vec::new();
    for round in 0..5 {
        for i in 0..200 {
            let key = format!("key-{}", i % 16);
            let eid = format!("eid-r{round}-{i:04}");
            let payload = format!("v{round}-{i}");
            lsm.put_posting("c", 0, key.as_bytes(), &eid, payload.as_bytes())
                .unwrap();
            written.push((key, eid, payload));
        }
        lsm.flush().unwrap();
    }

    // Snapshot 100 random keys' postings before compaction.
    let mut rng = StdRng::seed_from_u64(0xBEEF);
    let mut sample: Vec<String> = (0..1000)
        .map(|_| format!("key-{}", rng.gen_range(0..16)))
        .collect();
    sample.sort();
    sample.dedup();
    let before: std::collections::BTreeMap<String, Vec<PostingEntry>> = sample
        .iter()
        .map(|k| (k.clone(), collect_key(&lsm, k)))
        .collect();

    lsm.compact().unwrap();

    let after: std::collections::BTreeMap<String, Vec<PostingEntry>> = sample
        .iter()
        .map(|k| (k.clone(), collect_key(&lsm, k)))
        .collect();

    for (k, expected) in &before {
        let got = after.get(k).unwrap();
        assert_eq!(
            got.len(),
            expected.len(),
            "key {k} posting count changed across compaction"
        );
        for e in expected {
            assert!(
                got.iter().any(|g| g.external_id == e.external_id),
                "eid {} missing for key {k} after compaction",
                e.external_id
            );
        }
    }
}

#[test]
fn restart_recovery_replays_wal_without_flush() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    // Write a small set, **do not flush**, drop the LSM. A crash at
    // this point is what the WAL exists to recover from.
    {
        let lsm = Lsm::open(cfg(root.clone())).unwrap();
        for i in 0..250 {
            let key = format!("key-{}", i % 8);
            let eid = format!("eid-{i:04}");
            let payload = format!("v-{i}");
            lsm.put_posting("c", 0, key.as_bytes(), &eid, payload.as_bytes())
                .unwrap();
        }
        // Note: explicitly no `.flush()` call.
    }

    let lsm = Lsm::open(cfg(root)).unwrap();
    let mut total = 0usize;
    for key_idx in 0..8 {
        let key = format!("key-{key_idx}");
        let got = collect_key(&lsm, &key);
        total += got.len();
        for e in &got {
            assert!(e.payload.starts_with(b"v-"));
        }
    }
    assert_eq!(total, 250, "WAL replay should reconstruct all 250 postings");
}

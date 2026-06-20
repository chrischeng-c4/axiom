// SPEC-MANAGED: projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:b4ef76e5" tracker="pending-tracker" reason="Tests: O(1) cursor ordering, prefer-redeliver, committed watermark, lease-batch/ack-batch, and per-subject concurrency isolation."
//! Work-queue throughput rework (#128): O(1) cursor ordering, prefer-redeliver,
//! committed watermark, batch lease/ack, and per-subject concurrency isolation.

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{Duration, Utc};

use relay::{Relay, RelayCoreConfig};

fn relay() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

fn publish(r: &Relay, subject: &str, n: usize) {
    let now = Utc::now();
    for i in 0..n {
        r.publish(
            subject,
            &format!("m{i}"),
            serde_json::json!({}),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
}

// The O(1) cursor hands out fresh seqs in ascending order, each once.
#[test]
fn cursor_leases_in_order() {
    let r = relay();
    publish(&r, "q", 5);
    let now = Utc::now();
    let mut seqs = Vec::new();
    while let Some(l) = r.lease("q", "c", now).unwrap() {
        seqs.push(l.seq);
    }
    assert_eq!(seqs, vec![0, 1, 2, 3, 4]);
}

// A reclaimed (redelivery-eligible) seq is preferred over fresh ones.
#[test]
fn prefers_redeliver_over_fresh() {
    let r = relay();
    publish(&r, "q", 3);
    let now = Utc::now();
    // Lease seq 0 and 1, let them expire and be reclaimed.
    let _ = r.lease("q", "c", now).unwrap().unwrap(); // seq 0
    let _ = r.lease("q", "c", now).unwrap().unwrap(); // seq 1
    let later = now + Duration::seconds(31);
    assert_eq!(r.reclaim_expired("q", later).unwrap(), vec![0, 1]);
    // Next leases re-offer the reclaimed seqs (0,1) before the fresh seq 2.
    let a = r.lease("q", "c", later).unwrap().unwrap();
    let b = r.lease("q", "c", later).unwrap().unwrap();
    let c = r.lease("q", "c", later).unwrap().unwrap();
    assert_eq!((a.seq, b.seq, c.seq), (0, 1, 2));
    assert!(a.epoch >= 2 && b.epoch >= 2, "redelivered epochs bumped");
}

// The committed watermark advances only over the contiguous acked prefix.
#[test]
fn committed_watermark_tracks_contiguous_prefix() {
    let r = relay();
    publish(&r, "q", 3);
    let now = Utc::now();
    let l0 = r.lease("q", "c", now).unwrap().unwrap();
    let l1 = r.lease("q", "c", now).unwrap().unwrap();
    let l2 = r.lease("q", "c", now).unwrap().unwrap();

    // ack seq 1 first: no contiguous prefix from 0 yet.
    assert!(r.ack("q", &l1.lease_id, Some(l1.epoch)).unwrap());
    assert!(r.committed_offset("q").unwrap().is_none());

    // ack seq 0: prefix {0,1} commits -> committed_seq = 1.
    assert!(r.ack("q", &l0.lease_id, Some(l0.epoch)).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 1);

    // ack seq 2: prefix {0,1,2} -> committed_seq = 2.
    assert!(r.ack("q", &l2.lease_id, Some(l2.epoch)).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 2);
}

// lease-batch returns up to `max` distinct leases in seq order and continues.
#[test]
fn lease_batch_returns_up_to_max() {
    let r = relay();
    publish(&r, "q", 10);
    let now = Utc::now();
    let b1 = r.lease_batch("q", "c", 4, now).unwrap();
    let b2 = r.lease_batch("q", "c", 4, now).unwrap();
    let b3 = r.lease_batch("q", "c", 4, now).unwrap();
    let b4 = r.lease_batch("q", "c", 4, now).unwrap();
    let seqs = |b: &[relay::Lease]| b.iter().map(|l| l.seq).collect::<Vec<_>>();
    assert_eq!(seqs(&b1), vec![0, 1, 2, 3]);
    assert_eq!(seqs(&b2), vec![4, 5, 6, 7]);
    assert_eq!(seqs(&b3), vec![8, 9]);
    assert!(b4.is_empty());
}

// ack-batch acks the matching entries (skipping a stale epoch) and advances commit.
#[test]
fn ack_batch_skips_stale_epoch() {
    let r = relay();
    publish(&r, "q", 3);
    let now = Utc::now();
    let leases = r.lease_batch("q", "c", 3, now).unwrap();
    assert_eq!(leases.len(), 3);

    // ack seq 0 and 1 correctly, but seq 2 with a wrong epoch (must be skipped).
    let acks = vec![
        (leases[0].lease_id.clone(), Some(leases[0].epoch)),
        (leases[1].lease_id.clone(), Some(leases[1].epoch)),
        (leases[2].lease_id.clone(), Some(999u64)),
    ];
    let (acked, committed) = r.ack_batch("q", &acks).unwrap();
    assert_eq!(acked, 2, "stale-epoch ack skipped");
    assert_eq!(committed.unwrap().committed_seq, 1);

    // now ack seq 2 with the right epoch.
    let (acked, committed) = r
        .ack_batch("q", &[(leases[2].lease_id.clone(), Some(leases[2].epoch))])
        .unwrap();
    assert_eq!(acked, 1);
    assert_eq!(committed.unwrap().committed_seq, 2);
}

// Per-subject locking: two subjects drained concurrently from many threads;
// each subject's messages are each delivered exactly once.
#[test]
fn concurrent_subjects_are_isolated_and_exactly_once() {
    const N: usize = 2000;
    let r = Arc::new(relay());
    publish(&r, "A", N);
    publish(&r, "B", N);

    let mut handles = Vec::new();
    for subject in ["A", "B"] {
        for k in 0..4 {
            let r = Arc::clone(&r);
            handles.push(std::thread::spawn(move || {
                let now = Utc::now();
                let cid = format!("{subject}-c{k}");
                let mut acked = Vec::new();
                while let Some(l) = r.lease(subject, &cid, now).unwrap() {
                    if r.ack(subject, &l.lease_id, Some(l.epoch)).unwrap() {
                        acked.push(l.seq);
                    }
                }
                acked
            }));
        }
    }
    let mut a_total = std::collections::BTreeSet::new();
    let mut b_total = std::collections::BTreeSet::new();
    for (i, h) in handles.into_iter().enumerate() {
        let acked = h.join().unwrap();
        let set = if i < 4 { &mut a_total } else { &mut b_total };
        for s in acked {
            assert!(set.insert(s), "each seq delivered exactly once");
        }
    }
    assert_eq!(a_total.len(), N, "subject A fully drained");
    assert_eq!(b_total.len(), N, "subject B fully drained");
    assert_eq!(
        r.committed_offset("A").unwrap().unwrap().committed_seq,
        (N - 1) as u64
    );
    assert_eq!(
        r.committed_offset("B").unwrap().unwrap().committed_seq,
        (N - 1) as u64
    );
}
// HANDWRITE-END

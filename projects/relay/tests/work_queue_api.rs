// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/work-queue-api-lease-ack-heartbeat.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:8b15e52b" tracker="pending-tracker" reason="Tests for prefer-redeliver lease pick, epoch-fenced + idempotent ack, and heartbeat extend / fence."
//! Work-queue API semantics (#113): prefer-redeliver lease pick, epoch-fenced
//! and idempotent ack, and heartbeat extend / fence — exercised through the
//! Relay core facade.

use std::collections::BTreeMap;

use chrono::{Duration, Utc};

use relay::{Relay, RelayCoreConfig};

fn relay() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

fn publish(r: &mut Relay, subject: &str, id: &str) {
    r.publish(
        subject,
        id,
        serde_json::json!({}),
        BTreeMap::new(),
        Utc::now(),
    )
    .unwrap();
}

// lease prefers a redeliver-eligible seq over a fresh one.
#[test]
fn lease_prefers_redeliver() {
    let mut r = relay();
    let now = Utc::now();
    publish(&mut r, "q", "m0");
    publish(&mut r, "q", "m1");

    // seq 0 leased then reclaimed -> redeliver-eligible.
    let l0 = r.lease("q", "c1", now).unwrap().unwrap();
    assert_eq!(l0.seq, 0);
    assert_eq!(l0.epoch, 1);
    let later = now + Duration::seconds(31);
    assert_eq!(r.reclaim_expired("q", later).unwrap(), vec![0]);

    // next lease must prefer the redeliver-eligible seq 0 over the fresh seq 1.
    let l = r.lease("q", "c2", later).unwrap().unwrap();
    assert_eq!(l.seq, 0, "prefers redeliver-eligible");
    assert_eq!(l.epoch, 2, "epoch bumped on re-lease");
}

// ack is epoch-fenced: a stale epoch (or unknown lease) is a no-op.
#[test]
fn ack_is_epoch_fenced() {
    let mut r = relay();
    let now = Utc::now();
    publish(&mut r, "q", "m0");

    let old = r.lease("q", "c1", now).unwrap().unwrap();
    let later = now + Duration::seconds(31);
    r.reclaim_expired("q", later).unwrap();
    let new = r.lease("q", "c2", later).unwrap().unwrap();

    // old worker's late ack (old lease id / old epoch) is rejected.
    assert!(!r.ack("q", &old.lease_id, Some(old.epoch)).unwrap());
    // a present lease with a mismatched epoch is also rejected.
    assert!(!r.ack("q", &new.lease_id, Some(999)).unwrap());
    // the live grant acks.
    assert!(r.ack("q", &new.lease_id, Some(new.epoch)).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 0);
}

// ack is idempotent: a second ack of the same lease is a no-op.
#[test]
fn ack_is_idempotent() {
    let mut r = relay();
    let now = Utc::now();
    publish(&mut r, "q", "m0");
    let l = r.lease("q", "c1", now).unwrap().unwrap();
    assert!(r.ack("q", &l.lease_id, Some(l.epoch)).unwrap());
    assert!(!r.ack("q", &l.lease_id, Some(l.epoch)).unwrap());
}

// heartbeat extends a live lease so it is not reclaimed at the original ttl.
#[test]
fn heartbeat_extends_lease() {
    let mut r = relay();
    let now = Utc::now();
    publish(&mut r, "q", "m0");
    let l = r.lease("q", "c1", now).unwrap().unwrap();

    // heartbeat just before the ttl pushes expiry out by another ttl.
    let hb_at = now + Duration::seconds(29);
    assert!(r
        .heartbeat("q", &l.lease_id, l.epoch, hb_at)
        .unwrap()
        .is_some());

    // past the ORIGINAL ttl the lease is still held (not reclaimed).
    let past_original = now + Duration::seconds(31);
    assert!(r.reclaim_expired("q", past_original).unwrap().is_empty());
}

// heartbeat is fenced: unknown lease / stale epoch returns None.
#[test]
fn heartbeat_is_fenced() {
    let mut r = relay();
    let now = Utc::now();
    publish(&mut r, "q", "m0");
    let l = r.lease("q", "c1", now).unwrap().unwrap();

    // wrong epoch on a live lease.
    assert!(r.heartbeat("q", &l.lease_id, 999, now).unwrap().is_none());

    // after reclaim the old lease id is gone -> fenced.
    let later = now + Duration::seconds(31);
    r.reclaim_expired("q", later).unwrap();
    assert!(r
        .heartbeat("q", &l.lease_id, l.epoch, later)
        .unwrap()
        .is_none());
}
// HANDWRITE-END

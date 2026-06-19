// SPEC-MANAGED: projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:361a28a7" tracker="pending-tracker" reason="Tests: dead-worker redeliver + complete, late-ack fenced, live-worker kept, frontier-only, background-task auto-reclaim."
//! Reconciler tests (#109): a dead worker's lease is reclaimed and redelivered
//! (epoch-fenced), a live worker is kept, only the in-flight frontier is swept,
//! and the background task auto-reclaims over time.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};

use relay::{spawn_reconciler, Relay, RelayCoreConfig};

fn relay() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

fn publish(r: &Relay, subject: &str, id: &str) {
    r.publish(
        subject,
        id,
        serde_json::json!({}),
        BTreeMap::new(),
        Utc::now(),
    )
    .unwrap();
}

// #109 acceptance: a killed worker's range is redelivered and completes; the
// late ack from the dead worker is rejected by epoch.
#[test]
fn dead_worker_redelivered_and_late_ack_fenced() {
    let r = relay();
    let now = Utc::now();
    publish(&r, "q", "m0");

    // c1 leases and then "dies" (never acks).
    let dead = r.lease("q", "c1", now).unwrap().unwrap();

    // Reconcile after the lease expires -> the seq is reclaimed.
    let later = now + ChronoDuration::seconds(31);
    assert_eq!(r.reconcile(later), 1);

    // c2 picks it up (redelivered, epoch bumped) and completes.
    let live = r.lease("q", "c2", later).unwrap().unwrap();
    assert_eq!(live.seq, 0);
    assert!(live.epoch > dead.epoch, "epoch bumped on redeliver");
    assert!(r.ack("q", &live.lease_id, Some(live.epoch)).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 0);

    // The dead worker's late ack is fenced (no double-completion).
    assert!(!r.ack("q", &dead.lease_id, Some(dead.epoch)).unwrap());
}

// A worker that heartbeats is kept: reconcile does not reclaim its lease.
#[test]
fn live_worker_is_kept() {
    let r = relay();
    let now = Utc::now();
    publish(&r, "q", "m0");
    let l = r.lease("q", "c1", now).unwrap().unwrap();

    // heartbeat just before the ttl extends the lease.
    let hb_at = now + ChronoDuration::seconds(29);
    assert!(r
        .heartbeat("q", &l.lease_id, l.epoch, hb_at)
        .unwrap()
        .is_some());

    // reconcile past the ORIGINAL ttl reclaims nothing (worker is alive).
    assert_eq!(r.reconcile(now + ChronoDuration::seconds(31)), 0);
}

// Reconcile sweeps only the in-flight frontier: acked entries are never
// re-offered, only the still-leased expired ones are reclaimed.
#[test]
fn reconcile_is_frontier_only() {
    let r = relay();
    let now = Utc::now();
    publish(&r, "q", "m0");
    publish(&r, "q", "m1");

    // seq 0 acked; seq 1 leased and left to expire.
    let l0 = r.lease("q", "c1", now).unwrap().unwrap();
    assert!(r.ack("q", &l0.lease_id, Some(l0.epoch)).unwrap());
    let _l1 = r.lease("q", "c1", now).unwrap().unwrap();

    let later = now + ChronoDuration::seconds(31);
    assert_eq!(
        r.reconcile(later),
        1,
        "only the in-flight seq 1 is reclaimed"
    );

    // the redelivered entry is seq 1, never the acked seq 0.
    let l = r.lease("q", "c2", later).unwrap().unwrap();
    assert_eq!(l.seq, 1);
}

// The background reconciler auto-reclaims an expired lease without any manual
// reconcile call.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn background_reconciler_auto_reclaims() {
    let mut core = RelayCoreConfig::in_memory();
    core.work_queue.lease_ttl_ms = 50;
    let relay = Arc::new(Relay::new(core));

    relay
        .publish(
            "q",
            "m0",
            serde_json::json!({}),
            BTreeMap::new(),
            Utc::now(),
        )
        .unwrap();
    // lease and "die" (never ack).
    let _ = relay.lease("q", "c1", Utc::now()).unwrap().unwrap();
    // nothing else is leasable while it is held.
    assert!(relay.lease("q", "c2", Utc::now()).unwrap().is_none());

    let handle = spawn_reconciler(Arc::clone(&relay), Duration::from_millis(20));

    // Give the reconciler several ticks past the 50ms lease ttl.
    tokio::time::sleep(Duration::from_millis(300)).await;

    let leased = relay.lease("q", "c2", Utc::now()).unwrap();
    handle.stop();
    assert!(
        leased.is_some(),
        "expired lease auto-reclaimed and redelivered"
    );
    assert_eq!(leased.unwrap().seq, 0);
}
// HANDWRITE-END

// SPEC-MANAGED: projects/relay/tech-design/logic/reconciler-lease-reclaim-redeliver-liveness.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:delayed-eta" tracker="pending-tracker" reason="Tests: delayed/ETA delivery — a future-dated entry is not leasable until due, an immediate sibling is leasable at once, reconcile promotes a due entry, and the delay survives a restart."
//! Delayed / ETA / countdown delivery (Phase 3): a future-dated entry is durably
//! appended but withheld from the work queue until its `not_before`; `promote_due`
//! (driven by the next lease or a reconcile tick) releases it. An immediate
//! sibling is leasable at once, and the delay survives a restart.

use std::collections::BTreeMap;

use chrono::{Duration, Utc};

use relay::{Relay, RelayCoreConfig};

fn disk_cfg(dir: &std::path::Path) -> RelayCoreConfig {
    let mut c = RelayCoreConfig::default();
    c.data_dir = dir.to_string_lossy().into_owned();
    c
}

// A future-dated entry is withheld until its not_before; a non-delayed sibling
// is leasable immediately; the delayed entry is leasable once due.
#[test]
fn delayed_entry_not_leasable_until_due() {
    let r = Relay::new(RelayCoreConfig::in_memory());
    let now = Utc::now();
    let due = now + Duration::seconds(60);

    r.publish_at(
        "q",
        "delayed",
        serde_json::json!({ "t": "later" }),
        BTreeMap::new(),
        Some(due),
        0,
        now,
    )
    .unwrap();
    r.publish(
        "q",
        "immediate",
        serde_json::json!({ "t": "now" }),
        BTreeMap::new(),
        now,
    )
    .unwrap();

    // The first lease skips the not-yet-due entry and returns the immediate one.
    let l = r.lease("q", "c", now).unwrap().unwrap();
    let body = r.entry("q", l.shard, l.seq).unwrap().unwrap();
    assert_eq!(body.message_id, "immediate");
    assert!(r.ack("q", &l.lease_id, Some(l.epoch)).unwrap());

    // Before the due time nothing else is leasable.
    assert!(r
        .lease("q", "c", now + Duration::seconds(30))
        .unwrap()
        .is_none());

    // At/after the due time the delayed entry becomes leasable.
    let after = due + Duration::seconds(1);
    let l2 = r.lease("q", "c", after).unwrap().unwrap();
    let body2 = r.entry("q", l2.shard, l2.seq).unwrap().unwrap();
    assert_eq!(body2.message_id, "delayed");
}

// reconcile promotes a due entry so an idle queue is woken without an explicit
// lease attempt; before due, it promotes nothing.
#[test]
fn reconcile_promotes_due_entry() {
    let r = Relay::new(RelayCoreConfig::in_memory());
    let now = Utc::now();
    let due = now + Duration::seconds(10);
    r.publish_at("q", "d", serde_json::json!({}), BTreeMap::new(), Some(due), 0, now)
        .unwrap();

    // Not due yet.
    r.reconcile(now);
    assert!(r.lease("q", "c", now).unwrap().is_none());

    // Past due: reconcile releases it, and it then leases.
    r.reconcile(due + Duration::seconds(1));
    let l = r.lease("q", "c", due + Duration::seconds(1)).unwrap().unwrap();
    assert_eq!(r.entry("q", l.shard, l.seq).unwrap().unwrap().message_id, "d");
}

// The delay survives a restart: a future-dated entry is still withheld until due
// after the relay is reopened from disk (the delay index is rebuilt on recover).
#[test]
fn delay_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let due = now + Duration::seconds(60);
    {
        let r = Relay::new(disk_cfg(dir.path()));
        r.publish_at("q", "d", serde_json::json!({}), BTreeMap::new(), Some(due), 0, now)
            .unwrap();
    }
    // Reopen: the delayed entry must still be held until due.
    let r2 = Relay::new(disk_cfg(dir.path()));
    assert!(
        r2.lease("q", "c", now + Duration::seconds(30))
            .unwrap()
            .is_none(),
        "delay survives restart"
    );
    let l = r2.lease("q", "c", due + Duration::seconds(1)).unwrap().unwrap();
    assert_eq!(r2.entry("q", l.shard, l.seq).unwrap().unwrap().message_id, "d");
}
// HANDWRITE-END

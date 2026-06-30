// SPEC-MANAGED: projects/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:priority-bands" tracker="pending-tracker" reason="Tests: priority bands — higher priority leases first, a later high-priority entry preempts an earlier low one, same-band order is FIFO, and out-of-range priority clamps to the top band."
//! Work-queue priority bands (Phase 5): `pick` scans bands high → low, so a
//! higher-priority entry leases before a lower one even if published later;
//! within a band the order is FIFO (publish order); an out-of-range priority
//! clamps into the top band.

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Relay, RelayCoreConfig};

fn relay() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

fn pub_p(r: &Relay, subject: &str, id: &str, priority: u8) {
    r.publish_at(
        subject,
        id,
        serde_json::json!({ "id": id }),
        BTreeMap::new(),
        None,
        priority,
        Utc::now(),
    )
    .unwrap();
}

// Drain the subject by leasing+acking and return the message_id order.
fn drain(r: &Relay, subject: &str, n: usize) -> Vec<String> {
    let now = Utc::now();
    (0..n)
        .map(|_| {
            let l = r.lease(subject, "c", now).unwrap().unwrap();
            let id = r.entry(subject, l.shard, l.seq).unwrap().unwrap().message_id;
            assert!(r.ack(subject, &l.lease_id, Some(l.epoch)).unwrap());
            id
        })
        .collect()
}

// Higher priority leases first; a later-published high-priority entry preempts an
// earlier low-priority one; within a band, order is FIFO (publish order).
#[test]
fn higher_priority_leases_first_same_band_fifo() {
    let r = relay();
    pub_p(&r, "q", "low-a", 0);
    pub_p(&r, "q", "high", 5); // published after low-a but higher band
    pub_p(&r, "q", "low-b", 0);
    pub_p(&r, "q", "mid", 2);

    assert_eq!(
        drain(&r, "q", 4),
        vec!["high", "mid", "low-a", "low-b"],
        "bands high->low; same-band FIFO"
    );
}

// A priority beyond the band count clamps into the top band (FIFO with other
// top-band entries), never panics or routes out of range.
#[test]
fn out_of_range_priority_clamps_to_top_band() {
    let r = relay();
    pub_p(&r, "q", "max", 200); // clamps to the top band
    pub_p(&r, "q", "top", 7); // PRIORITY_BANDS - 1, same top band
    pub_p(&r, "q", "low", 0);

    assert_eq!(
        drain(&r, "q", 3),
        vec!["max", "top", "low"],
        "200 clamps into the top band, FIFO with 7"
    );
}
// HANDWRITE-END

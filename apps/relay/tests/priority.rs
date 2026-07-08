// SPEC-MANAGED: apps/relay/tech-design/logic/work-queue-throughput-per-shard-lock-o-1-lease-cursor-batch-leas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:priority-bands" tracker="pending-tracker" reason="Tests: u8 priority bands — higher priority leases first, default priority is 10, batch publish preserves per-message priority, same-band order is FIFO, and u8::MAX is the top band."
//! Work-queue priority bands (Phase 5): `pick` scans bands high → low, so a
//! higher-priority entry leases before a lower one even if published later;
//! within a band the order is FIFO (publish order); u8 priority values are
//! distinct bands from 0 through 255.

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{Relay, RelayCoreConfig, DEFAULT_PRIORITY};

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

fn item_p(id: &str, priority: u8) -> (String, serde_json::Value, BTreeMap<String, String>, u8) {
    (
        id.to_string(),
        serde_json::json!({ "id": id }),
        BTreeMap::new(),
        priority,
    )
}

// Drain the subject by leasing+acking and return the message_id order.
fn drain(r: &Relay, subject: &str, n: usize) -> Vec<String> {
    let now = Utc::now();
    (0..n)
        .map(|_| {
            let l = r.lease(subject, "c", now).unwrap().unwrap();
            let id = r
                .entry(subject, l.shard, l.seq)
                .unwrap()
                .unwrap()
                .message_id;
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

#[test]
fn default_priority_is_baseline_10() {
    let r = relay();
    pub_p(&r, "q", "below-default", DEFAULT_PRIORITY - 1);
    r.publish(
        "q",
        "default",
        serde_json::json!({ "id": "default" }),
        BTreeMap::new(),
        Utc::now(),
    )
    .unwrap();
    pub_p(&r, "q", "above-default", DEFAULT_PRIORITY + 1);

    assert_eq!(
        drain(&r, "q", 3),
        vec!["above-default", "default", "below-default"],
        "Relay::publish uses the shared default priority"
    );
}

#[test]
fn publish_batch_keeps_per_message_priority() {
    let r = relay();
    r.publish_batch(
        "q",
        vec![
            item_p("low", 1),
            item_p("high", 200),
            item_p("normal", DEFAULT_PRIORITY),
        ],
        Utc::now(),
    )
    .unwrap();

    assert_eq!(
        drain(&r, "q", 3),
        vec!["high", "normal", "low"],
        "batch publish must preserve each message priority"
    );
}

// The u8 maximum priority leases ahead of lower priorities and stays FIFO with
// other max-priority entries.
#[test]
fn max_priority_is_highest_band() {
    let r = relay();
    pub_p(&r, "q", "max-a", 255);
    pub_p(&r, "q", "max-b", 255);
    pub_p(&r, "q", "low", 0);

    assert_eq!(
        drain(&r, "q", 3),
        vec!["max-a", "max-b", "low"],
        "255 is the highest band and remains FIFO within the band"
    );
}
// HANDWRITE-END

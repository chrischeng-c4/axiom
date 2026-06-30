// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:consumer-groups" tracker="pending-tracker" reason="Tests: named consumer groups (multicast) — each group independently receives every message, competing consumers within a group split the work, and delete-on-ack GC is pinned by the slowest group."
//! Named consumer groups / multicast (Phase 6): each subject can have multiple
//! independent competing-consumer groups over the shared log. Each group receives
//! every message once; within a group, consumers compete. Delete-on-ack GC
//! truncates below the MIN committed watermark across groups (a lagging group
//! pins the head).

use std::collections::{BTreeMap, HashSet};

use chrono::Utc;

use relay::{Relay, RelayCoreConfig, RetentionMode};

fn item(id: &str) -> (String, serde_json::Value, BTreeMap<String, String>) {
    (id.to_string(), serde_json::json!({ "id": id }), BTreeMap::new())
}

fn pub1(r: &Relay, subject: &str, id: &str) {
    r.publish(subject, id, serde_json::json!({}), BTreeMap::new(), Utc::now())
        .unwrap();
}

fn segmented_ack_cfg(dir: &std::path::Path) -> RelayCoreConfig {
    let mut c = RelayCoreConfig::default();
    c.data_dir = dir.to_string_lossy().into_owned();
    c.segment_bytes = 200;
    c.ram_ring_entries = 4;
    c
}

fn count_segments(dir: &std::path::Path, subject: &str) -> usize {
    let prefix = format!("{subject}__shard0__");
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name();
            let n = n.to_string_lossy();
            n.starts_with(&prefix) && n.ends_with(".ndjson")
        })
        .count()
}

// Drain a group fully (lease+ack each), returning the message_id order.
fn drain_group(r: &Relay, subject: &str, group: &str, n: usize) -> Vec<String> {
    let now = Utc::now();
    (0..n)
        .map(|_| {
            let l = r.lease_in(subject, group, "c", now).unwrap().unwrap();
            let id = r.entry(subject, l.shard, l.seq).unwrap().unwrap().message_id;
            assert!(r.ack_in(subject, group, &l.lease_id, Some(l.epoch)).unwrap());
            id
        })
        .collect()
}

// Two named groups each independently receive every message (multicast fan-out).
#[test]
fn two_groups_each_receive_every_message() {
    let r = Relay::new(RelayCoreConfig::in_memory());
    let now = Utc::now();
    for i in 0..3 {
        pub1(&r, "q", &format!("m{i}"));
    }

    assert_eq!(drain_group(&r, "q", "stage-a", 3), vec!["m0", "m1", "m2"]);
    assert_eq!(drain_group(&r, "q", "stage-b", 3), vec!["m0", "m1", "m2"]);
    assert!(r.lease_in("q", "stage-a", "c", now).unwrap().is_none());
    assert!(r.lease_in("q", "stage-b", "c", now).unwrap().is_none());
}

// Within one group, competing consumers split the work — each message leased once.
#[test]
fn competing_consumers_share_one_group() {
    let r = Relay::new(RelayCoreConfig::in_memory());
    let now = Utc::now();
    for i in 0..4 {
        pub1(&r, "q", &format!("m{i}"));
    }

    let seqs: HashSet<u64> = ["c1", "c2", "c1", "c2"]
        .iter()
        .map(|c| r.lease_in("q", "workers", c, now).unwrap().unwrap().seq)
        .collect();
    assert_eq!(seqs.len(), 4, "each message leased exactly once across the group");
    assert!(
        r.lease_in("q", "workers", "c1", now).unwrap().is_none(),
        "group drained"
    );
}

// Delete-on-ack GC is pinned by the slowest group: the log is not truncated until
// EVERY group has acked, then truncation proceeds.
#[test]
fn lagging_group_pins_delete_on_ack_gc() {
    let dir = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let r = Relay::new(segmented_ack_cfg(dir.path()));
    r.set_retention_mode("q", RetentionMode::Ack).unwrap();
    // Create group "b" up front (empty lease) so it counts toward GC before publish.
    assert!(r.lease_in("q", "b", "cb", now).unwrap().is_none());
    r.publish_batch("q", (0..20).map(|i| item(&format!("m{i}"))).collect(), now)
        .unwrap();
    let before = count_segments(dir.path(), "q");
    assert!(before > 1, "rolled into multiple segments");

    // Group "a" fully consumes; "b" lags → the min watermark pins the head.
    drain_group(&r, "q", "a", 20);
    assert_eq!(
        count_segments(dir.path(), "q"),
        before,
        "lagging group 'b' pins truncation"
    );

    // Once "b" also drains, the min advances and the prefix is reclaimed.
    drain_group(&r, "q", "b", 20);
    assert!(
        count_segments(dir.path(), "q") < before,
        "truncates only after every group acked"
    );
}
// HANDWRITE-END

// SPEC-MANAGED: projects/relay/tech-design/logic/multi-shard-per-subject-server-side-sharding-horizontal-scale.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:4a1c74a7" tracker="pending-tracker" reason="Tests: publish spread across shards, whole-subject exactly-once drain across shards, broadcast merge across shards, and default_shards=1 parity with single-shard semantics."
//! Multi-shard per subject (#132): routing spreads a subject across shards, the
//! whole subject still drains exactly-once and broadcasts in full, and
//! default_shards=1 is identical to single-shard.

use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;

use relay::{DeliveryModel, Relay, RelayCoreConfig};

fn relay(shards: u32) -> Relay {
    let mut cfg = RelayCoreConfig::in_memory();
    cfg.default_shards = shards;
    Relay::new(cfg)
}

fn publish(r: &Relay, subject: &str, n: usize) {
    let now = Utc::now();
    for i in 0..n {
        r.publish(
            subject,
            &format!("m{i}"),
            serde_json::json!({ "i": i }),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
}

// Publishing spreads across shards, and the whole subject still drains
// exactly-once through lease/ack (each shard has its own seq space).
#[test]
fn spreads_and_drains_exactly_once() {
    let r = relay(4);
    const N: usize = 200;
    publish(&r, "q", N);
    assert_eq!(
        r.log_len("q").unwrap(),
        N as u64,
        "all messages stored across shards"
    );

    let now = Utc::now();
    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
    let mut shards_used: BTreeSet<u32> = BTreeSet::new();
    while let Some(l) = r.lease("q", "c", now).unwrap() {
        assert!(
            seen.insert((l.shard, l.seq)),
            "each (shard,seq) leased once"
        );
        shards_used.insert(l.shard);
        assert!(r.ack("q", &l.lease_id, Some(l.epoch)).unwrap());
    }
    assert_eq!(seen.len(), N, "whole subject drained across shards");
    assert!(
        shards_used.len() > 1,
        "messages spread across multiple shards"
    );
}

// Broadcast subscribe spans all shards: every message is delivered (merged).
#[test]
fn broadcast_merges_across_shards() {
    let r = relay(4);
    const N: usize = 200;
    r.subscribe("events", "watcher", 0).unwrap();
    publish(&r, "events", N);

    let got = r.poll("events", "watcher").unwrap();
    let ids: BTreeSet<String> = got.iter().map(|e| e.message_id.clone()).collect();
    let expected: BTreeSet<String> = (0..N).map(|i| format!("m{i}")).collect();
    assert_eq!(
        ids, expected,
        "every message delivered, merged across shards"
    );
}

// default_shards = 1 collapses to a single shard 0 — identical to single-shard.
#[test]
fn single_shard_parity() {
    let r = relay(1);
    let now = Utc::now();
    r.set_delivery_model("s", DeliveryModel::Broadcast).unwrap();
    for id in ["m0", "m1", "m2"] {
        r.publish("s", id, serde_json::json!({}), BTreeMap::new(), now)
            .unwrap();
    }
    // single seq space 0,1,2 on shard 0
    let mut acked = Vec::new();
    while let Some(l) = r.lease("s", "c", now).unwrap() {
        assert_eq!(l.shard, 0);
        acked.push(l.seq);
        assert!(r.ack("s", &l.lease_id, Some(l.epoch)).unwrap());
    }
    assert_eq!(acked, vec![0, 1, 2]);
    assert_eq!(r.committed_offset("s").unwrap().unwrap().committed_seq, 2);
    assert_eq!(r.delivery_model("s"), Some(DeliveryModel::Broadcast));
}
// HANDWRITE-END

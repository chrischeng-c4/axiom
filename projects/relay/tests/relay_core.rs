// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:ac3b11e2" tracker="pending-tracker" reason="Deterministic tests for the unit-test plan, including the #122 acceptance (both delivery models over the same log)."
//! Deterministic tests for the relay core, mapping 1:1 to the unit-test plan in
//! the TD: sequencing, idempotency, broadcast fan-out + replay-from-seq,
//! work-queue single-delivery, lease-expiry redelivery, ack/commit, and the
//! #122 acceptance — both delivery models over the same durable log.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Utc};

use relay::{DeliveryModel, Payload, Relay, RelayCoreConfig};

fn msg(name: &str) -> Payload {
    serde_json::json!({ "task": name })
}

fn ram() -> Relay {
    Relay::new(RelayCoreConfig::in_memory())
}

// case: sequencing — append assigns monotonic, gap-free seq.
#[test]
fn append_assigns_monotonic_seq() {
    let r = ram();
    let now = Utc::now();
    for (i, id) in ["a", "b", "c"].iter().enumerate() {
        let out = r.publish("s", id, msg("t"), BTreeMap::new(), now).unwrap();
        assert_eq!(out.seq, i as u64, "seq is the append index");
        assert!(!out.deduped);
    }
    assert_eq!(r.log_len("s").unwrap(), 3);
}

// case: idempotency — a repeated id appends nothing and returns the same seq.
#[test]
fn idempotent_dedupe() {
    let r = ram();
    let now = Utc::now();
    let first = r
        .publish("s", "dup", msg("t"), BTreeMap::new(), now)
        .unwrap();
    let second = r
        .publish("s", "dup", msg("t"), BTreeMap::new(), now)
        .unwrap();
    assert_eq!(
        first,
        relay::AppendOutcome {
            seq: 0,
            deduped: false
        }
    );
    assert_eq!(
        second,
        relay::AppendOutcome {
            seq: 0,
            deduped: true
        }
    );
    assert_eq!(r.log_len("s").unwrap(), 1);
}

// case: fan-out — every broadcast subscriber receives every message in order.
#[test]
fn broadcast_fanout_all_subscribers() {
    let r = ram();
    let now = Utc::now();
    r.subscribe("s", "a", 0).unwrap();
    r.subscribe("s", "b", 0).unwrap();
    for id in ["m0", "m1", "m2"] {
        r.publish("s", id, msg("t"), BTreeMap::new(), now).unwrap();
    }
    let seqs = |v: Vec<relay::LogEntry>| v.iter().map(|e| e.seq).collect::<Vec<_>>();
    assert_eq!(seqs(r.poll("s", "a").unwrap()), vec![0, 1, 2]);
    assert_eq!(seqs(r.poll("s", "b").unwrap()), vec![0, 1, 2]);
    // caught up — nothing more until new messages arrive.
    assert!(r.poll("s", "a").unwrap().is_empty());
}

// case: replay — subscribing from a seq replays from there in order.
#[test]
fn broadcast_replay_from_seq() {
    let r = ram();
    let now = Utc::now();
    for id in ["m0", "m1", "m2", "m3"] {
        r.publish("s", id, msg("t"), BTreeMap::new(), now).unwrap();
    }
    r.subscribe("s", "late", 2).unwrap();
    let got: Vec<u64> = r.poll("s", "late").unwrap().iter().map(|e| e.seq).collect();
    assert_eq!(got, vec![2, 3]);
}

// case: competing — each message is leased to exactly one consumer.
#[test]
fn workqueue_single_delivery() {
    let r = ram();
    let now = Utc::now();
    for id in ["m0", "m1", "m2"] {
        r.publish("q", id, msg("t"), BTreeMap::new(), now).unwrap();
    }
    let mut leased: Vec<(String, u64)> = Vec::new();
    let mut toggle = true;
    loop {
        let consumer = if toggle { "c1" } else { "c2" };
        match r.lease("q", consumer, now).unwrap() {
            Some(l) => leased.push((consumer.to_string(), l.seq)),
            None => break,
        }
        toggle = !toggle;
    }
    let seqs: BTreeSet<u64> = leased.iter().map(|(_, s)| *s).collect();
    assert_eq!(seqs, BTreeSet::from([0, 1, 2]), "all messages leased");
    assert_eq!(
        leased.len(),
        seqs.len(),
        "no message leased to two consumers"
    );
}

// case: redelivery — an expired lease becomes redelivery-eligible, attempt++.
#[test]
fn lease_expiry_redelivers() {
    let r = ram();
    let now = Utc::now();
    r.publish("q", "m0", msg("t"), BTreeMap::new(), now)
        .unwrap();

    let first = r.lease("q", "c1", now).unwrap().expect("a lease");
    assert_eq!(first.attempt, 1);
    // nothing else available while it is leased
    assert!(r.lease("q", "c2", now).unwrap().is_none());

    // past the lease TTL, reclaim makes it eligible again
    let later = now
        + Duration::milliseconds(RelayCoreConfig::default().work_queue.lease_ttl_ms as i64)
        + Duration::seconds(1);
    assert_eq!(r.reclaim_expired("q", later).unwrap(), vec![0]);

    let redelivered = r.lease("q", "c2", later).unwrap().expect("redelivered");
    assert_eq!(redelivered.seq, 0);
    assert_eq!(redelivered.attempt, 2, "attempt carried forward");
    assert_eq!(redelivered.consumer_id, "c2");
}

// case: ack/commit — acking advances the committed offset and stops redelivery.
#[test]
fn ack_advances_committed_offset() {
    let r = ram();
    let now = Utc::now();
    r.publish("q", "m0", msg("t"), BTreeMap::new(), now)
        .unwrap();
    r.publish("q", "m1", msg("t"), BTreeMap::new(), now)
        .unwrap();

    assert!(r.committed_offset("q").unwrap().is_none());

    let l0 = r.lease("q", "c1", now).unwrap().unwrap();
    assert!(r.ack("q", &l0.lease_id, None).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 0);

    let l1 = r.lease("q", "c1", now).unwrap().unwrap();
    assert_eq!(l1.seq, 1, "acked entry is not re-offered");
    assert!(r.ack("q", &l1.lease_id, None).unwrap());
    assert_eq!(r.committed_offset("q").unwrap().unwrap().committed_seq, 1);
}

// acceptance (#122): one broadcast subscriber AND one work-queue consumer over
// the same subject/log.
#[test]
fn both_models_over_same_log() {
    let r = ram();
    let now = Utc::now();
    r.set_delivery_model("events", DeliveryModel::Broadcast)
        .unwrap();
    r.subscribe("events", "watcher", 0).unwrap();

    for id in ["e0", "e1", "e2"] {
        r.publish("events", id, msg("t"), BTreeMap::new(), now)
            .unwrap();
    }

    // broadcast subscriber sees every message
    let seen: Vec<u64> = r
        .poll("events", "watcher")
        .unwrap()
        .iter()
        .map(|e| e.seq)
        .collect();
    assert_eq!(seen, vec![0, 1, 2]);

    // work-queue consumer leases each exactly once over the SAME log
    let mut acked: BTreeSet<u64> = BTreeSet::new();
    while let Some(l) = r.lease("events", "worker", now).unwrap() {
        assert!(r.ack("events", &l.lease_id, None).unwrap());
        acked.insert(l.seq);
    }
    assert_eq!(acked, BTreeSet::from([0, 1, 2]));
    assert_eq!(
        r.committed_offset("events").unwrap().unwrap().committed_seq,
        2
    );
    assert_eq!(r.delivery_model("events"), Some(DeliveryModel::Broadcast));
}

// durability — entries and the dedupe index survive reopening from disk.
#[test]
fn durable_log_recovers_on_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = RelayCoreConfig::in_memory();
    cfg.data_dir = dir.path().to_string_lossy().into_owned();
    let now = Utc::now();

    {
        let r = Relay::new(cfg.clone());
        r.publish("s", "a", msg("t"), BTreeMap::new(), now).unwrap();
        r.publish("s", "b", msg("t"), BTreeMap::new(), now).unwrap();
    }

    let r2 = Relay::new(cfg);
    assert_eq!(r2.log_len("s").unwrap(), 2, "entries recovered from disk");
    let out = r2
        .publish("s", "a", msg("t"), BTreeMap::new(), now)
        .unwrap();
    assert!(out.deduped, "dedupe index recovered");
    assert_eq!(out.seq, 0);
}
// HANDWRITE-END

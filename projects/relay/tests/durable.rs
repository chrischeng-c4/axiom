// SPEC-MANAGED: projects/relay/tech-design/logic/default-durable-engine-throughput-group-commit-fsync-publish-bat.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:7e699a0c" tracker="pending-tracker" reason="Tests: default config is durable, publish_batch group commit + dedupe, log recovery on reopen, and committed-offset crash recovery."
//! Default-durable throughput (#129): durable-by-default config, group-commit
//! publish_batch with in-batch dedupe, durable-log recovery on reopen, and
//! committed-offset crash recovery (resume after the committed watermark).

use std::collections::BTreeMap;

use chrono::Utc;

use relay::{FsyncPolicy, Relay, RelayCoreConfig};

fn item(id: &str) -> (String, serde_json::Value, BTreeMap<String, String>) {
    (
        id.to_string(),
        serde_json::json!({ "id": id }),
        BTreeMap::new(),
    )
}

fn disk_config(dir: &std::path::Path) -> RelayCoreConfig {
    let mut cfg = RelayCoreConfig::default();
    cfg.data_dir = dir.to_string_lossy().into_owned();
    cfg
}

// relay is durable + power-safe out of the box.
#[test]
fn default_config_is_durable_and_power_safe() {
    let cfg = RelayCoreConfig::default();
    assert!(!cfg.data_dir.is_empty(), "default persists to disk");
    assert_eq!(
        cfg.fsync,
        FsyncPolicy::Always,
        "default fsync is power-safe"
    );
}

// publish_batch returns one outcome per message (group commit) and dedupes a
// repeated id within the batch.
#[test]
fn publish_batch_group_commit_and_dedupe() {
    let dir = tempfile::tempdir().unwrap();
    let r = Relay::new(disk_config(dir.path()));
    let now = Utc::now();
    let outcomes = r
        .publish_batch(
            "q",
            vec![item("m0"), item("m1"), item("m1"), item("m2")],
            now,
        )
        .unwrap();
    assert_eq!(outcomes.len(), 4);
    assert_eq!((outcomes[0].seq, outcomes[0].deduped), (0, false));
    assert_eq!((outcomes[1].seq, outcomes[1].deduped), (1, false));
    assert_eq!(
        (outcomes[2].seq, outcomes[2].deduped),
        (1, true),
        "in-batch dup"
    );
    assert_eq!((outcomes[3].seq, outcomes[3].deduped), (2, false));
    assert_eq!(r.log_len("q").unwrap(), 3);
}

// The durable log survives reopening from the same data_dir.
#[test]
fn durable_log_recovers_after_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let now = Utc::now();
    {
        let r = Relay::new(disk_config(dir.path()));
        r.publish_batch("q", vec![item("m0"), item("m1"), item("m2")], now)
            .unwrap();
    }
    let r2 = Relay::new(disk_config(dir.path()));
    assert_eq!(r2.log_len("q").unwrap(), 3, "entries recovered from disk");
    // a re-published id is still deduped after recovery.
    let out = r2.publish_batch("q", vec![item("m0")], now).unwrap();
    assert!(out[0].deduped);
}

// Crash recovery: the committed offset is durable, so a reopened relay resumes
// AFTER the committed watermark instead of redelivering everything.
#[test]
fn committed_offset_survives_crash() {
    let dir = tempfile::tempdir().unwrap();
    let now = Utc::now();
    {
        let r = Relay::new(disk_config(dir.path()));
        r.publish_batch("q", (0..5).map(|i| item(&format!("m{i}"))).collect(), now)
            .unwrap();
        // lease + ack the first 3 (durably committed via ack-batch).
        let leases = r.lease_batch("q", "c", 3, now).unwrap();
        assert_eq!(leases.len(), 3);
        let acks: Vec<_> = leases
            .iter()
            .map(|l| (l.lease_id.clone(), Some(l.epoch)))
            .collect();
        let (acked, committed) = r.ack_batch("q", &acks).unwrap();
        assert_eq!(acked, 3);
        assert_eq!(committed.unwrap().committed_seq, 2);
    }

    // Reopen: committed offset recovered; lease resumes at seq 3, not 0.
    let r2 = Relay::new(disk_config(dir.path()));
    assert_eq!(
        r2.committed_offset("q").unwrap().unwrap().committed_seq,
        2,
        "committed offset is durable"
    );
    let next = r2.lease("q", "c", now).unwrap().unwrap();
    assert_eq!(
        next.seq, 3,
        "resumes after the committed watermark (no redelivery of 0..2)"
    );
}
// HANDWRITE-END

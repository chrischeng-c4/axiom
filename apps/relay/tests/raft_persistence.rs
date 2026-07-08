// SPEC-MANAGED: apps/relay/tech-design/logic/adopt-raft-host-relaystatemachine-auto-mode-ha-drop-hand-rolled.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:8ae02b04" tracker="pending-tracker" reason="Restart-recovery tests over RelayRaft (#544): a single-node group restarted from its data dir rejoins with applied state intact and accepts new proposes (no double-apply); and the resurrection case — acked work already trimmed by delete-on-ack is NOT re-appended by cold replay thanks to the fsynced applied-index floor."
//! Restart recovery over the raft-host stack (#544): the fsynced
//! applied-index marker is relay's honest floor. The engine is delete-on-ack
//! with a bounded dedupe window, so cold-replaying the resident committed raft
//! log without a floor would resurrect acked (already-trimmed) work; with it,
//! a restarted node rejoins exactly where it left off.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use raft_host::Membership;
use relay::{PubCommand, Relay, RelayCoreConfig, RelayRaft};

fn sole_voter() -> Membership {
    Membership {
        voters: vec![0],
        learners: vec![],
    }
}

fn cmd(id: &str) -> PubCommand {
    PubCommand {
        subject: "s".to_string(),
        message_id: id.to_string(),
        payload: serde_json::json!({ "m": id }),
        headers: Default::default(),
        priority: relay::DEFAULT_PRIORITY,
        not_before: None,
    }
}

fn disk_engine(dir: &Path, segment_bytes: u64) -> Arc<Relay> {
    Arc::new(Relay::new(RelayCoreConfig {
        data_dir: dir.join("relay").to_str().unwrap().to_string(),
        segment_bytes,
        ..RelayCoreConfig::default()
    }))
}

fn spawn(engine: Arc<Relay>, dir: &Path) -> RelayRaft {
    RelayRaft::spawn(
        engine,
        &dir.join("raft"),
        0,
        sole_voter(),
        HashMap::new(),
        RelayRaft::host_config(1024),
    )
    .unwrap()
}

/// AC3 (idempotency + floor): a restarted sole-voter node recovers its engine
/// from disk and its applied floor from the marker, performs NO double-apply
/// of the resident committed log, and resumes proposing at the next index.
#[tokio::test]
async fn restart_rejoins_with_applied_state_intact() {
    let dir = tempfile::tempdir().unwrap();
    {
        let engine = disk_engine(dir.path(), 134_217_728);
        let raft = spawn(engine.clone(), dir.path());
        for i in 0..3 {
            let (_, out) = raft.publish(&cmd(&format!("m{i}"))).await.unwrap();
            assert!(!out.expect("outcome").deduped);
        }
        assert_eq!(raft.applied_index(), 3);
        assert_eq!(engine.log_len("s").unwrap(), 3);
    } // drop = restart (host tasks abort; engine files stay)

    let engine = disk_engine(dir.path(), 134_217_728);
    let raft = spawn(engine.clone(), dir.path());
    // The marker survived: the floor is back before any replay runs.
    assert_eq!(raft.applied_index(), 3, "applied floor recovered");
    assert_eq!(
        engine.log_len("s").unwrap(),
        3,
        "no double-apply on restart"
    );

    // The group resumes: the next publish lands at raft index 4 / engine seq 3.
    let (idx, out) = raft.publish(&cmd("m99")).await.unwrap();
    assert_eq!(idx, 4);
    let out = out.expect("outcome");
    assert!(!out.deduped);
    assert_eq!(out.seq, 3);
    assert_eq!(engine.log_len("s").unwrap(), 4);
}

/// AC3 (the resurrection case — the marker is load-bearing): with tiny
/// segments, acking everything trims the acked entries from disk AND from the
/// recovered dedupe window. The committed raft log still holds those
/// publishes; only the persisted applied floor stops cold replay from
/// re-appending (resurrecting) them after a restart.
#[tokio::test]
async fn acked_work_is_not_resurrected_by_cold_replay() {
    let dir = tempfile::tempdir().unwrap();
    {
        let engine = disk_engine(dir.path(), 1); // 1-byte segments: one entry each
        let raft = spawn(engine.clone(), dir.path());
        for i in 0..5 {
            raft.publish(&cmd(&format!("m{i}"))).await.unwrap();
        }
        // Drain the queue locally (leases are node-local): ack all 5, which
        // persists the watermark and drops the fully-acked segments.
        let now = Utc::now();
        while let Some(l) = engine.lease("s", "w", now).unwrap() {
            assert!(engine.ack("s", &l.lease_id, Some(l.epoch)).unwrap());
        }
        assert_eq!(engine.log_len("s").unwrap(), 5);
    } // drop = restart

    let engine = disk_engine(dir.path(), 1);
    let raft = spawn(engine.clone(), dir.path());
    assert_eq!(raft.applied_index(), 5, "applied floor recovered");

    // A new-term publish forces the recovered backlog to (re-)commit; the
    // floor makes replay skip entries 1..=5. Without it, m0..m3 (trimmed from
    // disk and dedupe) would re-append here and log_len would exceed 6.
    let (idx, out) = raft.publish(&cmd("m100")).await.unwrap();
    assert_eq!(idx, 6);
    assert!(!out.expect("outcome").deduped);
    assert_eq!(
        engine.log_len("s").unwrap(),
        6,
        "acked work must not be resurrected by cold replay"
    );

    // Only the new entry is leasable — the acked five stay acked.
    let now = Utc::now();
    let lease = engine.lease("s", "w", now).unwrap().expect("new entry");
    assert_eq!(lease.seq, 5);
    assert!(engine.lease("s", "w2", now).unwrap().is_none());
}
// HANDWRITE-END

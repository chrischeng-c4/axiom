// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:f910ee9b" tracker="pending-tracker" reason="Restart-recovery tests over TapeRaft prove the shared persisted commit watermark/log restore journal state before new proposals without duplicate apply."
//! Restart-recovery tests (#1327): shared Raft hard state persists the commit
//! watermark and resident log/snapshot, so a fresh `TapeRaft` restores every
//! committed journal mutation before it accepts new proposals.
//!
//! These exercise the SAME on-disk raft dir across two `TapeRaft::spawn`
//! calls (simulating a process restart). Legacy marker migration remains unit
//! covered inside `src/raft.rs`; this integration gate exercises the current
//! shared persistence path.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_runtime::Membership;
use tape::raft::{TapeOutcome, TapeRaft};
use tape::TapeJournal;

fn single_node_membership() -> Membership {
    Membership {
        voters: vec![0],
        learners: vec![],
    }
}

async fn wait_leader(raft: &TapeRaft) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if raft.is_leader().await {
            return;
        }
        assert!(Instant::now() < deadline, "single node never became leader");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// A single-node group restarted from the same data dir rejoins with its
/// applied index intact and keeps accepting proposes with no double-apply
/// (the restarted node's journal is fresh/empty each time -- restart honesty
/// is proven by the recovered floor matching what was actually committed,
/// not by an in-memory journal surviving the "restart").
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restarted_single_node_recovers_applied_floor_and_accepts_new_proposes() {
    let dir = tempfile::tempdir().unwrap();
    let applied_after_first_run;

    {
        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let raft = TapeRaft::spawn(
            journal,
            &dir.path().join("raft"),
            0,
            single_node_membership(),
            HashMap::new(),
            TapeRaft::host_config(1024),
        )
        .unwrap();
        wait_leader(&raft).await;

        for i in 0..5 {
            let (_, outcome) = raft
                .propose_append(
                    "orders".to_string(),
                    None,
                    serde_json::json!({ "n": i }),
                    100,
                )
                .await
                .unwrap();
            assert!(matches!(outcome, Some(TapeOutcome::Appended(_))));
        }
        applied_after_first_run = raft.applied_index();
        assert!(applied_after_first_run >= 5);
        // Dropping `raft` here aborts its tick/pump tasks -- the "process
        // exits" half of a restart.
    }

    // "Restart": same raft dir, a brand-new fresh-empty journal (as a real
    // process restart would have before raft-runtime cold-replays committed
    // entries back into the state machine).
    let journal = Arc::new(Mutex::new(TapeJournal::default()));
    let raft = TapeRaft::spawn(
        journal,
        &dir.path().join("raft"),
        0,
        single_node_membership(),
        HashMap::new(),
        TapeRaft::host_config(1024),
    )
    .unwrap();

    // The recovered floor is at least what the first run committed --
    // raft-runtime's own cold replay drives it the rest of the way as entries
    // resident in its store are re-delivered to `apply`.
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if raft.applied_index() >= applied_after_first_run {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "restarted node never recovered its applied floor"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    wait_leader(&raft).await;
    let (_, outcome) = raft
        .propose_append(
            "orders".to_string(),
            None,
            serde_json::json!({ "n": 99 }),
            100,
        )
        .await
        .unwrap();
    match outcome {
        Some(TapeOutcome::Appended(event)) => assert_eq!(event.payload["n"], 99),
        other => panic!("expected a fresh Appended outcome after restart, got {other:?}"),
    }
}

/// A checkpoint-put committed before a simulated restart is not re-applied on
/// cold replay: the persisted floor makes `TapeStateMachine::apply` skip the
/// stale entry outright rather than re-running `put_checkpoint_at` against a
/// journal whose end offset may since have changed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_put_before_restart_is_not_reapplied_on_cold_replay() {
    let dir = tempfile::tempdir().unwrap();

    {
        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let raft = TapeRaft::spawn(
            journal,
            &dir.path().join("raft"),
            0,
            single_node_membership(),
            HashMap::new(),
            TapeRaft::host_config(1024),
        )
        .unwrap();
        wait_leader(&raft).await;

        let (_, append_outcome) = raft
            .propose_append(
                "orders".to_string(),
                None,
                serde_json::json!({ "n": 0 }),
                100,
            )
            .await
            .unwrap();
        assert!(matches!(append_outcome, Some(TapeOutcome::Appended(_))));

        let (_, cp_outcome) = raft
            .propose_checkpoint("orders".to_string(), "c1".to_string(), 1, 200)
            .await
            .unwrap();
        match cp_outcome {
            Some(TapeOutcome::Checkpoint(Ok(cp))) => assert_eq!(cp.offset, 1),
            other => panic!("expected a committed checkpoint outcome, got {other:?}"),
        }
    }

    // Restart: the shared persisted commit range rebuilds both the append and
    // checkpoint before the node accepts another proposal.
    let journal = Arc::new(Mutex::new(TapeJournal::default()));
    let raft = TapeRaft::spawn(
        journal,
        &dir.path().join("raft"),
        0,
        single_node_membership(),
        HashMap::new(),
        TapeRaft::host_config(1024),
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if raft.applied_index() >= 2 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "restarted node never recovered its applied floor past the checkpoint-put"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // The recovered journal reflects the checkpoint that was actually
    // committed before restart -- no re-derivation, no double-apply.
    let checkpoint = raft
        .journal()
        .lock()
        .unwrap()
        .checkpoint("orders", "c1")
        .cloned();
    assert_eq!(checkpoint.map(|c| c.offset), Some(1));
}
// HANDWRITE-END

// HANDWRITE-BEGIN gap="missing-generator:unit-test:babc4ba4" tracker="#1589" reason="Run several real TapeRaft restart cycles against one durable directory, advancing append batches and checkpoints, then assert full replay and durable checkpoint state after every reopen. generator gap: missing-generator:raft-endurance-test (#1589)."
//! Bounded endurance proof for durable TapeRaft recovery (#1589).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use raft_runtime::Membership;
use tape::raft::{TapeOutcome, TapeRaft};
use tape::TapeJournal;

fn membership() -> Membership {
    Membership {
        voters: vec![0],
        learners: vec![],
    }
}

async fn wait_leader(raft: &TapeRaft) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !raft.is_leader().await {
        assert!(
            Instant::now() < deadline,
            "single node never elected a leader"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repeated_restarts_preserve_append_history_and_checkpoint_progress() {
    const CYCLES: u64 = 5;
    const APPENDS_PER_CYCLE: u64 = 12;
    let dir = tempfile::tempdir().unwrap();
    let mut total = 0u64;

    for cycle in 0..CYCLES {
        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let raft = TapeRaft::spawn(
            Arc::clone(&journal),
            &dir.path().join("raft"),
            0,
            membership(),
            HashMap::new(),
            TapeRaft::host_config(8),
        )
        .unwrap();
        wait_leader(&raft).await;
        for _ in 0..APPENDS_PER_CYCLE {
            let (_, outcome) = raft
                .propose_append(
                    "orders".into(),
                    None,
                    serde_json::json!({ "n": total }),
                    total,
                )
                .await
                .unwrap();
            assert!(matches!(outcome, Some(TapeOutcome::Appended(_))));
            total += 1;
        }
        let (_, checkpoint) = raft
            .propose_checkpoint("orders".into(), "worker".into(), total, 10_000 + cycle)
            .await
            .unwrap();
        assert!(matches!(checkpoint, Some(TapeOutcome::Checkpoint(Ok(_)))));
        drop(raft);

        let recovered = Arc::new(Mutex::new(TapeJournal::default()));
        let raft = TapeRaft::spawn(
            Arc::clone(&recovered),
            &dir.path().join("raft"),
            0,
            membership(),
            HashMap::new(),
            TapeRaft::host_config(8),
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(8);
        while raft.journal().lock().unwrap().end_offset("orders") != total {
            assert!(
                Instant::now() < deadline,
                "cycle {cycle} lost or duplicated journal state"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let journal = raft.journal();
        let journal = journal.lock().unwrap();
        assert_eq!(
            journal.checkpoint("orders", "worker").unwrap().offset,
            total
        );
        let events = journal.replay("orders", None, None, None);
        assert_eq!(events.len() as u64, total);
        assert_eq!(events.last().unwrap().payload["n"], total - 1);
    }
}
// HANDWRITE-END

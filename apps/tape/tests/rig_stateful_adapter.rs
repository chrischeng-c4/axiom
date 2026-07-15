// HANDWRITE-BEGIN gap="missing-generator:e2e-test:6ec544eb" tracker="#1645" reason="Bind Tape replay and checkpoint continuity assertions to the shared Rig stateful lifecycle. generator gap: missing-generator:tape-stateful-adapter (#1645)."
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, ensure};
use rig::engine::stateful::{run_stateful, StatefulActions, StatefulLimits, StatefulScenario};
use serde_json::json;
use tape::TapeJournal;

#[test]
fn tape_replay_continuity_uses_shared_stateful_runner() {
    let journal = Arc::new(Mutex::new(TapeJournal::default()));
    let unavailable = Arc::new(AtomicBool::new(false));
    let report = run_stateful(
        StatefulScenario::new(
            "tape-replay-continuity",
            StatefulActions {
                warmup: {
                    let journal = Arc::clone(&journal);
                    Box::new(move |evidence| {
                        let mut journal = journal
                            .lock()
                            .map_err(|_| anyhow!("journal lock poisoned"))?;
                        for n in 0..3 {
                            journal.append(
                                "orders",
                                Some(format!("order-{n}")),
                                json!({"sequence": n}),
                                Some(1_000 + n),
                            );
                        }
                        journal.put_checkpoint_at("orders", "worker", 2, 2_000)?;
                        evidence.record("append_window", json!({"events": 3, "checkpoint": 2}));
                        Ok(())
                    })
                },
                observe: {
                    let journal = Arc::clone(&journal);
                    Box::new(move |evidence| {
                        let journal = journal
                            .lock()
                            .map_err(|_| anyhow!("journal lock poisoned"))?;
                        let offsets = journal
                            .replay("orders", None, None, None)
                            .into_iter()
                            .map(|event| event.offset)
                            .collect::<Vec<_>>();
                        evidence.record("ordered_replay", json!({"offsets": offsets}));
                        ensure!(offsets == [0, 1, 2]);
                        Ok(())
                    })
                },
                fault: {
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        unavailable.store(true, Ordering::Release);
                        evidence.record("journal_fault", json!({"available": false}));
                        Ok(())
                    })
                },
                recover: {
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        unavailable.store(false, Ordering::Release);
                        evidence.record("journal_recovered", json!({"available": true}));
                        Ok(())
                    })
                },
                verify: {
                    let journal = Arc::clone(&journal);
                    let unavailable = Arc::clone(&unavailable);
                    Box::new(move |evidence| {
                        ensure!(!unavailable.load(Ordering::Acquire));
                        let journal = journal
                            .lock()
                            .map_err(|_| anyhow!("journal lock poisoned"))?;
                        let events = journal.replay("orders", None, None, None);
                        let checkpoint = journal
                            .checkpoint("orders", "worker")
                            .ok_or_else(|| anyhow!("checkpoint missing after recovery"))?;
                        let offsets = events.iter().map(|event| event.offset).collect::<Vec<_>>();
                        evidence.record(
                            "replay_continuity",
                            json!({"offsets": offsets, "checkpoint": checkpoint.offset}),
                        );
                        ensure!(offsets == [0, 1, 2]);
                        ensure!(checkpoint.offset == 2);
                        Ok(())
                    })
                },
                teardown: Box::new(|evidence| {
                    evidence.record("adapter_cleanup", json!({"complete": true}));
                    Ok(())
                }),
            },
        )
        .with_limits(StatefulLimits {
            phase_timeout: Duration::from_secs(2),
            scenario_timeout: Duration::from_secs(6),
            teardown_timeout: Duration::from_secs(1),
        }),
    );

    assert!(report.passed, "{report:#?}");
    assert_eq!(report.protocol, "rig.stateful.v1");
    assert!(report
        .evidence
        .iter()
        .any(|record| record.kind == "replay_continuity"));
}
// HANDWRITE-END

// SPEC-MANAGED: apps/relay/tech-design/logic/competitor-perf-gate-vs-rabbitmq-nats-redis-work-queue-ratchet.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:b7f01a68" tracker="pending-tracker" reason="Tests for the ratchet rule (holds / regresses / must-beat lost) and a small-scale smoke of the benched workloads."
//! Perf-gate ratchet rule (#125) plus a small-scale smoke that keeps the two
//! benched gate workloads honest in CI without any competitor running.

use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;

use relay::perf_gate::{evaluate, Cell};
use relay::{Relay, RelayCoreConfig};

fn cell(name: &str, ratio: f64, baseline: f64, must_beat: bool) -> Cell {
    Cell {
        name: name.into(),
        ratio,
        baseline_ratio: baseline,
        must_beat,
    }
}

#[test]
fn ratchet_holds_when_no_regression() {
    let v = evaluate(&[cell("work_queue", 1.55, 1.60, true)], 0.95);
    assert!(v.passed, "1.55 >= 1.60*0.95 and still winning");
}

#[test]
fn ratchet_fails_on_regression() {
    let v = evaluate(&[cell("work_queue", 1.40, 1.60, false)], 0.95);
    assert!(!v.passed);
    assert_eq!(v.regressions, vec!["work_queue".to_string()]);
}

#[test]
fn gate_fails_when_must_beat_cell_is_lost() {
    // No regression vs its own baseline, but relay is now slower than the bar.
    let v = evaluate(&[cell("work_queue", 0.90, 0.90, true)], 0.95);
    assert!(!v.passed);
    assert!(v.regressions.is_empty());
    assert_eq!(v.must_beat_losses, vec!["work_queue".to_string()]);
}

#[test]
fn report_only_cell_does_not_fail_when_behind() {
    // Optional/report-only cells are allowed to be behind as long as they do not
    // regress against their own baseline.
    let v = evaluate(&[cell("optional_external_bar", 0.80, 0.80, false)], 0.95);
    assert!(v.passed);
}

// Smoke: the two benched workloads execute and remain correct at small scale.
#[test]
fn gate_workloads_are_valid() {
    let r = Relay::new(RelayCoreConfig::in_memory());
    let now = Utc::now();
    const M: usize = 200;

    // work-queue publish.
    for i in 0..M {
        r.publish(
            "bench",
            &format!("m{i}"),
            serde_json::json!({ "i": i }),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
    assert_eq!(r.log_len("bench").unwrap(), M as u64);

    // work-queue: lease + ack each entry exactly once.
    let mut acked: BTreeSet<u64> = BTreeSet::new();
    while let Some(l) = r.lease("bench", "c", now).unwrap() {
        assert!(r.ack("bench", &l.lease_id, Some(l.epoch)).unwrap());
        assert!(acked.insert(l.seq), "each seq leased exactly once");
    }
    assert_eq!(acked.len(), M);
    assert_eq!(
        r.committed_offset("bench").unwrap().unwrap().committed_seq,
        (M - 1) as u64
    );
}
// HANDWRITE-END

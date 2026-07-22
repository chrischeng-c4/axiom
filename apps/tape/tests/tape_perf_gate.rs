// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-tape-perf-gate-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-competitor-performance" tracker="#768" reason="Initial local performance regression and external calibration-status gate.">
use std::process::Command;

const EVENTS: usize = 1_000;
const PAYLOAD_BYTES: usize = 128;
const MAX_APPEND_P95_US: u128 = 5_000;
const MAX_REPLAY_FULL_US: u128 = 50_000;
const MAX_CHECKPOINT_P95_US: u128 = 5_000;

#[test]
fn local_replay_perf_gate_passes_without_external_win_claims() {
    let report = tape::bench::run_benchmark(EVENTS, PAYLOAD_BYTES);
    assert_eq!(report.events, EVENTS);
    assert_eq!(report.payload_bytes, PAYLOAD_BYTES);
    assert!(
        report.append_p95_us <= MAX_APPEND_P95_US,
        "append p95 {}us exceeds EC limit {MAX_APPEND_P95_US}us",
        report.append_p95_us
    );
    assert!(
        report.replay_full_us <= MAX_REPLAY_FULL_US,
        "full replay {}us exceeds EC limit {MAX_REPLAY_FULL_US}us",
        report.replay_full_us
    );
    assert!(
        report.checkpoint_p95_us <= MAX_CHECKPOINT_P95_US,
        "checkpoint p95 {}us exceeds EC limit {MAX_CHECKPOINT_P95_US}us",
        report.checkpoint_p95_us
    );
    assert!(
        !report.external_peer_win_claim,
        "Tape must not claim Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams wins without calibrated peer runs"
    );
    for peer in report.peers {
        if !peer.replay_baseline {
            continue;
        }
        assert!(!peer.win_claim);
        match peer.peer {
            "Kafka topic log" | "NATS JetStream stream" => {
                assert_eq!(peer.status, "calibrated_separate_gate")
            }
            _ => assert_eq!(peer.status, "not_calibrated"),
        }
    }
}

#[test]
fn tape_bench_cli_reports_calibration_status() {
    let output = Command::new(env!("CARGO_BIN_EXE_tape-bench"))
        .args(["run", "--events", "1000", "--format", "json"])
        .output()
        .expect("run tape-bench");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("local_regression_passed_external_wins_require_separate_gates"));
    assert!(stdout.contains("Kafka topic log"));
    assert!(stdout.contains("calibrated_separate_gate"));
    assert!(stdout.contains("not_calibrated"));
    assert!(stdout.contains("RabbitMQ topic exchange"));
    assert!(stdout.contains("not_a_replay_baseline"));
}
// </HANDWRITE>

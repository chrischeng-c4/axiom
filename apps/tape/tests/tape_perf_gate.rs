// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-tests-tape-perf-gate-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:tape-competitor-performance" tracker="#768" reason="Initial local performance regression and external calibration-status gate.">
use std::process::Command;

#[test]
fn local_replay_perf_gate_passes_without_external_win_claims() {
    let report = tape::bench::run_benchmark(1_000, 128);
    tape::bench::verify_report(&report).expect("local Tape perf gate passes");
    assert!(report.local_regression_passed);
    assert!(
        !report.external_peer_win_claim,
        "Tape must not claim Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams wins without calibrated peer runs"
    );
    for peer in report.peers {
        if peer.replay_baseline {
            assert_eq!(peer.status, "not_calibrated");
            assert!(!peer.win_claim);
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
    assert!(stdout.contains("local_regression_passed_external_broker_wins_not_calibrated"));
    assert!(stdout.contains("Kafka topic log"));
    assert!(stdout.contains("not_calibrated"));
    assert!(stdout.contains("RabbitMQ topic exchange"));
    assert!(stdout.contains("not_a_replay_baseline"));
}
// </HANDWRITE>

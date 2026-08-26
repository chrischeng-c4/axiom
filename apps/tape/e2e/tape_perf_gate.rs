// <HANDWRITE gap="missing-generator:test:tape-competitor-performance" tracker="#768" reason="Initial local performance regression and external calibration-status gate.">
use std::process::Command;

const EVENTS: usize = 1_000;
const PAYLOAD_BYTES: usize = 128;
const MAX_APPEND_P95_US: u128 = 5_000;
const MAX_REPLAY_FULL_US: u128 = 50_000;
const MAX_CHECKPOINT_P95_US: u128 = 5_000;

/// WI #3052 AC1: connection counts to sample for the durable throughput
/// scaling gate. 1 is the pre-#3052 flat-line baseline; 16 is the point the
/// group-commit design must have visibly amortized the fsync barrier by.
const DURABLE_CONNECTION_COUNTS: &[usize] = &[1, 4, 16];
const DURABLE_EVENTS_PER_CONNECTION: usize = 200;
const DURABLE_PAYLOAD_BYTES: usize = 128;

/// AC1's ratio gate: 16-connection durable append throughput must be at
/// least this many times the 1-connection throughput. A **ratio**, not an
/// absolute ops/s, because the absolute number is a property of this
/// machine's fsync and the ratio is a property of the group-commit design.
/// This threshold is fixed by the accepted TD -- if a real run falls short,
/// report the measured numbers; do not lower this constant to match them.
const DURABLE_SCALING_MIN_RATIO: f64 = 4.0;
/// How far below the 1-connection sample any other sample is allowed to
/// fall before it counts as a real regression rather than scheduler noise.
const DURABLE_NOISE_FLOOR_RATIO: f64 = 0.8;

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

/// WI #3052 AC1: "Durable append throughput rises with connection count
/// instead of staying flat, measured with the same harness that produced the
/// 85-89 flat line." Drives real HTTP against the real `WalStore` +
/// `CommitCoordinator` group-commit path (`FsyncPolicy::Always` -- never
/// weakened to hit this gate) at 1/4/16 connections and asserts the
/// 16-connection sample beats the 1-connection sample by
/// `DURABLE_SCALING_MIN_RATIO`.
#[test]
fn durable_append_throughput_rises_with_connection_count() {
    let report = tape::bench::run_durable_benchmark(
        DURABLE_EVENTS_PER_CONNECTION,
        DURABLE_PAYLOAD_BYTES,
        DURABLE_CONNECTION_COUNTS,
    );
    assert_eq!(report.samples.len(), DURABLE_CONNECTION_COUNTS.len());
    assert_eq!(report.payload_bytes, DURABLE_PAYLOAD_BYTES);

    let one_conn = report
        .samples
        .iter()
        .find(|s| s.connections == 1)
        .expect("a 1-connection sample");
    let sixteen_conn = report
        .samples
        .iter()
        .find(|s| s.connections == 16)
        .expect("a 16-connection sample");

    assert!(
        one_conn.ops_per_sec > 0.0,
        "1-connection durable sample measured zero throughput: {one_conn:?}"
    );
    let measured_ratio = sixteen_conn.ops_per_sec / one_conn.ops_per_sec;
    assert!(
        measured_ratio >= DURABLE_SCALING_MIN_RATIO,
        "16-connection durable throughput {:.2} ops/s is only {:.2}x the \
         1-connection throughput {:.2} ops/s (need >= {DURABLE_SCALING_MIN_RATIO}x); \
         samples: {:?}",
        sixteen_conn.ops_per_sec,
        measured_ratio,
        one_conn.ops_per_sec,
        report.samples
    );
    assert!(
        (measured_ratio - report.scaling_ratio).abs() < 1e-6,
        "report.scaling_ratio {} does not match the measured 1-vs-16 ratio {measured_ratio}",
        report.scaling_ratio
    );

    // Not-strictly-monotonic tolerance: no sampled connection count may fall
    // meaningfully below the 1-connection baseline (scheduler noise, not a
    // real regression, is the only thing allowed to dip below 1.0x).
    let noise_floor = one_conn.ops_per_sec * DURABLE_NOISE_FLOOR_RATIO;
    for sample in &report.samples {
        assert!(
            sample.ops_per_sec >= noise_floor,
            "connections={} ops_per_sec={:.2} fell below the 1-connection \
             noise floor {:.2} ({:.2} ops/s x {DURABLE_NOISE_FLOOR_RATIO}); samples: {:?}",
            sample.connections,
            sample.ops_per_sec,
            noise_floor,
            one_conn.ops_per_sec,
            report.samples
        );
    }
}

/// `tape-bench durable` CLI smoke: the `--durable` mode surfaces the same
/// per-connection samples and ratio the library report computes.
#[test]
fn tape_bench_cli_durable_mode_reports_samples_and_ratio() {
    let output = Command::new(env!("CARGO_BIN_EXE_tape-bench"))
        .args([
            "durable",
            "--events-per-connection",
            "20",
            "--connections",
            "1,4",
            "--format",
            "json",
        ])
        .output()
        .expect("run tape-bench durable");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"scaling_ratio\""));
    assert!(stdout.contains("\"connections\": 1"));
    assert!(stdout.contains("\"connections\": 4"));
}
// </HANDWRITE>

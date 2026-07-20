// HANDWRITE-BEGIN gap="missing-generator:unit-test:c71dc69c" tracker="pending-tracker" reason="Define a serde report for workload relay-durable-publish-lease-ack-v1; a report-only ignored child measures 2000 128-byte messages in 100-message batches on temporary FsyncPolicy Always storage, while the ignored parent parses the child stdout and requires both phases to have at least 20 samples, zero errors, complete counts, at least 500 messages per second, and batch p95 no greater than 500000 microseconds."
use std::collections::BTreeMap;
use std::process::Command;
use std::time::{Duration, Instant};

use chrono::Utc;
use relay::{FsyncPolicy, Relay, RelayCoreConfig, DEFAULT_PRIORITY};
use serde::{Deserialize, Serialize};

const WORKLOAD_ID: &str = "relay-durable-publish-lease-ack-v1";
const REPORT_PREFIX: &str = "RELAY_PERF_JSON=";
const OPS: usize = 2_000;
const BATCH: usize = 100;
const PAYLOAD_BYTES: usize = 128;
const MIN_SAMPLES_PER_PHASE: usize = OPS / BATCH;
const MIN_PUBLISH_OPS_PER_SEC: f64 = 500.0;
const MIN_LEASE_ACK_OPS_PER_SEC: f64 = 500.0;
const MAX_BATCH_P95_US: u64 = 500_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PhaseReport {
    ops: usize,
    samples: usize,
    elapsed_us: u64,
    throughput_ops_per_sec: f64,
    p95_batch_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceReport {
    workload_id: String,
    payload_bytes: usize,
    batch_size: usize,
    errors: usize,
    acked_ops: usize,
    publish: PhaseReport,
    lease_ack: PhaseReport,
}

fn phase_report(ops: usize, elapsed: Duration, samples_us: &[u64]) -> PhaseReport {
    let elapsed_us = elapsed.as_micros().max(1) as u64;
    let mut ordered = samples_us.to_vec();
    ordered.sort_unstable();
    let p95_index = if ordered.is_empty() {
        0
    } else {
        ((ordered.len() - 1) * 95).div_ceil(100)
    };
    PhaseReport {
        ops,
        samples: ordered.len(),
        elapsed_us,
        throughput_ops_per_sec: ops as f64 * 1_000_000.0 / elapsed_us as f64,
        p95_batch_us: ordered.get(p95_index).copied().unwrap_or_default(),
    }
}

fn measure_durable_lifecycle() -> Result<PerformanceReport, String> {
    let data_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let relay = Relay::new(RelayCoreConfig {
        data_dir: data_dir.path().to_string_lossy().into_owned(),
        fsync: FsyncPolicy::Always,
        ..RelayCoreConfig::default()
    });
    let payload = serde_json::Value::String("x".repeat(PAYLOAD_BYTES));
    let now = Utc::now();

    let publish_started = Instant::now();
    let mut publish_samples = Vec::with_capacity(MIN_SAMPLES_PER_PHASE);
    for request in 0..MIN_SAMPLES_PER_PHASE {
        let messages = (0..BATCH)
            .map(|item| {
                (
                    format!("message-{}", request * BATCH + item),
                    payload.clone(),
                    BTreeMap::new(),
                    DEFAULT_PRIORITY,
                )
            })
            .collect();
        let sample_started = Instant::now();
        let outcomes = relay
            .publish_batch("perf", messages, now)
            .map_err(|error| error.to_string())?;
        if outcomes.len() != BATCH {
            return Err(format!(
                "publish batch {request} returned {} outcomes, expected {BATCH}",
                outcomes.len()
            ));
        }
        publish_samples.push(sample_started.elapsed().as_micros().max(1) as u64);
    }
    let publish = phase_report(OPS, publish_started.elapsed(), &publish_samples);

    let lease_ack_started = Instant::now();
    let mut lease_ack_samples = Vec::with_capacity(MIN_SAMPLES_PER_PHASE);
    let mut acked_ops = 0usize;
    while acked_ops < OPS {
        let sample_started = Instant::now();
        let leases = relay
            .lease_batch("perf", "perf-consumer", BATCH, Utc::now())
            .map_err(|error| error.to_string())?;
        if leases.is_empty() {
            return Err(format!(
                "durable queue drained early after {acked_ops} of {OPS} acknowledgements"
            ));
        }
        let acknowledgements = leases
            .iter()
            .map(|lease| (lease.lease_id.clone(), Some(lease.epoch)))
            .collect::<Vec<_>>();
        let (acked, _) = relay
            .ack_batch("perf", &acknowledgements)
            .map_err(|error| error.to_string())?;
        if acked != leases.len() {
            return Err(format!(
                "ack batch committed {acked} of {} leases",
                leases.len()
            ));
        }
        acked_ops += acked;
        lease_ack_samples.push(sample_started.elapsed().as_micros().max(1) as u64);
    }
    let lease_ack = phase_report(acked_ops, lease_ack_started.elapsed(), &lease_ack_samples);

    Ok(PerformanceReport {
        workload_id: WORKLOAD_ID.to_owned(),
        payload_bytes: PAYLOAD_BYTES,
        batch_size: BATCH,
        errors: 0,
        acked_ops,
        publish,
        lease_ack,
    })
}

fn validate_report(report: &PerformanceReport) -> Result<(), String> {
    if report.workload_id != WORKLOAD_ID {
        return Err(format!("unexpected workload id: {}", report.workload_id));
    }
    if report.payload_bytes != PAYLOAD_BYTES || report.batch_size != BATCH {
        return Err("workload shape does not match the pinned EC constants".to_owned());
    }
    if report.errors != 0 {
        return Err(format!(
            "measured workload reported {} errors",
            report.errors
        ));
    }
    if report.publish.ops != OPS || report.lease_ack.ops != OPS || report.acked_ops != OPS {
        return Err(format!(
            "incomplete lifecycle: published={} leased_and_acked={} acked={}",
            report.publish.ops, report.lease_ack.ops, report.acked_ops
        ));
    }
    for (name, phase) in [
        ("publish", &report.publish),
        ("lease_ack", &report.lease_ack),
    ] {
        if phase.samples < MIN_SAMPLES_PER_PHASE || phase.elapsed_us == 0 {
            return Err(format!(
                "{name} has missing observations: samples={} elapsed_us={}",
                phase.samples, phase.elapsed_us
            ));
        }
        if !phase.throughput_ops_per_sec.is_finite() || phase.throughput_ops_per_sec <= 0.0 {
            return Err(format!("{name} throughput is absent or non-positive"));
        }
        if phase.p95_batch_us == 0 {
            return Err(format!("{name} p95 observation is zero"));
        }
        if phase.p95_batch_us > MAX_BATCH_P95_US {
            return Err(format!(
                "{name} p95 {}us exceeds {}us",
                phase.p95_batch_us, MAX_BATCH_P95_US
            ));
        }
    }
    if report.publish.throughput_ops_per_sec < MIN_PUBLISH_OPS_PER_SEC {
        return Err(format!(
            "publish throughput {:.1} msg/s is below {:.1} msg/s",
            report.publish.throughput_ops_per_sec, MIN_PUBLISH_OPS_PER_SEC
        ));
    }
    if report.lease_ack.throughput_ops_per_sec < MIN_LEASE_ACK_OPS_PER_SEC {
        return Err(format!(
            "lease/ack throughput {:.1} msg/s is below {:.1} msg/s",
            report.lease_ack.throughput_ops_per_sec, MIN_LEASE_ACK_OPS_PER_SEC
        ));
    }
    Ok(())
}

fn passing_report() -> PerformanceReport {
    let phase = PhaseReport {
        ops: OPS,
        samples: MIN_SAMPLES_PER_PHASE,
        elapsed_us: 1_000_000,
        throughput_ops_per_sec: OPS as f64,
        p95_batch_us: 10_000,
    };
    PerformanceReport {
        workload_id: WORKLOAD_ID.to_owned(),
        payload_bytes: PAYLOAD_BYTES,
        batch_size: BATCH,
        errors: 0,
        acked_ops: OPS,
        publish: phase.clone(),
        lease_ack: phase,
    }
}

#[test]
fn report_validation_rejects_zero_samples() {
    let mut report = passing_report();
    report.publish.samples = 0;
    assert!(validate_report(&report)
        .unwrap_err()
        .contains("missing observations"));
}

#[test]
fn report_validation_rejects_incomplete_or_error_lifecycle() {
    let mut incomplete = passing_report();
    incomplete.acked_ops -= 1;
    assert!(validate_report(&incomplete)
        .unwrap_err()
        .contains("incomplete lifecycle"));

    let mut errored = passing_report();
    errored.errors = 1;
    assert!(validate_report(&errored)
        .unwrap_err()
        .contains("reported 1 errors"));
}

#[test]
fn report_validation_rejects_threshold_regression() {
    let mut slow = passing_report();
    slow.lease_ack.throughput_ops_per_sec = MIN_LEASE_ACK_OPS_PER_SEC - 1.0;
    assert!(validate_report(&slow).unwrap_err().contains("below"));

    let mut latent = passing_report();
    latent.publish.p95_batch_us = MAX_BATCH_P95_US + 1;
    assert!(validate_report(&latent).unwrap_err().contains("exceeds"));
}

#[test]
#[ignore = "report producer; invoked only by measured_durable_lifecycle_gate"]
fn durable_lifecycle_report_child() {
    assert_eq!(std::env::var("RELAY_PERF_REPORT_CHILD").as_deref(), Ok("1"));
    let report = measure_durable_lifecycle().expect("measure durable Relay lifecycle");
    println!(
        "{REPORT_PREFIX}{}",
        serde_json::to_string(&report).expect("serialize performance report")
    );
}

#[test]
#[ignore = "release-mode production performance gate"]
fn measured_durable_lifecycle_gate() {
    assert!(
        !cfg!(debug_assertions),
        "production performance gate must run with --release"
    );
    let output = Command::new(std::env::current_exe().expect("current integration test binary"))
        .args([
            "--exact",
            "durable_lifecycle_report_child",
            "--ignored",
            "--nocapture",
        ])
        .env("RELAY_PERF_REPORT_CHILD", "1")
        .output()
        .expect("run independent report producer");
    assert!(
        output.status.success(),
        "report producer failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("report stdout is UTF-8");
    let reports = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(REPORT_PREFIX))
        .collect::<Vec<_>>();
    assert_eq!(
        reports.len(),
        1,
        "expected exactly one machine report, got {} in:\n{stdout}",
        reports.len()
    );
    let report: PerformanceReport =
        serde_json::from_str(reports[0]).expect("parse child performance report");
    validate_report(&report).expect("measured durable lifecycle stays inside pinned envelope");
    println!(
        "relay_perf_gate workload={} ops={} samples={}/{} publish={:.1}msg/s lease_ack={:.1}msg/s p95={}/{}us errors={}",
        report.workload_id,
        OPS,
        report.publish.samples,
        report.lease_ack.samples,
        report.publish.throughput_ops_per_sec,
        report.lease_ack.throughput_ops_per_sec,
        report.publish.p95_batch_us,
        report.lease_ack.p95_batch_us,
        report.errors
    );
}

// HANDWRITE-END

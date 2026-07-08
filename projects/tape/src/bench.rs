// SPEC-MANAGED: projects/tape/tech-design/semantic/source/projects-tape-src-bench-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-competitor-performance" tracker="#768" reason="Initial local benchmark and external peer calibration ledger before generated efficiency primitives exist.">
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};

use crate::TapeJournal;

const DEFAULT_EVENTS: usize = 1_000;
const DEFAULT_PAYLOAD_BYTES: usize = 128;

#[derive(Clone, Debug, Serialize)]
pub struct PerfBudget {
    pub append_p95_us: u128,
    pub replay_full_us: u128,
    pub checkpoint_p95_us: u128,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerCalibration {
    pub peer: &'static str,
    pub replay_baseline: bool,
    pub status: &'static str,
    pub win_claim: bool,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct CompetitiveBaseline {
    pub events: usize,
    pub payload_bytes: usize,
    pub ratchet: f64,
    pub budget: PerfBudget,
    pub peers: Vec<PeerCalibration>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BenchReport {
    pub project: &'static str,
    pub events: usize,
    pub payload_bytes: usize,
    pub append_p50_us: u128,
    pub append_p95_us: u128,
    pub replay_full_us: u128,
    pub checkpoint_p50_us: u128,
    pub checkpoint_p95_us: u128,
    pub local_regression_passed: bool,
    pub external_peer_win_claim: bool,
    pub verdict: &'static str,
    pub peers: Vec<PeerCalibration>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalReplayWin {
    pub peer: &'static str,
    pub workload: &'static str,
    pub events: usize,
    pub payload_bytes: usize,
    pub tape_replay_us: u128,
    pub peer_replay_us: u128,
    pub ratio: f64,
    pub required_ratio: f64,
    pub win_claim: bool,
    pub evidence: &'static str,
}

pub fn default_baseline() -> CompetitiveBaseline {
    CompetitiveBaseline {
        events: DEFAULT_EVENTS,
        payload_bytes: DEFAULT_PAYLOAD_BYTES,
        ratchet: 0.8,
        budget: PerfBudget {
            append_p95_us: 5_000,
            replay_full_us: 50_000,
            checkpoint_p95_us: 5_000,
        },
        peers: vec![
            uncalibrated_peer("Kafka topic log"),
            uncalibrated_peer("Redpanda topic log"),
            uncalibrated_peer("Pulsar topic"),
            uncalibrated_peer("NATS JetStream stream"),
            uncalibrated_peer("RabbitMQ Streams"),
            PeerCalibration {
                peer: "RabbitMQ topic exchange",
                replay_baseline: false,
                status: "not_a_replay_baseline",
                win_claim: true,
                reason: "Tape has offset/time replay and durable checkpoints; RabbitMQ topic exchange is routing/fanout only.",
            },
        ],
    }
}

pub fn run_benchmark(events: usize, payload_bytes: usize) -> BenchReport {
    let baseline = default_baseline();
    let events = events.max(1);
    let payload_bytes = payload_bytes.max(1);
    let payload = payload(payload_bytes);
    let mut journal = TapeJournal::default();
    let mut append_samples = Vec::with_capacity(events);

    for i in 0..events {
        let started = Instant::now();
        journal.append(
            "orders.created",
            Some(format!("orders.created.{i}")),
            payload.clone(),
            Some(i as u64),
        );
        append_samples.push(started.elapsed().as_micros());
    }

    let replay_started = Instant::now();
    let replayed = journal.replay_refs("orders.created", Some(0), None, Some(events));
    let replay_full_us = replay_started.elapsed().as_micros();
    assert_eq!(replayed.len(), events);

    let mut checkpoint_samples = Vec::with_capacity(events);
    for offset in 0..=events {
        let started = Instant::now();
        journal
            .put_checkpoint("orders.created", "bench-worker", offset as u64)
            .expect("checkpoint advances within topic end offset");
        checkpoint_samples.push(started.elapsed().as_micros());
    }

    append_samples.sort_unstable();
    checkpoint_samples.sort_unstable();
    let append_p50_us = percentile(&append_samples, 0.50);
    let append_p95_us = percentile(&append_samples, 0.95);
    let checkpoint_p50_us = percentile(&checkpoint_samples, 0.50);
    let checkpoint_p95_us = percentile(&checkpoint_samples, 0.95);
    let local_regression_passed = append_p95_us <= baseline.budget.append_p95_us
        && replay_full_us <= baseline.budget.replay_full_us
        && checkpoint_p95_us <= baseline.budget.checkpoint_p95_us;
    let external_peer_win_claim = baseline
        .peers
        .iter()
        .any(|peer| peer.replay_baseline && peer.win_claim);

    BenchReport {
        project: "tape",
        events,
        payload_bytes,
        append_p50_us,
        append_p95_us,
        replay_full_us,
        checkpoint_p50_us,
        checkpoint_p95_us,
        local_regression_passed,
        external_peer_win_claim,
        verdict: if local_regression_passed && !external_peer_win_claim {
            "local_regression_passed_external_broker_wins_not_calibrated"
        } else {
            "failed_or_overclaimed"
        },
        peers: baseline.peers,
    }
}

pub fn verify_report(report: &BenchReport) -> Result<(), String> {
    let baseline = default_baseline();
    if report.append_p95_us > baseline.budget.append_p95_us {
        return Err(format!(
            "append p95 {}us exceeds {}us",
            report.append_p95_us, baseline.budget.append_p95_us
        ));
    }
    if report.replay_full_us > baseline.budget.replay_full_us {
        return Err(format!(
            "full replay {}us exceeds {}us",
            report.replay_full_us, baseline.budget.replay_full_us
        ));
    }
    if report.checkpoint_p95_us > baseline.budget.checkpoint_p95_us {
        return Err(format!(
            "checkpoint p95 {}us exceeds {}us",
            report.checkpoint_p95_us, baseline.budget.checkpoint_p95_us
        ));
    }
    if report.external_peer_win_claim {
        return Err("external broker win claim requires calibrated peer evidence".to_string());
    }
    Ok(())
}

pub fn external_replay_win(
    peer: &'static str,
    workload: &'static str,
    events: usize,
    payload_bytes: usize,
    tape_replay_us: u128,
    peer_replay_us: u128,
    required_ratio: f64,
    evidence: &'static str,
) -> ExternalReplayWin {
    let ratio = if tape_replay_us == 0 {
        f64::INFINITY
    } else {
        peer_replay_us as f64 / tape_replay_us as f64
    };
    ExternalReplayWin {
        peer,
        workload,
        events,
        payload_bytes,
        tape_replay_us,
        peer_replay_us,
        ratio,
        required_ratio,
        win_claim: ratio >= required_ratio,
        evidence,
    }
}

pub fn verify_external_replay_win(report: &ExternalReplayWin) -> Result<(), String> {
    if report.events == 0 {
        return Err("external replay win requires at least one event".to_string());
    }
    if !report.win_claim {
        return Err(format!(
            "{} replay ratio {:.2}x is below required {:.2}x (peer {}us, tape {}us)",
            report.peer,
            report.ratio,
            report.required_ratio,
            report.peer_replay_us,
            report.tape_replay_us
        ));
    }
    Ok(())
}

fn uncalibrated_peer(peer: &'static str) -> PeerCalibration {
    PeerCalibration {
        peer,
        replay_baseline: true,
        status: "not_calibrated",
        win_claim: false,
        reason: "No real-service external benchmark has been run in this checkout; Tape reports local regression only.",
    }
}

fn payload(bytes: usize) -> Value {
    json!({
        "id": "bench",
        "body": "x".repeat(bytes),
    })
}

fn percentile(samples: &[u128], q: f64) -> u128 {
    let idx = (((samples.len() - 1) as f64) * q).round() as usize;
    samples[idx]
}
// </HANDWRITE>

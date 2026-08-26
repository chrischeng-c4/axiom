// <HANDWRITE gap="missing-generator:logic:tape-competitor-performance" tracker="#768" reason="Initial local benchmark and external peer calibration ledger before generated efficiency primitives exist.">
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};

use crate::server::{router, AppState};
use crate::wal::{CommitCoordinator, WalStore};
use crate::TapeJournal;

const DEFAULT_EVENTS: usize = 1_000;
const DEFAULT_PAYLOAD_BYTES: usize = 128;

/// Data-plane request body size limit for [`run_durable_benchmark`]'s
/// `AppState` -- arbitrary but generous headroom over the small bench
/// payloads this drives, matching the limit other real-HTTP tape tests use.
const DURABLE_BENCH_BODY_LIMIT_BYTES: usize = 8 * 1024 * 1024;

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
            separate_gate_peer(
                "Kafka topic log",
                "Calibrated by the release real-service tape_vs_kafka gate; the local-only report never imports or claims that result.",
            ),
            uncalibrated_peer("Redpanda topic log"),
            uncalibrated_peer("Pulsar topic"),
            separate_gate_peer(
                "NATS JetStream stream",
                "Calibrated by the release real-service tape_vs_nats_jetstream gate; the local-only report never imports or claims that result.",
            ),
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
            "local_regression_passed_external_wins_require_separate_gates"
        } else {
            "failed_or_overclaimed"
        },
        peers: baseline.peers,
    }
}

/// One connection-count's measured durable append throughput from
/// [`run_durable_benchmark`].
#[derive(Clone, Debug, Serialize)]
pub struct DurableConnectionSample {
    pub connections: usize,
    pub events: usize,
    pub elapsed_us: u128,
    pub ops_per_sec: f64,
}

/// WI #3052 AC1 report: durable append throughput at each sampled connection
/// count, driven over real HTTP against the real [`crate::wal::CommitCoordinator`]
/// group-commit path (`FsyncPolicy::Always`), plus the scaling ratio between
/// the highest and lowest sampled connection count. `tape_perf_gate.rs` gates
/// on `scaling_ratio`, never on an absolute `ops_per_sec` value -- the
/// absolute number is a property of the machine's fsync, the ratio is a
/// property of the group-commit design.
#[derive(Clone, Debug, Serialize)]
pub struct DurableBenchReport {
    pub payload_bytes: usize,
    pub samples: Vec<DurableConnectionSample>,
    /// `ops_per_sec` at the highest sampled connection count divided by
    /// `ops_per_sec` at the lowest sampled connection count.
    pub scaling_ratio: f64,
}

/// Drive `connections` concurrent HTTP clients, each issuing
/// `events_per_connection` sequential `POST /topics/{topic}/append` requests,
/// against a real axum router wired to a real [`WalStore`] +
/// [`CommitCoordinator`] over `FsyncPolicy::Always` -- the same durable path
/// `tape serve --data-dir` runs in production, and the same shape of harness
/// that measured the pre-#3052 85-89 ops/s flat line (in-process HTTP over a
/// real `127.0.0.1:0` socket, not an in-memory journal call).
///
/// Each sampled connection count gets its own fresh [`tempfile::TempDir`] and
/// [`WalStore`] so one sample's growing WAL/journal never contaminates the
/// next sample's measurement. Sequential requests *within* one connection are
/// the point: that is what makes group commit's cross-connection batching
/// observable in the throughput curve, rather than measuring one connection's
/// own request/response round-trip latency.
pub fn run_durable_benchmark(
    events_per_connection: usize,
    payload_bytes: usize,
    connection_counts: &[usize],
) -> DurableBenchReport {
    let events_per_connection = events_per_connection.max(1);
    let payload_bytes = payload_bytes.max(1);
    let payload = payload(payload_bytes);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime for durable benchmark");

    let samples: Vec<DurableConnectionSample> = connection_counts
        .iter()
        .map(|&connections| {
            let connections = connections.max(1);
            runtime.block_on(run_one_durable_sample(
                connections,
                events_per_connection,
                payload.clone(),
            ))
        })
        .collect();

    let min_connections = connection_counts.iter().copied().min().unwrap_or(1).max(1);
    let max_connections = connection_counts.iter().copied().max().unwrap_or(1).max(1);
    let base = samples.iter().find(|s| s.connections == min_connections);
    let top = samples.iter().find(|s| s.connections == max_connections);
    let scaling_ratio = match (base, top) {
        (Some(base), Some(top)) if base.ops_per_sec > 0.0 => top.ops_per_sec / base.ops_per_sec,
        _ => 0.0,
    };

    DurableBenchReport {
        payload_bytes,
        samples,
        scaling_ratio,
    }
}

/// One connection-count sample of [`run_durable_benchmark`]: spins up a fresh
/// durable `AppState` (real `WalStore` + `CommitCoordinator`, real HTTP
/// listener), drives `connections` concurrent client tasks each issuing
/// `events_per_connection` sequential appends, then tears the server down.
async fn run_one_durable_sample(
    connections: usize,
    events_per_connection: usize,
    payload: Value,
) -> DurableConnectionSample {
    let dir = tempfile::TempDir::new().expect("create durable bench temp dir");
    let (wal_store, journal) =
        WalStore::open(dir.path()).expect("open WalStore for durable bench sample");
    let state = AppState::new(journal, None, DURABLE_BENCH_BODY_LIMIT_BYTES);
    let coordinator = CommitCoordinator::spawn(wal_store, state.journal_handle());
    let state = state.with_wal(std::sync::Arc::new(coordinator));
    let app = router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind durable bench listener");
    let addr = listener.local_addr().expect("durable bench listener addr");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(service_http::serve(listener, app, async move {
        let _ = shutdown_rx.await;
    }));

    let client = reqwest::Client::new();
    let started = Instant::now();
    let mut handles = Vec::with_capacity(connections);
    for _ in 0..connections {
        let client = client.clone();
        let payload = payload.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..events_per_connection {
                let response = client
                    .post(format!("http://{addr}/topics/durable-bench/append"))
                    .json(&json!({
                        "payload": payload,
                        "timestamp_ms": i as u64,
                    }))
                    .send()
                    .await
                    .expect("durable bench append request");
                assert!(
                    response.status().is_success(),
                    "durable bench append returned {} for event {i}",
                    response.status()
                );
            }
        }));
    }
    for handle in handles {
        handle.await.expect("durable bench client task panicked");
    }
    let elapsed_us = started.elapsed().as_micros();

    // Best-effort graceful shutdown; the next sample uses a fresh listener
    // regardless, so a slow/failed shutdown here cannot leak into the next
    // sample's measurement.
    let _ = shutdown_tx.send(());
    let _ = server.await;

    let events = connections * events_per_connection;
    let ops_per_sec = if elapsed_us == 0 {
        0.0
    } else {
        events as f64 / (elapsed_us as f64 / 1_000_000.0)
    };

    DurableConnectionSample {
        connections,
        events,
        elapsed_us,
        ops_per_sec,
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

fn separate_gate_peer(peer: &'static str, reason: &'static str) -> PeerCalibration {
    PeerCalibration {
        peer,
        replay_baseline: true,
        status: "calibrated_separate_gate",
        win_claim: false,
        reason,
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

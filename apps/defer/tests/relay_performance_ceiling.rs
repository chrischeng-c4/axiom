// HANDWRITE-BEGIN gap="missing-generator:efficiency-test:defer-relay-ceiling" tracker="#766" reason="Sibling-service scheduler overhead gate under identical Raft/fsync/payload/batch lifecycle."
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Utc;
use defer::{
    AttemptSettlement, CreateTask, DeferRaft, DeferScheduler, QueuePolicy, SettlementOutcome,
    Target,
};
use raft_runtime::Membership;
use relay::{PubCommand, Relay, RelayCoreConfig, RelayRaft};
use service_observability::process_usage;

const BATCHES: usize = 10;
const BATCH_SIZE: usize = 100;
const PAYLOAD_BYTES: usize = 128;
const MIN_DEFER_TO_RELAY_RATIO: f64 = 0.80;
const CONTRACT: &str = include_str!("../benchmarks/relay-performance-ceiling.md");

fn sole_voter() -> Membership {
    Membership {
        voters: vec![0],
        learners: Vec::new(),
    }
}

async fn wait_defer_leader(raft: &DeferRaft) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !raft.is_leader().await {
        assert!(Instant::now() < deadline, "Defer did not elect");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn wait_relay_leader(raft: &RelayRaft) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !raft.is_leader().await {
        assert!(Instant::now() < deadline, "Relay did not elect");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn percentile(samples: &[u64], fraction: f64) -> u64 {
    let mut values = samples.to_vec();
    values.sort_unstable();
    values[((values.len() - 1) as f64 * fraction).round() as usize]
}

fn directory_bytes(path: &Path) -> u64 {
    std::fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                directory_bytes(&path)
            } else {
                entry.metadata().map(|meta| meta.len()).unwrap_or_default()
            }
        })
        .sum()
}

fn sample_process() -> (f64, u64) {
    let usage = process_usage(std::process::id()).expect("sample benchmark process");
    (usage.cpu_seconds, usage.rss_bytes)
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="#2217" reason="Own the release-mode oracle for exact same-host workload identity, completed-operation cardinality, positive finite metrics, explicit no-overclaim scope, and the hard Defer-to-Relay ratio threshold.">
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "performance gate: run explicitly on an otherwise idle host"]
async fn defer_stays_within_twenty_percent_of_relay_scheduler_ceiling() {
    let total = BATCHES * BATCH_SIZE;
    let contract_yaml = CONTRACT
        .split_once("```yaml\n")
        .and_then(|(_, rest)| rest.split_once("\n```").map(|(yaml, _)| yaml))
        .expect("machine-readable benchmark contract exists");
    let contract: serde_yaml::Value = serde_yaml::from_str(contract_yaml).unwrap();
    let contract = &contract["benchmark_contract"];
    assert_eq!(contract["performance_comparator"], "relay");
    assert_eq!(
        contract["scope"],
        "same-host-sibling-implementation-ceiling"
    );
    assert_eq!(
        contract["rss_measurement"],
        "process-shared-not-component-isolated"
    );
    assert_eq!(contract["dated_observation_authoritative"], false);
    assert_eq!(contract["cloud_tasks_performance_claim"], false);
    assert_eq!(contract["universal_superiority_claim"], false);
    assert_eq!(contract["messages"].as_u64(), Some(total as u64));
    assert_eq!(contract["batches"].as_u64(), Some(BATCHES as u64));
    assert_eq!(contract["batch_size"].as_u64(), Some(BATCH_SIZE as u64));
    assert_eq!(
        contract["payload_serialized_bytes"].as_u64(),
        Some(PAYLOAD_BYTES as u64)
    );
    assert_eq!(
        contract["minimum_ratio"].as_f64(),
        Some(MIN_DEFER_TO_RELAY_RATIO)
    );

    let json_envelope_bytes = br#"{"body":""}"#.len();
    let payload = serde_json::json!({"body": "x".repeat(PAYLOAD_BYTES - json_envelope_bytes)});
    let serialized_payload = serde_json::to_vec(&payload).unwrap();
    assert_eq!(
        serialized_payload.len(),
        PAYLOAD_BYTES,
        "payload contract measures serialized JSON bytes"
    );

    let defer_dir = tempfile::tempdir().unwrap();
    let defer = Arc::new(
        DeferRaft::spawn(
            Arc::new(Mutex::new(DeferScheduler::new())),
            &defer_dir.path().join("raft"),
            0,
            sole_voter(),
            HashMap::new(),
            DeferRaft::host_config(1_024),
        )
        .unwrap(),
    );
    wait_defer_leader(&defer).await;
    defer
        .configure_queue(
            "bench".into(),
            QueuePolicy {
                max_in_flight: BATCH_SIZE,
                max_dispatch_per_tick: BATCH_SIZE,
                max_dispatches_per_second: 1_000_000,
                max_burst_size: total,
                lease_ttl_ms: 30_000,
                retry_backoff_ms: 1_000,
            },
        )
        .await
        .unwrap();

    let relay_dir = tempfile::tempdir().unwrap();
    let relay_engine = Arc::new(Relay::new(RelayCoreConfig {
        data_dir: relay_dir.path().join("engine").to_str().unwrap().into(),
        ..RelayCoreConfig::default()
    }));
    let relay = Arc::new(
        RelayRaft::spawn(
            relay_engine,
            &relay_dir.path().join("raft"),
            0,
            sole_voter(),
            HashMap::new(),
            RelayRaft::host_config(1_024),
        )
        .unwrap(),
    );
    wait_relay_leader(&relay).await;

    let mut defer_latencies = Vec::with_capacity(total);
    let mut defer_completed = 0usize;
    let (defer_cpu_before, _) = sample_process();
    let defer_started = Instant::now();
    for batch in 0..BATCHES {
        let now = Utc::now();
        let batch_started = Instant::now();
        let tasks = (0..BATCH_SIZE)
            .map(|item| CreateTask {
                task_id: format!("defer-{batch}-{item}"),
                target: Target {
                    url: "http://target.invalid/bench".into(),
                    method: "POST".into(),
                    headers: Default::default(),
                },
                payload: payload.clone(),
                schedule_at: now,
                priority: 10,
                max_attempts: 3,
            })
            .collect();
        assert_eq!(
            defer.create_tasks("bench".into(), tasks).await.unwrap(),
            BATCH_SIZE
        );
        let leases = defer
            .lease_due("bench".into(), now, BATCH_SIZE)
            .await
            .unwrap();
        assert_eq!(leases.len(), BATCH_SIZE);
        let attempts = leases
            .into_iter()
            .map(|lease| AttemptSettlement {
                attempt_id: lease.attempt_id,
                epoch: lease.epoch,
                completed_at: now,
                success: true,
            })
            .collect();
        let outcomes = defer.settle_batch("bench".into(), attempts).await.unwrap();
        assert_eq!(outcomes.len(), BATCH_SIZE);
        assert!(outcomes
            .iter()
            .all(|outcome| matches!(outcome, SettlementOutcome::Acked(true))));
        defer_completed += outcomes.len();
        defer_latencies.extend(std::iter::repeat_n(
            batch_started.elapsed().as_micros() as u64,
            BATCH_SIZE,
        ));
    }
    let defer_elapsed = defer_started.elapsed();
    let (defer_cpu_after, defer_rss) = sample_process();
    assert_eq!(defer_completed, total);
    assert_eq!(defer_latencies.len(), defer_completed);

    let mut relay_latencies = Vec::with_capacity(total);
    let mut relay_completed = 0usize;
    let (relay_cpu_before, _) = sample_process();
    let relay_started = Instant::now();
    for batch in 0..BATCHES {
        let now = Utc::now();
        let batch_started = Instant::now();
        let commands = (0..BATCH_SIZE)
            .map(|item| PubCommand {
                subject: "bench".into(),
                message_id: format!("relay-{batch}-{item}"),
                payload: payload.clone(),
                headers: Default::default(),
                priority: 10,
                not_before: None,
                appended_at: now,
            })
            .collect();
        let (_, published) = relay
            .publish_batch("bench".into(), commands, now)
            .await
            .unwrap();
        assert_eq!(published.len(), BATCH_SIZE);
        assert!(published.iter().all(|outcome| !outcome.deduped));
        let leases = relay
            .lease_batch("bench".into(), "worker".into(), BATCH_SIZE, now)
            .await
            .unwrap();
        assert_eq!(leases.len(), BATCH_SIZE);
        let acks = leases
            .into_iter()
            .map(|lease| (lease.lease_id, lease.epoch))
            .collect();
        let (acked, _) = relay.ack_batch("bench".into(), acks, now).await.unwrap();
        assert_eq!(acked, BATCH_SIZE);
        relay_completed += acked;
        relay_latencies.extend(std::iter::repeat_n(
            batch_started.elapsed().as_micros() as u64,
            BATCH_SIZE,
        ));
    }
    let relay_elapsed = relay_started.elapsed();
    let (relay_cpu_after, relay_rss) = sample_process();
    assert_eq!(relay_completed, total);
    assert_eq!(relay_latencies.len(), relay_completed);

    let defer_ops = defer_completed as f64 / defer_elapsed.as_secs_f64();
    let relay_ops = relay_completed as f64 / relay_elapsed.as_secs_f64();
    let defer_disk = directory_bytes(defer_dir.path());
    let relay_disk = directory_bytes(relay_dir.path());
    let defer_p50 = percentile(&defer_latencies, 0.50);
    let defer_p95 = percentile(&defer_latencies, 0.95);
    let defer_p99 = percentile(&defer_latencies, 0.99);
    let relay_p50 = percentile(&relay_latencies, 0.50);
    let relay_p95 = percentile(&relay_latencies, 0.95);
    let relay_p99 = percentile(&relay_latencies, 0.99);
    let defer_cpu_ms = (defer_cpu_after - defer_cpu_before) * 1000.0;
    let relay_cpu_ms = (relay_cpu_after - relay_cpu_before) * 1000.0;
    let payload_total_bytes = total * serialized_payload.len();
    let defer_disk_amplification = defer_disk as f64 / payload_total_bytes as f64;
    let relay_disk_amplification = relay_disk as f64 / payload_total_bytes as f64;

    for (name, value) in [
        ("defer throughput", defer_ops),
        ("relay throughput", relay_ops),
        ("defer CPU", defer_cpu_ms),
        ("relay CPU", relay_cpu_ms),
        ("defer disk amplification", defer_disk_amplification),
        ("relay disk amplification", relay_disk_amplification),
    ] {
        assert!(
            value.is_finite() && value > 0.0,
            "{name} must be finite and non-zero, got {value}"
        );
    }
    for (name, value) in [
        ("defer p50", defer_p50),
        ("defer p95", defer_p95),
        ("defer p99", defer_p99),
        ("relay p50", relay_p50),
        ("relay p95", relay_p95),
        ("relay p99", relay_p99),
        ("defer RSS", defer_rss),
        ("relay RSS", relay_rss),
        ("defer disk bytes", defer_disk),
        ("relay disk bytes", relay_disk),
    ] {
        assert!(value > 0, "{name} must be non-zero");
    }

    let report = serde_json::json!({
        "scope": {"kind": "same-host-sibling-implementation-ceiling", "performance_comparator": "relay", "rss_measurement": "process-shared-not-component-isolated", "dated_observation_authoritative": false, "cloud_tasks_performance_claim": false, "universal_superiority_claim": false},
        "workload": {"messages": total, "batches": BATCHES, "batch_size": BATCH_SIZE, "payload_serialized_bytes": serialized_payload.len(), "lifecycle": "durable enqueue -> committed lease -> committed ack", "raft": "single voter, fsync always"},
        "defer": {"operations_completed": defer_completed, "throughput_ops_s": defer_ops, "p50_us": defer_p50, "p95_us": defer_p95, "p99_us": defer_p99, "cpu_ms": defer_cpu_ms, "rss_bytes_process_shared": defer_rss, "disk_bytes": defer_disk, "disk_amplification": defer_disk_amplification, "errors": 0},
        "relay": {"operations_completed": relay_completed, "throughput_ops_s": relay_ops, "p50_us": relay_p50, "p95_us": relay_p95, "p99_us": relay_p99, "cpu_ms": relay_cpu_ms, "rss_bytes_process_shared": relay_rss, "disk_bytes": relay_disk, "disk_amplification": relay_disk_amplification, "errors": 0},
        "defer_to_relay_ratio": defer_ops / relay_ops,
        "minimum_ratio": MIN_DEFER_TO_RELAY_RATIO,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
    assert!(
        defer_ops >= relay_ops * MIN_DEFER_TO_RELAY_RATIO,
        "Defer scheduler throughput may trail Relay by at most 20%: defer={defer_ops:.1}, relay={relay_ops:.1}"
    );
    defer.shutdown().await.unwrap();
    relay.shutdown().await.unwrap();
}
// </HANDWRITE>
// HANDWRITE-END

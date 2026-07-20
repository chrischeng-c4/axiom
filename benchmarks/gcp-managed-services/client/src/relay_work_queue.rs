use crate::metrics::{LatencySeries, ProcessDelta, ProcessSample};
use crate::Config;
use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct RelayWorkQueueReport {
    pub workload: Workload,
    pub publish_throughput_per_second: f64,
    pub consume: LatencySeries,
    pub client_process: ProcessDelta,
    pub final_committed_seq: u64,
    pub unique_leases: usize,
    pub errors: usize,
}

#[derive(Debug, Serialize)]
pub struct Workload {
    pub journey: &'static str,
    pub messages: usize,
    pub publish_batch_size: usize,
    pub lease_ack_batch_size: usize,
}

#[derive(Debug, Deserialize)]
struct LeaseBatchResponse {
    leases: Vec<Lease>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Lease {
    lease_id: String,
    seq: u64,
    epoch: u64,
}

#[derive(Debug, Deserialize)]
struct AckBatchResponse {
    acked: usize,
    committed_seq: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct LogLength {
    latest_seq: u64,
}

pub async fn run(config: &Config) -> Result<RelayWorkQueueReport> {
    let client = Client::builder()
        .http2_prior_knowledge()
        .pool_max_idle_per_host(64)
        .build()
        .context("build Relay h2c client")?;
    wait_healthy(&client, &config.relay_url).await?;

    let subject = format!("gcp-{}", config.run_id);
    let before = ProcessSample::capture()?;
    let publish_started = Instant::now();
    for start in (0..config.relay_messages).step_by(config.relay_batch_size) {
        let end = (start + config.relay_batch_size).min(config.relay_messages);
        let messages = (start..end)
            .map(|index| {
                json!({
                    "message_id": format!("relay-{}-{index}", config.run_id),
                    "payload": {"index": index, "body": "x".repeat(128)},
                    "headers": {"x-axiom-bench": config.run_id},
                    "priority": 10
                })
            })
            .collect::<Vec<_>>();
        checked(
            client
                .post(format!("{}/v1/{subject}/publish-batch", config.relay_url))
                .json(&json!({"messages": messages}))
                .send()
                .await?,
            "Relay publish-batch",
        )
        .await?;
    }
    let publish_elapsed = publish_started.elapsed();

    let consume_started = Instant::now();
    let deadline = consume_started + Duration::from_secs(300);
    let mut unique = BTreeSet::new();
    let mut committed_seq = None;
    while unique.len() < config.relay_messages {
        let remaining = config.relay_messages - unique.len();
        let response = checked(
            client
                .post(format!("{}/v1/{subject}/lease-batch", config.relay_url))
                .json(&json!({
                    "consumer_id": format!("gke-{}", config.run_id),
                    "max": remaining.min(config.relay_batch_size)
                }))
                .send()
                .await?,
            "Relay lease-batch",
        )
        .await?
        .json::<LeaseBatchResponse>()
        .await?;
        if response.leases.is_empty() {
            if Instant::now() >= deadline {
                bail!(
                    "Relay stopped at {}/{} unique leases",
                    unique.len(),
                    config.relay_messages
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }
        for lease in &response.leases {
            anyhow::ensure!(
                unique.insert(lease.seq),
                "Relay leased duplicate seq {} before acknowledgement",
                lease.seq
            );
        }
        let acks = response
            .leases
            .iter()
            .map(|lease| json!({"lease_id": lease.lease_id, "epoch": lease.epoch}))
            .collect::<Vec<_>>();
        let ack = checked(
            client
                .post(format!("{}/v1/{subject}/ack-batch", config.relay_url))
                .json(&json!({"acks": acks}))
                .send()
                .await?,
            "Relay ack-batch",
        )
        .await?
        .json::<AckBatchResponse>()
        .await?;
        anyhow::ensure!(
            ack.acked == response.leases.len(),
            "Relay acknowledged {}/{} leases",
            ack.acked,
            response.leases.len()
        );
        committed_seq = ack.committed_seq;
    }
    let consume_elapsed = consume_started.elapsed();
    let after = ProcessSample::capture()?;

    let length = checked(
        client
            .get(format!("{}/v1/{subject}/len", config.relay_url))
            .send()
            .await?,
        "Relay log length",
    )
    .await?
    .json::<LogLength>()
    .await?;
    anyhow::ensure!(
        length.latest_seq as usize == config.relay_messages,
        "Relay durable log length is {}, expected {}",
        length.latest_seq,
        config.relay_messages
    );
    let expected_committed = config.relay_messages.saturating_sub(1) as u64;
    anyhow::ensure!(
        committed_seq == Some(expected_committed),
        "Relay committed offset is {:?}, expected {expected_committed}",
        committed_seq
    );

    Ok(RelayWorkQueueReport {
        workload: Workload {
            journey: "durable h2c publish-batch -> replicated work-queue lease-batch -> epoch-fenced ack-batch",
            messages: config.relay_messages,
            publish_batch_size: config.relay_batch_size,
            lease_ack_batch_size: config.relay_batch_size,
        },
        publish_throughput_per_second: config.relay_messages as f64
            / publish_elapsed.as_secs_f64().max(f64::EPSILON),
        consume: LatencySeries::from_durations(config.relay_messages, &[consume_elapsed]),
        client_process: before.delta(after),
        final_committed_seq: expected_committed,
        unique_leases: unique.len(),
        errors: 0,
    })
}

async fn checked(response: reqwest::Response, operation: &str) -> Result<reqwest::Response> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    bail!("{operation} returned {status}: {body}")
}

async fn wait_healthy(client: &Client, url: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        let ready = client
            .get(format!("{url}/readyz"))
            .send()
            .await
            .is_ok_and(|response| response.status().is_success());
        let leader_known = match client.get(format!("{url}/raftz")).send().await {
            Ok(response) if response.status().is_success() => response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|status| status["leader"].as_u64())
                .is_some(),
            _ => false,
        };
        if ready && leader_known {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("Relay never became ready with a known Raft leader at {url}");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

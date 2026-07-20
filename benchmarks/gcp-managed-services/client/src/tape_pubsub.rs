use crate::metrics::{LatencySeries, ProcessDelta, ProcessSample};
use crate::Config;
use anyhow::{bail, Context, Result};
use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use google_cloud_pubsub::client::{Publisher, Subscriber};
use google_cloud_pubsub::model::Message;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, Instant};

const PULL_BATCH: usize = 1_000;

#[derive(Debug, Serialize)]
pub struct TapePubsubReport {
    pub workload: Workload,
    pub tape: Backend,
    pub cloud_pubsub: Backend,
    pub comparison: Comparison,
}

#[derive(Debug, Serialize)]
pub struct Workload {
    pub journey: &'static str,
    pub events: usize,
    pub payload_bytes: usize,
    pub samples: usize,
    pub prepare_concurrency: usize,
    pub pubsub_delivery: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Backend {
    pub prepare_throughput_per_second: f64,
    pub drain: LatencySeries,
    pub client_process: ProcessDelta,
    pub errors: usize,
    pub server_resources: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Comparison {
    pub tape_to_pubsub_throughput_ratio: f64,
    pub claim: &'static str,
}

#[derive(Debug, Deserialize)]
struct PullBatch {
    next_offset: u64,
    events: Vec<serde_json::Value>,
}

pub async fn run(config: &Config) -> Result<TapePubsubReport> {
    let http = Client::builder()
        .http2_prior_knowledge()
        .pool_max_idle_per_host(128)
        .build()
        .context("build Tape h2c client")?;
    wait_healthy(&http, &config.tape_url).await?;

    let payload = serde_json::to_vec(&json!({"body": "x".repeat(128)}))?;
    let tape_subscriptions = (0..config.pubsub_subscriptions.len())
        .map(|index| format!("gcp-{}-{index}", config.run_id))
        .collect::<Vec<_>>();
    for subscription in &tape_subscriptions {
        let response = http
            .post(format!(
                "{}/topics/gcp-bench/subscriptions",
                config.tape_url
            ))
            .json(&json!({"name": subscription}))
            .send()
            .await
            .context("create Tape benchmark subscription")?;
        if response.status() != StatusCode::CREATED {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            bail!("Tape subscription create returned {status}: {body}");
        }
    }

    let tape_prepare = Instant::now();
    futures::stream::iter(0..config.tape_events)
        .map(|index| {
            let http = http.clone();
            let url = config.tape_url.clone();
            let payload: serde_json::Value = serde_json::from_slice(&payload).unwrap();
            async move {
                let response = http
                    .post(format!("{url}/topics/gcp-bench/append"))
                    .json(&json!({"key": format!("event-{index}"), "payload": payload}))
                    .send()
                    .await?;
                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    bail!("Tape append returned {status}: {body}");
                }
                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(config.tape_prepare_concurrency)
        .try_collect::<Vec<_>>()
        .await?;
    let tape_prepare_elapsed = tape_prepare.elapsed();

    let topic = full_topic(config);
    let publisher = Publisher::builder(&topic).build().await?;
    let pubsub_prepare = Instant::now();
    let mut publish_results = FuturesUnordered::new();
    for _ in 0..config.tape_events {
        publish_results.push(publisher.publish(Message::new().set_data(payload.clone())));
    }
    while let Some(result) = publish_results.next().await {
        result.context("Pub/Sub publish acknowledgement")?;
    }
    let pubsub_prepare_elapsed = pubsub_prepare.elapsed();

    let tape_before = ProcessSample::capture()?;
    let mut tape_durations = Vec::with_capacity(tape_subscriptions.len());
    for subscription in &tape_subscriptions {
        tape_durations
            .push(drain_tape(&http, &config.tape_url, subscription, config.tape_events).await?);
    }
    let tape_after = ProcessSample::capture()?;

    let subscriber = Subscriber::builder().build().await?;
    let pubsub_before = ProcessSample::capture()?;
    let mut pubsub_durations = Vec::with_capacity(config.pubsub_subscriptions.len());
    for subscription in &config.pubsub_subscriptions {
        pubsub_durations.push(
            drain_pubsub(
                &subscriber,
                &full_subscription(config, subscription),
                config.tape_events,
                payload.len(),
            )
            .await?,
        );
    }
    let pubsub_after = ProcessSample::capture()?;

    let tape_series = LatencySeries::from_durations(config.tape_events, &tape_durations);
    let pubsub_series = LatencySeries::from_durations(config.tape_events, &pubsub_durations);
    let ratio = tape_series.throughput_per_second / pubsub_series.throughput_per_second;

    Ok(TapePubsubReport {
        workload: Workload {
            journey: "durable prepare outside sample; named pull drains full backlog and explicitly acknowledges progress",
            events: config.tape_events,
            payload_bytes: payload.len(),
            samples: config.pubsub_subscriptions.len(),
            prepare_concurrency: config.tape_prepare_concurrency,
            pubsub_delivery: "standard at-least-once StreamingPull; exactly-once is intentionally not enabled",
        },
        tape: Backend {
            prepare_throughput_per_second: config.tape_events as f64
                / tape_prepare_elapsed.as_secs_f64(),
            drain: tape_series,
            client_process: tape_before.delta(tape_after),
            errors: 0,
            server_resources: "collected separately from GKE pods/PVCs",
        },
        cloud_pubsub: Backend {
            prepare_throughput_per_second: config.tape_events as f64
                / pubsub_prepare_elapsed.as_secs_f64(),
            drain: pubsub_series,
            client_process: pubsub_before.delta(pubsub_after),
            errors: 0,
            server_resources: "provider_opaque",
        },
        comparison: Comparison {
            tape_to_pubsub_throughput_ratio: ratio,
            claim: "client-observed throughput/latency only; no managed-service server CPU, RSS, or disk claim",
        },
    })
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
            bail!("Tape never became ready with a known Raft leader at {url}");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn drain_tape(
    client: &Client,
    url: &str,
    subscription: &str,
    expected: usize,
) -> Result<Duration> {
    let started = Instant::now();
    let mut received = 0usize;
    while received < expected {
        let batch: PullBatch = client
            .post(format!(
                "{url}/topics/gcp-bench/subscriptions/{subscription}/pull"
            ))
            .json(&json!({"limit": PULL_BATCH}))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if batch.events.is_empty() {
            bail!("Tape subscription {subscription} ended at {received}/{expected}");
        }
        received += batch.events.len();
        client
            .post(format!(
                "{url}/topics/gcp-bench/subscriptions/{subscription}/ack"
            ))
            .json(&json!({"offset": batch.next_offset}))
            .send()
            .await?
            .error_for_status()?;
    }
    anyhow::ensure!(
        received == expected,
        "Tape returned {received}/{expected} events"
    );
    Ok(started.elapsed())
}

async fn drain_pubsub(
    subscriber: &Subscriber,
    subscription: &str,
    expected: usize,
    payload_bytes: usize,
) -> Result<Duration> {
    let started = Instant::now();
    let mut stream = subscriber.subscribe(subscription).build();
    for index in 0..expected {
        let (message, handle) = tokio::time::timeout(Duration::from_secs(120), stream.next())
            .await
            .with_context(|| format!("Pub/Sub timed out at {index}/{expected}"))?
            .context("Pub/Sub stream ended early")??;
        anyhow::ensure!(
            message.data.len() == payload_bytes,
            "Pub/Sub payload length mismatch"
        );
        handle.ack();
    }
    // The high-level client batches acknowledgements. Include one bounded
    // flush interval so the sample is not merely a delivery-only number.
    tokio::time::sleep(Duration::from_millis(250)).await;
    Ok(started.elapsed())
}

fn full_topic(config: &Config) -> String {
    if config.pubsub_topic.starts_with("projects/") {
        config.pubsub_topic.clone()
    } else {
        format!(
            "projects/{}/topics/{}",
            config.project_id, config.pubsub_topic
        )
    }
}

fn full_subscription(config: &Config, subscription: &str) -> String {
    if subscription.starts_with("projects/") {
        subscription.to_string()
    } else {
        format!(
            "projects/{}/subscriptions/{subscription}",
            config.project_id
        )
    }
}

use crate::metrics::{LatencySeries, ProcessDelta, ProcessSample};
use crate::receiver::{Receiver, ReceiverStats};
use crate::Config;
use anyhow::{bail, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{SecondsFormat, Utc};
use futures::{StreamExt, TryStreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct DeferCloudTasksReport {
    pub workload: Workload,
    pub defer: Backend,
    pub cloud_tasks: Backend,
    pub comparison: Comparison,
}

#[derive(Debug, Serialize)]
pub struct Workload {
    pub journey: &'static str,
    pub tasks_per_sample: usize,
    pub samples: usize,
    pub queue_dispatches_per_second: usize,
    pub queue_concurrency: usize,
    pub create_concurrency: usize,
}

#[derive(Debug, Serialize)]
pub struct Backend {
    pub create_throughput_per_second: f64,
    pub lifecycle: LatencySeries,
    pub client_process: ProcessDelta,
    pub requests_received: usize,
    pub duplicate_attempts: usize,
    pub errors: usize,
    pub server_resources: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Comparison {
    pub defer_to_cloud_tasks_throughput_ratio: f64,
    pub claim: &'static str,
}

#[derive(Debug, Deserialize)]
struct QueueSnapshot {
    terminal_count: usize,
}

#[derive(Debug, Deserialize)]
struct TaskList {
    #[serde(default)]
    tasks: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct MetadataToken {
    access_token: String,
}

pub async fn run(config: &Config) -> Result<DeferCloudTasksReport> {
    let client = Client::builder()
        .pool_max_idle_per_host(128)
        .build()
        .context("build task benchmark client")?;
    wait_healthy(&client, &config.defer_url).await?;
    let receiver = Receiver::new(
        client.clone(),
        config.receiver_url.clone(),
        config.receiver_secret.clone(),
    );
    receiver.warm().await?;
    let token = metadata_token(&client).await?;
    let samples = 5usize;

    let cloud_before = ProcessSample::capture()?;
    let mut cloud_create = Duration::ZERO;
    let mut cloud_lifecycle = Vec::with_capacity(samples);
    let mut cloud_stats = Vec::with_capacity(samples);
    for sample in 0..samples {
        let backend = format!("cloud-tasks-{sample}");
        receiver.reset(&backend).await?;
        let started = Instant::now();
        cloud_create += create_cloud_tasks(
            &client,
            config,
            &token,
            &backend,
            sample,
            config.tasks_per_sample,
        )
        .await?;
        cloud_stats.push(
            receiver
                .wait_unique(&backend, config.tasks_per_sample, Duration::from_secs(300))
                .await?,
        );
        wait_cloud_tasks_empty(&client, config, &token, Duration::from_secs(300)).await?;
        cloud_lifecycle.push(started.elapsed());
    }
    let cloud_after = ProcessSample::capture()?;

    let defer_before = ProcessSample::capture()?;
    let mut defer_create = Duration::ZERO;
    let mut defer_lifecycle = Vec::with_capacity(samples);
    let mut defer_stats = Vec::with_capacity(samples);
    for sample in 0..samples {
        let backend = format!("defer-{sample}");
        let queue = format!("gcp-{sample}");
        receiver.reset(&backend).await?;
        configure_defer_queue(&client, config, &queue).await?;
        let started = Instant::now();
        defer_create += create_defer_tasks(
            &client,
            config,
            &queue,
            &backend,
            sample,
            config.tasks_per_sample,
        )
        .await?;
        defer_stats.push(
            receiver
                .wait_unique(&backend, config.tasks_per_sample, Duration::from_secs(300))
                .await?,
        );
        wait_defer_terminal(
            &client,
            config,
            &queue,
            config.tasks_per_sample,
            Duration::from_secs(300),
        )
        .await?;
        defer_lifecycle.push(started.elapsed());
    }
    let defer_after = ProcessSample::capture()?;

    let cloud_series = LatencySeries::from_durations(config.tasks_per_sample, &cloud_lifecycle);
    let defer_series = LatencySeries::from_durations(config.tasks_per_sample, &defer_lifecycle);
    let ratio = defer_series.throughput_per_second / cloud_series.throughput_per_second;

    Ok(DeferCloudTasksReport {
        workload: Workload {
            journey: "individual durable create -> queue rate/concurrency permit -> real Cloud Run HTTP 204 -> terminal removal/commit",
            tasks_per_sample: config.tasks_per_sample,
            samples,
            queue_dispatches_per_second: 500,
            queue_concurrency: 100,
            create_concurrency: config.task_create_concurrency,
        },
        defer: backend(
            config.tasks_per_sample * samples,
            defer_create,
            defer_series,
            defer_before.delta(defer_after),
            &defer_stats,
            "collected separately from GKE pods/PVCs",
        ),
        cloud_tasks: backend(
            config.tasks_per_sample * samples,
            cloud_create,
            cloud_series,
            cloud_before.delta(cloud_after),
            &cloud_stats,
            "provider_opaque",
        ),
        comparison: Comparison {
            defer_to_cloud_tasks_throughput_ratio: ratio,
            claim: "client-observed create and terminal lifecycle only; no managed-service server CPU, RSS, or disk claim",
        },
    })
}

fn backend(
    tasks: usize,
    create: Duration,
    lifecycle: LatencySeries,
    client_process: ProcessDelta,
    stats: &[ReceiverStats],
    server_resources: &'static str,
) -> Backend {
    Backend {
        create_throughput_per_second: tasks as f64 / create.as_secs_f64(),
        lifecycle,
        client_process,
        requests_received: stats.iter().map(|sample| sample.requests).sum(),
        duplicate_attempts: stats.iter().map(|sample| sample.duplicates).sum(),
        errors: 0,
        server_resources,
    }
}

async fn create_cloud_tasks(
    client: &Client,
    config: &Config,
    token: &str,
    backend: &str,
    sample: usize,
    tasks: usize,
) -> Result<Duration> {
    let queue = full_queue(config);
    let endpoint = format!("https://cloudtasks.googleapis.com/v2/{queue}/tasks");
    let started = Instant::now();
    futures::stream::iter(0..tasks)
        .map(|index| {
            let client = client.clone();
            let endpoint = endpoint.clone();
            let token = token.to_string();
            let task_name = format!("{queue}/tasks/axb-{}-{sample}-{index}", config.run_id);
            let receiver_url = config.receiver_url.clone();
            let receiver_secret = config.receiver_secret.clone();
            let backend = backend.to_string();
            async move {
                let body = BASE64.encode(format!("{{\"sample\":{sample},\"index\":{index}}}"));
                let response = client
                    .post(endpoint)
                    .bearer_auth(token)
                    .json(&json!({
                        "task": {
                            "name": task_name,
                            "httpRequest": {
                                "httpMethod": "POST",
                                "url": format!("{receiver_url}/task"),
                                "headers": {
                                    "content-type": "application/json",
                                    "x-axiom-bench-secret": receiver_secret,
                                    "x-axiom-bench-backend": backend,
                                    "x-axiom-bench-key": format!("{sample}/{index}")
                                },
                                "body": body
                            }
                        }
                    }))
                    .send()
                    .await?;
                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    bail!("Cloud Tasks create returned {status}: {body}");
                }
                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(config.task_create_concurrency)
        .try_collect::<Vec<_>>()
        .await?;
    Ok(started.elapsed())
}

async fn create_defer_tasks(
    client: &Client,
    config: &Config,
    queue: &str,
    backend: &str,
    sample: usize,
    tasks: usize,
) -> Result<Duration> {
    let started = Instant::now();
    futures::stream::iter(0..tasks)
        .map(|index| {
            let client = client.clone();
            let url = config.defer_url.clone();
            let receiver_url = config.receiver_url.clone();
            let receiver_secret = config.receiver_secret.clone();
            let queue = queue.to_string();
            let backend = backend.to_string();
            async move {
                let response = client
                    .post(format!("{url}/v1/queues/{queue}/tasks"))
                    .json(&json!({
                        "task_id": format!("axb-{}-{sample}-{index}", config.run_id),
                        "target": {
                            "url": format!("{receiver_url}/task"),
                            "method": "POST",
                            "headers": {
                                "content-type": "application/json",
                                "x-axiom-bench-secret": receiver_secret,
                                "x-axiom-bench-backend": backend,
                                "x-axiom-bench-key": format!("{sample}/{index}")
                            }
                        },
                        "payload": {"sample": sample, "index": index},
                        "schedule_at": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                        "priority": 10,
                        "max_attempts": 3
                    }))
                    .send()
                    .await?;
                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    bail!("Defer task create returned {status}: {body}");
                }
                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(config.task_create_concurrency)
        .try_collect::<Vec<_>>()
        .await?;
    Ok(started.elapsed())
}

async fn configure_defer_queue(client: &Client, config: &Config, queue: &str) -> Result<()> {
    let response = client
        .put(format!("{}/v1/queues/{queue}", config.defer_url))
        .json(&json!({
            "max_in_flight": 100,
            "max_dispatch_per_tick": 100,
            "max_dispatches_per_second": 500,
            "max_burst_size": 100,
            "lease_ttl_ms": 30_000,
            "retry_backoff_ms": 1_000
        }))
        .send()
        .await?;
    if !response.status().is_success() {
        bail!("Defer queue configure returned {}", response.status());
    }
    Ok(())
}

async fn wait_defer_terminal(
    client: &Client,
    config: &Config,
    queue: &str,
    expected: usize,
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let snapshot: QueueSnapshot = client
            .get(format!("{}/v1/queues/{queue}", config.defer_url))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if snapshot.terminal_count >= expected {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "Defer queue {queue} stopped at {}/{} terminal tasks",
                snapshot.terminal_count,
                expected
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_cloud_tasks_empty(
    client: &Client,
    config: &Config,
    token: &str,
    timeout: Duration,
) -> Result<()> {
    let endpoint = format!(
        "https://cloudtasks.googleapis.com/v2/{}/tasks?pageSize=1",
        full_queue(config)
    );
    let deadline = Instant::now() + timeout;
    loop {
        let response: TaskList = client
            .get(&endpoint)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if response.tasks.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("Cloud Tasks queue did not become empty");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn metadata_token(client: &Client) -> Result<String> {
    let token: MetadataToken = client
        .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
        .header("Metadata-Flavor", "Google")
        .send()
        .await
        .context("request Workload Identity token")?
        .error_for_status()?
        .json()
        .await
        .context("decode Workload Identity token")?;
    Ok(token.access_token)
}

async fn wait_healthy(client: &Client, url: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        if client
            .get(format!("{url}/healthz"))
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!("Defer never became healthy at {url}");
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn full_queue(config: &Config) -> String {
    if config.cloud_tasks_queue.starts_with("projects/") {
        config.cloud_tasks_queue.clone()
    } else {
        format!(
            "projects/{}/locations/{}/queues/{}",
            config.project_id, config.region, config.cloud_tasks_queue
        )
    }
}

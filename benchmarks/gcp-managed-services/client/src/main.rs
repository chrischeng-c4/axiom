mod defer_tasks;
mod metrics;
mod receiver;
mod relay_work_queue;
mod tape_pubsub;

use anyhow::{Context, Result};
use serde::Serialize;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub project_id: String,
    pub region: String,
    pub run_id: String,
    pub tape_url: String,
    pub defer_url: String,
    pub relay_url: String,
    pub receiver_url: String,
    pub receiver_secret: String,
    pub pubsub_topic: String,
    pub pubsub_subscriptions: Vec<String>,
    pub cloud_tasks_queue: String,
    pub tape_events: usize,
    pub tape_prepare_concurrency: usize,
    pub tasks_per_sample: usize,
    pub task_create_concurrency: usize,
    pub relay_messages: usize,
    pub relay_batch_size: usize,
    pub tape_ready_replicas: usize,
    pub defer_ready_replicas: usize,
    pub relay_ready_replicas: usize,
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    status: &'static str,
    project_id: String,
    region: String,
    run_id: String,
    deployment: Deployment,
    tape_pubsub: Outcome<tape_pubsub::TapePubsubReport>,
    defer_cloud_tasks: Outcome<defer_tasks::DeferCloudTasksReport>,
    relay_work_queue: Outcome<relay_work_queue::RelayWorkQueueReport>,
}

#[derive(Serialize)]
struct Outcome<T> {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> Outcome<T> {
    fn capture(result: Result<T>) -> Self {
        match result {
            Ok(report) => Self {
                status: "passed",
                report: Some(report),
                error: None,
            },
            Err(error) => Self {
                status: "failed",
                report: None,
                error: Some(format!("{error:#}")),
            },
        }
    }

    fn passed(&self) -> bool {
        self.report.is_some()
    }
}

#[derive(Serialize)]
struct Deployment {
    gke: &'static str,
    tape_replicas_ready: usize,
    defer_replicas_ready: usize,
    relay_replicas_ready: usize,
    voters_per_service: usize,
    disk: &'static str,
    scaling_boundary: ScalingBoundary,
    cloud_run_min_instances: usize,
    security_note: &'static str,
}

#[derive(Serialize)]
struct ScalingBoundary {
    gke_capacity: &'static str,
    cpu_memory_replica_policy: &'static str,
    disk_sharding: &'static str,
    claim: &'static str,
}

impl Config {
    fn from_env() -> Result<Self> {
        Ok(Self {
            project_id: required("PROJECT_ID")?,
            region: required("REGION")?,
            run_id: required("RUN_ID")?,
            tape_url: required("TAPE_URL")?,
            defer_url: required("DEFER_URL")?,
            relay_url: required("RELAY_URL")?,
            receiver_url: required("RECEIVER_URL")?.trim_end_matches('/').to_string(),
            receiver_secret: required("RECEIVER_SECRET")?,
            pubsub_topic: required("PUBSUB_TOPIC")?,
            pubsub_subscriptions: required("PUBSUB_SUBSCRIPTIONS")?
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            cloud_tasks_queue: required("CLOUD_TASKS_QUEUE")?,
            tape_events: optional_usize("TAPE_EVENTS", 5_000)?,
            tape_prepare_concurrency: optional_usize("TAPE_PREPARE_CONCURRENCY", 64)?,
            tasks_per_sample: optional_usize("TASKS_PER_SAMPLE", 200)?,
            task_create_concurrency: optional_usize("TASK_CREATE_CONCURRENCY", 64)?,
            relay_messages: optional_usize("RELAY_MESSAGES", 200)?,
            relay_batch_size: optional_usize("RELAY_BATCH_SIZE", 25)?,
            tape_ready_replicas: required_usize("TAPE_READY_REPLICAS")?,
            defer_ready_replicas: required_usize("DEFER_READY_REPLICAS")?,
            relay_ready_replicas: required_usize("RELAY_READY_REPLICAS")?,
        })
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    anyhow::ensure!(
        config.pubsub_subscriptions.len() >= 3,
        "at least three Pub/Sub subscriptions are required"
    );
    anyhow::ensure!(config.relay_messages > 0, "RELAY_MESSAGES must be positive");
    anyhow::ensure!(
        config.relay_batch_size > 0,
        "RELAY_BATCH_SIZE must be positive"
    );
    let relay_work_queue = Outcome::capture(relay_work_queue::run(&config).await);
    let tape_pubsub = Outcome::capture(tape_pubsub::run(&config).await);
    let defer_cloud_tasks = Outcome::capture(defer_tasks::run(&config).await);
    let outcomes = [
        relay_work_queue.passed(),
        tape_pubsub.passed(),
        defer_cloud_tasks.passed(),
    ];
    let status = if outcomes.iter().all(|passed| *passed) {
        "passed"
    } else if outcomes.iter().any(|passed| *passed) {
        "partial"
    } else {
        "failed"
    };
    let report = Report {
        schema: "axiom.gcp-managed-bench/2",
        status,
        project_id: config.project_id,
        region: config.region,
        run_id: config.run_id,
        deployment: Deployment {
            gke: "Autopilot regional cluster; benchmark client runs in-cluster",
            tape_replicas_ready: config.tape_ready_replicas,
            defer_replicas_ready: config.defer_ready_replicas,
            relay_replicas_ready: config.relay_ready_replicas,
            voters_per_service: 3,
            disk: "three 10Gi pd-standard WaitForFirstConsumer PVCs for each of Tape, Defer, and Relay",
            scaling_boundary: ScalingBoundary {
                gke_capacity: "Autopilot provisions the bounded capacity needed to schedule the three 3-voter services",
                cpu_memory_replica_policy: "service-k8s whole-layer planning is unit-tested but not wired to an operator membership transition",
                disk_sharding: "service-k8s plans one new shard above 1GiB per shard, but all three services still lack the durable-byte signal and migration actuator, pin shardCount=1, and fix each PVC at 10Gi in this run",
                claim: "no autonomous service autoscaling claim; no extra scale-out resources are created",
            },
            cloud_run_min_instances: 0,
            security_note: "short-lived public receiver protected by a random per-run header; Axiom client auth and peer TLS are disabled for this calibration cell",
        },
        tape_pubsub,
        defer_cloud_tasks,
        relay_work_queue,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("missing environment variable {name}"))
}

fn optional_usize(name: &str, default: usize) -> Result<usize> {
    env::var(name)
        .ok()
        .map(|value| {
            value
                .parse()
                .with_context(|| format!("{name} must be an unsigned integer"))
        })
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn required_usize(name: &str) -> Result<usize> {
    required(name)?
        .parse()
        .with_context(|| format!("{name} must be an unsigned integer"))
}

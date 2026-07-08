//! Closed-loop work-queue competitor comparison for relay.
//!
//! One harness drives the same two phases against every backend:
//! publish N messages, then lease/reserve/read and ack/delete N messages.
//! Backends:
//!
//! - `engine`: relay in-process durable core baseline (disk + fsync policy).
//! - `relay`: relay over h2c using CBOR publish/lease/ack batch routes.
//! - `rabbitmq`: RabbitMQ queue, persistent messages, publisher confirms,
//!   manual ack, quorum queue by default.
//! - `nats`: NATS JetStream WorkQueue stream, explicit ack, file storage.
//! - `redis` / `dragonfly`: Redis Streams consumer group, XREADGROUP, XACK,
//!   and XDEL to match relay's delete-on-ack posture. Append-only persistence
//!   must be enabled or the harness refuses to run the backend.
//!
//! Connections and backend setup happen outside the timed regions.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use futures::StreamExt;
use serde_json::json;

#[derive(Parser, Debug, Clone)]
struct Args {
    /// engine | relay | rabbitmq | nats | redis | dragonfly
    #[arg(long)]
    backend: String,
    /// Total messages per phase.
    #[arg(long, default_value_t = 200_000)]
    ops: usize,
    /// Concurrent producers/consumers.
    #[arg(long, default_value_t = 50)]
    concurrency: usize,
    /// Messages per publish/lease/ack batch.
    #[arg(long, default_value_t = 100)]
    batch: usize,
    /// Payload size in bytes.
    #[arg(long, default_value_t = 64)]
    payload_size: usize,
    /// Subject/queue/stream. Defaults to a unique bench name.
    #[arg(long)]
    subject: Option<String>,
    /// relay service URL.
    #[arg(long, default_value = "http://127.0.0.1:7000")]
    relay_url: String,
    /// Redis/Dragonfly URL.
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    redis_url: String,
    /// NATS URL.
    #[arg(long, default_value = "nats://127.0.0.1:4222")]
    nats_url: String,
    /// RabbitMQ AMQP URL.
    #[arg(long, default_value = "amqp://127.0.0.1:5672/%2f")]
    rabbitmq_url: String,
    /// Use a classic RabbitMQ queue instead of quorum.
    #[arg(long)]
    rabbitmq_classic: bool,
    /// Number of h2c clients for relay-over-h2c.
    #[arg(long)]
    relay_clients: Option<usize>,
}

#[derive(Debug, Clone)]
struct Plan {
    subject: String,
    batch: usize,
    requests_per_worker: usize,
    actual_ops: usize,
    payload: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let plan = make_plan(&args);
    println!(
        "\n== backend={} subject={} ops={} concurrency={} batch={} payload={}B ==",
        args.backend,
        plan.subject,
        plan.actual_ops,
        args.concurrency,
        plan.batch,
        args.payload_size
    );
    match args.backend.as_str() {
        "engine" => bench_engine(&args, &plan).await,
        "relay" => bench_relay_h2c(&args, &plan).await,
        "rabbitmq" => bench_rabbitmq(&args, &plan).await,
        "nats" => bench_nats(&args, &plan).await,
        "redis" | "dragonfly" => bench_redis_streams(&args, &plan).await,
        other => bail!("unknown backend: {other}"),
    }
}

fn make_plan(args: &Args) -> Plan {
    let batch = args.batch.max(1);
    let per_worker = args.ops / args.concurrency.max(1);
    let requests_per_worker = (per_worker / batch).max(1);
    let actual_ops = args.concurrency.max(1) * requests_per_worker * batch;
    let subject = args.subject.clone().unwrap_or_else(|| {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("relay-bench-{now}-{}", std::process::id())
    });
    Plan {
        subject,
        batch,
        requests_per_worker,
        actual_ops,
        payload: "x".repeat(args.payload_size),
    }
}

fn msg_id(worker: usize, request: usize, item: usize, batch: usize) -> String {
    format!("m-{worker}-{}", request * batch + item)
}

fn summarize(phase: &str, ops: usize, batch: usize, elapsed: Duration, mut lat_us: Vec<u64>) {
    lat_us.sort_unstable();
    let pct = |p: f64| -> u64 {
        if lat_us.is_empty() {
            0
        } else {
            lat_us[(((lat_us.len() - 1) as f64) * p).round() as usize]
        }
    };
    let unit = if batch > 1 { "batch" } else { "op" };
    println!(
        "  {phase:9} {:>11.0} msg/s   p50 {:>6}us/{unit}   p99 {:>7}us   p99.9 {:>7}us   ({:.2}s)",
        ops as f64 / elapsed.as_secs_f64(),
        pct(0.50),
        pct(0.99),
        pct(0.999),
        elapsed.as_secs_f64(),
    );
}

async fn bench_engine(args: &Args, plan: &Plan) -> Result<()> {
    use chrono::Utc;
    use relay::{Relay, RelayCoreConfig};

    let data_dir = tempfile::Builder::new()
        .prefix("relay-bench-engine-")
        .tempdir()?;
    let mut config = RelayCoreConfig::default();
    config.data_dir = data_dir.path().to_string_lossy().into_owned();
    println!("  (engine durable data dir -> {})", config.data_dir);
    let relay = Arc::new(Relay::new(config));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let relay = Arc::clone(&relay);
        let subject = plan.subject.clone();
        let payload = plan.payload.clone();
        let batch = plan.batch;
        let requests = plan.requests_per_worker;
        handles.push(tokio::task::spawn_blocking(move || -> Result<Vec<u64>> {
            let mut lat = Vec::with_capacity(requests);
            for request in 0..requests {
                let messages = (0..batch)
                    .map(|item| {
                        (
                            msg_id(worker, request, item, batch),
                            json!({ "payload": payload }),
                            BTreeMap::new(),
                            relay::DEFAULT_PRIORITY,
                        )
                    })
                    .collect();
                let t = Instant::now();
                relay.publish_batch(&subject, messages, Utc::now())?;
                lat.push(t.elapsed().as_micros() as u64);
            }
            Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize("publish", plan.actual_ops, plan.batch, start.elapsed(), all);

    let acked = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let relay = Arc::clone(&relay);
        let subject = plan.subject.clone();
        let acked = Arc::clone(&acked);
        let batch = plan.batch;
        let total = plan.actual_ops;
        handles.push(tokio::task::spawn_blocking(move || -> Result<Vec<u64>> {
            let consumer = format!("c-{worker}");
            let mut lat = Vec::new();
            while acked.load(Ordering::Relaxed) < total {
                let t = Instant::now();
                let leases = relay.lease_batch(&subject, &consumer, batch, Utc::now())?;
                if leases.is_empty() {
                    break;
                }
                let acks: Vec<_> = leases
                    .iter()
                    .map(|lease| (lease.lease_id.clone(), Some(lease.epoch)))
                    .collect();
                let (n, _) = relay.ack_batch(&subject, &acks)?;
                acked.fetch_add(n, Ordering::Relaxed);
                lat.push(t.elapsed().as_micros() as u64);
            }
            Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize(
        "lease_ack",
        acked.load(Ordering::Relaxed),
        plan.batch,
        start.elapsed(),
        all,
    );
    Ok(())
}

async fn bench_relay_h2c(args: &Args, plan: &Plan) -> Result<()> {
    use relay::wire::{
        self, AckBatchRequest, AckBatchResponse, AckOne, LeaseBatchRequest, LeaseBatchResponse,
        PublishBatchItem, PublishBatchRequest,
    };

    let clients = args.relay_clients.unwrap_or(args.concurrency).max(1);
    let pool = h2c::H2cPool::with_connections_and(clients, Some(Duration::from_secs(10)), None)?;
    for _ in 0..pool.connections() {
        let _ = pool.get(format!("{}/healthz", args.relay_url)).send().await;
    }

    let publish_url = format!("{}/v1/{}/publish-batch", args.relay_url, plan.subject);
    let lease_url = format!("{}/v1/{}/lease-batch", args.relay_url, plan.subject);
    let ack_url = format!("{}/v1/{}/ack-batch", args.relay_url, plan.subject);

    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let client = pool.client().clone();
        let url = publish_url.clone();
        let payload = plan.payload.clone();
        let batch = plan.batch;
        let requests = plan.requests_per_worker;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::with_capacity(requests);
            for request in 0..requests {
                let messages = (0..batch)
                    .map(|item| PublishBatchItem {
                        message_id: msg_id(worker, request, item, batch),
                        payload: json!({ "payload": payload }),
                        headers: BTreeMap::new(),
                        priority: relay::DEFAULT_PRIORITY,
                    })
                    .collect();
                let body = wire::to_cbor(&PublishBatchRequest { messages });
                let t = Instant::now();
                let res = client
                    .post(&url)
                    .header("content-type", wire::CBOR)
                    .header("accept", wire::CBOR)
                    .body(body)
                    .send()
                    .await?;
                if !res.status().is_success() {
                    bail!("relay publish-batch failed: {}", res.status());
                }
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize("publish", plan.actual_ops, plan.batch, start.elapsed(), all);

    let acked = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let client = pool.client().clone();
        let lease_url = lease_url.clone();
        let ack_url = ack_url.clone();
        let acked = Arc::clone(&acked);
        let batch = plan.batch;
        let total = plan.actual_ops;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            let consumer_id = format!("c-{worker}");
            while acked.load(Ordering::Relaxed) < total {
                let t = Instant::now();
                let body = wire::to_cbor(&LeaseBatchRequest {
                    consumer_id: consumer_id.clone(),
                    max: batch,
                });
                let res = client
                    .post(&lease_url)
                    .header("content-type", wire::CBOR)
                    .header("accept", wire::CBOR)
                    .body(body)
                    .send()
                    .await?;
                if !res.status().is_success() {
                    bail!("relay lease-batch failed: {}", res.status());
                }
                let bytes = res.bytes().await?;
                let leases: LeaseBatchResponse = wire::from_cbor(&bytes)?;
                if leases.leases.is_empty() {
                    break;
                }
                let acks = leases
                    .leases
                    .into_iter()
                    .map(|lease| AckOne {
                        lease_id: lease.lease_id,
                        epoch: Some(lease.epoch),
                    })
                    .collect();
                let body = wire::to_cbor(&AckBatchRequest { acks });
                let res = client
                    .post(&ack_url)
                    .header("content-type", wire::CBOR)
                    .header("accept", wire::CBOR)
                    .body(body)
                    .send()
                    .await?;
                if !res.status().is_success() {
                    bail!("relay ack-batch failed: {}", res.status());
                }
                let bytes = res.bytes().await?;
                let ack: AckBatchResponse = wire::from_cbor(&bytes)?;
                acked.fetch_add(ack.acked, Ordering::Relaxed);
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize(
        "lease_ack",
        acked.load(Ordering::Relaxed),
        plan.batch,
        start.elapsed(),
        all,
    );
    Ok(())
}

async fn bench_redis_streams(args: &Args, plan: &Plan) -> Result<()> {
    use redis::streams::{StreamReadOptions, StreamReadReply};

    let client = redis::Client::open(args.redis_url.clone())?;
    let group = "relay-bench-group";
    {
        let mut con = client.get_multiplexed_async_connection().await?;
        require_redis_appendonly(&mut con).await?;
        redis::cmd("DEL")
            .arg(&plan.subject)
            .query_async::<()>(&mut con)
            .await?;
        let _: redis::RedisResult<()> = redis::cmd("XGROUP")
            .arg("CREATE")
            .arg(&plan.subject)
            .arg(group)
            .arg("0")
            .arg("MKSTREAM")
            .query_async(&mut con)
            .await;
    }

    let start = Instant::now();
    let conns = connect_redis_workers(&client, args.concurrency).await?;
    let mut handles = Vec::new();
    for (worker, mut con) in conns.into_iter().enumerate() {
        let stream = plan.subject.clone();
        let payload = plan.payload.clone();
        let batch = plan.batch;
        let requests = plan.requests_per_worker;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::with_capacity(requests);
            for request in 0..requests {
                let mut pipe = redis::pipe();
                for item in 0..batch {
                    pipe.cmd("XADD")
                        .arg(&stream)
                        .arg("*")
                        .arg("message_id")
                        .arg(msg_id(worker, request, item, batch))
                        .arg("payload")
                        .arg(&payload)
                        .arg("priority")
                        .arg(relay::DEFAULT_PRIORITY);
                }
                let t = Instant::now();
                pipe.query_async::<()>(&mut con).await?;
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize("publish", plan.actual_ops, plan.batch, start.elapsed(), all);

    let acked = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let conns = connect_redis_workers(&client, args.concurrency).await?;
    let mut handles = Vec::new();
    for (worker, mut con) in conns.into_iter().enumerate() {
        let stream = plan.subject.clone();
        let consumer = format!("c-{worker}");
        let acked = Arc::clone(&acked);
        let batch = plan.batch;
        let total = plan.actual_ops;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            while acked.load(Ordering::Relaxed) < total {
                let opts = StreamReadOptions::default()
                    .group(group, &consumer)
                    .count(batch)
                    .block(10);
                let t = Instant::now();
                let reply: StreamReadReply = redis::cmd("XREADGROUP")
                    .arg(&opts)
                    .arg("STREAMS")
                    .arg(&stream)
                    .arg(">")
                    .query_async(&mut con)
                    .await?;
                let ids: Vec<String> = reply
                    .keys
                    .into_iter()
                    .flat_map(|key| key.ids.into_iter().map(|id| id.id))
                    .collect();
                if ids.is_empty() {
                    break;
                }
                let mut pipe = redis::pipe();
                pipe.cmd("XACK").arg(&stream).arg(group).arg(&ids);
                pipe.cmd("XDEL").arg(&stream).arg(&ids);
                pipe.query_async::<()>(&mut con).await?;
                acked.fetch_add(ids.len(), Ordering::Relaxed);
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize(
        "lease_ack",
        acked.load(Ordering::Relaxed),
        plan.batch,
        start.elapsed(),
        all,
    );
    Ok(())
}

async fn require_redis_appendonly(con: &mut redis::aio::MultiplexedConnection) -> Result<()> {
    let info: String = redis::cmd("INFO")
        .arg("persistence")
        .query_async(con)
        .await?;
    let append_only = info
        .lines()
        .any(|line| matches!(line.trim(), "aof_enabled:1" | "appendonly:1"));
    if !append_only {
        bail!(
            "Redis/Dragonfly backend is not durable: append-only persistence is disabled. Start Redis with `appendonly yes` or Dragonfly with append-only persistence before running this benchmark."
        );
    }
    Ok(())
}

async fn connect_redis_workers(
    client: &redis::Client,
    n: usize,
) -> Result<Vec<redis::aio::MultiplexedConnection>> {
    let mut conns = Vec::with_capacity(n);
    for _ in 0..n {
        conns.push(client.get_multiplexed_async_connection().await?);
    }
    Ok(conns)
}

async fn bench_nats(args: &Args, plan: &Plan) -> Result<()> {
    use async_nats::jetstream::{
        self,
        consumer::{pull::Config as ConsumerConfig, AckPolicy, DeliverPolicy},
        stream::{Config as StreamConfig, RetentionPolicy, StorageType},
    };

    let client = async_nats::connect(&args.nats_url).await?;
    let js = jetstream::new(client);
    let stream_name = plan.subject.replace('-', "_").to_uppercase();
    let nats_subject = format!("bench.{}", plan.subject);
    let _ = js.delete_stream(&stream_name).await;
    let stream = js
        .get_or_create_stream(StreamConfig {
            name: stream_name.clone(),
            subjects: vec![nats_subject.clone()],
            retention: RetentionPolicy::WorkQueue,
            storage: StorageType::File,
            ..Default::default()
        })
        .await?;

    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let js = js.clone();
        let subject = nats_subject.clone();
        let payload = plan.payload.clone();
        let batch = plan.batch;
        let requests = plan.requests_per_worker;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::with_capacity(requests);
            for request in 0..requests {
                let t = Instant::now();
                for item in 0..batch {
                    let mut headers = async_nats::HeaderMap::new();
                    headers.insert("Nats-Msg-Id", msg_id(worker, request, item, batch));
                    js.publish_with_headers(subject.clone(), headers, payload.clone().into())
                        .await?
                        .await?;
                }
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize("publish", plan.actual_ops, plan.batch, start.elapsed(), all);

    let consumer = stream
        .get_or_create_consumer(
            "relay_bench",
            ConsumerConfig {
                durable_name: Some("relay_bench".to_string()),
                ack_policy: AckPolicy::Explicit,
                deliver_policy: DeliverPolicy::All,
                filter_subject: nats_subject,
                ack_wait: Duration::from_secs(30),
                max_deliver: 5,
                ..Default::default()
            },
        )
        .await?;
    let consumer = Arc::new(consumer);
    let acked = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..args.concurrency {
        let consumer = Arc::clone(&consumer);
        let acked = Arc::clone(&acked);
        let batch = plan.batch;
        let total = plan.actual_ops;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::new();
            while acked.load(Ordering::Relaxed) < total {
                let t = Instant::now();
                let mut messages = consumer
                    .batch()
                    .max_messages(batch)
                    .expires(Duration::from_millis(100))
                    .messages()
                    .await?;
                let mut n = 0usize;
                while let Some(message) = messages.next().await {
                    let message = message.map_err(|e| anyhow!("{e}"))?;
                    message.ack().await.map_err(|e| anyhow!("{e}"))?;
                    n += 1;
                }
                if n == 0 {
                    break;
                }
                acked.fetch_add(n, Ordering::Relaxed);
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize(
        "lease_ack",
        acked.load(Ordering::Relaxed),
        plan.batch,
        start.elapsed(),
        all,
    );
    Ok(())
}

async fn bench_rabbitmq(args: &Args, plan: &Plan) -> Result<()> {
    use lapin::{
        options::{
            BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
            ConfirmSelectOptions, QueueDeclareOptions,
        },
        types::{AMQPValue, FieldTable, LongString, ShortString},
        BasicProperties, Connection, ConnectionProperties,
    };

    let conn = Connection::connect(&args.rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    let mut args_table = FieldTable::default();
    if !args.rabbitmq_classic {
        args_table.insert(
            ShortString::from("x-queue-type"),
            AMQPValue::LongString(LongString::from("quorum")),
        );
    }
    channel
        .queue_declare(
            &plan.subject,
            QueueDeclareOptions {
                durable: true,
                auto_delete: false,
                ..Default::default()
            },
            args_table,
        )
        .await?;

    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let conn = Connection::connect(&args.rabbitmq_url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;
        let queue = plan.subject.clone();
        let payload = plan.payload.clone();
        let batch = plan.batch;
        let requests = plan.requests_per_worker;
        handles.push(tokio::spawn(async move {
            let mut lat = Vec::with_capacity(requests);
            for request in 0..requests {
                let t = Instant::now();
                let mut confirms = Vec::with_capacity(batch);
                for item in 0..batch {
                    let id = msg_id(worker, request, item, batch);
                    let confirm = channel
                        .basic_publish(
                            "",
                            &queue,
                            BasicPublishOptions::default(),
                            payload.as_bytes(),
                            BasicProperties::default()
                                .with_delivery_mode(2)
                                .with_message_id(id.into()),
                        )
                        .await?;
                    confirms.push(confirm);
                }
                for confirm in confirms {
                    confirm.await?;
                }
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize("publish", plan.actual_ops, plan.batch, start.elapsed(), all);

    let acked = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut handles = Vec::new();
    for worker in 0..args.concurrency {
        let conn = Connection::connect(&args.rabbitmq_url, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        channel
            .basic_qos(plan.batch as u16, BasicQosOptions::default())
            .await?;
        let consumer = channel
            .basic_consume(
                &plan.subject,
                &format!("c-{worker}"),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let acked = Arc::clone(&acked);
        let batch = plan.batch;
        let total = plan.actual_ops;
        handles.push(tokio::spawn(async move {
            let mut consumer = consumer;
            let mut lat = Vec::new();
            while acked.load(Ordering::Relaxed) < total {
                let t = Instant::now();
                let mut deliveries = Vec::with_capacity(batch);
                while deliveries.len() < batch {
                    match tokio::time::timeout(Duration::from_millis(100), consumer.next()).await {
                        Ok(Some(delivery)) => deliveries.push(delivery?),
                        Ok(None) | Err(_) => break,
                    }
                }
                if deliveries.is_empty() {
                    break;
                }
                for delivery in &deliveries {
                    delivery.ack(BasicAckOptions::default()).await?;
                }
                acked.fetch_add(deliveries.len(), Ordering::Relaxed);
                lat.push(t.elapsed().as_micros() as u64);
            }
            Result::<Vec<u64>>::Ok(lat)
        }));
    }
    let mut all = Vec::new();
    for handle in handles {
        all.extend(handle.await??);
    }
    summarize(
        "lease_ack",
        acked.load(Ordering::Relaxed),
        plan.batch,
        start.elapsed(),
        all,
    );
    Ok(())
}

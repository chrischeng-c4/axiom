//! One-off competitor perf comparison (issue #126).
//!
//! ONE closed-loop load harness, four interchangeable backends running the
//! identical SET-then-GET workload and the identical measurement code:
//!
//!   engine     keep's KvEngine in-process (raw ceiling, no network/protocol)
//!   keep       keep over HTTP/2 cleartext + JSON
//!   redis      RESP over TCP (single-thread baseline)
//!   dragonfly  RESP over TCP (the multi-core bar)
//!
//! `--batch B` (B>1) amortizes per-request overhead over B keys per round trip:
//! keep uses `:mset`/`:mget`, RESP uses `MSET`/`MGET`, engine uses `mset`/`mget`.
//! One "op" = one key either way, so ops/s stays comparable; latency becomes
//! per-batch.
//!
//! Connections are pre-established / pre-warmed OUTSIDE the timed region.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use clap::Parser;
use serde_json::json;

#[derive(Parser, Debug, Clone)]
struct Args {
    /// engine | keep | redis | dragonfly
    #[arg(long)]
    backend: String,
    /// Total operations (keys) per phase.
    #[arg(long, default_value_t = 200_000)]
    ops: usize,
    /// Concurrent workers.
    #[arg(long, default_value_t = 50)]
    concurrency: usize,
    /// Keys per request (1 = single-key; >1 = MSET/MGET / :mset / :mget batch).
    #[arg(long, default_value_t = 1)]
    batch: usize,
    /// Value size in bytes.
    #[arg(long, default_value_t = 64)]
    value_size: usize,
    /// Distinct keys.
    #[arg(long, default_value_t = 100_000)]
    keyspace: usize,
    /// keep base URL.
    #[arg(long, default_value = "http://127.0.0.1:7117")]
    keep_url: String,
    /// RESP address for redis/dragonfly.
    #[arg(long, default_value = "redis://127.0.0.1:6379")]
    addr: String,
    /// Engine shard count (engine backend only).
    #[arg(long, default_value_t = 256)]
    shards: usize,
    /// keep backend: number of separate HTTP/2 connections (reqwest Clients).
    #[arg(long)]
    keep_clients: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let val = "x".repeat(args.value_size);
    println!(
        "\n== backend={} ops={} concurrency={} batch={} value={}B keyspace={} ==",
        args.backend, args.ops, args.concurrency, args.batch, args.value_size, args.keyspace
    );
    match args.backend.as_str() {
        "engine" => bench_engine(&args, &val).await?,
        "keep" => bench_keep(&args, &val).await?,
        "redis" | "dragonfly" => bench_resp(&args, &val).await?,
        other => return Err(format!("unknown backend: {other}").into()),
    }
    Ok(())
}

fn key_for(i: usize, keyspace: usize) -> String {
    format!("benchkey:{}", i % keyspace)
}

/// requests-per-worker and actual measured op count for the chosen batch.
fn plan(args: &Args) -> (usize, usize) {
    let batch = args.batch.max(1);
    let per_keys = args.ops / args.concurrency;
    let n_req = (per_keys / batch).max(1);
    let actual_ops = args.concurrency * n_req * batch;
    (n_req, actual_ops)
}

fn summarize(phase: &str, ops: usize, batch: usize, elapsed: Duration, mut lat_us: Vec<u64>) {
    lat_us.sort_unstable();
    let pct = |p: f64| -> u64 {
        if lat_us.is_empty() {
            return 0;
        }
        lat_us[(((lat_us.len() - 1) as f64) * p).round() as usize]
    };
    let secs = elapsed.as_secs_f64();
    let unit = if batch > 1 { "batch" } else { "op" };
    println!(
        "  {phase:4}  {:>11.0} ops/s   p50 {:>6}us/{unit}   p99 {:>7}us   p99.9 {:>7}us   ({secs:.2}s)",
        ops as f64 / secs,
        pct(0.50),
        pct(0.99),
        pct(0.999),
    );
}

// --------------------------------------------------------------------------
// engine
// --------------------------------------------------------------------------
async fn bench_engine(args: &Args, val: &str) -> Result<(), Box<dyn std::error::Error>> {
    use keep::{KvEngine, KvKey, KvValue};
    let engine = Arc::new(KvEngine::with_shards(args.shards));
    let (n_req, actual) = plan(args);
    let batch = args.batch.max(1);

    for phase in ["SET", "GET"] {
        let start = Instant::now();
        let mut handles = Vec::new();
        for w in 0..args.concurrency {
            let engine = engine.clone();
            let val = val.to_string();
            let keyspace = args.keyspace;
            handles.push(tokio::task::spawn_blocking(move || {
                let mut lat = Vec::with_capacity(n_req);
                for r in 0..n_req {
                    let base = (w * n_req + r) * batch;
                    let keys: Vec<KvKey> = (0..batch)
                        .map(|j| KvKey::new(key_for(base + j, keyspace)).unwrap())
                        .collect();
                    let t = Instant::now();
                    if phase == "SET" {
                        let pairs: Vec<(&KvKey, KvValue)> = keys
                            .iter()
                            .map(|k| (k, KvValue::String(val.clone())))
                            .collect();
                        engine.mset(&pairs, None).unwrap();
                    } else {
                        let refs: Vec<&KvKey> = keys.iter().collect();
                        let _ = engine.mget(&refs);
                    }
                    lat.push(t.elapsed().as_micros() as u64);
                }
                lat
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.await?);
        }
        summarize(phase, actual, batch, start.elapsed(), all);
    }
    Ok(())
}

// --------------------------------------------------------------------------
// keep over HTTP/2 + JSON
// --------------------------------------------------------------------------
async fn bench_keep(args: &Args, val: &str) -> Result<(), Box<dyn std::error::Error>> {
    let n_clients = args.keep_clients.unwrap_or(args.concurrency).max(1);
    let mut clients = Vec::with_capacity(n_clients);
    for _ in 0..n_clients {
        clients.push(
            reqwest::Client::builder()
                .http2_prior_knowledge()
                .pool_max_idle_per_host(1)
                .build()?,
        );
    }
    println!("  (keep over {n_clients} HTTP/2 connection(s))");
    for c in &clients {
        let _ = c.get(format!("{}/healthz", args.keep_url)).send().await;
    }

    let (n_req, actual) = plan(args);
    let batch = args.batch.max(1);

    for phase in ["SET", "GET"] {
        let start = Instant::now();
        let mut handles = Vec::new();
        for w in 0..args.concurrency {
            let client = clients[w % n_clients].clone();
            let base_url = args.keep_url.clone();
            let val = val.to_string();
            let keyspace = args.keyspace;
            handles.push(tokio::spawn(async move {
                let mut lat = Vec::with_capacity(n_req);
                for r in 0..n_req {
                    let kbase = (w * n_req + r) * batch;
                    let keys: Vec<String> =
                        (0..batch).map(|j| key_for(kbase + j, keyspace)).collect();
                    let t = Instant::now();
                    if batch == 1 {
                        let url = format!("{base_url}/v1/kv/{}", keys[0]);
                        if phase == "SET" {
                            let _ = client.put(&url).json(&json!({ "value": val })).send().await;
                        } else {
                            let _ = client.get(&url).send().await;
                        }
                    } else if phase == "SET" {
                        let entries: HashMap<&str, &str> =
                            keys.iter().map(|k| (k.as_str(), val.as_str())).collect();
                        let _ = client
                            .post(format!("{base_url}/v1/kv:mset"))
                            .json(&json!({ "entries": entries }))
                            .send()
                            .await;
                    } else {
                        let _ = client
                            .post(format!("{base_url}/v1/kv:mget"))
                            .json(&json!({ "keys": keys }))
                            .send()
                            .await;
                    }
                    lat.push(t.elapsed().as_micros() as u64);
                }
                lat
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.await?);
        }
        summarize(phase, actual, batch, start.elapsed(), all);
    }
    Ok(())
}

// --------------------------------------------------------------------------
// redis / dragonfly (RESP)
// --------------------------------------------------------------------------
async fn bench_resp(args: &Args, val: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open(args.addr.clone())?;
    {
        let mut c = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING").query_async::<String>(&mut c).await?;
    }
    let (n_req, actual) = plan(args);
    let batch = args.batch.max(1);

    for phase in ["SET", "GET"] {
        // pre-establish one connection per worker, OUTSIDE the timed region.
        let conns: Vec<_> = futures::future::join_all(
            (0..args.concurrency).map(|_| client.get_multiplexed_async_connection()),
        )
        .await
        .into_iter()
        .map(|r| r.expect("connect"))
        .collect();

        let start = Instant::now();
        let mut handles = Vec::new();
        for (w, mut con) in conns.into_iter().enumerate() {
            let val = val.to_string();
            let keyspace = args.keyspace;
            handles.push(tokio::spawn(async move {
                let mut lat = Vec::with_capacity(n_req);
                for r in 0..n_req {
                    let kbase = (w * n_req + r) * batch;
                    let keys: Vec<String> =
                        (0..batch).map(|j| key_for(kbase + j, keyspace)).collect();
                    let t = Instant::now();
                    if phase == "SET" {
                        let mut cmd = redis::cmd(if batch == 1 { "SET" } else { "MSET" });
                        for k in &keys {
                            cmd.arg(k).arg(&val);
                        }
                        let _: () = cmd.query_async(&mut con).await.unwrap();
                    } else if batch == 1 {
                        let _: Option<String> = redis::cmd("GET")
                            .arg(&keys[0])
                            .query_async(&mut con)
                            .await
                            .unwrap();
                    } else {
                        let mut cmd = redis::cmd("MGET");
                        for k in &keys {
                            cmd.arg(k);
                        }
                        let _: Vec<Option<String>> = cmd.query_async(&mut con).await.unwrap();
                    }
                    lat.push(t.elapsed().as_micros() as u64);
                }
                lat
            }));
        }
        let mut all = Vec::new();
        for h in handles {
            all.extend(h.await?);
        }
        summarize(phase, actual, batch, start.elapsed(), all);
    }
    Ok(())
}

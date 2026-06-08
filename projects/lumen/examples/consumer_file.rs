//! Example consumer: reads NDJSON documents from a file (or stdin) and
//! posts them to a running lumen instance through the shard-aware
//! [`lumen::client::Client`].
//!
//! ```bash
//! cargo run --example consumer_file -- \
//!   --base-url http://localhost:8080 \
//!   --collection users \
//!   --input data.ndjson
//! ```
//!
//! Each input line is a JSON object: `{"external_id": "...", "field":
//! "...", "value": ...}`. Items are batched up to lumen's
//! `MAX_INDEX_ITEMS` and flushed; failed batches retry once.
//!
//! This is a sample, not a production adapter — real upstreams plug
//! into AlloyDB CDC, Postgres logical replication, Kafka, etc., and
//! handle watermarks / dead-letter queues themselves.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

use lumen::client::Client;
use lumen::storage::MAX_INDEX_ITEMS;
use lumen::types::IndexItem;

#[derive(Parser, Debug)]
struct Args {
    /// Lumen base URL (or `{shard}` template if `--shard-count` is set).
    #[arg(long, default_value = "http://localhost:8080")]
    base_url: String,
    /// Optional bearer token for authenticated lumen.
    #[arg(long)]
    bearer: Option<String>,
    /// If set, enable shard-aware routing with this many shards.
    #[arg(long)]
    shard_count: Option<u32>,
    /// Target collection.
    #[arg(long)]
    collection: String,
    /// Input file. Reads stdin when omitted.
    #[arg(long)]
    input: Option<PathBuf>,
    /// Batch size (capped at `MAX_INDEX_ITEMS`).
    #[arg(long, default_value_t = 1_000)]
    batch: usize,
}

#[derive(Debug, Deserialize)]
struct WireItem {
    external_id: String,
    field: String,
    value: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let batch = args.batch.min(MAX_INDEX_ITEMS);

    let mut client = Client::new(args.base_url);
    if let Some(b) = args.bearer {
        client = client.with_bearer(b);
    }
    if let Some(n) = args.shard_count {
        client = client.with_shard_routing(n);
    }

    let reader: Box<dyn BufRead> = match args.input {
        Some(p) => Box::new(BufReader::new(
            std::fs::File::open(&p).with_context(|| format!("open {}", p.display()))?,
        )),
        None => Box::new(BufReader::new(std::io::stdin().lock())),
    };

    let mut buffer: Vec<IndexItem> = Vec::with_capacity(batch);
    let mut total = 0usize;

    for (lineno, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read line {}", lineno + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let wire: WireItem = serde_json::from_str(&line)
            .with_context(|| format!("parse line {}: {}", lineno + 1, line))?;
        buffer.push(IndexItem {
            external_id: wire.external_id,
            field: wire.field,
            value: serde_json::from_value(wire.value)
                .with_context(|| format!("decode value on line {}", lineno + 1))?,
        });
        if buffer.len() >= batch {
            total += flush(&client, &args.collection, std::mem::take(&mut buffer)).await?;
        }
    }
    if !buffer.is_empty() {
        total += flush(&client, &args.collection, buffer).await?;
    }
    eprintln!("indexed {total} items");
    Ok(())
}

async fn flush(client: &Client, collection: &str, items: Vec<IndexItem>) -> Result<usize> {
    let attempt = client.index(collection, items.clone(), None).await;
    match attempt {
        Ok(r) => Ok(r.indexed as usize),
        Err(e) => {
            eprintln!("batch failed ({e}); retrying once");
            let r = client.index(collection, items, None).await?;
            Ok(r.indexed as usize)
        }
    }
}

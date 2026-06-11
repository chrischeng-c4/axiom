// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-write-qps-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Write-path QPS bench. Default runs are report-only; `LUMEN_PERF_STRICT=1`
//! (or `LUMEN_WRITE_GATE=1` for this test only) turns the NATS-vs-peer
//! 100-worker index row into a strict competitive gate.
//!
//! This complements the read/search competitive gate. It measures the HTTP
//! write handlers that matter operationally:
//!   - `PUT /collections/{id}`: schema write through the coordinator.
//!   - `POST /collections/{id}/index`: document-field writes through the
//!     coordinator.
//!
//! The embedded leg uses the in-process WAL. The sharded leg splits one HTTP
//! request across multiple local coordinators. The NATS legs use real
//! `NatsWal -> WriteCoordinator -> local apply` paths: `nats` is the official
//! strict JetStream row, while `natssharded` uses one JetStream stream/subject per
//! write shard as an exploratory trend row. They skip gracefully when no
//! JetStream broker is reachable.
//!
//! Run:
//!   cargo test --release -p lumen --test write_qps -- --ignored --nocapture
//!   LUMEN_WRITE_MODES=embedded,sharded LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=0.3 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
//!   LUMEN_WRITE_MODES=nats,natssharded LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=1.0 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
//!   LUMEN_PERF_STRICT=1 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
//!
//! Strict mode defaults to `LUMEN_WRITE_MODES=nats,pg,os`. Explicitly include
//! `natssharded` only when collecting the partitioned JetStream trend row.

use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteCoordinator;
use lumen::log_entry::RaftLogEntry;
use lumen::routing::EngineShardWrite;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::WalRecord;
use lumen::wal::{MemWal, SharedWal};
use lumen::wal_nats::{NatsWal, NatsWalConfig};

const DEFAULT_WARMUP_S: f64 = 2.0;
const PUT_WORKERS: &[usize] = &[1, 10];
const INDEX_WORKERS: &[usize] = &[1, 10, 100];
const DEFAULT_WINDOW_S: f64 = 5.0;
const DEFAULT_BATCH_DOCS: usize = 100;
const DEFAULT_REQ_TIMEOUT_MS: u64 = 2_000;
const PG_DSN: &str = "host=/tmp dbname=lumenbench";
const OS_URL: &str = "http://localhost:9200";
const PG_MAX_POOL: usize = 90;

fn nats_url() -> String {
    std::env::var("LUMEN_TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into())
}

fn window_s() -> f64 {
    std::env::var("LUMEN_WRITE_WINDOW_S")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WINDOW_S)
}

fn warmup_s() -> f64 {
    std::env::var("LUMEN_WRITE_WARMUP_S")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WARMUP_S)
}

fn batch_docs() -> usize {
    std::env::var("LUMEN_WRITE_BATCH_DOCS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_BATCH_DOCS)
}

fn write_mode_enabled(name: &str) -> bool {
    let Some(raw) = std::env::var("LUMEN_WRITE_MODES").ok() else {
        if write_gate_enabled() {
            return matches!(name, "nats" | "pg" | "os");
        }
        return true;
    };
    let mut saw_any = false;
    for mode in raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        saw_any = true;
        let mode = mode.to_ascii_lowercase();
        let normalized = match mode.as_str() {
            "nats-sharded" | "nats_sharded" | "natsshard" => "natssharded",
            other => other,
        };
        match normalized {
            "embedded" | "sharded" | "nats" | "natssharded" | "pg" | "os" | "opensearch" => {}
            _ => panic!("unknown LUMEN_WRITE_MODES entry `{mode}`"),
        }
        if normalized == name || (name == "os" && normalized == "opensearch") {
            return true;
        }
    }
    !saw_any
}

fn write_modes_label() -> String {
    std::env::var("LUMEN_WRITE_MODES").unwrap_or_else(|_| {
        if write_gate_enabled() {
            "nats,pg,os".into()
        } else {
            "embedded,sharded,nats,natssharded,pg,os".into()
        }
    })
}

fn write_shards() -> usize {
    std::env::var("LUMEN_WRITE_SHARDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
}

fn req_timeout() -> Duration {
    Duration::from_millis(
        std::env::var("LUMEN_WRITE_REQ_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_REQ_TIMEOUT_MS),
    )
}

fn docs_schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        "bio".into(),
        FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "city".into(),
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    fields.insert(
        "age".into(),
        FieldSpec {
            field_type: FieldType::Number,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields }
}

async fn reset_nats_stream_config(config: &NatsWalConfig) -> Option<()> {
    let client = async_nats::connect(&nats_url()).await.ok()?;
    let js = async_nats::jetstream::new(client);
    let _ = js.delete_stream(config.stream_name.clone()).await;
    js.get_or_create_stream(async_nats::jetstream::stream::Config {
        name: config.stream_name.clone(),
        subjects: vec![config.subject.clone()],
        ..Default::default()
    })
    .await
    .ok()?;
    Some(())
}

async fn reset_nats_stream() -> Option<()> {
    reset_nats_stream_config(&NatsWalConfig::default()).await
}

struct Server {
    client: reqwest::Client,
    base: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(state: AppState) -> Server {
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind lumen write bench server");
    let addr = listener.local_addr().expect("local addr");
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    for _ in 0..50 {
        if client.get(format!("{base}/healthz")).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Server { client, base, task }
}

async fn serve_embedded() -> Server {
    let engine = Arc::new(Engine::new());
    serve(AppState::open(engine)).await
}

async fn serve_sharded_embedded() -> Server {
    let shards = write_shards().max(1);
    let writers = (0..shards)
        .map(|_| {
            let engine = Arc::new(Engine::new());
            WriteCoordinator::start(Arc::new(MemWal::new()), engine)
        })
        .collect();
    let state = AppState::open(Arc::new(Engine::new()))
        .with_write_backend(Arc::new(EngineShardWrite::new(writers)));
    serve(state).await
}

async fn serve_nats() -> Option<Server> {
    reset_nats_stream().await?;
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(NatsWal::connect(&nats_url()).await.ok()?);
    let state = AppState::with_wal(engine, Arc::new(AuthConfig::open()), wal);
    Some(serve(state).await)
}

async fn serve_sharded_nats() -> Option<Server> {
    let shards = write_shards().max(1);
    for shard in 0..shards {
        reset_nats_stream_config(&NatsWalConfig::shard(shard)).await?;
    }
    let mut writers = Vec::with_capacity(shards);
    for shard in 0..shards {
        let engine = Arc::new(Engine::new());
        let wal: SharedWal = Arc::new(
            NatsWal::connect_with_config(&nats_url(), NatsWalConfig::shard(shard))
                .await
                .ok()?,
        );
        writers.push(WriteCoordinator::start(wal, engine));
    }
    let state = AppState::open(Arc::new(Engine::new()))
        .with_write_backend(Arc::new(EngineShardWrite::new(writers)));
    Some(serve(state).await)
}

#[derive(Clone, Copy)]
enum Mode {
    Embedded,
    Sharded,
    Nats,
    NatsSharded,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Sharded => "sharded",
            Self::Nats => "nats",
            Self::NatsSharded => "natssharded",
        }
    }

    async fn serve(self) -> Option<Server> {
        match self {
            Self::Embedded => Some(serve_embedded().await),
            Self::Sharded => Some(serve_sharded_embedded().await),
            Self::Nats => serve_nats().await,
            Self::NatsSharded => serve_sharded_nats().await,
        }
    }
}

async fn create_docs_collection(server: &Server, coll: &str) {
    server
        .client
        .put(format!("{}/collections/{coll}", server.base))
        .json(&docs_schema())
        .send()
        .await
        .expect("put collection")
        .error_for_status()
        .expect("put collection status");
}

fn index_request(start_doc: u64, docs: usize) -> IndexRequest {
    let mut items = Vec::with_capacity(docs * 3);
    for i in 0..docs {
        let n = start_doc + i as u64;
        let city = match n % 5 {
            0 => "taipei",
            1 => "tokyo",
            2 => "seoul",
            3 => "singapore",
            _ => "london",
        };
        items.push(IndexItem {
            external_id: format!("d{n}"),
            field: "bio".into(),
            value: FieldValue::String(format!("rust backend engineer event stream {n}")),
        });
        items.push(IndexItem {
            external_id: format!("d{n}"),
            field: "city".into(),
            value: FieldValue::String(city.into()),
        });
        items.push(IndexItem {
            external_id: format!("d{n}"),
            field: "age".into(),
            value: FieldValue::Number(18.0 + (n % 63) as f64),
        });
    }
    IndexRequest {
        items,
        request_id: None,
    }
}

fn doc_fields(n: u64) -> (String, &'static str, i32) {
    let city = match n % 5 {
        0 => "taipei",
        1 => "tokyo",
        2 => "seoul",
        3 => "singapore",
        _ => "london",
    };
    (
        format!("rust backend engineer event stream {n}"),
        city,
        18 + (n % 63) as i32,
    )
}

fn sql_lit(s: &str) -> String {
    s.replace('\'', "''")
}

fn pg_insert_sql(table: &str, start_doc: u64, docs: usize) -> String {
    let mut sql = format!("INSERT INTO {table} (eid,bio,city,age) VALUES ");
    for i in 0..docs {
        if i > 0 {
            sql.push(',');
        }
        let n = start_doc + i as u64;
        let (bio, city, age) = doc_fields(n);
        sql.push_str(&format!(
            "('d{n}','{}','{}',{age})",
            sql_lit(&bio),
            sql_lit(city)
        ));
    }
    sql
}

fn os_bulk_body(start_doc: u64, docs: usize) -> String {
    let mut body = String::new();
    for i in 0..docs {
        let n = start_doc + i as u64;
        let (bio, city, age) = doc_fields(n);
        body.push_str(&format!("{{\"index\":{{\"_id\":\"d{n}\"}}}}\n"));
        body.push_str(
            &serde_json::to_string(&json!({
                "bio": bio,
                "city": city,
                "age": age,
            }))
            .expect("serialize os bulk doc"),
        );
        body.push('\n');
    }
    body
}

#[derive(Clone)]
enum WriteReq {
    PutCollection {
        client: reqwest::Client,
        base: String,
        timeout: Duration,
    },
    IndexBatch {
        client: reqwest::Client,
        base: String,
        coll: String,
        seq: Arc<AtomicU64>,
        batch_docs: usize,
        timeout: Duration,
    },
}

impl WriteReq {
    fn docs_per_request(&self) -> usize {
        match self {
            Self::PutCollection { .. } => 0,
            Self::IndexBatch { batch_docs, .. } => *batch_docs,
        }
    }

    fn items_per_request(&self) -> usize {
        self.docs_per_request() * 3
    }
}

#[derive(Clone)]
enum PeerWriteReq {
    PgInsert {
        clients: Arc<Vec<tokio_postgres::Client>>,
        sem: Arc<Semaphore>,
        table: String,
        seq: Arc<AtomicU64>,
        batch_docs: usize,
        timeout: Duration,
    },
    OsBulk {
        client: reqwest::Client,
        base: String,
        index: String,
        seq: Arc<AtomicU64>,
        batch_docs: usize,
        timeout: Duration,
    },
}

impl PeerWriteReq {
    fn docs_per_request(&self) -> usize {
        match self {
            Self::PgInsert { batch_docs, .. } | Self::OsBulk { batch_docs, .. } => *batch_docs,
        }
    }

    fn items_per_request(&self) -> usize {
        self.docs_per_request() * 3
    }
}

async fn pg_connect() -> Option<tokio_postgres::Client> {
    let (client, connection) = match tokio_postgres::connect(PG_DSN, tokio_postgres::NoTls).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("skipping pg write QPS: postgres unavailable ({e})");
            return None;
        }
    };
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Some(client)
}

async fn pg_prepare_table(table: &str) -> Option<()> {
    let client = pg_connect().await?;
    client
        .batch_execute(&format!("DROP TABLE IF EXISTS {table}"))
        .await
        .ok()?;
    client
        .batch_execute(&format!(
            "CREATE TABLE {table} (
                eid text PRIMARY KEY,
                bio text,
                bio_tsv tsvector GENERATED ALWAYS AS (to_tsvector('simple', bio)) STORED,
                city text,
                age int
            );
             CREATE INDEX {table}_bio_gin ON {table} USING gin (bio_tsv);
             CREATE INDEX {table}_city ON {table} (city);
             CREATE INDEX {table}_age ON {table} (age)"
        ))
        .await
        .ok()?;
    Some(())
}

async fn pg_insert_pool(workers: usize) -> Option<Arc<Vec<tokio_postgres::Client>>> {
    let n = workers.max(1).min(PG_MAX_POOL);
    let mut clients = Vec::with_capacity(n);
    for _ in 0..n {
        clients.push(pg_connect().await?);
    }
    Some(Arc::new(clients))
}

async fn os_prepare_index(client: &reqwest::Client, index: &str) -> Option<String> {
    let base = OS_URL.to_string();
    if client.get(&base).send().await.is_err() {
        eprintln!("skipping OpenSearch write QPS: unavailable on {base}");
        return None;
    }
    let _ = client.delete(format!("{base}/{index}")).send().await;
    client
        .put(format!("{base}/{index}"))
        .json(&json!({
            "settings": {
                "number_of_shards": 1,
                "number_of_replicas": 0,
                "refresh_interval": "-1"
            },
            "mappings": {
                "properties": {
                    "bio": { "type": "text" },
                    "city": { "type": "keyword" },
                    "age": { "type": "integer" }
                }
            }
        }))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    Some(base)
}

#[derive(Clone, Copy, Debug, Default)]
struct ErrorCounts {
    timeout: u64,
    status: u64,
    transport: u64,
}

impl ErrorCounts {
    fn add(&mut self, kind: WriteErrorKind) {
        match kind {
            WriteErrorKind::Timeout => self.timeout += 1,
            WriteErrorKind::Status => self.status += 1,
            WriteErrorKind::Transport => self.transport += 1,
        }
    }

    fn total(self) -> u64 {
        self.timeout + self.status + self.transport
    }
}

#[derive(Clone, Copy, Debug)]
enum WriteErrorKind {
    Timeout,
    Status,
    Transport,
}

fn classify_reqwest(err: &reqwest::Error) -> WriteErrorKind {
    if err.is_status() {
        WriteErrorKind::Status
    } else {
        WriteErrorKind::Transport
    }
}

async fn issue(req: &WriteReq) -> Result<(), WriteErrorKind> {
    match req {
        WriteReq::PutCollection {
            client,
            base,
            timeout,
        } => tokio::time::timeout(*timeout, async {
            client
                .put(format!("{base}/collections/qps_put"))
                .json(&docs_schema())
                .send()
                .await?
                .error_for_status()?;
            Ok::<(), reqwest::Error>(())
        })
        .await
        .map_err(|_| WriteErrorKind::Timeout)
        .and_then(|r| r.map_err(|e| classify_reqwest(&e))),
        WriteReq::IndexBatch {
            client,
            base,
            coll,
            seq,
            batch_docs,
            timeout,
        } => {
            let start = seq.fetch_add(*batch_docs as u64, Ordering::Relaxed);
            tokio::time::timeout(*timeout, async {
                client
                    .post(format!("{base}/collections/{coll}/index"))
                    .json(&index_request(start, *batch_docs))
                    .send()
                    .await?
                    .error_for_status()?;
                Ok::<(), reqwest::Error>(())
            })
            .await
            .map_err(|_| WriteErrorKind::Timeout)
            .and_then(|r| r.map_err(|e| classify_reqwest(&e)))
        }
    }
}

async fn issue_peer(req: &PeerWriteReq) -> Result<(), WriteErrorKind> {
    match req {
        PeerWriteReq::PgInsert {
            clients,
            sem,
            table,
            seq,
            batch_docs,
            timeout,
        } => {
            let start = seq.fetch_add(*batch_docs as u64, Ordering::Relaxed);
            let sql = pg_insert_sql(table, start, *batch_docs);
            tokio::time::timeout(*timeout, async {
                let _permit = sem.acquire().await.expect("pg semaphore closed");
                let idx = ((start / *batch_docs as u64) as usize) % clients.len();
                clients[idx]
                    .batch_execute(&sql)
                    .await
                    .map_err(|_| WriteErrorKind::Transport)
            })
            .await
            .map_err(|_| WriteErrorKind::Timeout)?
        }
        PeerWriteReq::OsBulk {
            client,
            base,
            index,
            seq,
            batch_docs,
            timeout,
        } => {
            let start = seq.fetch_add(*batch_docs as u64, Ordering::Relaxed);
            let body = os_bulk_body(start, *batch_docs);
            tokio::time::timeout(*timeout, async {
                client
                    .post(format!("{base}/{index}/_bulk"))
                    .header("content-type", "application/x-ndjson")
                    .body(body)
                    .send()
                    .await?
                    .error_for_status()?;
                Ok::<(), reqwest::Error>(())
            })
            .await
            .map_err(|_| WriteErrorKind::Timeout)
            .and_then(|r| r.map_err(|e| classify_reqwest(&e)))
        }
    }
}

async fn prime_write_path(req: &WriteReq) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if issue(req).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("write path did not accept a warmup request before benchmark window");
}

async fn prime_peer_write_path(req: &PeerWriteReq) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if issue_peer(req).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("peer write path did not accept a warmup request before benchmark window");
}

#[derive(Clone, Copy, Default)]
struct Load {
    achieved_qps: f64,
    docs_per_s: f64,
    items_per_s: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    errors: ErrorCounts,
}

async fn run_write_load(req: WriteReq, workers: usize) -> Load {
    let warmup_end = Instant::now() + Duration::from_secs_f64(warmup_s());
    let mut warmup: JoinSet<()> = JoinSet::new();
    for _ in 0..workers.max(1) {
        let req = req.clone();
        warmup.spawn(async move {
            while Instant::now() < warmup_end {
                let _ = issue(&req).await;
            }
        });
    }
    while warmup.join_next().await.is_some() {}

    let window = Duration::from_secs_f64(window_s());
    let window_end = Instant::now() + window;
    let docs_per_req = req.docs_per_request() as f64;
    let items_per_req = req.items_per_request() as f64;
    let mut set: JoinSet<(Vec<f64>, ErrorCounts)> = JoinSet::new();

    for _ in 0..workers.max(1) {
        let req = req.clone();
        set.spawn(async move {
            let mut samples = Vec::new();
            let mut errors = ErrorCounts::default();
            loop {
                let started = Instant::now();
                if started >= window_end {
                    break;
                }
                let result = issue(&req).await;
                let finished = Instant::now();
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                if finished <= window_end {
                    if result.is_ok() {
                        samples.push(elapsed_ms);
                    } else if let Err(kind) = result {
                        errors.add(kind);
                    }
                } else if let Err(kind) = result {
                    errors.add(kind);
                }
                if finished >= window_end {
                    break;
                }
            }
            (samples, errors)
        });
    }

    let mut all = Vec::new();
    let mut errors = ErrorCounts::default();
    while let Some(r) = set.join_next().await {
        if let Ok((samples, worker_errors)) = r {
            all.extend(samples);
            errors.timeout += worker_errors.timeout;
            errors.status += worker_errors.status;
            errors.transport += worker_errors.transport;
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |q: f64| {
        if all.is_empty() {
            0.0
        } else {
            all[(((all.len() - 1) as f64) * q).round() as usize]
        }
    };
    let achieved_qps = all.len() as f64 / window.as_secs_f64();
    Load {
        achieved_qps,
        docs_per_s: achieved_qps * docs_per_req,
        items_per_s: achieved_qps * items_per_req,
        p50_ms: pct(0.50),
        p95_ms: pct(0.95),
        p99_ms: pct(0.99),
        errors,
    }
}

async fn run_peer_write_load(req: PeerWriteReq, workers: usize) -> Load {
    let warmup_end = Instant::now() + Duration::from_secs_f64(warmup_s());
    let mut warmup: JoinSet<()> = JoinSet::new();
    for _ in 0..workers.max(1) {
        let req = req.clone();
        warmup.spawn(async move {
            while Instant::now() < warmup_end {
                let _ = issue_peer(&req).await;
            }
        });
    }
    while warmup.join_next().await.is_some() {}

    let window = Duration::from_secs_f64(window_s());
    let window_end = Instant::now() + window;
    let docs_per_req = req.docs_per_request() as f64;
    let items_per_req = req.items_per_request() as f64;
    let mut set: JoinSet<(Vec<f64>, ErrorCounts)> = JoinSet::new();

    for _ in 0..workers.max(1) {
        let req = req.clone();
        set.spawn(async move {
            let mut samples = Vec::new();
            let mut errors = ErrorCounts::default();
            loop {
                let started = Instant::now();
                if started >= window_end {
                    break;
                }
                let result = issue_peer(&req).await;
                let finished = Instant::now();
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                if finished <= window_end {
                    if result.is_ok() {
                        samples.push(elapsed_ms);
                    } else if let Err(kind) = result {
                        errors.add(kind);
                    }
                } else if let Err(kind) = result {
                    errors.add(kind);
                }
                if finished >= window_end {
                    break;
                }
            }
            (samples, errors)
        });
    }

    let mut all = Vec::new();
    let mut errors = ErrorCounts::default();
    while let Some(r) = set.join_next().await {
        if let Ok((samples, worker_errors)) = r {
            all.extend(samples);
            errors.timeout += worker_errors.timeout;
            errors.status += worker_errors.status;
            errors.transport += worker_errors.transport;
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |q: f64| {
        if all.is_empty() {
            0.0
        } else {
            all[(((all.len() - 1) as f64) * q).round() as usize]
        }
    };
    let achieved_qps = all.len() as f64 / window.as_secs_f64();
    Load {
        achieved_qps,
        docs_per_s: achieved_qps * docs_per_req,
        items_per_s: achieved_qps * items_per_req,
        p50_ms: pct(0.50),
        p95_ms: pct(0.95),
        p99_ms: pct(0.99),
        errors,
    }
}

async fn run_engine_index_load(engine: Arc<Engine>, coll: String, workers: usize) -> Load {
    let batch_docs = batch_docs();
    let seq = Arc::new(AtomicU64::new(0));
    let warmup_end = Instant::now() + Duration::from_secs_f64(warmup_s());
    let mut warmup: JoinSet<()> = JoinSet::new();
    for _ in 0..workers.max(1) {
        let engine = engine.clone();
        let coll = coll.clone();
        let seq = seq.clone();
        warmup.spawn_blocking(move || {
            while Instant::now() < warmup_end {
                let start = seq.fetch_add(batch_docs as u64, Ordering::Relaxed);
                let _ = engine.index(&coll, index_request(start, batch_docs));
            }
        });
    }
    while warmup.join_next().await.is_some() {}

    let window = Duration::from_secs_f64(window_s());
    let window_end = Instant::now() + window;
    let mut set: JoinSet<(Vec<f64>, ErrorCounts)> = JoinSet::new();
    for _ in 0..workers.max(1) {
        let engine = engine.clone();
        let coll = coll.clone();
        let seq = seq.clone();
        set.spawn_blocking(move || {
            let mut samples = Vec::new();
            let mut errors = ErrorCounts::default();
            loop {
                let started = Instant::now();
                if started >= window_end {
                    break;
                }
                let start = seq.fetch_add(batch_docs as u64, Ordering::Relaxed);
                let result = engine.index(&coll, index_request(start, batch_docs));
                let finished = Instant::now();
                let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                if finished <= window_end {
                    if result.is_ok() {
                        samples.push(elapsed_ms);
                    } else {
                        errors.transport += 1;
                    }
                } else if result.is_err() {
                    errors.transport += 1;
                }
                if finished >= window_end {
                    break;
                }
            }
            (samples, errors)
        });
    }

    let mut all = Vec::new();
    let mut errors = ErrorCounts::default();
    while let Some(r) = set.join_next().await {
        if let Ok((samples, worker_errors)) = r {
            all.extend(samples);
            errors.timeout += worker_errors.timeout;
            errors.status += worker_errors.status;
            errors.transport += worker_errors.transport;
        }
    }
    all.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |q: f64| {
        if all.is_empty() {
            0.0
        } else {
            all[(((all.len() - 1) as f64) * q).round() as usize]
        }
    };
    let achieved_qps = all.len() as f64 / window.as_secs_f64();
    Load {
        achieved_qps,
        docs_per_s: achieved_qps * batch_docs as f64,
        items_per_s: achieved_qps * (batch_docs * 3) as f64,
        p50_ms: pct(0.50),
        p95_ms: pct(0.95),
        p99_ms: pct(0.99),
        errors,
    }
}

fn print_row(mode: &str, op: &str, workers: usize, batch_docs: usize, load: &Load) {
    println!(
        "{mode:8} {op:6} workers={workers:<3} batch_docs={batch_docs:<4} req/s={:>9.1} docs/s={:>10.0} items/s={:>10.0} p50={:>7.3}ms p95={:>7.3}ms p99={:>7.3}ms errors={} timeout={} status={} transport={}",
        load.achieved_qps,
        load.docs_per_s,
        load.items_per_s,
        load.p50_ms,
        load.p95_ms,
        load.p99_ms,
        load.errors.total(),
        load.errors.timeout,
        load.errors.status,
        load.errors.transport
    );
}

async fn run_http_write_mode(mode: Mode) -> Option<BTreeMap<usize, Load>> {
    let batch_docs = batch_docs();
    let timeout = req_timeout();
    let label = mode.label();
    let Some(server) = mode.serve().await else {
        return None;
    };
    println!(
        "\n# {label} write QPS (window={}s warmup={}s timeout={}ms)",
        window_s(),
        warmup_s(),
        timeout.as_millis()
    );

    for &workers in PUT_WORKERS {
        let req = WriteReq::PutCollection {
            client: server.client.clone(),
            base: server.base.clone(),
            timeout,
        };
        prime_write_path(&req).await;
        let load = run_write_load(req, workers).await;
        print_row(label, "put", workers, 0, &load);
    }

    let mut index_loads = BTreeMap::new();
    for &workers in INDEX_WORKERS {
        let coll = format!("docs_w{workers}");
        create_docs_collection(&server, &coll).await;
        let req = WriteReq::IndexBatch {
            client: server.client.clone(),
            base: server.base.clone(),
            coll,
            seq: Arc::new(AtomicU64::new(0)),
            batch_docs,
            timeout,
        };
        prime_write_path(&req).await;
        let load = run_write_load(req, workers).await;
        print_row(label, "index", workers, batch_docs, &load);
        index_loads.insert(workers, load);
    }
    Some(index_loads)
}

async fn run_pg_write_mode() -> Option<BTreeMap<usize, Load>> {
    let batch_docs = batch_docs();
    let timeout = req_timeout();
    if pg_connect().await.is_none() {
        return None;
    }
    println!(
        "\n# pg write QPS (window={}s warmup={}s timeout={}ms)",
        window_s(),
        warmup_s(),
        timeout.as_millis()
    );
    let mut index_loads = BTreeMap::new();
    for &workers in INDEX_WORKERS {
        let table = format!("docs_pg_w{workers}");
        if pg_prepare_table(&table).await.is_none() {
            eprintln!("skipping pg index workers={workers}: setup failed");
            return None;
        }
        let Some(clients) = pg_insert_pool(workers).await else {
            eprintln!("skipping pg index workers={workers}: pool setup failed");
            return None;
        };
        let req = PeerWriteReq::PgInsert {
            clients: clients.clone(),
            sem: Arc::new(Semaphore::new(clients.len())),
            table,
            seq: Arc::new(AtomicU64::new(0)),
            batch_docs,
            timeout,
        };
        prime_peer_write_path(&req).await;
        let load = run_peer_write_load(req, workers).await;
        print_row("pg", "index", workers, batch_docs, &load);
        index_loads.insert(workers, load);
    }
    Some(index_loads)
}

async fn run_os_write_mode() -> Option<BTreeMap<usize, Load>> {
    let batch_docs = batch_docs();
    let timeout = req_timeout();
    let client = reqwest::Client::new();
    if client.get(OS_URL).send().await.is_err() {
        eprintln!("skipping OpenSearch write QPS: unavailable on {OS_URL}");
        return None;
    }
    println!(
        "\n# opensearch write QPS (window={}s warmup={}s timeout={}ms)",
        window_s(),
        warmup_s(),
        timeout.as_millis()
    );
    let mut index_loads = BTreeMap::new();
    for &workers in INDEX_WORKERS {
        let index = format!("docs-os-w{workers}");
        let Some(base) = os_prepare_index(&client, &index).await else {
            eprintln!("skipping OpenSearch index workers={workers}: setup failed");
            return None;
        };
        let req = PeerWriteReq::OsBulk {
            client: client.clone(),
            base,
            index,
            seq: Arc::new(AtomicU64::new(0)),
            batch_docs,
            timeout,
        };
        prime_peer_write_path(&req).await;
        let load = run_peer_write_load(req, workers).await;
        print_row("os", "index", workers, batch_docs, &load);
        index_loads.insert(workers, load);
    }
    Some(index_loads)
}

fn write_gate_enabled() -> bool {
    env_flag_enabled("LUMEN_WRITE_GATE") || env_flag_enabled("LUMEN_PERF_STRICT")
}

fn env_flag_enabled(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}

fn write_gate_threshold(cell: &str, peer: &str) -> Option<f64> {
    let baseline: Value = serde_json::from_str(include_str!("perf-baseline.json"))
        .expect("perf-baseline.json parses");
    let ratchet = baseline["ratchet"].as_f64().unwrap_or(0.8);
    let gate = &baseline["write_cells"][cell][peer];
    if gate["gate"].as_str() != Some("win") {
        return None;
    }
    let baseline_ratio = gate["baseline"].as_f64().unwrap_or(1.0);
    Some((baseline_ratio * ratchet).max(1.0))
}

fn gate_load<'a>(
    label: &str,
    loads: Option<&'a BTreeMap<usize, Load>>,
    strict: bool,
    failures: &mut Vec<String>,
) -> Option<&'a Load> {
    let Some(loads) = loads else {
        let msg = format!("{label} write row missing; peer service unavailable or skipped");
        if strict {
            failures.push(msg);
        } else {
            eprintln!("skipping write gate for {label}: peer service unavailable or skipped");
        }
        return None;
    };
    let Some(load) = loads.get(&100) else {
        let msg = format!("{label} write workers=100 row missing");
        if strict {
            failures.push(msg);
        } else {
            eprintln!("skipping write gate for {label}: workers=100 row missing");
        }
        return None;
    };
    Some(load)
}

fn judge_write_peer(
    cell: &str,
    lumen_label: &str,
    peer_key: &str,
    peer_label: &str,
    lumen100: &Load,
    peer100: &Load,
    strict: bool,
    failures: &mut Vec<String>,
) {
    if lumen100.errors.total() > 0 || peer100.errors.total() > 0 || peer100.docs_per_s <= 0.0 {
        let msg = format!(
            "write gate {cell} vs {peer_label}: unusable row ({lumen_label}_errors={} peer_errors={} peer_docs_s={:.0})",
            lumen100.errors.total(),
            peer100.errors.total(),
            peer100.docs_per_s
        );
        if strict {
            failures.push(msg);
        } else {
            eprintln!("skipping {msg}");
        }
        return;
    }
    let ratio = lumen100.docs_per_s / peer100.docs_per_s;
    let threshold = write_gate_threshold(cell, peer_key).unwrap_or(1.0);
    let verdict = if ratio >= threshold { "WIN" } else { "RED" };
    println!(
        "write_gate {cell} vs {peer_label}: ratio={ratio:.2} threshold={threshold:.2} verdict={verdict}"
    );
    if ratio < threshold {
        let msg = format!(
            "write gate {cell} vs {peer_label}: ratio {ratio:.2} below threshold {threshold:.2}"
        );
        if strict {
            failures.push(msg);
        } else {
            eprintln!("WRITE TARGET: {msg}");
        }
    }
}

fn check_write_gate_cell(
    cell: &str,
    lumen_label: &str,
    lumen: Option<&BTreeMap<usize, Load>>,
    pg: Option<&BTreeMap<usize, Load>>,
    os: Option<&BTreeMap<usize, Load>>,
    strict: bool,
    failures: &mut Vec<String>,
) {
    let Some(lumen100) = gate_load(lumen_label, lumen, strict, failures) else {
        return;
    };
    let pg100 = gate_load("pg", pg, strict, failures);
    let os100 = gate_load("opensearch", os, strict, failures);
    if let Some(pg100) = pg100 {
        judge_write_peer(
            cell,
            lumen_label,
            "pg",
            "pg",
            lumen100,
            pg100,
            strict,
            failures,
        );
    }
    if let Some(os100) = os100 {
        judge_write_peer(
            cell,
            lumen_label,
            "os",
            "opensearch",
            lumen100,
            os100,
            strict,
            failures,
        );
    }
}

fn check_write_gate(
    nats: Option<&BTreeMap<usize, Load>>,
    natssharded: Option<&BTreeMap<usize, Load>>,
    pg: Option<&BTreeMap<usize, Load>>,
    os: Option<&BTreeMap<usize, Load>>,
) {
    let strict = write_gate_enabled();
    let mut failures = Vec::new();
    if strict || nats.is_some() || write_mode_enabled("nats") {
        check_write_gate_cell(
            "nats_index_100",
            "nats",
            nats,
            pg,
            os,
            strict,
            &mut failures,
        );
    }
    // `nats` is the official JetStream write gate. `natssharded` is an
    // exploratory partitioned-stream trend row: report it, but do not let it
    // block the official gate until it is stable under the same timeout/error
    // envelope.
    if natssharded.is_some() || write_mode_enabled("natssharded") {
        check_write_gate_cell(
            "natssharded_index_100",
            "natssharded",
            natssharded,
            pg,
            os,
            false,
            &mut failures,
        );
    }
    if strict && !failures.is_empty() {
        panic!(
            "write competitive gate: {} WIN-cell regression(s): {}",
            failures.len(),
            failures.join("; ")
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "write-path QPS bench; strict gate requires LUMEN_PERF_STRICT=1 and JetStream"]
async fn write_qps_bench() {
    println!(
        "# lumen write-path QPS bench; modes={} batch_docs={} sharded_write_shards={} env: LUMEN_WRITE_MODES LUMEN_WRITE_WARMUP_S LUMEN_WRITE_WINDOW_S LUMEN_WRITE_BATCH_DOCS LUMEN_WRITE_SHARDS LUMEN_TEST_NATS_URL",
        write_modes_label(),
        batch_docs(),
        write_shards()
    );
    let _embedded = if write_mode_enabled("embedded") {
        run_http_write_mode(Mode::Embedded).await
    } else {
        None
    };
    let _sharded = if write_mode_enabled("sharded") {
        run_http_write_mode(Mode::Sharded).await
    } else {
        None
    };

    let nats = if write_mode_enabled("nats") {
        run_http_write_mode(Mode::Nats).await
    } else {
        None
    };
    if write_mode_enabled("nats") && nats.is_none() {
        eprintln!(
            "skipping nats write QPS: no JetStream NATS at {}",
            nats_url()
        );
    }
    let natssharded = if write_mode_enabled("natssharded") {
        let result = run_http_write_mode(Mode::NatsSharded).await;
        if result.is_none() {
            eprintln!(
                "skipping natssharded write QPS: no JetStream NATS at {}",
                nats_url()
            );
        }
        result
    } else {
        None
    };

    let pg = if write_mode_enabled("pg") {
        run_pg_write_mode().await
    } else {
        None
    };
    let os = if write_mode_enabled("os") {
        run_os_write_mode().await
    } else {
        None
    };
    if write_gate_enabled() || write_mode_enabled("nats") || write_mode_enabled("natssharded") {
        check_write_gate(
            nats.as_ref(),
            natssharded.as_ref(),
            pg.as_ref(),
            os.as_ref(),
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "report-only direct Engine::index QPS probe; isolates engine apply from HTTP/WAL"]
async fn engine_index_qps_probe() {
    let batch_docs = batch_docs();
    let engine = Arc::new(Engine::new());
    println!(
        "# direct engine index QPS probe; batch_docs={} window={}s warmup={}s",
        batch_docs,
        window_s(),
        warmup_s()
    );
    for &workers in INDEX_WORKERS {
        let coll = format!("engine_probe_w{workers}");
        engine
            .create_collection(&coll, docs_schema())
            .expect("create engine probe collection");
        let load = run_engine_index_load(engine.clone(), coll, workers).await;
        print_row("engine", "index", workers, batch_docs, &load);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "direct NATS WriteCoordinator latency probe; requires JetStream"]
async fn nats_submit_latency_probe() {
    let Some(()) = reset_nats_stream().await else {
        eprintln!(
            "skipping nats submit probe: no JetStream NATS at {}",
            nats_url()
        );
        return;
    };
    let engine = Arc::new(Engine::new());
    let wal: SharedWal = Arc::new(NatsWal::connect(&nats_url()).await.expect("connect nats"));
    let coord = WriteCoordinator::start(wal, engine);
    let timeout = req_timeout();

    let mut put_lat = Vec::new();
    let mut put_timeouts = 0u64;
    for i in 0..500u64 {
        let started = Instant::now();
        let result = tokio::time::timeout(
            timeout,
            coord.submit(RaftLogEntry::CreateCollection {
                collection_id: "probe_put".into(),
                req: docs_schema(),
            }),
        )
        .await;
        match result {
            Ok(Ok(_)) => put_lat.push(started.elapsed().as_secs_f64() * 1000.0),
            Ok(Err(e)) => panic!("put submit failed at {i}: {e}"),
            Err(_) => {
                put_timeouts += 1;
                eprintln!("put submit timeout at i={i}");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }

    coord
        .submit(RaftLogEntry::CreateCollection {
            collection_id: "probe_docs".into(),
            req: docs_schema(),
        })
        .await
        .expect("create probe docs");
    let seq = Arc::new(AtomicU64::new(0));
    let mut index_lat = Vec::new();
    let mut index_timeouts = 0u64;
    for i in 0..500u64 {
        let start_doc = seq.fetch_add(batch_docs() as u64, Ordering::Relaxed);
        let started = Instant::now();
        let result = tokio::time::timeout(
            timeout,
            coord.submit(RaftLogEntry::Index {
                collection_id: "probe_docs".into(),
                req: index_request(start_doc, batch_docs()),
            }),
        )
        .await;
        match result {
            Ok(Ok(_)) => index_lat.push(started.elapsed().as_secs_f64() * 1000.0),
            Ok(Err(e)) => panic!("index submit failed at {i}: {e}"),
            Err(_) => {
                index_timeouts += 1;
                eprintln!("index submit timeout at i={i}");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }

    put_lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    index_lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pct = |v: &[f64], q: f64| {
        if v.is_empty() {
            0.0
        } else {
            v[(((v.len() - 1) as f64) * q).round() as usize]
        }
    };
    println!(
        "direct nats submit: put ok={} timeout={} p50={:.3}ms p99={:.3}ms; index ok={} timeout={} p50={:.3}ms p99={:.3}ms",
        put_lat.len(),
        put_timeouts,
        pct(&put_lat, 0.50),
        pct(&put_lat, 0.99),
        index_lat.len(),
        index_timeouts,
        pct(&index_lat, 0.50),
        pct(&index_lat, 0.99),
    );
}

#[test]
#[ignore = "WAL codec micro-probe; report-only"]
fn wal_codec_probe() {
    let rec = WalRecord::new(RaftLogEntry::Index {
        collection_id: "probe_docs".into(),
        req: index_request(0, batch_docs()),
    });
    let iters = 5_000usize;

    let started = Instant::now();
    let mut json_len = 0usize;
    let mut json_payload = Vec::new();
    for _ in 0..iters {
        json_payload = serde_json::to_vec(&rec).expect("json encode wal");
        json_len += std::hint::black_box(json_payload.len());
    }
    let json_encode = started.elapsed();

    let started = Instant::now();
    let mut cbor_len = 0usize;
    let mut cbor_payload = Vec::new();
    for _ in 0..iters {
        cbor_payload = {
            let mut payload = Vec::new();
            ciborium::ser::into_writer(&rec, &mut payload).expect("cbor encode wal");
            payload
        };
        cbor_len += std::hint::black_box(cbor_payload.len());
    }
    let cbor_encode = started.elapsed();

    let started = Instant::now();
    let mut wal_len = 0usize;
    let mut wal_payload = Vec::new();
    for _ in 0..iters {
        wal_payload = rec.encode().expect("wal encode");
        wal_len += std::hint::black_box(wal_payload.len());
    }
    let wal_encode = started.elapsed();

    let started = Instant::now();
    for _ in 0..iters {
        let _: WalRecord =
            serde_json::from_slice(std::hint::black_box(&json_payload)).expect("json decode wal");
    }
    let json_decode = started.elapsed();

    let started = Instant::now();
    for _ in 0..iters {
        let _: WalRecord = ciborium::de::from_reader(std::hint::black_box(&cbor_payload[..]))
            .expect("cbor decode wal");
    }
    let cbor_decode = started.elapsed();

    let started = Instant::now();
    for _ in 0..iters {
        let _: WalRecord =
            WalRecord::decode(std::hint::black_box(&wal_payload[..])).expect("wal decode");
    }
    let wal_decode = started.elapsed();

    println!(
        "wal codec probe batch_docs={} iters={iters}: json_len={} cbor_len={} wal_len={} json_encode={:.3}us cbor_encode={:.3}us wal_encode={:.3}us json_decode={:.3}us cbor_decode={:.3}us wal_decode={:.3}us",
        batch_docs(),
        json_len / iters,
        cbor_len / iters,
        wal_len / iters,
        json_encode.as_secs_f64() * 1_000_000.0 / iters as f64,
        cbor_encode.as_secs_f64() * 1_000_000.0 / iters as f64,
        wal_encode.as_secs_f64() * 1_000_000.0 / iters as f64,
        json_decode.as_secs_f64() * 1_000_000.0 / iters as f64,
        cbor_decode.as_secs_f64() * 1_000_000.0 / iters as f64,
        wal_decode.as_secs_f64() * 1_000_000.0 / iters as f64,
    );
}

// </HANDWRITE>

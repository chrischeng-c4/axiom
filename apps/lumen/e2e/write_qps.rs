// CODEGEN-BEGIN
//! Write-path QPS bench. Default runs are report-only.
//!
//! This complements the read/search competitive gate. It measures the HTTP
//! write handlers that matter operationally:
//!   - `PUT /collections/{id}`: schema write through the coordinator.
//!   - `POST /collections/{id}/index`: document-field writes through the
//!     coordinator.
//!
//! The embedded leg uses the in-process WAL. The sharded leg splits one HTTP
//! request across multiple local coordinators. PostgreSQL and OpenSearch rows
//! remain optional comparison probes.
//!
//! Run:
//!   cargo test --release -p lumen --test write_qps -- --ignored --nocapture
//!   LUMEN_WRITE_MODES=embedded,sharded LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=0.3 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture
//!   LUMEN_WRITE_MODES=pg,os LUMEN_WRITE_WARMUP_S=0.1 LUMEN_WRITE_WINDOW_S=1.0 cargo test --release -p lumen --test write_qps write_qps_bench -- --ignored --nocapture

use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use lumen::api::{router, AppState};
use lumen::coordinator::WriteCoordinator;
use lumen::log_entry::RaftLogEntry;
use lumen::routing::EngineShardWrite;
use lumen::storage::Engine;
use lumen::types::{
    Analyzer, CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest,
};
use lumen::wal::{MemWal, WalRecord};

const DEFAULT_WARMUP_S: f64 = 2.0;
const PUT_WORKERS: &[usize] = &[1, 10];
const INDEX_WORKERS: &[usize] = &[1, 10, 100];
const DEFAULT_WINDOW_S: f64 = 5.0;
const DEFAULT_BATCH_DOCS: usize = 100;
const DEFAULT_REQ_TIMEOUT_MS: u64 = 2_000;
const PG_DSN: &str = "host=/tmp dbname=lumenbench";
const OS_URL: &str = "http://localhost:9200";
const PG_MAX_POOL: usize = 90;

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
        return true;
    };
    let mut saw_any = false;
    for mode in raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        saw_any = true;
        let mode = mode.to_ascii_lowercase();
        let normalized = match mode.as_str() {
            "opensearch" => "os",
            other => other,
        };
        match normalized {
            "embedded" | "sharded" | "pg" | "os" => {}
            _ => panic!("unknown LUMEN_WRITE_MODES entry `{mode}`"),
        }
        if normalized == name {
            return true;
        }
    }
    !saw_any
}

fn write_modes_label() -> String {
    std::env::var("LUMEN_WRITE_MODES").unwrap_or_else(|_| "embedded,sharded,pg,os".into())
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

struct Server {
    client: reqwest::Client,
    base: String,
    task: tokio::task::JoinHandle<()>,
    aux_tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
        for task in &self.aux_tasks {
            task.abort();
        }
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
    Server {
        client,
        base,
        task,
        aux_tasks: Vec::new(),
    }
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

#[derive(Clone, Copy)]
enum Mode {
    Embedded,
    Sharded,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::Sharded => "sharded",
        }
    }

    async fn serve(self) -> Server {
        match self {
            Self::Embedded => serve_embedded().await,
            Self::Sharded => serve_sharded_embedded().await,
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
            version: None,
        });
        items.push(IndexItem {
            external_id: format!("d{n}"),
            field: "city".into(),
            value: FieldValue::String(city.into()),
            version: None,
        });
        items.push(IndexItem {
            external_id: format!("d{n}"),
            field: "age".into(),
            value: FieldValue::Number(18.0 + (n % 63) as f64),
            version: None,
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
    let server = mode.serve().await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "report-only write-path QPS bench"]
async fn write_qps_bench() {
    println!(
        "# lumen write-path QPS bench; modes={} batch_docs={} sharded_write_shards={} env: LUMEN_WRITE_MODES LUMEN_WRITE_WARMUP_S LUMEN_WRITE_WINDOW_S LUMEN_WRITE_BATCH_DOCS LUMEN_WRITE_SHARDS",
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
    let _pg = if write_mode_enabled("pg") {
        run_pg_write_mode().await
    } else {
        None
    };
    let _os = if write_mode_enabled("os") {
        run_os_write_mode().await
    } else {
        None
    };
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
// CODEGEN-END

mod committed_publish_cancellation {
    //! # Facets
    //!
    //! - Behavior: `apps/lumen/e2e/write_qps.rs:1305`, `:1313`, `:1321`,
    //!   and `:1337` require a committed write to remain in the recovery cut
    //!   through caller cancellation and delayed publish return. `:1355` and
    //!   `:1361` require the record to apply once, then reopen the fence.
    //! - Security: `apps/lumen/e2e/write_qps.rs:1313`, `:1321`, and
    //!   `:1337` cover the `WalLog` trust boundary. A cancelled caller must
    //!   not make the recovery fence honour a record the WAL already accepted.
    //!   This is the fail-closed path in `apps/lumen/src/coordinator.rs:555-577`.
    //! - Performance: gap carried by the separate capacity case. The current
    //!   promise is `apps/lumen/ROADMAP.md:60-70`; this case's 150 ms waits are
    //!   test bounds, not a product latency budget. That capacity case needs the
    //!   pending-budget probe before it can assert the 128/256 MiB promise.

    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use anyhow::Result;
    use async_trait::async_trait;
    use futures::StreamExt;
    use tokio::sync::{watch, Notify};

    use lumen::coordinator::WriteCoordinator;
    use lumen::log_entry::RaftLogEntry;
    use lumen::storage::Engine;
    use lumen::types::CreateCollectionRequest;
    use lumen::wal::{MemWal, SharedWal, WalLog, WalRecord, WalStream};

    /// A test WAL whose `publish` enqueues the record in its ordered store and
    /// makes it available to subscribers, but holds its successful return.
    /// `submit` has already crossed this WAL commit boundary when the caller
    /// cancels it.
    ///
    /// Delivery has a second gate. That lets the test prove the restore fence
    /// stays closed after cancellation and before the committed record applies.
    #[derive(Clone)]
    struct CommitThenBlockWal {
        inner: MemWal,
        committed: Arc<Notify>,
        release_publish: Arc<Notify>,
        publish_returned: Arc<Notify>,
        delivery_open: watch::Sender<bool>,
        publish_called: Arc<AtomicBool>,
        publish_returned_called: Arc<AtomicBool>,
    }

    impl CommitThenBlockWal {
        fn new() -> Self {
            let (delivery_open, _delivery_rx) = watch::channel(false);
            Self {
                inner: MemWal::new(),
                committed: Arc::new(Notify::new()),
                release_publish: Arc::new(Notify::new()),
                publish_returned: Arc::new(Notify::new()),
                delivery_open,
                publish_called: Arc::new(AtomicBool::new(false)),
                publish_returned_called: Arc::new(AtomicBool::new(false)),
            }
        }

        async fn wait_until_committed(&self) {
            loop {
                if self
                    .inner
                    .latest_seq()
                    .await
                    .expect("read committed WAL head")
                    != 0
                {
                    return;
                }
                // `notify_one` stores a permit if the producer crossed the
                // commit boundary just after the head check.
                self.committed.notified().await;
            }
        }

        async fn wait_until_publish_returned(&self) {
            loop {
                if self.publish_returned_called.load(Ordering::SeqCst) {
                    return;
                }
                self.publish_returned.notified().await;
            }
        }

        fn release_publish(&self) {
            self.release_publish.notify_one();
        }

        fn release_delivery(&self) {
            self.delivery_open.send_replace(true);
        }
    }

    #[async_trait]
    impl WalLog for CommitThenBlockWal {
        async fn publish(&self, record: WalRecord) -> Result<u64> {
            let seq = self.inner.publish(record).await?;
            self.publish_called.store(true, Ordering::SeqCst);
            self.committed.notify_one();

            self.release_publish.notified().await;
            self.publish_returned_called.store(true, Ordering::SeqCst);
            self.publish_returned.notify_one();
            Ok(seq)
        }

        async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
            let stream = self.inner.subscribe(from_seq).await?;
            let delivery_open = self.delivery_open.subscribe();
            Ok(Box::pin(stream.then(move |item| {
                let mut delivery_open = delivery_open.clone();
                async move {
                    while !*delivery_open.borrow_and_update() {
                        delivery_open
                            .changed()
                            .await
                            .map_err(|_| anyhow::anyhow!("test WAL delivery gate closed"))?;
                    }
                    item
                }
            })))
        }

        async fn latest_seq(&self) -> Result<u64> {
            self.inner.latest_seq().await
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_committed_submit_keeps_restore_fence_closed_until_apply() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(CommitThenBlockWal::new());
        let shared_wal: SharedWal = wal.clone();
        let coordinator = WriteCoordinator::start(shared_wal, engine.clone());

        let submit = {
            let coordinator = coordinator.clone();
            tokio::spawn(async move {
                coordinator
                    .submit(RaftLogEntry::CreateCollection {
                        collection_id: "cancelled-commit".into(),
                        req: CreateCollectionRequest {
                            fields: BTreeMap::new(),
                        },
                    })
                    .await
            })
        };

        tokio::time::timeout(Duration::from_secs(1), wal.wait_until_committed())
            .await
            .expect("custom WAL must enqueue the record before submit cancellation");
        assert!(wal.publish_called.load(Ordering::SeqCst));
        assert_eq!(
            wal.latest_seq().await.expect("read WAL head"),
            1,
            "the case must not pass because no record was committed"
        );

        submit.abort();
        assert!(
            submit
                .await
                .expect_err("cancelling submit must cancel its caller task")
                .is_cancelled(),
            "the submit caller must actually be cancelled after WAL commit"
        );

        assert!(
            tokio::time::timeout(Duration::from_millis(150), coordinator.fence_mutations())
                .await
                .is_err(),
            "a cancelled caller must not open the restore fence while its committed record waits to apply"
        );

        // A correct coordinator leaves a detached publication task alive after
        // caller cancellation. Let it receive its sequence and hand the permit
        // into sequence ownership, but keep delivery closed.
        wal.release_publish();
        tokio::time::timeout(Duration::from_secs(1), wal.wait_until_publish_returned())
            .await
            .expect(
                "detached publication must receive the committed sequence after caller cancellation",
            );
        assert!(
            tokio::time::timeout(Duration::from_millis(150), coordinator.fence_mutations())
                .await
                .is_err(),
            "the restore fence must remain closed after publish returns and before the committed record applies"
        );

        wal.release_delivery();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if coordinator.applied_seq() == 1 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("the committed record must apply after delivery opens");
        assert_eq!(
            engine.list_collections().expect("list applied collections"),
            vec!["cancelled-commit"],
            "the committed record must apply exactly once after cancellation"
        );

        let _restore_fence =
            tokio::time::timeout(Duration::from_secs(1), coordinator.fence_mutations())
                .await
                .expect("restore fence must open after the sequence completes")
                .expect("restore fence remains usable after the sequence completes");
    }
}

mod local_oversized_record_capacity {
    //! # Facets
    //!
    //! - Behavior: write_qps.rs:1583, :1589, :1594, :1599, :1604, :1609,
    //!   :1625, :1626, :1631, and :1636 require a valid oversized local index request
    //!   to return the capacity refusal, leave state unchanged, and then accept
    //!   a small request. Change points: apps/lumen/src/coordinator.rs:776-800
    //!   and apps/lumen/src/api.rs:3170-3178.
    //! - Security: write_qps.rs:1583, :1599, :1604, and :1609 keep the
    //!   caller-controlled 100 MiB Keyword body outside WAL and the collection.
    //!   The closed HTTP input boundary is apps/lumen/src/coordinator.rs:776-800
    //!   through apps/lumen/src/api.rs:3170-3178.
    //! - Performance: apps/lumen/docs/indexing.md:264-270 promises the 256 MiB
    //!   pending budget and its pre-submit 429 response. write_qps.rs:1516 and
    //!   :1522 measure a real body below its child-only HTTP allowance and above
    //!   one third of that budget. Child and request timeouts are cleanup bounds,
    //!   not latency claims.

    use std::fs::{self, File};
    use std::process::{Child, Command, Stdio};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use axum::http::{header::RETRY_AFTER, StatusCode};
    use axum_test::TestServer;
    use serde_json::{json, Value};

    use lumen::api::{router, AppState};
    use lumen::auth::AuthConfig;
    use lumen::coordinator::{WriteCoordinator, WriteSink};
    use lumen::storage::Engine;
    use lumen::wal::{MemWal, SharedWal, WalLog};

    const COLLECTION: &str = "oversized-local-record";
    const PENDING_HARD_LIMIT_BYTES: usize = 256 * 1024 * 1024;
    const OVERSIZED_KEYWORD_BYTES: usize = 100 * 1024 * 1024;
    const CHILD_BODY_LIMIT_BYTES: usize = OVERSIZED_KEYWORD_BYTES + 2 * 1024 * 1024;
    const CHILD_CASE_ENV: &str = "LUMEN_OVERSIZED_LOCAL_RECORD_CHILD";
    const CHILD_HANDSHAKE_ENV: &str = "LUMEN_OVERSIZED_LOCAL_RECORD_HANDSHAKE";
    const CHILD_CASE: &str = "oversized-local-record";
    const TEST_NAME: &str =
        "local_oversized_record_capacity::oversized_local_index_refuses_before_wal_publish";
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
    const CHILD_TIMEOUT: Duration = Duration::from_secs(90);

    /// Owns a child until it has exited. A test timeout therefore cannot leave
    /// the isolated process or its process-wide budget alive for later tests.
    struct ChildCleanup(Option<Child>);

    impl Drop for ChildCleanup {
        fn drop(&mut self) {
            let Some(child) = self.0.as_mut() else {
                return;
            };
            if !matches!(child.try_wait(), Ok(Some(_))) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }

    fn child_enters() -> bool {
        if std::env::var(CHILD_CASE_ENV).ok().as_deref() != Some(CHILD_CASE) {
            return false;
        }
        let handshake = std::env::var_os(CHILD_HANDSHAKE_ENV)
            .expect("oversized capacity child needs a handshake path");
        fs::write(handshake, CHILD_CASE).expect("write oversized capacity child handshake");
        true
    }

    async fn run_isolated_child() {
        let dir = tempfile::tempdir().expect("oversized capacity child directory");
        let handshake = dir.path().join("entered-case");
        let stdout_path = dir.path().join("child.stdout");
        let stderr_path = dir.path().join("child.stderr");
        let executable = std::env::current_exe().expect("current write_qps test executable");
        let stdout = File::create(&stdout_path).expect("create oversized child stdout");
        let stderr = File::create(&stderr_path).expect("create oversized child stderr");
        let child = Command::new(executable)
            .env(CHILD_CASE_ENV, CHILD_CASE)
            .env(CHILD_HANDSHAKE_ENV, &handshake)
            .env("LUMEN_BODY_LIMIT_BYTES", CHILD_BODY_LIMIT_BYTES.to_string())
            .arg(TEST_NAME)
            .arg("--exact")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .expect("spawn isolated oversized capacity child");
        let mut child = ChildCleanup(Some(child));
        let deadline = Instant::now() + CHILD_TIMEOUT;
        let status = loop {
            match child
                .0
                .as_mut()
                .expect("child remains owned until it exits")
                .try_wait()
            {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Ok(None) => panic!(
                    "isolated oversized capacity child exceeded {:?}",
                    CHILD_TIMEOUT
                ),
                Err(error) => panic!("poll isolated oversized capacity child: {error}"),
            }
        };
        let stdout = fs::read_to_string(&stdout_path).expect("read oversized child stdout");
        let stderr = fs::read_to_string(&stderr_path).expect("read oversized child stderr");
        let entered = fs::read_to_string(&handshake).unwrap_or_else(|error| {
            panic!(
                "oversized capacity child did not reach {TEST_NAME}: {error}; stdout={stdout}; stderr={stderr}"
            )
        });
        assert_eq!(
            entered, CHILD_CASE,
            "child must enter the exact oversized-capacity body before it can pass"
        );
        assert!(
            status.success(),
            "isolated oversized capacity child failed: status={status}; stdout={stdout}; stderr={stderr}"
        );
        child.0.take();
    }

    fn oversized_index_body() -> Vec<u8> {
        let mut value = vec![b'x'; OVERSIZED_KEYWORD_BYTES];
        let marker = b"oversized-local-keyword-";
        value[..marker.len()].copy_from_slice(marker);
        let request = json!({
            "items": [{
                "external_id": "oversized",
                "field": "kw",
                "value": String::from_utf8(value).expect("ASCII oversized Keyword value"),
            }]
        });
        assert_eq!(
            request["items"].as_array().map(Vec::len),
            Some(1),
            "the capacity request uses one valid index item"
        );
        let body = serde_json::to_vec(&request).expect("serialize oversized Keyword request");
        drop(request);
        assert!(
            body.len() < CHILD_BODY_LIMIT_BYTES,
            "the real request body must stay below its child-only HTTP allowance: {} >= {}",
            body.len(),
            CHILD_BODY_LIMIT_BYTES
        );
        assert!(
            body.len() > PENDING_HARD_LIMIT_BYTES / 3,
            "the real body must make the local raw plus two transport copies exceed the approved 256 MiB budget: {} <= {}",
            body.len(),
            PENDING_HARD_LIMIT_BYTES / 3
        );
        body
    }

    async fn indexed_total(server: &TestServer) -> u64 {
        let response = server
            .post(&format!("/collections/{COLLECTION}/search"))
            .json(&json!({
                "query": { "exists": { "field": "kw" } },
                "limit": 10,
            }))
            .await;
        response.assert_status_ok();
        response.json::<Value>()["total"]
            .as_u64()
            .expect("keyword exists response has a total")
    }

    async fn oversized_local_index_refuses_before_wal_publish_body() {
        let engine = Arc::new(Engine::new());
        let wal = Arc::new(MemWal::new());
        let shared_wal: SharedWal = wal.clone();
        let writer = WriteCoordinator::start(shared_wal, engine.clone());
        let sink: Arc<dyn WriteSink> = writer.clone();
        let server = TestServer::new(router(AppState::with_components(
            engine,
            Arc::new(AuthConfig::open()),
            sink,
        )))
        .expect("oversized capacity HTTP server");

        server
            .put(&format!("/collections/{COLLECTION}"))
            .json(&json!({ "fields": { "kw": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
        let applied_before = writer.applied_seq();
        let wal_before = wal.latest_seq().await.expect("read MemWal before refusal");
        assert_eq!(
            indexed_total(&server).await,
            0,
            "the new collection must start without indexed Keyword values"
        );

        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            server
                .post(&format!("/collections/{COLLECTION}/index"))
                .bytes(oversized_index_body().into())
                .content_type("application/json"),
        )
        .await
        .expect("oversized local index request must finish");
        let retry_after = response
            .maybe_header(RETRY_AFTER)
            .and_then(|value| value.to_str().ok().map(ToOwned::to_owned));
        assert_eq!(
            response.status_code(),
            StatusCode::TOO_MANY_REQUESTS,
            "a valid local record that cannot fit the 256 MiB pending budget must refuse before WAL publication"
        );
        let envelope = response.json::<Value>();
        assert_eq!(
            envelope["error"], "pending_change_capacity",
            "the valid oversized request must reach the capacity refusal, not a validation error"
        );
        assert_eq!(
            retry_after.as_deref(),
            Some("1"),
            "pre-publication capacity refusal must expose Retry-After: 1"
        );
        assert_eq!(
            wal.latest_seq().await.expect("read MemWal after refusal"),
            wal_before,
            "the refused caller body must not allocate a WAL sequence"
        );
        assert_eq!(
            writer.applied_seq(),
            applied_before,
            "the refused caller body must not advance the applied sequence"
        );
        assert_eq!(
            indexed_total(&server).await,
            0,
            "the refused caller body must not index a document"
        );

        server
            .post(&format!("/collections/{COLLECTION}/index"))
            .json(&json!({
                "items": [{
                    "external_id": "small-after-refusal",
                    "field": "kw",
                    "value": "small",
                }]
            }))
            .await
            .assert_status_ok();
        assert_eq!(
            wal.latest_seq()
                .await
                .expect("read MemWal after small request"),
            wal_before + 1,
            "only the later small request may allocate the next WAL sequence"
        );
        assert_eq!(
            writer.applied_seq(),
            applied_before + 1,
            "only the later small request may advance the applied sequence"
        );
        assert_eq!(
            indexed_total(&server).await,
            1,
            "the later small request must remain usable after a capacity refusal"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn oversized_local_index_refuses_before_wal_publish() {
        if child_enters() {
            oversized_local_index_refuses_before_wal_publish_body().await;
        } else {
            run_isolated_child().await;
        }
    }
}

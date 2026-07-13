// HANDWRITE-BEGIN gap="sift-service-core" tracker="1576" reason="Implement the versioned operational-event envelope, durable raw journal, idempotency, query, and replay core."
//! Sift's bootstrap service core: a versioned six-signal envelope and the
//! canonical, fsync-before-ack raw event journal. Materialized log, trace,
//! error, metric, and audit/change stores deliberately build from this journal
//! in later slices rather than becoming alternate sources of truth.

use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use anyhow::{bail, Context, Result};
use axum::{
    extract::{rejection::JsonRejection, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{OpenApi, ToSchema};

pub const EVENT_SCHEMA_VERSION: u16 = 1;
const JOURNAL_FILE: &str = "raw-events.ndjson";

/// The six operational signal kinds accepted into Sift's canonical event log.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, ToSchema, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    Log,
    Span,
    Metric,
    Exception,
    AuditEvent,
    ChangeEvent,
}

impl SignalKind {
    pub const ALL: [Self; 6] = [
        Self::Log,
        Self::Span,
        Self::Metric,
        Self::Exception,
        Self::AuditEvent,
        Self::ChangeEvent,
    ];
}

impl std::fmt::Display for SignalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Log => "log",
            Self::Span => "span",
            Self::Metric => "metric",
            Self::Exception => "exception",
            Self::AuditEvent => "audit_event",
            Self::ChangeEvent => "change_event",
        };
        f.write_str(name)
    }
}

/// Metric aggregation semantics are preserved in raw events; Sift does not
/// rewrite direct points into logs before the metric store consumes them.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MetricTemporality {
    Delta,
    Cumulative,
    Gauge,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricExemplar {
    pub value: f64,
    pub trace_id: String,
    pub span_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct MetricPoint {
    pub name: String,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub temporality: MetricTemporality,
    #[serde(default)]
    pub exemplars: Vec<MetricExemplar>,
}

/// Stable, producer-neutral envelope written to the raw journal.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub event_id: String,
    #[serde(default = "now_rfc3339")]
    pub occurred_at: String,
    pub signal: SignalKind,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub resource: BTreeMap<String, String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric: Option<MetricPoint>,
    #[schema(value_type = Object)]
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(event_id: impl Into<String>, signal: SignalKind, payload: Value) -> Self {
        Self {
            schema_version: EVENT_SCHEMA_VERSION,
            event_id: event_id.into(),
            occurred_at: now_rfc3339(),
            signal,
            resource: BTreeMap::new(),
            attributes: BTreeMap::new(),
            trace_id: None,
            span_id: None,
            severity: None,
            metric: None,
            payload,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != EVENT_SCHEMA_VERSION {
            bail!(
                "unsupported schema_version {}; expected {}",
                self.schema_version,
                EVENT_SCHEMA_VERSION
            );
        }
        if self.event_id.trim().is_empty() {
            bail!("event_id must not be empty");
        }
        if self.occurred_at.trim().is_empty() {
            bail!("occurred_at must not be empty");
        }
        if self.resource.is_empty() {
            bail!("resource must contain at least one stable identity field");
        }
        if self.payload.is_null() {
            bail!("payload must not be null");
        }
        if self.signal == SignalKind::Metric {
            let metric = self
                .metric
                .as_ref()
                .context("metric signals require a direct metric point")?;
            if metric.name.trim().is_empty() || !metric.value.is_finite() {
                bail!("metric name must be non-empty and value must be finite");
            }
            for exemplar in &metric.exemplars {
                if exemplar.trace_id.trim().is_empty() || exemplar.span_id.trim().is_empty() {
                    bail!("metric exemplars require trace_id and span_id");
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct StoredEvent {
    pub cursor: u64,
    pub acknowledged_at: String,
    pub event: EventEnvelope,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
pub struct AppendResult {
    pub event_id: String,
    pub cursor: u64,
    pub duplicate: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EventQuery {
    pub signal: Option<SignalKind>,
    pub after: u64,
    pub limit: usize,
}

#[derive(Default)]
struct JournalState {
    events: Vec<StoredEvent>,
    cursors_by_event_id: HashMap<String, u64>,
}

/// Append-only JSONL journal. State is updated only after `sync_data` succeeds,
/// making a successful [`append`](Self::append) acknowledgement durable.
pub struct DurableJournal {
    journal_path: PathBuf,
    state: RwLock<JournalState>,
    accepted: AtomicU64,
    duplicates: AtomicU64,
    fsyncs: AtomicU64,
}

impl DurableJournal {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)
            .with_context(|| format!("create Sift data directory {}", data_dir.display()))?;
        let journal_path = data_dir.join(JOURNAL_FILE);
        let mut state = JournalState::default();

        if journal_path.exists() {
            let file = File::open(&journal_path)
                .with_context(|| format!("open journal {}", journal_path.display()))?;
            for (line_number, line) in BufReader::new(file).lines().enumerate() {
                let line =
                    line.with_context(|| format!("read journal line {}", line_number + 1))?;
                if line.trim().is_empty() {
                    continue;
                }
                let stored: StoredEvent = serde_json::from_str(&line)
                    .with_context(|| format!("decode journal line {}", line_number + 1))?;
                stored.event.validate().with_context(|| {
                    format!(
                        "validate recovered event at journal line {}",
                        line_number + 1
                    )
                })?;
                if state
                    .cursors_by_event_id
                    .insert(stored.event.event_id.clone(), stored.cursor)
                    .is_some()
                {
                    bail!(
                        "journal contains duplicate event_id {}",
                        stored.event.event_id
                    );
                }
                state.events.push(stored);
            }
        }

        let accepted = state.events.len() as u64;
        Ok(Self {
            journal_path,
            state: RwLock::new(state),
            accepted: AtomicU64::new(accepted),
            duplicates: AtomicU64::new(0),
            fsyncs: AtomicU64::new(0),
        })
    }

    pub fn append(&self, event: EventEnvelope) -> Result<AppendResult> {
        event.validate()?;
        let mut state = self.state.write().expect("journal state lock poisoned");
        if let Some(cursor) = state.cursors_by_event_id.get(&event.event_id).copied() {
            self.duplicates.fetch_add(1, Ordering::Relaxed);
            return Ok(AppendResult {
                event_id: event.event_id,
                cursor,
                duplicate: true,
            });
        }

        let cursor = state
            .events
            .last()
            .map(|entry| entry.cursor + 1)
            .unwrap_or(1);
        let stored = StoredEvent {
            cursor,
            acknowledged_at: now_rfc3339(),
            event,
        };
        let encoded = serde_json::to_vec(&stored).context("encode raw event")?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.journal_path)
            .with_context(|| format!("open journal {} for append", self.journal_path.display()))?;
        file.write_all(&encoded).context("append raw event")?;
        file.write_all(b"\n").context("terminate raw event")?;
        file.sync_data()
            .context("fsync raw event before acknowledgement")?;
        self.fsyncs.fetch_add(1, Ordering::Relaxed);

        state
            .cursors_by_event_id
            .insert(stored.event.event_id.clone(), stored.cursor);
        state.events.push(stored.clone());
        self.accepted.fetch_add(1, Ordering::Relaxed);
        Ok(AppendResult {
            event_id: stored.event.event_id,
            cursor,
            duplicate: false,
        })
    }

    pub fn query(&self, query: EventQuery) -> Result<Vec<StoredEvent>> {
        let limit = if query.limit == 0 {
            100
        } else {
            query.limit.clamp(1, 10_000)
        };
        let state = self.state.read().expect("journal state lock poisoned");
        Ok(state
            .events
            .iter()
            .filter(|entry| entry.cursor > query.after)
            .filter(|entry| {
                query
                    .signal
                    .is_none_or(|signal| entry.event.signal == signal)
            })
            .take(limit)
            .cloned()
            .collect())
    }

    pub fn replay(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        self.query(EventQuery {
            signal: None,
            after,
            limit,
        })
    }

    fn metrics_text(&self) -> String {
        format!(
            "# TYPE sift_raw_events_total counter\nsift_raw_events_total {}\n# TYPE sift_duplicate_events_total counter\nsift_duplicate_events_total {}\n# TYPE sift_journal_fsync_total counter\nsift_journal_fsync_total {}\n",
            self.accepted.load(Ordering::Relaxed),
            self.duplicates.load(Ordering::Relaxed),
            self.fsyncs.load(Ordering::Relaxed),
        )
    }
}

/// Shared HTTP state: journal access plus the drain bit read by `/readyz`.
#[derive(Clone)]
pub struct ServiceState {
    journal: Arc<DurableJournal>,
    draining: Arc<AtomicBool>,
}

impl ServiceState {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            journal: Arc::new(DurableJournal::open(data_dir)?),
            draining: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::Release);
    }
}

impl service_http::ReadinessHook for ServiceState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::Acquire)
    }
}

impl service_http::MetricsProvider for ServiceState {
    fn render_metrics(&self) -> String {
        self.journal.metrics_text()
    }
}

#[derive(Serialize, ToSchema)]
struct ErrorEnvelope {
    error: String,
    message: String,
}

struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "journal_failure",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: self.error.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}

/// Build Sift's data-plane routes. Probe/admin routes are intentionally added
/// by `service-http` so all k8s-native services have the same shape.
pub fn router(state: Arc<ServiceState>) -> Router {
    Router::new()
        .route("/v1/events", post(ingest).get(query_events))
        .route("/v1/replay", get(replay_events))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/v1/events",
    request_body = EventEnvelope,
    responses(
        (status = 201, description = "raw event appended and fsynced", body = AppendResult),
        (status = 200, description = "idempotent retry", body = AppendResult),
        (status = 400, description = "invalid envelope", body = ErrorEnvelope),
        (status = 500, description = "journal write failure", body = ErrorEnvelope)
    )
)]
async fn ingest(
    State(state): State<Arc<ServiceState>>,
    payload: Result<Json<EventEnvelope>, JsonRejection>,
) -> Result<(StatusCode, Json<AppendResult>), ApiError> {
    let Json(event) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    let result = state
        .journal
        .append(event)
        .map_err(|error| ApiError::bad_request("invalid_event", error.to_string()))?;
    let status = if result.duplicate {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    tracing::info!(
        cursor = result.cursor,
        duplicate = result.duplicate,
        "raw event acknowledged"
    );
    Ok((status, Json(result)))
}

#[derive(Debug, Deserialize)]
struct HttpEventQuery {
    signal: Option<SignalKind>,
    after: Option<u64>,
    limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/v1/events",
    responses((status = 200, description = "durable raw events", body = [StoredEvent]))
)]
async fn query_events(
    State(state): State<Arc<ServiceState>>,
    Query(query): Query<HttpEventQuery>,
) -> Result<Json<Vec<StoredEvent>>, ApiError> {
    let rows = state
        .journal
        .query(EventQuery {
            signal: query.signal,
            after: query.after.unwrap_or(0),
            limit: query.limit.unwrap_or(100),
        })
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
struct HttpReplayQuery {
    after: Option<u64>,
    limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/v1/replay",
    responses((status = 200, description = "ordered replay from the raw journal", body = [StoredEvent]))
)]
async fn replay_events(
    State(state): State<Arc<ServiceState>>,
    Query(query): Query<HttpReplayQuery>,
) -> Result<Json<Vec<StoredEvent>>, ApiError> {
    let rows = state
        .journal
        .replay(query.after.unwrap_or(0), query.limit.unwrap_or(100))
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Ok(Json(rows))
}

#[derive(OpenApi)]
#[openapi(
    paths(ingest, query_events, replay_events),
    components(schemas(
        EventEnvelope,
        SignalKind,
        MetricPoint,
        MetricTemporality,
        MetricExemplar,
        StoredEvent,
        AppendResult,
        ErrorEnvelope
    )),
    tags((name = "events", description = "Versioned operational-event ingestion and durable replay"))
)]
struct SiftApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    SiftApi::openapi()
}

pub fn openapi_json() -> Result<String> {
    serde_json::to_string_pretty(&openapi()).context("serialize OpenAPI contract")
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}
// HANDWRITE-END

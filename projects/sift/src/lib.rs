// HANDWRITE-BEGIN gap="sift-service-core" tracker="1576" reason="Implement the versioned operational-event envelope, durable raw journal, idempotency, query, and replay core."
//! Sift's service core: a versioned eight-signal envelope and the
//! canonical, fsync-before-ack raw event journal. Materialized log, trace,
//! error, metric, and audit/change stores deliberately build from this journal
//! in later slices rather than becoming alternate sources of truth.

pub mod auth;
pub mod backup;
pub mod deploy;
pub mod durability;
pub mod event;
pub mod operator;

pub use event::{
    decode_event_json, AttributeValue, EventEnvelope, EventEnvelopeV1, GovernancePolicy,
    GovernancePolicySet, IncomingEvent, InstrumentationScope, MetricExemplar, MetricPoint,
    MetricTemporality, OperationalEventV2, SignalKind, EVENT_SCHEMA_URL, EVENT_SCHEMA_VERSION,
    EVENT_SCHEMA_VERSION_V1,
};

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, RwLock,
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
use serde::{Deserialize, Deserializer, Serialize};
use service_metrics::{Counter, Sample};
use utoipa::{OpenApi, ToSchema};

const JOURNAL_FILE: &str = "raw-events.framed";
const SNAPSHOT_FILE: &str = "raw-events.snapshot.json";

#[derive(Clone, Debug, PartialEq, Serialize, ToSchema)]
pub struct StoredEvent {
    pub cursor: u64,
    pub acknowledged_at: String,
    pub event: EventEnvelope,
}

#[derive(Deserialize)]
struct StoredEventWire {
    cursor: u64,
    acknowledged_at: String,
    event: IncomingEvent,
}

impl<'de> Deserialize<'de> for StoredEvent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = StoredEventWire::deserialize(deserializer)?;
        Ok(Self {
            cursor: wire.cursor,
            acknowledged_at: wire.acknowledged_at,
            event: wire.event.into_inner(),
        })
    }
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
    snapshot_path: PathBuf,
    writer: Mutex<service_durability::FramedLogWriter>,
    state: RwLock<JournalState>,
    governance: GovernancePolicySet,
    accepted: Counter,
    duplicates: Counter,
    fsyncs: Counter,
}

impl DurableJournal {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_governance(data_dir, GovernancePolicySet::from_env()?)
    }

    pub fn open_with_governance(
        data_dir: impl AsRef<Path>,
        governance: GovernancePolicySet,
    ) -> Result<Self> {
        governance.validate()?;
        let data_dir = data_dir.as_ref();
        fs::create_dir_all(data_dir)
            .with_context(|| format!("create Sift data directory {}", data_dir.display()))?;
        let journal_path = data_dir.join(JOURNAL_FILE);
        let snapshot_path = data_dir.join(SNAPSHOT_FILE);
        let mut state = JournalState::default();

        if snapshot_path.exists() {
            let snapshot: durability::JournalSnapshot = serde_json::from_slice(
                &fs::read(&snapshot_path)
                    .with_context(|| format!("read snapshot {}", snapshot_path.display()))?,
            )
            .with_context(|| format!("decode snapshot {}", snapshot_path.display()))?;
            Self::replace_state(&mut state, snapshot.events)?;
        }

        for frame in service_durability::FramedLogReader::read_frames(&journal_path, 0)? {
            let stored: StoredEvent = serde_json::from_slice(&frame.payload)
                .with_context(|| format!("decode journal frame {}", frame.seq))?;
            if frame.seq != stored.cursor {
                bail!(
                    "journal frame sequence {} does not match stored cursor {}",
                    frame.seq,
                    stored.cursor
                );
            }
            if state
                .cursors_by_event_id
                .contains_key(&stored.event.event_id)
            {
                continue;
            }
            Self::insert_recovered(&mut state, stored)?;
        }

        let accepted = state.events.len() as u64;
        let journal = Self {
            snapshot_path,
            writer: Mutex::new(service_durability::FramedLogWriter::open(
                data_dir.join(JOURNAL_FILE),
                service_durability::FsyncPolicy::Always,
            )?),
            state: RwLock::new(state),
            governance,
            accepted: Counter::new(),
            duplicates: Counter::new(),
            fsyncs: Counter::new(),
        };
        journal.accepted.add(accepted);
        Ok(journal)
    }

    pub fn append(&self, event: EventEnvelope) -> Result<AppendResult> {
        self.append_with_cursor(None, event)
    }

    pub(crate) fn append_at(&self, cursor: u64, event: EventEnvelope) -> Result<AppendResult> {
        self.append_with_cursor(Some(cursor), event)
    }

    fn append_with_cursor(
        &self,
        expected_cursor: Option<u64>,
        event: EventEnvelope,
    ) -> Result<AppendResult> {
        let event = self.govern_event(event)?;
        let mut state = self.state.write().expect("journal state lock poisoned");
        if let Some(cursor) = state.cursors_by_event_id.get(&event.event_id).copied() {
            self.duplicates.incr();
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
        if let Some(expected_cursor) = expected_cursor {
            if cursor != expected_cursor {
                bail!(
                    "replicated cursor {expected_cursor} does not follow local durable cursor {}",
                    cursor.saturating_sub(1)
                );
            }
        }
        let stored = StoredEvent {
            cursor,
            acknowledged_at: now_rfc3339(),
            event,
        };
        let encoded = serde_json::to_vec(&stored).context("encode raw event")?;
        self.writer
            .lock()
            .expect("journal writer lock poisoned")
            .append(cursor, &encoded)
            .context("CRC-frame and fsync raw event before acknowledgement")?;
        self.fsyncs.incr();

        state
            .cursors_by_event_id
            .insert(stored.event.event_id.clone(), stored.cursor);
        state.events.push(stored.clone());
        self.accepted.incr();
        Ok(AppendResult {
            event_id: stored.event.event_id,
            cursor,
            duplicate: false,
        })
    }

    pub fn govern_event(&self, event: EventEnvelope) -> Result<EventEnvelope> {
        self.governance.govern(event)
    }

    fn insert_recovered(state: &mut JournalState, stored: StoredEvent) -> Result<()> {
        stored.event.validate()?;
        let expected = state
            .events
            .last()
            .map(|entry| entry.cursor + 1)
            .unwrap_or(1);
        if stored.cursor != expected {
            bail!(
                "journal cursor {} is out of order; expected {expected}",
                stored.cursor
            );
        }
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
        Ok(())
    }

    fn replace_state(state: &mut JournalState, events: Vec<StoredEvent>) -> Result<()> {
        *state = JournalState::default();
        for event in events {
            Self::insert_recovered(state, event)?;
        }
        Ok(())
    }

    pub(crate) fn last_cursor(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .events
            .last()
            .map(|event| event.cursor)
            .unwrap_or(0)
    }

    pub(crate) fn snapshot_events(&self) -> Vec<StoredEvent> {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .events
            .clone()
    }

    pub fn snapshot_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&durability::JournalSnapshot::from_events(
            self.snapshot_events(),
        ))
        .context("serialize durable journal snapshot")
    }

    pub fn restore_snapshot_bytes(&self, bytes: &[u8]) -> Result<()> {
        let snapshot: durability::JournalSnapshot =
            serde_json::from_slice(bytes).context("decode durable journal snapshot")?;
        self.restore_snapshot(snapshot.events)
    }

    pub(crate) fn restore_snapshot(&self, events: Vec<StoredEvent>) -> Result<()> {
        let snapshot = durability::JournalSnapshot::from_events(events.clone());
        service_durability::atomic_write(
            &self.snapshot_path,
            &serde_json::to_vec(&snapshot).context("serialize snapshot replacement")?,
            service_durability::FsyncPolicy::Always,
        )?;
        let mut state = self.state.write().expect("journal state lock poisoned");
        Self::replace_state(&mut state, events)?;
        Ok(())
    }

    fn result_for(&self, event_id: &str) -> Option<AppendResult> {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .cursors_by_event_id
            .get(event_id)
            .copied()
            .map(|cursor| AppendResult {
                event_id: event_id.to_string(),
                cursor,
                duplicate: true,
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
        service_metrics::render(&[
            Sample::new(
                "sift_raw_events_total",
                "counter",
                "Durably accepted Sift raw events.",
                self.accepted.get(),
            ),
            Sample::new(
                "sift_duplicate_events_total",
                "counter",
                "Idempotent duplicate Sift event submissions.",
                self.duplicates.get(),
            ),
            Sample::new(
                "sift_journal_fsync_total",
                "counter",
                "Sift journal fsync operations completed before acknowledgement.",
                self.fsyncs.get(),
            ),
        ])
    }
}

/// Shared HTTP state: journal access plus the drain bit read by `/readyz`.
#[derive(Clone)]
pub struct ServiceState {
    journal: Arc<DurableJournal>,
    draining: Arc<AtomicBool>,
    raft: Option<Arc<raft_host::RaftHost>>,
}

impl ServiceState {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let journal = Arc::new(DurableJournal::open(data_dir)?);
        let raft = if raft_host::replica_mode() {
            let topology =
                raft_host::ClusterTopology::from_env("sift", "sift-peer", 7380, "SIFT_PEERS")?;
            let state_machine = Arc::new(durability::SiftStateMachine::new(journal.clone()));
            let store = raft_host::RaftStore::open(
                data_dir
                    .to_str()
                    .context("Sift data directory must be valid UTF-8 for raft storage")?,
                topology.node_id,
                raft_host::FsyncPolicy::Always,
            )
            .context("open Sift raft store")?;
            Some(Arc::new(raft_host::RaftHost::spawn(
                topology.node_id,
                topology.membership,
                topology.peers,
                store,
                state_machine as Arc<dyn raft_host::RaftStateMachine>,
                raft_host::HostConfig::default(),
            )))
        } else {
            None
        };
        Ok(Self {
            journal,
            draining: Arc::new(AtomicBool::new(false)),
            raft,
        })
    }

    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::Release);
    }

    async fn append(&self, event: EventEnvelope) -> Result<AppendResult> {
        // Govern before the Raft proposal so sensitive content never enters a
        // replicated log, even transiently. DurableJournal repeats the policy
        // idempotently at the raw boundary for direct/single-node callers.
        let event = self.journal.govern_event(event)?;
        if let Some(accepted) = self.journal.result_for(&event.event_id) {
            return Ok(accepted);
        }
        let event_id = event.event_id.clone();
        if let Some(raft) = &self.raft {
            raft.propose(serde_json::to_vec(&event).context("encode replicated raw event")?)
                .await?;
            return self
                .journal
                .result_for(&event_id)
                .context("Raft proposal completed without applying the Sift event");
        }
        self.journal.append(event)
    }

    pub fn raft_router(&self) -> Option<Router> {
        self.raft.as_ref().map(|raft| raft.router())
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

/// Build the production data-plane router. The standard operational probe
/// router is intentionally composed outside this function, so its endpoints
/// remain reachable when `SIFT_AUTH=required`.
pub fn protected_router(state: Arc<ServiceState>, verifier: Arc<auth::SiftVerifier>) -> Router {
    router(state).layer(axum::middleware::from_fn_with_state(
        verifier,
        auth::auth_middleware,
    ))
}

#[utoipa::path(
    post,
    path = "/v1/events",
    request_body = OperationalEventV2,
    responses(
        (status = 201, description = "raw event appended and fsynced", body = AppendResult),
        (status = 200, description = "idempotent retry", body = AppendResult),
        (status = 400, description = "invalid envelope", body = ErrorEnvelope),
        (status = 500, description = "journal write failure", body = ErrorEnvelope)
    )
)]
async fn ingest(
    State(state): State<Arc<ServiceState>>,
    payload: Result<Json<IncomingEvent>, JsonRejection>,
) -> Result<(StatusCode, Json<AppendResult>), ApiError> {
    let Json(event) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    let result = state
        .append(event.into_inner())
        .await
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
        OperationalEventV2,
        AttributeValue,
        InstrumentationScope,
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

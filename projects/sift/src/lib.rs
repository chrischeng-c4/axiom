// HANDWRITE-BEGIN gap="sift-service-core" tracker="1576" reason="Implement the versioned operational-event envelope, durable raw journal, idempotency, query, and replay core."
//! Sift's service core: a versioned eight-signal envelope and the
//! canonical, fsync-before-ack raw event journal. Materialized log, trace,
//! error, metric, and audit/change stores deliberately build from this journal
//! in later slices rather than becoming alternate sources of truth.

pub mod auth;
pub mod backup;
pub mod collector;
pub mod deploy;
pub mod durability;
pub mod event;
pub mod ingest;
pub mod operator;
pub mod projection;
pub mod storage;

pub use event::{
    decode_event_json, AttributeValue, ContentBlobRef, EventEnvelope, EventEnvelopeV1,
    GovernancePolicy, GovernancePolicySet, IncomingEvent, InstrumentationScope, MetricExemplar,
    MetricPoint, MetricTemporality, OperationalEventV2, SignalKind, EVENT_SCHEMA_URL,
    EVENT_SCHEMA_VERSION, EVENT_SCHEMA_VERSION_V1,
};

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use anyhow::{bail, Context, Result};
use axum::{
    body::{Body, Bytes},
    extract::{rejection::JsonRejection, Extension, Path as AxumPath, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use service_auth::{Role, RoleMapPrincipal};
use service_metrics::{Counter, Sample};
use utoipa::{OpenApi, ToSchema};

const JOURNAL_FILE: &str = "raw-events.framed";
const SNAPSHOT_FILE: &str = "raw-events.snapshot.json";

fn rewrite_compatibility_journal(path: &Path, events: &[StoredEvent]) -> Result<()> {
    let temporary = path.with_extension(format!("rebuild-{}", std::process::id()));
    if temporary.exists() {
        fs::remove_file(&temporary)
            .with_context(|| format!("remove stale journal rebuild {}", temporary.display()))?;
    }
    let mut writer = service_durability::FramedLogWriter::open(
        &temporary,
        service_durability::FsyncPolicy::Always,
    )?;
    for event in events {
        writer.append(event.cursor, &serde_json::to_vec(event)?)?;
    }
    writer.sync()?;
    drop(writer);
    fs::rename(&temporary, path).with_context(|| {
        format!(
            "replace compatibility journal {} from canonical raw storage",
            path.display()
        )
    })?;
    service_durability::sync_parent_dir(path)?;
    Ok(())
}

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
    /// Compatibility alias for `raw_cursor`.
    pub cursor: u64,
    pub raw_cursor: u64,
    pub commit_index: u64,
    pub duplicate: bool,
}

impl AppendResult {
    fn with_commit_index(mut self, commit_index: u64) -> Self {
        self.commit_index = commit_index;
        self
    }
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
    storage: storage::RawStorage,
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
        let storage = storage::RawStorage::open(data_dir)?;
        let mut state = JournalState::default();

        if snapshot_path.exists() {
            let snapshot: durability::JournalSnapshot = serde_json::from_slice(
                &fs::read(&snapshot_path)
                    .with_context(|| format!("read snapshot {}", snapshot_path.display()))?,
            )
            .with_context(|| format!("decode snapshot {}", snapshot_path.display()))?;
            Self::replace_state(&mut state, snapshot.events)?;
        }

        let mut compatibility_rows = Vec::new();
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
            compatibility_rows.push(stored.clone());
            if state
                .cursors_by_event_id
                .contains_key(&stored.event.event_id)
            {
                continue;
            }
            Self::insert_recovered(&mut state, stored)?;
        }

        for stored in storage.recovered_events()? {
            if let Some(cursor) = state
                .cursors_by_event_id
                .get(&stored.event.event_id)
                .copied()
            {
                if cursor != stored.cursor {
                    bail!(
                        "raw storage event {} has cursor {}, but compatibility state has {cursor}",
                        stored.event.event_id,
                        stored.cursor
                    );
                }
                continue;
            }
            Self::insert_recovered(&mut state, stored)?;
        }

        // Adopt retained v1/snapshot data into the canonical sharded plane.
        // Segment append is cursor-idempotent, so this is safe on every open.
        for stored in &state.events {
            storage.append(stored)?;
        }

        let compatibility_identity = compatibility_rows
            .iter()
            .map(|stored| (stored.cursor, stored.event.event_id.as_str()))
            .collect::<HashSet<_>>();
        let canonical_identity = state
            .events
            .iter()
            .map(|stored| (stored.cursor, stored.event.event_id.as_str()))
            .collect::<HashSet<_>>();
        if compatibility_identity != canonical_identity {
            rewrite_compatibility_journal(&journal_path, &state.events)?;
        }

        let accepted = state.events.len() as u64;
        let journal = Self {
            snapshot_path,
            storage,
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

    fn append_with_cursor(
        &self,
        expected_cursor: Option<u64>,
        event: EventEnvelope,
    ) -> Result<AppendResult> {
        let mut event = self.govern_event(event)?;
        let mut state = self.state.write().expect("journal state lock poisoned");
        if let Some(cursor) = state.cursors_by_event_id.get(&event.event_id).copied() {
            self.duplicates.incr();
            return Ok(AppendResult {
                event_id: event.event_id,
                cursor,
                raw_cursor: cursor,
                commit_index: cursor,
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
        self.storage
            .externalize_event(&mut event)
            .context("durably externalize raw event payload")?;
        event.validate()?;
        let stored = StoredEvent {
            cursor,
            acknowledged_at: now_rfc3339(),
            event,
        };
        let encoded = serde_json::to_vec(&stored).context("encode raw event")?;
        self.storage
            .append(&stored)
            .context("CRC-frame and fsync sharded raw event before acknowledgement")?;
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
            raw_cursor: cursor,
            commit_index: cursor,
            duplicate: false,
        })
    }

    pub fn govern_event(&self, event: EventEnvelope) -> Result<EventEnvelope> {
        self.governance.govern(event)
    }

    pub fn storage(&self) -> &storage::RawStorage {
        &self.storage
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
        for event in &events {
            self.storage
                .append(event)
                .context("restore snapshot event into canonical raw storage")?;
        }
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
                raw_cursor: cursor,
                commit_index: cursor,
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
    state_machine: Arc<durability::SiftStateMachine>,
    local_command: Arc<tokio::sync::Mutex<()>>,
    projections: Arc<projection::ProjectionRuntime>,
    admission: Arc<ingest::AdmissionController>,
}

impl ServiceState {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_ingest_limits(data_dir, ingest::IngestLimits::from_env()?)
    }

    pub fn open_with_ingest_limits(
        data_dir: impl AsRef<Path>,
        limits: ingest::IngestLimits,
    ) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let journal = Arc::new(DurableJournal::open(data_dir)?);
        let state_machine = Arc::new(durability::SiftStateMachine::open(
            data_dir,
            journal.clone(),
        )?);
        let raft = if raft_host::replica_mode() {
            let topology =
                raft_host::ClusterTopology::from_env("sift", "sift-peer", 7380, "SIFT_PEERS")?;
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
                state_machine.clone() as Arc<dyn raft_host::RaftStateMachine>,
                raft_host::HostConfig::default(),
            )))
        } else {
            None
        };
        Ok(Self {
            projections: Arc::new(projection::ProjectionRuntime::open(
                data_dir,
                journal.clone(),
            )?),
            journal,
            draining: Arc::new(AtomicBool::new(false)),
            raft,
            state_machine,
            local_command: Arc::new(tokio::sync::Mutex::new(())),
            admission: Arc::new(ingest::AdmissionController::new(limits)?),
        })
    }

    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::Release);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::Acquire)
    }

    pub fn journal(&self) -> &DurableJournal {
        &self.journal
    }

    pub fn projections(&self) -> &projection::ProjectionRuntime {
        &self.projections
    }

    /// Start the one in-process projection worker owned by the Sift service.
    /// The worker has no listener, WAL, or Raft group of its own and can be
    /// stopped after HTTP drain during graceful shutdown.
    pub fn start_projection_worker(&self) -> ProjectionWorker {
        let projections = self.projections.clone();
        let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    changed = shutdown_rx.changed() => {
                        if changed.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        let runtime = projections.clone();
                        match tokio::task::spawn_blocking(move || {
                            for name in runtime.projection_names() {
                                runtime.catch_up(&name)?;
                            }
                            anyhow::Ok(())
                        }).await {
                            Ok(Ok(_)) => {}
                            Ok(Err(error)) => tracing::warn!(%error, "projection worker catch-up failed"),
                            Err(error) => tracing::warn!(%error, "projection worker task panicked"),
                        }
                    }
                }
            }
        });
        ProjectionWorker {
            shutdown: Some(shutdown),
            task,
        }
    }

    async fn append(&self, event: EventEnvelope) -> Result<AppendResult> {
        // Govern before the Raft proposal so sensitive content never enters a
        // replicated log, even transiently. DurableJournal repeats the policy
        // idempotently at the raw boundary for direct/single-node callers.
        let event = self.journal.govern_event(event)?;
        if let Some(accepted) = self.journal.result_for(&event.event_id) {
            return Ok(accepted.with_commit_index(self.state_machine.applied_commit_index()));
        }
        let event_id = event.event_id.clone();
        let commit_index = self
            .commit_command(durability::SiftCommandV1::AppendEvent {
                event: Box::new(event),
            })
            .await?;
        let result = self
            .state_machine
            .take_append_outcome(commit_index)
            .or_else(|| {
                self.journal
                    .result_for(&event_id)
                    .map(|result| result.with_commit_index(commit_index))
            })
            .context("state-machine commit completed without applying the Sift event")?;
        let projections = self.projections.clone();
        tokio::task::spawn_blocking(move || {
            for name in projections.projection_names() {
                if let Err(error) = projections.catch_up(&name) {
                    tracing::warn!(projection = name, %error, "asynchronous projection failed");
                }
            }
        });
        Ok(result)
    }

    async fn commit_command(&self, command: durability::SiftCommandV1) -> Result<u64> {
        let bytes = serde_json::to_vec(&command).context("encode Sift state-machine command")?;
        if let Some(raft) = &self.raft {
            return raft.propose(bytes).await;
        }
        let _guard = self.local_command.lock().await;
        let index = self.state_machine.applied_commit_index() + 1;
        self.state_machine.apply_local(index, &bytes)?;
        Ok(index)
    }

    fn replay_job(&self, id: &str) -> Option<projection::ReplayJob> {
        self.state_machine.replay_job(id)
    }

    fn error_lifecycle(
        &self,
        project: &str,
        fingerprint: &str,
    ) -> Option<projection::ErrorLifecycleV1> {
        self.state_machine.error_lifecycle(project, fingerprint)
    }

    fn audit_legal_hold(&self, project: &str, id: &str) -> Option<projection::AuditLegalHoldV1> {
        self.state_machine.audit_legal_hold(project, id)
    }

    fn audit_legal_holds(&self, project: &str) -> Vec<projection::AuditLegalHoldV1> {
        self.state_machine.audit_legal_holds(project)
    }

    fn audit_export(&self, project: &str, id: &str) -> Option<projection::AuditExportManifestV1> {
        self.state_machine.audit_export(project, id)
    }

    pub fn raft_router(&self) -> Option<Router> {
        self.raft.as_ref().map(|raft| raft.router())
    }
}

pub struct ProjectionWorker {
    shutdown: Option<tokio::sync::watch::Sender<bool>>,
    task: tokio::task::JoinHandle<()>,
}

impl ProjectionWorker {
    pub async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        let _ = self.task.await;
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
    retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    projection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
    retryable: bool,
    retry_after_secs: Option<u64>,
    projection_lag: Option<Box<projection::ProjectionLag>>,
}

impl ApiError {
    fn bad_request(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error,
            message: message.into(),
            retryable: false,
            retry_after_secs: None,
            projection_lag: None,
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "journal_failure",
            message: message.into(),
            retryable: true,
            retry_after_secs: Some(1),
            projection_lag: None,
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            error: "project_forbidden",
            message: message.into(),
            retryable: false,
            retry_after_secs: None,
            projection_lag: None,
        }
    }

    fn not_found(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error,
            message: message.into(),
            retryable: false,
            retry_after_secs: None,
            projection_lag: None,
        }
    }

    fn unavailable(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error,
            message: message.into(),
            retryable: true,
            retry_after_secs: Some(1),
            projection_lag: None,
        }
    }

    fn from_admission(error: ingest::AdmissionError) -> Self {
        Self {
            status: error.status,
            error: error.code,
            message: error.message,
            retryable: error.retryable,
            retry_after_secs: error.retry_after_secs,
            projection_lag: None,
        }
    }

    fn projection_lag(lag: projection::ProjectionLag) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: "projection_lag",
            message: format!(
                "projection `{}` is at cursor {}, below required cursor {}",
                lag.projection, lag.current_cursor, lag.required_cursor
            ),
            retryable: true,
            retry_after_secs: Some(lag.retry_after_seconds),
            projection_lag: Some(Box::new(lag)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let retry_after_secs = self.retry_after_secs;
        let lag = self.projection_lag;
        let mut response = (
            self.status,
            Json(ErrorEnvelope {
                error: self.error.to_string(),
                message: if self.retryable {
                    format!("{} (retryable)", self.message)
                } else {
                    self.message
                },
                retryable: self.retryable,
                projection: lag.as_ref().map(|lag| lag.projection.clone()),
                required_cursor: lag.as_ref().map(|lag| lag.required_cursor),
                current_cursor: lag.as_ref().map(|lag| lag.current_cursor),
                retry_after_seconds: lag.as_ref().map(|lag| lag.retry_after_seconds),
            }),
        )
            .into_response();
        if let Some(seconds) = retry_after_secs {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

/// Build Sift's data-plane routes. Probe/admin routes are intentionally added
/// by `service-http` so all k8s-native services have the same shape.
pub fn router(state: Arc<ServiceState>) -> Router {
    Router::new()
        .route("/v1/events", post(ingest).get(query_events))
        .route("/v1/events:write", post(write_events))
        .route("/v1/logs", post(ingest_logs))
        .route("/v1/traces", post(ingest_traces))
        .route("/v1/metrics", post(ingest_metrics))
        .route("/v1/profiles", post(ingest_profiles))
        .route("/v1/logs:query", post(query_logs))
        .route("/v1/logs:tail", get(tail_logs))
        .route("/v1/traces/{id}", get(get_trace))
        .route("/v1/errors:query", post(query_errors))
        .route("/v1/metrics:query", post(query_metrics))
        .route("/v1/profiles:query", post(query_profiles))
        .route("/v1/audit:query", post(query_audit))
        .route("/v1/audit:export", post(export_audit))
        .route(
            "/v1/audit/holds/{id}",
            put(upsert_audit_hold).delete(release_audit_hold),
        )
        .route("/v1/errors/{fingerprint}", get(get_error_group))
        .route(
            "/v1/errors/{fingerprint}/state",
            put(transition_error_group),
        )
        .route("/v1/replay", get(replay_events))
        .route("/v1/replays", post(start_replay))
        .route("/v1/replays/{id}", get(get_replay))
        .route("/admin/backup", get(admin_backup))
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
    get,
    path = "/admin/backup",
    responses(
        (status = 200, description = "exact durable-journal snapshot bytes"),
        (status = 403, description = "wildcard admin role required", body = ErrorEnvelope),
        (status = 500, description = "snapshot serialization failed", body = ErrorEnvelope)
    )
)]
async fn admin_backup(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
) -> Result<Response, ApiError> {
    authorize_global_admin(principal.as_ref().map(|principal| &principal.0))?;
    let snapshot = state
        .journal()
        .snapshot_bytes()
        .map_err(|error| ApiError::internal(format!("create durable journal snapshot: {error}")))?;
    tracing::info!(
        event = "backup_started",
        subject = principal
            .as_ref()
            .and_then(|principal| principal.0.subject())
            .unwrap_or("open-auth"),
        bytes = snapshot.len(),
        "durable journal snapshot exported"
    );
    let mut response = Response::new(Body::from(snapshot));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    Ok(response)
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
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<IncomingEvent>, JsonRejection>,
) -> Result<(StatusCode, Json<AppendResult>), ApiError> {
    let Json(event) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    let event = event.into_inner();
    authorize_project(principal.as_ref().map(|value| &value.0), &event.project)?;
    let result = state
        .append(event)
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

#[utoipa::path(
    post,
    path = "/v1/events:write",
    request_body = ingest::EventWriteRequest,
    responses(
        (status = 200, description = "ordered per-item durable outcomes", body = ingest::EventWriteResponse),
        (status = 400, description = "invalid batch", body = ErrorEnvelope),
        (status = 413, description = "bounded body or batch exceeded", body = ErrorEnvelope),
        (status = 429, description = "project admission quota exceeded", body = ErrorEnvelope),
        (status = 503, description = "service draining or overloaded", body = ErrorEnvelope)
    )
)]
async fn write_events(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ingest::EventWriteResponse>, ApiError> {
    let project = project_header(&headers)?;
    authorize_project(principal.as_ref().map(|value| &value.0), project)?;
    let decoded = state
        .admission
        .decode_body(&headers, body)
        .map_err(ApiError::from_admission)?;
    let request: ingest::EventWriteRequest = serde_json::from_slice(&decoded)
        .map_err(|error| ApiError::bad_request("invalid_json", error.to_string()))?;
    let _permit = state
        .admission
        .acquire(project, request.events.len(), state.is_draining())
        .map_err(ApiError::from_admission)?;
    let mut results = Vec::with_capacity(request.events.len());
    for (index, value) in request.events.into_iter().enumerate() {
        let event_id = ingest::batch::event_id_hint(&value);
        let item_bytes = serde_json::to_vec(&value)
            .map(|value| value.len())
            .unwrap_or(usize::MAX);
        if let Err(error) = state.admission.validate_event_bytes(item_bytes) {
            results.push(ingest::BatchItemResult::rejected(
                index,
                event_id,
                error.code,
                error.message,
                error.retryable,
            ));
            continue;
        }
        let event = match ingest::batch::decode_item(value, project) {
            Ok(event) => event,
            Err(error) => {
                results.push(ingest::BatchItemResult::rejected(
                    index,
                    event_id,
                    "invalid_event",
                    error.to_string(),
                    false,
                ));
                continue;
            }
        };
        if event.project != project {
            results.push(ingest::BatchItemResult::rejected(
                index,
                Some(event.event_id),
                "project_mismatch",
                format!(
                    "event project `{}` does not match admitted project `{project}`",
                    event.project
                ),
                false,
            ));
            continue;
        }
        match state.append(event).await {
            Ok(result) => results.push(ingest::BatchItemResult::accepted(
                index,
                result.event_id,
                result.cursor,
                result.duplicate,
            )),
            Err(error) => results.push(ingest::BatchItemResult::rejected(
                index,
                event_id,
                "append_failed",
                error.to_string(),
                true,
            )),
        }
    }
    Ok(Json(ingest::EventWriteResponse::from_results(results)))
}

#[utoipa::path(post, path = "/v1/logs", responses((status = 200, description = "OTLP logs export response")))]
async fn ingest_logs(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    ingest_otlp(
        state,
        principal,
        headers,
        body,
        ingest::otlp::OtlpSignal::Logs,
    )
    .await
}

#[utoipa::path(post, path = "/v1/traces", responses((status = 200, description = "OTLP traces export response")))]
async fn ingest_traces(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    ingest_otlp(
        state,
        principal,
        headers,
        body,
        ingest::otlp::OtlpSignal::Traces,
    )
    .await
}

#[utoipa::path(post, path = "/v1/metrics", responses((status = 200, description = "OTLP metrics export response")))]
async fn ingest_metrics(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    ingest_otlp(
        state,
        principal,
        headers,
        body,
        ingest::otlp::OtlpSignal::Metrics,
    )
    .await
}

#[utoipa::path(post, path = "/v1/profiles", responses((status = 200, description = "OTLP profiles export response")))]
async fn ingest_profiles(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    ingest_otlp(
        state,
        principal,
        headers,
        body,
        ingest::otlp::OtlpSignal::Profiles,
    )
    .await
}

async fn ingest_otlp(
    state: Arc<ServiceState>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
    signal: ingest::otlp::OtlpSignal,
) -> Result<Response, ApiError> {
    let project = project_header(&headers)?;
    authorize_project(principal.as_ref().map(|value| &value.0), project)?;
    let media = ingest::otlp::OtlpMediaType::parse(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
    )
    .map_err(|error| ApiError::bad_request("unsupported_content_type", error.to_string()))?;
    let decoded_body = state
        .admission
        .decode_body(&headers, body)
        .map_err(ApiError::from_admission)?;
    let decoded = ingest::otlp::decode(signal, media, &decoded_body, project)
        .map_err(|error| ApiError::bad_request("invalid_otlp", error.to_string()))?;
    let _permit = state
        .admission
        .acquire(project, decoded.item_count(), state.is_draining())
        .map_err(ApiError::from_admission)?;
    let mut rejected = 0usize;
    let mut messages = Vec::new();
    for item in decoded.items {
        let event = match item {
            Ok(event) => event,
            Err(error) => {
                rejected += 1;
                if messages.len() < 8 {
                    messages.push(error.message);
                }
                continue;
            }
        };
        if event.project != project {
            rejected += 1;
            if messages.len() < 8 {
                messages.push(format!(
                    "event project `{}` does not match admitted project `{project}`",
                    event.project
                ));
            }
            continue;
        }
        let event_bytes = serde_json::to_vec(&event)
            .map(|value| value.len())
            .unwrap_or(usize::MAX);
        if let Err(error) = state.admission.validate_event_bytes(event_bytes) {
            rejected += 1;
            if messages.len() < 8 {
                messages.push(error.message);
            }
            continue;
        }
        if let Err(error) = state.append(event).await {
            rejected += 1;
            if messages.len() < 8 {
                messages.push(format!("durable append failed: {error}"));
            }
        }
    }
    let encoded = ingest::otlp::encode_response(signal, media, rejected, &messages)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, encoded.content_type)
        .body(Body::from(encoded.body))
        .map_err(|error| ApiError::internal(error.to_string()))
}

fn project_header(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("x-sift-project")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ApiError::bad_request(
                "missing_project",
                "x-sift-project is required for bounded ingest",
            )
        })
}

fn authorize_project(principal: Option<&RoleMapPrincipal>, project: &str) -> Result<(), ApiError> {
    authorize_project_role(principal, project, Role::Write)
}

fn authorize_project_read(
    principal: Option<&RoleMapPrincipal>,
    project: &str,
) -> Result<(), ApiError> {
    authorize_project_role(principal, project, Role::Read)
}

fn authorize_global_admin(principal: Option<&RoleMapPrincipal>) -> Result<(), ApiError> {
    match principal {
        None | Some(RoleMapPrincipal::Open) => Ok(()),
        Some(principal) => principal.ensure("*", Role::Admin).map_err(|denied| {
            ApiError::forbidden(format!(
                "subject `{}` lacks wildcard admin access required for journal backup",
                denied.subject
            ))
        }),
    }
}

fn authorize_project_role(
    principal: Option<&RoleMapPrincipal>,
    project: &str,
    role: Role,
) -> Result<(), ApiError> {
    match principal {
        None | Some(RoleMapPrincipal::Open) => Ok(()),
        Some(principal) => principal.ensure(project, role).map_err(|denied| {
            ApiError::forbidden(format!(
                "subject `{}` lacks {:?} access to project `{}`",
                denied.subject, denied.needed, denied.resource
            ))
        }),
    }
}

const LOG_QUERY_PROJECTION_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

#[utoipa::path(
    post,
    path = "/v1/logs:query",
    request_body = projection::LogQuery,
    responses(
        (status = 200, description = "stable typed log page", body = projection::LogPage),
        (status = 400, description = "invalid log query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn query_logs(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<projection::LogQuery>, JsonRejection>,
) -> Result<Json<projection::LogPage>, ApiError> {
    let Json(query) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    query_logs_page(state, principal.as_ref().map(|value| &value.0), query).await
}

#[utoipa::path(
    get,
    path = "/v1/logs:tail",
    params(
        ("project" = String, Query, description = "authorized project"),
        ("after_cursor" = Option<u64>, Query, description = "exclusive raw cursor"),
        ("min_cursor" = Option<u64>, Query, description = "read-your-write projection cursor"),
        ("limit" = Option<usize>, Query, description = "bounded page size, maximum 1000")
    ),
    responses(
        (status = 200, description = "bounded cursor-resumable log page", body = projection::LogPage),
        (status = 400, description = "invalid tail query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn tail_logs(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    Query(query): Query<projection::LogQuery>,
) -> Result<Json<projection::LogPage>, ApiError> {
    query_logs_page(state, principal.as_ref().map(|value| &value.0), query).await
}

async fn query_logs_page(
    state: Arc<ServiceState>,
    principal: Option<&RoleMapPrincipal>,
    query: projection::LogQuery,
) -> Result<Json<projection::LogPage>, ApiError> {
    authorize_project_read(principal, &query.project)?;
    state
        .projections
        .catch_up(projection::PROJECTION_LOGGING_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let required = query.min_cursor.unwrap_or(0);
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_LOGGING_STORE,
            required,
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let mut page = state
        .projections
        .query_logs(&query)
        .map_err(|error| ApiError::bad_request("invalid_log_query", error.to_string()))?;
    page.projection_cursor = projection_cursor;
    Ok(Json(page))
}

#[derive(Debug, Deserialize)]
struct HttpTraceQuery {
    project: String,
    min_cursor: Option<u64>,
}

#[utoipa::path(
    get,
    path = "/v1/traces/{id}",
    params(
        ("id" = String, Path, description = "trace id"),
        ("project" = String, Query, description = "authorized project"),
        ("min_cursor" = Option<u64>, Query, description = "read-your-write projection cursor")
    ),
    responses(
        (status = 200, description = "complete or explicitly partial trace", body = projection::TraceResultV1),
        (status = 400, description = "invalid trace query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 404, description = "trace not found", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn get_trace(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<HttpTraceQuery>,
) -> Result<Json<projection::TraceResultV1>, ApiError> {
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    if id.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_trace_id",
            "trace id must not be empty",
        ));
    }
    state
        .projections
        .catch_up(projection::PROJECTION_TRACE_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_TRACE_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let mut trace = state
        .projections
        .get_trace(&query.project, &id)
        .map_err(|error| ApiError::bad_request("invalid_trace_query", error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                "trace_not_found",
                format!("trace `{id}` was not found in project `{}`", query.project),
            )
        })?;
    trace.projection_cursor = projection_cursor;
    Ok(Json(trace))
}

#[utoipa::path(
    post,
    path = "/v1/errors:query",
    request_body = projection::ErrorQuery,
    responses(
        (status = 200, description = "stable error group page", body = projection::ErrorPage),
        (status = 400, description = "invalid error query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn query_errors(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<projection::ErrorQuery>, JsonRejection>,
) -> Result<Json<projection::ErrorPage>, ApiError> {
    let Json(query) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    state
        .projections
        .catch_up(projection::PROJECTION_ERROR_REPORT_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_ERROR_REPORT_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let requested_state = query.state;
    let mut page = state
        .projections
        .query_errors(&query)
        .map_err(|error| ApiError::bad_request("invalid_error_query", error.to_string()))?;
    let now = Utc::now();
    page.groups = page
        .groups
        .into_iter()
        .map(|group| {
            let lifecycle = state.error_lifecycle(&group.project, &group.fingerprint);
            group.apply_lifecycle(lifecycle.as_ref(), now)
        })
        .filter(|group| requested_state.is_none_or(|requested| group.state == requested))
        .collect();
    page.next_cursor = page
        .groups
        .last()
        .map(|group| group.last_cursor)
        .unwrap_or(query.after_cursor);
    page.projection_cursor = projection_cursor;
    Ok(Json(page))
}

#[utoipa::path(
    post,
    path = "/v1/metrics:query",
    request_body = projection::MetricQuery,
    responses(
        (status = 200, description = "stable typed metric series page", body = projection::MetricPage),
        (status = 400, description = "invalid metric query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn query_metrics(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<projection::MetricQuery>, JsonRejection>,
) -> Result<Json<projection::MetricPage>, ApiError> {
    let Json(query) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    state
        .projections
        .catch_up(projection::PROJECTION_METRIC_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_METRIC_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let mut page = state
        .projections
        .query_metrics(&query)
        .map_err(|error| ApiError::bad_request("invalid_metric_query", error.to_string()))?;
    page.projection_cursor = projection_cursor;
    Ok(Json(page))
}

#[utoipa::path(
    post,
    path = "/v1/profiles:query",
    request_body = projection::ProfileQuery,
    responses(
        (status = 200, description = "profile list or deterministic analysis", body = projection::ProfilePage),
        (status = 400, description = "invalid profile query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn query_profiles(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<projection::ProfileQuery>, JsonRejection>,
) -> Result<Json<projection::ProfilePage>, ApiError> {
    let Json(query) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    state
        .projections
        .catch_up(projection::PROJECTION_PROFILE_STORE)
        .map_err(|error| ApiError::bad_request("invalid_profile_store", error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_PROFILE_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let mut page = state
        .projections
        .query_profiles(&query, Utc::now())
        .map_err(|error| ApiError::bad_request("invalid_profile_query", error.to_string()))?;
    page.projection_cursor = projection_cursor;
    Ok(Json(page))
}

#[utoipa::path(
    post,
    path = "/v1/audit:query",
    request_body = projection::AuditQuery,
    responses(
        (status = 200, description = "immutable retained audit/change timeline", body = projection::AuditPage),
        (status = 400, description = "invalid audit query", body = ErrorEnvelope),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn query_audit(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<projection::AuditQuery>, JsonRejection>,
) -> Result<Json<projection::AuditPage>, ApiError> {
    let Json(query) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    let page = query_audit_page(&state, &query).await?;
    Ok(Json(page))
}

async fn query_audit_page(
    state: &ServiceState,
    query: &projection::AuditQuery,
) -> Result<projection::AuditPage, ApiError> {
    state
        .projections
        .catch_up(projection::PROJECTION_AUDIT_CHANGE_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_AUDIT_CHANGE_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let holds = state.audit_legal_holds(&query.project);
    let mut page = state
        .projections
        .query_audit(query, &holds, Utc::now())
        .map_err(|error| ApiError::bad_request("invalid_audit_query", error.to_string()))?;
    page.projection_cursor = projection_cursor;
    Ok(page)
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct AuditLegalHoldRequest {
    start_time: String,
    end_time: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct HttpAuditControlQuery {
    project: String,
}

#[utoipa::path(
    put,
    path = "/v1/audit/holds/{id}",
    request_body = AuditLegalHoldRequest,
    params(
        ("id" = String, Path, description = "stable legal-hold id"),
        ("project" = String, Query, description = "administered project")
    ),
    responses(
        (status = 200, description = "durably active legal hold", body = projection::AuditLegalHoldV1),
        (status = 400, description = "invalid legal hold", body = ErrorEnvelope),
        (status = 403, description = "project admin denied", body = ErrorEnvelope)
    )
)]
async fn upsert_audit_hold(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<HttpAuditControlQuery>,
    payload: Result<Json<AuditLegalHoldRequest>, JsonRejection>,
) -> Result<Json<projection::AuditLegalHoldV1>, ApiError> {
    authorize_project_role(
        principal.as_ref().map(|value| &value.0),
        &query.project,
        Role::Admin,
    )?;
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    let hold = projection::AuditLegalHoldV1 {
        id,
        project: query.project,
        start_time: request.start_time,
        end_time: request.end_time,
        reason: request.reason,
        actor: principal
            .as_ref()
            .and_then(|principal| principal.0.subject())
            .unwrap_or("open")
            .into(),
        active: true,
        updated_at: now_rfc3339(),
        commit_index: 0,
    };
    commit_audit_hold(&state, hold).await.map(Json)
}

#[utoipa::path(
    delete,
    path = "/v1/audit/holds/{id}",
    params(
        ("id" = String, Path, description = "stable legal-hold id"),
        ("project" = String, Query, description = "administered project")
    ),
    responses(
        (status = 200, description = "durably released legal hold", body = projection::AuditLegalHoldV1),
        (status = 403, description = "project admin denied", body = ErrorEnvelope),
        (status = 404, description = "legal hold not found", body = ErrorEnvelope)
    )
)]
async fn release_audit_hold(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<HttpAuditControlQuery>,
) -> Result<Json<projection::AuditLegalHoldV1>, ApiError> {
    authorize_project_role(
        principal.as_ref().map(|value| &value.0),
        &query.project,
        Role::Admin,
    )?;
    let mut hold = state.audit_legal_hold(&query.project, &id).ok_or_else(|| {
        ApiError::not_found(
            "audit_hold_not_found",
            format!(
                "audit legal hold `{id}` was not found in project `{}`",
                query.project
            ),
        )
    })?;
    hold.active = false;
    hold.actor = principal
        .as_ref()
        .and_then(|principal| principal.0.subject())
        .unwrap_or("open")
        .into();
    hold.updated_at = now_rfc3339();
    hold.commit_index = 0;
    commit_audit_hold(&state, hold).await.map(Json)
}

async fn commit_audit_hold(
    state: &ServiceState,
    hold: projection::AuditLegalHoldV1,
) -> Result<projection::AuditLegalHoldV1, ApiError> {
    state
        .commit_command(durability::SiftCommandV1::UpsertAuditLegalHold {
            hold: Box::new(hold.clone()),
        })
        .await
        .map_err(|error| ApiError::bad_request("invalid_audit_hold", error.to_string()))?;
    state
        .audit_legal_hold(&hold.project, &hold.id)
        .ok_or_else(|| ApiError::internal("committed audit legal hold was not available"))
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct AuditExportRequest {
    id: String,
    query: projection::AuditQuery,
}

#[utoipa::path(
    post,
    path = "/v1/audit:export",
    request_body = AuditExportRequest,
    responses(
        (status = 200, description = "bounded controlled export and committed manifest", body = projection::AuditExportResponseV1),
        (status = 400, description = "invalid or duplicate export", body = ErrorEnvelope),
        (status = 403, description = "project admin denied", body = ErrorEnvelope)
    )
)]
async fn export_audit(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<AuditExportRequest>, JsonRejection>,
) -> Result<Json<projection::AuditExportResponseV1>, ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    authorize_project_role(
        principal.as_ref().map(|value| &value.0),
        &request.query.project,
        Role::Admin,
    )?;
    if request.id.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_export_id",
            "audit export id must not be empty",
        ));
    }
    let page = query_audit_page(&state, &request.query).await?;
    let actor = principal
        .as_ref()
        .and_then(|principal| principal.0.subject())
        .unwrap_or("open")
        .to_string();
    let manifest = projection::AuditExportManifestV1 {
        id: request.id,
        project: request.query.project.clone(),
        start_time: request.query.start_time.clone(),
        end_time: request.query.end_time.clone(),
        record_count: page.records.len() as u64,
        content_sha256: projection::export_content_sha256(&page.records)
            .map_err(|error| ApiError::internal(error.to_string()))?,
        actor,
        exported_at: now_rfc3339(),
        commit_index: 0,
    };
    state
        .commit_command(durability::SiftCommandV1::RecordAuditExport {
            export: Box::new(manifest.clone()),
        })
        .await
        .map_err(|error| ApiError::bad_request("invalid_audit_export", error.to_string()))?;
    let manifest = state
        .audit_export(&manifest.project, &manifest.id)
        .ok_or_else(|| ApiError::internal("committed audit export was not available"))?;
    Ok(Json(projection::AuditExportResponseV1 {
        manifest,
        records: page.records,
    }))
}

#[derive(Debug, Deserialize)]
struct HttpErrorGroupQuery {
    project: String,
    min_cursor: Option<u64>,
}

#[utoipa::path(
    get,
    path = "/v1/errors/{fingerprint}",
    params(
        ("fingerprint" = String, Path, description = "versioned error fingerprint"),
        ("project" = String, Query, description = "authorized project"),
        ("min_cursor" = Option<u64>, Query, description = "read-your-write projection cursor")
    ),
    responses(
        (status = 200, description = "error group detail", body = projection::ErrorGroupV1),
        (status = 403, description = "project read denied", body = ErrorEnvelope),
        (status = 404, description = "error group not found", body = ErrorEnvelope),
        (status = 503, description = "projection has not reached min_cursor", body = ErrorEnvelope)
    )
)]
async fn get_error_group(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(fingerprint): AxumPath<String>,
    Query(query): Query<HttpErrorGroupQuery>,
) -> Result<Json<projection::ErrorGroupV1>, ApiError> {
    authorize_project_read(principal.as_ref().map(|value| &value.0), &query.project)?;
    state
        .projections
        .catch_up(projection::PROJECTION_ERROR_REPORT_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let projection_cursor = state
        .projections
        .wait_for_min_cursor(
            projection::PROJECTION_ERROR_REPORT_STORE,
            query.min_cursor.unwrap_or(0),
            LOG_QUERY_PROJECTION_WAIT,
        )
        .await
        .map_err(ApiError::projection_lag)?;
    let lifecycle = state.error_lifecycle(&query.project, &fingerprint);
    let mut group = state
        .projections
        .get_error_group(&query.project, &fingerprint)
        .map_err(|error| ApiError::bad_request("invalid_error_query", error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                "error_group_not_found",
                format!(
                    "error group `{fingerprint}` was not found in project `{}`",
                    query.project
                ),
            )
        })?
        .apply_lifecycle(lifecycle.as_ref(), Utc::now());
    group.projection_cursor = projection_cursor;
    Ok(Json(group))
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
struct ErrorLifecycleRequest {
    state: projection::ErrorLifecycleState,
    #[serde(default)]
    muted_until: Option<String>,
    #[serde(default)]
    reason: Option<String>,
}

#[utoipa::path(
    put,
    path = "/v1/errors/{fingerprint}/state",
    request_body = ErrorLifecycleRequest,
    params(
        ("fingerprint" = String, Path, description = "versioned error fingerprint"),
        ("project" = String, Query, description = "authorized project")
    ),
    responses(
        (status = 200, description = "durably committed error lifecycle", body = projection::ErrorLifecycleV1),
        (status = 400, description = "invalid lifecycle transition", body = ErrorEnvelope),
        (status = 403, description = "project write denied", body = ErrorEnvelope),
        (status = 404, description = "error group not found", body = ErrorEnvelope)
    )
)]
async fn transition_error_group(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(fingerprint): AxumPath<String>,
    Query(query): Query<HttpErrorGroupQuery>,
    payload: Result<Json<ErrorLifecycleRequest>, JsonRejection>,
) -> Result<Json<projection::ErrorLifecycleV1>, ApiError> {
    authorize_project(principal.as_ref().map(|value| &value.0), &query.project)?;
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    state
        .projections
        .catch_up(projection::PROJECTION_ERROR_REPORT_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let group = state
        .projections
        .get_error_group(&query.project, &fingerprint)
        .map_err(|error| ApiError::bad_request("invalid_error_query", error.to_string()))?
        .ok_or_else(|| {
            ApiError::not_found(
                "error_group_not_found",
                format!(
                    "error group `{fingerprint}` was not found in project `{}`",
                    query.project
                ),
            )
        })?;
    validate_error_lifecycle_request(&request)?;
    let lifecycle = projection::ErrorLifecycleV1 {
        project: query.project,
        fingerprint,
        state: request.state,
        muted_until: request.muted_until,
        actor: principal
            .as_ref()
            .and_then(|principal| principal.0.subject())
            .unwrap_or("open")
            .to_string(),
        reason: request.reason,
        occurrence_cursor: group.last_cursor,
        updated_at: now_rfc3339(),
        commit_index: 0,
    };
    state
        .commit_command(durability::SiftCommandV1::TransitionErrorGroup {
            lifecycle: Box::new(lifecycle.clone()),
        })
        .await
        .map_err(|error| ApiError::internal(error.to_string()))?;
    state
        .error_lifecycle(&lifecycle.project, &lifecycle.fingerprint)
        .map(Json)
        .ok_or_else(|| ApiError::internal("committed error lifecycle was not available"))
}

fn validate_error_lifecycle_request(request: &ErrorLifecycleRequest) -> Result<(), ApiError> {
    match (request.state, request.muted_until.as_deref()) {
        (projection::ErrorLifecycleState::Muted, Some(until)) => {
            let until = DateTime::parse_from_rfc3339(until).map_err(|_| {
                ApiError::bad_request("invalid_muted_until", "muted_until must be RFC3339")
            })?;
            if until.with_timezone(&Utc) <= Utc::now() {
                return Err(ApiError::bad_request(
                    "invalid_muted_until",
                    "muted_until must be in the future",
                ));
            }
        }
        (projection::ErrorLifecycleState::Muted, None) => {
            return Err(ApiError::bad_request(
                "missing_muted_until",
                "muted state requires muted_until",
            ));
        }
        (_, Some(_)) => {
            return Err(ApiError::bad_request(
                "unexpected_muted_until",
                "only muted state accepts muted_until",
            ));
        }
        (_, None) => {}
    }
    Ok(())
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

#[derive(Debug, Deserialize, ToSchema)]
struct StartReplayRequest {
    projection: String,
}

static REPLAY_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[utoipa::path(
    post,
    path = "/v1/replays",
    request_body = StartReplayRequest,
    responses(
        (status = 202, description = "durable replay scheduled", body = projection::ReplayJob),
        (status = 400, description = "unknown projection", body = ErrorEnvelope),
        (status = 503, description = "state-machine mutation unavailable", body = ErrorEnvelope)
    )
)]
async fn start_replay(
    State(state): State<Arc<ServiceState>>,
    payload: Result<Json<StartReplayRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<projection::ReplayJob>), ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    if !state.projections.has_projection(&request.projection) {
        return Err(ApiError::bad_request(
            "unknown_projection",
            format!("projection `{}` is not registered", request.projection),
        ));
    }
    let id = format!(
        "replay-{}-{}",
        Utc::now().timestamp_millis(),
        REPLAY_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let job =
        projection::ReplayJob::pending(id.clone(), request.projection, state.journal.last_cursor());
    state
        .commit_command(durability::SiftCommandV1::UpsertReplayJob { job: Box::new(job) })
        .await
        .map_err(|error| ApiError::unavailable("replay_commit_failed", error.to_string()))?;
    let durable = state
        .replay_job(&id)
        .context("replay state missing after durable commit")
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let replay_state = state.clone();
    let replay_id = id.clone();
    tokio::spawn(async move {
        if let Err(error) = run_replay(replay_state, &replay_id).await {
            tracing::error!(replay_id, %error, "projection replay task failed");
        }
    });
    Ok((StatusCode::ACCEPTED, Json(durable)))
}

async fn run_replay(state: Arc<ServiceState>, id: &str) -> Result<()> {
    let mut job = state
        .replay_job(id)
        .with_context(|| format!("replay job {id} disappeared"))?;
    job.mark_running();
    state
        .commit_command(durability::SiftCommandV1::UpsertReplayJob { job: Box::new(job) })
        .await?;

    let projection_name = state
        .replay_job(id)
        .with_context(|| format!("replay job {id} disappeared after running transition"))?
        .projection;
    let runtime = state.projections.clone();
    let result = tokio::task::spawn_blocking(move || runtime.rebuild_and_compare(&projection_name))
        .await
        .context("projection replay task panicked")?;

    let mut job = state
        .replay_job(id)
        .with_context(|| format!("replay job {id} disappeared before terminal transition"))?;
    match result {
        Ok(comparison) if comparison.equal => job.mark_completed(comparison),
        Ok(comparison) => {
            let live = comparison.live_digest.clone();
            let rebuilt = comparison.rebuilt_digest.clone();
            job.mark_completed(comparison);
            job.state = projection::ReplayState::Failed;
            job.error = Some(format!(
                "projection rebuild digest mismatch: live={live}, rebuilt={rebuilt}"
            ));
        }
        Err(error) => job.mark_failed(error.to_string()),
    }
    state
        .commit_command(durability::SiftCommandV1::UpsertReplayJob { job: Box::new(job) })
        .await?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/v1/replays/{id}",
    params(("id" = String, Path, description = "durable replay id")),
    responses(
        (status = 200, description = "durable replay status", body = projection::ReplayJob),
        (status = 404, description = "replay not found", body = ErrorEnvelope)
    )
)]
async fn get_replay(
    State(state): State<Arc<ServiceState>>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<projection::ReplayJob>, ApiError> {
    state
        .replay_job(&id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found("replay_not_found", format!("replay `{id}` not found")))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        ingest,
        write_events,
        ingest_logs,
        ingest_traces,
        ingest_metrics,
        ingest_profiles,
        query_logs,
        tail_logs,
        get_trace,
        query_errors,
        query_metrics,
        query_profiles,
        query_audit,
        upsert_audit_hold,
        release_audit_hold,
        export_audit,
        get_error_group,
        transition_error_group,
        query_events,
        replay_events,
        start_replay,
        get_replay,
        admin_backup
    ),
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
        ingest::EventWriteRequest,
        ingest::EventWriteResponse,
        ingest::BatchItemResult,
        ingest::BatchOutcome,
        ingest::IngestErrorDetail,
        projection::ProjectionLag,
        projection::LogRecordV1,
        projection::LogQuery,
        projection::LogPage,
        projection::SpanLinkV1,
        projection::SpanEventV1,
        projection::SpanRecordV1,
        projection::TraceResultV1,
        projection::ErrorOccurrenceV1,
        projection::ErrorGroupV1,
        projection::ErrorQuery,
        projection::ErrorPage,
        projection::ErrorLifecycleState,
        projection::ErrorLifecycleV1,
        ErrorLifecycleRequest,
        projection::HistogramKind,
        projection::MetricHistogramV1,
        projection::MetricPointV1,
        projection::MetricChunkV1,
        projection::MetricRollupV1,
        projection::MetricAggregation,
        projection::MetricQuery,
        projection::MetricSeriesResultV1,
        projection::MetricPage,
        projection::ProfileMappingV1,
        projection::ProfileFunctionV1,
        projection::ProfileLineV1,
        projection::ProfileLocationV1,
        projection::ProfileStackSampleV1,
        projection::ProfileRecordV1,
        projection::ProfileView,
        projection::ProfileQuery,
        projection::ProfileFlamegraphEntryV1,
        projection::ProfileFunctionValueV1,
        projection::ProfilePage,
        projection::AuditChangeRecordV1,
        projection::AuditQuery,
        projection::AuditPage,
        projection::AuditLegalHoldV1,
        projection::AuditExportManifestV1,
        projection::AuditExportResponseV1,
        AuditLegalHoldRequest,
        AuditExportRequest,
        projection::ReplayJob,
        projection::ReplayState,
        StartReplayRequest,
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

// HANDWRITE-BEGIN gap="sift-service-core" tracker="1576" reason="Implement the versioned operational-event envelope, durable raw journal, idempotency, query, and replay core."
//! Sift's service core for logs, metrics, and traces. The canonical per-signal
//! WAL is fsynced before acknowledgement. Rebuildable indexes are never a
//! second source of truth.

pub mod api;
pub mod auth;
pub mod backup;
pub mod collector;
pub mod deploy;
pub mod durability;
pub mod event;
pub mod grpc;
pub mod ingest;
pub mod mcp;
pub mod operator;
pub mod projection;
pub mod prometheus;
pub mod proxy;
pub mod storage;

pub use event::{
    decode_event_json, AttributeValue, ContentBlobRef, EventEnvelope, GovernancePolicy,
    GovernancePolicySet, IncomingEvent, InstrumentationScope, MetricExemplar, MetricPoint,
    MetricTemporality, OperationalEventV2, SignalKind, EVENT_SCHEMA_URL, EVENT_SCHEMA_VERSION,
};

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use anyhow::{bail, Context, Result};
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{rejection::JsonRejection, Extension, Path as AxumPath, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use metrics_prometheus::{Counter, Sample};
use serde::{Deserialize, Deserializer, Serialize};
use service_auth::{Role, RoleMapPrincipal};
use service_http::{DetailedErrorEnvelope as ErrorEnvelope, ProjectionMetadata};
use sha2::{Digest, Sha256};
use tower::ServiceExt as _;
use utoipa::{OpenApi, ToSchema};

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

const DEFAULT_RESIDENT_JOURNAL_EVENTS: usize = 100_000;
const RECOVERY_PAGE_EVENTS: usize = 10_000;

#[derive(Default)]
struct JournalState {
    recent_events: VecDeque<StoredEvent>,
    recent_cursors_by_event_id: HashMap<String, u64>,
    last_cursor: u64,
    total_events: u64,
}

fn canonical_recovery_page(
    storage: &storage::RawStorage,
    wal: &storage::SignalWal,
    archived: storage::archive::ArchiveWatermarks,
    after: u64,
    repair_segments: bool,
) -> Result<Vec<StoredEvent>> {
    let mut candidates = BTreeMap::<u64, (Option<StoredEvent>, Option<StoredEvent>)>::new();
    for event in storage.query_events(None, after, RECOVERY_PAGE_EVENTS)? {
        let cursor = event.cursor;
        if candidates
            .entry(cursor)
            .or_default()
            .0
            .replace(event)
            .is_some()
        {
            bail!("segments contain duplicate cursor {cursor}");
        }
    }
    for event in wal.query_events(after, RECOVERY_PAGE_EVENTS)? {
        let cursor = event.cursor;
        if candidates
            .entry(cursor)
            .or_default()
            .1
            .replace(event)
            .is_some()
        {
            bail!("WAL contains duplicate cursor {cursor}");
        }
    }

    candidates
        .into_iter()
        .take(RECOVERY_PAGE_EVENTS)
        .map(
            |(cursor, (segment, wal_event))| match (segment, wal_event) {
                (Some(segment), Some(wal_event)) => {
                    if segment != wal_event {
                        bail!("WAL and segment disagree at cursor {cursor}");
                    }
                    Ok(wal_event)
                }
                (None, Some(wal_event)) if repair_segments => {
                    storage
                        .append(&wal_event)
                        .context("recover committed WAL event into a segment")?;
                    Ok(wal_event)
                }
                (None, Some(_)) => {
                    bail!("WAL cursor {cursor} was not recovered into a segment")
                }
                (Some(segment), None) => {
                    if !archived.covers(segment.event.signal, cursor) {
                        bail!("segment cursor {cursor} has no committed WAL or archive receipt");
                    }
                    Ok(segment)
                }
                (None, None) => unreachable!("cursor came from neither canonical source"),
            },
        )
        .collect()
}

fn rebuild_dedupe_index(
    root: &Path,
    storage: &storage::RawStorage,
    wal: &storage::SignalWal,
    dedupe: &storage::DedupeIndex,
    archived: storage::archive::ArchiveWatermarks,
    expected_count: u64,
    expected_last_cursor: u64,
) -> Result<()> {
    dedupe.reset()?;
    let mut page = Vec::with_capacity(RECOVERY_PAGE_EVENTS);
    let mut rebuilt_count = 0_u64;
    let mut rebuilt_last_cursor = 0_u64;

    let remote = storage::archive::replay_all_committed_events(root, |event| {
        rebuilt_last_cursor = rebuilt_last_cursor.max(event.cursor);
        page.push(event);
        if page.len() == RECOVERY_PAGE_EVENTS {
            append_unique_dedupe_page(dedupe, &mut page)?;
        }
        rebuilt_count = rebuilt_count.saturating_add(1);
        Ok(())
    })?;
    if !page.is_empty() {
        append_unique_dedupe_page(dedupe, &mut page)?;
    }

    let remote_watermarks = remote
        .map(|_| archived)
        .unwrap_or_else(storage::archive::ArchiveWatermarks::default);
    let mut after = 0_u64;
    loop {
        let local = canonical_recovery_page(storage, wal, archived, after, false)?;
        let Some(last) = local.last() else {
            break;
        };
        after = last.cursor;
        let mut new_events = local
            .into_iter()
            .filter(|event| !remote_watermarks.covers(event.event.signal, event.cursor))
            .collect::<Vec<_>>();
        rebuilt_count = rebuilt_count.saturating_add(new_events.len() as u64);
        rebuilt_last_cursor = rebuilt_last_cursor.max(
            new_events
                .last()
                .map(|event| event.cursor)
                .unwrap_or_default(),
        );
        append_unique_dedupe_page(dedupe, &mut new_events)?;
    }

    if rebuilt_count != expected_count || rebuilt_last_cursor > expected_last_cursor {
        bail!(
            "rebuilt dedupe index has {rebuilt_count} events through cursor {rebuilt_last_cursor}; journal head expects {expected_count} retained events through cursor {expected_last_cursor}"
        );
    }
    Ok(())
}

fn append_unique_dedupe_page(
    dedupe: &storage::DedupeIndex,
    page: &mut Vec<StoredEvent>,
) -> Result<()> {
    if page.is_empty() {
        return Ok(());
    }
    let mut page_ids = HashMap::with_capacity(page.len());
    for stored in page.iter() {
        if let Some(previous) = page_ids
            .insert(stored.event.event_id.clone(), stored.cursor)
            .or(dedupe.lookup(&stored.event.event_id)?)
        {
            bail!(
                "journal contains duplicate event_id {} at cursors {previous} and {}",
                stored.event.event_id,
                stored.cursor
            );
        }
    }
    dedupe.append_batch(page)?;
    page.clear();
    Ok(())
}

/// Append-only JSONL journal. State is updated only after `sync_data` succeeds,
/// making a successful [`append`](Self::append) acknowledgement durable.
pub struct DurableJournal {
    _layout: storage::DataLayout,
    wal: storage::SignalWal,
    storage: storage::RawStorage,
    dedupe: storage::DedupeIndex,
    state: RwLock<JournalState>,
    resident_limit: usize,
    governance: GovernancePolicySet,
    accepted: Counter,
    duplicates: Counter,
    fsyncs: Counter,
}

impl DurableJournal {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_role(data_dir, storage::StorageRole::All)
    }

    pub fn open_with_role(data_dir: impl AsRef<Path>, role: storage::StorageRole) -> Result<Self> {
        Self::open_with_governance_and_role(data_dir, GovernancePolicySet::from_env()?, role)
    }

    pub fn open_with_governance(
        data_dir: impl AsRef<Path>,
        governance: GovernancePolicySet,
    ) -> Result<Self> {
        Self::open_with_governance_and_role(data_dir, governance, storage::StorageRole::All)
    }

    pub fn open_with_governance_and_role(
        data_dir: impl AsRef<Path>,
        governance: GovernancePolicySet,
        role: storage::StorageRole,
    ) -> Result<Self> {
        Self::open_configured(data_dir, governance, role, DEFAULT_RESIDENT_JOURNAL_EVENTS)
    }

    pub fn open_with_resident_limit(
        data_dir: impl AsRef<Path>,
        resident_limit: usize,
    ) -> Result<Self> {
        Self::open_configured(
            data_dir,
            GovernancePolicySet::from_env()?,
            storage::StorageRole::All,
            resident_limit,
        )
    }

    fn open_configured(
        data_dir: impl AsRef<Path>,
        governance: GovernancePolicySet,
        role: storage::StorageRole,
        resident_limit: usize,
    ) -> Result<Self> {
        governance.validate()?;
        if resident_limit == 0 {
            bail!("resident journal event limit must be greater than zero");
        }
        let layout = storage::DataLayout::open(data_dir, role)?;
        let data_dir = layout.root().to_path_buf();
        let wal = storage::SignalWal::open(&data_dir)?;
        let storage = storage::RawStorage::open(&data_dir)?;
        let archived = storage::archive::committed_watermarks(&data_dir)?;
        let remote_retained = storage::archive::remote_retained_state(&data_dir)?;
        let mut state = JournalState::default();
        let (dedupe, dedupe_stats) = storage::DedupeIndex::open(&data_dir)?;
        let mut after = 0;
        let mut dedupe_matches = true;
        let mut local_after_remote = 0_u64;
        loop {
            let page = canonical_recovery_page(&storage, &wal, archived, after, true)?;
            let Some(last) = page.last() else {
                break;
            };
            after = last.cursor;
            for stored in page {
                if remote_retained.is_none_or(|remote| {
                    !remote.watermarks.covers(stored.event.signal, stored.cursor)
                }) {
                    local_after_remote = local_after_remote.saturating_add(1);
                }
                if dedupe_stats.count != 0
                    && dedupe.lookup(&stored.event.event_id)? != Some(stored.cursor)
                {
                    dedupe_matches = false;
                }
                Self::insert_recovered(&mut state, stored, resident_limit)?;
            }
        }

        let stored_head = storage::JournalHead::load(&data_dir)?;
        let mut head = stored_head.unwrap_or_else(|| {
            storage::JournalHead::new(
                state
                    .last_cursor
                    .max(dedupe_stats.last_cursor)
                    .max(archived.max_cursor()),
                state.total_events.max(dedupe_stats.count),
            )
        });
        head.last_cursor = head
            .last_cursor
            .max(state.last_cursor)
            .max(archived.max_cursor())
            .max(
                remote_retained
                    .map(|remote| remote.snapshot_index)
                    .unwrap_or_default(),
            );
        head.retained_events = match remote_retained {
            Some(remote) => remote
                .event_count
                .checked_add(local_after_remote)
                .context("retained event count exhausted u64")?,
            None => head.retained_events.max(state.total_events),
        };
        if head.retained_events > head.last_cursor {
            bail!("journal head retained event count exceeds the recovered cursor range");
        }

        if dedupe_stats.count != head.retained_events
            || dedupe_stats.last_cursor > head.last_cursor
            || !dedupe_matches
        {
            rebuild_dedupe_index(
                &data_dir,
                &storage,
                &wal,
                &dedupe,
                archived,
                head.retained_events,
                head.last_cursor,
            )?;
        }
        head.persist(&data_dir)?;
        state.last_cursor = head.last_cursor;
        state.total_events = head.retained_events;
        let accepted = head.retained_events;
        let journal = Self {
            _layout: layout,
            wal,
            storage,
            dedupe,
            state: RwLock::new(state),
            resident_limit,
            governance,
            accepted: Counter::new(),
            duplicates: Counter::new(),
            fsyncs: Counter::new(),
        };
        journal.accepted.add(accepted);
        Ok(journal)
    }

    pub fn append(&self, event: EventEnvelope) -> Result<AppendResult> {
        self.append_durable_batch(vec![event])?
            .pop()
            .context("single-event durable batch returned no result")
    }

    /// Apply one committed single-signal batch to the canonical WAL.
    ///
    /// The batch is encoded as one WAL frame and reaches one fsync boundary.
    /// Segment writes are rebuildable work and do not participate in the
    /// acknowledgement boundary.
    pub(crate) fn append_durable_batch(
        &self,
        events: Vec<EventEnvelope>,
    ) -> Result<Vec<AppendResult>> {
        let signal = events
            .first()
            .context("Sift durable batch must not be empty")?
            .signal;
        if events.iter().any(|event| event.signal != signal) {
            bail!("Sift durable batch must contain exactly one signal");
        }
        let mut governed = Vec::with_capacity(events.len());
        for event in events {
            let event = self.govern_event(event)?;
            event.validate()?;
            if let Some(message) = retention_rejection(&event) {
                bail!(message);
            }
            governed.push(event);
        }

        let mut state = self.state.write().expect("journal state lock poisoned");
        let mut next_cursor = state
            .last_cursor
            .checked_add(1)
            .context("Sift journal cursor exhausted u64")?;
        let mut staged = Vec::with_capacity(governed.len());
        let mut staged_cursors = HashMap::<String, u64>::new();
        let mut results = Vec::with_capacity(governed.len());

        for mut event in governed {
            if let Some(cursor) = state
                .recent_cursors_by_event_id
                .get(&event.event_id)
                .copied()
                .or_else(|| staged_cursors.get(&event.event_id).copied())
                .or(self.dedupe.lookup(&event.event_id)?)
            {
                self.duplicates.incr();
                results.push(AppendResult {
                    event_id: event.event_id,
                    cursor,
                    raw_cursor: cursor,
                    commit_index: cursor,
                    duplicate: true,
                });
                continue;
            }

            self.storage
                .externalize_event(&mut event)
                .context("durably externalize raw event payload")?;
            event.validate()?;
            let cursor = next_cursor;
            next_cursor = next_cursor
                .checked_add(1)
                .context("Sift journal cursor exhausted u64")?;
            let event_id = event.event_id.clone();
            staged_cursors.insert(event_id.clone(), cursor);
            staged.push(StoredEvent {
                cursor,
                acknowledged_at: now_rfc3339(),
                event,
            });
            results.push(AppendResult {
                event_id,
                cursor,
                raw_cursor: cursor,
                commit_index: cursor,
                duplicate: false,
            });
        }

        if !staged.is_empty() {
            self.wal
                .append_batch(&staged)
                .context("append and fsync one signal WAL batch before acknowledgement")?;
            self.fsyncs.incr();
            if let Err(error) = self.storage.append_batch(&staged) {
                tracing::warn!(
                    %error,
                    first_cursor = staged.first().map(|event| event.cursor),
                    last_cursor = staged.last().map(|event| event.cursor),
                    "deferred segment append failed; canonical WAL remains recoverable"
                );
            }
            if let Err(error) = self.dedupe.append_batch(&staged) {
                tracing::warn!(
                    %error,
                    "rebuildable dedupe index append failed; canonical WAL remains recoverable"
                );
            }
            for stored in staged {
                Self::push_resident(&mut state, stored, self.resident_limit);
            }
            storage::JournalHead::new(state.last_cursor, state.total_events)
                .persist(self.data_dir())
                .context("persist journal head before acknowledging the durable batch")?;
            self.accepted.add(staged_cursors.len() as u64);
        }
        Ok(results)
    }

    pub fn govern_event(&self, event: EventEnvelope) -> Result<EventEnvelope> {
        self.governance.govern(event)
    }

    pub fn storage(&self) -> &storage::RawStorage {
        &self.storage
    }

    pub(crate) fn compact_archived_wal(
        &self,
        watermarks: storage::archive::ArchiveWatermarks,
    ) -> Result<()> {
        self.wal.compact_through(watermarks)
    }

    pub(crate) fn evict_resident_before(&self, cutoff: DateTime<Utc>) -> Result<usize> {
        let mut state = self.state.write().expect("journal state lock poisoned");
        let before = state.recent_events.len();
        let mut retained = VecDeque::with_capacity(before);
        while let Some(event) = state.recent_events.pop_front() {
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("resident event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if occurred >= cutoff {
                retained.push_back(event);
            }
        }
        state.recent_events = retained;
        state.recent_cursors_by_event_id = state
            .recent_events
            .iter()
            .map(|event| (event.event.event_id.clone(), event.cursor))
            .collect();
        Ok(before.saturating_sub(state.recent_events.len()))
    }

    pub(crate) fn apply_expiration_head(
        &self,
        cutoff: DateTime<Utc>,
        retained_events: u64,
    ) -> Result<()> {
        self.evict_resident_before(cutoff)?;
        let last_cursor = {
            let mut state = self.state.write().expect("journal state lock poisoned");
            if retained_events > state.last_cursor {
                bail!("retained event count exceeds the journal cursor high-water mark");
            }
            state.total_events = retained_events;
            state.last_cursor
        };
        storage::JournalHead::new(last_cursor, retained_events).persist(self.data_dir())?;
        let archived = storage::archive::committed_watermarks(self.data_dir())?;
        rebuild_dedupe_index(
            self.data_dir(),
            &self.storage,
            &self.wal,
            &self.dedupe,
            archived,
            retained_events,
            last_cursor,
        )
    }

    pub(crate) fn set_restored_head(&self, last_cursor: u64, retained_events: u64) -> Result<()> {
        let mut state = self.state.write().expect("journal state lock poisoned");
        if last_cursor < state.last_cursor || retained_events != state.total_events {
            bail!("restored archive head disagrees with restored event state");
        }
        state.last_cursor = last_cursor;
        storage::JournalHead::new(last_cursor, retained_events).persist(self.data_dir())
    }

    fn insert_recovered(
        state: &mut JournalState,
        stored: StoredEvent,
        resident_limit: usize,
    ) -> Result<()> {
        stored.event.validate()?;
        if stored.cursor <= state.last_cursor {
            bail!(
                "journal cursor {} is not strictly after recovered cursor {}",
                stored.cursor,
                state.last_cursor
            );
        }
        if state
            .recent_cursors_by_event_id
            .contains_key(&stored.event.event_id)
        {
            bail!(
                "journal contains duplicate event_id {}",
                stored.event.event_id
            );
        }
        Self::push_resident(state, stored, resident_limit);
        Ok(())
    }

    fn push_resident(state: &mut JournalState, stored: StoredEvent, resident_limit: usize) {
        state.last_cursor = stored.cursor;
        state.total_events = state.total_events.saturating_add(1);
        state
            .recent_cursors_by_event_id
            .insert(stored.event.event_id.clone(), stored.cursor);
        state.recent_events.push_back(stored);
        while state.recent_events.len() > resident_limit {
            if let Some(evicted) = state.recent_events.pop_front() {
                state
                    .recent_cursors_by_event_id
                    .remove(&evicted.event.event_id);
            }
        }
    }

    pub(crate) fn last_cursor(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .last_cursor
    }

    pub(crate) fn data_dir(&self) -> &Path {
        self._layout.root()
    }

    pub(crate) fn snapshot_bounds(&self) -> (u64, u64) {
        let state = self.state.read().expect("journal state lock poisoned");
        (state.last_cursor, state.total_events)
    }

    pub fn resident_event_count(&self) -> usize {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .recent_events
            .len()
    }

    pub fn total_event_count(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .total_events
    }

    pub fn snapshot_bytes(&self) -> Result<Vec<u8>> {
        let mut snapshot = Vec::new();
        durability::write_snapshot(self, self.last_cursor(), &mut snapshot)
            .context("serialize durable journal snapshot")?;
        Ok(snapshot)
    }

    pub fn restore_snapshot_bytes(&self, bytes: &[u8]) -> Result<()> {
        let mut cursor = std::io::Cursor::new(bytes);
        durability::restore_seekable_snapshot(self, &mut cursor)
            .context("restore durable journal snapshot")?;
        Ok(())
    }

    /// Restore one globally ordered page without creating a second full-copy
    /// JSON snapshot. Cold archive restore calls this repeatedly, so resident
    /// memory stays bounded by the journal cache plus one recovery page.
    pub(crate) fn restore_stored_page(&self, events: Vec<StoredEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let mut state = self.state.write().expect("journal state lock poisoned");
        let mut previous_cursor = state.last_cursor;
        let mut page_ids = HashMap::with_capacity(events.len());
        for event in &events {
            event.event.validate()?;
            if event.cursor <= previous_cursor {
                bail!(
                    "restored journal cursor {} is not strictly after cursor {previous_cursor}",
                    event.cursor,
                );
            }
            previous_cursor = event.cursor;
            if let Some(previous) = page_ids
                .insert(event.event.event_id.clone(), event.cursor)
                .or_else(|| {
                    state
                        .recent_cursors_by_event_id
                        .get(&event.event.event_id)
                        .copied()
                })
                .or(self.dedupe.lookup(&event.event.event_id)?)
            {
                bail!(
                    "restored journal contains duplicate event_id {} at cursors {previous} and {}",
                    event.event.event_id,
                    event.cursor
                );
            }
        }

        // A signal WAL frame must contain one contiguous same-signal run.
        // Preserve global cursor order while still avoiding one fsync per item.
        let mut start = 0;
        while start < events.len() {
            let signal = events[start].event.signal;
            let mut end = start + 1;
            while end < events.len()
                && events[end].event.signal == signal
                && events[end - 1].cursor.checked_add(1) == Some(events[end].cursor)
            {
                end += 1;
            }
            self.wal
                .append_batch(&events[start..end])
                .context("restore ordered page into signal WAL")?;
            self.fsyncs.incr();
            start = end;
        }
        self.storage
            .append_batch(&events)
            .context("restore ordered page into signal segments")?;
        self.dedupe
            .append_batch(&events)
            .context("restore ordered page into dedupe index")?;
        for event in events {
            Self::push_resident(&mut state, event, self.resident_limit);
        }
        storage::JournalHead::new(state.last_cursor, state.total_events)
            .persist(self.data_dir())
            .context("persist restored journal head")?;
        self.accepted.add(page_ids.len() as u64);
        Ok(())
    }

    fn result_for(&self, event_id: &str) -> Result<Option<AppendResult>> {
        let recent = self
            .state
            .read()
            .expect("journal state lock poisoned")
            .recent_cursors_by_event_id
            .get(event_id)
            .copied();
        let cursor = match recent {
            Some(cursor) => Some(cursor),
            None => self.dedupe.lookup(event_id)?,
        };
        Ok(cursor.map(|cursor| AppendResult {
            event_id: event_id.to_string(),
            cursor,
            raw_cursor: cursor,
            commit_index: cursor,
            duplicate: true,
        }))
    }

    pub fn query(&self, query: EventQuery) -> Result<Vec<StoredEvent>> {
        let limit = if query.limit == 0 {
            100
        } else {
            query.limit.clamp(1, 10_000)
        };
        let mut by_cursor = BTreeMap::<u64, StoredEvent>::new();
        for event in self
            .storage
            .query_events(query.signal, query.after, limit)?
        {
            by_cursor.insert(event.cursor, event);
        }
        let state = self.state.read().expect("journal state lock poisoned");
        for event in state
            .recent_events
            .iter()
            .filter(|entry| entry.cursor > query.after)
            .filter(|entry| {
                query
                    .signal
                    .is_none_or(|signal| entry.event.signal == signal)
            })
        {
            if let Some(existing) = by_cursor.insert(event.cursor, event.clone()) {
                if existing.event.event_id != event.event.event_id {
                    bail!(
                        "disk and resident journal disagree at cursor {}",
                        event.cursor
                    );
                }
            }
        }
        Ok(by_cursor.into_values().take(limit).collect())
    }

    pub fn replay(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        self.query(EventQuery {
            signal: None,
            after,
            limit,
        })
    }

    fn metrics_text(&self) -> String {
        metrics_prometheus::render(&[
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
            Sample::new(
                "sift_journal_resident_events",
                "gauge",
                "Sift events currently retained in journal memory.",
                self.resident_event_count() as u64,
            ),
        ])
    }
}

/// Shared HTTP state: journal access plus the drain bit read by `/readyz`.
#[derive(Clone)]
pub struct ServiceState {
    journal: Arc<DurableJournal>,
    draining: Arc<AtomicBool>,
    raft: Option<Arc<raft_runtime::RaftHost>>,
    peer_transport: Option<raft_runtime::PeerTransport>,
    peer_port: Option<u16>,
    state_machine: Arc<durability::SiftStateMachine>,
    local_command: Arc<tokio::sync::Mutex<()>>,
    projections: Arc<projection::ProjectionRuntime>,
    admission: Arc<ingest::AdmissionController>,
    local_capacity: Arc<storage::LocalCapacity>,
    query_jobs: Arc<api::QueryJobStore>,
    batch_coordinator: Arc<std::sync::Mutex<IngestBatchCoordinator>>,
}

struct IngestBatchRequest {
    events: Vec<EventEnvelope>,
    encoded_bytes: usize,
}

impl IngestBatchRequest {
    fn new(events: Vec<EventEnvelope>) -> Result<Self> {
        let encoded_bytes = durability::SiftCommandV1::AppendEvents {
            events: events.clone(),
        }
        .encoded()?
        .len();
        Ok(Self {
            events,
            encoded_bytes,
        })
    }
}

impl service_executor::GroupCommitRequest for IngestBatchRequest {
    type Item = EventEnvelope;
    type Key = SignalKind;

    fn key(&self) -> Self::Key {
        self.events[0].signal
    }

    fn item_count(&self) -> usize {
        self.events.len()
    }

    fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    fn into_items(self) -> Vec<Self::Item> {
        self.events
    }
}

#[derive(Default)]
struct IngestBatchCoordinator {
    queue:
        Option<service_executor::GroupCommitQueue<IngestBatchRequest, AppendResult, anyhow::Error>>,
    worker: Option<service_executor::GroupCommitWorker>,
}

#[derive(Clone)]
struct CommitContext {
    journal: Arc<DurableJournal>,
    raft: Option<Arc<raft_runtime::RaftHost>>,
    state_machine: Arc<durability::SiftStateMachine>,
    local_command: Arc<tokio::sync::Mutex<()>>,
    local_capacity: Arc<storage::LocalCapacity>,
}

struct SiftMembershipPolicy;

impl raft_runtime::MembershipPolicy for SiftMembershipPolicy {
    fn validate(&self, topology: &raft_runtime::ClusterTopology) -> anyhow::Result<()> {
        if topology.replicas_per_shard != 3
            || topology.membership.voters.len() != 3
            || !topology.membership.learners.is_empty()
        {
            bail!("Sift replicated mode requires exactly three durable voting replicas per shard");
        }
        Ok(())
    }
}

fn storage_reservation(encoded_bytes: u64, event_count: usize) -> u64 {
    encoded_bytes
        .saturating_mul(3)
        .saturating_add((event_count as u64).saturating_mul(128))
}

impl ServiceState {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_role(data_dir, storage::StorageRole::All)
    }

    pub fn open_with_role(data_dir: impl AsRef<Path>, role: storage::StorageRole) -> Result<Self> {
        Self::open_with_ingest_limits_and_role(data_dir, ingest::IngestLimits::from_env()?, role)
    }

    pub fn open_with_ingest_limits(
        data_dir: impl AsRef<Path>,
        limits: ingest::IngestLimits,
    ) -> Result<Self> {
        Self::open_with_ingest_limits_and_role(data_dir, limits, storage::StorageRole::All)
    }

    pub fn open_with_ingest_limits_and_role(
        data_dir: impl AsRef<Path>,
        limits: ingest::IngestLimits,
        role: storage::StorageRole,
    ) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let journal = Arc::new(DurableJournal::open_with_role(data_dir, role)?);
        let local_capacity = Arc::new(storage::LocalCapacity::open(
            data_dir,
            limits.max_local_storage_bytes,
            limits.min_local_free_bytes,
        )?);
        let state_machine = Arc::new(durability::SiftStateMachine::open(
            data_dir,
            journal.clone(),
        )?);
        let (raft, peer_transport, peer_port) = if raft_runtime::replica_mode() {
            let peer_port = std::env::var("SIFT_PEER_PORT")
                .unwrap_or_else(|_| "7381".to_string())
                .parse::<u16>()
                .context("SIFT_PEER_PORT must be a valid TCP port")?;
            let headless =
                std::env::var("SIFT_RAFT_HEADLESS").unwrap_or_else(|_| "sift-peer".to_string());
            let runtime = raft_runtime::ReplicaHostBuilder::new(
                "sift",
                headless,
                peer_port,
                "SIFT_PEERS",
                "https",
                SiftMembershipPolicy,
            )?
            .build_secure(
                data_dir,
                state_machine.clone(),
                "SIFT_PEER",
                raft_runtime::FsyncPolicy::Always,
                raft_runtime::HostConfig::default(),
            )
            .context("replicated Sift requires peer mTLS")?;
            (
                Some(runtime.host),
                Some(runtime.peer_transport),
                Some(runtime.peer_port),
            )
        } else {
            (None, None, None)
        };
        Ok(Self {
            projections: Arc::new(projection::ProjectionRuntime::open(
                data_dir,
                journal.clone(),
            )?),
            journal,
            draining: Arc::new(AtomicBool::new(false)),
            raft,
            peer_transport,
            peer_port,
            state_machine,
            local_command: Arc::new(tokio::sync::Mutex::new(())),
            admission: Arc::new(ingest::AdmissionController::new(limits)?),
            local_capacity,
            query_jobs: Arc::new(api::QueryJobStore::open(data_dir.join("query-jobs"))?),
            batch_coordinator: Arc::new(std::sync::Mutex::new(IngestBatchCoordinator::default())),
        })
    }

    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::Release);
        self.batch_coordinator
            .lock()
            .expect("Sift ingest batch coordinator lock poisoned")
            .queue
            .take();
    }

    /// Stop accepting new batches and wait until every accepted batch has a
    /// durable result. This also releases the journal lock held by the worker.
    pub async fn finish_drain(&self) -> Result<()> {
        self.start_drain();
        let worker = self
            .batch_coordinator
            .lock()
            .expect("Sift ingest batch coordinator lock poisoned")
            .worker
            .take();
        if let Some(worker) = worker {
            worker
                .join()
                .await
                .context("join Sift ingest batch coordinator")?;
        }
        Ok(())
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

    pub(crate) fn ensure_local_capacity(
        &self,
        incoming_bytes: usize,
    ) -> Result<(), ingest::AdmissionError> {
        self.local_capacity
            .preflight(storage_reservation(incoming_bytes as u64, 1))
            .map_err(|error| ingest::AdmissionError::local_storage_backpressure(error.to_string()))
    }

    /// Start the one in-process projection worker owned by the Sift service.
    /// The worker has no listener, WAL, or Raft group of its own and can be
    /// stopped after HTTP drain during graceful shutdown.
    pub fn start_projection_worker(&self) -> ProjectionWorker {
        let projections = self.projections.clone();
        let flush_projections = self.projections.clone();
        let journal = self.journal.clone();
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
                        let journal = journal.clone();
                        match tokio::task::spawn_blocking(move || {
                            journal.storage().seal_ready()?;
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
            projections: flush_projections,
        }
    }

    /// Start the store lifecycle worker. A replicated follower stays idle.
    /// The worker records the remote manifest before it permits WAL compaction.
    pub fn start_archive_worker(
        &self,
        destination: impl Into<String>,
        interval: std::time::Duration,
    ) -> ArchiveWorker {
        self.start_lifecycle_worker(Some(destination.into()), interval)
    }

    /// Start the same leader-only lifecycle worker for an installation that
    /// has no remote archive. It commits the local immutable segment set before
    /// it compacts WAL bytes.
    pub fn start_local_archive_worker(&self, interval: std::time::Duration) -> ArchiveWorker {
        self.start_lifecycle_worker(None, interval)
    }

    fn start_lifecycle_worker(
        &self,
        destination: Option<String>,
        interval: std::time::Duration,
    ) -> ArchiveWorker {
        let journal = self.journal.clone();
        let raft = self.raft.clone();
        let local_capacity = self.local_capacity.clone();
        let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                let leader = match &raft {
                    Some(raft) => raft.is_leader().await,
                    None => true,
                };
                if leader {
                    let journal = journal.clone();
                    let destination = destination.clone();
                    let archive = tokio::task::spawn_blocking(move || {
                        let committed = match destination.as_ref() {
                            Some(_) => storage::archive::remote_committed_watermarks(
                                journal.storage().root(),
                            )?,
                            None => storage::archive::local_committed_watermarks(
                                journal.storage().root(),
                            )?,
                        };
                        let committed_cursor = committed.max_cursor();
                        match destination {
                            Some(destination) => {
                                let mut committed = if journal.last_cursor() > committed_cursor {
                                    let receipt = storage::archive::archive_journal_gcs(
                                        &journal,
                                        &destination,
                                    )?;
                                    Some(LifecycleCommit {
                                        manifest_uri: Some(receipt.manifest_uri),
                                        event_count: receipt.manifest.event_count,
                                        segment_count: receipt.manifest.segments.len(),
                                    })
                                } else {
                                    None
                                };
                                if storage::archive::remote_retained_state(
                                    journal.storage().root(),
                                )?
                                .is_some()
                                {
                                    let expired = storage::archive::expire_committed_events_at(
                                        &journal,
                                        Utc::now(),
                                    )?;
                                    if expired.expired_events > 0 {
                                        committed = Some(LifecycleCommit {
                                            manifest_uri: Some(expired.manifest_uri),
                                            event_count: expired.retained_events,
                                            segment_count: expired.retained_segments,
                                        });
                                    }
                                }
                                anyhow::Ok(committed)
                            }
                            None => {
                                if journal.last_cursor() <= committed_cursor {
                                    return anyhow::Ok(None);
                                }
                                let receipt = storage::archive::archive_journal_local(&journal)?;
                                anyhow::Ok(Some(LifecycleCommit {
                                    manifest_uri: None,
                                    event_count: receipt.event_count,
                                    segment_count: receipt.segment_count,
                                }))
                            }
                        }
                    })
                    .await;
                    match archive {
                        Ok(Ok(Some(receipt))) => {
                            if let Err(error) = local_capacity.reconcile() {
                                tracing::warn!(
                                    %error,
                                    "Sift lifecycle committed but capacity reconciliation failed"
                                );
                            }
                            tracing::info!(
                                manifest_uri = receipt.manifest_uri.as_deref().unwrap_or("local"),
                                event_count = receipt.event_count,
                                segment_count = receipt.segment_count,
                                "Sift lifecycle manifest committed"
                            );
                        }
                        Ok(Ok(None)) => {}
                        Ok(Err(error)) => tracing::warn!(
                            %error,
                            "Sift archive attempt failed; WAL remains uncompacted"
                        ),
                        Err(error) => tracing::warn!(
                            %error,
                            "Sift archive worker task panicked; WAL remains uncompacted"
                        ),
                    }
                }

                tokio::select! {
                    changed = shutdown_rx.changed() => {
                        if changed.is_err() || *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(interval.max(std::time::Duration::from_millis(1))) => {}
                }
            }
        });
        ArchiveWorker {
            shutdown: Some(shutdown),
            task,
        }
    }

    /// Govern and durably commit one Raft batch. Every returned event shares
    /// the same commit index, which is acknowledged only after local apply (or
    /// quorum apply in three-replica mode).
    pub async fn append_batch(&self, events: Vec<EventEnvelope>) -> Result<Vec<AppendResult>> {
        if events.is_empty() {
            bail!("Sift Raft batch must not be empty");
        }
        ensure_single_signal(&events)?;
        // Govern before the Raft proposal so sensitive content never enters a
        // replicated log, even transiently. DurableJournal repeats the policy
        // idempotently at the raw boundary for direct/single-node callers.
        let governed = self.govern_events(events)?;
        self.append_governed_batch(governed).await
    }

    /// Split one decoded ingest request into Raft commands below the 1 MiB
    /// hard limit. The caller receives outcomes in the original event order.
    pub async fn append_events(&self, events: Vec<EventEnvelope>) -> Result<Vec<AppendResult>> {
        if events.is_empty() {
            return Ok(Vec::new());
        }
        if self.is_draining() {
            bail!("Sift is draining and cannot accept a new ingest batch");
        }
        ensure_single_signal(&events)?;
        let governed = self.govern_events(events)?;
        let chunks = split_governed_batches(governed)?;
        let queue = self.ingest_batch_queue()?;
        let mut results = Vec::new();
        for events in chunks {
            let request = IngestBatchRequest::new(events)?;
            results.extend(
                queue
                    .submit(request)
                    .await
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?,
            );
        }
        Ok(results)
    }

    fn ingest_batch_queue(
        &self,
    ) -> Result<service_executor::GroupCommitQueue<IngestBatchRequest, AppendResult, anyhow::Error>>
    {
        let mut coordinator = self
            .batch_coordinator
            .lock()
            .expect("Sift ingest batch coordinator lock poisoned");
        if self.is_draining() {
            bail!("Sift is draining and cannot start an ingest batch coordinator");
        }
        if let Some(queue) = coordinator.queue.as_ref() {
            return Ok(queue.clone());
        }
        let config = service_executor::GroupCommitConfig::new(
            durability::RAFT_BATCH_MAX_DELAY,
            durability::RAFT_BATCH_MAX_ITEMS,
            durability::RAFT_BATCH_MAX_BYTES,
        )?;
        let context = self.commit_context();
        let (queue, worker) =
            service_executor::spawn_group_commit(config, move |events: Vec<EventEnvelope>| {
                let context = context.clone();
                async move { context.append_governed_batch(events).await }
            });
        coordinator.queue = Some(queue.clone());
        coordinator.worker = Some(worker);
        Ok(queue)
    }

    fn commit_context(&self) -> CommitContext {
        CommitContext {
            journal: self.journal.clone(),
            raft: self.raft.clone(),
            state_machine: self.state_machine.clone(),
            local_command: self.local_command.clone(),
            local_capacity: self.local_capacity.clone(),
        }
    }

    fn govern_events(&self, events: Vec<EventEnvelope>) -> Result<Vec<EventEnvelope>> {
        let mut governed = Vec::with_capacity(events.len());
        for event in events {
            let event = self.journal.govern_event(event)?;
            event.validate()?;
            governed.push(event);
        }
        Ok(governed)
    }

    async fn append_governed_batch(
        &self,
        governed: Vec<EventEnvelope>,
    ) -> Result<Vec<AppendResult>> {
        self.commit_context().append_governed_batch(governed).await
    }

    /// Return the dedicated mutually authenticated Raft listener parts.
    /// Raft routes must never be merged into the public Sift API router.
    pub fn peer_server(&self) -> Option<(raft_runtime::PeerTransport, u16, Router)> {
        Some((
            self.peer_transport.clone()?,
            self.peer_port?,
            self.raft.as_ref()?.router(),
        ))
    }
}

impl CommitContext {
    async fn append_governed_batch(
        &self,
        governed: Vec<EventEnvelope>,
    ) -> Result<Vec<AppendResult>> {
        if governed.is_empty() {
            bail!("Sift Raft batch must not be empty");
        }
        let encoded_bytes = governed.iter().try_fold(0u64, |total, event| {
            let bytes = serde_json::to_vec(event)
                .context("encode governed Sift event for local capacity reservation")?;
            anyhow::Ok(total.saturating_add(bytes.len() as u64))
        })?;
        let capacity_reservation = self
            .local_capacity
            .reserve(storage_reservation(encoded_bytes, governed.len()))
            .context("reserve local WAL and segment capacity before Raft admission")?;
        let current_commit = self.state_machine.applied_commit_index();
        let mut duplicate_results = Vec::with_capacity(governed.len());
        let mut all_duplicates = true;
        for event in &governed {
            match self.journal.result_for(&event.event_id)? {
                Some(result) => duplicate_results.push(result.with_commit_index(current_commit)),
                None => {
                    all_duplicates = false;
                    break;
                }
            }
        }
        if all_duplicates {
            return Ok(duplicate_results);
        }
        let event_ids = governed
            .iter()
            .map(|event| event.event_id.clone())
            .collect::<Vec<_>>();
        let commit_index = self
            .commit_command(durability::SiftCommandV1::AppendEvents { events: governed })
            .await?;
        let results = if let Some(results) = self.state_machine.take_append_outcomes(commit_index) {
            results
        } else {
            let mut recovered = Vec::with_capacity(event_ids.len());
            for event_id in event_ids {
                recovered.push(
                    self.journal
                        .result_for(&event_id)?
                        .map(|result| result.with_commit_index(commit_index))
                        .context(
                            "state-machine commit completed without applying the Sift batch",
                        )?,
                );
            }
            recovered
        };
        capacity_reservation.commit();
        Ok(results)
    }

    async fn commit_command(&self, command: durability::SiftCommandV1) -> Result<u64> {
        let bytes = command.encoded()?;
        if let Some(raft) = &self.raft {
            return raft.propose(bytes).await;
        }
        let _guard = self.local_command.lock().await;
        let index = self.state_machine.applied_commit_index() + 1;
        self.state_machine.apply_local(index, &bytes)?;
        Ok(index)
    }
}

fn split_governed_batches(events: Vec<EventEnvelope>) -> Result<Vec<Vec<EventEnvelope>>> {
    let empty_size = durability::SiftCommandV1::AppendEvents { events: Vec::new() }
        .encoded()?
        .len();
    let mut chunks = Vec::new();
    let mut batch = Vec::new();
    let mut encoded_size = empty_size;
    for event in events {
        let event_size = serde_json::to_vec(&event)
            .context("encode governed Sift event for Raft batching")?
            .len();
        let separator = usize::from(!batch.is_empty());
        if !batch.is_empty()
            && (batch.len() >= durability::RAFT_BATCH_MAX_ITEMS
                || encoded_size + separator + event_size > durability::RAFT_BATCH_MAX_BYTES)
        {
            chunks.push(std::mem::take(&mut batch));
            encoded_size = empty_size;
        }
        encoded_size += usize::from(!batch.is_empty()) + event_size;
        batch.push(event);
    }
    if !batch.is_empty() {
        chunks.push(batch);
    }
    Ok(chunks)
}

fn ensure_single_signal(events: &[EventEnvelope]) -> Result<()> {
    let signal = events
        .first()
        .context("Sift batch must not be empty")?
        .signal;
    if events.iter().any(|event| event.signal != signal) {
        bail!("Sift batch must contain exactly one signal");
    }
    Ok(())
}

pub struct ProjectionWorker {
    shutdown: Option<tokio::sync::watch::Sender<bool>>,
    task: tokio::task::JoinHandle<()>,
    projections: Arc<projection::ProjectionRuntime>,
}

pub struct ArchiveWorker {
    shutdown: Option<tokio::sync::watch::Sender<bool>>,
    task: tokio::task::JoinHandle<()>,
}

struct LifecycleCommit {
    manifest_uri: Option<String>,
    event_count: u64,
    segment_count: usize,
}

impl ArchiveWorker {
    pub async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        let _ = self.task.await;
    }
}

impl ProjectionWorker {
    pub async fn stop(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(true);
        }
        let _ = self.task.await;
        let projections = self.projections;
        match tokio::task::spawn_blocking(move || projections.persist_all()).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => tracing::warn!(%error, "projection shutdown flush failed"),
            Err(error) => tracing::warn!(%error, "projection shutdown flush task panicked"),
        }
    }
}

impl service_http::ReadinessHook for ServiceState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::Acquire)
            || self.local_capacity.level() == storage::CapacityLevel::Critical
    }
}

impl service_http::MetricsProvider for ServiceState {
    fn render_metrics(&self) -> String {
        let mut text = self.journal.metrics_text();
        text.push_str(&metrics_prometheus::render(&[
            Sample::new(
                "sift_local_storage_used_bytes",
                "gauge",
                "Reserved bytes in the local Sift data root.",
                self.local_capacity.used_bytes(),
            ),
            Sample::new(
                "sift_local_storage_max_bytes",
                "gauge",
                "Configured local Sift storage safety capacity.",
                self.local_capacity.max_bytes(),
            ),
            Sample::new(
                "sift_local_storage_warning",
                "gauge",
                "One when local Sift storage is at or above the 70 percent warning threshold.",
                u64::from(matches!(
                    self.local_capacity.level(),
                    storage::CapacityLevel::Warning
                        | storage::CapacityLevel::Backpressure
                        | storage::CapacityLevel::Critical
                )),
            ),
            Sample::new(
                "sift_local_storage_critical",
                "gauge",
                "One when local Sift storage is at or above the 90 percent readiness threshold.",
                u64::from(self.local_capacity.level() == storage::CapacityLevel::Critical),
            ),
        ]));
        text
    }
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

    fn unsupported_media(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            error: "unsupported_media_type",
            message: message.into(),
            retryable: false,
            retry_after_secs: None,
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
        let lag = self.projection_lag;
        let message = if self.retryable {
            format!("{} (retryable)", self.message)
        } else {
            self.message
        };
        let mut error = service_http::ApiErr::new(self.status, self.error, message)
            .with_retryable(self.retryable);
        if let Some(seconds) = self.retry_after_secs {
            error = error.with_retry_after_seconds(seconds);
        }
        if let Some(lag) = lag {
            error = error.with_projection(ProjectionMetadata {
                projection: lag.projection,
                required_cursor: lag.required_cursor,
                current_cursor: lag.current_cursor,
            });
        }
        error.into_response()
    }
}

/// Build Sift's data-plane routes. Probe/admin routes are intentionally added
/// by `service-http` so all k8s-native services have the same shape.
pub fn router(state: Arc<ServiceState>) -> Router {
    Router::new()
        .route("/api/v1/query", post(query_v1))
        .route("/api/v1/logs/tail", post(tail_logs_v1))
        .route("/api/v1/traces/{trace_id}", get(get_trace))
        .route("/api/v1/correlate", post(correlate_v1))
        .route("/api/v1/services", get(list_services_v1))
        .route("/api/v1/queries/{query_id}", get(get_query_job_v1))
        .route("/prometheus/api/v1/write", post(prometheus_remote_write))
        .route(
            "/prometheus/api/v1/query",
            get(prometheus_instant_query).post(prometheus_instant_query),
        )
        .route(
            "/prometheus/api/v1/query_range",
            get(prometheus_range_query).post(prometheus_range_query),
        )
        .route("/v1/logs", post(ingest_logs))
        .route("/v1/traces", post(ingest_traces))
        .route("/v1/metrics", post(ingest_metrics))
        .route("/admin/backup", get(admin_backup))
        .route("/admin/integrity", get(admin_integrity))
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

/// Build the protected HTTP data plane plus the official MCP Streamable HTTP
/// endpoint. MCP tools forward the caller's credential to these same routes.
pub fn protected_router_with_mcp(
    state: Arc<ServiceState>,
    verifier: Arc<auth::SiftVerifier>,
    internal_endpoint: &str,
) -> Result<Router> {
    Ok(router(state)
        .merge(mcp::http_router(internal_endpoint)?)
        .layer(axum::middleware::from_fn_with_state(
            verifier,
            auth::auth_middleware,
        )))
}

#[derive(Clone)]
struct QueryRoleState {
    service: Arc<ServiceState>,
    store: Router,
    max_body_bytes: usize,
}

/// Build the query role. Sync work uses the store as its source of truth.
/// Async job state stays on the query role's persistent data root.
pub fn query_role_router(
    state: Arc<ServiceState>,
    store_endpoint: &str,
    max_body_bytes: usize,
) -> Result<Router> {
    let store = proxy::query_router(store_endpoint, max_body_bytes)?;
    Ok(Router::new()
        .route("/api/v1/query", post(query_role_query_v1))
        .route("/api/v1/queries/{query_id}", get(get_query_role_job_v1))
        .fallback_service(store.clone())
        .with_state(Arc::new(QueryRoleState {
            service: state,
            store,
            max_body_bytes,
        })))
}

async fn query_role_query_v1(
    State(state): State<Arc<QueryRoleState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    payload: Result<Json<api::QueryRequestV1>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    request
        .validate()
        .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &request.project,
    )?;
    let asynchronous = request.mode == api::QueryModeV1::Async
        || (request.mode == api::QueryModeV1::Auto && request.limit > 500);
    if !asynchronous {
        return forward_query_to_store(state.store.clone(), headers, &request)
            .await
            .map_err(ApiError::internal);
    }

    let job = state
        .service
        .query_jobs
        .create(request.clone())
        .map_err(|error| ApiError::internal(format!("create query job: {error}")))?;
    let query_id = job.query_id.clone();
    let worker_id = query_id.clone();
    let log_query_id = query_id.clone();
    let store = state.store.clone();
    let max_body_bytes = state.max_body_bytes;
    let runner = service_executor::JobRunner::new(state.service.query_jobs.clone());
    let task = runner.spawn_async(worker_id, job.request, move |mut request| async move {
        request.mode = api::QueryModeV1::Sync;
        let response = forward_query_to_store(store, headers, &request).await?;
        decode_store_query_response(response, max_body_bytes).await
    });
    tokio::spawn(async move {
        match task.await {
            Ok(report) if report.persistence_error.is_none() => {}
            Ok(report) => tracing::error!(
                query_id = log_query_id,
                error = report
                    .persistence_error
                    .as_deref()
                    .unwrap_or("unknown error"),
                "persist query role job transition failed"
            ),
            Err(error) => tracing::error!(
                query_id = log_query_id,
                %error,
                "query role job runner task failed"
            ),
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(api::QueryResponseV1 {
            data: serde_json::json!({"status": "queued"}),
            next_cursor: None,
            watermark: 0,
            partial: false,
            warnings: Vec::new(),
            stats: api::QueryStatsV1 {
                elapsed_ms: 0,
                scanned: 0,
                returned: 0,
            },
            query_id: Some(query_id),
        }),
    )
        .into_response())
}

async fn forward_query_to_store(
    store: Router,
    headers: HeaderMap,
    request: &api::QueryRequestV1,
) -> std::result::Result<Response, String> {
    let body = serde_json::to_vec(request).map_err(|error| error.to_string())?;
    let mut upstream = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/query")
        .body(Body::from(body))
        .map_err(|error| error.to_string())?;
    *upstream.headers_mut() = headers;
    Ok(store
        .oneshot(upstream)
        .await
        .expect("Sift store proxy router is infallible"))
}

async fn decode_store_query_response(
    response: Response,
    max_body_bytes: usize,
) -> std::result::Result<api::QueryResponseV1, String> {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), max_body_bytes)
        .await
        .map_err(|error| format!("read store query response: {error}"))?;
    if !status.is_success() {
        let message = serde_json::from_slice::<serde_json::Value>(&bytes)
            .ok()
            .and_then(|body| body["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
        return Err(format!("store query returned HTTP {status}: {message}"));
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("decode store query response: {error}"))
}

async fn get_query_role_job_v1(
    State(state): State<Arc<QueryRoleState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<QueryJobHttpQuery>,
) -> Result<Json<api::QueryJobV1>, ApiError> {
    query_job_for_project(&state.service, principal.as_ref(), &id, &query.project).map(Json)
}

async fn query_v1(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<api::QueryRequestV1>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    request
        .validate()
        .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &request.project,
    )?;
    let asynchronous = request.mode == api::QueryModeV1::Async
        || (request.mode == api::QueryModeV1::Auto && request.limit > 500);
    if asynchronous {
        let job = state
            .query_jobs
            .create(request.clone())
            .map_err(|error| ApiError::internal(format!("create query job: {error}")))?;
        let query_id = job.query_id.clone();
        let worker_id = query_id.clone();
        let log_query_id = query_id.clone();
        let worker_state = state.clone();
        let runner = service_executor::JobRunner::new(state.query_jobs.clone());
        let task = runner.spawn_blocking(worker_id, job.request, move |request| {
            execute_query_v1(&worker_state, &request).map_err(|error| error.message)
        });
        tokio::spawn(async move {
            match task.await {
                Ok(report) if report.persistence_error.is_none() => {}
                Ok(report) => tracing::error!(
                    query_id = log_query_id,
                    error = report
                        .persistence_error
                        .as_deref()
                        .unwrap_or("unknown error"),
                    "persist query job transition failed"
                ),
                Err(error) => tracing::error!(
                    query_id = log_query_id,
                    %error,
                    "query job runner task failed"
                ),
            }
        });
        let response = api::QueryResponseV1 {
            data: serde_json::json!({"status": "queued"}),
            next_cursor: None,
            watermark: 0,
            partial: false,
            warnings: Vec::new(),
            stats: api::QueryStatsV1 {
                elapsed_ms: 0,
                scanned: 0,
                returned: 0,
            },
            query_id: Some(query_id),
        };
        return Ok((StatusCode::ACCEPTED, Json(response)).into_response());
    }
    let query_state = state.clone();
    let response = tokio::task::spawn_blocking(move || execute_query_v1(&query_state, &request))
        .await
        .map_err(|error| ApiError::internal(format!("query worker failed: {error}")))??;
    Ok(Json(response).into_response())
}

async fn tail_logs_v1(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<api::LogTailRequestV1>, JsonRejection>,
) -> Result<Json<api::QueryResponseV1>, ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    request
        .validate()
        .map_err(|error| ApiError::bad_request("invalid_tail_query", error.to_string()))?;
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &request.project,
    )?;
    let query = api::QueryRequestV1 {
        version: request.version,
        project: request.project,
        environment: request.environment,
        time_range: api::TimeRangeV1::default(),
        signal: api::QuerySignalV1::Logs {
            filter: request.filter,
        },
        limit: request.limit,
        cursor: request.after_cursor.clone(),
        mode: api::QueryModeV1::Sync,
    };
    query
        .validate()
        .map_err(|error| ApiError::bad_request("invalid_tail_query", error.to_string()))?;
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(request.wait_ms);
    loop {
        let mut response = execute_query_v1(&state, &query)?;
        let last_cursor = response.data["records"]
            .as_array()
            .and_then(|records| records.last())
            .and_then(|record| record["cursor"].as_u64());
        if let Some(last_cursor) = last_cursor {
            response.next_cursor = Some(encode_query_cursor("logs", &last_cursor.to_string()));
            return Ok(Json(response));
        }
        if tokio::time::Instant::now() >= deadline {
            response.next_cursor = request.after_cursor;
            return Ok(Json(response));
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

#[derive(Deserialize)]
struct QueryJobHttpQuery {
    project: String,
}

async fn get_query_job_v1(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    AxumPath(id): AxumPath<String>,
    Query(query): Query<QueryJobHttpQuery>,
) -> Result<Json<api::QueryJobV1>, ApiError> {
    query_job_for_project(&state, principal.as_ref(), &id, &query.project).map(Json)
}

fn query_job_for_project(
    state: &ServiceState,
    principal: Option<&Extension<RoleMapPrincipal>>,
    id: &str,
    project: &str,
) -> Result<api::QueryJobV1, ApiError> {
    authorize_project_read(principal.map(|principal| &principal.0), project)?;
    let job = state
        .query_jobs
        .get(id)
        .map_err(|error| ApiError::bad_request("invalid_query_id", error.to_string()))?
        .filter(|job| job.project == project)
        .ok_or_else(|| {
            ApiError::not_found(
                "query_not_found",
                format!("query `{id}` was not found in project `{project}`"),
            )
        })?;
    Ok(job)
}

async fn list_services_v1(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    Query(query): Query<api::ServiceQueryV1>,
) -> Result<Json<api::ServiceListResponseV1>, ApiError> {
    if query.project.trim().is_empty()
        || query
            .environment
            .as_deref()
            .is_some_and(|environment| environment.trim().is_empty())
    {
        return Err(ApiError::bad_request(
            "invalid_service_query",
            "project and environment must not be empty",
        ));
    }
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &query.project,
    )?;
    let mut summaries =
        BTreeMap::<String, (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>)>::new();
    let mut after = 0;
    loop {
        let events = state
            .journal
            .query(EventQuery {
                signal: None,
                after,
                limit: 10_000,
            })
            .map_err(|error| ApiError::internal(error.to_string()))?;
        if events.is_empty() {
            break;
        }
        for stored in &events {
            after = after.max(stored.cursor);
            let event = &stored.event;
            if event.project != query.project
                || query
                    .environment
                    .as_ref()
                    .is_some_and(|environment| &event.environment != environment)
            {
                continue;
            }
            let signal = match event.signal {
                SignalKind::Log => "logs",
                SignalKind::Metric => "metrics",
                SignalKind::Span => "traces",
            };
            let Some(service) = event
                .resource
                .get("service.name")
                .filter(|service| !service.trim().is_empty())
            else {
                continue;
            };
            let entry = summaries.entry(service.clone()).or_default();
            entry.0.insert(event.environment.clone());
            if let Some(version) = event
                .resource
                .get("service.version")
                .filter(|version| !version.trim().is_empty())
            {
                entry.1.insert(version.clone());
            }
            entry.2.insert(signal.into());
        }
        if events.len() < 10_000 {
            break;
        }
    }
    let services = summaries
        .into_iter()
        .map(
            |(name, (environments, versions, signals))| api::ServiceSummaryV1 {
                name,
                environments: environments.into_iter().collect(),
                versions: versions.into_iter().collect(),
                signals: signals.into_iter().collect(),
            },
        )
        .collect();
    Ok(Json(api::ServiceListResponseV1 {
        services,
        watermark: after,
    }))
}

async fn correlate_v1(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    payload: Result<Json<api::CorrelationRequestV1>, JsonRejection>,
) -> Result<Json<api::CorrelationResponseV1>, ApiError> {
    let Json(request) =
        payload.map_err(|error| ApiError::bad_request("invalid_json", error.body_text()))?;
    request
        .validate()
        .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?;
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &request.project,
    )?;
    for projection in [
        projection::PROJECTION_LOGGING_STORE,
        projection::PROJECTION_METRIC_STORE,
        projection::PROJECTION_TRACE_STORE,
    ] {
        state
            .projections
            .catch_up(projection)
            .map_err(|error| ApiError::internal(error.to_string()))?;
    }
    let attributes = request
        .attributes
        .iter()
        .map(|(key, value)| {
            serde_json::from_value(value.clone())
                .map(|value| (key.clone(), value))
                .with_context(|| format!("attribute `{key}` has an unsupported value"))
        })
        .collect::<Result<BTreeMap<_, _>>>()
        .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?;

    let mut log_query = projection::LogQuery::for_project(&request.project);
    log_query.environment = request.environment.clone();
    log_query.start_time = request.time_range.start.clone();
    log_query.end_time = request.time_range.end.clone();
    log_query.trace_id = request.trace_id.clone();
    log_query.span_id = request.span_id.clone();
    log_query.service_name = request.service.clone();
    log_query.attribute_equals = attributes.clone();
    log_query.limit = request.limit;
    let logs = state
        .projections
        .query_logs(&log_query)
        .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?
        .records;

    let mut metric_query = projection::MetricQuery::for_project(&request.project);
    metric_query.environment = request.environment.clone();
    metric_query.start_time = request.time_range.start.clone();
    metric_query.end_time = request.time_range.end.clone();
    metric_query.attribute_equals = attributes.clone();
    if let Some(service) = &request.service {
        metric_query
            .resource_equals
            .insert("service.name".into(), service.clone());
    }
    metric_query.limit = projection::MAX_METRIC_QUERY_LIMIT;
    let mut metrics = state
        .projections
        .query_metrics(&metric_query)
        .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?
        .series;
    if request.trace_id.is_some() || request.span_id.is_some() {
        metrics.retain(|series| {
            series.points.iter().any(|point| {
                point.exemplars.iter().any(|exemplar| {
                    request
                        .trace_id
                        .as_ref()
                        .is_none_or(|trace_id| &exemplar.trace_id == trace_id)
                        && request
                            .span_id
                            .as_ref()
                            .is_none_or(|span_id| &exemplar.span_id == span_id)
                })
            })
        });
    }
    metrics.truncate(request.limit);

    let mut warnings = Vec::new();
    let traces = if let Some(trace_id) = &request.trace_id {
        match state
            .projections
            .get_trace(&request.project, trace_id)
            .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?
        {
            Some(trace) => vec![trace],
            None => {
                warnings.push(format!("trace `{trace_id}` was not found"));
                Vec::new()
            }
        }
    } else {
        let mut trace_query = projection::TraceQuery::for_project(&request.project);
        trace_query.environment = request.environment.clone();
        trace_query.start_time_unix_nano = request
            .time_range
            .start
            .as_deref()
            .map(parse_query_time_nanos)
            .transpose()?;
        trace_query.end_time_unix_nano = request
            .time_range
            .end
            .as_deref()
            .map(parse_query_time_nanos)
            .transpose()?;
        trace_query.service = request.service.clone();
        trace_query.attributes = attributes;
        trace_query.limit = request.limit;
        let mut traces = state
            .projections
            .query_traces(&trace_query)
            .map_err(|error| ApiError::bad_request("invalid_correlation", error.to_string()))?
            .traces;
        if let Some(span_id) = &request.span_id {
            traces.retain(|trace| trace.spans.iter().any(|span| &span.span_id == span_id));
        }
        traces
    };
    let watermark = [
        projection::PROJECTION_LOGGING_STORE,
        projection::PROJECTION_METRIC_STORE,
        projection::PROJECTION_TRACE_STORE,
    ]
    .into_iter()
    .map(|projection| state.projections.current_cursor(projection))
    .collect::<Result<Vec<_>>>()
    .map_err(|error| ApiError::internal(error.to_string()))?
    .into_iter()
    .max()
    .unwrap_or(0);
    Ok(Json(api::CorrelationResponseV1 {
        logs,
        metrics,
        traces,
        watermark,
        partial: !warnings.is_empty(),
        warnings,
    }))
}

async fn prometheus_remote_write(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let content_encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let remote_write_version = headers
        .get("x-prometheus-remote-write-version")
        .and_then(|value| value.to_str().ok());
    metrics_remote_write::validate_headers(content_type, content_encoding, remote_write_version)
        .map_err(|error| ApiError::unsupported_media(error.to_string()))?;
    let limits = state.admission.limits();
    if body.len() > limits.max_compressed_body_bytes {
        return Err(ApiError::from_admission(ingest::AdmissionError::invalid(
            "compressed_body_too_large",
            "compressed remote write body exceeds the configured limit",
        )));
    }
    let decoded = metrics_remote_write::decode_snappy(&body, limits.max_decoded_body_bytes)
        .map_err(|error| match error {
            metrics_remote_write::DecodeError::BodyTooLarge { .. } => ApiError::bad_request(
                "decoded_body_too_large",
                "decoded remote write body exceeds the configured limit",
            ),
            other => ApiError::bad_request("invalid_snappy", other.to_string()),
        })?;
    state
        .ensure_local_capacity(decoded.len())
        .map_err(ApiError::from_admission)?;
    let admitted_project = headers
        .get("x-sift-project")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty());
    let decoded = prometheus::decode_remote_write(&decoded, admitted_project)
        .map_err(|error| ApiError::bad_request("invalid_remote_write", error.to_string()))?;
    authorize_project(
        principal.as_ref().map(|principal| &principal.0),
        &decoded.project,
    )?;
    let _permit = state
        .admission
        .acquire(&decoded.project, decoded.events.len(), state.is_draining())
        .map_err(ApiError::from_admission)?;
    let written = decoded.events.len();
    for event in &decoded.events {
        let bytes = serde_json::to_vec(&event)
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX);
        state
            .admission
            .validate_event_bytes(bytes)
            .map_err(ApiError::from_admission)?;
        if let Some(message) = retention_rejection(event) {
            return Err(ApiError::bad_request("outside_retention", message));
        }
    }
    state
        .append_events(decoded.events)
        .await
        .map_err(|error| ApiError::internal(format!("remote write append failed: {error}")))?;
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-prometheus-remote-write-samples-written", written)
        .body(Body::empty())
        .map_err(|error| ApiError::internal(error.to_string()))
}

async fn prometheus_instant_query(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    Query(params): Query<prometheus::InstantQueryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &params.project,
    )?;
    let parsed = prometheus::parse_promql(&params.query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let evaluation_time = params
        .time
        .as_deref()
        .map(prometheus::parse_prom_time)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?
        .unwrap_or_else(|| Utc::now().timestamp_millis() as f64 / 1_000.0);
    state
        .projections
        .catch_up(projection::PROJECTION_METRIC_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut query = prom_metric_query(&params.project, params.environment, &parsed)?;
    query.start_time = Some(
        prometheus::seconds_rfc3339(evaluation_time - 300.0)
            .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?,
    );
    query.end_time = Some(
        prometheus::seconds_rfc3339(evaluation_time + 0.001)
            .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?,
    );
    let page = state
        .projections
        .query_metrics(&query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let mut latest = page
        .series
        .iter()
        .filter_map(|series| {
            prom_active_points(&series.points)
                .last()
                .map(|point| (series, point.value))
        })
        .collect::<Vec<_>>();
    let result = match parsed.function {
        prometheus::PromFunction::Raw => latest
            .drain(..)
            .map(|(series, value)| {
                serde_json::json!({
                    "metric": prom_series_labels(series),
                    "value": [evaluation_time, prom_number(value)]
                })
            })
            .collect::<Vec<_>>(),
        prometheus::PromFunction::Rate => page
            .series
            .iter()
            .filter_map(|series| {
                prom_rate(prom_active_points(&series.points)).map(|value| (series, value))
            })
            .map(|(series, value)| {
                serde_json::json!({
                    "metric": prom_series_labels(series),
                    "value": [evaluation_time, prom_number(value)]
                })
            })
            .collect::<Vec<_>>(),
        function => {
            let values = latest
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            prom_aggregate(function, &values)
                .map(|value| {
                    vec![serde_json::json!({
                        "metric": {},
                        "value": [evaluation_time, prom_number(value)]
                    })]
                })
                .unwrap_or_default()
        }
    };
    Ok(Json(serde_json::json!({
        "status": "success",
        "data": {"resultType": "vector", "result": result}
    })))
}

async fn prometheus_range_query(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    Query(params): Query<prometheus::RangeQueryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    authorize_project_read(
        principal.as_ref().map(|principal| &principal.0),
        &params.project,
    )?;
    let parsed = prometheus::parse_promql(&params.query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let start = prometheus::parse_prom_time(&params.start)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let end = prometheus::parse_prom_time(&params.end)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let step = params
        .step
        .parse::<f64>()
        .map_err(|_| ApiError::bad_request("bad_data", "step must be seconds"))?;
    if !start.is_finite() || !end.is_finite() || start >= end || !step.is_finite() || step <= 0.0 {
        return Err(ApiError::bad_request(
            "bad_data",
            "query_range requires start < end and step > 0",
        ));
    }
    state
        .projections
        .catch_up(projection::PROJECTION_METRIC_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut query = prom_metric_query(&params.project, params.environment, &parsed)?;
    query.start_time = Some(
        prometheus::seconds_rfc3339(start)
            .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?,
    );
    query.end_time = Some(
        prometheus::seconds_rfc3339(end + 0.001)
            .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?,
    );
    let page = state
        .projections
        .query_metrics(&query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let result = prom_range_result(&page.series, parsed.function, start, step);
    Ok(Json(serde_json::json!({
        "status": "success",
        "data": {"resultType": "matrix", "result": result}
    })))
}

fn prom_metric_query(
    project: &str,
    environment: Option<String>,
    parsed: &prometheus::ParsedPromQuery,
) -> Result<projection::MetricQuery, ApiError> {
    let mut query = projection::MetricQuery::for_project(project);
    query.environment = environment;
    query.name = Some(parsed.metric.clone());
    query.limit = projection::MAX_METRIC_QUERY_LIMIT;
    for (name, value) in &parsed.labels {
        if name.starts_with("service.") || name.starts_with("cloud.") || name.starts_with("k8s.") {
            query.resource_equals.insert(name.clone(), value.clone());
        } else {
            query
                .attribute_equals
                .insert(name.clone(), AttributeValue::String(value.clone()));
        }
    }
    Ok(query)
}

fn prom_series_labels(series: &projection::MetricSeriesResultV1) -> BTreeMap<String, String> {
    let mut labels = BTreeMap::from([("__name__".into(), series.name.clone())]);
    labels.extend(
        series
            .resource
            .iter()
            .filter(|(name, _)| name.as_str() != "telemetry.sdk.name")
            .map(|(name, value)| (name.clone(), value.clone())),
    );
    for (name, value) in &series.attributes {
        let value = match value {
            AttributeValue::String(value) => value.clone(),
            AttributeValue::Bool(value) => value.to_string(),
            AttributeValue::Int(value) => value.to_string(),
            AttributeValue::Double(value) => prom_number(*value),
            _ => continue,
        };
        labels.insert(name.clone(), value);
    }
    labels
}

fn prom_number(value: f64) -> String {
    if value.is_nan() {
        "NaN".into()
    } else if value == f64::INFINITY {
        "+Inf".into()
    } else if value == f64::NEG_INFINITY {
        "-Inf".into()
    } else {
        value.to_string()
    }
}

fn prom_rate(points: &[projection::MetricPointV1]) -> Option<f64> {
    let first = points.first()?;
    let last = points.last()?;
    let seconds = (last.time_unix_nano - first.time_unix_nano) as f64 / 1_000_000_000.0;
    if seconds <= 0.0 {
        return None;
    }
    let delta = if last.value >= first.value {
        last.value - first.value
    } else {
        last.value.max(0.0)
    };
    Some(delta / seconds)
}

fn prom_active_points(points: &[projection::MetricPointV1]) -> &[projection::MetricPointV1] {
    let start = points
        .iter()
        .rposition(|point| point.stale)
        .map_or(0, |index| index + 1);
    &points[start..]
}

fn prom_aggregate(function: prometheus::PromFunction, values: &[f64]) -> Option<f64> {
    match function {
        prometheus::PromFunction::Sum => Some(values.iter().sum()),
        prometheus::PromFunction::Avg => {
            (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
        }
        prometheus::PromFunction::Min => values.iter().copied().reduce(f64::min),
        prometheus::PromFunction::Max => values.iter().copied().reduce(f64::max),
        prometheus::PromFunction::Count => Some(values.len() as f64),
        _ => None,
    }
}

fn prom_range_result(
    series: &[projection::MetricSeriesResultV1],
    function: prometheus::PromFunction,
    start: f64,
    step: f64,
) -> Vec<serde_json::Value> {
    if function == prometheus::PromFunction::Raw || function == prometheus::PromFunction::Rate {
        return series
            .iter()
            .filter_map(|series| {
                let points = if function == prometheus::PromFunction::Rate {
                    series
                        .points
                        .windows(2)
                        .filter(|window| !window[0].stale && !window[1].stale)
                        .filter_map(|window| {
                            prom_rate(window).map(|value| (&window[1], value))
                        })
                        .collect::<Vec<_>>()
                } else {
                    series
                        .points
                        .iter()
                        .filter(|point| !point.stale)
                        .map(|point| (point, point.value))
                        .collect::<Vec<_>>()
                };
                if points.is_empty() {
                    return None;
                }
                let mut buckets = BTreeMap::<i64, (f64, String)>::new();
                for (point, value) in points {
                    let timestamp = point.time_unix_nano as f64 / 1_000_000_000.0;
                    let bucket = ((timestamp - start) / step).floor() as i64;
                    buckets.insert(bucket, (timestamp, prom_number(value)));
                }
                Some(serde_json::json!({
                    "metric": prom_series_labels(series),
                    "values": buckets.into_values().map(|(timestamp, value)| serde_json::json!([timestamp, value])).collect::<Vec<_>>()
                }))
            })
            .collect();
    }
    let mut buckets = BTreeMap::<i64, (f64, Vec<f64>)>::new();
    for series in series {
        for point in series.points.iter().filter(|point| !point.stale) {
            let timestamp = point.time_unix_nano as f64 / 1_000_000_000.0;
            let bucket = ((timestamp - start) / step).floor() as i64;
            let entry = buckets.entry(bucket).or_insert((timestamp, Vec::new()));
            entry.0 = entry.0.max(timestamp);
            entry.1.push(point.value);
        }
    }
    let values = buckets
        .into_values()
        .filter_map(|(timestamp, values)| {
            prom_aggregate(function, &values)
                .map(|value| serde_json::json!([timestamp, prom_number(value)]))
        })
        .collect::<Vec<_>>();
    vec![serde_json::json!({"metric": {}, "values": values})]
}

fn execute_query_v1(
    state: &ServiceState,
    request: &api::QueryRequestV1,
) -> Result<api::QueryResponseV1, ApiError> {
    let started = Instant::now();
    match &request.signal {
        api::QuerySignalV1::Logs { filter } => {
            state
                .projections
                .catch_up(projection::PROJECTION_LOGGING_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let watermark = state
                .projections
                .current_cursor(projection::PROJECTION_LOGGING_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let after_cursor = decode_query_cursor("logs", request.cursor.as_deref())?
                .map(|value| {
                    value.parse::<u64>().map_err(|_| {
                        ApiError::bad_request("invalid_cursor", "log cursor is not an integer")
                    })
                })
                .transpose()?
                .unwrap_or(0);
            let mut query = projection::LogQuery::for_project(&request.project);
            query.environment = request.environment.clone();
            query.start_time = request.time_range.start.clone();
            query.end_time = request.time_range.end.clone();
            query.after_cursor = after_cursor;
            query.limit = projection::MAX_LOG_QUERY_LIMIT;
            let (page, archive_status) = if cold_query_requested(request) {
                let archived = projection::LoggingProjection::new()
                    .map_err(|error| ApiError::internal(error.to_string()))?;
                let status = replay_cold_query(state, request, SignalKind::Log, &archived);
                let page = match status {
                    ArchiveQueryStatus::Ready(_) => archived.query(&query),
                    ArchiveQueryStatus::NotRequired | ArchiveQueryStatus::Unavailable(_) => {
                        state.projections.query_logs(&query)
                    }
                }
                .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
                (page, status)
            } else {
                (
                    state.projections.query_logs(&query).map_err(|error| {
                        ApiError::bad_request("invalid_query", error.to_string())
                    })?,
                    ArchiveQueryStatus::NotRequired,
                )
            };
            let scanned = page.records.len();
            let scanned_cursor = page
                .records
                .last()
                .map(|record| record.cursor)
                .unwrap_or(after_cursor);
            let mut records = Vec::new();
            for record in page.records {
                if filter
                    .as_ref()
                    .map(|filter| {
                        serde_json::to_value(&record)
                            .map_err(anyhow::Error::from)
                            .and_then(|document| api::evaluate_filter(filter, &document))
                    })
                    .transpose()
                    .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?
                    .unwrap_or(true)
                {
                    records.push(record);
                    if records.len() > request.limit {
                        break;
                    }
                }
            }
            let filtered_more = records.len() > request.limit;
            records.truncate(request.limit);
            let has_more = filtered_more || page.has_more;
            let next_cursor =
                has_more.then(|| encode_query_cursor("logs", &scanned_cursor.to_string()));
            let returned = records.len();
            Ok(apply_archive_query_status(
                api::QueryResponseV1::complete(
                    serde_json::json!({"records": records}),
                    next_cursor,
                    watermark,
                    scanned,
                    returned,
                    started.elapsed(),
                ),
                archive_status,
            ))
        }
        api::QuerySignalV1::Metrics {
            name,
            function,
            filter,
            ..
        } => {
            state
                .projections
                .catch_up(projection::PROJECTION_METRIC_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let watermark = state
                .projections
                .current_cursor(projection::PROJECTION_METRIC_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let mut query = projection::MetricQuery::for_project(&request.project);
            query.environment = request.environment.clone();
            query.name = name.clone();
            query.start_time = request.time_range.start.clone();
            query.end_time = request.time_range.end.clone();
            query.aggregation = match function {
                api::MetricFunctionV1::Raw => projection::MetricAggregation::Raw,
                api::MetricFunctionV1::Sum => projection::MetricAggregation::Sum,
                api::MetricFunctionV1::Avg => projection::MetricAggregation::Avg,
                api::MetricFunctionV1::Min => projection::MetricAggregation::Min,
                api::MetricFunctionV1::Max => projection::MetricAggregation::Max,
                api::MetricFunctionV1::Count => projection::MetricAggregation::Count,
                api::MetricFunctionV1::Rate => projection::MetricAggregation::Rate,
            };
            query.after_series_id = decode_query_cursor("metrics", request.cursor.as_deref())?;
            query.limit = projection::MAX_METRIC_QUERY_LIMIT;
            let (page, archive_status) = if cold_query_requested(request) {
                let archived = projection::MetricProjection::new();
                let status = replay_cold_query(state, request, SignalKind::Metric, &archived);
                let page = match status {
                    ArchiveQueryStatus::Ready(_) => archived.query(&query),
                    ArchiveQueryStatus::NotRequired | ArchiveQueryStatus::Unavailable(_) => {
                        state.projections.query_metrics(&query)
                    }
                }
                .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
                (page, status)
            } else {
                (
                    state.projections.query_metrics(&query).map_err(|error| {
                        ApiError::bad_request("invalid_query", error.to_string())
                    })?,
                    ArchiveQueryStatus::NotRequired,
                )
            };
            let scanned = page.series.len();
            let scanned_cursor = page.series.last().map(|series| series.series_id.clone());
            let mut series = Vec::new();
            for item in page.series {
                if filter
                    .as_ref()
                    .map(|filter| {
                        serde_json::to_value(&item)
                            .map_err(anyhow::Error::from)
                            .and_then(|document| api::evaluate_filter(filter, &document))
                    })
                    .transpose()
                    .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?
                    .unwrap_or(true)
                {
                    series.push(item);
                    if series.len() > request.limit {
                        break;
                    }
                }
            }
            let filtered_more = series.len() > request.limit;
            series.truncate(request.limit);
            let has_more = filtered_more || page.has_more;
            let next_cursor = has_more
                .then(|| {
                    scanned_cursor
                        .as_deref()
                        .map(|value| encode_query_cursor("metrics", value))
                })
                .flatten();
            let returned = series.len();
            Ok(apply_archive_query_status(
                api::QueryResponseV1::complete(
                    serde_json::json!({
                        "series": series,
                        "overflowed_series": page.overflowed_series,
                        "overflowed_points": page.overflowed_points
                    }),
                    next_cursor,
                    watermark,
                    scanned,
                    returned,
                    started.elapsed(),
                ),
                archive_status,
            ))
        }
        api::QuerySignalV1::Traces {
            service,
            operation,
            min_duration_ms,
            max_duration_ms,
            status,
            attributes,
            filter,
        } => {
            state
                .projections
                .catch_up(projection::PROJECTION_TRACE_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let watermark = state
                .projections
                .current_cursor(projection::PROJECTION_TRACE_STORE)
                .map_err(|error| ApiError::internal(error.to_string()))?;
            let mut query = projection::TraceQuery::for_project(&request.project);
            query.environment = request.environment.clone();
            query.start_time_unix_nano = request
                .time_range
                .start
                .as_deref()
                .map(parse_query_time_nanos)
                .transpose()?;
            query.end_time_unix_nano = request
                .time_range
                .end
                .as_deref()
                .map(parse_query_time_nanos)
                .transpose()?;
            query.service = service.clone();
            query.operation = operation.clone();
            query.min_duration_unix_nano =
                min_duration_ms.map(|value| value.saturating_mul(1_000_000));
            query.max_duration_unix_nano =
                max_duration_ms.map(|value| value.saturating_mul(1_000_000));
            query.status = status.clone();
            query.attributes = attributes
                .iter()
                .map(|(key, value)| {
                    serde_json::from_value(value.clone())
                        .map(|value| (key.clone(), value))
                        .with_context(|| {
                            format!("trace attribute `{key}` has an unsupported value")
                        })
                })
                .collect::<Result<_>>()
                .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
            query.after_trace_id = decode_query_cursor("traces", request.cursor.as_deref())?;
            query.limit = projection::MAX_TRACE_QUERY_LIMIT;
            let (page, archive_status) = if cold_query_requested(request) {
                let archived = projection::TraceProjection::new();
                let status = replay_cold_query(state, request, SignalKind::Span, &archived);
                let page = match status {
                    ArchiveQueryStatus::Ready(_) => archived.query(&query),
                    ArchiveQueryStatus::NotRequired | ArchiveQueryStatus::Unavailable(_) => {
                        state.projections.query_traces(&query)
                    }
                }
                .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?;
                (page, status)
            } else {
                (
                    state.projections.query_traces(&query).map_err(|error| {
                        ApiError::bad_request("invalid_query", error.to_string())
                    })?,
                    ArchiveQueryStatus::NotRequired,
                )
            };
            let scanned = page.traces.len();
            let scanned_cursor = page.traces.last().map(|trace| trace.trace_id.clone());
            let mut traces = Vec::new();
            for trace in page.traces {
                if filter
                    .as_ref()
                    .map(|filter| {
                        serde_json::to_value(&trace)
                            .map_err(anyhow::Error::from)
                            .and_then(|document| api::evaluate_filter(filter, &document))
                    })
                    .transpose()
                    .map_err(|error| ApiError::bad_request("invalid_query", error.to_string()))?
                    .unwrap_or(true)
                {
                    traces.push(trace);
                    if traces.len() > request.limit {
                        break;
                    }
                }
            }
            let filtered_more = traces.len() > request.limit;
            traces.truncate(request.limit);
            let has_more = filtered_more || page.has_more;
            let next_cursor = has_more
                .then(|| {
                    scanned_cursor
                        .as_deref()
                        .map(|value| encode_query_cursor("traces", value))
                })
                .flatten();
            let returned = traces.len();
            Ok(apply_archive_query_status(
                api::QueryResponseV1::complete(
                    serde_json::json!({"traces": traces}),
                    next_cursor,
                    watermark,
                    scanned,
                    returned,
                    started.elapsed(),
                ),
                archive_status,
            ))
        }
    }
}

#[derive(Debug)]
enum ArchiveQueryStatus {
    NotRequired,
    Ready(storage::archive::ArchiveReplay),
    Unavailable(String),
}

fn cold_query_requested(request: &api::QueryRequestV1) -> bool {
    let Some(start) = request.time_range.start.as_deref() else {
        return false;
    };
    DateTime::parse_from_rfc3339(start)
        .map(|start| start.with_timezone(&Utc) < Utc::now() - chrono::Duration::days(30))
        .unwrap_or(false)
}

fn replay_cold_query(
    state: &ServiceState,
    request: &api::QueryRequestV1,
    signal: SignalKind,
    projection: &dyn projection::Projection,
) -> ArchiveQueryStatus {
    let replay = storage::archive::replay_committed_events(
        state.journal().storage().root(),
        signal,
        &request.project,
        request.environment.as_deref(),
        request.time_range.start.as_deref(),
        request.time_range.end.as_deref(),
        |event| projection.apply_idempotent(&event),
    );
    let replay = match replay {
        Ok(Some(replay)) => replay,
        Ok(None) => {
            return ArchiveQueryStatus::Unavailable(
                "archive manifest is not committed for the requested cold time range".to_string(),
            )
        }
        Err(error) => {
            return ArchiveQueryStatus::Unavailable(format!(
                "archive is unavailable for the requested cold time range: {error}"
            ))
        }
    };
    if let Err(error) =
        replay_local_events_after_archive(state, request, signal, replay.watermark, projection)
    {
        return ArchiveQueryStatus::Unavailable(format!(
            "local hot data could not be joined to the archive: {error}"
        ));
    }
    ArchiveQueryStatus::Ready(replay)
}

fn replay_local_events_after_archive(
    state: &ServiceState,
    request: &api::QueryRequestV1,
    signal: SignalKind,
    mut after: u64,
    projection: &dyn projection::Projection,
) -> Result<()> {
    loop {
        let events = state.journal().query(EventQuery {
            signal: Some(signal),
            after,
            limit: 10_000,
        })?;
        let Some(last) = events.last() else {
            break;
        };
        after = last.cursor;
        for event in events {
            if event.event.project != request.project
                || request
                    .environment
                    .as_ref()
                    .is_some_and(|environment| event.event.environment != *environment)
            {
                continue;
            }
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("local event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            let start = request
                .time_range
                .start
                .as_deref()
                .map(DateTime::parse_from_rfc3339)
                .transpose()?
                .map(|value| value.with_timezone(&Utc));
            let end = request
                .time_range
                .end
                .as_deref()
                .map(DateTime::parse_from_rfc3339)
                .transpose()?
                .map(|value| value.with_timezone(&Utc));
            if start.is_some_and(|start| occurred < start) || end.is_some_and(|end| occurred >= end)
            {
                continue;
            }
            projection.apply_idempotent(&event)?;
        }
    }
    Ok(())
}

fn apply_archive_query_status(
    mut response: api::QueryResponseV1,
    status: ArchiveQueryStatus,
) -> api::QueryResponseV1 {
    match status {
        ArchiveQueryStatus::NotRequired => {}
        ArchiveQueryStatus::Ready(replay) => {
            if replay.replayed == 0 && replay.scanned > 0 {
                response
                    .warnings
                    .push("archive scan completed with no matching events".to_string());
            }
        }
        ArchiveQueryStatus::Unavailable(warning) => {
            response.partial = true;
            response.warnings.push(warning);
        }
    }
    response
}

fn encode_query_cursor(signal: &str, value: &str) -> String {
    format!("{signal}:{}", URL_SAFE_NO_PAD.encode(value.as_bytes()))
}

fn decode_query_cursor(signal: &str, cursor: Option<&str>) -> Result<Option<String>, ApiError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    let Some((actual_signal, encoded)) = cursor.split_once(':') else {
        return Err(ApiError::bad_request(
            "invalid_cursor",
            "cursor has an invalid format",
        ));
    };
    if actual_signal != signal {
        return Err(ApiError::bad_request(
            "invalid_cursor",
            format!("{actual_signal} cursor cannot be used for {signal}"),
        ));
    }
    let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| {
        ApiError::bad_request("invalid_cursor", "cursor payload is not valid base64url")
    })?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| ApiError::bad_request("invalid_cursor", "cursor payload is not UTF-8"))
}

fn parse_query_time_nanos(value: &str) -> Result<u64, ApiError> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| ApiError::bad_request("invalid_query", "time range must be RFC 3339"))?;
    let nanos = parsed.timestamp_nanos_opt().ok_or_else(|| {
        ApiError::bad_request("invalid_query", "time range is outside the supported range")
    })?;
    u64::try_from(nanos).map_err(|_| {
        ApiError::bad_request("invalid_query", "time range must not precede Unix epoch")
    })
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
        HeaderValue::from_static(durability::SNAPSHOT_CONTENT_TYPE),
    );
    Ok(response)
}

#[derive(Debug, Deserialize)]
struct IntegrityHttpQuery {
    project: String,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegritySignalV1 {
    pub count: u64,
    pub watermark: u64,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegritySignalsV1 {
    pub logs: IntegritySignalV1,
    pub metrics: IntegritySignalV1,
    pub traces: IntegritySignalV1,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegrityWatermarksV1 {
    pub logs: u64,
    pub metrics: u64,
    pub traces: u64,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegrityWalBytesV1 {
    pub logs: u64,
    pub metrics: u64,
    pub traces: u64,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegrityArchiveV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub committed_at: Option<String>,
    pub watermarks: IntegrityWatermarksV1,
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
pub struct IntegrityStorageV1 {
    pub wal_bytes: IntegrityWalBytesV1,
    pub archive: IntegrityArchiveV1,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct IntegrityReportV1 {
    pub version: u16,
    pub project: String,
    pub cluster_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_from: Option<String>,
    pub event_count: u64,
    pub event_id_digest_algorithm: String,
    pub event_id_sha256: String,
    pub watermark: u64,
    pub signals: IntegritySignalsV1,
    pub storage: IntegrityStorageV1,
}

impl IntegritySignalsV1 {
    fn include(&mut self, event: &StoredEvent) {
        let signal = match event.event.signal {
            SignalKind::Log => &mut self.logs,
            SignalKind::Metric => &mut self.metrics,
            SignalKind::Span => &mut self.traces,
        };
        signal.count = signal.count.saturating_add(1);
        signal.watermark = signal.watermark.max(event.cursor);
    }
}

#[utoipa::path(
    get,
    path = "/admin/integrity",
    params(("project" = String, Query, description = "project to verify")),
    responses(
        (status = 200, description = "project count, ID digest, and watermarks", body = IntegrityReportV1),
        (status = 403, description = "wildcard admin role required", body = ErrorEnvelope)
    )
)]
async fn admin_integrity(
    State(state): State<Arc<ServiceState>>,
    principal: Option<Extension<RoleMapPrincipal>>,
    Query(query): Query<IntegrityHttpQuery>,
) -> Result<Json<IntegrityReportV1>, ApiError> {
    authorize_global_admin(principal.as_ref().map(|principal| &principal.0))?;
    let project = query.project.trim();
    if project.is_empty() {
        return Err(ApiError::bad_request(
            "invalid_project",
            "integrity project must not be empty",
        ));
    }

    let layout_path = state.journal().storage().root().join("layout.json");
    let layout: storage::LayoutManifest = serde_json::from_slice(
        &std::fs::read(&layout_path)
            .map_err(|error| ApiError::internal(format!("read integrity layout: {error}")))?,
    )
    .map_err(|error| ApiError::internal(format!("decode integrity layout: {error}")))?;
    let storage_root = state.journal().storage().root();
    let archive = storage::archive::committed_status(storage_root)
        .map_err(|error| ApiError::internal(format!("read archive integrity: {error}")))?;
    let watermarks = archive
        .as_ref()
        .map(|status| status.watermarks)
        .unwrap_or_default();
    let wal_bytes = |signal: &str| {
        std::fs::metadata(storage_root.join("wal").join(signal).join("events.framed"))
            .map(|metadata| metadata.len())
            .unwrap_or(0)
    };

    let mut after = 0_u64;
    let mut event_count = 0_u64;
    let mut watermark = 0_u64;
    let mut event_id_digest = [0_u8; 32];
    let mut signals = IntegritySignalsV1::default();
    loop {
        let page = state
            .journal()
            .query(EventQuery {
                signal: None,
                after,
                limit: 10_000,
            })
            .map_err(|error| ApiError::internal(format!("scan integrity events: {error}")))?;
        let Some(last) = page.last() else {
            break;
        };
        after = last.cursor;
        for event in page.iter().filter(|event| event.event.project == project) {
            event_count = event_count.saturating_add(1);
            watermark = watermark.max(event.cursor);
            signals.include(event);
            let digest: [u8; 32] = Sha256::digest(event.event.event_id.as_bytes()).into();
            for (slot, byte) in event_id_digest.iter_mut().zip(digest) {
                *slot ^= byte;
            }
        }
    }

    Ok(Json(IntegrityReportV1 {
        version: 1,
        project: project.to_string(),
        cluster_id: layout.cluster_id,
        restored_from: layout.restored_from,
        event_count,
        event_id_digest_algorithm: "xor-sha256-v1".to_string(),
        event_id_sha256: hex::encode(event_id_digest),
        watermark,
        signals,
        storage: IntegrityStorageV1 {
            wal_bytes: IntegrityWalBytesV1 {
                logs: wal_bytes("logs"),
                metrics: wal_bytes("metrics"),
                traces: wal_bytes("traces"),
            },
            archive: IntegrityArchiveV1 {
                manifest_uri: archive.as_ref().map(|status| status.manifest_uri.clone()),
                manifest_sha256: archive
                    .as_ref()
                    .map(|status| status.manifest_sha256.clone()),
                committed_at: archive.as_ref().map(|status| status.committed_at.clone()),
                watermarks: IntegrityWatermarksV1 {
                    logs: watermarks.logs,
                    metrics: watermarks.metrics,
                    traces: watermarks.traces,
                },
            },
        },
    }))
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
    state
        .ensure_local_capacity(decoded_body.len())
        .map_err(ApiError::from_admission)?;
    let decoded = ingest::otlp::decode(signal, media, &decoded_body, project)
        .map_err(|error| ApiError::bad_request("invalid_otlp", error.to_string()))?;
    let _permit = state
        .admission
        .acquire(project, decoded.item_count(), state.is_draining())
        .map_err(ApiError::from_admission)?;
    let mut rejected = 0usize;
    let mut messages = Vec::new();
    let mut accepted = Vec::new();
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
        if let Some(message) = retention_rejection(&event) {
            rejected += 1;
            if messages.len() < 8 {
                messages.push(message);
            }
            continue;
        }
        accepted.push(event);
    }
    let accepted_count = accepted.len();
    if let Err(error) = state.append_events(accepted).await {
        rejected += accepted_count;
        if messages.len() < 8 {
            messages.push(format!("durable batch append failed: {error}"));
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
                "subject `{}` lacks wildcard admin access required for this admin operation",
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

#[derive(OpenApi)]
#[openapi(
    paths(
        ingest_logs,
        ingest_traces,
        ingest_metrics,
        admin_backup,
        admin_integrity
    ),
    components(schemas(
        AttributeValue,
        InstrumentationScope,
        MetricPoint,
        MetricTemporality,
        MetricExemplar,
        projection::LogRecordV1,
        projection::SpanLinkV1,
        projection::SpanEventV1,
        projection::SpanRecordV1,
        projection::TraceResultV1,
        projection::HistogramKind,
        projection::MetricHistogramV1,
        projection::MetricPointV1,
        projection::MetricChunkV1,
        projection::MetricRollupV1,
        projection::MetricAggregation,
        projection::MetricSeriesResultV1,
        IntegritySignalV1,
        IntegritySignalsV1,
        IntegrityWatermarksV1,
        IntegrityWalBytesV1,
        IntegrityArchiveV1,
        IntegrityStorageV1,
        IntegrityReportV1,
        ErrorEnvelope
    )),
    tags((name = "telemetry", description = "Sift logs, metrics, and traces"))
)]
struct SiftApi;

pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::openapi::{
        path::{OperationBuilder, PathItem, PathItemType},
        response::ResponseBuilder,
    };

    let mut document = SiftApi::openapi();
    for (path, method, summary) in [
        (
            "/api/v1/query",
            "post",
            "Run one versioned logs, metrics, or traces query",
        ),
        (
            "/api/v1/logs/tail",
            "post",
            "Read a bounded resumable log tail",
        ),
        ("/api/v1/traces/{trace_id}", "get", "Read one trace"),
        ("/api/v1/correlate", "post", "Find related telemetry"),
        ("/api/v1/services", "get", "List observed services"),
        (
            "/api/v1/queries/{query_id}",
            "get",
            "Read a persistent asynchronous query job",
        ),
        (
            "/prometheus/api/v1/write",
            "post",
            "Receive Prometheus Remote Write 1.0",
        ),
        (
            "/prometheus/api/v1/query",
            "get",
            "Run an instant PromQL query",
        ),
        (
            "/prometheus/api/v1/query_range",
            "get",
            "Run a range PromQL query",
        ),
    ] {
        let method = match method {
            "get" => PathItemType::Get,
            "post" => PathItemType::Post,
            _ => unreachable!("phase-one OpenAPI method is fixed"),
        };
        let operation = OperationBuilder::new()
            .summary(Some(summary))
            .response("200", ResponseBuilder::new().description("Success").build())
            .build();
        document
            .paths
            .paths
            .insert(path.to_string(), PathItem::new(method, operation));
    }
    document
}

pub fn openapi_json() -> Result<String> {
    serde_json::to_string_pretty(&openapi()).context("serialize OpenAPI contract")
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) fn retention_rejection(event: &EventEnvelope) -> Option<String> {
    let occurred = DateTime::parse_from_rfc3339(&event.occurred_at).ok()?;
    let cutoff = Utc::now() - chrono::Duration::days(180);
    (occurred.with_timezone(&Utc) < cutoff).then(|| {
        format!(
            "event `{}` occurred before Sift's 180-day retention boundary",
            event.event_id
        )
    })
}
// HANDWRITE-END

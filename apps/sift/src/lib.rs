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
        Arc, Mutex, RwLock,
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
use chrono::{DateTime, Duration, Utc};
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
    /// Sift acceptance time. Exact event-id idempotency starts from this time.
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
    /// Leader-selected Raft decision time. The exact six-hour window starts
    /// at this returned timestamp, not at client-side response receipt time.
    pub acknowledged_at: String,
    /// Compatibility alias for `raw_cursor`.
    pub cursor: u64,
    pub raw_cursor: u64,
    pub commit_index: u64,
    /// True when Sift found the event ID inside its six-hour exact window.
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
const RECOVERY_PAGE_BYTES: usize = 16 * 1024 * 1024;
const PROJECTION_LOCAL_BUFFER_EVENTS: usize = 100_000;
const ALL_VOTER_CHECKPOINT_ATTEMPT: std::time::Duration = std::time::Duration::from_secs(30);
const ARCHIVE_GC_BATCH_OBJECTS: usize = 128;

#[derive(Default)]
struct JournalState {
    recent_events: VecDeque<StoredEvent>,
    recent_cursors_by_event_id: HashMap<(String, String), RecentCursor>,
    last_cursor: u64,
    total_events: u64,
    projection_generation: u64,
    retention_generation: u64,
    event_content_digest: [u8; 32],
}

#[derive(Clone, Debug)]
struct RecentCursor {
    cursor: u64,
    acknowledged_at: DateTime<Utc>,
}

impl RecentCursor {
    fn from_stored(event: &StoredEvent) -> Result<Self> {
        Ok(Self {
            cursor: event.cursor,
            acknowledged_at: DateTime::parse_from_rfc3339(&event.acknowledged_at)
                .context("stored event acknowledged_at must be RFC3339")?
                .with_timezone(&Utc),
        })
    }

    fn active_at(&self, now: DateTime<Utc>) -> bool {
        self.acknowledged_at >= now - Duration::seconds(storage::IDEMPOTENCY_WINDOW_SECONDS)
    }
}

fn recent_cursor_at(
    cursors: &HashMap<(String, String), RecentCursor>,
    project: &str,
    event_id: &str,
    now: DateTime<Utc>,
) -> Option<u64> {
    recent_receipt_at(cursors, project, event_id, now).map(|(cursor, _)| cursor)
}

fn recent_receipt_at(
    cursors: &HashMap<(String, String), RecentCursor>,
    project: &str,
    event_id: &str,
    now: DateTime<Utc>,
) -> Option<(u64, DateTime<Utc>)> {
    cursors
        .get(&(project.to_owned(), event_id.to_owned()))
        .filter(|recent| recent.active_at(now))
        .map(|recent| (recent.cursor, recent.acknowledged_at))
}

fn recent_cursor_map(
    events: &VecDeque<StoredEvent>,
) -> Result<HashMap<(String, String), RecentCursor>> {
    events
        .iter()
        .map(|event| {
            Ok((
                (event.event.project.clone(), event.event.event_id.clone()),
                RecentCursor::from_stored(event)?,
            ))
        })
        .collect()
}

fn xor_event_content_digest(accumulator: &mut [u8; 32], event: &EventEnvelope) -> Result<()> {
    let encoded = serde_json::to_vec(event).context("encode Sift event content digest")?;
    let digest: [u8; 32] = Sha256::digest(encoded).into();
    for (slot, byte) in accumulator.iter_mut().zip(digest) {
        *slot ^= byte;
    }
    Ok(())
}

fn xor_digest(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut combined = left;
    for (slot, byte) in combined.iter_mut().zip(right) {
        *slot ^= byte;
    }
    combined
}

fn decode_digest(value: &str) -> Result<[u8; 32]> {
    hex::decode(value)
        .context("decode Sift SHA-256 digest")?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Sift SHA-256 digest must be 32 bytes"))
}

struct CanonicalRecoveryReader<'a> {
    storage: &'a storage::RawStorage,
    segments: storage::RawStorageReader,
    wal: storage::SignalWalReader,
    archived: storage::archive::ArchiveWatermarks,
    repair_segments: bool,
    segment_next: Option<StoredEvent>,
    wal_next: Option<StoredEvent>,
    pending: Option<StoredEvent>,
}

impl<'a> CanonicalRecoveryReader<'a> {
    fn open(
        storage: &'a storage::RawStorage,
        wal: &'a storage::SignalWal,
        archived: storage::archive::ArchiveWatermarks,
        after: u64,
        repair_segments: bool,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            segments: storage.reader(after)?,
            wal: wal.reader(after)?,
            archived,
            repair_segments,
            segment_next: None,
            wal_next: None,
            pending: None,
        })
    }

    fn read_page(&mut self) -> Result<Vec<StoredEvent>> {
        self.read_page_with_limits(RECOVERY_PAGE_EVENTS, RECOVERY_PAGE_BYTES)
            .map(|(page, _)| page)
    }

    fn read_page_with_limits(
        &mut self,
        max_events: usize,
        max_bytes: usize,
    ) -> Result<(Vec<StoredEvent>, bool)> {
        if max_events == 0 || max_bytes == 0 {
            bail!("canonical recovery page limits must be greater than zero");
        }
        let mut page = Vec::with_capacity(max_events.min(1_000));
        let mut bytes = 0_usize;
        while page.len() < max_events {
            let Some(event) = self.pending.take().or(self.next_event()?) else {
                return Ok((page, true));
            };
            let encoded = serde_json::to_vec(&event)?.len();
            if !page.is_empty() && bytes.saturating_add(encoded) > max_bytes {
                self.pending = Some(event);
                return Ok((page, false));
            }
            bytes = bytes.saturating_add(encoded);
            page.push(event);
        }
        Ok((page, false))
    }

    fn next_event(&mut self) -> Result<Option<StoredEvent>> {
        loop {
            if self.segment_next.is_none() {
                self.segment_next = self.segments.next_event()?;
            }
            if self.wal_next.is_none() {
                self.wal_next = self.wal.read_page(1, usize::MAX)?.pop();
            }
            let segment_cursor = self.segment_next.as_ref().map(|event| event.cursor);
            let wal_cursor = self.wal_next.as_ref().map(|event| event.cursor);
            match (segment_cursor, wal_cursor) {
                (None, None) => return Ok(None),
                (Some(segment_cursor), Some(wal_cursor)) if segment_cursor == wal_cursor => {
                    let segment = self.segment_next.take().expect("segment event exists");
                    let wal_event = self.wal_next.take().expect("WAL event exists");
                    if segment != wal_event {
                        bail!("WAL and segment disagree at cursor {segment_cursor}");
                    }
                    return Ok(Some(wal_event));
                }
                (Some(segment_cursor), Some(wal_cursor)) if segment_cursor < wal_cursor => {
                    let segment = self.segment_next.take().expect("segment event exists");
                    if !self.archived.covers(segment.event.signal, segment_cursor) {
                        bail!(
                            "segment cursor {segment_cursor} has no committed WAL or archive receipt"
                        );
                    }
                    return Ok(Some(segment));
                }
                (Some(_), Some(_)) | (None, Some(_)) => {
                    let wal_event = self.wal_next.take().expect("WAL event exists");
                    if self
                        .archived
                        .covers(wal_event.event.signal, wal_event.cursor)
                    {
                        continue;
                    }
                    if !self.repair_segments {
                        bail!(
                            "WAL cursor {} was not recovered into a segment",
                            wal_event.cursor
                        );
                    }
                    self.storage
                        .append(&wal_event)
                        .context("recover committed WAL event into a segment")?;
                    return Ok(Some(wal_event));
                }
                (Some(segment_cursor), None) => {
                    let segment = self.segment_next.take().expect("segment event exists");
                    if !self.archived.covers(segment.event.signal, segment_cursor) {
                        bail!(
                            "segment cursor {segment_cursor} has no committed WAL or archive receipt"
                        );
                    }
                    return Ok(Some(segment));
                }
            }
        }
    }
}

fn rebuild_dedupe_index(
    root: &Path,
    storage: &storage::RawStorage,
    wal: &storage::SignalWal,
    dedupe: &storage::DedupeIndex,
    archived: storage::archive::ArchiveWatermarks,
    expected_last_cursor: u64,
) -> Result<()> {
    dedupe.reset()?;
    let mut page = Vec::with_capacity(RECOVERY_PAGE_EVENTS);
    let now = Utc::now();

    let cutoff = now - Duration::seconds(storage::IDEMPOTENCY_WINDOW_SECONDS);
    let remote = storage::archive::replay_recent_committed_events(root, cutoff, |event| {
        page.push(event);
        if page.len() == RECOVERY_PAGE_EVENTS {
            append_unique_dedupe_page_at(dedupe, &mut page, now)?;
            dedupe.maintain_at(now, false)?;
        }
        Ok(())
    })?;
    if !page.is_empty() {
        append_unique_dedupe_page_at(dedupe, &mut page, now)?;
        dedupe.maintain_at(now, false)?;
    }

    let mut receipt_page = Vec::<storage::DedupeReceipt>::with_capacity(RECOVERY_PAGE_EVENTS);
    storage::archive::replay_recent_committed_receipts(root, cutoff, |receipt| {
        receipt_page.push(receipt);
        if receipt_page.len() == RECOVERY_PAGE_EVENTS {
            dedupe.append_receipts_at(&receipt_page, expected_last_cursor, now)?;
            receipt_page.clear();
            dedupe.maintain_at(now, false)?;
        }
        Ok(())
    })?;
    if !receipt_page.is_empty() {
        dedupe.append_receipts_at(&receipt_page, expected_last_cursor, now)?;
        dedupe.maintain_at(now, false)?;
    }

    let remote_watermarks = remote
        .map(|_| archived)
        .unwrap_or_else(storage::archive::ArchiveWatermarks::default);
    let mut local_reader = CanonicalRecoveryReader::open(storage, wal, archived, 0, false)?;
    loop {
        let local = local_reader.read_page()?;
        if local.is_empty() {
            break;
        }
        let mut new_events = local
            .into_iter()
            .filter(|event| !remote_watermarks.covers(event.event.signal, event.cursor))
            .collect::<Vec<_>>();
        append_unique_dedupe_page_at(dedupe, &mut new_events, now)?;
        dedupe.maintain_at(now, false)?;
    }

    let stats = dedupe.stats_at(now)?;
    if stats.newest_cursor > expected_last_cursor {
        bail!(
            "rebuilt dedupe index cursor {} is ahead of journal cursor {expected_last_cursor}",
            stats.newest_cursor
        );
    }
    dedupe.mark_rebuilt_through(expected_last_cursor)?;
    dedupe.maintain_at(now, true)?;
    Ok(())
}

fn append_unique_dedupe_page(
    dedupe: &storage::DedupeIndex,
    page: &mut Vec<StoredEvent>,
) -> Result<()> {
    append_unique_dedupe_page_at(dedupe, page, Utc::now())
}

fn append_unique_dedupe_page_at(
    dedupe: &storage::DedupeIndex,
    page: &mut Vec<StoredEvent>,
    now: DateTime<Utc>,
) -> Result<()> {
    if page.is_empty() {
        return Ok(());
    }
    let mut page_ids = HashMap::with_capacity(page.len());
    for stored in page.iter() {
        if !dedupe.covers(stored, now)? {
            continue;
        }
        if let Some(previous) = page_ids
            .insert(
                (stored.event.project.clone(), stored.event.event_id.clone()),
                stored.cursor,
            )
            .or(dedupe.lookup_at(&stored.event.project, &stored.event.event_id, now)?)
        {
            bail!(
                "journal contains duplicate event_id {} at cursors {previous} and {}",
                stored.event.event_id,
                stored.cursor
            );
        }
    }
    dedupe.append_batch_at(page, now)?;
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
    blob_gate: Mutex<()>,
    state: RwLock<JournalState>,
    resident_limit: usize,
    governance: GovernancePolicySet,
    accepted: Counter,
    duplicates: Counter,
    fsyncs: Counter,
    recovery_required: AtomicBool,
    retention_fenced: AtomicBool,
}

#[doc(hidden)]
pub struct JournalProjectionReadSession {
    journal: Arc<DurableJournal>,
    archive: Option<storage::archive::CommittedEventReader>,
    archive_through: u64,
    cursor: u64,
    local: VecDeque<StoredEvent>,
}

impl JournalProjectionReadSession {
    #[doc(hidden)]
    pub fn read_next(&mut self, limit: usize) -> Result<Vec<StoredEvent>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let mut page = Vec::with_capacity(limit);
        if let Some(archive) = self.archive.as_mut() {
            let archived = archive.read_next(limit)?;
            if let Some(last) = archived.last() {
                self.cursor = last.cursor;
            }
            let exhausted = archived.len() < limit;
            page.extend(archived);
            if !exhausted {
                return Ok(page);
            }
            self.archive = None;
            self.cursor = self.cursor.max(self.archive_through);
        }
        while page.len() < limit {
            if self.local.is_empty() {
                // A background archive can commit and evict the next local
                // suffix after this session opened. Refresh the manifest at
                // the current cursor before consulting local files.
                if self.refresh_archive()? {
                    let archived = self
                        .archive
                        .as_mut()
                        .expect("refreshed archive reader exists")
                        .read_next(limit - page.len())?;
                    if let Some(last) = archived.last() {
                        self.cursor = last.cursor;
                    }
                    let exhausted = archived.len() < limit - page.len();
                    page.extend(archived);
                    if !exhausted {
                        return Ok(page);
                    }
                    self.archive = None;
                    self.cursor = self.cursor.max(self.archive_through);
                    continue;
                }
                self.local = self
                    .journal
                    .query_local_unchecked(
                        EventQuery {
                            signal: None,
                            after: self.cursor,
                            limit: PROJECTION_LOCAL_BUFFER_EVENTS,
                        },
                        PROJECTION_LOCAL_BUFFER_EVENTS,
                    )?
                    .into();
                if self.local.is_empty() {
                    // Close the commit/eviction race. The first refresh can
                    // observe the old receipt immediately before local files
                    // are evicted under a new committed receipt.
                    if self.refresh_archive()? {
                        continue;
                    }
                    break;
                }
            }
            while page.len() < limit {
                let Some(event) = self.local.pop_front() else {
                    break;
                };
                if event.cursor <= self.cursor {
                    bail!("projection source cursors are not strictly increasing");
                }
                self.cursor = event.cursor;
                page.push(event);
            }
        }
        Ok(page)
    }

    fn refresh_archive(&mut self) -> Result<bool> {
        let Some(reader) =
            storage::archive::CommittedEventReader::open(self.journal.data_dir(), self.cursor)?
        else {
            return Ok(false);
        };
        self.archive_through = reader.snapshot_index();
        self.archive = Some(reader);
        Ok(true)
    }
}

impl DurableJournal {
    fn ensure_recovered(&self) -> Result<()> {
        if self.recovery_required.load(Ordering::Acquire) {
            bail!("Sift journal requires archive recovery before it can serve data");
        }
        Ok(())
    }

    fn ensure_queryable(&self) -> Result<()> {
        self.ensure_recovered()?;
        if self.retention_fenced.load(Ordering::Acquire) {
            bail!("Sift queries wait for the committed retention checkpoint");
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn projection_read_session(
        self: &Arc<Self>,
        after: u64,
    ) -> Result<JournalProjectionReadSession> {
        self.ensure_recovered()?;
        let archive = storage::archive::CommittedEventReader::open(self.data_dir(), after)?;
        let archive_through = archive
            .as_ref()
            .map(storage::archive::CommittedEventReader::snapshot_index)
            .unwrap_or(after);
        Ok(JournalProjectionReadSession {
            journal: self.clone(),
            archive,
            archive_through,
            cursor: after,
            local: VecDeque::new(),
        })
    }

    pub(crate) fn set_retention_fenced(&self, fenced: bool) {
        self.retention_fenced.store(fenced, Ordering::Release);
    }

    pub(crate) fn mark_recovery_required(&self) {
        self.recovery_required.store(true, Ordering::Release);
    }

    pub(crate) fn recovery_required(&self) -> bool {
        self.recovery_required.load(Ordering::Acquire)
    }

    pub(crate) fn retention_generation(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .retention_generation
    }

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
        storage::archive::cleanup_orphan_spills(&data_dir)?;
        storage::archive::reconcile_staged_archive_gc(&data_dir)?;
        let wal = storage::SignalWal::open(&data_dir)?;
        let storage = storage::RawStorage::open(&data_dir)?;
        let stored_head = storage::JournalHead::load(&data_dir)?;
        storage::archive::reconcile_committed_retention(
            &data_dir,
            &storage,
            stored_head
                .as_ref()
                .map(|head| head.retention_generation)
                .unwrap_or_default(),
        )?;
        let archived = storage::archive::committed_watermarks(&data_dir)?;
        // A committed manifest is the durable authority for this prefix. Retry
        // compaction before comparing local segments with WAL bytes so a crash
        // after archive reconciliation cannot resurrect an older WAL copy.
        wal.compact_through(archived)?;
        let remote_retained = storage::archive::remote_retained_state(&data_dir)?;
        let mut state = JournalState::default();
        let (dedupe, dedupe_stats) = storage::DedupeIndex::open(&data_dir)?;
        let recovery_time = Utc::now();
        let mut recovery = CanonicalRecoveryReader::open(&storage, &wal, archived, 0, true)?;
        let mut dedupe_matches = true;
        let mut local_after_remote = 0_u64;
        let mut local_after_remote_digest = [0_u8; 32];
        loop {
            let page = recovery.read_page()?;
            if page.is_empty() {
                break;
            }
            for stored in page {
                if remote_retained.is_none_or(|remote| {
                    !remote.watermarks.covers(stored.event.signal, stored.cursor)
                }) {
                    local_after_remote = local_after_remote.saturating_add(1);
                    xor_event_content_digest(&mut local_after_remote_digest, &stored.event)?;
                }
                if !dedupe_stats.rebuild_required
                    && dedupe.covers(&stored, recovery_time)?
                    && dedupe.lookup_at(
                        &stored.event.project,
                        &stored.event.event_id,
                        recovery_time,
                    )? != Some(stored.cursor)
                {
                    dedupe_matches = false;
                }
                Self::insert_recovered(&mut state, stored, resident_limit)?;
            }
        }

        let mut head = stored_head.unwrap_or_else(|| {
            storage::JournalHead::new(
                state.last_cursor.max(archived.max_cursor()),
                state.total_events,
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
        if let Some(remote) = remote_retained {
            if remote.retention_generation > head.retention_generation {
                if !remote.retention_scan_pending {
                    head.projection_generation = head.projection_generation.saturating_add(1);
                }
                head.retention_generation = remote.retention_generation;
            }
        }
        if head.retained_events > head.last_cursor {
            bail!("journal head retained event count exceeds the recovered cursor range");
        }

        if dedupe_stats.rebuild_required
            || dedupe_stats.indexed_through_cursor != head.last_cursor
            || dedupe_stats.newest_cursor > head.last_cursor
            || !dedupe_matches
        {
            rebuild_dedupe_index(
                &data_dir,
                &storage,
                &wal,
                &dedupe,
                archived,
                head.last_cursor,
            )?;
        }
        head.persist(&data_dir)?;
        // A durable archive receipt authorizes WAL truncation. Retry a crash
        // or late I/O failure before this journal starts serving requests.
        wal.compact_through(archived)?;
        state.last_cursor = head.last_cursor;
        state.total_events = head.retained_events;
        state.projection_generation = head.projection_generation;
        state.retention_generation = head.retention_generation;
        if let Some(remote) = remote_retained {
            state.event_content_digest =
                xor_digest(remote.event_content_sha256, local_after_remote_digest);
        }
        let accepted = head.retained_events;
        let journal = Self {
            _layout: layout,
            wal,
            storage,
            dedupe,
            blob_gate: Mutex::new(()),
            state: RwLock::new(state),
            resident_limit,
            governance,
            accepted: Counter::new(),
            duplicates: Counter::new(),
            fsyncs: Counter::new(),
            recovery_required: AtomicBool::new(false),
            retention_fenced: AtomicBool::new(false),
        };
        journal.accepted.add(accepted);
        if let Err(error) = storage::archive::resume_local_blob_gc_batch(&journal, 128, 1_280_000) {
            tracing::warn!(%error, "resume local blob GC after restart failed; durable progress is retained");
        }
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
        self.append_durable_batch_at(events, Utc::now())
    }

    pub(crate) fn append_durable_batch_at(
        &self,
        events: Vec<EventEnvelope>,
        acknowledged_at: DateTime<Utc>,
    ) -> Result<Vec<AppendResult>> {
        self.ensure_recovered()?;
        self.dedupe.advance_window_at(acknowledged_at)?;
        self.dedupe
            .preflight_append_at(acknowledged_at, events.len())?;
        // Blob externalization happens before the event reaches the WAL. Hold
        // this gate through the durable append so retention cannot delete a
        // newly created blob before its event reference becomes visible.
        let _blob_gate = self.blob_gate.lock().expect("Sift blob gate poisoned");
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
            governed.push(event);
        }

        let mut state = self.state.write().expect("journal state lock poisoned");
        let mut next_cursor = state
            .last_cursor
            .checked_add(1)
            .context("Sift journal cursor exhausted u64")?;
        let mut staged = Vec::with_capacity(governed.len());
        let mut staged_cursors = HashMap::<(String, String), u64>::new();
        let mut results = Vec::with_capacity(governed.len());

        for mut event in governed {
            let duplicate_receipt = recent_receipt_at(
                &state.recent_cursors_by_event_id,
                &event.project,
                &event.event_id,
                acknowledged_at,
            )
            .map(|(cursor, accepted)| (cursor, accepted.to_rfc3339()))
            .or_else(|| {
                staged_cursors
                    .get(&(event.project.clone(), event.event_id.clone()))
                    .copied()
                    .map(|cursor| (cursor, acknowledged_at.to_rfc3339()))
            })
            .or(self
                .dedupe
                .lookup_record_at(&event.project, &event.event_id, acknowledged_at)?
                .map(|(cursor, accepted)| {
                    (
                        cursor,
                        DateTime::<Utc>::from_timestamp_nanos(accepted).to_rfc3339(),
                    )
                }));
            if let Some((cursor, original_acknowledged_at)) = duplicate_receipt {
                self.duplicates.incr();
                results.push(AppendResult {
                    event_id: event.event_id,
                    acknowledged_at: original_acknowledged_at,
                    cursor,
                    raw_cursor: cursor,
                    commit_index: cursor,
                    duplicate: true,
                });
                continue;
            }

            // A successful acknowledgement owns its exact six-hour retry
            // window even when the telemetry event crosses the 180-day
            // retention boundary meanwhile. Only a new event is subject to
            // retention admission.
            if let Some(message) = retention_rejection_at(&event, acknowledged_at) {
                bail!(message);
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
            staged_cursors.insert((event.project.clone(), event_id.clone()), cursor);
            staged.push(StoredEvent {
                cursor,
                acknowledged_at: acknowledged_at.to_rfc3339(),
                event,
            });
            results.push(AppendResult {
                event_id,
                acknowledged_at: acknowledged_at.to_rfc3339(),
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
            if let Err(append_error) = self.dedupe.append_batch_at(&staged, acknowledged_at) {
                let archived = storage::archive::committed_watermarks(self.data_dir())?;
                let expected_last_cursor = staged
                    .last()
                    .map(|stored| stored.cursor)
                    .unwrap_or(state.last_cursor);
                if let Err(rebuild_error) = rebuild_dedupe_index(
                    self.data_dir(),
                    &self.storage,
                    &self.wal,
                    &self.dedupe,
                    archived,
                    expected_last_cursor,
                ) {
                    self.recovery_required.store(true, Ordering::Release);
                    bail!(
                        "dedupe index append failed ({append_error:#}); rebuild failed ({rebuild_error:#})"
                    );
                }
            }
            for stored in staged {
                Self::push_resident(&mut state, stored, self.resident_limit)?;
            }
            storage::JournalHead::new(state.last_cursor, state.total_events)
                .with_projection_generation(state.projection_generation)
                .with_retention_generation(state.retention_generation)
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

    fn maintain_dedupe_at(&self, _now: DateTime<Utc>, force: bool) -> Result<usize> {
        match self.dedupe.maintain_applied(force) {
            Ok(flushed) => Ok(flushed),
            Err(maintenance_error) => {
                let archived = storage::archive::committed_watermarks(self.data_dir())?;
                let expected_last_cursor = self.last_cursor();
                rebuild_dedupe_index(
                    self.data_dir(),
                    &self.storage,
                    &self.wal,
                    &self.dedupe,
                    archived,
                    expected_last_cursor,
                )
                .with_context(|| {
                    format!(
                        "dedupe projection maintenance failed ({maintenance_error:#}); canonical rebuild failed"
                    )
                })?;
                self.dedupe.maintain_applied(force)
            }
        }
    }

    pub(crate) fn scan_blob_references_page(
        &self,
        after: u64,
        limit: usize,
    ) -> Result<(Vec<String>, u64, bool)> {
        if limit == 0 {
            bail!("blob reference scan limit must be greater than zero");
        }
        let archived = storage::archive::committed_watermarks(self.data_dir())?;
        let mut reader =
            CanonicalRecoveryReader::open(&self.storage, &self.wal, archived, after, true)?;
        let (page, exhausted) = reader.read_page_with_limits(limit, RECOVERY_PAGE_BYTES)?;
        let scanned_through = page.last().map(|event| event.cursor).unwrap_or_else(|| {
            if exhausted {
                self.last_cursor()
            } else {
                after
            }
        });
        let mut references = BTreeSet::new();
        for event in page {
            references.extend(
                event
                    .event
                    .blob_refs
                    .into_iter()
                    .map(|reference| reference.hash),
            );
        }
        Ok((references.into_iter().collect(), scanned_through, exhausted))
    }

    pub(crate) fn finalize_blob_candidates_with_index<Mark, IsLive>(
        &self,
        hashes: &[String],
        after: u64,
        limit: usize,
        mut mark_live: Mark,
        mut is_live: IsLive,
    ) -> Result<(u64, usize, bool)>
    where
        Mark: FnMut(&str) -> Result<()>,
        IsLive: FnMut(&str) -> Result<bool>,
    {
        let _blob_gate = self.blob_gate.lock().expect("Sift blob gate poisoned");
        let (references, scanned_through, exhausted) =
            self.scan_blob_references_page(after, limit)?;
        for hash in references {
            mark_live(&hash)?;
        }
        if !exhausted {
            return Ok((scanned_through, 0, false));
        }
        let mut removed = 0_usize;
        for hash in hashes {
            if !is_live(hash)? && self.storage.remove_blob(hash)? {
                removed = removed.saturating_add(1);
            }
        }
        Ok((self.last_cursor(), removed, true))
    }

    /// Seal one globally consistent archive prefix.
    ///
    /// Ingest holds this same journal-state write lock while it allocates a
    /// cursor, writes the canonical WAL, and appends rebuildable segments. The
    /// archive therefore cannot observe a later signal while missing an
    /// earlier cursor from another signal.
    pub(crate) fn seal_archive_prefix(
        &self,
    ) -> Result<(u64, Vec<(SignalKind, storage::SegmentManifest)>)> {
        let state = self.state.write().expect("journal state lock poisoned");
        let captured_cursor = state.last_cursor;
        let segments = self.storage.seal_all_with_signal()?;
        drop(state);
        Ok((captured_cursor, segments))
    }

    pub(crate) fn compact_archived_wal(
        &self,
        watermarks: storage::archive::ArchiveWatermarks,
    ) -> Result<()> {
        self.wal.compact_through(watermarks)
    }

    /// Adopt a hash-verified archive checkpoint on a caught-up replica.
    ///
    /// The journal write lock also blocks queries and appends. The local
    /// segment prefix is rewritten before the archive receipt permits WAL
    /// compaction. A new source generation then forces every typed projection
    /// to rebuild from the retained canonical rows.
    pub(crate) fn adopt_archive_checkpoint(
        &self,
        restored: &DurableJournal,
        receipt: &storage::archive::ArchiveReceipt,
        expected_raw_cursor: u64,
    ) -> Result<()> {
        if receipt.manifest.raft_snapshot_index != expected_raw_cursor
            || restored.last_cursor() != expected_raw_cursor
            || restored.total_event_count() != receipt.manifest.event_count
        {
            bail!("verified archive checkpoint does not match its Raft cursor");
        }

        self.recovery_required.store(true, Ordering::Release);
        let mut state = self.state.write().expect("journal state lock poisoned");
        self.dedupe.preflight_rebuild()?;
        let staged_dedupe = self.dedupe.replace_from(&restored.dedupe)?;
        if staged_dedupe.indexed_through_cursor != expected_raw_cursor
            || staged_dedupe.newest_cursor > expected_raw_cursor
            || staged_dedupe.rebuild_required
        {
            bail!("validated archive checkpoint dedupe index disagrees with its manifest");
        }
        let last_cursor = state.last_cursor.max(expected_raw_cursor);
        let prior_archive = storage::archive::committed_status(self.data_dir())?;
        let archive_identity_changed = prior_archive.as_ref().is_some_and(|status| {
            status.manifest_uri != receipt.manifest_uri
                || status.manifest_sha256 != receipt.manifest_sha256
        });
        self.storage
            .reconcile_retained_prefix(restored.storage(), receipt.manifest.raft_snapshot_index)?;
        let watermarks =
            storage::archive::adopt_verified_archive_receipt(self.data_dir(), receipt)?;
        self.wal.compact_through(watermarks)?;

        let projection_generation = if receipt.manifest.retention_scan.is_none()
            && (receipt.manifest.retention_generation > state.retention_generation
                || archive_identity_changed)
        {
            state.projection_generation.saturating_add(1)
        } else {
            state.projection_generation
        };
        let mut rebuilt = JournalState {
            projection_generation,
            retention_generation: receipt.manifest.retention_generation,
            ..JournalState::default()
        };
        let mut recovery =
            CanonicalRecoveryReader::open(&self.storage, &self.wal, watermarks, 0, false)?;
        let mut local_after_archive = 0_u64;
        let mut local_after_archive_digest = [0_u8; 32];
        let mut suffix_dedupe_page = Vec::with_capacity(RECOVERY_PAGE_EVENTS);
        loop {
            let page = recovery.read_page()?;
            if page.is_empty() {
                break;
            }
            for event in page {
                if !watermarks.covers(event.event.signal, event.cursor) {
                    local_after_archive = local_after_archive.saturating_add(1);
                    xor_event_content_digest(&mut local_after_archive_digest, &event.event)?;
                    suffix_dedupe_page.push(event.clone());
                    if suffix_dedupe_page.len() == RECOVERY_PAGE_EVENTS {
                        append_unique_dedupe_page(&self.dedupe, &mut suffix_dedupe_page)?;
                    }
                }
                Self::insert_recovered(&mut rebuilt, event, self.resident_limit)?;
            }
        }
        append_unique_dedupe_page(&self.dedupe, &mut suffix_dedupe_page)?;
        rebuilt.last_cursor = rebuilt.last_cursor.max(last_cursor);
        let retained_events = receipt
            .manifest
            .event_count
            .checked_add(local_after_archive)
            .context("retained event count exhausted u64")?;
        rebuilt.total_events = retained_events;
        let archived_digest: [u8; 32] = hex::decode(&receipt.manifest.event_content_sha256)
            .context("decode adopted archive event content digest")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("archive event content digest must be 32 bytes"))?;
        rebuilt.event_content_digest = xor_digest(archived_digest, local_after_archive_digest);
        storage::JournalHead::new(rebuilt.last_cursor, retained_events)
            .with_projection_generation(projection_generation)
            .with_retention_generation(rebuilt.retention_generation)
            .persist(self.data_dir())?;
        *state = rebuilt;
        drop(state);
        storage::archive::resume_local_blob_gc_batch(self, 128, 1_280_000)?;
        self.recovery_required.store(false, Ordering::Release);
        Ok(())
    }

    /// Apply one manifest-backed retention generation to a caught-up voter.
    /// The voter uses its local hot cache plus the small source/target delta.
    /// It does not download every cumulative Parquet segment.
    pub(crate) fn adopt_archive_retention_delta(
        &self,
        receipt: &storage::archive::ArchiveReceipt,
        expected_raw_cursor: u64,
    ) -> Result<()> {
        let delta = receipt
            .manifest
            .retention_delta
            .as_ref()
            .context("archive retention checkpoint is missing its source delta")?;
        if receipt.manifest.raft_snapshot_index != expected_raw_cursor
            || receipt.manifest.retention_generation != delta.source_generation.saturating_add(1)
        {
            bail!("archive retention delta does not match its Raft cursor or generation");
        }
        let local_status = storage::archive::committed_status(self.data_dir())?
            .context("archive retention delta requires its source receipt")?;
        if local_status.manifest_uri != delta.source_manifest_uri
            || local_status.manifest_sha256 != delta.source_manifest_sha256
            || local_status.retention_generation != delta.source_generation
        {
            bail!("archive retention delta source receipt changed");
        }
        let (prefix_events, prefix_digest, generation) =
            self.checkpoint_identity(expected_raw_cursor)?;
        if generation != delta.source_generation
            || prefix_events != delta.source_event_count
            || hex::encode(prefix_digest) != delta.source_event_content_sha256
        {
            bail!("archive retention delta source content disagrees with the voter");
        }

        self.recovery_required.store(true, Ordering::Release);
        let cutoff = DateTime::<Utc>::from_timestamp_nanos(delta.cutoff_unix_nano);
        self.storage
            .evict_expired_before(cutoff, expected_raw_cursor)?;
        let watermarks =
            storage::archive::adopt_verified_archive_receipt(self.data_dir(), receipt)?;
        self.apply_expiration_head(
            cutoff,
            delta.source_event_count,
            receipt.manifest.event_count,
            decode_digest(&delta.source_event_content_sha256)?,
            decode_digest(&receipt.manifest.event_content_sha256)?,
            false,
        )?;
        storage::archive::resume_local_blob_gc_batch(self, 128, 1_280_000)?;
        self.wal.compact_through(watermarks)?;
        self.recovery_required.store(false, Ordering::Release);
        Ok(())
    }

    /// Adopt newer durable archive coverage when retention did not change.
    /// A caught-up voter already has the same logical rows through Raft, so it
    /// only needs the small manifest receipt and can avoid a cumulative GCS
    /// restore on every lifecycle tick.
    pub(crate) fn adopt_archive_coverage(
        &self,
        receipt: &storage::archive::ArchiveReceipt,
        expected_raw_cursor: u64,
    ) -> Result<()> {
        if receipt.manifest.raft_snapshot_index != expected_raw_cursor {
            bail!("verified archive coverage does not match its Raft cursor");
        }
        let (prefix_events, prefix_digest, _) = self.checkpoint_identity(expected_raw_cursor)?;
        if prefix_events != receipt.manifest.event_count
            || hex::encode(prefix_digest) != receipt.manifest.event_content_sha256
        {
            bail!("archive coverage content disagrees with the caught-up journal");
        }
        self.recovery_required.store(true, Ordering::Release);
        let mut state = self.state.write().expect("journal state lock poisoned");
        if state.last_cursor < expected_raw_cursor {
            bail!("Sift journal is behind archive coverage");
        }
        if receipt.manifest.retention_generation != state.retention_generation {
            bail!("archive coverage changed retention and requires full reconciliation");
        }
        let suffix_events = state.last_cursor.saturating_sub(expected_raw_cursor);
        let expected_events = receipt
            .manifest
            .event_count
            .checked_add(suffix_events)
            .context("archive coverage event count exhausted u64")?;
        if state.total_events != expected_events {
            bail!("archive coverage event count disagrees with the caught-up journal");
        }
        let watermarks =
            storage::archive::adopt_verified_archive_receipt(self.data_dir(), receipt)?;
        state.retention_generation = receipt.manifest.retention_generation;
        storage::JournalHead::new(state.last_cursor, state.total_events)
            .with_projection_generation(state.projection_generation)
            .with_retention_generation(state.retention_generation)
            .persist(self.data_dir())?;
        self.wal.compact_through(watermarks)?;
        self.recovery_required.store(false, Ordering::Release);
        Ok(())
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
        state.recent_cursors_by_event_id = recent_cursor_map(&state.recent_events)?;
        Ok(before.saturating_sub(state.recent_events.len()))
    }

    pub(crate) fn apply_expiration_head(
        &self,
        cutoff: DateTime<Utc>,
        archived_prefix_events: u64,
        retained_prefix_events: u64,
        archived_prefix_digest: [u8; 32],
        retained_prefix_digest: [u8; 32],
        repair_dedupe: bool,
    ) -> Result<()> {
        let retention_status = storage::archive::committed_status(self.data_dir())?
            .context("expiration requires a committed archive")?;
        let retention_generation = retention_status.retention_generation;
        let archived = storage::archive::committed_watermarks(self.data_dir())?;
        let mut state = self.state.write().expect("journal state lock poisoned");
        if !repair_dedupe && state.total_events < archived_prefix_events {
            bail!("journal contains fewer events than its prior archive prefix");
        }
        if retained_prefix_events > archived_prefix_events {
            bail!("retention cannot add events to an archived prefix");
        }
        let suffix_events = state
            .last_cursor
            .checked_sub(archived.max_cursor())
            .context("archive cursor is ahead of the journal head")?;
        let retained_events = retained_prefix_events
            .checked_add(suffix_events)
            .context("retained event count exhausted u64")?;
        if retained_events > state.last_cursor {
            bail!("retained event count exceeds the journal cursor high-water mark");
        }

        let mut retained_resident = VecDeque::with_capacity(state.recent_events.len());
        while let Some(event) = state.recent_events.pop_front() {
            let occurred = DateTime::parse_from_rfc3339(&event.event.occurred_at)
                .context("resident event occurred_at must be RFC3339")?
                .with_timezone(&Utc);
            if event.cursor > archived.max_cursor() || occurred >= cutoff {
                retained_resident.push_back(event);
            }
        }
        state.recent_events = retained_resident;
        state.recent_cursors_by_event_id = recent_cursor_map(&state.recent_events)?;
        if !retention_status.retention_scan_pending
            && state.retention_generation != retention_generation
        {
            state.projection_generation = state.projection_generation.saturating_add(1);
        }
        state.total_events = retained_events;
        let suffix_digest = if repair_dedupe {
            let mut digest = [0_u8; 32];
            let mut recovery = CanonicalRecoveryReader::open(
                &self.storage,
                &self.wal,
                archived,
                archived.max_cursor(),
                true,
            )?;
            loop {
                let page = recovery.read_page()?;
                if page.is_empty() {
                    break;
                }
                for event in page {
                    xor_event_content_digest(&mut digest, &event.event)?;
                }
            }
            digest
        } else {
            xor_digest(state.event_content_digest, archived_prefix_digest)
        };
        state.event_content_digest = xor_digest(retained_prefix_digest, suffix_digest);
        state.retention_generation = retention_generation;
        let last_cursor = state.last_cursor;
        storage::JournalHead::new(last_cursor, retained_events)
            .with_projection_generation(state.projection_generation)
            .with_retention_generation(retention_generation)
            .persist(self.data_dir())?;
        if repair_dedupe {
            rebuild_dedupe_index(
                self.data_dir(),
                &self.storage,
                &self.wal,
                &self.dedupe,
                archived,
                last_cursor,
            )?;
        }
        drop(state);
        self.recovery_required.store(false, Ordering::Release);
        Ok(())
    }

    fn insert_recovered(
        state: &mut JournalState,
        stored: StoredEvent,
        resident_limit: usize,
    ) -> Result<()> {
        stored.event.validate()?;
        let acknowledged_at = DateTime::parse_from_rfc3339(&stored.acknowledged_at)
            .context("recovered event acknowledged_at must be RFC3339")?
            .with_timezone(&Utc);
        if stored.cursor <= state.last_cursor {
            bail!(
                "journal cursor {} is not strictly after recovered cursor {}",
                stored.cursor,
                state.last_cursor
            );
        }
        if recent_cursor_at(
            &state.recent_cursors_by_event_id,
            &stored.event.project,
            &stored.event.event_id,
            acknowledged_at,
        )
        .is_some()
        {
            bail!(
                "journal contains duplicate event_id {}",
                stored.event.event_id
            );
        }
        Self::push_resident(state, stored, resident_limit)?;
        Ok(())
    }

    fn push_resident(
        state: &mut JournalState,
        stored: StoredEvent,
        resident_limit: usize,
    ) -> Result<()> {
        xor_event_content_digest(&mut state.event_content_digest, &stored.event)
            .expect("validated Sift event must have a stable content digest");
        let recent = RecentCursor::from_stored(&stored)?;
        state.last_cursor = stored.cursor;
        state.total_events = state.total_events.saturating_add(1);
        state.recent_cursors_by_event_id.insert(
            (stored.event.project.clone(), stored.event.event_id.clone()),
            recent,
        );
        state.recent_events.push_back(stored);
        while state.recent_events.len() > resident_limit {
            if let Some(evicted) = state.recent_events.pop_front() {
                let remove = state
                    .recent_cursors_by_event_id
                    .get(&(
                        evicted.event.project.clone(),
                        evicted.event.event_id.clone(),
                    ))
                    .is_some_and(|recent| recent.cursor == evicted.cursor);
                if remove {
                    state.recent_cursors_by_event_id.remove(&(
                        evicted.event.project.clone(),
                        evicted.event.event_id.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn last_cursor(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .last_cursor
    }

    pub(crate) fn projection_generation(&self) -> u64 {
        self.state
            .read()
            .expect("journal state lock poisoned")
            .projection_generation
    }

    pub(crate) fn data_dir(&self) -> &Path {
        self._layout.root()
    }

    pub(crate) fn snapshot_bounds(&self) -> (u64, u64) {
        let state = self.state.read().expect("journal state lock poisoned");
        (state.last_cursor, state.total_events)
    }

    pub(crate) fn checkpoint_identity(&self, raw_cursor: u64) -> Result<(u64, [u8; 32], u64)> {
        let (last_cursor, total_events, total_digest, retention_generation) = {
            let state = self.state.read().expect("journal state lock poisoned");
            (
                state.last_cursor,
                state.total_events,
                state.event_content_digest,
                state.retention_generation,
            )
        };
        if raw_cursor > last_cursor {
            bail!("checkpoint cursor is ahead of the Sift journal");
        }
        let mut suffix_count = 0_u64;
        let mut suffix_digest = [0_u8; 32];
        let mut after = raw_cursor;
        loop {
            let page = self.query_unchecked(EventQuery {
                signal: None,
                after,
                limit: RECOVERY_PAGE_EVENTS,
            })?;
            let Some(last) = page.last().map(|event| event.cursor) else {
                break;
            };
            for event in page {
                suffix_count = suffix_count.saturating_add(1);
                xor_event_content_digest(&mut suffix_digest, &event.event)?;
            }
            if last <= after {
                bail!("checkpoint suffix scan made no progress");
            }
            after = last;
        }
        if suffix_count != last_cursor.saturating_sub(raw_cursor) {
            bail!("checkpoint suffix is not a contiguous Raft cursor range");
        }
        let prefix_events = total_events
            .checked_sub(suffix_count)
            .context("checkpoint suffix exceeds retained event count")?;
        Ok((
            prefix_events,
            xor_digest(total_digest, suffix_digest),
            retention_generation,
        ))
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
        let restored_events = events.len() as u64;
        let restore_time = Utc::now();
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
            if !self.dedupe.covers(event, restore_time)? {
                continue;
            }
            if let Some(previous) = page_ids
                .insert(
                    (event.event.project.clone(), event.event.event_id.clone()),
                    event.cursor,
                )
                .or_else(|| {
                    recent_cursor_at(
                        &state.recent_cursors_by_event_id,
                        &event.event.project,
                        &event.event.event_id,
                        restore_time,
                    )
                })
                .or(self.dedupe.lookup_at(
                    &event.event.project,
                    &event.event.event_id,
                    restore_time,
                )?)
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
            .append_batch_at(&events, restore_time)
            .context("restore ordered page into dedupe index")?;
        self.dedupe.maintain_at(restore_time, false)?;
        for event in events {
            Self::push_resident(&mut state, event, self.resident_limit)?;
        }
        storage::JournalHead::new(state.last_cursor, state.total_events)
            .with_projection_generation(state.projection_generation)
            .with_retention_generation(state.retention_generation)
            .persist(self.data_dir())
            .context("persist restored journal head")?;
        self.accepted.add(restored_events);
        Ok(())
    }

    pub(crate) fn restore_archive_dedupe_page(&self, events: &[StoredEvent]) -> Result<()> {
        self.dedupe
            .append_batch(events)
            .context("restore cold archive IDs into the dedupe index")?;
        self.dedupe.maintain_at(Utc::now(), false)?;
        Ok(())
    }

    pub(crate) fn restore_archive_receipts(
        &self,
        receipts: &[storage::DedupeReceipt],
        indexed_through_cursor: u64,
    ) -> Result<()> {
        self.dedupe
            .append_receipts_at(receipts, indexed_through_cursor, Utc::now())
            .context("restore independent archive dedupe receipts")?;
        self.dedupe.maintain_at(Utc::now(), false)?;
        Ok(())
    }

    pub(crate) fn set_restored_archive_head(
        &self,
        manifest: &storage::archive::ArchiveManifest,
    ) -> Result<()> {
        self.dedupe
            .mark_rebuilt_through(manifest.raft_snapshot_index)?;
        let dedupe = self.dedupe.stats()?;
        if dedupe.newest_cursor > manifest.raft_snapshot_index
            || dedupe.window_seconds != storage::IDEMPOTENCY_WINDOW_SECONDS as u64
        {
            bail!("restored archive dedupe index disagrees with its manifest");
        }
        let mut state = self.state.write().expect("journal state lock poisoned");
        if state.last_cursor > manifest.raft_snapshot_index
            || state.total_events > manifest.event_count
        {
            bail!("restored hot set exceeds its archive manifest");
        }
        let event_content_digest: [u8; 32] = hex::decode(&manifest.event_content_sha256)
            .context("decode restored archive event content digest")?
            .try_into()
            .map_err(|_| anyhow::anyhow!("archive event content digest must be 32 bytes"))?;
        let newly_counted = manifest.event_count.saturating_sub(state.total_events);
        state.last_cursor = manifest.raft_snapshot_index;
        state.total_events = manifest.event_count;
        state.retention_generation = manifest.retention_generation;
        state.event_content_digest = event_content_digest;
        state.projection_generation = state.projection_generation.saturating_add(1);
        storage::JournalHead::new(state.last_cursor, state.total_events)
            .with_projection_generation(state.projection_generation)
            .with_retention_generation(state.retention_generation)
            .persist(self.data_dir())?;
        self.accepted.add(newly_counted);
        Ok(())
    }

    fn result_for_at(
        &self,
        project: &str,
        event_id: &str,
        decision_time: DateTime<Utc>,
    ) -> Result<Option<AppendResult>> {
        let recent = {
            let state = self.state.read().expect("journal state lock poisoned");
            recent_receipt_at(
                &state.recent_cursors_by_event_id,
                project,
                event_id,
                decision_time,
            )
            .map(|(cursor, acknowledged_at)| (cursor, acknowledged_at.to_rfc3339()))
        };
        let receipt = match recent {
            Some(receipt) => Some(receipt),
            None => self
                .dedupe
                .lookup_record_at(project, event_id, decision_time)?
                .map(|(cursor, acknowledged_at)| {
                    (
                        cursor,
                        DateTime::<Utc>::from_timestamp_nanos(acknowledged_at).to_rfc3339(),
                    )
                }),
        };
        Ok(receipt.map(|(cursor, acknowledged_at)| AppendResult {
            event_id: event_id.to_string(),
            acknowledged_at,
            cursor,
            raw_cursor: cursor,
            commit_index: cursor,
            duplicate: true,
        }))
    }

    pub fn query(&self, query: EventQuery) -> Result<Vec<StoredEvent>> {
        self.ensure_queryable()?;
        self.query_unchecked(query)
    }

    fn query_unchecked(&self, query: EventQuery) -> Result<Vec<StoredEvent>> {
        self.query_local_unchecked(query, 10_000)
    }

    fn query_local_unchecked(
        &self,
        query: EventQuery,
        maximum_limit: usize,
    ) -> Result<Vec<StoredEvent>> {
        let limit = if query.limit == 0 {
            100
        } else {
            query.limit.clamp(1, maximum_limit.max(1))
        };
        let state = self.state.read().expect("journal state lock poisoned");
        let mut by_cursor = BTreeMap::<u64, StoredEvent>::new();
        for event in self
            .storage
            .query_events(query.signal, query.after, limit)?
        {
            by_cursor.insert(event.cursor, event);
        }
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

    pub(crate) fn query_projection_events(
        &self,
        after: u64,
        limit: usize,
    ) -> Result<Vec<StoredEvent>> {
        self.ensure_recovered()?;
        let limit = limit.clamp(1, 10_000);
        let mut by_cursor = BTreeMap::<u64, StoredEvent>::new();
        let archive_status = storage::archive::committed_status(self.data_dir())?;
        if archive_status
            .as_ref()
            .is_some_and(|status| after < status.snapshot_index)
        {
            if let Some(events) =
                storage::archive::read_committed_events_after(self.data_dir(), after, limit)?
            {
                for event in events {
                    by_cursor.insert(event.cursor, event);
                }
            }
        }
        if by_cursor.len() < limit {
            for event in self.query_unchecked(EventQuery {
                signal: None,
                after,
                limit,
            })? {
                by_cursor.insert(event.cursor, event);
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
        let encoded_bytes = durability::SiftCommandV1::append_events_size_bound(events.clone())
            .uncompressed_len()?;
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
        let flush_journal = self.journal.clone();
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
                            journal.maintain_dedupe_at(Utc::now(), false)?;
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
            journal: flush_journal,
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
        let state_machine = self.state_machine.clone();
        let local_capacity = self.local_capacity.clone();
        let (shutdown, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                let leader = match &raft {
                    Some(raft) => raft.is_leader().await,
                    None => true,
                };
                if leader {
                    let remote_archive = destination.is_some();
                    let replicated = raft.is_some();
                    let retention_capable = if remote_archive && replicated {
                        match raft
                            .as_ref()
                            .expect("replicated lifecycle has a Raft host")
                            .require_snapshot_capability_on_all_voters()
                            .await
                        {
                            Ok(()) => true,
                            Err(error) => {
                                tracing::warn!(
                                    %error,
                                    "Sift retention waits for every voter snapshot capability"
                                );
                                false
                            }
                        }
                    } else {
                        true
                    };
                    let retention_fence = prepare_retention_fence(
                        &journal,
                        &state_machine,
                        raft.as_ref(),
                        remote_archive,
                        retention_capable,
                    )
                    .await;
                    let archive = match retention_fence {
                        Ok(retention_fence) => {
                            let attempt_journal = journal.clone();
                            let attempt_state_machine = state_machine.clone();
                            let attempt_destination = destination.clone();
                            match tokio::task::spawn_blocking(move || {
                                run_lifecycle_attempt(
                                    &attempt_journal,
                                    &attempt_state_machine,
                                    attempt_destination.as_deref(),
                                    retention_fence,
                                )
                            })
                            .await
                            {
                                Ok(result) => result,
                                Err(error) => Err(anyhow::anyhow!(
                                    "Sift archive worker task panicked: {error}"
                                )),
                            }
                        }
                        Err(error) => Err(error),
                    };
                    match archive {
                        Ok(mut outcome) => {
                            let mut changed = outcome.commit.is_some();
                            let mut replicated_checkpoint_installed = false;
                            let still_leader = match &raft {
                                Some(raft) => raft.is_leader().await,
                                None => true,
                            };
                            if !still_leader {
                                tracing::warn!(
                                    "Sift lifecycle lost leadership after archive work; checkpoint and GC are deferred"
                                );
                            }
                            if let Some(raft) = &raft {
                                let mut checkpoint_allowed =
                                    still_leader && outcome.captured_applied_index > 0;
                                if outcome.retention_scan_pending {
                                    checkpoint_allowed = false;
                                    tracing::debug!(
                                        "Sift keeps the retention fence and Raft log until the bounded scan completes"
                                    );
                                }
                                let mut quorum_only_checkpoint = false;
                                if checkpoint_allowed
                                    && remote_archive
                                    && outcome.pending_archive_gc
                                {
                                    if !retention_capable {
                                        checkpoint_allowed = false;
                                        if let (
                                            Some(retention_generation),
                                            Some(manifest_uri),
                                            Some(manifest_sha256),
                                        ) = (
                                            outcome.retention_generation,
                                            outcome.manifest_uri.clone(),
                                            outcome.manifest_sha256.clone(),
                                        ) {
                                            match (durability::SiftCommandV1::
                                                ArchiveCheckpointBarrier {
                                                    retention_generation,
                                                    manifest_uri,
                                                    manifest_sha256,
                                                })
                                                .encoded()
                                            {
                                                Ok(command) => match raft.propose(command).await {
                                                    Ok(index) => {
                                                        outcome.captured_applied_index = index;
                                                        quorum_only_checkpoint = true;
                                                    }
                                                    Err(error) => tracing::warn!(
                                                        %error,
                                                        "Sift no-GC quorum checkpoint barrier proposal failed"
                                                    ),
                                                },
                                                Err(error) => tracing::warn!(
                                                    %error,
                                                    "Sift no-GC quorum checkpoint barrier encoding failed"
                                                ),
                                            }
                                        } else {
                                            tracing::warn!(
                                                "Sift archive GC is pending without a checkpoint identity"
                                            );
                                        }
                                    } else if let (
                                        Some(retention_generation),
                                        Some(manifest_uri),
                                        Some(manifest_sha256),
                                    ) = (
                                        outcome.retention_generation,
                                        outcome.manifest_uri.clone(),
                                        outcome.manifest_sha256.clone(),
                                    ) {
                                        match (durability::SiftCommandV1::
                                            ArchiveCheckpointBarrier {
                                                retention_generation,
                                                manifest_uri,
                                                manifest_sha256,
                                            })
                                            .encoded()
                                        {
                                            Ok(command) => match raft.propose(command).await {
                                                Ok(index) => {
                                                    outcome.captured_applied_index = index;
                                                }
                                                Err(error) => {
                                                    checkpoint_allowed = false;
                                                    tracing::warn!(
                                                        %error,
                                                        "Sift retention barrier proposal failed"
                                                    );
                                                }
                                            },
                                            Err(error) => {
                                                checkpoint_allowed = false;
                                                tracing::warn!(
                                                    %error,
                                                    "Sift retention barrier encoding failed"
                                                );
                                            }
                                        }
                                    } else {
                                        checkpoint_allowed = false;
                                        tracing::warn!(
                                            "Sift archive GC is pending without a retention generation"
                                        );
                                    }
                                }
                                if checkpoint_allowed {
                                    let checkpoint_journal = journal.clone();
                                    let checkpoint_state_machine = state_machine.clone();
                                    let checkpoint_destination = destination.clone();
                                    let prepared = tokio::task::spawn_blocking(move || {
                                        prepare_lifecycle_checkpoint(
                                            &checkpoint_journal,
                                            &checkpoint_state_machine,
                                            checkpoint_destination.as_deref(),
                                            true,
                                        )
                                    })
                                    .await;
                                    match prepared {
                                        Ok(Ok(up_to)) => {
                                            let all_voter_checkpoint = tokio::time::timeout(
                                                ALL_VOTER_CHECKPOINT_ATTEMPT,
                                                raft.snapshot_and_compact_through_outcome(up_to),
                                            )
                                            .await
                                            .map_err(|_| {
                                                anyhow::anyhow!(
                                                    "Sift all-voter checkpoint attempt exceeded {} seconds",
                                                    ALL_VOTER_CHECKPOINT_ATTEMPT.as_secs()
                                                )
                                            })
                                            .and_then(|result| result);
                                            match all_voter_checkpoint {
                                                Ok(compaction) if compaction.installed => {
                                                    changed = true;
                                                    replicated_checkpoint_installed = true;
                                                    if let Some(retention_generation) =
                                                        outcome.retention_generation
                                                    {
                                                        if let Err(error) =
                                                            clear_completed_retention_fence(
                                                                raft,
                                                                &state_machine,
                                                                retention_generation,
                                                            )
                                                            .await
                                                        {
                                                            tracing::warn!(
                                                                %error,
                                                                "Sift retention checkpoint installed but its quorum fence-clear command failed"
                                                            );
                                                        }
                                                    }
                                                    tracing::info!(
                                                        compacted_raft_index =
                                                            compaction.snapshot_index,
                                                        "Sift archived Raft prefix compacted"
                                                    );
                                                }
                                                Ok(_) => {}
                                                Err(error) => {
                                                    tracing::warn!(
                                                        %error,
                                                        up_to,
                                                        "Sift all-voter checkpoint failed; retain the Raft prefix for voter catch-up"
                                                    );
                                                    if remote_archive {
                                                        match compact_remote_quorum_without_gc(
                                                            &journal,
                                                            &state_machine,
                                                            &destination,
                                                            raft,
                                                        )
                                                        .await
                                                        {
                                                            Ok(compaction)
                                                                if compaction.installed =>
                                                            {
                                                                changed = true;
                                                                if let Some(retention_generation) =
                                                                    outcome.retention_generation
                                                                {
                                                                    if let Err(error) =
                                                                        clear_completed_retention_fence(
                                                                            raft,
                                                                            &state_machine,
                                                                            retention_generation,
                                                                        )
                                                                        .await
                                                                    {
                                                                        tracing::warn!(
                                                                            %error,
                                                                            "Sift quorum checkpoint installed but its fence-clear command failed"
                                                                        );
                                                                    }
                                                                }
                                                                tracing::info!(
                                                                    compacted_raft_index =
                                                                        compaction.snapshot_index,
                                                                    "Sift compacted a GCS-backed Raft prefix on quorum; archive GC waits for every voter"
                                                                );
                                                            }
                                                            Ok(_) => {}
                                                            Err(quorum_error) => tracing::warn!(
                                                                %quorum_error,
                                                                up_to,
                                                                "Sift quorum archive checkpoint also failed"
                                                            ),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Ok(Err(error)) => {
                                            tracing::warn!(
                                                %error,
                                                "Sift checkpoint preparation failed; try resident all-voter checkpoint"
                                            );
                                            if let Err(fallback_error) =
                                                compact_resident_all_voters(&state_machine, raft)
                                                    .await
                                            {
                                                tracing::warn!(
                                                    %fallback_error,
                                                    "Sift resident all-voter checkpoint failed"
                                                );
                                            } else {
                                                changed = true;
                                            }
                                        }
                                        Err(error) => tracing::warn!(
                                            %error,
                                            "Sift checkpoint preparation task panicked"
                                        ),
                                    }
                                }
                                if quorum_only_checkpoint {
                                    match compact_remote_quorum_without_gc(
                                        &journal,
                                        &state_machine,
                                        &destination,
                                        raft,
                                    )
                                    .await
                                    {
                                        Ok(compaction) if compaction.installed => {
                                            changed = true;
                                            if let Some(retention_generation) =
                                                outcome.retention_generation
                                            {
                                                if let Err(error) = clear_completed_retention_fence(
                                                    raft,
                                                    &state_machine,
                                                    retention_generation,
                                                )
                                                .await
                                                {
                                                    tracing::warn!(
                                                        %error,
                                                        "Sift no-GC quorum checkpoint installed but its fence-clear command failed"
                                                    );
                                                }
                                            }
                                            tracing::info!(
                                                    compacted_raft_index =
                                                        compaction.snapshot_index,
                                                    "Sift installed a no-GC archive checkpoint on quorum; every prior archive object is retained"
                                                );
                                        }
                                        Ok(_) => {}
                                        Err(error) => tracing::warn!(
                                            %error,
                                            "Sift no-GC quorum archive checkpoint failed"
                                        ),
                                    }
                                }
                            }
                            let archive_gc_is_safe = still_leader
                                && remote_archive
                                && !outcome.retention_scan_pending
                                && (!replicated || replicated_checkpoint_installed);
                            if archive_gc_is_safe {
                                let gc_journal = journal.clone();
                                match tokio::task::spawn_blocking(move || {
                                    storage::archive::finish_local_blob_gc(&gc_journal)?;
                                    storage::archive::finalize_archive_gc_batch_after_checkpoint(
                                        gc_journal.storage().root(),
                                        ARCHIVE_GC_BATCH_OBJECTS,
                                    )
                                })
                                .await
                                {
                                    Ok(Ok((deleted, complete))) if deleted > 0 => tracing::info!(
                                        deleted_archive_objects = deleted,
                                        archive_gc_complete = complete,
                                        "Sift obsolete archive objects deleted after checkpoint"
                                    ),
                                    Ok(Ok((_, false))) => tracing::debug!(
                                        archive_gc_batch_objects = ARCHIVE_GC_BATCH_OBJECTS,
                                        "Sift archive GC saved its cursor for the next lifecycle pass"
                                    ),
                                    Ok(Ok((_, true))) => {}
                                    Ok(Err(error)) => tracing::warn!(
                                        %error,
                                        "Sift archive checkpoint committed but obsolete object cleanup failed"
                                    ),
                                    Err(error) => tracing::warn!(
                                        %error,
                                        "Sift archive cleanup task panicked after checkpoint"
                                    ),
                                }
                            }
                            if changed {
                                if let Err(error) = local_capacity.reconcile() {
                                    tracing::warn!(
                                        %error,
                                        "Sift lifecycle committed but capacity reconciliation failed"
                                    );
                                }
                            }
                            if let Some(receipt) = outcome.commit {
                                tracing::info!(
                                    manifest_uri =
                                        receipt.manifest_uri.as_deref().unwrap_or("local"),
                                    event_count = receipt.event_count,
                                    segment_count = receipt.segment_count,
                                    "Sift lifecycle manifest committed"
                                );
                            }
                        }
                        Err(error) => {
                            tracing::warn!(
                                %error,
                                "Sift archive attempt failed; WAL remains uncompacted"
                            );
                            if let Some(raft) = &raft {
                                if raft.is_leader().await {
                                    if let Err(fallback_error) =
                                        compact_resident_all_voters(&state_machine, raft).await
                                    {
                                        tracing::warn!(
                                            %fallback_error,
                                            "Sift resident all-voter checkpoint failed after archive outage"
                                        );
                                    }
                                }
                            }
                        }
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
        let acknowledged_at = Utc::now();
        let current_commit = self.state_machine.applied_commit_index();
        let mut duplicate_results = Vec::with_capacity(governed.len());
        let mut all_duplicates = true;
        for event in &governed {
            match self
                .journal
                .result_for_at(&event.project, &event.event_id, acknowledged_at)?
            {
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
            .map(|event| (event.project.clone(), event.event_id.clone()))
            .collect::<Vec<_>>();
        let commit_index = self
            .commit_command(durability::SiftCommandV1::append_events_at(
                governed,
                acknowledged_at,
            ))
            .await?;
        let results = if let Some(results) = self.state_machine.take_append_outcomes(commit_index) {
            results
        } else {
            let mut recovered = Vec::with_capacity(event_ids.len());
            for (project, event_id) in event_ids {
                recovered.push(
                    self.journal
                        .result_for_at(&project, &event_id, acknowledged_at)?
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
    let empty_size =
        durability::SiftCommandV1::append_events_size_bound(Vec::new()).uncompressed_len()?;
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
    journal: Arc<DurableJournal>,
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

#[derive(Default)]
struct LifecycleOutcome {
    commit: Option<LifecycleCommit>,
    captured_applied_index: u64,
    pending_archive_gc: bool,
    retention_generation: Option<u64>,
    manifest_uri: Option<String>,
    manifest_sha256: Option<String>,
    retention_scan_pending: bool,
}

async fn prepare_retention_fence(
    journal: &Arc<DurableJournal>,
    state_machine: &Arc<durability::SiftStateMachine>,
    raft: Option<&Arc<raft_runtime::RaftHost>>,
    remote_archive: bool,
    retention_capable: bool,
) -> Result<Option<durability::RetentionFenceV1>> {
    if let Some((fence, applied_index)) = state_machine.pending_retention_fence() {
        if let Some(raft) = raft {
            raft.require_applied_index_on_all_voters(applied_index)
                .await
                .context("wait for every Sift voter to apply the retention fence")?;
        }
        if let Some(status) = storage::archive::committed_status(journal.storage().root())? {
            if status.retention_scan_pending
                && status.retention_generation >= fence.target_generation
            {
                let next = durability::RetentionFenceV1 {
                    source_manifest_uri: status.manifest_uri,
                    source_manifest_sha256: status.manifest_sha256,
                    target_generation: status.retention_generation.saturating_add(1),
                    evaluate_at: fence.evaluate_at,
                };
                if let Some(raft) = raft {
                    let command = durability::SiftCommandV1::RetentionFence {
                        fence: next.clone(),
                    }
                    .encoded()?;
                    let fence_index = raft
                        .propose(command)
                        .await
                        .context("advance the bounded Sift retention fence")?;
                    raft.require_applied_index_on_all_voters(fence_index)
                        .await
                        .context(
                            "wait for every Sift voter to apply the advanced retention fence",
                        )?;
                    return state_machine
                        .pending_retention_fence()
                        .context("advanced Sift retention fence was not applied locally")
                        .map(|(fence, _)| Some(fence));
                }
                return Ok(Some(next));
            }
        }
        return Ok(Some(fence));
    }
    if !remote_archive || !retention_capable {
        return Ok(None);
    }
    let evaluate_at = Utc::now();
    if !storage::archive::retention_due_at(journal.storage().root(), evaluate_at)? {
        return Ok(None);
    }
    let status = storage::archive::committed_status(journal.storage().root())?
        .context("Sift retention requires a committed archive")?;
    let fence = durability::RetentionFenceV1 {
        source_manifest_uri: status.manifest_uri,
        source_manifest_sha256: status.manifest_sha256,
        target_generation: status.retention_generation.saturating_add(1),
        evaluate_at: evaluate_at.to_rfc3339(),
    };
    if let Some(raft) = raft {
        let command = durability::SiftCommandV1::RetentionFence {
            fence: fence.clone(),
        }
        .encoded()?;
        let fence_index = raft
            .propose(command)
            .await
            .context("commit Sift retention fence")?;
        raft.require_applied_index_on_all_voters(fence_index)
            .await
            .context("wait for every Sift voter to apply the retention fence")?;
        return state_machine
            .pending_retention_fence()
            .context("committed Sift retention fence was not applied locally")
            .map(|(fence, _)| Some(fence));
    }
    Ok(Some(fence))
}

fn run_lifecycle_attempt(
    journal: &DurableJournal,
    state_machine: &durability::SiftStateMachine,
    destination: Option<&str>,
    retention_fence: Option<durability::RetentionFenceV1>,
) -> Result<LifecycleOutcome> {
    storage::archive::reconcile_live_committed_retention(journal)?;
    storage::archive::reconcile_committed_wal(journal)?;
    storage::archive::resume_local_blob_gc_batch(journal, 128, 1_280_000)?;
    let (applied_index, raw_cursor, segments) = state_machine.capture_archive_prefix()?;
    match destination {
        Some(destination) => {
            let committed_cursor = storage::archive::committed_status(journal.storage().root())?
                .map(|status| status.snapshot_index)
                .unwrap_or_default();
            let mut commit = if retention_fence.is_none() && raw_cursor > committed_cursor {
                let receipt = storage::archive::archive_journal_gcs_captured(
                    journal,
                    destination,
                    raw_cursor,
                    segments,
                )?;
                Some(LifecycleCommit {
                    manifest_uri: Some(receipt.manifest_uri),
                    event_count: receipt.manifest.event_count,
                    segment_count: receipt.manifest.segment_count as usize,
                })
            } else {
                None
            };
            if storage::archive::committed_status(journal.storage().root())?.is_some() {
                storage::archive::evict_committed_cold_segments_at(journal, Utc::now())?;
                if let Some(fence) = retention_fence {
                    let status = storage::archive::committed_status(journal.storage().root())?
                        .context("Sift retention fence requires a committed archive")?;
                    if status.retention_generation < fence.target_generation {
                        if status.manifest_uri != fence.source_manifest_uri
                            || status.manifest_sha256 != fence.source_manifest_sha256
                            || status.retention_generation.saturating_add(1)
                                != fence.target_generation
                        {
                            bail!("Sift retention fence source no longer matches local archive");
                        }
                        let evaluate_at = DateTime::parse_from_rfc3339(&fence.evaluate_at)
                            .context("Sift retention fence time must be RFC3339")?
                            .with_timezone(&Utc);
                        if let Some(expired) =
                            state_machine.expire_current_archive_at(evaluate_at)?
                        {
                            commit = Some(LifecycleCommit {
                                manifest_uri: Some(expired.manifest_uri),
                                event_count: expired.retained_events,
                                segment_count: expired.retained_segments,
                            });
                        }
                    } else if status.retention_generation > fence.target_generation {
                        bail!("Sift retention fence target is behind local retention");
                    }
                }
            }
            let status = storage::archive::committed_status(journal.storage().root())?;
            Ok(LifecycleOutcome {
                commit,
                captured_applied_index: applied_index,
                pending_archive_gc: storage::archive::archive_gc_pending(journal.storage().root()),
                retention_generation: status.as_ref().map(|status| status.retention_generation),
                manifest_uri: status.as_ref().map(|status| status.manifest_uri.clone()),
                manifest_sha256: status.as_ref().map(|status| status.manifest_sha256.clone()),
                retention_scan_pending: status
                    .as_ref()
                    .is_some_and(|status| status.retention_scan_pending),
            })
        }
        None => {
            let committed_cursor =
                storage::archive::local_committed_watermarks(journal.storage().root())?
                    .max_cursor();
            let commit = if raw_cursor > committed_cursor {
                let receipt = storage::archive::archive_journal_local_captured(
                    journal, raw_cursor, segments,
                )?;
                Some(LifecycleCommit {
                    manifest_uri: None,
                    event_count: receipt.event_count,
                    segment_count: receipt.segment_count,
                })
            } else {
                None
            };
            Ok(LifecycleOutcome {
                commit,
                captured_applied_index: applied_index,
                ..LifecycleOutcome::default()
            })
        }
    }
}

fn prepare_lifecycle_checkpoint(
    journal: &DurableJournal,
    state_machine: &durability::SiftStateMachine,
    destination: Option<&str>,
    archive_gc_authorized: bool,
) -> Result<u64> {
    let (applied_index, raw_cursor, segments) = state_machine.capture_archive_prefix()?;
    if applied_index == 0 {
        return Ok(0);
    }
    match destination {
        Some(destination) => {
            let committed_cursor = storage::archive::committed_status(journal.storage().root())?
                .map(|status| status.snapshot_index)
                .unwrap_or_default();
            if raw_cursor > committed_cursor {
                storage::archive::archive_journal_gcs_captured(
                    journal,
                    destination,
                    raw_cursor,
                    segments,
                )?;
            }
            if archive_gc_authorized {
                state_machine.prepare_archive_checkpoint(applied_index, raw_cursor)?;
            } else {
                state_machine.prepare_archive_checkpoint_without_gc(applied_index, raw_cursor)?;
            }
        }
        None => {
            let committed_cursor =
                storage::archive::local_committed_watermarks(journal.storage().root())?
                    .max_cursor();
            if raw_cursor > committed_cursor {
                storage::archive::archive_journal_local_captured(journal, raw_cursor, segments)?;
            }
            state_machine.prepare_local_checkpoint(applied_index, raw_cursor)?;
        }
    }
    Ok(applied_index)
}

async fn compact_remote_quorum_without_gc(
    journal: &Arc<DurableJournal>,
    state_machine: &Arc<durability::SiftStateMachine>,
    destination: &Option<String>,
    raft: &Arc<raft_runtime::RaftHost>,
) -> Result<raft_runtime::SnapshotCompactionOutcome> {
    let checkpoint_journal = journal.clone();
    let checkpoint_state_machine = state_machine.clone();
    let checkpoint_destination = destination.clone();
    let up_to = tokio::task::spawn_blocking(move || {
        prepare_lifecycle_checkpoint(
            &checkpoint_journal,
            &checkpoint_state_machine,
            checkpoint_destination.as_deref(),
            false,
        )
    })
    .await
    .context("Sift no-GC quorum checkpoint preparation task panicked")??;
    raft.snapshot_and_compact_through_quorum_outcome(up_to)
        .await
}

async fn clear_completed_retention_fence(
    raft: &Arc<raft_runtime::RaftHost>,
    state_machine: &Arc<durability::SiftStateMachine>,
    retention_generation: u64,
) -> Result<()> {
    let Some((fence, _)) = state_machine.pending_retention_fence() else {
        return Ok(());
    };
    if fence.target_generation > retention_generation {
        return Ok(());
    }
    raft.propose(durability::SiftCommandV1::clear_retention_fence(retention_generation).encoded()?)
        .await
        .context("commit Sift retention fence clear to quorum")?;
    Ok(())
}

async fn compact_resident_all_voters(
    state_machine: &Arc<durability::SiftStateMachine>,
    raft: &Arc<raft_runtime::RaftHost>,
) -> Result<raft_runtime::SnapshotCompactionOutcome> {
    let checkpoint_state_machine = state_machine.clone();
    let up_to = tokio::task::spawn_blocking(move || {
        let (applied_index, raw_cursor, _) = checkpoint_state_machine.capture_archive_prefix()?;
        if applied_index > 0 {
            checkpoint_state_machine.prepare_resident_checkpoint(applied_index, raw_cursor)?;
        }
        anyhow::Ok(applied_index)
    })
    .await
    .context("Sift resident checkpoint preparation task panicked")??;
    raft.snapshot_and_compact_through_outcome(up_to).await
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
        let journal = self.journal;
        match tokio::task::spawn_blocking(move || journal.maintain_dedupe_at(Utc::now(), true))
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => tracing::warn!(%error, "dedupe shutdown flush failed"),
            Err(error) => tracing::warn!(%error, "dedupe shutdown flush task panicked"),
        }
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
            || self.journal.recovery_required()
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

    fn temporarily_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: "retention_checkpoint_pending",
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
    state
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
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
    state
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
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
    state
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
    let parsed = prometheus::parse_promql(&params.query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let evaluation_time = params
        .time
        .as_deref()
        .map(prometheus::parse_prom_time_nanos)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?
        .unwrap_or_else(|| Utc::now().timestamp_nanos_opt().unwrap_or(i64::MAX));
    state
        .projections
        .catch_up(projection::PROJECTION_METRIC_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut query = prom_metric_query(&params.project, params.environment, &parsed)?;
    query.start_time = evaluation_time
        .checked_sub(PROM_LOOKBACK_NANOS)
        .map(prometheus::nanos_rfc3339)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    query.end_time = evaluation_time
        .checked_add(1)
        .map(prometheus::nanos_rfc3339)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let page = state
        .projections
        .query_metrics(&query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    ensure_complete_prom_metric_page(&page)?;
    let mut latest = page
        .series
        .iter()
        .filter_map(|series| {
            prom_latest_values(&series.points, &[evaluation_time])
                .into_iter()
                .next()
                .flatten()
                .map(|value| (series, value))
        })
        .collect::<Vec<_>>();
    let result = match parsed.function {
        prometheus::PromFunction::Raw => latest
            .drain(..)
            .map(|(series, value)| {
                serde_json::json!({
                    "metric": prom_series_labels(series),
                    "value": [prom_nanos_to_seconds(evaluation_time), prom_number(value)]
                })
            })
            .collect::<Vec<_>>(),
        prometheus::PromFunction::Rate => page
            .series
            .iter()
            .filter_map(|series| {
                prom_rate_values(&series.points, &[evaluation_time])
                    .into_iter()
                    .next()
                    .flatten()
                    .map(|value| (series, value))
            })
            .map(|(series, value)| {
                serde_json::json!({
                    "metric": prom_series_labels(series),
                    "value": [prom_nanos_to_seconds(evaluation_time), prom_number(value)]
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
                        "value": [prom_nanos_to_seconds(evaluation_time), prom_number(value)]
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
    state
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
    let parsed = prometheus::parse_promql(&params.query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let start = prometheus::parse_prom_time_nanos(&params.start)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let end = prometheus::parse_prom_time_nanos(&params.end)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let step = prometheus::parse_prom_duration_nanos(&params.step)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    if start >= end || step <= 0 {
        return Err(ApiError::bad_request(
            "bad_data",
            "query_range requires start < end and step > 0",
        ));
    }
    let evaluation_count = (i128::from(end) - i128::from(start)) / i128::from(step) + 1;
    if evaluation_count > i128::from(PROM_MAX_RANGE_EVALUATIONS) {
        return Err(ApiError::bad_request(
            "bad_data",
            format!("query_range supports at most {PROM_MAX_RANGE_EVALUATIONS} evaluation steps"),
        ));
    }
    state
        .projections
        .catch_up(projection::PROJECTION_METRIC_STORE)
        .map_err(|error| ApiError::internal(error.to_string()))?;
    let mut query = prom_metric_query(&params.project, params.environment, &parsed)?;
    query.start_time = start
        .checked_sub(PROM_LOOKBACK_NANOS)
        .map(prometheus::nanos_rfc3339)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    query.end_time = end
        .checked_add(1)
        .map(prometheus::nanos_rfc3339)
        .transpose()
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    let page = state
        .projections
        .query_metrics(&query)
        .map_err(|error| ApiError::bad_request("bad_data", error.to_string()))?;
    ensure_complete_prom_metric_page(&page)?;
    let evaluation_count = usize::try_from(evaluation_count)
        .map_err(|_| ApiError::bad_request("bad_data", "invalid evaluation count"))?;
    ensure_prom_range_work_budget(page.series.len(), evaluation_count)?;
    let result = prom_range_result(&page.series, parsed.function, start, end, step);
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

fn ensure_complete_prom_metric_page(page: &projection::MetricPage) -> Result<(), ApiError> {
    if page.has_more {
        return Err(ApiError::bad_request(
            "bad_data",
            format!(
                "Prometheus selector matches more than {} series; narrow the selector",
                projection::MAX_METRIC_QUERY_LIMIT
            ),
        ));
    }
    Ok(())
}

fn ensure_prom_range_work_budget(
    series_count: usize,
    evaluation_count: usize,
) -> Result<(), ApiError> {
    let work_samples = series_count
        .checked_mul(evaluation_count)
        .ok_or_else(|| ApiError::bad_request("bad_data", "query_range sample budget overflowed"))?;
    if work_samples > PROM_MAX_RANGE_WORK_SAMPLES {
        return Err(ApiError::bad_request(
            "bad_data",
            format!(
                "query_range would evaluate {work_samples} series samples; the limit is {PROM_MAX_RANGE_WORK_SAMPLES}"
            ),
        ));
    }
    Ok(())
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
    start: i64,
    end: i64,
    step: i64,
) -> Vec<serde_json::Value> {
    let evaluation_count = ((i128::from(end) - i128::from(start)) / i128::from(step) + 1) as usize;
    let evaluation_times = (0..evaluation_count)
        .map(|index| {
            (i128::from(start) + i128::from(step) * index as i128)
                .try_into()
                .expect("evaluation time was validated")
        })
        .collect::<Vec<_>>();
    if function == prometheus::PromFunction::Raw || function == prometheus::PromFunction::Rate {
        return series
            .iter()
            .filter_map(|series| {
                let step_values = if function == prometheus::PromFunction::Rate {
                    prom_rate_values(&series.points, &evaluation_times)
                } else {
                    prom_latest_values(&series.points, &evaluation_times)
                };
                let values = evaluation_times
                    .iter()
                    .zip(step_values)
                    .filter_map(|(evaluation_time, value)| {
                        value.map(|value| {
                            serde_json::json!([
                                prom_nanos_to_seconds(*evaluation_time),
                                prom_number(value)
                            ])
                        })
                    })
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    return None;
                }
                Some(serde_json::json!({
                    "metric": prom_series_labels(series),
                    "values": values
                }))
            })
            .collect();
    }
    let mut aggregates = vec![PromStepAggregate::default(); evaluation_times.len()];
    for series in series {
        for (aggregate, value) in aggregates
            .iter_mut()
            .zip(prom_latest_values(&series.points, &evaluation_times))
        {
            if let Some(value) = value {
                aggregate.observe(value);
            }
        }
    }
    let values = evaluation_times
        .iter()
        .zip(aggregates)
        .filter_map(|(evaluation_time, aggregate)| {
            aggregate.value(function).map(|value| {
                serde_json::json!([prom_nanos_to_seconds(*evaluation_time), prom_number(value)])
            })
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        Vec::new()
    } else {
        vec![serde_json::json!({"metric": {}, "values": values})]
    }
}

const PROM_LOOKBACK_NANOS: i64 = 300_000_000_000;
const PROM_MAX_RANGE_EVALUATIONS: i64 = 11_000;
const PROM_MAX_RANGE_WORK_SAMPLES: usize = 1_000_000;

fn prom_nanos_to_seconds(nanos: i64) -> f64 {
    nanos as f64 / 1_000_000_000.0
}

fn prom_latest_values(
    points: &[projection::MetricPointV1],
    evaluation_times: &[i64],
) -> Vec<Option<f64>> {
    let mut next_point = 0usize;
    evaluation_times
        .iter()
        .map(|evaluation_time| {
            while next_point < points.len() && points[next_point].time_unix_nano <= *evaluation_time
            {
                next_point += 1;
            }
            let point = points.get(next_point.checked_sub(1)?)?;
            let oldest_visible = evaluation_time.checked_sub(PROM_LOOKBACK_NANOS);
            (!point.stale
                && oldest_visible
                    .is_none_or(|oldest_visible| point.time_unix_nano > oldest_visible))
            .then_some(point.value)
        })
        .collect()
}

fn prom_rate_values(
    points: &[projection::MetricPointV1],
    evaluation_times: &[i64],
) -> Vec<Option<f64>> {
    let mut counter_prefix = Vec::with_capacity(points.len());
    let mut previous = None;
    let mut cumulative = 0.0;
    for point in points {
        if point.stale {
            previous = None;
            cumulative = 0.0;
        } else if let Some(previous_value) = previous {
            cumulative += if point.value >= previous_value {
                point.value - previous_value
            } else {
                point.value.max(0.0)
            };
            previous = Some(point.value);
        } else {
            previous = Some(point.value);
        }
        counter_prefix.push(cumulative);
    }

    let mut next_point = 0usize;
    let mut epoch_start = 0usize;
    let mut window_start = 0usize;
    evaluation_times
        .iter()
        .map(|evaluation_time| {
            while next_point < points.len() && points[next_point].time_unix_nano <= *evaluation_time
            {
                if points[next_point].stale {
                    epoch_start = next_point + 1;
                    window_start = epoch_start;
                }
                next_point += 1;
            }
            if next_point <= epoch_start {
                return None;
            }
            window_start = window_start.max(epoch_start);
            if let Some(oldest_visible) = evaluation_time.checked_sub(PROM_LOOKBACK_NANOS) {
                while window_start < next_point
                    && points[window_start].time_unix_nano <= oldest_visible
                {
                    window_start += 1;
                }
            }
            if window_start >= next_point {
                return None;
            }
            let last = next_point - 1;
            let elapsed = points[last].time_unix_nano - points[window_start].time_unix_nano;
            if elapsed <= 0 {
                return None;
            }
            let delta = counter_prefix[last] - counter_prefix[window_start];
            Some(delta / (elapsed as f64 / 1_000_000_000.0))
        })
        .collect()
}

#[derive(Clone, Default)]
struct PromStepAggregate {
    count: usize,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl PromStepAggregate {
    fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = Some(self.min.map_or(value, |current| current.min(value)));
        self.max = Some(self.max.map_or(value, |current| current.max(value)));
    }

    fn value(&self, function: prometheus::PromFunction) -> Option<f64> {
        if self.count == 0 {
            return None;
        }
        match function {
            prometheus::PromFunction::Sum => Some(self.sum),
            prometheus::PromFunction::Avg => Some(self.sum / self.count as f64),
            prometheus::PromFunction::Min => self.min,
            prometheus::PromFunction::Max => self.max,
            prometheus::PromFunction::Count => Some(self.count as f64),
            _ => None,
        }
    }
}

#[cfg(test)]
mod prometheus_evaluator_tests {
    use super::*;

    fn point(time_unix_nano: i64, value: f64) -> projection::MetricPointV1 {
        projection::MetricPointV1 {
            cursor: 1,
            event_id: format!("point-{time_unix_nano}"),
            occurred_at: prometheus::nanos_rfc3339(time_unix_nano).unwrap(),
            time_unix_nano,
            value,
            stale: false,
            histogram: None,
            exemplars: Vec::new(),
        }
    }

    #[test]
    fn minimum_nanosecond_has_no_false_lookback_boundary() {
        assert_eq!(
            prom_latest_values(&[point(i64::MIN, 8.0)], &[i64::MIN]),
            vec![Some(8.0)]
        );
        assert!(prom_rate_values(
            &[point(i64::MIN, 1.0), point(i64::MIN + 1, 2.0)],
            &[i64::MIN + 1]
        )[0]
        .is_some());
    }
}

fn execute_query_v1(
    state: &ServiceState,
    request: &api::QueryRequestV1,
) -> Result<api::QueryResponseV1, ApiError> {
    state
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
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
    pub retention_generation: u64,
    pub retention_scan_pending: bool,
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

    let mut reader = state
        .journal
        .projection_read_session(0)
        .map_err(|error| ApiError::internal(format!("open integrity scan: {error}")))?;
    let mut event_count = 0_u64;
    let mut watermark = 0_u64;
    let mut event_id_digest = [0_u8; 32];
    let mut signals = IntegritySignalsV1::default();
    loop {
        let page = reader
            .read_next(10_000)
            .map_err(|error| ApiError::internal(format!("scan integrity events: {error}")))?;
        if page.is_empty() {
            break;
        }
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
                retention_generation: archive
                    .as_ref()
                    .map(|status| status.retention_generation)
                    .unwrap_or_default(),
                retention_scan_pending: archive
                    .as_ref()
                    .is_some_and(|status| status.retention_scan_pending),
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
        .journal
        .ensure_queryable()
        .map_err(|error| ApiError::temporarily_unavailable(error.to_string()))?;
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

pub(crate) fn retention_rejection(event: &EventEnvelope) -> Option<String> {
    retention_rejection_at(event, Utc::now())
}

fn retention_rejection_at(event: &EventEnvelope, decision_time: DateTime<Utc>) -> Option<String> {
    let occurred = DateTime::parse_from_rfc3339(&event.occurred_at).ok()?;
    let cutoff = decision_time - chrono::Duration::days(180);
    (occurred.with_timezone(&Utc) < cutoff).then(|| {
        format!(
            "event `{}` occurred before Sift's 180-day retention boundary",
            event.event_id
        )
    })
}
// HANDWRITE-END

// HANDWRITE-BEGIN gap="sift-projection-runtime" tracker="1660" reason="Register factories, restore atomic states, catch up asynchronously, wait for cursors, and rebuild from raw."
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use sha2::{Digest, Sha256};
use tokio::sync::Notify;

use crate::{DurableJournal, EventQuery, StoredEvent};

use super::{
    lumen::EmbeddedLumenProjection,
    model::{
        ProjectionCheckpoint, ProjectionDescriptor, ProjectionLag, ProjectionStateEnvelope,
        RebuildComparison, PROJECTION_STATE_FORMAT_VERSION,
    },
};

pub const PROJECTION_EVENT_INDEX: &str = "event-index";
pub const PROJECTION_BATCH_SIZE: usize = 1_000;
pub const PROJECTION_RETRY_AFTER_SECONDS: u64 = 1;

pub trait Projection: Send + Sync {
    fn descriptor(&self) -> ProjectionDescriptor;
    fn apply_idempotent(&self, event: &StoredEvent) -> Result<()>;
    fn snapshot(&self) -> Result<Vec<u8>>;
    fn restore(&self, state: &[u8]) -> Result<()>;

    fn semantic_digest(&self) -> Result<String> {
        Ok(sha256(&self.snapshot()?))
    }
}

type ProjectionFactory = Arc<dyn Fn() -> Result<Arc<dyn Projection>> + Send + Sync>;

struct LiveProjection {
    implementation: Arc<dyn Projection>,
    checkpoint: ProjectionCheckpoint,
}

struct ProjectionSlot {
    factory: ProjectionFactory,
    state_path: PathBuf,
    live: Mutex<LiveProjection>,
    published: Notify,
}

pub struct ProjectionRuntime {
    journal: Arc<DurableJournal>,
    slots: BTreeMap<String, Arc<ProjectionSlot>>,
}

impl ProjectionRuntime {
    pub fn open(data_dir: impl AsRef<Path>, journal: Arc<DurableJournal>) -> Result<Self> {
        let projection_root = data_dir.as_ref().join("projections");
        fs::create_dir_all(&projection_root).with_context(|| {
            format!(
                "create projection state root {}",
                projection_root.display()
            )
        })?;

        let mut slots = BTreeMap::new();
        let factory: ProjectionFactory = Arc::new(|| {
            Ok(Arc::new(EmbeddedLumenProjection::new()?) as Arc<dyn Projection>)
        });
        let implementation = factory()?;
        let descriptor = implementation.descriptor();
        let state_path = projection_root.join(&descriptor.name).join("state.json");
        let checkpoint = if state_path.exists() {
            let envelope: ProjectionStateEnvelope = serde_json::from_slice(
                &fs::read(&state_path)
                    .with_context(|| format!("read projection state {}", state_path.display()))?,
            )
            .with_context(|| format!("decode projection state {}", state_path.display()))?;
            validate_envelope(&descriptor, &envelope)?;
            let state = BASE64
                .decode(&envelope.state_base64)
                .context("decode projection state_base64")?;
            if sha256(&state) != envelope.checkpoint.state_sha256 {
                bail!(
                    "projection {} state checksum does not match its checkpoint",
                    descriptor.name
                );
            }
            implementation.restore(&state)?;
            envelope.checkpoint
        } else {
            ProjectionCheckpoint::empty(&descriptor)
        };
        slots.insert(
            descriptor.name.clone(),
            Arc::new(ProjectionSlot {
                factory,
                state_path,
                live: Mutex::new(LiveProjection {
                    implementation,
                    checkpoint,
                }),
                published: Notify::new(),
            }),
        );

        Ok(Self { journal, slots })
    }

    pub fn projection_names(&self) -> Vec<String> {
        self.slots.keys().cloned().collect()
    }

    pub fn has_projection(&self, name: &str) -> bool {
        self.slots.contains_key(name)
    }

    pub fn current_cursor(&self, name: &str) -> Result<u64> {
        let slot = self.slot(name)?;
        Ok(slot
            .live
            .lock()
            .expect("projection state lock poisoned")
            .checkpoint
            .cursor)
    }

    pub fn semantic_digest(&self, name: &str) -> Result<String> {
        let slot = self.slot(name)?;
        slot.live
            .lock()
            .expect("projection state lock poisoned")
            .implementation
            .semantic_digest()
    }

    pub fn catch_up(&self, name: &str) -> Result<u64> {
        let target = self.journal.last_cursor();
        let slot = self.slot(name)?;
        let mut live = slot.live.lock().expect("projection state lock poisoned");
        self.catch_up_locked(slot, &mut live, target)
    }

    pub async fn wait_for_min_cursor(
        &self,
        name: &str,
        required_cursor: u64,
        timeout: Duration,
    ) -> std::result::Result<u64, ProjectionLag> {
        let slot = match self.slots.get(name) {
            Some(slot) => slot,
            None => {
                return Err(ProjectionLag::new(
                    name,
                    required_cursor,
                    0,
                    PROJECTION_RETRY_AFTER_SECONDS,
                ))
            }
        };
        let started = Instant::now();
        loop {
            let published = slot.published.notified();
            let current = slot
                .live
                .lock()
                .expect("projection state lock poisoned")
                .checkpoint
                .cursor;
            if current >= required_cursor {
                return Ok(current);
            }
            let Some(remaining) = timeout.checked_sub(started.elapsed()) else {
                return Err(ProjectionLag::new(
                    name,
                    required_cursor,
                    current,
                    PROJECTION_RETRY_AFTER_SECONDS,
                ));
            };
            if tokio::time::timeout(remaining, published).await.is_err() {
                let current = slot
                    .live
                    .lock()
                    .expect("projection state lock poisoned")
                    .checkpoint
                    .cursor;
                return Err(ProjectionLag::new(
                    name,
                    required_cursor,
                    current,
                    PROJECTION_RETRY_AFTER_SECONDS,
                ));
            }
        }
    }

    pub fn rebuild_and_compare(&self, name: &str) -> Result<RebuildComparison> {
        let source_cursor = self.journal.last_cursor();
        let slot = self.slot(name)?;
        let mut live = slot.live.lock().expect("projection state lock poisoned");
        self.catch_up_locked(slot, &mut live, source_cursor)?;
        let live_digest = live.implementation.semantic_digest()?;

        let rebuilt = (slot.factory)()?;
        let mut after = 0;
        while after < source_cursor {
            let events = self.journal.query(EventQuery {
                signal: None,
                after,
                limit: PROJECTION_BATCH_SIZE,
            })?;
            if events.is_empty() {
                break;
            }
            for event in events.iter().filter(|event| event.cursor <= source_cursor) {
                rebuilt.apply_idempotent(event)?;
                after = event.cursor;
            }
            if events.last().is_some_and(|event| event.cursor > source_cursor) {
                break;
            }
        }
        if after != source_cursor {
            bail!(
                "projection {name} rebuild stopped at cursor {after}, expected {source_cursor}"
            );
        }
        let rebuilt_digest = rebuilt.semantic_digest()?;
        let equal = live_digest == rebuilt_digest;
        if equal {
            let descriptor = rebuilt.descriptor();
            let state = rebuilt.snapshot()?;
            let checkpoint = checkpoint(&descriptor, source_cursor, None, &state);
            persist(&slot.state_path, &checkpoint, &state)?;
            live.implementation = rebuilt;
            live.checkpoint = checkpoint;
            slot.published.notify_waiters();
        }

        Ok(RebuildComparison {
            source_cursor,
            rebuilt_cursor: after,
            live_digest,
            rebuilt_digest,
            equal,
        })
    }

    fn catch_up_locked(
        &self,
        slot: &ProjectionSlot,
        live: &mut LiveProjection,
        target: u64,
    ) -> Result<u64> {
        while live.checkpoint.cursor < target {
            let events = self.journal.query(EventQuery {
                signal: None,
                after: live.checkpoint.cursor,
                limit: PROJECTION_BATCH_SIZE,
            })?;
            if events.is_empty() {
                break;
            }
            let mut last_event_id = None;
            for event in events.iter().filter(|event| event.cursor <= target) {
                live.implementation.apply_idempotent(event)?;
                live.checkpoint.cursor = event.cursor;
                last_event_id = Some(event.event.event_id.clone());
            }
            if last_event_id.is_none() {
                break;
            }
            let state = live.implementation.snapshot()?;
            live.checkpoint = checkpoint(
                &live.implementation.descriptor(),
                live.checkpoint.cursor,
                last_event_id,
                &state,
            );
            persist(&slot.state_path, &live.checkpoint, &state)?;
            slot.published.notify_waiters();
        }
        Ok(live.checkpoint.cursor)
    }

    fn slot(&self, name: &str) -> Result<&Arc<ProjectionSlot>> {
        self.slots
            .get(name)
            .with_context(|| format!("unknown projection {name}"))
    }
}

fn checkpoint(
    descriptor: &ProjectionDescriptor,
    cursor: u64,
    event_id: Option<String>,
    state: &[u8],
) -> ProjectionCheckpoint {
    ProjectionCheckpoint {
        projection: descriptor.name.clone(),
        schema_version: descriptor.schema_version,
        cursor,
        event_id,
        state_sha256: sha256(state),
        updated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    }
}

fn persist(path: &Path, checkpoint: &ProjectionCheckpoint, state: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("projection state path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create projection directory {}", parent.display()))?;
    let envelope = ProjectionStateEnvelope {
        format_version: PROJECTION_STATE_FORMAT_VERSION,
        checkpoint: checkpoint.clone(),
        state_encoding: "base64".into(),
        state_base64: BASE64.encode(state),
    };
    service_durability::atomic_write(
        path,
        &serde_json::to_vec_pretty(&envelope)?,
        service_durability::FsyncPolicy::Always,
    )
    .with_context(|| format!("atomically persist projection state {}", path.display()))
}

fn validate_envelope(
    descriptor: &ProjectionDescriptor,
    envelope: &ProjectionStateEnvelope,
) -> Result<()> {
    if envelope.format_version != PROJECTION_STATE_FORMAT_VERSION {
        bail!(
            "unsupported projection state format {}",
            envelope.format_version
        );
    }
    if envelope.checkpoint.projection != descriptor.name
        || envelope.checkpoint.schema_version != descriptor.schema_version
    {
        bail!("projection checkpoint descriptor does not match registered projection");
    }
    if envelope.state_encoding != "base64" {
        bail!("unsupported projection state encoding");
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

<!-- marker: sift-projection-runtime path: projects/sift/src/projection/runtime.rs reason: Register factories, restore atomic states, catch up asynchronously, wait for cursors, and rebuild from raw. -->
// HANDWRITE-END

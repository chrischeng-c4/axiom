//! Shared typed projection runtime.
//!
//! The runtime owns checkpoints, catch-up, rebuild, publication, and flush.
//! Products keep typed handles and define their record and projection logic.

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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::sync::Notify;
use utoipa::ToSchema;

pub const PROJECTION_STATE_FORMAT_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionDescriptor {
    pub name: String,
    pub schema_version: u32,
    pub retention: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionCheckpoint {
    pub projection: String,
    pub schema_version: u32,
    pub cursor: u64,
    pub event_id: Option<String>,
    pub state_sha256: String,
    pub updated_at: String,
}

impl ProjectionCheckpoint {
    pub fn empty(descriptor: &ProjectionDescriptor) -> Self {
        Self {
            projection: descriptor.name.clone(),
            schema_version: descriptor.schema_version,
            cursor: 0,
            event_id: None,
            state_sha256: String::new(),
            updated_at: now(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionStateEnvelope {
    pub format_version: u16,
    pub checkpoint: ProjectionCheckpoint,
    pub state_encoding: String,
    pub state_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
pub struct ProjectionLag {
    pub error: String,
    pub projection: String,
    pub required_cursor: u64,
    pub current_cursor: u64,
    pub retryable: bool,
    pub retry_after_seconds: u64,
}

impl ProjectionLag {
    pub fn new(
        projection: impl Into<String>,
        required_cursor: u64,
        current_cursor: u64,
        retry_after_seconds: u64,
    ) -> Self {
        Self {
            error: "projection_lag".to_string(),
            projection: projection.into(),
            required_cursor,
            current_cursor,
            retryable: true,
            retry_after_seconds,
        }
    }
}

impl std::fmt::Display for ProjectionLag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "projection {} is at cursor {}, requires {}",
            self.projection, self.current_cursor, self.required_cursor
        )
    }
}

impl std::error::Error for ProjectionLag {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RebuildComparison {
    pub source_cursor: u64,
    pub rebuilt_cursor: u64,
    pub live_digest: String,
    pub rebuilt_digest: String,
    pub equal: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectionRuntimeConfig {
    pub batch_size: usize,
    pub snapshot_interval_events: u64,
    pub retry_after_seconds: u64,
}

impl ProjectionRuntimeConfig {
    pub fn new(batch_size: usize, snapshot_interval_events: u64, retry_after_seconds: u64) -> Self {
        Self {
            batch_size: batch_size.max(1),
            snapshot_interval_events: snapshot_interval_events.max(1),
            retry_after_seconds: retry_after_seconds.max(1),
        }
    }
}

pub trait ProjectionRecord: Clone + Send + Sync + 'static {
    fn projection_cursor(&self) -> u64;
    fn projection_event_id(&self) -> &str;
}

pub trait ProjectionSource<Record>: Send + Sync + 'static
where
    Record: ProjectionRecord,
{
    fn current_cursor(&self) -> u64;
    fn read_after(&self, after: u64, limit: usize) -> Result<Vec<Record>>;
}

pub trait Projection<Record>: Send + Sync + 'static
where
    Record: ProjectionRecord,
{
    fn descriptor(&self) -> ProjectionDescriptor;
    fn apply_idempotent(&self, record: &Record) -> Result<()>;
    fn snapshot(&self) -> Result<Vec<u8>>;
    fn restore(&self, state: &[u8]) -> Result<()>;

    fn semantic_digest(&self) -> Result<String> {
        Ok(sha256(&self.snapshot()?))
    }
}

struct LiveProjection<P> {
    implementation: Arc<P>,
    checkpoint: ProjectionCheckpoint,
    persisted_cursor: u64,
}

type ProjectionFactory<P> = Arc<dyn Fn() -> Result<Arc<P>> + Send + Sync>;

pub struct ProjectionHandle<Record, P>
where
    Record: ProjectionRecord,
    P: Projection<Record>,
{
    source: Arc<dyn ProjectionSource<Record>>,
    factory: ProjectionFactory<P>,
    state_path: PathBuf,
    live: Mutex<LiveProjection<P>>,
    published: Notify,
    config: ProjectionRuntimeConfig,
}

impl<Record, P> ProjectionHandle<Record, P>
where
    Record: ProjectionRecord,
    P: Projection<Record>,
{
    fn open(
        root: &Path,
        source: Arc<dyn ProjectionSource<Record>>,
        factory: ProjectionFactory<P>,
        config: ProjectionRuntimeConfig,
    ) -> Result<Self> {
        let implementation = factory()?;
        let descriptor = implementation.descriptor();
        validate_descriptor(&descriptor)?;
        let state_path = root.join(&descriptor.name).join("state.json");
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
        Ok(Self {
            source,
            factory,
            state_path,
            live: Mutex::new(LiveProjection {
                implementation,
                persisted_cursor: checkpoint.cursor,
                checkpoint,
            }),
            published: Notify::new(),
            config,
        })
    }

    pub fn projection(&self) -> Arc<P> {
        self.live
            .lock()
            .expect("projection state lock poisoned")
            .implementation
            .clone()
    }

    pub fn descriptor(&self) -> ProjectionDescriptor {
        self.projection().descriptor()
    }

    pub fn current_cursor(&self) -> u64 {
        self.live
            .lock()
            .expect("projection state lock poisoned")
            .checkpoint
            .cursor
    }

    pub fn semantic_digest(&self) -> Result<String> {
        self.projection().semantic_digest()
    }

    pub fn catch_up(&self) -> Result<u64> {
        let target = self.source.current_cursor();
        let mut live = self.live.lock().expect("projection state lock poisoned");
        self.catch_up_locked(&mut live, target)
    }

    pub async fn wait_for_min_cursor(
        &self,
        required_cursor: u64,
        timeout: Duration,
    ) -> std::result::Result<u64, ProjectionLag> {
        let started = Instant::now();
        loop {
            let published = self.published.notified();
            let current = self.current_cursor();
            if current >= required_cursor {
                return Ok(current);
            }
            let Some(remaining) = timeout.checked_sub(started.elapsed()) else {
                return Err(self.lag(required_cursor, current));
            };
            if tokio::time::timeout(remaining, published).await.is_err() {
                return Err(self.lag(required_cursor, self.current_cursor()));
            }
        }
    }

    pub fn rebuild_and_compare(&self) -> Result<RebuildComparison> {
        let source_cursor = self.source.current_cursor();
        {
            let mut live = self.live.lock().expect("projection state lock poisoned");
            self.catch_up_locked(&mut live, source_cursor)?;
        }
        let live_digest = self.semantic_digest()?;
        let rebuilt = (self.factory)()?;
        let mut after = 0;
        while after < source_cursor {
            let records = self.source.read_after(after, self.config.batch_size)?;
            if records.is_empty() {
                break;
            }
            for record in records
                .iter()
                .filter(|record| record.projection_cursor() <= source_cursor)
            {
                rebuilt.apply_idempotent(record)?;
                after = record.projection_cursor();
            }
            if records
                .last()
                .is_some_and(|record| record.projection_cursor() > source_cursor)
            {
                break;
            }
        }
        if after != source_cursor {
            bail!(
                "projection {} rebuild stopped at cursor {after}, expected {source_cursor}",
                rebuilt.descriptor().name
            );
        }
        let rebuilt_digest = rebuilt.semantic_digest()?;
        let equal = live_digest == rebuilt_digest;
        if equal {
            let descriptor = rebuilt.descriptor();
            let state = rebuilt.snapshot()?;
            let checkpoint = checkpoint(&descriptor, source_cursor, None, &state);
            persist(&self.state_path, &checkpoint, &state)?;
            let mut live = self.live.lock().expect("projection state lock poisoned");
            live.implementation = rebuilt;
            live.checkpoint = checkpoint;
            live.persisted_cursor = source_cursor;
            self.published.notify_waiters();
        }
        Ok(RebuildComparison {
            source_cursor,
            rebuilt_cursor: after,
            live_digest,
            rebuilt_digest,
            equal,
        })
    }

    pub fn flush(&self) -> Result<()> {
        let target = self.source.current_cursor();
        let mut live = self.live.lock().expect("projection state lock poisoned");
        self.catch_up_locked(&mut live, target)?;
        if live.checkpoint.cursor != live.persisted_cursor {
            self.persist_live(&mut live)?;
        }
        Ok(())
    }

    fn catch_up_locked(&self, live: &mut LiveProjection<P>, target: u64) -> Result<u64> {
        while live.checkpoint.cursor < target {
            let records = self
                .source
                .read_after(live.checkpoint.cursor, self.config.batch_size)?;
            if records.is_empty() {
                break;
            }
            let mut last_event_id = None;
            for record in records
                .iter()
                .filter(|record| record.projection_cursor() <= target)
            {
                live.implementation.apply_idempotent(record)?;
                live.checkpoint.cursor = record.projection_cursor();
                last_event_id = Some(record.projection_event_id().to_string());
            }
            if last_event_id.is_none() {
                break;
            }
            live.checkpoint.event_id = last_event_id;
            self.published.notify_waiters();
        }
        let advanced = live.checkpoint.cursor.saturating_sub(live.persisted_cursor);
        if live.checkpoint.cursor > live.persisted_cursor
            && (live.persisted_cursor == 0 || advanced >= self.config.snapshot_interval_events)
        {
            self.persist_live(live)?;
        }
        Ok(live.checkpoint.cursor)
    }

    fn persist_live(&self, live: &mut LiveProjection<P>) -> Result<()> {
        let state = live.implementation.snapshot()?;
        let checkpoint = checkpoint(
            &live.implementation.descriptor(),
            live.checkpoint.cursor,
            live.checkpoint.event_id.clone(),
            &state,
        );
        persist(&self.state_path, &checkpoint, &state)?;
        live.persisted_cursor = checkpoint.cursor;
        live.checkpoint = checkpoint;
        Ok(())
    }

    fn lag(&self, required_cursor: u64, current_cursor: u64) -> ProjectionLag {
        ProjectionLag::new(
            self.descriptor().name,
            required_cursor,
            current_cursor,
            self.config.retry_after_seconds,
        )
    }
}

trait ProjectionControl: Send + Sync {
    fn current_cursor(&self) -> u64;
    fn catch_up(&self) -> Result<u64>;
    fn semantic_digest(&self) -> Result<String>;
    fn rebuild_and_compare(&self) -> Result<RebuildComparison>;
    fn flush(&self) -> Result<()>;
}

impl<Record, P> ProjectionControl for ProjectionHandle<Record, P>
where
    Record: ProjectionRecord,
    P: Projection<Record>,
{
    fn current_cursor(&self) -> u64 {
        ProjectionHandle::current_cursor(self)
    }

    fn catch_up(&self) -> Result<u64> {
        ProjectionHandle::catch_up(self)
    }

    fn semantic_digest(&self) -> Result<String> {
        ProjectionHandle::semantic_digest(self)
    }

    fn rebuild_and_compare(&self) -> Result<RebuildComparison> {
        ProjectionHandle::rebuild_and_compare(self)
    }

    fn flush(&self) -> Result<()> {
        ProjectionHandle::flush(self)
    }
}

pub struct ProjectionRegistry<Record>
where
    Record: ProjectionRecord,
{
    root: PathBuf,
    source: Arc<dyn ProjectionSource<Record>>,
    config: ProjectionRuntimeConfig,
    controls: BTreeMap<String, Arc<dyn ProjectionControl>>,
}

impl<Record> ProjectionRegistry<Record>
where
    Record: ProjectionRecord,
{
    pub fn new(
        root: impl AsRef<Path>,
        source: Arc<dyn ProjectionSource<Record>>,
        config: ProjectionRuntimeConfig,
    ) -> Result<Self> {
        let root = root.as_ref().join("indexes");
        fs::create_dir_all(&root)
            .with_context(|| format!("create projection state root {}", root.display()))?;
        set_directory_mode(&root)?;
        Ok(Self {
            root,
            source,
            config,
            controls: BTreeMap::new(),
        })
    }

    pub fn register<P, Factory>(
        &mut self,
        factory: Factory,
    ) -> Result<Arc<ProjectionHandle<Record, P>>>
    where
        P: Projection<Record>,
        Factory: Fn() -> Result<Arc<P>> + Send + Sync + 'static,
    {
        let handle = Arc::new(ProjectionHandle::open(
            &self.root,
            self.source.clone(),
            Arc::new(factory),
            self.config,
        )?);
        let name = handle.descriptor().name;
        if self.controls.contains_key(&name) {
            bail!("projection {name} is registered more than once");
        }
        self.controls.insert(name, handle.clone());
        Ok(handle)
    }

    pub fn projection_names(&self) -> Vec<String> {
        self.controls.keys().cloned().collect()
    }

    pub fn has_projection(&self, name: &str) -> bool {
        self.controls.contains_key(name)
    }

    pub fn current_cursor(&self, name: &str) -> Result<u64> {
        Ok(self.control(name)?.current_cursor())
    }

    pub fn catch_up(&self, name: &str) -> Result<u64> {
        self.control(name)?.catch_up()
    }

    pub fn semantic_digest(&self, name: &str) -> Result<String> {
        self.control(name)?.semantic_digest()
    }

    pub fn rebuild_and_compare(&self, name: &str) -> Result<RebuildComparison> {
        self.control(name)?.rebuild_and_compare()
    }

    pub fn flush_all(&self) -> Result<()> {
        for control in self.controls.values() {
            control.flush()?;
        }
        Ok(())
    }

    fn control(&self, name: &str) -> Result<&Arc<dyn ProjectionControl>> {
        self.controls
            .get(name)
            .with_context(|| format!("unknown projection {name}"))
    }
}

fn validate_descriptor(descriptor: &ProjectionDescriptor) -> Result<()> {
    if descriptor.name.trim().is_empty()
        || descriptor.name.contains('/')
        || descriptor.name.contains('\0')
    {
        bail!("projection name is invalid");
    }
    Ok(())
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
        updated_at: now(),
    }
}

fn persist(path: &Path, checkpoint: &ProjectionCheckpoint, state: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("projection state path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create projection directory {}", parent.display()))?;
    set_directory_mode(parent)?;
    let envelope = ProjectionStateEnvelope {
        format_version: PROJECTION_STATE_FORMAT_VERSION,
        checkpoint: checkpoint.clone(),
        state_encoding: "base64".to_string(),
        state_base64: BASE64.encode(state),
    };
    storage_durable::atomic_write(
        path,
        &serde_json::to_vec_pretty(&envelope)?,
        storage_durable::FsyncPolicy::Always,
    )
    .with_context(|| format!("atomically persist projection state {}", path.display()))?;
    set_file_mode(path)
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(unix)]
fn set_directory_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).with_context(|| {
        format!(
            "set private projection directory mode on {}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn set_directory_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("set private projection file mode on {}", path.display()))
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}

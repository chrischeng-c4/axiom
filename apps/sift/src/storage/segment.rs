// HANDWRITE-BEGIN gap="sift-sealed-segment-store" tracker="1659" reason="Append CRC frames per epoch/shard, recover torn tails, seal manifests, and move immutable segments without rewriting bytes."
use std::{
    collections::{HashMap, HashSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{bail, Context, Result};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::StoredEvent;

use super::shard::{bucket_for, Route};

const FRAMED_LOG_HEADER_BYTES: u64 = 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentState {
    Sealed,
    Moved,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SegmentManifest {
    pub segment_id: String,
    pub epoch: u64,
    pub shard: u16,
    pub bucket_min: u16,
    pub bucket_max: u16,
    pub first_cursor: u64,
    pub last_cursor: u64,
    pub event_count: u64,
    #[serde(default)]
    pub min_event_time_unix_nano: i64,
    #[serde(default)]
    pub max_event_time_unix_nano: i64,
    pub bytes: u64,
    pub sha256: String,
    pub state: SegmentState,
    pub local_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_uri: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendLocation {
    pub route: Route,
    pub segment_id: String,
    pub path: PathBuf,
}

struct ActiveSegment {
    route: Route,
    segment_id: String,
    path: PathBuf,
    writer: storage_durable::FramedLogWriter,
    first_cursor: u64,
    last_cursor: u64,
    event_count: u64,
    min_event_time_unix_nano: i64,
    max_event_time_unix_nano: i64,
    encoded_bytes: u64,
    bucket_min: u16,
    bucket_max: u16,
}

#[derive(Default)]
struct SegmentStateData {
    active: HashMap<(u64, u16), ActiveSegment>,
    sealed: HashMap<String, SegmentManifest>,
    cursors: HashMap<u64, (String, AppendLocation)>,
}

pub struct SegmentStore {
    root: PathBuf,
    manifests_root: PathBuf,
    max_segment_events: usize,
    max_segment_bytes: usize,
    inner: Mutex<SegmentStateData>,
}

impl SegmentStore {
    pub(crate) fn open_at(
        root: PathBuf,
        max_segment_events: usize,
        max_segment_bytes: usize,
    ) -> Result<Self> {
        if max_segment_events == 0 {
            bail!("max_segment_events must be greater than zero");
        }
        if max_segment_bytes == 0 {
            bail!("max_segment_bytes must be greater than zero");
        }
        let manifests_root = root.join("manifests");
        fs::create_dir_all(&manifests_root)?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        fs::set_permissions(&manifests_root, fs::Permissions::from_mode(0o700))?;
        let store = Self {
            root,
            manifests_root,
            max_segment_events,
            max_segment_bytes,
            inner: Mutex::new(SegmentStateData::default()),
        };
        store.load()?;
        Ok(store)
    }

    fn load(&self) -> Result<()> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        for path in files_with_extension(&self.manifests_root, "json")? {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            let manifest: SegmentManifest = serde_json::from_slice(&fs::read(&path)?)
                .with_context(|| format!("decode segment manifest {}", path.display()))?;
            fs::set_permissions(&manifest.local_path, fs::Permissions::from_mode(0o600))?;
            verify_segment(&manifest)?;
            state.sealed.insert(manifest.segment_id.clone(), manifest);
        }
        let sealed_paths = state
            .sealed
            .values()
            .map(|manifest| manifest.local_path.clone())
            .collect::<HashSet<_>>();
        for path in files_with_extension(&self.root, "open")? {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            if sealed_paths.contains(&path) {
                continue;
            }
            let (epoch, shard) = route_from_path(&path)?;
            let writer = storage_durable::FramedLogWriter::open(
                &path,
                storage_durable::FsyncPolicy::Interval,
            )?;
            if let Some(shard_root) = path.parent() {
                fs::set_permissions(shard_root, fs::Permissions::from_mode(0o700))?;
                if let Some(epoch_root) = shard_root.parent() {
                    fs::set_permissions(epoch_root, fs::Permissions::from_mode(0o700))?;
                }
            }
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            let events = read_events(&path)?;
            if events.is_empty() {
                continue;
            }
            let encoded_bytes = fs::metadata(&path)?.len();
            let segment_id = path
                .file_stem()
                .and_then(|value| value.to_str())
                .context("active segment file has invalid name")?
                .to_string();
            let mut bucket_min = u16::MAX;
            let mut bucket_max = 0;
            let mut min_event_time_unix_nano = i64::MAX;
            let mut max_event_time_unix_nano = i64::MIN;
            for event in &events {
                let bucket = bucket_for(&event.event.event_id);
                let event_time = event_time_unix_nano(event)?;
                bucket_min = bucket_min.min(bucket);
                bucket_max = bucket_max.max(bucket);
                min_event_time_unix_nano = min_event_time_unix_nano.min(event_time);
                max_event_time_unix_nano = max_event_time_unix_nano.max(event_time);
                self.index_event(
                    &mut state,
                    event,
                    AppendLocation {
                        route: Route {
                            epoch,
                            shard,
                            bucket,
                        },
                        segment_id: segment_id.clone(),
                        path: path.clone(),
                    },
                )?;
            }
            state.active.insert(
                (epoch, shard),
                ActiveSegment {
                    route: Route {
                        epoch,
                        shard,
                        bucket: bucket_min,
                    },
                    segment_id,
                    path,
                    writer,
                    first_cursor: events.first().unwrap().cursor,
                    last_cursor: events.last().unwrap().cursor,
                    event_count: events.len() as u64,
                    min_event_time_unix_nano,
                    max_event_time_unix_nano,
                    encoded_bytes,
                    bucket_min,
                    bucket_max,
                },
            );
        }
        let full_segments = state
            .active
            .iter()
            .filter_map(|(key, active)| self.segment_is_ready(active).then_some(*key))
            .collect::<Vec<_>>();
        for key in full_segments {
            self.seal_locked(&mut state, key)?;
        }
        Ok(())
    }

    fn index_event(
        &self,
        state: &mut SegmentStateData,
        event: &StoredEvent,
        location: AppendLocation,
    ) -> Result<()> {
        if let Some((existing_id, _)) = state.cursors.get(&event.cursor) {
            if existing_id != &event.event.event_id {
                bail!(
                    "raw segments contain conflicting cursor {} for {} and {}",
                    event.cursor,
                    existing_id,
                    event.event.event_id
                );
            }
            return Ok(());
        }
        state
            .cursors
            .insert(event.cursor, (event.event.event_id.clone(), location));
        Ok(())
    }

    pub fn append(&self, route: Route, stored: &StoredEvent) -> Result<AppendLocation> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        if let Some((event_id, location)) = state.cursors.get(&stored.cursor) {
            if event_id != &stored.event.event_id {
                bail!("cursor {} already belongs to {event_id}", stored.cursor);
            }
            return Ok(location.clone());
        }
        let key = (route.epoch, route.shard);
        let event_time = event_time_unix_nano(stored)?;
        if let std::collections::hash_map::Entry::Vacant(entry) = state.active.entry(key) {
            let segment_id = format!(
                "segment-e{:020}-s{:04}-c{:020}",
                route.epoch, route.shard, stored.cursor
            );
            let path = self
                .root
                .join(format!("epoch-{:020}", route.epoch))
                .join(format!("shard-{:04}", route.shard))
                .join(format!("{segment_id}.open"));
            let writer = storage_durable::FramedLogWriter::open(
                &path,
                storage_durable::FsyncPolicy::Interval,
            )?;
            entry.insert(ActiveSegment {
                route,
                segment_id,
                path,
                writer,
                first_cursor: stored.cursor,
                last_cursor: stored.cursor,
                event_count: 0,
                min_event_time_unix_nano: event_time,
                max_event_time_unix_nano: event_time,
                encoded_bytes: 0,
                bucket_min: route.bucket,
                bucket_max: route.bucket,
            });
        }
        let encoded = serde_json::to_vec(stored)?;
        let (location, should_seal) = {
            let active = state.active.get_mut(&key).unwrap();
            active.writer.append(stored.cursor, &encoded)?;
            active.last_cursor = stored.cursor;
            active.event_count += 1;
            active.min_event_time_unix_nano = active.min_event_time_unix_nano.min(event_time);
            active.max_event_time_unix_nano = active.max_event_time_unix_nano.max(event_time);
            active.encoded_bytes = active
                .encoded_bytes
                .saturating_add(FRAMED_LOG_HEADER_BYTES)
                .saturating_add(encoded.len() as u64);
            active.bucket_min = active.bucket_min.min(route.bucket);
            active.bucket_max = active.bucket_max.max(route.bucket);
            (
                AppendLocation {
                    route,
                    segment_id: active.segment_id.clone(),
                    path: active.path.clone(),
                },
                self.segment_is_ready(active),
            )
        };
        self.index_event(&mut state, stored, location.clone())?;
        // Sealing performs fsync, hashing, rename, and manifest replacement.
        // Keep it out of the acknowledgement path. The background storage
        // worker calls `seal_ready`; an explicit archive calls `seal_all`.
        let _ = should_seal;
        Ok(location)
    }

    pub fn seal_ready(&self) -> Result<Vec<SegmentManifest>> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        let keys = state
            .active
            .iter()
            .filter_map(|(key, active)| self.segment_is_ready(active).then_some(*key))
            .collect::<Vec<_>>();
        let mut manifests = Vec::with_capacity(keys.len());
        for key in keys {
            manifests.push(self.seal_locked(&mut state, key)?);
        }
        Ok(manifests)
    }

    pub fn flush_active(&self) -> Result<()> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        for active in state.active.values_mut() {
            active.writer.flush()?;
        }
        Ok(())
    }

    fn segment_is_ready(&self, active: &ActiveSegment) -> bool {
        active.event_count as usize >= self.max_segment_events
            || active.encoded_bytes >= self.max_segment_bytes as u64
    }

    fn seal_locked(
        &self,
        state: &mut SegmentStateData,
        key: (u64, u16),
    ) -> Result<SegmentManifest> {
        let mut active = state
            .active
            .remove(&key)
            .context("active segment disappeared before seal")?;
        active.writer.sync()?;
        drop(active.writer);
        let sealed_path = active.path.with_extension("framed");
        fs::rename(&active.path, &sealed_path)?;
        storage_durable::sync_parent_dir(&sealed_path)?;
        let bytes = fs::metadata(&sealed_path)?.len();
        let manifest = SegmentManifest {
            segment_id: active.segment_id,
            epoch: active.route.epoch,
            shard: active.route.shard,
            bucket_min: active.bucket_min,
            bucket_max: active.bucket_max,
            first_cursor: active.first_cursor,
            last_cursor: active.last_cursor,
            event_count: active.event_count,
            min_event_time_unix_nano: active.min_event_time_unix_nano,
            max_event_time_unix_nano: active.max_event_time_unix_nano,
            bytes,
            sha256: sha256_file(&sealed_path)?,
            state: SegmentState::Sealed,
            local_path: sealed_path.clone(),
            object_uri: None,
        };
        self.write_manifest(&manifest)?;
        state
            .cursors
            .retain(|_, (_, location)| location.segment_id != manifest.segment_id);
        state
            .sealed
            .insert(manifest.segment_id.clone(), manifest.clone());
        Ok(manifest)
    }

    fn write_manifest(&self, manifest: &SegmentManifest) -> Result<()> {
        let path = self
            .manifests_root
            .join(format!("{}.json", manifest.segment_id));
        storage_durable::atomic_write(
            &path,
            &serde_json::to_vec_pretty(manifest)?,
            storage_durable::FsyncPolicy::Always,
        )?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(())
    }

    pub fn seal_all(&self) -> Result<Vec<SegmentManifest>> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        let keys = state.active.keys().copied().collect::<Vec<_>>();
        for key in keys {
            self.seal_locked(&mut state, key)?;
        }
        let mut manifests = state.sealed.values().cloned().collect::<Vec<_>>();
        manifests.sort_by_key(|manifest| manifest.first_cursor);
        Ok(manifests)
    }

    pub fn manifests(&self) -> Result<Vec<SegmentManifest>> {
        let state = self.inner.lock().expect("segment state lock poisoned");
        let mut manifests = state.sealed.values().cloned().collect::<Vec<_>>();
        manifests.sort_by_key(|manifest| manifest.first_cursor);
        Ok(manifests)
    }

    /// Materialize the retained subset of a sealed segment before the remote
    /// retention manifest commits. The old segment remains in place until the
    /// caller commits that manifest. A crash can therefore leave duplicate,
    /// byte-identical cursors, but it cannot lose an acknowledged event.
    pub(crate) fn write_retained_segment(
        &self,
        source_segment_id: &str,
        retained: &[StoredEvent],
    ) -> Result<Option<SegmentManifest>> {
        self.write_derived_segment(source_segment_id, retained, true)
    }

    /// Materialize a prefix taken from a hash-verified archive checkpoint.
    /// The archive is authoritative, so its row can replace a conflicting row
    /// at the same local cursor. Normal retention continues to use the stricter
    /// `write_retained_segment` path above.
    pub(crate) fn write_reconciled_segment(
        &self,
        source_segment_id: &str,
        retained: &[StoredEvent],
    ) -> Result<Option<SegmentManifest>> {
        self.write_derived_segment(source_segment_id, retained, false)
    }

    fn write_derived_segment(
        &self,
        source_segment_id: &str,
        retained: &[StoredEvent],
        require_source_match: bool,
    ) -> Result<Option<SegmentManifest>> {
        if retained.is_empty() {
            bail!("retained local segment cannot be empty");
        }
        if retained
            .windows(2)
            .any(|pair| pair[0].cursor >= pair[1].cursor)
        {
            bail!("retained local segment cursors must be strictly increasing");
        }

        let mut state = self.inner.lock().expect("segment state lock poisoned");
        let Some(source) = state.sealed.get(source_segment_id).cloned() else {
            return Ok(None);
        };
        verify_segment(&source)?;
        if require_source_match {
            let source_events = read_events(&source.local_path)?;
            let mut source_index = source_events
                .iter()
                .map(|event| (event.cursor, event))
                .collect::<HashMap<_, _>>();
            for event in retained {
                let Some(source_event) = source_index.remove(&event.cursor) else {
                    bail!(
                        "retained cursor {} is absent from local segment {}",
                        event.cursor,
                        source.segment_id
                    );
                };
                if source_event != event {
                    bail!(
                        "retained cursor {} disagrees with local segment {}",
                        event.cursor,
                        source.segment_id
                    );
                }
            }
        }

        let mut identity = Sha256::new();
        identity.update(source.epoch.to_le_bytes());
        identity.update(source.shard.to_le_bytes());
        for event in retained {
            let encoded = serde_json::to_vec(event)?;
            identity.update(event.cursor.to_le_bytes());
            identity.update((encoded.len() as u64).to_le_bytes());
            identity.update(encoded);
        }
        let identity = hex::encode(identity.finalize());
        let segment_id = format!("retained-{}", &identity[..32]);
        if let Some(existing) = state.sealed.get(&segment_id).cloned() {
            verify_segment(&existing)?;
            if read_events(&existing.local_path)? != retained {
                bail!("retained segment identity collision for {segment_id}");
            }
            return Ok(Some(existing));
        }

        let parent = source
            .local_path
            .parent()
            .context("sealed segment has no parent directory")?;
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        let sealed_path = parent.join(format!("{segment_id}.framed"));
        if sealed_path.exists() {
            if read_events(&sealed_path)? != retained {
                bail!(
                    "uncommitted retained segment {} has unexpected contents",
                    sealed_path.display()
                );
            }
        } else {
            let rewrite_path = parent.join(format!("{segment_id}.rewrite"));
            match fs::remove_file(&rewrite_path) {
                Ok(()) => storage_durable::sync_parent_dir(&rewrite_path)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let mut writer = storage_durable::FramedLogWriter::open(
                &rewrite_path,
                storage_durable::FsyncPolicy::Interval,
            )?;
            for event in retained {
                writer.append(event.cursor, &serde_json::to_vec(event)?)?;
            }
            writer.sync()?;
            drop(writer);
            fs::rename(&rewrite_path, &sealed_path).with_context(|| {
                format!(
                    "commit retained segment {} -> {}",
                    rewrite_path.display(),
                    sealed_path.display()
                )
            })?;
            storage_durable::sync_parent_dir(&sealed_path)?;
        }
        fs::set_permissions(&sealed_path, fs::Permissions::from_mode(0o600))?;

        let mut bucket_min = u16::MAX;
        let mut bucket_max = 0_u16;
        let mut min_event_time_unix_nano = i64::MAX;
        let mut max_event_time_unix_nano = i64::MIN;
        for event in retained {
            let bucket = bucket_for(&event.event.event_id);
            let event_time = event_time_unix_nano(event)?;
            bucket_min = bucket_min.min(bucket);
            bucket_max = bucket_max.max(bucket);
            min_event_time_unix_nano = min_event_time_unix_nano.min(event_time);
            max_event_time_unix_nano = max_event_time_unix_nano.max(event_time);
        }
        let manifest = SegmentManifest {
            segment_id: segment_id.clone(),
            epoch: source.epoch,
            shard: source.shard,
            bucket_min,
            bucket_max,
            first_cursor: retained.first().expect("retained is non-empty").cursor,
            last_cursor: retained.last().expect("retained is non-empty").cursor,
            event_count: retained.len() as u64,
            min_event_time_unix_nano,
            max_event_time_unix_nano,
            bytes: fs::metadata(&sealed_path)?.len(),
            sha256: sha256_file(&sealed_path)?,
            state: SegmentState::Sealed,
            local_path: sealed_path,
            object_uri: None,
        };
        self.write_manifest(&manifest)?;
        state.sealed.insert(segment_id, manifest.clone());
        Ok(Some(manifest))
    }

    /// Remove one local immutable copy after its remote archive receipt is
    /// durable. Renaming the manifest first is the crash boundary: a restart
    /// cannot reopen the segment after that rename.
    pub(crate) fn evict_segment(
        &self,
        segment_id: &str,
        receipt_root: &Path,
    ) -> Result<Option<SegmentManifest>> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        let Some(manifest) = state.sealed.get(segment_id).cloned() else {
            return Ok(None);
        };
        verify_segment(&manifest)?;
        fs::create_dir_all(receipt_root)?;
        fs::set_permissions(receipt_root, fs::Permissions::from_mode(0o700))?;
        let source_manifest = self.manifests_root.join(format!("{segment_id}.json"));
        let receipt = receipt_root.join(format!("{segment_id}.json"));
        if receipt.exists() {
            let prior: SegmentManifest = serde_json::from_slice(&fs::read(&receipt)?)?;
            if prior != manifest {
                bail!("local eviction receipt for {segment_id} changed");
            }
            if source_manifest.exists() {
                fs::remove_file(&source_manifest)?;
                storage_durable::sync_parent_dir(&source_manifest)?;
            }
        } else {
            fs::rename(&source_manifest, &receipt).with_context(|| {
                format!(
                    "move local segment manifest {} to eviction receipt {}",
                    source_manifest.display(),
                    receipt.display()
                )
            })?;
            fs::set_permissions(&receipt, fs::Permissions::from_mode(0o600))?;
            storage_durable::sync_parent_dir(&source_manifest)?;
            storage_durable::sync_parent_dir(&receipt)?;
        }
        match fs::remove_file(&manifest.local_path) {
            Ok(()) => storage_durable::sync_parent_dir(&manifest.local_path)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        state.sealed.remove(segment_id);
        Ok(Some(manifest))
    }

    pub fn query_events(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let state = self.inner.lock().expect("segment state lock poisoned");
        let mut paths = state
            .sealed
            .values()
            .map(|manifest| {
                (
                    manifest.first_cursor,
                    manifest.last_cursor,
                    manifest.local_path.clone(),
                )
            })
            .chain(
                state
                    .active
                    .values()
                    .map(|active| (active.first_cursor, active.last_cursor, active.path.clone())),
            )
            .filter(|(_, last, _)| *last > after)
            .collect::<Vec<_>>();
        drop(state);
        paths.sort_by_key(|(first, _, _)| *first);

        let mut events = Vec::with_capacity(limit.min(1_000));
        for (_, _, path) in paths {
            let remaining = limit.saturating_sub(events.len());
            events.extend(read_events_after(&path, after, remaining)?);
            if events.len() == limit {
                return Ok(events);
            }
        }
        Ok(events)
    }

    pub(crate) fn read_manifest_events(
        &self,
        manifest: &SegmentManifest,
    ) -> Result<Vec<StoredEvent>> {
        let state = self.inner.lock().expect("segment state lock poisoned");
        let owned = state
            .sealed
            .get(&manifest.segment_id)
            .filter(|owned| *owned == manifest)
            .is_some();
        drop(state);
        if !owned {
            bail!(
                "segment {} is not owned by this signal store",
                manifest.segment_id
            );
        }
        verify_segment(manifest)?;
        let events = read_events(&manifest.local_path)?;
        if events.len() as u64 != manifest.event_count
            || events.first().map(|event| event.cursor) != Some(manifest.first_cursor)
            || events.last().map(|event| event.cursor) != Some(manifest.last_cursor)
        {
            bail!(
                "segment {} content does not match its manifest",
                manifest.segment_id
            );
        }
        Ok(events)
    }

    pub fn recovered_events(&self) -> Result<Vec<StoredEvent>> {
        let state = self.inner.lock().expect("segment state lock poisoned");
        let mut paths = state
            .sealed
            .values()
            .map(|manifest| manifest.local_path.clone())
            .chain(state.active.values().map(|active| active.path.clone()))
            .collect::<Vec<_>>();
        paths.sort();
        paths.dedup();
        drop(state);
        let mut by_cursor = HashMap::new();
        for path in paths {
            for event in read_events(&path)? {
                if let Some(existing) = by_cursor.insert(event.cursor, event.clone()) {
                    if existing.event.event_id != event.event.event_id {
                        bail!("conflicting raw segment cursor {}", event.cursor);
                    }
                }
            }
        }
        let mut events = by_cursor.into_values().collect::<Vec<_>>();
        events.sort_by_key(|event| event.cursor);
        Ok(events)
    }

    pub fn active_paths(&self) -> Vec<PathBuf> {
        self.inner
            .lock()
            .expect("segment state lock poisoned")
            .active
            .values()
            .map(|active| active.path.clone())
            .collect()
    }

    pub fn move_segment(&self, segment_id: &str, destination: &Path) -> Result<SegmentManifest> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        let mut manifest = state
            .sealed
            .get(segment_id)
            .cloned()
            .with_context(|| format!("unknown sealed segment {segment_id}"))?;
        verify_segment(&manifest)?;
        fs::create_dir_all(destination)?;
        let target = destination.join(
            manifest
                .local_path
                .file_name()
                .context("segment path has no file name")?,
        );
        match fs::rename(&manifest.local_path, &target) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {
                let bytes = fs::read(&manifest.local_path)?;
                storage_durable::atomic_write(
                    &target,
                    &bytes,
                    storage_durable::FsyncPolicy::Always,
                )?;
                let mut copied = manifest.clone();
                copied.local_path = target.clone();
                verify_segment(&copied)?;
                fs::remove_file(&manifest.local_path)?;
                storage_durable::sync_parent_dir(&manifest.local_path)?;
            }
            Err(error) => return Err(error.into()),
        }
        storage_durable::sync_parent_dir(&target)?;
        manifest.local_path = target.clone();
        manifest.state = SegmentState::Moved;
        verify_segment(&manifest)?;
        self.write_manifest(&manifest)?;
        state
            .sealed
            .insert(segment_id.to_string(), manifest.clone());
        Ok(manifest)
    }
}

fn read_events(path: &Path) -> Result<Vec<StoredEvent>> {
    storage_durable::FramedLogReader::read_frames(path, 0)?
        .into_iter()
        .map(|frame| {
            let event: StoredEvent = serde_json::from_slice(&frame.payload)?;
            if event.cursor != frame.seq {
                bail!(
                    "segment frame {} contains cursor {} in {}",
                    frame.seq,
                    event.cursor,
                    path.display()
                );
            }
            Ok(event)
        })
        .collect()
}

fn event_time_unix_nano(event: &StoredEvent) -> Result<i64> {
    DateTime::parse_from_rfc3339(&event.event.occurred_at)
        .context("segment event occurred_at must be RFC3339")?
        .timestamp_nanos_opt()
        .context("segment event occurred_at is outside the nanosecond range")
}

fn read_events_after(path: &Path, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
    storage_durable::FramedLogReader::read_frames_bounded(path, after, limit)?
        .into_iter()
        .map(|frame| {
            let event: StoredEvent = serde_json::from_slice(&frame.payload)?;
            if event.cursor != frame.seq {
                bail!(
                    "segment frame {} contains cursor {} in {}",
                    frame.seq,
                    event.cursor,
                    path.display()
                );
            }
            Ok(event)
        })
        .collect()
}

fn sha256_file(path: &Path) -> Result<String> {
    Ok(hex::encode(Sha256::digest(fs::read(path)?)))
}

fn verify_segment(manifest: &SegmentManifest) -> Result<()> {
    let bytes = fs::metadata(&manifest.local_path)
        .with_context(|| format!("stat segment {}", manifest.local_path.display()))?
        .len();
    if bytes != manifest.bytes || sha256_file(&manifest.local_path)? != manifest.sha256 {
        bail!(
            "sealed segment {} failed size/hash verification",
            manifest.segment_id
        );
    }
    Ok(())
}

fn route_from_path(path: &Path) -> Result<(u64, u16)> {
    let shard = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_prefix("shard-"))
        .context("segment path lacks shard directory")?
        .parse()?;
    let epoch = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_prefix("epoch-"))
        .context("segment path lacks epoch directory")?
        .parse()?;
    Ok((epoch, shard))
}

fn files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut output = Vec::new();
    if !root.exists() {
        return Ok(output);
    }
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some(extension) {
                output.push(entry.path());
            }
        }
    }
    output.sort();
    Ok(output)
}
// HANDWRITE-END

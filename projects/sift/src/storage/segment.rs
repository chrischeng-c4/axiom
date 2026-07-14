// HANDWRITE-BEGIN gap="sift-sealed-segment-store" tracker="1659" reason="Append CRC frames per epoch/shard, recover torn tails, seal manifests, and move immutable segments without rewriting bytes."
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::StoredEvent;

use super::shard::{bucket_for, Route};

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
    writer: service_durability::FramedLogWriter,
    first_cursor: u64,
    last_cursor: u64,
    event_count: u64,
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
    inner: Mutex<SegmentStateData>,
}

impl SegmentStore {
    pub fn open(root: impl AsRef<Path>, max_segment_events: usize) -> Result<Self> {
        if max_segment_events == 0 {
            bail!("max_segment_events must be greater than zero");
        }
        let root = root.as_ref().join("segments");
        let manifests_root = root.join("manifests");
        fs::create_dir_all(&manifests_root)?;
        let store = Self {
            root,
            manifests_root,
            max_segment_events,
            inner: Mutex::new(SegmentStateData::default()),
        };
        store.load()?;
        Ok(store)
    }

    fn load(&self) -> Result<()> {
        let mut state = self.inner.lock().expect("segment state lock poisoned");
        for path in files_with_extension(&self.manifests_root, "json")? {
            let manifest: SegmentManifest = serde_json::from_slice(&fs::read(&path)?)
                .with_context(|| format!("decode segment manifest {}", path.display()))?;
            verify_segment(&manifest)?;
            self.index_file(&mut state, &manifest.local_path, manifest.epoch, manifest.shard)?;
            state.sealed.insert(manifest.segment_id.clone(), manifest);
        }
        let sealed_paths = state
            .sealed
            .values()
            .map(|manifest| manifest.local_path.clone())
            .collect::<HashSet<_>>();
        for path in files_with_extension(&self.root, "open")? {
            if sealed_paths.contains(&path) {
                continue;
            }
            let (epoch, shard) = route_from_path(&path)?;
            let writer = service_durability::FramedLogWriter::open(
                &path,
                service_durability::FsyncPolicy::Always,
            )?;
            let events = read_events(&path)?;
            if events.is_empty() {
                continue;
            }
            let segment_id = path
                .file_stem()
                .and_then(|value| value.to_str())
                .context("active segment file has invalid name")?
                .to_string();
            let mut bucket_min = u16::MAX;
            let mut bucket_max = 0;
            for event in &events {
                let bucket = bucket_for(&event.event.event_id);
                bucket_min = bucket_min.min(bucket);
                bucket_max = bucket_max.max(bucket);
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
                    bucket_min,
                    bucket_max,
                },
            );
        }
        Ok(())
    }

    fn index_file(
        &self,
        state: &mut SegmentStateData,
        path: &Path,
        epoch: u64,
        shard: u16,
    ) -> Result<()> {
        for event in read_events(path)? {
            let bucket = bucket_for(&event.event.event_id);
            let segment_id = state
                .sealed
                .values()
                .find(|manifest| manifest.local_path == path)
                .map(|manifest| manifest.segment_id.clone())
                .unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("segment")
                        .to_string()
                });
            self.index_event(
                state,
                &event,
                AppendLocation {
                    route: Route {
                        epoch,
                        shard,
                        bucket,
                    },
                    segment_id,
                    path: path.to_path_buf(),
                },
            )?;
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
        if !state.active.contains_key(&key) {
            let segment_id = format!(
                "segment-e{:020}-s{:04}-c{:020}",
                route.epoch, route.shard, stored.cursor
            );
            let path = self
                .root
                .join(format!("epoch-{:020}", route.epoch))
                .join(format!("shard-{:04}", route.shard))
                .join(format!("{segment_id}.open"));
            let writer = service_durability::FramedLogWriter::open(
                &path,
                service_durability::FsyncPolicy::Always,
            )?;
            state.active.insert(
                key,
                ActiveSegment {
                    route,
                    segment_id,
                    path,
                    writer,
                    first_cursor: stored.cursor,
                    last_cursor: stored.cursor,
                    event_count: 0,
                    bucket_min: route.bucket,
                    bucket_max: route.bucket,
                },
            );
        }
        let encoded = serde_json::to_vec(stored)?;
        let (location, should_seal) = {
            let active = state.active.get_mut(&key).unwrap();
            active.writer.append(stored.cursor, &encoded)?;
            active.last_cursor = stored.cursor;
            active.event_count += 1;
            active.bucket_min = active.bucket_min.min(route.bucket);
            active.bucket_max = active.bucket_max.max(route.bucket);
            (
                AppendLocation {
                    route,
                    segment_id: active.segment_id.clone(),
                    path: active.path.clone(),
                },
                active.event_count as usize >= self.max_segment_events,
            )
        };
        self.index_event(&mut state, stored, location.clone())?;
        if should_seal {
            let manifest = self.seal_locked(&mut state, key)?;
            let sealed_location = AppendLocation {
                route,
                segment_id: manifest.segment_id,
                path: manifest.local_path,
            };
            state.cursors.insert(
                stored.cursor,
                (stored.event.event_id.clone(), sealed_location.clone()),
            );
            return Ok(sealed_location);
        }
        Ok(location)
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
        service_durability::sync_parent_dir(&sealed_path)?;
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
            bytes,
            sha256: sha256_file(&sealed_path)?,
            state: SegmentState::Sealed,
            local_path: sealed_path.clone(),
            object_uri: None,
        };
        self.write_manifest(&manifest)?;
        for (_, location) in state.cursors.values_mut() {
            if location.segment_id == manifest.segment_id {
                location.path = sealed_path.clone();
            }
        }
        state
            .sealed
            .insert(manifest.segment_id.clone(), manifest.clone());
        Ok(manifest)
    }

    fn write_manifest(&self, manifest: &SegmentManifest) -> Result<()> {
        service_durability::atomic_write(
            self.manifests_root
                .join(format!("{}.json", manifest.segment_id)),
            &serde_json::to_vec_pretty(manifest)?,
            service_durability::FsyncPolicy::Always,
        )
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
        fs::rename(&manifest.local_path, &target)?;
        service_durability::sync_parent_dir(&target)?;
        manifest.local_path = target.clone();
        manifest.state = SegmentState::Moved;
        verify_segment(&manifest)?;
        self.write_manifest(&manifest)?;
        for (_, location) in state.cursors.values_mut() {
            if location.segment_id == segment_id {
                location.path = target.clone();
            }
        }
        state
            .sealed
            .insert(segment_id.to_string(), manifest.clone());
        Ok(manifest)
    }
}

fn read_events(path: &Path) -> Result<Vec<StoredEvent>> {
    service_durability::FramedLogReader::read_frames(path, 0)?
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
        bail!("sealed segment {} failed size/hash verification", manifest.segment_id);
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

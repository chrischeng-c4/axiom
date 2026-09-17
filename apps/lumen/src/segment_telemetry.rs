//! Checkpoint measurements use real file identities and completed lock intervals.
//! Byte counts are logical file bytes, not filesystem block/device traffic.

use super::{CollectionCatalog, SegmentKind};
use crate::capture_barrier::CaptureLease;
use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub(super) struct MeasuredCapture<'a> {
    lease: CaptureLease<'a>,
    total: &'a AtomicU64,
    started: Instant,
}
impl<'a> MeasuredCapture<'a> {
    pub(super) fn new(lease: CaptureLease<'a>, total: &'a AtomicU64) -> Self {
        Self {
            lease,
            total,
            started: Instant::now(),
        }
    }
}
impl<'a> std::ops::Deref for MeasuredCapture<'a> {
    type Target = CaptureLease<'a>;
    fn deref(&self) -> &Self::Target {
        &self.lease
    }
}
impl Drop for MeasuredCapture<'_> {
    fn drop(&mut self) {
        let nanos = u64::try_from(self.started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let _ = self
            .total
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |total| {
                Some(total.saturating_add(nanos))
            });
    }
}

#[cfg(unix)]
fn inventory(path: &Path, files: &mut BTreeMap<(u64, u64), u64>) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        bail!("segment metric refuses symlink");
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(path)? {
            inventory(&entry?.path(), files)?;
        }
    } else if metadata.is_file() {
        files.insert((metadata.dev(), metadata.ino()), metadata.len());
    } else {
        bail!("segment metric requires a regular file or directory");
    }
    Ok(())
}
#[cfg(not(unix))]
fn inventory(_path: &Path, _files: &mut BTreeMap<(u64, u64), u64>) -> Result<()> {
    bail!("segment file identity measurements require Unix")
}
fn sum(mut files: impl Iterator<Item = u64>) -> Result<u64> {
    files.try_fold(0u64, |total, bytes| {
        total
            .checked_add(bytes)
            .ok_or_else(|| anyhow::anyhow!("segment byte count overflow"))
    })
}
pub(super) fn new_file_bytes(staging: &Path, previous: Option<&Path>) -> Result<u64> {
    let mut old = BTreeMap::new();
    if let Some(previous) = previous {
        inventory(previous, &mut old)?;
    }
    let mut new = BTreeMap::new();
    inventory(staging, &mut new)?;
    sum(new
        .into_iter()
        .filter(|(id, _)| !old.contains_key(id))
        .map(|(_, bytes)| bytes))
}

/// `_generation.json` is written after the checkpoint payload snapshot.
pub(super) fn manifest_bytes(staging: &Path) -> Result<u64> {
    let metadata = std::fs::symlink_metadata(staging.join(super::GENERATION_MANIFEST_FILE))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("segment generation manifest must be a regular file");
    }
    Ok(metadata.len())
}
pub(super) fn generation_disk_bytes(root: &Path) -> Result<u64> {
    let mut files = BTreeMap::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        if name.to_str().is_some_and(|name| name.starts_with("gen-")) {
            inventory(&entry.path(), &mut files)?;
        }
    }
    sum(files.into_values())
}
pub(super) fn pending_deltas(root: &Path, collections: &[CollectionCatalog]) -> Result<(u64, u64)> {
    let mut bytes = 0u64;
    let mut layers = 0u64;
    for segment in collections
        .iter()
        .flat_map(|collection| &collection.segments)
    {
        if matches!(segment.kind, SegmentKind::Delta) {
            layers += 1;
            for path in std::iter::once(&segment.path)
                .chain(segment.local_rows.iter().map(|rows| &rows.path))
            {
                bytes = bytes
                    .checked_add(std::fs::metadata(root.join(path))?.len())
                    .ok_or_else(|| anyhow::anyhow!("pending delta bytes overflow"))?;
            }
        }
    }
    Ok((bytes, layers))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn linked_generations_count_payload_only_once() {
        let dir = tempfile::tempdir().unwrap();
        let one = dir.path().join("gen-1");
        let two = dir.path().join("gen-2");
        std::fs::create_dir(&one).unwrap();
        std::fs::create_dir(&two).unwrap();
        std::fs::write(one.join("data"), b"12345").unwrap();
        std::fs::hard_link(one.join("data"), two.join("data")).unwrap();
        std::fs::write(two.join("new"), b"67").unwrap();
        assert_eq!(new_file_bytes(&two, Some(&one)).unwrap(), 2);
        assert_eq!(generation_disk_bytes(dir.path()).unwrap(), 7);
    }

    #[test]
    fn premerge_snapshot_keeps_fresh_delta_out_of_merge_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let prior = dir.path().join("gen-1");
        let staging = dir.path().join("staging");
        std::fs::create_dir(&prior).unwrap();
        std::fs::create_dir(&staging).unwrap();
        std::fs::write(prior.join("base"), b"base").unwrap();
        std::fs::hard_link(prior.join("base"), staging.join("base")).unwrap();
        std::fs::write(staging.join("delta"), b"delta!!").unwrap();
        let checkpoint_payload = new_file_bytes(&staging, Some(&prior)).unwrap();
        assert_eq!(checkpoint_payload, 7);
        std::fs::write(staging.join("delta"), b"merged-output").unwrap();
        std::fs::write(staging.join("_generation.json"), b"meta!").unwrap();
        assert_eq!(manifest_bytes(&staging).unwrap(), 5);
        assert_eq!(checkpoint_payload + manifest_bytes(&staging).unwrap(), 12);
        assert_eq!(std::fs::metadata(staging.join("delta")).unwrap().len(), 13);
    }
}

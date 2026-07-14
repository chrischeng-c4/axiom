// HANDWRITE-BEGIN gap="sift-gcs-archive-manifest" tracker="1659" reason="Upload sealed segments and blobs before the archive manifest and restore only hash-verified objects."
use std::{collections::BTreeMap, path::{Path, PathBuf}};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ContentBlobRef;

use super::{blob::BlobStore, shard, EpochMap, RawStorage, SegmentManifest};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchiveBlob {
    pub reference: ContentBlobRef,
    pub object_uri: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchiveManifest {
    pub format_version: u16,
    pub generated_at: String,
    pub epochs: Vec<EpochMap>,
    pub segments: Vec<SegmentManifest>,
    pub blobs: Vec<ArchiveBlob>,
}

#[derive(Clone, Debug)]
pub struct ArchiveReceipt {
    pub manifest_uri: String,
    pub manifest: ArchiveManifest,
}

pub fn archive_gcs(storage: &RawStorage, destination_uri: &str) -> Result<ArchiveReceipt> {
    let destination = service_backup::BackupDestination::from_uri(destination_uri)?;
    let sink = service_backup::GcsSink::from_destination(&destination)?;
    let prefix = destination.default_prefix();
    let archive_id = format!(
        "{}-{}",
        Utc::now().format("%Y%m%dT%H%M%S%.fZ"),
        std::process::id()
    );
    let archive_prefix = format!("{prefix}/archives/{archive_id}");

    let mut segments = storage.seal_all()?;
    for manifest in &mut segments {
        let bytes = std::fs::read(&manifest.local_path)?;
        verify_bytes(&manifest.sha256, manifest.bytes, &bytes, "segment")?;
        let key = format!("{archive_prefix}/segments/{}.framed", manifest.segment_id);
        manifest.object_uri = Some(sink.put_object(&key, &bytes, "application/octet-stream")?);
        manifest.local_path = PathBuf::from(format!("segments/{}.framed", manifest.segment_id));
    }

    let referenced = storage
        .recovered_events()?
        .into_iter()
        .flat_map(|event| event.event.blob_refs)
        .map(|reference| (reference.hash.clone(), reference))
        .collect::<BTreeMap<_, _>>();
    let mut blobs = Vec::new();
    for reference in referenced.into_values() {
        let bytes = storage.read_blob(&reference.hash)?;
        if bytes.len() as u64 != reference.size {
            bail!("blob {} size changed before archive", reference.hash);
        }
        let digest = reference.hash.trim_start_matches("sha256:");
        let key = format!("{archive_prefix}/blobs/{digest}.blob");
        let object_uri = sink.put_object(&key, &bytes, "application/octet-stream")?;
        blobs.push(ArchiveBlob {
            reference,
            object_uri,
        });
    }
    blobs.sort_by(|left, right| left.reference.hash.cmp(&right.reference.hash));

    let manifest = ArchiveManifest {
        format_version: 1,
        generated_at: Utc::now().to_rfc3339(),
        epochs: storage.epoch_maps(),
        segments,
        blobs,
    };
    let manifest_key = format!("{archive_prefix}/manifest.json");
    let manifest_uri = sink.put_object(
        &manifest_key,
        &serde_json::to_vec_pretty(&manifest)?,
        "application/json",
    )?;
    Ok(ArchiveReceipt {
        manifest_uri,
        manifest,
    })
}

pub fn restore_gcs(manifest_uri: &str, target: impl AsRef<Path>) -> Result<ArchiveManifest> {
    let target = target.as_ref();
    std::fs::create_dir_all(target)?;
    let manifest_bytes = service_backup::fetch_backup_object(manifest_uri)?;
    let mut manifest: ArchiveManifest = serde_json::from_slice(&manifest_bytes)
        .context("decode Sift archive manifest")?;
    if manifest.format_version != 1 {
        bail!("unsupported Sift archive manifest version {}", manifest.format_version);
    }
    let manifests_root = target.join("segments").join("manifests");
    std::fs::create_dir_all(&manifests_root)?;
    for segment in &mut manifest.segments {
        let uri = segment
            .object_uri
            .as_deref()
            .context("archive segment lacks object_uri")?;
        let bytes = service_backup::fetch_backup_object(uri)?;
        verify_bytes(&segment.sha256, segment.bytes, &bytes, "segment")?;
        let local_path = target
            .join("segments")
            .join(format!("epoch-{:020}", segment.epoch))
            .join(format!("shard-{:04}", segment.shard))
            .join(format!("{}.framed", segment.segment_id));
        service_durability::atomic_write(
            &local_path,
            &bytes,
            service_durability::FsyncPolicy::Always,
        )?;
        segment.local_path = local_path;
        service_durability::atomic_write(
            manifests_root.join(format!("{}.json", segment.segment_id)),
            &serde_json::to_vec_pretty(segment)?,
            service_durability::FsyncPolicy::Always,
        )?;
    }
    let blob_store = BlobStore::open(target, 65_536)?;
    for blob in &manifest.blobs {
        let bytes = service_backup::fetch_backup_object(&blob.object_uri)?;
        let restored = blob_store.put(&bytes, blob.reference.encoding.clone())?;
        if restored.hash != blob.reference.hash || restored.size != blob.reference.size {
            bail!("restored blob {} failed hash/size verification", blob.reference.hash);
        }
    }
    shard::write_epoch_maps(target, &manifest.epochs)?;
    Ok(manifest)
}

fn verify_bytes(expected_hash: &str, expected_size: u64, bytes: &[u8], kind: &str) -> Result<()> {
    let actual = hex::encode(Sha256::digest(bytes));
    if bytes.len() as u64 != expected_size || actual != expected_hash {
        bail!("{kind} archive object failed hash/size verification");
    }
    Ok(())
}
// HANDWRITE-END

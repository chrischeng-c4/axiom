//! Sift policy for the shared private, versioned data-root mechanism.

use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage_durable::{DataRoot, DataRootPolicy};

pub const DEFAULT_DATA_DIR: &str = "/var/lib/sift";
const FORMAT_VERSION: u32 = 1;
const LEGACY_MARKERS: &[&str] = &[
    "raw-events.framed",
    "raw-events.snapshot.json",
    "raw",
    "blobs",
    "epochs.json",
];
const DIRECTORIES: &[&str] = &[
    "control",
    "wal/logs",
    "wal/metrics",
    "wal/traces",
    "segments/logs",
    "segments/metrics",
    "segments/traces",
    "indexes",
    "snapshots",
    "archive-cache",
    "gateway-spool",
    "query-jobs",
    "agent",
    "tmp",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRole {
    All,
    Agent,
    Gateway,
    Query,
    Store,
    Control,
    Operator,
}

impl StorageRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Agent => "agent",
            Self::Gateway => "gateway",
            Self::Query => "query",
            Self::Store => "store",
            Self::Control => "control",
            Self::Operator => "operator",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutManifest {
    pub format_version: u32,
    pub cluster_id: String,
    pub node_id: String,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restored_from: Option<String>,
}

#[derive(Clone, Copy)]
struct SiftDataRootPolicy {
    role: StorageRole,
}

impl DataRootPolicy for SiftDataRootPolicy {
    type Manifest = LayoutManifest;

    fn product_name(&self) -> &'static str {
        "Sift"
    }

    fn directories(&self) -> &'static [&'static str] {
        DIRECTORIES
    }

    fn legacy_markers(&self) -> &'static [&'static str] {
        LEGACY_MARKERS
    }

    fn create_manifest(&self, root: &Path) -> Result<Self::Manifest> {
        let canonical = root
            .canonicalize()
            .with_context(|| format!("canonicalize Sift data root {}", root.display()))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before Unix epoch")?
            .as_nanos();
        let seed = format!("{}:{}:{}", canonical.display(), std::process::id(), now);
        let digest = hex::encode(Sha256::digest(seed.as_bytes()));
        Ok(LayoutManifest {
            format_version: FORMAT_VERSION,
            cluster_id: format!("cluster-{}", &digest[..16]),
            node_id: format!("node-{}", &digest[16..32]),
            role: self.role.as_str().to_string(),
            restored_from: None,
        })
    }

    fn validate_manifest(&self, manifest: &Self::Manifest) -> Result<()> {
        if manifest.format_version != FORMAT_VERSION {
            bail!(
                "unsupported Sift data format {}; expected {}",
                manifest.format_version,
                FORMAT_VERSION
            );
        }
        if manifest.cluster_id.is_empty() || manifest.node_id.is_empty() {
            bail!("Sift layout cluster_id and node_id must not be empty");
        }
        if manifest.role != self.role.as_str() && manifest.role != StorageRole::All.as_str() {
            bail!(
                "Sift data root belongs to role {}, not {}",
                manifest.role,
                self.role.as_str()
            );
        }
        Ok(())
    }

    fn legacy_error(&self, marker: &Path) -> anyhow::Error {
        anyhow::anyhow!(
            "legacy Sift 0.1.1 data at {} is not compatible; preserve it and choose an empty data directory",
            marker.display()
        )
    }
}

/// Sift's thin domain adapter around [`storage_durable::DataRoot`].
pub struct DataLayout {
    inner: DataRoot<SiftDataRootPolicy>,
}

impl DataLayout {
    pub fn open(root: impl AsRef<Path>, role: StorageRole) -> Result<Self> {
        Ok(Self {
            inner: DataRoot::open(root, SiftDataRootPolicy { role })?,
        })
    }

    pub fn root(&self) -> &Path {
        self.inner.root()
    }

    pub fn manifest(&self) -> &LayoutManifest {
        self.inner.manifest()
    }

    pub fn mark_restored_from(&mut self, manifest_uri: &str) -> Result<()> {
        if manifest_uri.trim().is_empty() {
            bail!("restore manifest URI must not be empty");
        }
        let mut manifest = self.inner.manifest().clone();
        manifest.restored_from = Some(manifest_uri.to_string());
        self.inner.replace_manifest(manifest)
    }

    pub fn into_root_path(self) -> PathBuf {
        self.inner.root().to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::{DataLayout, StorageRole};

    #[test]
    fn one_process_owns_one_root() {
        let temp = tempfile::tempdir().unwrap();
        let first = DataLayout::open(temp.path(), StorageRole::All).unwrap();
        let error = DataLayout::open(temp.path(), StorageRole::All)
            .err()
            .expect("a second owner must be refused");
        assert!(error.to_string().contains("another Sift process"));
        drop(first);
        DataLayout::open(temp.path(), StorageRole::All).unwrap();
    }
}

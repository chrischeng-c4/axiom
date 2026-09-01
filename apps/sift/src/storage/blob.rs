// HANDWRITE-BEGIN gap="sift-content-addressed-blob-store" tracker="1659" reason="Atomically fsync SHA-256-addressed blobs and externalize large base64 payload fields before raw append."
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{ContentBlobRef, OperationalEventV2};

#[derive(Debug, Clone)]
pub struct BlobStore {
    root: PathBuf,
    externalize_bytes: usize,
}

impl BlobStore {
    pub fn open(root: impl AsRef<Path>, externalize_bytes: usize) -> Result<Self> {
        let root = root
            .as_ref()
            .join("archive-cache")
            .join("blobs")
            .join("sha256");
        fs::create_dir_all(&root)
            .with_context(|| format!("create blob store {}", root.display()))?;
        Ok(Self {
            root,
            externalize_bytes,
        })
    }

    pub fn put(&self, bytes: &[u8], encoding: impl Into<String>) -> Result<ContentBlobRef> {
        let digest = hex::encode(Sha256::digest(bytes));
        let hash = format!("sha256:{digest}");
        let path = self.path_for_hash(&hash)?;
        if path.exists() {
            let existing = fs::read(&path)
                .with_context(|| format!("read existing blob {}", path.display()))?;
            if existing != bytes {
                bail!("content-addressed blob collision for {hash}");
            }
        } else {
            storage_durable::atomic_write(&path, bytes, storage_durable::FsyncPolicy::Always)
                .with_context(|| format!("durably write blob {}", path.display()))?;
        }
        Ok(ContentBlobRef {
            hash,
            size: bytes.len() as u64,
            encoding: encoding.into(),
        })
    }

    pub fn read(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.path_for_hash(hash)?;
        let bytes = fs::read(&path).with_context(|| format!("read blob {}", path.display()))?;
        let actual = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));
        if actual != hash {
            bail!("blob hash mismatch: expected {hash}, got {actual}");
        }
        Ok(bytes)
    }

    pub fn externalize_event(&self, event: &mut OperationalEventV2) -> Result<()> {
        let mut refs = Vec::new();
        self.externalize_value(&mut event.payload, &mut refs)?;
        let mut seen = event
            .blob_refs
            .iter()
            .map(|reference| reference.hash.clone())
            .collect::<BTreeSet<_>>();
        for reference in refs {
            if seen.insert(reference.hash.clone()) {
                event.blob_refs.push(reference);
            }
        }
        Ok(())
    }

    pub fn validate_references(&self, references: &[ContentBlobRef]) -> Result<()> {
        for reference in references {
            let bytes = self
                .read(&reference.hash)
                .with_context(|| format!("read durable content reference {}", reference.hash))?;
            if bytes.len() as u64 != reference.size {
                bail!(
                    "blob {} size mismatch: reference {}, durable {}",
                    reference.hash,
                    reference.size,
                    bytes.len()
                );
            }
        }
        Ok(())
    }

    fn externalize_value(&self, value: &mut Value, refs: &mut Vec<ContentBlobRef>) -> Result<()> {
        match value {
            Value::Array(values) => {
                for value in values {
                    self.externalize_value(value, refs)?;
                }
            }
            Value::Object(object) => {
                let keys = object.keys().cloned().collect::<Vec<_>>();
                for key in keys {
                    let Some(child) = object.get_mut(&key) else {
                        continue;
                    };
                    let is_base64 = key.to_ascii_lowercase().ends_with("base64");
                    if is_base64 {
                        if let Some(encoded) = child.as_str() {
                            if let Ok(bytes) = BASE64.decode(encoded) {
                                if bytes.len() >= self.externalize_bytes {
                                    let reference = self.put(&bytes, "base64")?;
                                    *child = json!({"blob": reference});
                                    refs.push(reference);
                                    continue;
                                }
                            }
                        }
                    }
                    self.externalize_value(child, refs)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn path_for_hash(&self, hash: &str) -> Result<PathBuf> {
        let digest = hash
            .strip_prefix("sha256:")
            .context("blob hash must use sha256:<hex>")?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("blob hash must contain 64 hexadecimal characters");
        }
        Ok(self.root.join(&digest[..2]).join(format!("{digest}.blob")))
    }

    pub fn blob_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        if !self.root.exists() {
            return Ok(paths);
        }
        for prefix in fs::read_dir(&self.root)? {
            let prefix = prefix?;
            if !prefix.file_type()?.is_dir() {
                continue;
            }
            for entry in fs::read_dir(prefix.path())? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    paths.push(entry.path());
                }
            }
        }
        paths.sort();
        Ok(paths)
    }

    /// Delete local blobs that are no longer referenced by the committed
    /// retained event set. The archive manifest is the commit point. Callers
    /// must not use this before that manifest is durable.
    pub fn prune_except(&self, retained_hashes: &BTreeSet<String>) -> Result<usize> {
        let mut removed = 0_usize;
        for path in self.blob_paths()? {
            let digest = path
                .file_stem()
                .and_then(|value| value.to_str())
                .context("blob file name is not valid UTF-8")?;
            let hash = format!("sha256:{digest}");
            if self.path_for_hash(&hash)? != path {
                bail!(
                    "blob path does not match its content address: {}",
                    path.display()
                );
            }
            if retained_hashes.contains(&hash) {
                continue;
            }
            match fs::remove_file(&path) {
                Ok(()) => {
                    storage_durable::sync_parent_dir(&path)?;
                    removed += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("remove expired blob {}", path.display()));
                }
            }
        }
        Ok(removed)
    }
}
// HANDWRITE-END

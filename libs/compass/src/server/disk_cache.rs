//! Persistent AST index cache backed by bincode.
//!
//! Each source file gets a `{path_hash}.idx` file in the cache directory.
//! A `manifest.bin` tracks path→hash mappings for fast staleness checks.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing;

use crate::diagnostic::Diagnostic;
use crate::type_inference::SemanticModel;

/// Current schema version. Bump when PersistedEntry layout changes.
const SCHEMA_VERSION: u32 = 1;

/// Data persisted to disk for a single source file.
#[derive(Serialize, Deserialize)]
pub struct PersistedEntry {
    version: u32,
    pub content_hash: u64,
    pub semantic_model: SemanticModel,
    pub diagnostics: Vec<Diagnostic>,
}

/// Manifest mapping canonical paths to their cache files.
#[derive(Serialize, Deserialize, Default)]
struct CacheManifest {
    version: u32,
    entries: HashMap<String, ManifestEntry>,
}

/// Single entry within the manifest.
#[derive(Serialize, Deserialize)]
struct ManifestEntry {
    cache_file: String,
    content_hash: u64,
}

/// Disk-backed index cache.
pub struct DiskCache {
    cache_dir: PathBuf,
    manifest: RwLock<CacheManifest>,
}

impl DiskCache {
    /// Open (or create) the cache directory and load the manifest.
    pub fn new(cache_dir: PathBuf) -> Self {
        fs::create_dir_all(&cache_dir).ok();
        let manifest = Self::load_manifest(&cache_dir);
        Self {
            cache_dir,
            manifest: RwLock::new(manifest),
        }
    }

    /// Check whether the cached entry for `path` is still fresh.
    pub async fn is_fresh(&self, path: &Path, current_hash: u64) -> bool {
        let manifest = self.manifest.read().await;
        let key = path_key(path);
        match manifest.entries.get(&key) {
            Some(entry) => entry.content_hash == current_hash,
            None => false,
        }
    }

    /// Try to load a persisted entry from disk.
    ///
    /// Returns `None` when:
    /// - no cache file exists
    /// - content hash doesn't match (stale)
    /// - schema version mismatch
    /// - deserialization fails
    pub async fn load(&self, path: &Path, current_hash: u64) -> Option<PersistedEntry> {
        let manifest = self.manifest.read().await;
        let key = path_key(path);
        let entry = manifest.entries.get(&key)?;

        if entry.content_hash != current_hash {
            return None;
        }

        let idx_path = self.cache_dir.join(&entry.cache_file);
        let bytes = fs::read(&idx_path).ok()?;
        let persisted: PersistedEntry = bincode::deserialize(&bytes).ok()?;

        if persisted.version != SCHEMA_VERSION {
            return None;
        }
        if persisted.content_hash != current_hash {
            return None;
        }

        tracing::debug!("disk cache hit: {}", path.display());
        Some(persisted)
    }

    /// Persist an entry to disk (best-effort, errors are logged).
    pub async fn store(
        &self,
        path: &Path,
        content_hash: u64,
        semantic_model: &SemanticModel,
        diagnostics: &[Diagnostic],
    ) {
        let entry = PersistedEntry {
            version: SCHEMA_VERSION,
            content_hash,
            semantic_model: semantic_model.clone(),
            diagnostics: diagnostics.to_vec(),
        };

        let bytes = match bincode::serialize(&entry) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("disk cache serialize error: {e}");
                return;
            }
        };

        let key = path_key(path);
        let cache_file = format!("{}.idx", hex_hash(path));
        let idx_path = self.cache_dir.join(&cache_file);

        if let Err(e) = fs::write(&idx_path, &bytes) {
            tracing::warn!("disk cache write error: {e}");
            return;
        }

        let mut manifest = self.manifest.write().await;
        manifest.entries.insert(
            key,
            ManifestEntry {
                cache_file,
                content_hash,
            },
        );
    }

    /// Remove the disk cache entry for a file.
    pub async fn invalidate(&self, path: &Path) {
        let key = path_key(path);
        let mut manifest = self.manifest.write().await;
        if let Some(entry) = manifest.entries.remove(&key) {
            let idx_path = self.cache_dir.join(&entry.cache_file);
            fs::remove_file(&idx_path).ok();
        }
    }

    /// Flush the manifest to disk. Call on shutdown.
    pub async fn flush_manifest(&self) {
        let manifest = self.manifest.read().await;
        let manifest_path = self.cache_dir.join("manifest.bin");
        match bincode::serialize(&*manifest) {
            Ok(bytes) => {
                if let Err(e) = fs::write(&manifest_path, bytes) {
                    tracing::warn!("manifest write error: {e}");
                }
            }
            Err(e) => {
                tracing::warn!("manifest serialize error: {e}");
            }
        }
    }

    /// Remove cache files that are no longer referenced by the manifest.
    pub async fn cleanup_stale(&self) {
        let manifest = self.manifest.read().await;
        let valid_files: std::collections::HashSet<&str> = manifest
            .entries
            .values()
            .map(|e| e.cache_file.as_str())
            .collect();

        let entries = match fs::read_dir(&self.cache_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".idx") && !valid_files.contains(name_str.as_ref()) {
                fs::remove_file(entry.path()).ok();
            }
        }
    }

    // -- private helpers --

    fn load_manifest(cache_dir: &Path) -> CacheManifest {
        let manifest_path = cache_dir.join("manifest.bin");
        let bytes = match fs::read(&manifest_path) {
            Ok(b) => b,
            Err(_) => return CacheManifest::default(),
        };
        bincode::deserialize(&bytes).unwrap_or_default()
    }
}

/// Canonical string key for a path (lossy but deterministic).
fn path_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Stable hex hash of a path for use as a cache filename.
fn hex_hash(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    path.hash(&mut h);
    format!("{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_model() -> SemanticModel {
        SemanticModel::default()
    }

    #[tokio::test]
    async fn test_store_and_load() {
        let tmp = TempDir::new().unwrap();
        let cache = DiskCache::new(tmp.path().join("cache"));

        let path = PathBuf::from("/fake/src/main.py");
        let hash: u64 = 12345;

        cache.store(&path, hash, &make_model(), &[]).await;
        cache.flush_manifest().await;

        let loaded = cache.load(&path, hash).await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().content_hash, hash);
    }

    #[tokio::test]
    async fn test_stale_hash_returns_none() {
        let tmp = TempDir::new().unwrap();
        let cache = DiskCache::new(tmp.path().join("cache"));

        let path = PathBuf::from("/fake/src/main.py");
        cache.store(&path, 111, &make_model(), &[]).await;
        cache.flush_manifest().await;

        // Different hash → stale
        let loaded = cache.load(&path, 999).await;
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_invalidate_removes_file() {
        let tmp = TempDir::new().unwrap();
        let cache = DiskCache::new(tmp.path().join("cache"));

        let path = PathBuf::from("/fake/src/lib.py");
        cache.store(&path, 42, &make_model(), &[]).await;
        cache.flush_manifest().await;

        cache.invalidate(&path).await;
        let loaded = cache.load(&path, 42).await;
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_is_fresh() {
        let tmp = TempDir::new().unwrap();
        let cache = DiskCache::new(tmp.path().join("cache"));

        let path = PathBuf::from("/fake/src/app.py");
        cache.store(&path, 100, &make_model(), &[]).await;

        assert!(cache.is_fresh(&path, 100).await);
        assert!(!cache.is_fresh(&path, 200).await);
    }

    #[tokio::test]
    async fn test_cleanup_stale() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();

        // Create an orphan .idx file
        fs::write(cache_dir.join("orphan.idx"), b"garbage").unwrap();

        let cache = DiskCache::new(cache_dir.clone());
        cache.cleanup_stale().await;

        assert!(!cache_dir.join("orphan.idx").exists());
    }

    #[tokio::test]
    async fn test_manifest_persistence_across_instances() {
        let tmp = TempDir::new().unwrap();
        let cache_dir = tmp.path().join("cache");

        // First instance: store and flush
        {
            let cache = DiskCache::new(cache_dir.clone());
            let path = PathBuf::from("/fake/persist.py");
            cache.store(&path, 777, &make_model(), &[]).await;
            cache.flush_manifest().await;
        }

        // Second instance: should load from manifest
        {
            let cache = DiskCache::new(cache_dir.clone());
            let path = PathBuf::from("/fake/persist.py");
            let loaded = cache.load(&path, 777).await;
            assert!(loaded.is_some());
        }
    }
}

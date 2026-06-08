//! Checkpoint trait + Memory + File impls (#2040).
//!
//! A `Checkpoint<State>` is a key/value store of graph states. Callers
//! save under a stable key (e.g. `"thread-42:after-tool-call"`), then
//! `load` it later to resume — the basis for human-in-the-loop pause /
//! resume (#2041), parallel branches (#2042), and the visualizer's
//! state-inspector (#2053).
//!
//! Ships with two impls in this slice; a SQL impl is deferred so it can
//! pull in a proper migration story alongside #2050 (VectorStore).

// HANDWRITE-BEGIN reason: no generator for a typed key/value persistence
// trait that lifts `State: Serialize + DeserializeOwned` through async
// methods. Tracked under the same Epic-3 gap as the rest of the graph
// engine.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use agent::{NovaError, NovaResult};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

/// Persistence surface for graph states. Implementations are typed to a
/// single `State` shape so the trait can be used as a trait object without
/// runtime cast gymnastics.
#[async_trait]
pub trait Checkpoint<State>: Send + Sync
where
    State: Send + Sync + 'static,
{
    /// Store `state` under `key`. Overwrites silently.
    async fn save(&self, key: &str, state: &State) -> NovaResult<()>;

    /// Read back the state saved at `key`, or `None` if no such key.
    async fn load(&self, key: &str) -> NovaResult<Option<State>>;

    /// All keys currently present, in unspecified order.
    async fn list(&self) -> NovaResult<Vec<String>>;

    /// Remove `key`. Idempotent — returns Ok if `key` was not present.
    async fn delete(&self, key: &str) -> NovaResult<()>;
}

// ── MemoryCheckpoint ──────────────────────────────────────────────────────

/// In-process checkpoint backed by an `RwLock<HashMap>`. Useful for tests
/// and short-lived sessions where durability is not required. `State` is
/// stored by `Clone` so callers can keep reading without holding the lock.
pub struct MemoryCheckpoint<State> {
    inner: Arc<RwLock<HashMap<String, State>>>,
}

impl<State> Default for MemoryCheckpoint<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> MemoryCheckpoint<State> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<State> Clone for MemoryCheckpoint<State> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
impl<State> Checkpoint<State> for MemoryCheckpoint<State>
where
    State: Clone + Send + Sync + 'static,
{
    async fn save(&self, key: &str, state: &State) -> NovaResult<()> {
        self.inner.write().await.insert(key.to_string(), state.clone());
        Ok(())
    }

    async fn load(&self, key: &str) -> NovaResult<Option<State>> {
        Ok(self.inner.read().await.get(key).cloned())
    }

    async fn list(&self) -> NovaResult<Vec<String>> {
        Ok(self.inner.read().await.keys().cloned().collect())
    }

    async fn delete(&self, key: &str) -> NovaResult<()> {
        self.inner.write().await.remove(key);
        Ok(())
    }
}

// ── FileCheckpoint ────────────────────────────────────────────────────────

/// Disk-backed checkpoint. Each key maps to one JSON file at
/// `<root>/<sanitized-key>.json`. `State` must round-trip through
/// `serde_json`. The root directory is created on first use.
pub struct FileCheckpoint<State> {
    root: PathBuf,
    _state: PhantomData<fn() -> State>,
}

impl<State> FileCheckpoint<State> {
    /// Create a new file-backed checkpoint rooted at `root`. The directory
    /// is created if it does not yet exist.
    pub async fn new(root: impl AsRef<Path>) -> NovaResult<Self> {
        let root = root.as_ref().to_path_buf();
        tokio::fs::create_dir_all(&root).await.map_err(NovaError::IoError)?;
        Ok(Self {
            root,
            _state: PhantomData,
        })
    }

    fn path_for(&self, key: &str) -> PathBuf {
        // Keep filenames safe: replace `/` and any other directory separator
        // with `__` so a key like `"thread/42"` cannot escape the root.
        let safe = key
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' => '_',
                c => c,
            })
            .collect::<String>();
        self.root.join(format!("{safe}.json"))
    }
}

#[async_trait]
impl<State> Checkpoint<State> for FileCheckpoint<State>
where
    State: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn save(&self, key: &str, state: &State) -> NovaResult<()> {
        let bytes = serde_json::to_vec_pretty(state).map_err(NovaError::SerializationError)?;
        let path = self.path_for(key);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(NovaError::IoError)?;
        Ok(())
    }

    async fn load(&self, key: &str) -> NovaResult<Option<State>> {
        let path = self.path_for(key);
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let v: State =
                    serde_json::from_slice(&bytes).map_err(NovaError::SerializationError)?;
                Ok(Some(v))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(NovaError::IoError(e)),
        }
    }

    async fn list(&self) -> NovaResult<Vec<String>> {
        let mut out = Vec::new();
        let mut entries = tokio::fs::read_dir(&self.root)
            .await
            .map_err(NovaError::IoError)?;
        while let Some(entry) = entries.next_entry().await.map_err(NovaError::IoError)? {
            if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                if entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "json")
                    .unwrap_or(false)
                {
                    out.push(stem.to_string());
                }
            }
        }
        Ok(out)
    }

    async fn delete(&self, key: &str) -> NovaResult<()> {
        let path = self.path_for(key);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(NovaError::IoError(e)),
        }
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct DemoState {
        n: i32,
        label: String,
    }

    #[tokio::test]
    async fn memory_save_load_roundtrip() {
        let cp: MemoryCheckpoint<DemoState> = MemoryCheckpoint::new();
        let s = DemoState {
            n: 7,
            label: "x".into(),
        };
        cp.save("k1", &s).await.unwrap();
        assert_eq!(cp.load("k1").await.unwrap(), Some(s));
    }

    #[tokio::test]
    async fn memory_load_missing_returns_none() {
        let cp: MemoryCheckpoint<DemoState> = MemoryCheckpoint::new();
        assert_eq!(cp.load("missing").await.unwrap(), None);
    }

    #[tokio::test]
    async fn memory_list_and_delete() {
        let cp: MemoryCheckpoint<DemoState> = MemoryCheckpoint::new();
        for n in 0..3 {
            cp.save(
                &format!("k{n}"),
                &DemoState {
                    n,
                    label: "".into(),
                },
            )
            .await
            .unwrap();
        }
        let mut keys = cp.list().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["k0", "k1", "k2"]);

        cp.delete("k1").await.unwrap();
        cp.delete("k1").await.unwrap(); // idempotent
        let mut keys = cp.list().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["k0", "k2"]);
    }

    #[tokio::test]
    async fn file_save_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let cp: FileCheckpoint<DemoState> = FileCheckpoint::new(tmp.path()).await.unwrap();
        let s = DemoState {
            n: 99,
            label: "saved".into(),
        };
        cp.save("session-1", &s).await.unwrap();

        // Round-trip through a fresh handle pointed at the same dir.
        let cp2: FileCheckpoint<DemoState> = FileCheckpoint::new(tmp.path()).await.unwrap();
        assert_eq!(cp2.load("session-1").await.unwrap(), Some(s));
    }

    #[tokio::test]
    async fn file_list_and_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let cp: FileCheckpoint<DemoState> = FileCheckpoint::new(tmp.path()).await.unwrap();
        for n in 0..3 {
            cp.save(
                &format!("k{n}"),
                &DemoState {
                    n,
                    label: "".into(),
                },
            )
            .await
            .unwrap();
        }
        let mut keys = cp.list().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["k0", "k1", "k2"]);

        cp.delete("k1").await.unwrap();
        cp.delete("k1").await.unwrap(); // idempotent
        let mut keys = cp.list().await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["k0", "k2"]);
    }

    #[tokio::test]
    async fn file_sanitizes_dangerous_keys() {
        // A key containing `/` must not escape the root.
        let tmp = tempfile::tempdir().unwrap();
        let cp: FileCheckpoint<DemoState> = FileCheckpoint::new(tmp.path()).await.unwrap();
        let s = DemoState {
            n: 1,
            label: "".into(),
        };
        cp.save("../escape", &s).await.unwrap();

        // Everything stays inside the root.
        let entries: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
        assert!(cp.load("../escape").await.unwrap().is_some());
    }
}

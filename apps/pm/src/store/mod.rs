pub mod gate;
pub mod scheduler;
pub mod state;

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use anyhow::{Context, Result};
use fs2::FileExt;
use tokio::sync::RwLock;

pub use gate::{execute_gate, verify_and_merge_task};
pub use scheduler::{get_next_actionable_task, get_task_context};
pub use state::PmState;

#[derive(Clone)]
pub struct PmStore {
    state: Arc<RwLock<PmState>>,
    path: PathBuf,
    last_mtime: Arc<RwLock<Option<SystemTime>>>,
}

impl PmStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let state = PmState::load_or_default(&path)?;
        let mtime = std::fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            path,
            last_mtime: Arc::new(RwLock::new(mtime)),
        })
    }

    pub fn in_memory() -> Self {
        Self {
            state: Arc::new(RwLock::new(PmState::default())),
            path: PathBuf::from(":memory:"),
            last_mtime: Arc::new(RwLock::new(None)),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn reload_if_needed(&self) -> Result<()> {
        if self.path == Path::new(":memory:") || !self.path.exists() {
            return Ok(());
        }
        let disk_mtime = std::fs::metadata(&self.path).ok().and_then(|m| m.modified().ok());
        let current_mtime = *self.last_mtime.read().await;
        if disk_mtime != current_mtime {
            let reloaded = PmState::load_or_default(&self.path)?;
            let mut state_guard = self.state.write().await;
            *state_guard = reloaded;
            *self.last_mtime.write().await = disk_mtime;
        }
        Ok(())
    }

    pub async fn read<R, F: FnOnce(&PmState) -> R>(&self, f: F) -> R {
        let _ = self.reload_if_needed().await;
        let lock = self.state.read().await;
        f(&lock)
    }

    pub async fn write<R, F: FnOnce(&mut PmState) -> Result<R>>(&self, f: F) -> Result<R> {
        if self.path == Path::new(":memory:") {
            let mut lock = self.state.write().await;
            let res = f(&mut lock)?;
            return Ok(res);
        }

        // Advisory file lock on <state>.lock
        let lock_path = self.path.with_extension("lock");
        if let Some(parent) = lock_path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .with_context(|| format!("Failed to open lock file at {}", lock_path.display()))?;

        lock_file
            .lock_exclusive()
            .with_context(|| format!("Failed to acquire exclusive lock on {}", lock_path.display()))?;

        struct FileLockGuard(std::fs::File);
        impl Drop for FileLockGuard {
            fn drop(&mut self) {
                let _ = self.0.unlock();
            }
        }
        let _guard = FileLockGuard(lock_file);

        let mut state_guard = self.state.write().await;
        if self.path.exists() {
            *state_guard = PmState::load_or_default(&self.path)?;
        }
        let res = f(&mut state_guard)?;
        state_guard.save_atomic(&self.path)?;
        let mtime = std::fs::metadata(&self.path).ok().and_then(|m| m.modified().ok());
        *self.last_mtime.write().await = mtime;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;

    #[tokio::test]
    async fn test_store_crud_and_atomic_persistence() {
        let temp_dir = std::env::temp_dir().join(format!("pm_mod_test_{}", std::process::id()));
        let state_file = temp_dir.join("state.json");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let store = PmStore::new(&state_file).expect("Failed to initialize store");

        let proj = Project {
            id: "proj_test".to_string(),
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            root_path: ".".to_string(),
            default_gate_cmd: None,
            created_at: "2026-09-06T00:00:00Z".to_string(),
            updated_at: "2026-09-06T00:00:00Z".to_string(),
        };
        store
            .write(|s| {
                s.upsert_project(proj);
                Ok(())
            })
            .await
            .unwrap();

        assert!(state_file.exists());

        // Second store instance pointing to same file
        let store2 = PmStore::new(&state_file).expect("Failed to reload store");
        let loaded_proj = store2.read(|s| s.get_project("proj_test").cloned()).await;
        assert!(loaded_proj.is_some());
        assert_eq!(loaded_proj.unwrap().name, "Test Project");

        // Store 1 writes a second project
        store
            .write(|s| {
                s.upsert_project(Project {
                    id: "proj_2".to_string(),
                    name: "Project 2".to_string(),
                    description: "P2".to_string(),
                    root_path: ".".to_string(),
                    default_gate_cmd: None,
                    created_at: "2026-09-06T00:00:00Z".to_string(),
                    updated_at: "2026-09-06T00:00:00Z".to_string(),
                });
                Ok(())
            })
            .await
            .unwrap();

        // Store 2 reads without restart - mtime invalidation automatically picks it up!
        let loaded_proj2 = store2.read(|s| s.get_project("proj_2").cloned()).await;
        assert!(loaded_proj2.is_some());
        assert_eq!(loaded_proj2.unwrap().name, "Project 2");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = PmStore::in_memory();
        store
            .write(|s| {
                s.upsert_project(Project {
                    id: "proj_mem".to_string(),
                    name: "Memory".to_string(),
                    description: "Mem".to_string(),
                    root_path: ".".to_string(),
                    default_gate_cmd: None,
                    created_at: "2026-09-06T00:00:00Z".to_string(),
                    updated_at: "2026-09-06T00:00:00Z".to_string(),
                });
                Ok(())
            })
            .await
            .unwrap();

        let p = store.read(|s| s.get_project("proj_mem").cloned()).await;
        assert!(p.is_some());
    }
}


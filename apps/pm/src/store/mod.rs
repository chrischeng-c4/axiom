pub mod scheduler;
pub mod state;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;

pub use scheduler::{get_next_actionable_task, get_task_context};
pub use state::PmState;

#[derive(Clone)]
pub struct PmStore {
    state: Arc<RwLock<PmState>>,
    path: PathBuf,
}

impl PmStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let state = PmState::load_or_default(&path)?;
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            path,
        })
    }

    pub fn in_memory() -> Self {
        Self {
            state: Arc::new(RwLock::new(PmState::default())),
            path: PathBuf::from(":memory:"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn read<R, F: FnOnce(&PmState) -> R>(&self, f: F) -> R {
        let lock = self.state.read().await;
        f(&lock)
    }

    pub async fn write<R, F: FnOnce(&mut PmState) -> Result<R>>(&self, f: F) -> Result<R> {
        let mut lock = self.state.write().await;
        let res = f(&mut lock)?;
        if self.path != Path::new(":memory:") {
            lock.save_atomic(&self.path)?;
        }
        Ok(res)
    }
}

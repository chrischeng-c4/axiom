//! Local filesystem capacity admission with reversible reservations.

use std::{
    fmt,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityLevel {
    Normal,
    Warning,
    Backpressure,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityThresholds {
    pub warning_percent: u8,
    pub backpressure_percent: u8,
    pub critical_percent: u8,
}

impl CapacityThresholds {
    pub fn new(warning: u8, backpressure: u8, critical: u8) -> Result<Self> {
        anyhow::ensure!(
            warning < backpressure && backpressure < critical && critical <= 100,
            "capacity thresholds must satisfy warning < backpressure < critical <= 100"
        );
        Ok(Self {
            warning_percent: warning,
            backpressure_percent: backpressure,
            critical_percent: critical,
        })
    }
}

impl Default for CapacityThresholds {
    fn default() -> Self {
        Self {
            warning_percent: 70,
            backpressure_percent: 80,
            critical_percent: 90,
        }
    }
}

pub trait SpaceProbe: Send + Sync + 'static {
    fn available_space(&self, root: &Path) -> std::io::Result<u64>;
}

#[derive(Debug)]
pub struct FileSystemSpaceProbe;

impl SpaceProbe for FileSystemSpaceProbe {
    fn available_space(&self, root: &Path) -> std::io::Result<u64> {
        fs2::available_space(root)
    }
}

#[derive(Debug, Default)]
struct CapacityState {
    committed_bytes: u64,
    reserved_bytes: u64,
}

pub struct CapacityGuard {
    root: PathBuf,
    max_bytes: u64,
    min_free_bytes: u64,
    thresholds: CapacityThresholds,
    state: Mutex<CapacityState>,
    space_probe: Arc<dyn SpaceProbe>,
}

impl fmt::Debug for CapacityGuard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CapacityGuard")
            .field("root", &self.root)
            .field("max_bytes", &self.max_bytes)
            .field("min_free_bytes", &self.min_free_bytes)
            .field("thresholds", &self.thresholds)
            .field("used_bytes", &self.used_bytes())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(
    "local storage is at its safety threshold: used/reserved {used_bytes} bytes of {max_bytes}, free {free_bytes} bytes with {min_free_bytes} required"
)]
pub struct CapacityError {
    pub used_bytes: u64,
    pub max_bytes: u64,
    pub free_bytes: u64,
    pub min_free_bytes: u64,
}

pub struct CapacityReservation<'a> {
    guard: &'a CapacityGuard,
    bytes: u64,
    committed: bool,
}

impl CapacityGuard {
    pub fn open(root: impl AsRef<Path>, max_bytes: u64, min_free_bytes: u64) -> Result<Self> {
        Self::open_with_probe(
            root,
            max_bytes,
            min_free_bytes,
            CapacityThresholds::default(),
            Arc::new(FileSystemSpaceProbe),
        )
    }

    pub fn open_with_probe(
        root: impl AsRef<Path>,
        max_bytes: u64,
        min_free_bytes: u64,
        thresholds: CapacityThresholds,
        space_probe: Arc<dyn SpaceProbe>,
    ) -> Result<Self> {
        anyhow::ensure!(max_bytes > 0, "maximum local storage bytes must be positive");
        let root = root.as_ref().to_path_buf();
        let committed_bytes = directory_bytes(&root)?;
        Ok(Self {
            root,
            max_bytes,
            min_free_bytes,
            thresholds,
            state: Mutex::new(CapacityState {
                committed_bytes,
                reserved_bytes: 0,
            }),
            space_probe,
        })
    }

    pub fn preflight(&self, additional_bytes: u64) -> Result<(), CapacityError> {
        let state = self.state.lock().expect("capacity state lock poisoned");
        self.check_locked(&state, additional_bytes)
    }

    pub fn reserve(&self, additional_bytes: u64) -> Result<CapacityReservation<'_>, CapacityError> {
        let mut state = self.state.lock().expect("capacity state lock poisoned");
        self.check_locked(&state, additional_bytes)?;
        state.reserved_bytes = state.reserved_bytes.saturating_add(additional_bytes);
        Ok(CapacityReservation {
            guard: self,
            bytes: additional_bytes,
            committed: false,
        })
    }

    fn check_locked(
        &self,
        state: &CapacityState,
        additional_bytes: u64,
    ) -> Result<(), CapacityError> {
        let used = state.committed_bytes.saturating_add(state.reserved_bytes);
        let free = self.space_probe.available_space(&self.root).unwrap_or(0);
        let projected = used.saturating_add(additional_bytes);
        if percent_at_least(
            projected,
            self.max_bytes,
            self.thresholds.backpressure_percent,
        ) || free < self.min_free_bytes.saturating_add(additional_bytes)
        {
            return Err(CapacityError {
                used_bytes: used,
                max_bytes: self.max_bytes,
                free_bytes: free,
                min_free_bytes: self.min_free_bytes,
            });
        }
        Ok(())
    }

    pub fn reconcile(&self) -> Result<()> {
        let measured = directory_bytes(&self.root)?;
        let mut state = self.state.lock().expect("capacity state lock poisoned");
        state.committed_bytes = measured;
        Ok(())
    }

    pub fn level(&self) -> CapacityLevel {
        let used = self.used_bytes();
        if percent_at_least(used, self.max_bytes, self.thresholds.critical_percent) {
            CapacityLevel::Critical
        } else if percent_at_least(
            used,
            self.max_bytes,
            self.thresholds.backpressure_percent,
        ) {
            CapacityLevel::Backpressure
        } else if percent_at_least(used, self.max_bytes, self.thresholds.warning_percent) {
            CapacityLevel::Warning
        } else {
            CapacityLevel::Normal
        }
    }

    pub fn used_bytes(&self) -> u64 {
        let state = self.state.lock().expect("capacity state lock poisoned");
        state.committed_bytes.saturating_add(state.reserved_bytes)
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }
}

impl CapacityReservation<'_> {
    pub fn commit(mut self) {
        let mut state = self
            .guard
            .state
            .lock()
            .expect("capacity state lock poisoned");
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.bytes);
        state.committed_bytes = state.committed_bytes.saturating_add(self.bytes);
        self.committed = true;
    }
}

impl Drop for CapacityReservation<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        let mut state = self
            .guard
            .state
            .lock()
            .expect("capacity state lock poisoned");
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.bytes);
    }
}

fn percent_at_least(used: u64, max: u64, percent: u8) -> bool {
    used.saturating_mul(100) >= max.max(1).saturating_mul(u64::from(percent))
}

fn directory_bytes(root: &Path) -> Result<u64> {
    if !root.exists() {
        return Ok(0);
    }
    let mut bytes = 0u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path)
            .with_context(|| format!("measure local storage {}", path.display()))?
        {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                bytes = bytes.saturating_add(metadata.len());
            } else if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                bytes = bytes.saturating_add(metadata.len());
            }
        }
    }
    Ok(bytes)
}

//! Durable apply-floor coordination for state machines that also maintain
//! independent domain storage.

use std::io;
use std::path::{Path, PathBuf};

use raft_core::Index;
use storage_durable::{atomic_write, FsyncPolicy};

/// A tiny fsynced index marker. Most in-memory state machines should rely on
/// [`crate::RaftStore`]'s committed log and snapshot recovery instead. This is
/// for adapters such as Relay whose domain log is independently durable and
/// must not be replayed below its already-applied floor.
#[derive(Debug, Clone)]
pub struct AppliedIndexStore {
    path: PathBuf,
}

impl AppliedIndexStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> io::Result<Index> {
        match std::fs::read_to_string(&self.path) {
            Ok(value) => value.trim().parse::<Index>().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("corrupt applied marker {}: {error}", self.path.display()),
                )
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
            Err(error) => Err(error),
        }
    }

    pub fn store(&self, index: Index) -> io::Result<()> {
        atomic_write(
            &self.path,
            index.to_string().as_bytes(),
            FsyncPolicy::Always,
        )
        .map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::AppliedIndexStore;

    #[test]
    fn missing_is_zero_and_stored_floor_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppliedIndexStore::new(dir.path().join("applied.idx"));
        assert_eq!(store.load().unwrap(), 0);
        store.store(42).unwrap();
        assert_eq!(store.load().unwrap(), 42);
    }
}

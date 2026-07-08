---
id: libs-raft-host-src-store-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/store.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/store.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/store.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FsyncPolicy` | libs/raft-host/src/store.rs | re-export | pub | 14 | pub use service_durability::FsyncPolicy; |
| `RaftStore` | libs/raft-host/src/store.rs | struct | pub | 18 | pub struct RaftStore { |
| `open` | libs/raft-host/src/store.rs | function | pub | 25 | pub fn open(dir: &str, node_id: NodeId, fsync: FsyncPolicy) -> io::Result<RaftStore> { |
| `save` | libs/raft-host/src/store.rs | function | pub | 36 | pub fn save(&self, state: &PersistedState) -> io::Result<()> { |
| `load` | libs/raft-host/src/store.rs | function | pub | 44 | pub fn load(&self) -> io::Result<Option<PersistedState>> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Durable storage for a raft node's hard state.
//!
//! Persists [`PersistedState`] (term, votedFor, log, snapshot) to a single file
//! under a data dir, written atomically (temp + rename) and fsynced per
//! [`FsyncPolicy`]. The host calls [`RaftStore::save`] *before* flushing the
//! node's outbox, so no vote or ack is sent before the decision that produced it
//! is durable. (Lifted from lumen/relay's identical `raft_store`.)

use std::fs::create_dir_all;
use std::io;
use std::path::PathBuf;

use raft_core::{NodeId, PersistedState};
pub use service_durability::FsyncPolicy;

/// File-backed persistence for one raft node.
pub struct RaftStore {
    path: PathBuf,
    fsync: FsyncPolicy,
}

impl RaftStore {
    /// Open (creating the dir if needed) the state file `raft-<node_id>.state`.
    pub fn open(dir: &str, node_id: NodeId, fsync: FsyncPolicy) -> io::Result<RaftStore> {
        let dir = PathBuf::from(dir);
        create_dir_all(&dir)?;
        Ok(RaftStore {
            path: dir.join(format!("raft-{node_id}.state")),
            fsync,
        })
    }

    /// Durably persist the hard state (atomic temp-write + rename, fsync unless
    /// [`FsyncPolicy::Os`]).
    pub fn save(&self, state: &PersistedState) -> io::Result<()> {
        let bytes =
            serde_json::to_vec(state).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        service_durability::atomic_write(&self.path, &bytes, self.fsync)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    /// Load the persisted hard state, or `None` if this node has none yet.
    pub fn load(&self) -> io::Result<Option<PersistedState>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => {
                Ok(Some(serde_json::from_slice(&bytes).map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/store.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/store.rs` captured during libs codegen standardization.
```

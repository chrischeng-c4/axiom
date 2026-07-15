---
id: libs-storage-durable-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/storage-durable/src/lib.rs`.
capability_refs:
  - id: shared-storage-durable-contract
    role: primary
    gap: shared-storage-durable-contract
    claim: shared-storage-durable-contract
    coverage: full
    rationale: "The source unit exposes the shared durability API."
fill_sections: [overview, source, changes]
---

# Standardized libs/storage-durable/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/storage-durable/src/lib.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `atomic_write` | libs/storage-durable/src/lib.rs | re-export | pub | atomic::atomic_write |
| `sync_parent_dir` | libs/storage-durable/src/lib.rs | re-export | pub | atomic::sync_parent_dir |
| `FramedLogReader` | libs/storage-durable/src/lib.rs | re-export | pub | framed_log::FramedLogReader |
| `FramedLogWriter` | libs/storage-durable/src/lib.rs | re-export | pub | framed_log::FramedLogWriter |
| `LogFrame` | libs/storage-durable/src/lib.rs | re-export | pub | framed_log::LogFrame |
| `FsyncPolicy` | libs/storage-durable/src/lib.rs | re-export | pub | fsync::FsyncPolicy |
| `SnapshotFile` | libs/storage-durable/src/lib.rs | re-export | pub | snapshot_store::SnapshotFile |
| `SnapshotFileStore` | libs/storage-durable/src/lib.rs | re-export | pub | snapshot_store::SnapshotFileStore |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Shared durable local storage primitives for axiom services.
//!
//! Services still own their domain record and snapshot codecs. This crate owns
//! the repeated mechanics around durable local files: fsync policy, atomic
//! replacement, CRC-framed append logs, and sequence-named snapshot stores.

mod atomic;
mod framed_log;
mod fsync;
mod snapshot_store;

pub use atomic::{atomic_write, sync_parent_dir};
pub use framed_log::{FramedLogReader, FramedLogWriter, LogFrame};
pub use fsync::FsyncPolicy;
pub use snapshot_store::{SnapshotFile, SnapshotFileStore};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/storage-durable/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/storage-durable/src/lib.rs`.
```

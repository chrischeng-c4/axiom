---
id: libs-service-durability-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/service-durability/src/lib.rs`.
capability_refs:
  - id: shared-service-durability-contract
    role: primary
    gap: shared-service-durability-contract
    claim: shared-service-durability-contract
    coverage: full
    rationale: "The source unit exposes the shared durability API."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-durability/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Lossless rust-source-unit coverage for `libs/service-durability/src/lib.rs`.

### Symbols

| Name | Target | Kind | Visibility | Signature |
|------|--------|------|------------|-----------|
| `atomic_write` | libs/service-durability/src/lib.rs | re-export | pub | atomic::atomic_write |
| `sync_parent_dir` | libs/service-durability/src/lib.rs | re-export | pub | atomic::sync_parent_dir |
| `FramedLogReader` | libs/service-durability/src/lib.rs | re-export | pub | framed_log::FramedLogReader |
| `FramedLogWriter` | libs/service-durability/src/lib.rs | re-export | pub | framed_log::FramedLogWriter |
| `LogFrame` | libs/service-durability/src/lib.rs | re-export | pub | framed_log::LogFrame |
| `FsyncPolicy` | libs/service-durability/src/lib.rs | re-export | pub | fsync::FsyncPolicy |
| `SnapshotFile` | libs/service-durability/src/lib.rs | re-export | pub | snapshot_store::SnapshotFile |
| `SnapshotFileStore` | libs/service-durability/src/lib.rs | re-export | pub | snapshot_store::SnapshotFileStore |

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
  - path: "libs/service-durability/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-durability/src/lib.rs`.
```

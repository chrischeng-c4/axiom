---
id: libs-service-backup-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/service-backup/src/lib.rs`.
capability_refs:
  - id: shared-service-backup-contract
    role: primary
    claim: shared-service-backup-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Backup library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-backup/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-backup/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `llm` | libs/service-backup/src/lib.rs | module | pub | 17 | pub mod llm; |
| `BackupDestination` | libs/service-backup/src/lib.rs | re-export | pub | 25 | pub use destination::BackupDestination; |
| `BackupPolicy` | libs/service-backup/src/lib.rs | re-export | pub | 26 | pub use policy::{BackupPolicy, RetentionPolicy}; |
| `RetentionPolicy` | libs/service-backup/src/lib.rs | re-export | pub | 26 | pub use policy::{BackupPolicy, RetentionPolicy}; |
| `run_backup_once` | libs/service-backup/src/lib.rs | re-export | pub | 27 | pub use runner::{run_backup_once, BackupObject, BackupRunResult}; |
| `BackupObject` | libs/service-backup/src/lib.rs | re-export | pub | 27 | pub use runner::{run_backup_once, BackupObject, BackupRunResult}; |
| `BackupRunResult` | libs/service-backup/src/lib.rs | re-export | pub | 27 | pub use runner::{run_backup_once, BackupObject, BackupRunResult}; |
| `sink_from_destination` | libs/service-backup/src/lib.rs | re-export | pub | 28 | pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink}; |
| `BackupSink` | libs/service-backup/src/lib.rs | re-export | pub | 28 | pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink}; |
| `LocalFsSink` | libs/service-backup/src/lib.rs | re-export | pub | 28 | pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink}; |
| `UnsupportedCloudSink` | libs/service-backup/src/lib.rs | re-export | pub | 28 | pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink}; |
| `fetch_backup_object` | libs/service-backup/src/lib.rs | re-export | pub | 29 | pub use source::fetch_backup_object; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `service-backup` — shared backup contract for axiom services.
//!
//! The data plane owns snapshot consistency: each service state machine produces
//! bytes at a concrete applied index, and `raft-host` handles snapshot install
//! plus log compaction. This crate owns the cross-service backup shape around
//! those bytes: destination/policy schema, sink trait, local sink, and a small
//! runner primitive.
//!
//! Operator code should render/manage a backup runner from the policy. The
//! runner calls the service's admin backup endpoint or CLI, then writes the
//! returned bytes through a [`BackupSink`]. Local is always available, S3 is
//! feature-gated here, and `gs://` remains schema-compatible until this crate
//! grows a real GCS sink. Bootstrap/restore flows can read exact snapshot
//! object URIs through [`fetch_backup_object`].

mod destination;
pub mod llm;
mod policy;
mod runner;
#[cfg(feature = "s3")]
mod s3;
mod sink;
mod source;

pub use destination::BackupDestination;
pub use policy::{BackupPolicy, RetentionPolicy};
pub use runner::{run_backup_once, BackupObject, BackupRunResult};
pub use sink::{sink_from_destination, BackupSink, LocalFsSink, UnsupportedCloudSink};
pub use source::fetch_backup_object;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-backup/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-backup/src/lib.rs` captured during libs codegen standardization.
```

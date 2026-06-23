---
id: projects-lumen-src-lib-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/lib.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `aof` | projects/lumen/src/lib.rs | module | pub |
| `api` | projects/lumen/src/lib.rs | module | pub |
| `auth` | projects/lumen/src/lib.rs | module | pub |
| `backup_sink` | projects/lumen/src/lib.rs | module | pub |
| `config` | projects/lumen/src/lib.rs | module | pub |
| `consumer` | projects/lumen/src/lib.rs | module | pub |
| `coordinator` | projects/lumen/src/lib.rs | module | pub |
| `log_entry` | projects/lumen/src/lib.rs | module | pub |
| `metrics` | projects/lumen/src/lib.rs | module | pub |
| `native_wire` | projects/lumen/src/lib.rs | module | pub |
| `operator` | projects/lumen/src/lib.rs | module | pub |
| `raft` | projects/lumen/src/lib.rs | module | pub |
| `rdb` | projects/lumen/src/lib.rs | module | pub |
| `routing` | projects/lumen/src/lib.rs | module | pub |
| `segment_rdb` | projects/lumen/src/lib.rs | module | pub |
| `spec` | projects/lumen/src/lib.rs | module | pub |
| `storage` | projects/lumen/src/lib.rs | module | pub |
| `tls` | projects/lumen/src/lib.rs | module | pub |
| `tokenize` | projects/lumen/src/lib.rs | module | pub |
| `types` | projects/lumen/src/lib.rs | module | pub |
| `vector_index` | projects/lumen/src/lib.rs | module | pub |
| `wal` | projects/lumen/src/lib.rs | module | pub |
| `wal_nats` | projects/lumen/src/lib.rs | module | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! lumen — standalone search and duplicate-detection index.
//!
//! Solves the gap B-tree indexes can't fill: keyword search (incl. Chinese
//! tokenisation) and duplicate detection. Exposed as a generic
//! `Collection / Field` primitive over `external_id` — lumen never owns
//! the source of truth and has no document concept of its own.
//!
//! - Durable via the configured write log; multi-pod Lumen is moving to
//!   Lumen-owned primary/replica replication, while Relay remains an explicit
//!   external broker mode. Rebuildable from the caller.
//! - HTTP/2 transport, client-side collection-shard routing.
//!
//! Full surface and v1 scope: `projects/lumen/README.md`.

/// Local append-only log (Stage 2 Phase 2f-3): the binary's "AOF" — a framed,
/// crash-safe record of every APPLIED `(seq, WalRecord)`. Recovery is RDB (the
/// segment checkpoint, up to seq S) → AOF replay (S+1..A) → broker tail (A+1..),
/// so broker retention can be TRIMMED instead of kept from seq 0. Compiled by
/// default; only the runtime segment-persistence path (`--persistence=segment`)
/// drives the apply loop + cold-start through it.
pub mod aof;
pub mod api;
pub mod auth;
pub mod backup_sink;
pub mod config;
pub mod consumer;
pub mod coordinator;
/// Write-log entry vocabulary (always compiled; the active write path uses it).
pub mod log_entry;
pub mod metrics;
/// Native length-prefixed CBOR search wire for Rust clients that need the engine
/// over a lower fixed-cost transport than HTTP/JSON.
pub mod native_wire;
/// K8s Operator: the `Lumen` CRD plus the reconcile loop that renders + applies
/// the serving fleet and Relay broker. Behind the `operator` feature so the
/// serving binary never pulls in kube-rs.
#[cfg(feature = "operator")]
pub mod operator;
/// Cluster-state view types backing the read/admin API. This surface is the
/// compatibility bridge for Lumen-owned primary/replica replication.
pub mod raft;
pub mod rdb;
pub mod routing;
/// Columnar mmap disk segment (Stage 2 disk-tier): a single Number column
/// for `n_docs` rows at one `applied_seq`, written page-aligned for zero-copy
/// reads. Compiled by default; the disk tier is selected at runtime
/// (`--persistence=segment`), with the in-RAM CBOR RDB remaining the default.
mod segment;
/// Segment-checkpoint persistence store (Stage 2 Phase 2f-2): the disk engine
/// as the running binary's "RDB" — a generation-versioned directory of per-
/// collection segment checkpoints, written atomically (stage + rename) so a torn
/// checkpoint never replaces a good one. Parallels [`rdb::LocalFsRdbStore`].
/// Compiled by default; selected at runtime via `--persistence=segment` (the
/// default binary keeps the CBOR RDB).
pub mod segment_rdb;
/// Offline machine-readable self-description (`lumen spec`): OpenAPI / JSON
/// schema, the query-shape cookbook, and the field/analyzer catalog — the
/// agent-integration surface, emitted without a running server.
pub mod spec;
pub mod storage;
pub mod tls;
pub mod tokenize;
pub mod types;
pub mod vector_index;
pub mod wal;
pub mod wal_nats;
#[cfg(feature = "relay-wal")]
pub mod wal_relay;
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/lib.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

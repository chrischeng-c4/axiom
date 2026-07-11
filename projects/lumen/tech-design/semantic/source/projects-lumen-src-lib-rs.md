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

Public API manifest for `projects/lumen/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `aof` | projects/lumen/src/lib.rs | module | pub | 22 |  |
| `api` | projects/lumen/src/lib.rs | module | pub | 23 |  |
| `auth` | projects/lumen/src/lib.rs | module | pub | 24 |  |
| `backup` | projects/lumen/src/lib.rs | module | pub | 33 |  |
| `backup_sink` | projects/lumen/src/lib.rs | module | pub | 34 |  |
| `config` | projects/lumen/src/lib.rs | module | pub | 35 |  |
| `consumer` | projects/lumen/src/lib.rs | module | pub | 36 |  |
| `coordinator` | projects/lumen/src/lib.rs | module | pub | 37 |  |
| `log_entry` | projects/lumen/src/lib.rs | module | pub | 39 |  |
| `metrics` | projects/lumen/src/lib.rs | module | pub | 40 |  |
| `native_wire` | projects/lumen/src/lib.rs | module | pub | 43 |  |
| `operator` | projects/lumen/src/lib.rs | module | pub | 48 |  |
| `raft` | projects/lumen/src/lib.rs | module | pub | 51 |  |
| `raft_sm` | projects/lumen/src/lib.rs | module | pub | 57 |  |
| `rdb` | projects/lumen/src/lib.rs | module | pub | 58 |  |
| `reshard` | projects/lumen/src/lib.rs | module | pub | 59 |  |
| `routing` | projects/lumen/src/lib.rs | module | pub | 60 |  |
| `routing_remote` | projects/lumen/src/lib.rs | module | pub | 67 |  |
| `segment_rdb` | projects/lumen/src/lib.rs | module | pub | 79 |  |
| `spec` | projects/lumen/src/lib.rs | module | pub | 83 |  |
| `storage` | projects/lumen/src/lib.rs | module | pub | 84 |  |
| `tls` | projects/lumen/src/lib.rs | module | pub | 85 |  |
| `tokenize` | projects/lumen/src/lib.rs | module | pub | 86 |  |
| `types` | projects/lumen/src/lib.rs | module | pub | 87 |  |
| `vector_index` | projects/lumen/src/lib.rs | module | pub | 88 |  |
| `wal` | projects/lumen/src/lib.rs | module | pub | 89 |  |
| `wal_nats` | projects/lumen/src/lib.rs | module | pub | 90 |  |
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
//! - Durable via the configured write log; multi-pod Lumen uses Lumen-owned
//!   primary/replica replication. Rebuildable from the caller.
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
/// `lumen backup` (#808): fetches a consistent snapshot from a running
/// serving fleet's existing `GET /admin/backup` endpoint and hands it to a
/// `libs/service-backup` destination sink. No new snapshot mechanism — this
/// is transport/scheduling only, meant to be driven by the operator's
/// optional backup CronJob (`spec.backup`, see `operator::render`) or ad hoc.
/// Behind the `backup` feature (pulled in by `operator`) since it needs an
/// HTTP client; `raft-wal` already links one into every shipped binary.
#[cfg(feature = "backup")]
pub mod backup;
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
/// the Lumen serving/data-plane resources. Behind the `operator` feature so the
/// serving binary never pulls in kube-rs.
#[cfg(feature = "operator")]
pub mod operator;
/// Cluster-state view types backing the read/admin API. This surface is the
/// compatibility bridge for Lumen-owned primary/replica replication.
pub mod raft;
/// `EngineSm` — lumen's `Engine` as a shared-`raft_host` state machine: the
/// convergence onto `libs/raft-host` (#524). The host is the sole applier, so
/// the per-service driver, durable hard state, and the WAL seam are no longer
/// lumen's to own — they live in the shared lib.
#[cfg(feature = "raft-wal")]
pub mod raft_sm;
pub mod rdb;
pub mod reshard;
pub mod routing;
/// Cross-pod shard routing for operator/k8s serving pods (#1398 R1-R3): local
/// reads/writes hit the engine directly, remote-owned buckets forward one hop
/// over h2c. Behind `operator` because it is the only module that needs
/// `reqwest` as a directly-nameable type; every real deployment already links
/// it via `operator`'s `backup` feature, so this adds no new crate.
#[cfg(feature = "operator")]
pub mod routing_remote;
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

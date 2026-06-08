//! lumen — standalone search and duplicate-detection index.
//!
//! Solves the gap B-tree indexes can't fill: keyword search (incl. Chinese
//! tokenisation) and duplicate detection. Exposed as a generic
//! `Collection / Field` primitive over `external_id` — lumen never owns
//! the source of truth and has no document concept of its own.
//!
//! - Durable via Raft (per-shard group), rebuildable from the caller.
//! - HTTP/2 transport, server self-routing to the Raft leader (no proxy).
//!
//! Full surface and v1 scope: `projects/lumen/README.md`.

pub mod api;
pub mod auth;
pub mod backup_sink;
pub mod client;
pub mod config;
pub mod consumer;
pub mod coordinator;
pub mod gpu_probe;
/// Write-log entry vocabulary (always compiled; the active write path uses it).
pub mod log_entry;
pub mod metrics;
/// K8s Operator: the `Lumen` CRD plus the reconcile loop that renders + applies
/// the serving fleet and NATS broker. Behind the `operator` feature so the
/// serving binary never pulls in kube-rs.
#[cfg(feature = "operator")]
pub mod operator;
/// Cluster-state view types backing the read/admin API. lumen has no
/// application consensus layer — durability + replication is the NATS
/// JetStream write-log, and serving nodes are full replicas that tail it.
pub mod raft;
pub mod rdb;
pub mod routing;
/// Offline machine-readable self-description (`lumen spec`): OpenAPI / JSON
/// schema, the query-shape cookbook, and the field/analyzer catalog — the
/// agent-integration surface, emitted without a running server.
pub mod spec;
pub mod storage;
pub mod storage_backend;
/// Log-structured disk backend (the only `storage_backend::Backend` impl) —
/// feature-gated `experimental`; the serve path runs the in-memory engine.
#[cfg(feature = "experimental")]
pub mod storage_lsm;
pub mod tls;
pub mod tokenize;
pub mod types;
pub mod vector_index;
pub mod wal;
pub mod wal_nats;

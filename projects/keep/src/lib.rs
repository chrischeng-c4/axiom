//! keep — cloud-native, multi-core key-value / claim-check store.
//!
//! # Layers
//! - [`engine`] — sharded, multi-core in-memory store (CAS, locks, TTL, lists, ...)
//! - [`persistence`] — tiered RAM + disk durability (WAL, snapshot, recovery)
//! - [`http`] — HTTP/2 + OpenAPI transport (axum over hyper)
//!
//! The engine and persistence layers are transport-agnostic; [`http`] is the
//! only network surface. There is no raw-TCP protocol — polyglot clients
//! integrate against the generated OpenAPI document at `/openapi.json`.

// WIP: suppress clippy noise during the takeover from cclab-kv.
#![allow(clippy::all)]

#[cfg(feature = "client")]
pub mod client;
pub mod cluster;
pub mod engine;
pub mod error;
pub mod http;
pub mod metrics;
#[cfg(feature = "operator")]
pub mod operator;
pub mod persistence;
#[cfg(feature = "raft")]
pub mod raft;
pub mod tls;
pub mod types;

pub use cluster::{ClusterConfig, ClusterState};
pub use engine::{EvictionPolicy, KvEngine};
pub use error::KvError;
pub use http::{router, ApiDoc, AppState};
pub use types::{KvKey, KvValue};

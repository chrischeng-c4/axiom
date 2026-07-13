// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! Wire-shape response bodies for `/pools`, `/pools/{pool}/stats`,
//! `/readyz`, and `POST /drain` (TD Schema section). `PoolStatsResponse`/
//! `PoolListResponse` are field-identical to the `PoolStats`/`PoolList`
//! schemas `apps/pgpool/src/spec.rs`'s offline `schemas()` already
//! declares, so the served JSON and `pgpool spec --format openapi`'s
//! component schemas stay byte-for-byte in sync (R4/AC3). Named
//! `*Response` here (rather than reusing `PoolStats`/`PoolList` as Rust
//! type names) only to avoid colliding with `crate::pool::types::PoolStats`
//! (WI #1289's plain-Rust-API type, which carries no name/mode fields) —
//! the JSON shape produced by `#[derive(Serialize)]` is what matters for
//! parity, not the Rust type name.

use serde::Serialize;

/// Response body for `GET /pools/{pool}/stats` and each entry of
/// `PoolListResponse.pools`. Derived per-request from one
/// `crate::admin::NamedPool`: `name`/`mode` copied directly,
/// `frontend_active` from `budget.active()`, `backend_active`/
/// `backend_idle` from `pool.stats()` (WI #1289 `BackendPoolStats`).
#[derive(Debug, Clone, Serialize)]
pub struct PoolStatsResponse {
    pub name: String,
    /// `"session"` or `"transaction"`, matching `spec.rs`'s `PoolStats`
    /// schema enum exactly (lowercase) — NOT `PoolMode`'s own
    /// `Debug`/serde spelling.
    pub mode: String,
    pub frontend_active: usize,
    pub backend_active: usize,
    pub backend_idle: usize,
}

/// Response body for `GET /pools`; matches `spec.rs`'s offline `PoolList`
/// schema field-for-field (R4, AC3).
#[derive(Debug, Clone, Serialize)]
pub struct PoolListResponse {
    pub pools: Vec<PoolStatsResponse>,
}

/// Response body for `POST /drain`, matching `spec.rs`'s offline
/// `DrainState` schema (single required boolean field, R4/AC3). Always
/// `true` in the response body — there is no un-drain verb — reflecting
/// `AdminState.drain.is_draining()` immediately after the call
/// (idempotent: repeated `POST /drain` calls return the same body).
#[derive(Debug, Clone, Serialize)]
pub struct DrainResponse {
    pub draining: bool,
}

/// Plain-text-equivalent body for `GET /readyz`; the HTTP status code (200
/// vs 503) is the primary readiness signal, this body is a
/// human-diagnostic supplement (R2).
#[derive(Debug, Clone, Serialize)]
pub struct ReadyzResponse {
    pub status: &'static str,
}
// </HANDWRITE>

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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
// CODEGEN-BEGIN
use serde::Deserialize;

/// Internal (non-serialized-as-JSON) shape the /metrics handler folds every AdminState.pools entry into before rendering Prometheus text-format output; not part of the served JSON contract, only documents the pgpool_frontend_active / pgpool_backend_active / pgpool_backend_idle gauge rows (AC4). Rendered as `<metric>{pool="<pool>"} <value>` per Prometheus text exposition format 0.0.4.
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminMetricsLine {
    pub metric: String,
    pub pool: String,
    pub value: u64,
}

/// axum shared state for the admin Router (via axum::extract::State), constructed once in `serve()` alongside the TCP frontend's TcpServerConfig so both planes hold clones of the identical DrainController (R2). Cheap to clone per-request since DrainController, ConnectionBudget, and BackendPool are all Arc/watch-channel backed internally.
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminState {
    /// The one shared drain controller; /readyz reads drain.is_draining(), POST /drain calls drain.start_drain(), and the same clone is handed to TcpServerConfig.drain for the frontend accept loop and to the signal-handling task (R2).
    pub drain: server_core::DrainController,
    /// Every pool this pgpool process serves (currently always exactly one entry, named per Config's pool_name, since RuntimePlan is single-pool-per-process); GET /pools iterates this, GET /pools/{pool}/stats looks up by name (R3).
    pub pools: Vec<NamedPool>,
}

/// Response body for POST /drain, matching apps/pgpool/src/spec.rs's offline DrainState schema (single required boolean field, R4/AC3); returned after calling AdminState.drain.start_drain() (idempotent — repeated POSTs return the same {draining: true} body, see State Machine section).
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrainResponse {
    /// Always true in the response body (POST /drain only ever transitions toward draining; there is no un-drain verb) and reflects AdminState.drain.is_draining() immediately after the call.
    pub draining: bool,
}

/// Pairs one WI #1289 BackendPool (Arc-backed, cheap to clone) with the pool name and PoolMode the admin plane needs to answer /pools and /pools/{pool}/stats, since pool::types::PoolStats/BackendPoolStats carry no name/mode fields themselves (R3). Constructed once in `serve()` from RuntimePlan.pool_name (Config section) + RuntimePlan.pool_mode and stored in AdminState.pools.
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedPool {
    /// Operator-facing pool identifier; matches the {pool} path segment in GET /pools/{pool}/stats. Defaults to "default" (see Config section) since pgpool currently runs exactly one pool per process.
    pub name: String,
    /// Session or Transaction — the fixed-for-the-process mode already selected by RuntimePlan.pool_mode; surfaced read-only in PoolStats.mode.
    pub mode: crate::pool::PoolMode,
    /// The SAME ConnectionBudget instance the frontend accept path checks (RuntimePlan::frontend_budget); AdminState reads budget.active() for PoolStats.frontend_active and the pgpool_frontend_active metric gauge, never constructing a second budget (single source of truth).
    pub budget: server_core::ConnectionBudget,
    /// Arc-backed clone of the live BackendPool this pool name serves; AdminState calls pool.stats() (WI #1289) for backend_active/backend_idle on every /pools, /pools/{pool}/stats, and /metrics request — never a cached snapshot.
    pub pool: crate::pool::BackendPool,
}

/// Response body for GET /pools; matches apps/pgpool/src/spec.rs's offline PoolList schema field-for-field (R4, AC3).
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolList {
    pub pools: Vec<PoolStats>,
}

/// Response body for GET /pools/{pool}/stats and each entry of PoolList.pools; field names/shape are IDENTICAL to the `PoolStats` schema `apps/pgpool/src/spec.rs`'s offline `schemas()` already declares, so the served body and `pgpool spec --format openapi`'s component schema stay byte-for-byte in sync (R4, AC3). Derived per-request from one NamedPool: name/mode copied directly, frontend_active from budget.active(), backend_active/backend_idle from pool.stats() (WI #1289 pool::BackendPoolStats).
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolStats {
    pub name: String,
    pub mode: String,
    /// server_core::ConnectionBudget::active() for this pool's frontend budget (AC4 metric source).
    pub frontend_active: u64,
    /// pool::BackendPoolStats.backend_active from BackendPool::stats() (WI #1289) (AC4 metric source).
    pub backend_active: u64,
    /// pool::BackendPoolStats.backend_idle from BackendPool::stats() (WI #1289) (AC4 metric source).
    pub backend_idle: u64,
}

/// Plain-text-equivalent body for GET /readyz (status field mirrors the libs/service-http probe-route convention of a short status string); HTTP status code (200 vs 503) is the primary readiness signal consumers rely on, this body is a human-diagnostic supplement (R2).
/// @spec apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadyzResponse {
    pub status: String,
}
// CODEGEN-END

// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! `AdminState`/`NamedPool` (TD Schema section): the axum shared state for
//! the admin router — one shared `server_core::DrainController` clone (the
//! SAME instance handed to the TCP frontend's `TcpServerConfig.drain` and
//! the SIGTERM/SIGINT signal task in `serve()`, R2/R7) plus every named
//! pool this process serves.

use std::sync::Arc;

use server_core::{ConnectionBudget, DrainController, DrainSignal};

use crate::pool::BackendPool;
use crate::PoolMode;

/// Pairs one WI #1289 [`BackendPool`] (Arc-backed, cheap to clone) with the
/// pool name and [`PoolMode`] the admin plane needs to answer `/pools` and
/// `/pools/{pool}/stats`, since `pool::types::PoolStats`/`BackendPoolStats`
/// carry no name/mode fields themselves (R3).
#[derive(Clone)]
pub struct NamedPool {
    /// Operator-facing pool identifier; matches the `{pool}` path segment
    /// in `GET /pools/{pool}/stats`. Defaults to `"default"` (Config
    /// section) since pgpool currently runs exactly one pool per process.
    pub name: String,
    /// Session or Transaction — the fixed-for-the-process mode already
    /// selected by `RuntimePlan::pool_mode`; surfaced read-only in
    /// `PoolStatsResponse::mode`.
    pub mode: PoolMode,
    /// The SAME `ConnectionBudget` the frontend accept path checks
    /// (`RuntimePlan::frontend_budget`); never a second budget.
    pub budget: ConnectionBudget,
    /// Arc-backed clone of the live `BackendPool` this pool name serves;
    /// `pool.stats()` (WI #1289) is read live on every request, never
    /// cached.
    pub pool: BackendPool,
}

/// axum shared state for the admin `Router` (via `axum::extract::State`),
/// constructed once in `serve()` alongside the TCP frontend's
/// `TcpServerConfig` so both planes hold clones of the identical
/// `DrainController` (R2). Cheap to clone per-request since
/// `DrainController`, `ConnectionBudget`, and `BackendPool` are all
/// Arc/watch-channel backed internally.
#[derive(Clone)]
pub struct AdminState {
    /// The one shared drain controller; `/readyz` reads
    /// `drain.is_draining()`, `POST /drain` calls `drain.start_drain()`,
    /// and the same clone is handed to `TcpServerConfig.drain` and the
    /// signal-handling task (R2).
    pub drain: DrainController,
    /// Every pool this pgpool process serves (currently always exactly one
    /// entry, named per Config's `pool_name`, since `RuntimePlan` is
    /// single-pool-per-process); `GET /pools` iterates this, `GET
    /// /pools/{pool}/stats` looks up by name (R3).
    pub pools: Arc<Vec<NamedPool>>,
    /// A held-open subscription on `drain`, never read directly. Exists
    /// solely to keep `tokio::sync::watch::Sender::send` from no-op'ing:
    /// with zero live receivers, `DrainController::start_drain()` silently
    /// fails to update the shared state (tokio's watch channel treats a
    /// receiver-less send as a hint and drops it). In production a
    /// receiver is always kept alive by the TCP frontend's
    /// `TcpServerConfig` and the SIGTERM/SIGINT signal task, but the admin
    /// plane must not depend on that external wiring order to make
    /// `POST /drain` correct on its own (R2, R7).
    _drain_signal: DrainSignal,
}

impl AdminState {
    pub fn new(drain: DrainController, pools: Vec<NamedPool>) -> Self {
        let _drain_signal = drain.signal();
        Self {
            drain,
            pools: Arc::new(pools),
            _drain_signal,
        }
    }

    /// Looks up a named pool by its operator-facing name (R3).
    pub fn find(&self, name: &str) -> Option<&NamedPool> {
        self.pools.iter().find(|pool| pool.name == name)
    }
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

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

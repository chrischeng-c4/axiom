// SPEC-MANAGED: apps/pgpool/tech-design/logic/served-admin-plane-with-drain-aware-readiness.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-admin-plane" tracker="#1290" reason="Admin plane needs generator primitives that do not exist yet.">
//! `AdminState`/`NamedPool` (TD Schema section): the axum shared state for
//! the admin router — one shared `server_lifecycle::DrainController` clone (the
//! SAME instance handed to the TCP frontend's `TcpServerConfig.drain` and
//! the SIGTERM/SIGINT signal task in `serve()`, R2/R7) plus every named
//! pool this process serves.

use std::sync::Arc;

use server_lifecycle::{ConnectionBudget, DrainController, DrainSignal};

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

// <HANDWRITE gap="missing-generator:logic" tracker="#1884" reason="logic section in state.rs is hand-written pending codegen support">
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
    /// A held subscription that makes the admin plane an explicit startup
    /// participant in the shared drain channel. `DrainController::start_drain`
    /// uses durable publication, so correctness no longer relies on this
    /// receiver surviving a particular serve-startup ordering.
    _drain_signal: DrainSignal,
}
// </HANDWRITE>

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

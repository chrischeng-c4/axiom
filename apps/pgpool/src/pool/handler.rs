// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! `PoolHandler`: the single `tcp_server::TcpHandler` `pgpool serve` binds
//! to its listener, selected once at process start from
//! `RuntimePlan::pool_mode` (per the TD Schema section) and dispatching to
//! whichever mode-specific handler it wraps.

use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use tcp_server::{ConnectionContext, TcpHandler};
use tokio::net::TcpStream;

use crate::pool::transaction::TransactionHandler;
use crate::proxy::SessionHandler;

/// Process-start dispatch wrapper over the two pool-mode handlers; which
/// variant is constructed is decided once, from `RuntimePlan::pool_mode`,
/// not re-evaluated per connection.
#[derive(Debug, Clone)]
pub enum PoolHandler {
    Session(SessionHandler),
    Transaction(TransactionHandler),
}

impl TcpHandler for PoolHandler {
    type Future = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future {
        match self {
            PoolHandler::Session(handler) => handler.handle(stream, cx),
            PoolHandler::Transaction(handler) => handler.handle(stream, cx),
        }
    }
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// One leased physical backend connection returned by acquire()/acquire_fresh(): the live socket plus its BackendConnectionId and whether this lease required a fresh TCP connect.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendLease {
    pub id: BackendConnectionId,
    /// True when this lease required a brand-new TCP connect (acquire_fresh() always; acquire() only when the idle set was empty and capacity allowed a new connect) \u2014 signals the caller that a real startup+auth relay is required before the connection can carry client traffic; false means an already-authenticated idle connection was reused and only post-auth traffic should be relayed.
    pub fresh: bool,
    /// The live backend socket for this lease; the caller splits it (into_split) to run the same frame relay helpers session mode already uses.
    pub stream: tokio::net::TcpStream,
}

/// Raw counts BackendPool exposes for composition into PoolStats.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendPoolStats {
    /// Count of physical backend connections currently leased out (active) — includes both fresh and reused leases, both pool modes.
    pub backend_active: usize,
    /// Count of physical backend connections currently sitting in the shared idle set, already authenticated and liveness-eligible for reuse.
    pub backend_idle: usize,
}

/// How BackendPool::release() disposes of a returned lease: return_to_idle resets the connection (DISCARD ALL) and adds it to the shared idle set (R1, R2); close tears the physical connection down immediately and frees its capacity slot (session-mode teardown, or any lease whose session state is unknown/unsafe to reuse after a relay error, or a reset that itself failed).
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LeaseDisposition {
    #[serde(rename = "return_to_idle")]
    ReturnToIdle,
    #[serde(rename = "close")]
    Close,
}

/// Configuration for one BackendPool: the single configured backend endpoint (reused from proxy::BackendEndpointConfig), the capacity bound sourced from RuntimePlan::max_backend_connections (R1), and the timeouts/wire bounds the pool's own connect+relay/reset helpers need.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolConfig {
    /// The single configured backend this pool dials; multiple backend databases/pools keyed by database+user are out of scope for this slice (adapter-boundary epic #1283's seam).
    pub endpoint: crate::proxy::BackendEndpointConfig,
    /// Capacity bound shared by both pool modes (idle + active <= this value); sourced from RuntimePlan::max_backend_connections (default 512, R1).
    pub max_backend_connections: usize,
    /// Bounds how long BackendPool::acquire() waits for an idle/freed slot before returning PoolError::Saturated (R3, AC3).
    pub acquire_timeout: std::time::Duration,
    /// Bounds a fresh backend TCP connect from acquire()/acquire_fresh(); mirrors SessionProxyConfig::backend_connect_timeout.
    pub backend_connect_timeout: std::time::Duration,
    /// Frame bounds for the backend-role FrameReader the pool's own admission-handshake and reset helpers use.
    pub wire: crate::wire::WireCodecConfig,
}

/// Drives the synthesized wire ErrorResponse for a mid-session backend-pool-exhaustion rejection (R3, AC3), distinct from the existing proxy::RejectionReason (which covers frontend-admission and admission-handshake rejections): backend_pool_saturated maps to synthesized_error_response() returning an ErrorResponse with SQLSTATE 53300 too_many_connections, message text distinguishing "backend pool exhausted" from the frontend-budget wording so operators can tell the two saturation causes apart.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PoolRejectionReason {
    #[serde(rename = "backend_pool_saturated")]
    BackendPoolSaturated,
}

/// The plain Rust stats API (R4) a later admin-plane WI surfaces over HTTP (out of scope here \u2014 this slice ships only the Rust API); composes the existing server_core::ConnectionBudget::active() (frontend admission, unchanged from WI #1288) with BackendPoolStats (this TD) into one snapshot.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoolStats {
    /// ConnectionBudget::active() for the frontend listener — count of currently-admitted client connections, both pool modes.
    pub frontend_active: usize,
    /// Equal to BackendPoolStats.backend_active at snapshot time.
    pub backend_active: usize,
    /// Equal to BackendPoolStats.backend_idle at snapshot time.
    pub backend_idle: usize,
}

/// The tcp_server::TcpHandler impl pgpool serve binds to its listener in transaction mode: dispatches each accepted client through the admission-handshake-then-per-transaction-lease pipeline described in the Logic section, using its TransactionProxyConfig.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionHandler {
    /// Private field, constructed via TransactionHandler::new(config); mirrors SessionHandler's shape.
    pub config: TransactionProxyConfig,
}

/// Everything a TransactionHandler needs: reuses the same frontend_budget ConnectionBudget concept as SessionProxyConfig (frontend admission is a pool-mode-independent concern), the BackendPool this handler leases from, and the wire/drain bounds transaction-mode's own admission-handshake and per-transaction relay helpers need.
/// @spec apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionProxyConfig {
    /// Same admission primitive as SessionProxyConfig::frontend_budget — one shared frontend-connection cap regardless of pool mode.
    pub frontend_budget: server_core::ConnectionBudget,
    /// The shared, capacity-bounded backend pool this handler's admission handshakes and per-transaction leases draw from.
    pub backend_pool: crate::pool::BackendPool,
    /// Frame bounds used by the admission-handshake relay and the per-transaction bidirectional relay.
    pub wire: crate::wire::WireCodecConfig,
    /// Bounds how long an in-flight admission handshake or transaction lease is allowed to keep running after DrainSignal flips to Draining before the task is abandoned; mirrors SessionProxyConfig::drain_timeout.
    pub drain_timeout: std::time::Duration,
}
// CODEGEN-END

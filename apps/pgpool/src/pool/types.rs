// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! Backend connection pool types, per the TD Schema section: the pool's own
//! configuration, lease identity/disposition, its error taxonomy (including
//! the pool-saturation rejection distinct from `proxy::RejectionReason`),
//! and the plain Rust stats API (R4).

use std::time::Duration;

use crate::proxy::BackendEndpointConfig;
use crate::wire::{BackendMessage, ErrorResponse, WireCodecConfig};

/// Full configuration for one [`crate::pool::BackendPool`]: the single
/// configured backend it dials (shared with session-mode's target per this
/// slice — no per-mode backend target, no multi-backend keying; see the TD
/// Config section), its capacity bound (R1), and the timeouts that govern
/// acquire/connect.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub endpoint: BackendEndpointConfig,
    /// Capacity bound shared by idle+active backend connections, both pool
    /// modes (R1): `idle + active <= max_backend_connections`.
    pub max_backend_connections: usize,
    /// How long `BackendPool::acquire()`/`acquire_fresh()` wait for an
    /// idle/freed slot before `PoolError::Saturated` (R3b).
    pub acquire_timeout: Duration,
    /// Bounds every fresh backend TCP connect and the `DISCARD ALL` reset
    /// round-trip on `LeaseDisposition::ReturnToIdle` (no separate reset
    /// timeout config seam exists per the TD Config section).
    pub backend_connect_timeout: Duration,
    pub wire: WireCodecConfig,
}

/// Identifies one physical backend TCP connection across its idle/active
/// lifecycle, independent of which client currently leases it (if any).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BackendConnectionId(pub u64);

/// What `BackendPool::release` should do with a returned connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseDisposition {
    /// Reset via `DISCARD ALL` and, if that succeeds, parked in the idle set
    /// for reuse (R1, AC2). Closed instead if the reset fails, EOFs, or
    /// times out.
    ReturnToIdle,
    /// Torn down immediately; its capacity slot is freed for a future
    /// fresh-connect or idle-reuse.
    Close,
}

/// `BackendPool::acquire`/`acquire_fresh` failure modes.
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    /// `acquire_timeout` elapsed while the pool sat at
    /// `max_backend_connections` with no idle connection available; maps to
    /// [`PoolRejectionReason::BackendPoolSaturated`].
    #[error("backend pool saturated: max={max} waited={waited:?}")]
    Saturated { max: usize, waited: Duration },
    /// A fresh backend connect failed or timed out; maps to the existing
    /// `proxy::RejectionReason::BackendUnreachable` (SQLSTATE 08006).
    #[error("backend unreachable: {0}")]
    BackendUnreachable(String),
}

/// Pool-originated rejections that need their own synthesized wire
/// `ErrorResponse`, distinct from `proxy::RejectionReason` so operators can
/// tell frontend-budget saturation apart from backend-pool saturation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolRejectionReason {
    BackendPoolSaturated,
}

impl PoolRejectionReason {
    /// This variant always synthesizes an `ErrorResponse` (unlike
    /// `RejectionReason::synthesized_error_response`, which returns `None`
    /// for the backend-forwarded-verbatim case) — SQLSTATE `53300`
    /// (`too_many_connections`), with wording distinct from the frontend
    /// admission budget's message.
    pub fn synthesized_error_response(self) -> BackendMessage {
        let (sqlstate, message): (&str, &str) = match self {
            PoolRejectionReason::BackendPoolSaturated => (
                "53300",
                "pgpool backend connection pool exhausted; retry the transaction",
            ),
        };
        BackendMessage::ErrorResponse(ErrorResponse {
            fields: vec![
                (b'S', "FATAL".to_string()),
                (b'C', sqlstate.to_string()),
                (b'M', message.to_string()),
            ],
        })
    }
}

/// Snapshot of physical backend connections: how many are currently leased
/// out (active) vs. sitting idle, authenticated and liveness-eligible for
/// reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendPoolStats {
    /// Physical backend connections currently leased out — includes both
    /// fresh and reused leases, both pool modes.
    pub backend_active: usize,
    /// Connections currently sitting in the shared idle set.
    pub backend_idle: usize,
}

/// Composes `ConnectionBudget::active()` (frontend_active) with
/// `BackendPoolStats` (R4). The plain Rust stats API a later admin-plane WI
/// surfaces over HTTP — out of scope here, this slice ships only the Rust
/// API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolStats {
    pub frontend_active: usize,
    pub backend_active: usize,
    pub backend_idle: usize,
}
// </HANDWRITE>

// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Configuration types for the session-mode proxy: the single configured
//! Postgres backend endpoint and the full set of knobs `SessionHandler`
//! needs per the TD Schema section.

use std::time::Duration;

use server_core::ConnectionBudget;

use crate::wire::WireCodecConfig;

/// TCP host/port of the single configured Postgres backend this
/// session-mode proxy dials per client (R3). Credentials are never part of
/// this config — the proxy relays auth bytes opaquely and never persists
/// them (see [`crate::proxy::SessionHandler`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendEndpointConfig {
    pub host: String,
    pub port: u16,
}

/// Full configuration for one `SessionHandler`: the backend it dials, the
/// admission budget it enforces itself (deliberately not wired into
/// `tcp_server::TcpServerConfig.connection_budget` so a rejection can write
/// a wire-level `ErrorResponse` before closing), and the timeouts/wire
/// bounds that govern one session's lifetime.
#[derive(Debug, Clone)]
pub struct SessionProxyConfig {
    pub backend: BackendEndpointConfig,
    /// The same budget `RuntimePlan::frontend_budget()` constructs; admission
    /// is checked here (inside `SessionHandler::handle`), not via
    /// `tcp_server::TcpServerConfig.connection_budget`.
    pub frontend_budget: ConnectionBudget,
    /// Bounds the backend TCP connect (R3); exceeding this produces
    /// `RejectionReason::BackendUnreachable`.
    pub backend_connect_timeout: Duration,
    /// Mirrors `tcp_server::TcpServerConfig.drain_timeout` — one source of
    /// truth for AC4.
    pub drain_timeout: Duration,
    /// Frame bounds for the frontend-role/backend-role `FrameReader`
    /// instances this session's relay uses.
    pub wire: WireCodecConfig,
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// TCP host/port of the single configured Postgres backend this session-mode proxy dials per client (R3); sourced from PGPOOL_BACKEND_ADDR/--backend-addr. Credentials are never part of this config — auth is relayed frame-for-frame from the client, never generated or stored by pgpool (AC2). This seam is later formalized by the backend-adapter-seam epic #1283.
/// @spec apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendEndpointConfig {
    pub host: String,
    pub port: i64,
}

/// Drives the wire-level BackendMessage::ErrorResponse the client sees before the socket closes: frontend_budget_exhausted -> SQLSTATE 53300 too_many_connections (AC3); backend_unreachable -> SQLSTATE 08006 connection_failure; backend_auth_failed -> the backend's own ErrorResponse is forwarded verbatim instead of a synthesized one.
/// @spec apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RejectionReason {
    #[serde(rename = "frontend_budget_exhausted")]
    FrontendBudgetExhausted,
    #[serde(rename = "backend_unreachable")]
    BackendUnreachable,
    #[serde(rename = "backend_auth_failed")]
    BackendAuthFailed,
}

/// Terminal classification of one session, mirroring the Session State Machine's terminal states; SessionHandler::handle returns/records this so unit tests assert on a typed outcome instead of parsing logs (maps 1:1 onto rejected_saturated/rejected_backend_unreachable/rejected_auth_failed/closed(*) in the Session State Machine section).
/// @spec apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SessionOutcome {
    #[serde(rename = "rejected_saturated")]
    RejectedSaturated,
    #[serde(rename = "rejected_backend_unreachable")]
    RejectedBackendUnreachable,
    #[serde(rename = "rejected_auth_failed")]
    RejectedAuthFailed,
    #[serde(rename = "established_closed_clean")]
    EstablishedClosedClean,
    #[serde(rename = "established_closed_error")]
    EstablishedClosedError,
    #[serde(rename = "drain_abandoned")]
    DrainAbandoned,
}

/// Everything a SessionHandler needs to admit and relay one client session; constructed once from RuntimePlan + CLI/env backend config and shared (cheaply cloneable) across every accepted connection.
/// @spec apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionProxyConfig {
    pub backend: BackendEndpointConfig,
    /// Same budget RuntimePlan::frontend_budget() constructs; admission is checked here (inside SessionHandler::handle), not via tcp_server::TcpServerConfig.connection_budget, so a rejection can still write a wire-level ErrorResponse before the socket closes (R1, AC3).
    pub frontend_budget: server_core::ConnectionBudget,
    /// Bounds the backend TCP connect attempt (R3); exceeding it produces RejectionReason::BackendUnreachable.
    pub backend_connect_timeout: std::time::Duration,
    /// Mirrors tcp_server::TcpServerConfig.drain_timeout (itself from RuntimePlan.admin_drain_timeout-equivalent for the frontend listener) so the bounded-drain proof in AC4 has one source of truth.
    pub drain_timeout: std::time::Duration,
    /// Frame bounds/limits for the frontend-role and backend-role FrameReader instances this session constructs.
    pub wire: crate::wire::WireCodecConfig,
}
// CODEGEN-END

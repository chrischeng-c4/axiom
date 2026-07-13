// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Configuration types for the session-mode proxy: the single configured
//! Postgres backend endpoint and the full set of knobs `SessionHandler`
//! needs per the TD Schema section.

use std::time::Duration;

use server_core::ConnectionBudget;

use crate::pool::BackendPool;
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
    /// Shared backend-connection pool (WI #1289): session mode now dials via
    /// `BackendPool::acquire_fresh()` and tears down via
    /// `BackendPool::release(..., LeaseDisposition::Close)` instead of a raw
    /// `TcpStream::connect`, so both pool modes are capacity-bounded through
    /// the same `max_backend_connections` (R1).
    pub backend_pool: BackendPool,
}
// </HANDWRITE>

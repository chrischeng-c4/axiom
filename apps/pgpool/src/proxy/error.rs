// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#schema
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Rejection/outcome/error taxonomy for one session-mode proxy session, per
//! the TD Schema and Session State Machine sections.

use crate::wire::{BackendMessage, ErrorResponse, FrameError};

/// Drives the wire-level `BackendMessage::ErrorResponse` a rejected session
/// receives before the connection closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// The proxy's own frontend admission budget is exhausted (AC3);
    /// synthesizes SQLSTATE `53300` (`too_many_connections`).
    FrontendBudgetExhausted,
    /// The configured backend could not be reached within
    /// `backend_connect_timeout`; synthesizes SQLSTATE `08006`
    /// (`connection_failure`).
    BackendUnreachable,
    /// The backend itself sent an `ErrorResponse` before
    /// `AuthenticationOk` (or before `ReadyForQuery`); that response is
    /// forwarded to the client verbatim instead of being synthesized here.
    BackendAuthFailed,
}

impl RejectionReason {
    /// The synthesized `ErrorResponse` this proxy writes for a rejection it
    /// originates itself. Returns `None` for [`RejectionReason::BackendAuthFailed`],
    /// whose `ErrorResponse` is the backend's own frame, forwarded verbatim
    /// rather than synthesized (see the TD Schema section).
    pub fn synthesized_error_response(self) -> Option<BackendMessage> {
        let (sqlstate, message): (&str, &str) = match self {
            RejectionReason::FrontendBudgetExhausted => (
                "53300",
                "too many connections for this pgpool session-mode frontend",
            ),
            RejectionReason::BackendUnreachable => (
                "08006",
                "pgpool could not establish a connection to the configured backend",
            ),
            RejectionReason::BackendAuthFailed => return None,
        };
        Some(BackendMessage::ErrorResponse(ErrorResponse {
            fields: vec![
                (b'S', "FATAL".to_string()),
                (b'C', sqlstate.to_string()),
                (b'M', message.to_string()),
            ],
        }))
    }
}

/// Terminal classification mirroring the Session State Machine's terminal
/// states; `SessionHandler::handle` records this (via tracing) for every
/// session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionOutcome {
    RejectedSaturated,
    RejectedBackendUnreachable,
    RejectedAuthFailed,
    EstablishedClosedClean,
    EstablishedClosedError,
    /// Classification for a session whose `drain_timeout` elapsed while
    /// `Established`/`Draining`: `tcp_server::serve_arc` abandons the task at
    /// that point (see its bounded drain loop), so this variant documents
    /// the outcome rather than being returned by `run_session` itself — an
    /// abandoned future is dropped, not resolved.
    DrainAbandoned,
}

/// Every variant is handled by writing the appropriate wire frame (or none)
/// and releasing the admission permit; a session task built from these
/// variants never panics.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// Admission or backend-connect rejection.
    #[error("session rejected: {reason:?}")]
    Rejection { reason: RejectionReason },
    /// A frontend or backend leg's frame failed to decode; that leg's relay
    /// ends without forwarding the offending bytes.
    #[error(transparent)]
    Wire(#[from] FrameError),
    /// Underlying client/backend socket I/O error.
    #[error("proxy io error: {0}")]
    Io(String),
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

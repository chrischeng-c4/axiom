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

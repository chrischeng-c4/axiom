//! Error type for the frame-level [`H2cManager`](crate::H2cManager).

use std::io;
use std::time::Duration;

/// Errors from the frame-level h2c connection manager.
#[derive(Debug, thiserror::Error)]
pub enum H2cError {
    /// TCP connect / socket setup failed.
    #[error("connect {authority}: {source}")]
    Connect {
        /// The `host:port` we were dialing.
        authority: String,
        /// The underlying socket error.
        #[source]
        source: io::Error,
    },
    /// The h2 protocol surfaced an error (handshake, stream reset, GOAWAY, …).
    #[error("h2 protocol: {0}")]
    H2(#[from] h2::Error),
    /// No healthy connection to the authority was available.
    #[error("no healthy h2c connection to {0}")]
    NoConnection(String),
    /// A connect or request exceeded its configured deadline.
    #[error("timed out after {0:?}")]
    Timeout(Duration),
    /// The manager has been shut down and refuses new work.
    #[error("h2c manager is shut down")]
    Shutdown,
    /// Building the outbound request failed (bad URI / header).
    #[error("invalid request: {0}")]
    Request(#[from] http::Error),
}

impl H2cError {
    /// Whether this error means the connection is gone (GOAWAY / reset / I/O) —
    /// the manager retries such requests on a fresh connection.
    pub fn is_connection_lost(&self) -> bool {
        match self {
            H2cError::H2(e) => e.is_go_away() || e.is_io() || e.is_reset(),
            H2cError::Connect { .. } | H2cError::NoConnection(_) => true,
            _ => false,
        }
    }
}

/// Result alias for the frame-level manager.
pub type Result<T> = std::result::Result<T, H2cError>;

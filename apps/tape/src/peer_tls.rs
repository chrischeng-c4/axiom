// HANDWRITE-BEGIN gap="missing-generator:logic:f0da944f" tracker="#1704" reason="Tape owns only its TAPE_PEER environment prefix; peer-tls owns validated material and raft-runtime owns the reloadable transport."
//! Tape's peer TLS prefix binding.
//!
//! Generic PEM parsing, validation, and rustls builders are owned by
//! `peer-tls`; reloadable peer accept/connect behavior is owned by
//! `raft-runtime`. Tape deliberately owns only the externally visible
//! environment names and this thin conversion into the shared transport.

use anyhow::Result;

pub use peer_tls::PeerTlsConfig;

/// Prefix for Tape's peer TLS deployment contract.
pub const ENV_PREFIX: &str = "TAPE_PEER";

/// Load Tape's peer TLS material through the shared peer-TLS adapter.
pub fn from_env() -> Result<Option<PeerTlsConfig>> {
    PeerTlsConfig::from_env(ENV_PREFIX)
}

/// Construct the shared reloadable Raft peer transport.
pub fn peer_transport(config: &PeerTlsConfig) -> Result<raft_runtime::PeerTransport> {
    raft_runtime::PeerTransport::from_config(config)
}
// HANDWRITE-END

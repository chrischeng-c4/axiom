// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:f0da944f" tracker="#1704" reason="Tape owns only its TAPE_PEER environment prefix; raft-host owns validated peer TLS material and rustls configuration."
//! Tape's peer TLS prefix binding.
//!
//! Generic PEM parsing, validation, and rustls builders are owned by
//! `raft-host` for every Raft-hosted service. Tape deliberately owns only the
//! externally visible environment names. The Raft peer transport remains h2c
//! until its shared TLS acceptor/connector seam is implemented.

use anyhow::Result;

pub use peer_tls::PeerTlsConfig;

/// Prefix for Tape's peer TLS deployment contract.
pub const ENV_PREFIX: &str = "TAPE_PEER";

/// Load Tape's peer TLS material through the shared Raft-host adapter.
pub fn from_env() -> Result<Option<PeerTlsConfig>> {
    PeerTlsConfig::from_env(ENV_PREFIX)
}
// HANDWRITE-END

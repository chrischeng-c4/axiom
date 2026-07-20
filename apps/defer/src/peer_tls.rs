// HANDWRITE-BEGIN gap="missing-generator:logic:defer-peer-tls" tracker="#766" reason="Defer prefix adapter over shared peer TLS material and reloadable Raft transport."
use anyhow::Result;

pub use peer_tls::PeerTlsConfig;

pub const ENV_PREFIX: &str = "DEFER_PEER";

pub fn from_env() -> Result<Option<PeerTlsConfig>> {
    PeerTlsConfig::from_env(ENV_PREFIX)
}

pub fn peer_transport(config: &PeerTlsConfig) -> Result<raft_runtime::PeerTransport> {
    raft_runtime::PeerTransport::from_config(config)
}
// HANDWRITE-END

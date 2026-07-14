// HANDWRITE-BEGIN gap="missing-generator:logic:b147179b" tracker="#1704" reason="shared peer TLS adapter awaits a deterministic raft-host transport generator"
//! Peer TLS material for Raft-hosted services.
//!
//! This is the shared ownership boundary between a service-specific environment
//! prefix and the Raft peer transport. It deliberately delegates PEM loading
//! and rustls construction to `service-tls`; a service must not grow a second
//! copy of crypto configuration merely to participate in Raft.
//!
//! The current peer transport remains h2c. This adapter validates material and
//! exposes the future server/client configs, but does not claim TLS termination
//! until the h2c acceptor/connector seam exists.

use std::path::Path;

use anyhow::Result;

/// Validated peer TLS material for a Raft-hosted service.
#[derive(Debug, Clone)]
pub struct PeerTlsConfig(service_tls::PeerTlsConfig);

impl PeerTlsConfig {
    /// Load peer TLS material from `<prefix>_TLS_CERT`, `<prefix>_TLS_KEY`,
    /// `<prefix>_TLS_CA`, and `<prefix>_MTLS`.
    ///
    /// `None` means all material is intentionally absent. Partial, missing, or
    /// unusable material is a startup error rather than a plaintext fallback.
    pub fn from_env(prefix: &str) -> Result<Option<Self>> {
        service_tls::PeerTlsConfig::from_env(prefix).map(|config| config.map(Self))
    }

    /// PEM certificate-chain path.
    pub fn cert(&self) -> &Path {
        &self.0.cert
    }

    /// PEM private-key path.
    pub fn key(&self) -> &Path {
        &self.0.key
    }

    /// PEM CA-bundle path.
    pub fn ca(&self) -> &Path {
        &self.0.ca
    }

    /// Whether peer mTLS is requested once transport termination is wired.
    pub fn mtls_required(&self) -> bool {
        self.0.required
    }

    /// Build the server config future Raft peer acceptors will consume.
    pub fn rustls_server_config(&self) -> Result<rustls::ServerConfig> {
        self.0.rustls_server_config()
    }

    /// Build the client config future Raft peer dialers will consume.
    pub fn rustls_client_config(&self) -> Result<rustls::ClientConfig> {
        self.0.rustls_client_config()
    }
}
// HANDWRITE-END

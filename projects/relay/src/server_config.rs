// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#config
// HANDWRITE-BEGIN gap="missing-generator:config:04be064e" tracker="pending-tracker" reason="RelayServerConfig per the Config contract."
//! HTTP/2 transport configuration, embedding the relay core config.

use serde::{Deserialize, Serialize};

use crate::config::RelayCoreConfig;

/// Settings for the HTTP/2 (h2c) transport in front of the relay core.
///
/// @spec projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RelayServerConfig {
    /// h2c listen address for this shard.
    pub bind: String,
    /// HTTP/2 cleartext (TLS is terminated by the mesh / proxy, not here).
    pub h2c: bool,
    /// Total shards in the subject space (advertised for client-side sharding).
    pub shards: u32,
    /// Which shard this server instance serves.
    pub shard_index: u32,
    /// Embedded relay core engine settings.
    pub core: RelayCoreConfig,
}

impl Default for RelayServerConfig {
    fn default() -> Self {
        RelayServerConfig {
            bind: "0.0.0.0:7000".to_string(),
            h2c: true,
            shards: 1,
            shard_index: 0,
            core: RelayCoreConfig::default(),
        }
    }
}

impl RelayServerConfig {
    /// A config that binds an ephemeral local port with a RAM-only core —
    /// handy for tests and embedding.
    pub fn ephemeral() -> Self {
        RelayServerConfig {
            bind: "127.0.0.1:0".to_string(),
            core: RelayCoreConfig::in_memory(),
            ..RelayServerConfig::default()
        }
    }
}
// HANDWRITE-END

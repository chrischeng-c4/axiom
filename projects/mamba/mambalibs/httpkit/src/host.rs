//! Native host contract for httpkit.
//!
//! The host owns TCP, keepalive, HTTP/1.1, HTTP/2, shutdown, and backpressure.
//! It dispatches normalized protocol requests into `App`; it does not expose an
//! ASGI/WSGI server boundary.

use crate::protocol::{HttpProtocol, ProtocolLimits};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeepAliveConfig {
    pub enabled: bool,
    pub idle_timeout_ms: u64,
    pub max_requests_per_connection: Option<usize>,
}

impl Default for KeepAliveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout_ms: 75_000,
            max_requests_per_connection: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostConfig {
    pub bind_host: String,
    pub bind_port: u16,
    pub protocols: Vec<HttpProtocol>,
    pub keep_alive: KeepAliveConfig,
    pub limits: ProtocolLimits,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            bind_host: "0.0.0.0".to_string(),
            bind_port: 8000,
            protocols: vec![HttpProtocol::Http1, HttpProtocol::Http2],
            keep_alive: KeepAliveConfig::default(),
            limits: ProtocolLimits::default(),
        }
    }
}

impl HostConfig {
    pub fn supports(&self, protocol: HttpProtocol) -> bool {
        self.protocols.contains(&protocol)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub native_dispatch: bool,
    pub asgi_compatibility: bool,
    pub http1: bool,
    pub http2: bool,
    pub long_lived_connections: bool,
}

impl Default for HostCapabilities {
    fn default() -> Self {
        Self {
            native_dispatch: true,
            asgi_compatibility: false,
            http1: true,
            http2: true,
            long_lived_connections: true,
        }
    }
}

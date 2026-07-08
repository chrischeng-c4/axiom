use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Listener bind configuration shared by TCP and HTTP servers.
/// @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindConfig {
    pub host: IpAddr,
    pub port: u16,
}

impl BindConfig {
    /// Bind on all IPv4 interfaces.
    pub fn any(port: u16) -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port,
        }
    }

    /// Bind on localhost.
    pub fn localhost(port: u16) -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

impl Default for BindConfig {
    fn default() -> Self {
        Self::any(0)
    }
}

//! TCP client for cclab-ion KV store
//!
//! High-performance async client for connecting to kv-server.

mod protocol;
mod client;
mod pool;

pub use client::{ClientError, KvClient};
pub use pool::{KvPool, PoolConfig, PooledClient, PoolStats};
pub use protocol::{ProtocolError, Command, Status};

// Re-export KvValue for convenience
pub use crate::KvValue;

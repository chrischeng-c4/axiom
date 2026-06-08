//! TCP client for cclab-ion KV store
//!
//! High-performance async client for connecting to kv-server.

mod client;
mod pool;
mod protocol;

pub use client::{ClientError, KvClient};
pub use pool::{KvPool, PoolConfig, PoolStats, PooledClient};
pub use protocol::{Command, ProtocolError, Status};

// Re-export KvValue for convenience
pub use crate::KvValue;

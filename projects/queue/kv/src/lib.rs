//! High-performance, multi-core key-value store for cclab
//!
//! # Features
//! - Sharded storage engine for multi-core scalability
//! - High-precision numeric types (Decimal, f64, i64)
//! - Hybrid tiered storage (RAM + Disk)
//! - Compare-and-swap (CAS) for atomic state transitions
//! - Zero-copy serialization
//! - TCP server and client for remote access

// WIP: Suppress clippy warnings during development
#![allow(clippy::all)]

pub mod engine;
pub mod error;
pub mod metrics;
pub mod persistence;
pub mod types;

// TCP server and client
pub mod server;
pub mod client;


pub use cclab_core::{DataBridgeError, Result};
pub use engine::{EvictionPolicy, KvEngine};
pub use types::{KvKey, KvValue};
pub use error::KvError;

// Re-export server and client types
pub use server::{KvServer, WaiterManager};
pub use client::{KvClient, KvPool, PoolConfig, ClientError};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

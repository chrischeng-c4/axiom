//! Argus Daemon Server
//!
//! Provides a long-running daemon for code analysis with:
//! - In-memory code index
//! - File watching with incremental updates
//! - JSON-RPC over Unix socket

pub mod auto_discover;
pub mod daemon;
pub mod disk_cache;
pub mod handler;
pub mod incremental;
pub mod protocol;
pub mod watch_bridge;

#[cfg(test)]
mod tests;

pub use daemon::{ArgusDaemon, DaemonClient, DaemonConfig};
pub use handler::RequestHandler;
pub use incremental::{
    DependencyGraph, DirtyFileTracker, FileChangeKind, IncrementalUpdateManager,
};
pub use protocol::{
    CheckResult, DiagnosticInfo, IndexStatus, Request, Response, RpcError, SymbolInfo,
};
pub use watch_bridge::{
    spawn_watch_bridge, AsyncWatchBridgeBuilder, BridgeEvent, WatchBridge, WatchBridgeConfig,
    WatchBridgeHandle,
};

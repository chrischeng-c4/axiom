//! cclab-server - Unified HTTP server for cclab
//!
//! This crate provides a unified HTTP server for the dashboard + plan viewer UI.
//!
//! Note: MCP tools have been removed — SDD tool business logic is available via
//! CLI only (`score <subcommand>`). The MCP transport layer was redundant once
//! Claude Code could invoke the CLI directly via its Bash tool.
//!
//! Lens MCP tools were previously deregistered (lens-dissolution R7); SDD MCP
//! tools followed the same pattern.
//!
//! ## Architecture
//!
//! ```text
//! cc server start --port 3456
//!        │
//!        ▼
//! ┌──────────────────────────────────────────────┐
//! │         Unified HTTP Server (Axum)           │
//! ├──────────────────────────────────────────────┤
//! │  /              Dashboard (all projects)     │
//! │  /view/*        Plan Viewer UI               │
//! │  /health        Health check                 │
//! └──────────────────────────────────────────────┘
//!        │
//!        ▼
//! ┌──────────────────────────────────────────────┐
//! │     Registry (~/.cclab/registry.json)        │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Start server
//! cc server start --daemon
//!
//! # Register project
//! cc server register /path/to/project
//!
//! # List status
//! cc server list
//!
//! # Open viewer
//! cc server view myproject change-1
//!
//! # Shutdown
//! cc server shutdown
//! ```

pub mod cli;
pub mod http_server;
pub mod lens_pool;
pub mod registry;

pub use cli::{ensure_server_running, ServerCommands};
pub use http_server::{build_router, start_server, UnifiedAppState};
pub use lens_pool::LensHandlerPool;
pub use registry::Registry;

/// Result type alias for this crate
pub type Result<T> = anyhow::Result<T>;

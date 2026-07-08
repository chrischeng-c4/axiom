//! Shared server substrate for the Axiom runtime.
//! @spec projects/agentic-workflow/tech-design/logic/shared-server-substrate-performance-layers.md#logic
//!
//! This crate is intentionally below protocol crates. It owns the generic
//! lifecycle pieces that a raw TCP proxy, an HTTP dev server, and the
//! k8s-native service archetype all need: bind config, drain/readiness signals,
//! connection budgets, shutdown hooks, and metrics hooks. Protocol-specific
//! accept loops live above it in `tcp-server` and `http-server`.

pub mod config;
pub mod drain;
pub mod limits;
pub mod metrics;
pub mod signal;

pub use config::BindConfig;
pub use drain::{DrainController, DrainSignal, DrainState};
pub use limits::{ConnectionBudget, ConnectionLimitExceeded, ConnectionPermit};
pub use metrics::{ConnectionMetrics, NoopConnectionMetrics};
pub use signal::{shutdown_with_drain, wait_shutdown_signal};

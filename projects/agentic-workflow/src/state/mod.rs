// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/state/mod.md#source
// CODEGEN-BEGIN
//! STATE.yaml Management Module
//!
//! Handles persistence and tracking of change state, including:
//! - Phase transitions
//! - File checksums for staleness detection
//! - Validation history
//! - LLM telemetry

mod manager;

pub(crate) use manager::run_blocking_io;
pub use manager::{AgentLock, StalenessReport, StateManager};

// CODEGEN-END

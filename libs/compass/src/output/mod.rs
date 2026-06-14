//! Output formatters for analysis results.
//!
//! Post lens-dissolution location for output modules.
//! The `agent` submodule provides symbol-centric JSON output
//! optimized for LLM agent consumption.

pub mod agent;
pub mod agent_types;
pub mod reporter;

pub use agent::AgentOutputBuilder;
pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef};
pub use reporter::{OutputFormat, Reporter};

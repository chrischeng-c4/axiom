// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/mcp/mod.md#source
// CODEGEN-BEGIN
//! MCP Tool Definitions for SDD Generate
//!
//! Exposes diagram and spec generation as MCP tools.

mod handlers;
mod tools;

pub use handlers::*;
pub use tools::SddTools;

// CODEGEN-END

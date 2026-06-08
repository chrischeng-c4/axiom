// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/mod.md#source
// CODEGEN-BEGIN
//! State+ Machine Format
//!
//! XState-compatible state machine definitions with Mermaid stateDiagram-v2 output.
//! This module provides schema, validation, and generation for state machines.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/mod.md#source
// CODEGEN-BEGIN
//! Flowchart+ Diagram Format
//!
//! Enhanced flowchart definitions with semantic metadata, validation, and YAML frontmatter.
//! Supports semantic types for code generation (validation, db operations, API calls, etc.).

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

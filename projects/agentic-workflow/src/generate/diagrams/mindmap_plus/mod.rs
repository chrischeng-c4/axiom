// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/mod.md#source
// CODEGEN-BEGIN
//! Mindmap+ Diagram Format
//!
//! Enhanced mindmap definitions with validation and YAML frontmatter.
//! Supports hierarchical nodes with shapes and icons.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

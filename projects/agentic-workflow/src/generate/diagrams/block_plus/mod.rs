// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/mod.md#source
// CODEGEN-BEGIN
//! Block+ Diagram Format
//!
//! Mermaid block-beta diagrams with column layout, nested blocks, edges,
//! shapes, and YAML frontmatter validation.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

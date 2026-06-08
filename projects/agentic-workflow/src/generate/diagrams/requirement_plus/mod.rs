// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/mod.md#source
// CODEGEN-BEGIN
//! Requirement+ Diagram Format
//!
//! Enhanced requirement diagram definitions with validation and YAML frontmatter.
//! Supports requirements, design elements, and traceability relationships.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/mod.md#source
// CODEGEN-BEGIN
//! Class+ Diagram Format
//!
//! Enhanced class diagram definitions with validation and YAML frontmatter.
//! Supports classes, attributes, methods, relationships, and namespaces.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

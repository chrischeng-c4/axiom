// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/mod.md#source
// CODEGEN-BEGIN
//! ERD+ Diagram Format
//!
//! Enhanced Entity-Relationship diagram definitions with validation and YAML frontmatter.
//! Supports entities, attributes (PK/FK/UK), and relationships with cardinality.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

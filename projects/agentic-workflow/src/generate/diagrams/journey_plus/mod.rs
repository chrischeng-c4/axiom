// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/journey_plus/mod.md#source
// CODEGEN-BEGIN
//! Journey+ Diagram Format
//!
//! Enhanced user journey diagram definitions with validation and YAML frontmatter.
//! Supports sections, tasks with scores, and actors.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

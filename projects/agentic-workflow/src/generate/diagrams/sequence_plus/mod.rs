// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/mod.md#source
// CODEGEN-BEGIN
//! Sequence+ Diagram Format
//!
//! Enhanced sequence diagram definitions with validation and YAML frontmatter.
//! Supports participants, messages, loops, alt/opt blocks, and activation.

mod generator;
mod schema;
mod validator;

pub use generator::*;
pub use schema::*;
pub use validator::*;

// CODEGEN-END

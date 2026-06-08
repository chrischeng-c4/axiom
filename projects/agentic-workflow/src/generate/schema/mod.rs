// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/schema/mod.md#source
// CODEGEN-BEGIN
//! JSON Schema Core Implementation
//!
//! Provides strongly-typed structures for JSON Schema Draft 7 and Draft 2020-12.

mod parser;
mod types;

pub use parser::*;
pub use types::*;

// CODEGEN-END

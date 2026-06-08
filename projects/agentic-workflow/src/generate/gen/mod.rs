// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/mod.md#source
// CODEGEN-BEGIN
//! Code generators for all target languages.
//!
//! Currently implements: Rust (structural + behavioral).
//! Python and TypeScript translators share the same `AbstractType` enum
//! but have deferred implementations.

pub mod operations;
pub mod python;
pub mod rust;

pub use rust::*;

// CODEGEN-END

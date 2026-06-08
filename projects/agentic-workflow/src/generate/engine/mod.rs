// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/mod.md#source
// CODEGEN-BEGIN
//! Template Engine Module
//!
//! Provides Tera-based template rendering with custom filters.

mod error;
mod filters;
mod tera_engine;

pub use error::TemplateError;
pub use tera_engine::TemplateEngine;
// CODEGEN-END

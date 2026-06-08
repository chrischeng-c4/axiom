// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/validator/mod.md#source
// CODEGEN-BEGIN
//! Spec Completeness Validator
//!
//! Validates JSON Schemas and SpecIR payloads for completeness before code generation.
//!
//! ## Modules
//!
//! - [`completeness`] — JSON Schema type/ref/description validation (R1–R3)
//! - [`spec_ir_validator`] — SpecIR section-type validators (deploy, wireframe,
//!   component, design-token) with shared registration mechanism

mod completeness;
mod spec_ir_validator;

pub use completeness::{validate_schema, Severity, ValidationIssue, ValidationResult};
pub use spec_ir_validator::{
    validate_spec_ir, ComponentValidator, DeployValidator, DesignTokenValidator, SpecIRValidator,
    WireframeValidator,
};

// CODEGEN-END

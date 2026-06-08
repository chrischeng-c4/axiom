// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/mod.md#source
// CODEGEN-BEGIN
//! Spec alignment checking.
//!
//! Validates spec files for format compliance and logical consistency.
//! Two-layer validation:
//! - Format compliance: section annotations, duplicates, code block requirements
//! - Logical consistency: duplicate definitions, schema conflicts, field near-matches
//!
//! Entry point: `spec_alignment::check(path)`.

pub mod annotations;
pub mod check;
pub mod coverage;
pub mod format_rules;
pub mod logical_rules;
pub mod models;
pub mod parser;
pub mod requirement_coverage;
pub mod schema_struct;

pub use check::{check, check_with_coverage};
pub use models::{
    CheckResult, CodeBlock, CoverageEntry, CoverageReport, FileResult, OrphanRequirementEntry,
    SchemaStructMismatchEntry, SpecAnnotation, SpecDocument, SpecSection, UnspeccedFunction,
    Violation, ViolationKind,
};
// CODEGEN-END

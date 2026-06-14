// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/mod_types.md#source
// CODEGEN-BEGIN
//! Diagram and Specification Generation Library
//!
//! Provides pure Rust functions for generating:
//! - Mermaid diagrams (flowchart, sequence, class, state, ERD, mindmap, requirement, journey)
//! - API specifications (OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, Serverless Workflow 0.8)
//! - Code from JSON Schema (FastAPI, Express, Axum)
//!
//! Also exposes MCP tools for direct integration with cclab-server.

pub mod diagrams;
pub mod engine;
pub mod generators;
pub mod mcp;
pub mod patterns;
pub mod schema;
pub mod spec_ir;
pub mod specs;
pub mod validator;

// TD→code codegen modules (new)
pub mod apply;
pub mod audit;
pub mod diff;
pub mod frontmatter;
pub mod gen;
pub mod marker;
pub mod render;
pub mod types;
// Stage 2: TDAst-driven generator dispatch.
// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md
pub mod from_td_ast;
// HANDWRITE marker tooling — schema (codegen'd), scaffold inserter,
// and parser live in audit.rs; re-exported below.
// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
pub mod handwrite;
pub mod handwrite_scaffold;
// rust-source-unit: lossless-CST parse to a structured item-tree with
// byte-identical emit — the td_ast codegen primitive for arbitrary Rust units.
// @spec projects/agentic-workflow/tech-design/validate/rust-source-unit-ir-lossless-cst-parse-to-structured-item-tree-b.md#logic
pub mod rust_source_unit;

#[cfg(test)]
#[path = "tests/handwrite_scaffold_test.rs"]
mod handwrite_scaffold_test;

pub use diagrams::*;
pub use engine::{TemplateEngine, TemplateError};
pub use generators::{
    AxumGenerator,
    CoverageIssue,
    DeployGenerator,
    ExpressGenerator,
    FastAPIGenerator,
    FlowchartPlusGenerator,
    Generator,
    GeneratorError,
    GeneratorSettings,
    Manifest,
    ReactGenerator,
    SequencePlusGenerator,
    // SpecIR-based generators
    SpecIRGenerator,
    StateMachineGenerator,
    TestGenError,
    TestGenResult,
    TestGenerator,
};
pub use mcp::{call_tool, is_sdd_tool, SddTools};
pub use schema::{JsonSchema, SchemaType, SchemaVersion};
pub use spec_ir::{
    AttributeDef,
    BundleMetadata,
    ComponentSpec,
    // New spec payload types (deploy / wireframe / component / design-token section types)
    DeploySpec,
    DesignTokenEntry,
    DesignTokenSpec,
    EnvVar,
    EventDef,
    PropDef,
    ResourceLimits,
    SlotDef,
    SpecBundle,
    SpecIR,
    SpecMetadata,
    WireframeNode,
    WireframeSpec,
};
pub use specs::*;
pub use validator::{
    validate_schema,
    // SpecIR validators (deploy, wireframe, component, design-token section types)
    validate_spec_ir,
    ComponentValidator,
    DeployValidator,
    DesignTokenValidator,
    Severity,
    SpecIRValidator,
    ValidationIssue,
    ValidationResult,
    WireframeValidator,
};

// HANDWRITE marker public surface — types + scaffold + parser.
// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
pub use audit::{parse_handwrite_markers, HandwriteParseFailure};
pub use handwrite::{CoverageReport, HandwriteEntry, HandwriteMarker, HandwriteParseError};
pub use handwrite_scaffold::{scaffold_handwrite, ScaffoldOutcome, PENDING_TRACKER};

/// Result type for generate operations
/// @spec projects/agentic-workflow/tech-design/core/generate/mod_types.md#source
pub type Result<T> = std::result::Result<T, GenerateError>;

/// Diagram and spec generation errors.
/// @spec projects/agentic-workflow/tech-design/core/generate/mod_types.md#schema
#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    UnsupportedLanguage(String),
}

/// @spec projects/agentic-workflow/tech-design/core/generate/mod_types.md#source
impl From<serde_json::Error> for GenerateError {
    fn from(e: serde_json::Error) -> Self {
        GenerateError::Serialization(e.to_string())
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/mod_types.md#source
impl From<serde_yaml::Error> for GenerateError {
    fn from(e: serde_yaml::Error) -> Self {
        GenerateError::Serialization(e.to_string())
    }
}

// CODEGEN-END

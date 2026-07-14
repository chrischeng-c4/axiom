---
id: sdd-generate-mod-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
  - id: td-cb-lifecycle-automation
    role: primary
    gap: ambiguous-multi-target-generation-preflight
    claim: ambiguous-multi-target-generation-preflight
    coverage: full
    rationale: "GenerateError carries stable typed ambiguity and unavailable-plan details from pre-lifecycle admission to the public TD error envelope."
---

# GenerateError

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/generate/mod.rs` generated from AST during Score force-regeneration standardization.

Issue #1633 adds typed generation-plan admission errors. Ambiguity reports the
shared section, sorted eligible targets, remediation, and exact next command;
unavailable ownership reports the spec path and the command that can be rerun
after authoring a typed Changes plan.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GenerateError` | apps/agentic-workflow/src/generate/mod.rs | enum | pub | 115 |  |
| `Result` | apps/agentic-workflow/src/generate/mod.rs | type | pub | 110 |  |
| `apply` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 23 |  |
| `audit` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 24 |  |
| `diagrams` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 12 |  |
| `diff` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 25 |  |
| `engine` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 13 |  |
| `from_td_ast` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 33 |  |
| `frontmatter` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 26 |  |
| `gen` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 27 |  |
| `generators` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 14 |  |
| `handwrite` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 37 |  |
| `handwrite_scaffold` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 38 |  |
| `marker` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 28 |  |
| `mcp` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 15 |  |
| `patterns` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 16 |  |
| `render` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 29 |  |
| `rust_source_unit` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 42 |  |
| `schema` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 17 |  |
| `spec_ir` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 18 |  |
| `specs` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 19 |  |
| `types` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 30 |  |
| `validator` | apps/agentic-workflow/src/generate/mod.rs | module | pub | 20 |  |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  GenerateError:
    type: object
    description: Diagram and spec generation errors.
    x-rust-enum:
      derive: [Debug, "thiserror::Error"]
      variants:
        - name: MissingField
          kind: tuple
          error: "Missing required field: {0}"
          fields: [{ rust_type: String }]
        - name: InvalidValue
          kind: tuple
          error: "Invalid value: {0}"
          fields: [{ rust_type: String }]
        - name: AmbiguousGenerationPlan
          kind: struct
          error: "ambiguous multi-target generation plan for section `{section}`: eligible CODEGEN targets {targets:?}; {remediation}"
          fields:
            - { name: section, rust_type: String }
            - { name: targets, rust_type: "Vec<String>" }
            - { name: remediation, rust_type: String }
            - { name: next_command, rust_type: String }
        - name: GenerationPlanUnavailable
          kind: struct
          error: "generation plan unavailable for `{spec_path}` before lifecycle mutation; {remediation}"
          fields:
            - { name: spec_path, rust_type: String }
            - { name: remediation, rust_type: String }
            - { name: next_command, rust_type: String }
        - name: Serialization
          kind: tuple
          error: "Serialization error: {0}"
          fields: [{ rust_type: String }]
        - name: Io
          kind: tuple
          error: "IO error: {0}"
          fields: [{ rust_type: "std::io::Error", error_from: true }]
        - name: UnsupportedLanguage
          kind: tuple
          error: "{0}"
          fields: [{ rust_type: String }]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=apps/agentic-workflow/src/generate/mod.rs -->
```rust
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
// @spec apps/agentic-workflow/tech-design/core/generate/from-td-ast.md
pub mod from_td_ast;
// HANDWRITE marker tooling — schema (codegen'd), scaffold inserter,
// and parser live in audit.rs; re-exported below.
// @spec apps/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
pub mod handwrite;
pub mod handwrite_scaffold;
// rust-source-unit: lossless-CST parse to a structured item-tree with
// byte-identical emit — the td_ast codegen primitive for arbitrary Rust units.
// @spec apps/agentic-workflow/tech-design/validate/rust-source-unit-ir-lossless-cst-parse-to-structured-item-tree-b.md#logic
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
// @spec apps/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
pub use audit::{parse_handwrite_markers, HandwriteParseFailure};
pub use handwrite::{CoverageReport, HandwriteEntry, HandwriteMarker, HandwriteParseError};
pub use handwrite_scaffold::{scaffold_handwrite, ScaffoldOutcome, PENDING_TRACKER};

/// Result type for generate operations
/// @spec apps/agentic-workflow/tech-design/core/generate/mod_types.md#source
pub type Result<T> = std::result::Result<T, GenerateError>;

/// Diagram and spec generation errors.
/// @spec apps/agentic-workflow/tech-design/core/generate/mod_types.md#schema
#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error(
        "ambiguous multi-target generation plan for section `{section}`: eligible CODEGEN targets {targets:?}; {remediation}"
    )]
    AmbiguousGenerationPlan {
        section: String,
        targets: Vec<String>,
        remediation: String,
        next_command: String,
    },
    #[error(
        "generation plan unavailable for `{spec_path}` before lifecycle mutation; {remediation}"
    )]
    GenerationPlanUnavailable {
        spec_path: String,
        remediation: String,
        next_command: String,
    },
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    UnsupportedLanguage(String),
}

/// @spec apps/agentic-workflow/tech-design/core/generate/mod_types.md#source
impl From<serde_json::Error> for GenerateError {
    fn from(e: serde_json::Error) -> Self {
        GenerateError::Serialization(e.to_string())
    }
}

/// @spec apps/agentic-workflow/tech-design/core/generate/mod_types.md#source
impl From<serde_yaml::Error> for GenerateError {
    fn from(e: serde_yaml::Error) -> Self {
        GenerateError::Serialization(e.to_string())
    }
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/generate/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete generate module facade.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.

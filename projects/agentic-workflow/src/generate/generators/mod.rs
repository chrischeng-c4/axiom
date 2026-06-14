// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/module_registry.md#source
// CODEGEN-BEGIN
//! Code Generators
//!
//! Framework-specific code generators using the template engine.
//!
//! ## Generator families
//!
//! ### JSON Schema / OpenAPI generators ([`Generator`] trait)
//! - [`FastAPIGenerator`] — Python / FastAPI
//! - [`ExpressGenerator`] — TypeScript / Express
//! - [`AxumGenerator`]    — Rust / Axum
//!
//! ### SpecIR generators ([`SpecIRGenerator`] trait)
//! - [`DeployGenerator`]           — `deploy` section type → Kubernetes Deployment + Service YAML
//! - [`ReactGenerator`]            — `wireframe` section type → React functional component scaffold
//! - [`StateMachineGenerator`]     — `state-machine` section type → Python Enum + transition function
//! - [`FlowchartPlusGenerator`]    — `logic` (flowchart) section type → Python function skeletons
//! - [`SequencePlusGenerator`]     — `interaction` (sequence) section type → Python async call chain

mod axum;
pub mod cli_subcommand;
mod common;
mod deploy;
mod express;
mod fastapi;
mod flowchart_plus_gen;
pub mod logic_primitive_emitter;
pub mod module_facade;
pub mod primitive_registry;
pub mod quality_primitives;
mod react;
mod sequence_plus_gen;
mod state_machine_gen;
mod test_generator;
pub mod trait_impl;

pub use axum::AxumGenerator;
pub use cli_subcommand::{emit_cli_subcommand, CliArg, CliArgKind, CliCommand, CliEmitted};
pub use common::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
    SpecIRGenerator,
};
pub use deploy::DeployGenerator;
pub use express::ExpressGenerator;
pub use fastapi::FastAPIGenerator;
pub use flowchart_plus_gen::FlowchartPlusGenerator;
pub use logic_primitive_emitter::{emit_flowchart, LogicPrimitiveEmitter};
pub use module_facade::{run_module_facade, ExportEntry, ModuleFacadeOutput, ModuleFacadeSpec};
pub use primitive_registry::{
    is_prose_section, kind_to_name as primitive_kind_to_name, lookup as lookup_primitive,
    PrimitiveEntry, REGISTRY as PRIMITIVE_REGISTRY,
};
pub use quality_primitives::{
    default_quality_primitive_profiles, evaluate_primitive_review_checks,
    explain_primitive_selection, find_quality_primitive_profile,
    validate_quality_primitive_profiles, PrimitiveDialCompatibility, PrimitiveDialSupport,
    PrimitiveEvidenceExample, PrimitiveEvidenceKind, PrimitiveReviewCheck, PrimitiveReviewFinding,
    PrimitiveReviewSeverity, PrimitiveSelectionCitation, PrimitiveSelectionRequest,
    QualityPrimitiveProfile,
};
pub use react::{render_react_wireframe_file, ReactGenerator};
pub use sequence_plus_gen::SequencePlusGenerator;
pub use state_machine_gen::StateMachineGenerator;
pub use test_generator::{CoverageIssue, TestGenError, TestGenResult, TestGenerator};
pub use trait_impl::{run_trait_impl, MatchArm, TraitImplOutput, TraitImplSpec, TraitMethod};
// CODEGEN-END

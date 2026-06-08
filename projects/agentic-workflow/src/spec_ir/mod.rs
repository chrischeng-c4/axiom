// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_ir/mod.md#source
// CODEGEN-BEGIN
//! SpecIR YAML Manifest types (k8s/Kustomize style)
//!
//! Language-agnostic intermediate representation for the spec-to-code pipeline.
//! SDD writes these YAML files, Lens reads them for codegen.
//!
//! ## Manifest format
//!
//! ```yaml
//! apiVersion: cclab.dev/v1
//! kind: Api
//! metadata:
//!   name: user-service
//!   change_id: genesis-372
//! spec:
//!   # kind-specific payload
//! ```

pub mod codegen;
pub mod generator;
pub mod migration;
pub mod orchestrator;
mod types;

pub use types::*;

// CODEGEN-END

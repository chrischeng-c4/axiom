// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/spec_ir/mod.md#source
// CODEGEN-BEGIN
//! SpecIR — Specification Intermediate Representation
//!
//! The universal contract between SDD generate (spec format) and Lens (code generation).
//! SpecIR wraps diagram and schema types into a unified enum that
//! generators can consume via `can_generate()` / `generate_from_ir()`.
//!
//! ## Variants
//!
//! | Variant | Section type | Generator |
//! |---------|-------------|-----------|
//! | `Api` | `rest-api` / `schema` | `FastAPIGenerator`, `ExpressGenerator`, `AxumGenerator` |
//! | `FlowchartPlus` | `logic` (flowchart) | — |
//! | `ClassPlus` | `logic` (class) | — |
//! | `ErdPlus` | `db-model` | — |
//! | `SequencePlus` | `interaction` | — |
//! | `RequirementPlus` | `unit-test` | `TestGenerator` |
//! | `Deploy` | `deploy` | `DeployGenerator` |
//! | `Wireframe` | `wireframe` | `ReactGenerator` |
//! | `Component` | `component` | — (future) |
//! | `DesignToken` | `design-token` | — (future) |

mod types;

pub use types::*;

// CODEGEN-END

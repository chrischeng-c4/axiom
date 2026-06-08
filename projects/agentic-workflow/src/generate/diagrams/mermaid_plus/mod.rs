// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/mod.md#source
// CODEGEN-BEGIN
//! Mermaid+ State Machine Format
//!
//! XState-compatible state machine definitions with Mermaid stateDiagram-v2 output.
//! This module provides the core types and generator, independent of Lens IR.

mod generator;
pub mod migrate;
mod schema;
mod validator;

pub use generator::*;
pub use migrate::{
    apply_block_payload, detect_diagram_kind, enumerate_envelopes, mermaid_equivalent,
    run_migration, DiagramKind, MigrateState, MigrationEnvelope, MigrationOptions,
    MIGRATE_TOOL_VERSION, PAYLOAD_DIR,
};
pub use schema::*;
pub use validator::*;

// CODEGEN-END

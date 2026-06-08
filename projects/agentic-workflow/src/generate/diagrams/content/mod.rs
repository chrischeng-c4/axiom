// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/content/mod.md#source
// CODEGEN-BEGIN
//! Per-diagram Content types for Mermaid Plus codegen.
//!
//! Each diagram type has its own explicit Content struct (design decision D3).
//! No universal `Graph<N,E>` — each type is statically typed and XState-free (D8).

pub mod interaction;
pub mod logic;
pub mod requirement;
pub mod state_machine;

pub use interaction::InteractionContent;
pub use logic::LogicContent;
pub use requirement::RequirementContent;
pub use state_machine::StateMachineContent;

// CODEGEN-END

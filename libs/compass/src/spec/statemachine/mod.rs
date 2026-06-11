//! State machine definition parsing, validation, and Mermaid+ generation
//!
//! Flow:
//! 1. LLM generates structured JSON (state machine definition)
//! 2. Lens validates the JSON semantically
//! 3. Lens outputs Mermaid+ (YAML frontmatter + Mermaid diagram)
//!
//! The JSON schema is designed for:
//! - Easy generation by LLM
//! - Easy validation by code
//! - Conversion to Mermaid stateDiagram-v2

mod mermaid_plus;
mod schema;
mod validator;

pub use mermaid_plus::{MermaidPlusGenerator, MermaidPlusOutput};
pub use schema::{
    ActionDef, ActionRef, GuardDef, StateMachineDef, StateNodeDef, TransitionDetail,
    TransitionInput,
};
pub use validator::{Severity, StateMachineValidator, ValidationError, ValidationResult};

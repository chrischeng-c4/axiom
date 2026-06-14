//! Spec parsing module for spec-to-code generation
//!
//! This module contains parsers for various specification formats:
//! - JSON Schema
//! - OpenAPI 3.x
//! - AsyncAPI 2.x
//! - Mermaid diagrams (classDiagram, sequenceDiagram, stateDiagram, flowchart, ERD)
//! - State machine definitions (with Mermaid+ output)

pub mod asyncapi;
pub mod ir;
pub mod json_schema;
pub mod mermaid;
pub mod openapi;
pub mod statemachine;

pub use ir::*;
pub use statemachine::{
    MermaidPlusGenerator, MermaidPlusOutput, StateMachineDef, StateMachineValidator,
    ValidationError, ValidationResult,
};

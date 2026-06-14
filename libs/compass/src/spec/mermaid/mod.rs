//! Mermaid diagram parser and generator
//!
//! Supports parsing and generating:
//! - classDiagram - Class structures and relationships
//! - sequenceDiagram - Interaction flows
//! - stateDiagram - State machines
//! - flowchart - Control flow logic
//! - erDiagram - Entity-Relationship diagrams

pub mod generator;
pub mod parser;

pub use generator::MermaidGenerator;
pub use parser::MermaidParser;

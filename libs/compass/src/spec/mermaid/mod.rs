// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-spec-mermaid-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END

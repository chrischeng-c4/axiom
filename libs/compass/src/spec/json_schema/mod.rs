// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-spec-json-schema-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! JSON Schema parser
//!
//! Parses JSON Schema (draft-07, draft-2020-12) into SpecIR.

mod parser;

pub use parser::*;
// CODEGEN-END

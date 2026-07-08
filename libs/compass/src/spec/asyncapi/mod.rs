// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-spec-asyncapi-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! AsyncAPI specification parser
//!
//! Parses AsyncAPI 2.x and 3.x specifications into EventApiSpec IR.

pub mod parser;

pub use parser::AsyncApiParser;
// CODEGEN-END

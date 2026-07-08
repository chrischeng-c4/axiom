// SPEC-MANAGED: libs/compass/tech-design/semantic/source/libs-compass-src-syntax-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! AST parsing with tree-sitter

mod parser;

pub use parser::{Language, MultiParser, ParseError, ParsedFile};
// CODEGEN-END

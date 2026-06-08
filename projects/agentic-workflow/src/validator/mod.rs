// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/validator/mod.md#source
// CODEGEN-BEGIN
pub mod challenge;
pub mod consistency;
pub mod fix;
pub mod format;
pub mod schema;
pub mod semantic;

pub use challenge::ChallengeValidator;
pub use consistency::ConsistencyValidator;
pub use fix::{AutoFixer, FixResult};
pub use format::SpecFormatValidator;
pub use schema::{
    validate_frontmatter_content, validate_frontmatter_schema, DocumentType, SchemaValidator,
};
pub use semantic::SemanticValidator;

// CODEGEN-END

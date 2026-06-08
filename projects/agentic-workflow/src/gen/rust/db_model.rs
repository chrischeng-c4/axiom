// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/db_model_types.md#schema
// CODEGEN-BEGIN
/// Output from DB-model code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/db_model_types.md#schema
#[derive(Debug, Clone)]
pub struct DbModelGenOutput {
    /// The generated Rust struct(s) with sqlx derives.
    pub code: String,
}
// CODEGEN-END

//! Unified TD AST — `parse_td(path)` produces a typed view of a TD spec file
//! that downstream tooling (entity graph, drift audit, hashing, codegen
//! round-trip) consumes via [`TDAst`] + [`SectionEntities`].
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#schema
//! @spec projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#logic-parse_td

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/td_ast/types.md#exports
// CODEGEN-BEGIN
pub mod entities;
pub use entities::EntityRef;
pub use entities::SectionEntities;
pub mod parse;
pub use parse::parse_td;
pub use parse::parse_td_str;
pub mod types;
pub use types::MermaidPlusPayload;
pub use types::SectionKind;
pub use types::TDAst;
pub use types::TDSection;
pub use types::TdParseError;
pub use types::TypedBody;
pub mod query;
pub use query::Ref;
pub use query::RefKind;
pub use query::TypeDef;
pub mod validate;
pub use validate::validate_td;
pub use validate::validate_td_full;
pub use validate::TdError;
pub use validate::TdErrorCode;
pub mod payloads;
pub use payloads::AsyncApiChannel;
pub use payloads::AsyncApiPayload;
pub use payloads::CliArgDef;
pub use payloads::CliCommandDef;
pub use payloads::CliManifestPayload;
pub use payloads::ConfigKeyDef;
pub use payloads::ConfigManifestPayload;
pub use payloads::JsonSchemaPayload;
pub use payloads::OpenApiOperation;
pub use payloads::OpenApiPathItem;
pub use payloads::OpenApiPayload;
pub use payloads::OpenRpcPayload;
pub use payloads::PayloadTypeDef;
pub use payloads::RpcMethod;
pub use payloads::RpcParam;
pub use payloads::TdParseErrorKind;
pub mod anti_patterns;
pub use anti_patterns::check_content_anti_patterns;
pub use anti_patterns::check_filesystem_anti_patterns;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-td-ast.md#schema
// CODEGEN-BEGIN
pub mod mermaid_plus;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
pub fn enter() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-split_fence
    // TODO: Implement process step: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]
    todo!("process: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]");
    // Decision: Frontmatter markers found?
    if todo!("decision: Frontmatter markers found?")
    /* no */
    {
        todo!("terminal: Err(MP-FM-001 missing-frontmatter)");
    } else
    /* yes */
    {
        // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-parse_yaml
        // TODO: Implement process step: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)
        todo!("process: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)");
        // Decision: YAML parses into typed frontmatter?
        if todo!("decision: YAML parses into typed frontmatter?")
        /* no */
        {
            todo!("terminal: Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)");
        } else
        /* yes */
        {
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-derive_id
            // TODO: Implement process step: id = frontmatter.id().unwrap_or(format!("{section_type}:{line_start}"))
            todo!("process: id = frontmatter.id().unwrap_or(format!(\"{{section_type}}:{{line_start}}\"))");
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-build_block
            // TODO: Implement process step: Construct MermaidPlusBlockTyped { id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }
            todo!("process: Construct MermaidPlusBlockTyped {{ id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }}");
            todo!("terminal: Ok(MermaidPlusBlockTyped) — parse layer done");
        }
    }
    // Terminal: err_no_fence -> Err(MP-FM-001 missing-frontmatter)
    // Terminal: err_yaml -> Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)
    // Terminal: lower_unsupp -> (None, [info: MP-LO-999 unsupported-ir-family]) — mindmap / dependency carry no IR yet
    // Terminal: ret_block -> Ok(MermaidPlusBlockTyped) — parse layer done
    // Terminal: ret_ir -> (Some(IRFamily), diagnostics)
}
// CODEGEN-END

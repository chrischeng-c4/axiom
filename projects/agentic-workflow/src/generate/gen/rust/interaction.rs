// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/interaction.md#source
// CODEGEN-BEGIN

//! Interaction/sequence behavioral generator.
//!
//! Reads `InteractionContent` parsed from Mermaid Plus frontmatter and generates:
//! - Method signatures for each message (`async fn name(&self, ...) -> Result<T>`)
//! - Call-graph annotations: `// CALL: from -> to` comments
//! - SPEC-REF markers for method bodies (30% coverage — bodies require human/LLM authorship)
//!
//! All output lives inside CODEGEN-BEGIN/END markers.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R2

use crate::generate::diagrams::content::interaction::InteractionContent;
use crate::generate::marker::{emit_spec_ref, Lang};
use crate::generate::types::RustConfig;

/// Output from interaction code generation.
#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/interaction.md#source
pub struct InteractionGenOutput {
    /// The generated Rust code (trait or impl block with method signatures).
    pub code: String,
    /// SPEC-REF entries emitted inside the generated code.
    pub spec_refs: Vec<String>,
}

/// Generate Rust method signatures from an `InteractionContent`.
///
/// Each message in the interaction diagram becomes an `async fn` with:
/// - Parameter list from message metadata
/// - Call-graph annotation comment
/// - SPEC-REF body marker for the implementation
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R2
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R4
pub fn generate_interaction(
    content: &InteractionContent,
    spec_path: &str,
    config: &RustConfig,
) -> InteractionGenOutput {
    let vis = config.vis_prefix();

    let mut spec_refs = Vec::new();
    let mut lines = Vec::new();

    // Emit one method per message
    for msg in &content.messages {
        let method_name = msg.name.replace('-', "_").replace(' ', "_");
        let return_type = msg.returns.as_deref().unwrap_or("()");

        // Call-graph annotation
        lines.push(format!("// CALL: {} -> {}", msg.from, msg.to));

        // Method signature
        let async_kw = if msg.r#async { "async " } else { "" };
        lines.push(format!(
            "{}{}fn {}(&self) -> Result<{}> {{",
            vis, async_kw, method_name, return_type
        ));

        // SPEC-REF body marker
        let section_id = format!("{}-{}", content.id, method_name.replace('_', "-"));
        let marker = emit_spec_ref(
            spec_path,
            &section_id,
            &format!("Implement {} from {} to {}", msg.name, msg.from, msg.to),
            Lang::Rust,
        );
        for marker_line in marker.lines() {
            lines.push(format!("    {}", marker_line));
        }
        spec_refs.push(format!("{}#{}", spec_path, section_id));

        lines.push("    todo!()".to_string());
        lines.push("}".to_string());
        lines.push(String::new());
    }

    InteractionGenOutput {
        code: lines.join("\n").trim_end().to_string(),
        spec_refs,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::content::interaction::{Actor, ActorKind, Message};

    fn make_interaction() -> InteractionContent {
        InteractionContent {
            id: "create-issue-flow".to_string(),
            title: None,
            actors: vec![
                Actor {
                    id: "Client".to_string(),
                    kind: ActorKind::Actor,
                    label: None,
                },
                Actor {
                    id: "Server".to_string(),
                    kind: ActorKind::System,
                    label: None,
                },
            ],
            messages: vec![
                Message {
                    from: "Client".to_string(),
                    to: "Server".to_string(),
                    name: "create_issue".to_string(),
                    r#async: false,
                    returns: None,
                    label: None,
                },
                Message {
                    from: "Server".to_string(),
                    to: "Client".to_string(),
                    name: "issue_created".to_string(),
                    r#async: false,
                    returns: Some("Issue".to_string()),
                    label: None,
                },
            ],
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R2
    #[test]
    fn test_generates_method_signatures_for_messages() {
        let interaction = make_interaction();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_interaction(&interaction, "spec.md", &config);

        // Scenario S2: method signatures for each message
        assert!(
            output.code.contains("fn create_issue"),
            "Should have create_issue method"
        );
        assert!(
            output.code.contains("fn issue_created"),
            "Should have issue_created method"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R2
    #[test]
    fn test_generates_call_graph_annotations() {
        let interaction = make_interaction();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_interaction(&interaction, "spec.md", &config);

        assert!(
            output.code.contains("// CALL: Client -> Server"),
            "Should have CALL annotation for create_issue"
        );
        assert!(
            output.code.contains("// CALL: Server -> Client"),
            "Should have CALL annotation for issue_created"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R4
    #[test]
    fn test_generates_spec_ref_body_markers() {
        let interaction = make_interaction();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_interaction(&interaction, "spec.md", &config);

        // Scenario S2: SPEC-REF body markers for implementation
        assert!(
            output.code.contains("SPEC-REF"),
            "Should have SPEC-REF markers"
        );
        assert!(
            !output.spec_refs.is_empty(),
            "Should emit spec_refs for all messages"
        );
        assert_eq!(output.spec_refs.len(), 2, "One spec_ref per message");
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-behavioral.md#R2
    #[test]
    fn test_generates_return_type_from_message() {
        let interaction = make_interaction();
        let config = crate::generate::types::RustConfig::default();
        let output = generate_interaction(&interaction, "spec.md", &config);

        // issue_created returns Issue
        assert!(
            output.code.contains("Result<Issue>"),
            "issue_created should return Result<Issue>"
        );
    }
}

// CODEGEN-END

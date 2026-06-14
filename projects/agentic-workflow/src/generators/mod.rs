// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/mod_preamble_source.md#source
// CODEGEN-BEGIN
//! CLI generator infrastructure for change spec sections.
//!
//! Each section type has a dedicated generator that produces the initial
//! section content skeleton. Generators are invoked from the CLI with:
//!
//! ```text
//! score spec gen --type overview --sdd-id my-change [--sdd-refs ref1,ref2]
//! ```
//!
//! ## Design
//!
//! - `Generator` trait: single `generate()` method returning section content
//! - `GeneratorArgs`: shared CLI flags (`--type`, `--sdd-id`, `--sdd-refs`)
//! - Annotation injection: generators automatically prepend
//!   `<!-- type: <type> lang: <lang> -->` to their output
//! - Most section types have generators registered in `get_generator()`

// Core prose sections
pub mod changes;
pub mod overview;

// Behavioral sections
pub mod doc;
pub mod requirements;
pub mod scenarios;

// Mermaid diagram sections
pub mod db_model;
pub mod dependency;
pub mod flowchart;
pub mod mindmap;
pub mod sequence;
pub mod state_machine;
pub mod test_plan;

// API spec sections
pub mod async_api;
pub mod rest_api;
pub mod rpc_api;

// Frontend sections
pub mod frontend;

use crate::models::spec_rules::SectionType;

// ─── Generator Args ──────────────────────────────────────────────────────────
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/mod.md#schema
// CODEGEN-BEGIN
/// Arguments for invoking a structural generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mod.md#schema
#[derive(Debug, Clone)]
pub struct GeneratorArgs {
    /// Target section type.
    pub section_type: SectionType,
    /// Change ID providing context (from --sdd-id).
    pub sdd_id: Option<String>,
    /// Related spec references (from --sdd-refs).
    pub sdd_refs: Vec<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/mod_runtime_source.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mod_runtime_source.md#source
impl GeneratorArgs {
    /// Create `GeneratorArgs` with just a section type.
    pub fn new(section_type: SectionType) -> Self {
        Self {
            section_type,
            sdd_id: None,
            sdd_refs: Vec::new(),
        }
    }

    /// Builder: set sdd_id.
    pub fn with_sdd_id(mut self, sdd_id: impl Into<String>) -> Self {
        self.sdd_id = Some(sdd_id.into());
        self
    }

    /// Builder: set sdd_refs.
    pub fn with_sdd_refs(mut self, refs: Vec<String>) -> Self {
        self.sdd_refs = refs;
        self
    }
}

// ─── Generator Trait ─────────────────────────────────────────────────────────

/// Trait for section content generators.
///
/// Implementors return the body content for a section (everything after
/// the H2 heading). The annotation comment is injected automatically
/// by `inject_annotation()`.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mod_runtime_source.md#source
pub trait Generator {
    /// The section type this generator handles.
    fn section_type(&self) -> SectionType;

    /// Generate section body content (without heading or annotation).
    fn generate(&self, args: &GeneratorArgs) -> String;

    /// Generate section content with the type annotation prepended.
    ///
    /// Output format:
    /// ```text
    /// <!-- type: <type> lang: <lang> -->
    ///
    /// <body content>
    /// ```
    fn generate_with_annotation(&self, args: &GeneratorArgs) -> String {
        let st = self.section_type();
        let lang = st.default_lang();
        let annotation = format!("<!-- type: {} lang: {} -->", st.as_str(), lang);
        let body = self.generate(args);
        if body.trim().is_empty() {
            format!("{}\n", annotation)
        } else {
            format!("{}\n\n{}", annotation, body)
        }
    }
}

// ─── Generator Registry ──────────────────────────────────────────────────────

/// Look up a generator by section type.
///
/// Most section types have generators registered.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mod_runtime_source.md#source
pub fn get_generator(section_type: SectionType) -> Option<Box<dyn Generator>> {
    match section_type {
        // Core prose
        SectionType::Overview => Some(Box::new(overview::OverviewGenerator {})),
        SectionType::Changes => Some(Box::new(changes::ChangesGenerator {})),
        // Behavioral
        SectionType::Requirements => Some(Box::new(requirements::RequirementsGenerator {})),
        SectionType::Scenarios => Some(Box::new(scenarios::ScenariosGenerator {})),
        SectionType::UnitTest => Some(Box::new(test_plan::TestPlanGenerator {})),
        // Mermaid diagrams
        SectionType::Interaction => Some(Box::new(sequence::SequenceGenerator {})),
        SectionType::Logic => Some(Box::new(flowchart::FlowchartGenerator {})),
        SectionType::Dependency => Some(Box::new(dependency::DependencyGenerator {})),
        SectionType::StateMachine => Some(Box::new(state_machine::StateMachineGenerator {})),
        SectionType::DbModel => Some(Box::new(db_model::DbModelGenerator {})),
        SectionType::Mindmap => Some(Box::new(mindmap::MindmapGenerator {})),
        // API specs
        SectionType::RestApi => Some(Box::new(rest_api::RestApiGenerator {})),
        SectionType::RpcApi => Some(Box::new(rpc_api::RpcApiGenerator {})),
        SectionType::AsyncApi => Some(Box::new(async_api::AsyncApiGenerator {})),
        // New section types (no dedicated generators yet — return None)
        SectionType::Cli
        | SectionType::Schema
        | SectionType::Config
        | SectionType::Component
        | SectionType::DesignToken
        | SectionType::RuntimeImage
        | SectionType::Deployment
        | SectionType::Manifest
        | SectionType::ToolContract
        | SectionType::RustSourceUnit
        | SectionType::TextSourceUnit
        | SectionType::E2eTest => None,
        // Frontend / Doc
        SectionType::Wireframe => Some(Box::new(frontend::FrontendGenerator {})),
        SectionType::Doc => Some(Box::new(doc::DocGenerator {})),
    }
}

/// Generate section content for a given type, with annotation injected.
///
/// Returns `None` if no generator exists for the given type.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mod_runtime_source.md#source
pub fn generate_section(args: &GeneratorArgs) -> Option<String> {
    get_generator(args.section_type).map(|g| g.generate_with_annotation(args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::spec_rules::SectionType;

    #[test]
    fn test_generator_args_builder() {
        let args = GeneratorArgs::new(SectionType::Overview)
            .with_sdd_id("my-change")
            .with_sdd_refs(vec!["ref-spec".to_string()]);
        assert_eq!(args.section_type, SectionType::Overview);
        assert_eq!(args.sdd_id.as_deref(), Some("my-change"));
        assert_eq!(args.sdd_refs, vec!["ref-spec".to_string()]);
    }

    /// Section types that do not (yet) have dedicated generators in this
    /// generators module. `Manifest` and `E2eTest` are handled by the TD v2
    /// Rust-codegen path (`projects/agentic-workflow/src/generate/gen/rust/*`) not the
    /// legacy prose generators here.
    const NO_GENERATOR: &[SectionType] = &[
        SectionType::Cli,
        SectionType::Schema,
        SectionType::Config,
        SectionType::Component,
        SectionType::DesignToken,
        SectionType::RuntimeImage,
        SectionType::Deployment,
        SectionType::Manifest,
        SectionType::E2eTest,
    ];

    #[test]
    fn test_generators_exist_for_supported_types() {
        for st in SectionType::all_in_fill_order() {
            if NO_GENERATOR.contains(&st) {
                assert!(
                    get_generator(st).is_none(),
                    "Unexpected generator for {:?}",
                    st
                );
            } else {
                assert!(
                    get_generator(st).is_some(),
                    "Generator missing for {:?}",
                    st
                );
            }
        }
    }

    #[test]
    fn test_generators_produce_annotation() {
        for st in SectionType::all_in_fill_order() {
            if NO_GENERATOR.contains(&st) {
                continue;
            }
            let args = GeneratorArgs::new(st);
            let output = generate_section(&args).unwrap();
            let expected = format!("<!-- type: {} lang: {} -->", st.as_str(), st.default_lang());
            assert!(
                output.starts_with(&expected),
                "Annotation mismatch for {:?}: output starts with {:?}",
                st,
                &output[..output.len().min(60)]
            );
        }
    }

    #[test]
    fn test_generators_produce_nonempty_body() {
        for st in SectionType::all_in_fill_order() {
            if NO_GENERATOR.contains(&st) {
                continue;
            }
            let args = GeneratorArgs::new(st).with_sdd_id("test-id");
            let gen = get_generator(st).unwrap();
            let body = gen.generate(&args);
            assert!(
                !body.trim().is_empty(),
                "Generator for {:?} produced empty body",
                st
            );
        }
    }

    #[test]
    fn test_overview_generator_exists() {
        let gen = get_generator(SectionType::Overview);
        assert!(gen.is_some());
    }

    #[test]
    fn test_changes_generator_exists() {
        let gen = get_generator(SectionType::Changes);
        assert!(gen.is_some());
    }

    #[test]
    fn test_annotation_injection_overview() {
        let args = GeneratorArgs::new(SectionType::Overview);
        let output = generate_section(&args).unwrap();
        assert!(output.starts_with("<!-- type: overview lang: markdown -->"));
    }

    #[test]
    fn test_annotation_injection_changes() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = generate_section(&args).unwrap();
        assert!(output.starts_with("<!-- type: changes lang: yaml -->"));
    }

    #[test]
    fn test_mermaid_generators_produce_fenced_block() {
        let mermaid_types = [
            SectionType::Interaction,
            SectionType::Logic,
            SectionType::Dependency,
            SectionType::StateMachine,
            SectionType::DbModel,
            SectionType::Mindmap,
        ];
        for st in mermaid_types {
            let args = GeneratorArgs::new(st).with_sdd_id("test");
            let gen = get_generator(st).unwrap();
            let body = gen.generate(&args);
            assert!(
                body.contains("```mermaid"),
                "Mermaid generator for {:?} missing ```mermaid fence",
                st
            );
            assert!(
                body.contains("id: test"),
                "Mermaid generator for {:?} missing sdd-id in frontmatter",
                st
            );
        }
    }

    #[test]
    fn test_api_generators_inject_x_sdd() {
        let api_types = [
            SectionType::RestApi,
            SectionType::RpcApi,
            SectionType::AsyncApi,
        ];
        for st in api_types {
            let args = GeneratorArgs::new(st).with_sdd_id("test-api");
            let gen = get_generator(st).unwrap();
            let body = gen.generate(&args);
            assert!(
                body.contains("x-sdd") || body.contains("\"x-sdd\""),
                "API generator for {:?} missing x-sdd metadata",
                st
            );
        }
    }

    #[test]
    fn test_frontend_generator_injects_sdd() {
        let args = GeneratorArgs::new(SectionType::Wireframe).with_sdd_id("test-ui");
        let gen = get_generator(SectionType::Wireframe).unwrap();
        let body = gen.generate(&args);
        assert!(body.contains("_sdd:"));
        assert!(body.contains("id: \"test-ui\""));
    }
}
// CODEGEN-END

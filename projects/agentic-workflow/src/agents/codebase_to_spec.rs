// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
// CODEGEN-BEGIN
//! CodebaseToSpecAgent — generates SDD specifications from codebase context artifacts.
//!
//! Operates during the SDD fillback flow (phase: CodebaseToSpec). Takes a
//! [`ReferenceCodebaseArtifact`] produced by [`ReferenceCodebaseContextAgent`] and
//! reverse-engineers a structured SDD-compliant specification from it.
//!
//! # CRR Integration
//!
//! Like [`ChangeSpecAgent`], this agent supports both creator and reviser roles:
//!
//! - **Creator**: `run()` receives a JSON-encoded [`CodebaseToSpecInput`] →
//!   [`generate_spec`][CodebaseToSpecAgent::generate_spec].
//! - **Reviser**: `run()` receives a plain-text CRR revision prompt →
//!   calls the LLM directly with the SDD system prompt prepended.
//!
//! # SDD Format
//!
//! Output follows the artifact-oriented TD taxonomy enforced by [`ChangeSpecAgent`]:
//! codegen/handwrite sections, `unit-test`, and `e2e-test`. New TDs do not
//! include legacy `changes` sections.
//!
//! [`ChangeSpecAgent`]: crate::agents::change_spec::ChangeSpecAgent

use crate::agents::reference_codebase_context::ReferenceCodebaseArtifact;
use crate::agents::review::ReviewIssue;
use crate::agents::Agent;
use agent::error::{NovaError, NovaResult};
use agent::llm::{CompletionRequest, LLMProvider};
use agent::stream::StreamHandler;
use agent::types::Message;
use async_trait::async_trait;
use std::sync::Arc;

// ============================================================
// Input type
// ============================================================

use serde::{Deserialize, Serialize};

/// Input for CodebaseToSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseToSpecInput {
    /// Structured codebase context.
    pub codebase_context: ReferenceCodebaseArtifact,
    /// Optional target spec file path.
    pub target_spec_path: Option<String>,
    /// Optional additional constraints.
    pub additional_context: Option<String>,
}

/// Configuration for CodebaseToSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseToSpecAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Maximum response tokens.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Number of LLM completion retries.
    pub max_retries: u32,
}

/// Codebase-to-spec agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#schema
pub struct CodebaseToSpecAgent {
    /// Agent configuration.
    config: CodebaseToSpecAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
}

/// Builder for CodebaseToSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#schema
pub struct CodebaseToSpecAgentBuilder {
    /// Agent configuration.
    config: CodebaseToSpecAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#changes
// ============================================================
// Agent config
// ============================================================

impl Default for CodebaseToSpecAgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(8192),
            temperature: Some(0.3),
            max_retries: 2,
        }
    }
}

// ============================================================
// Agent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
impl std::fmt::Debug for CodebaseToSpecAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodebaseToSpecAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
impl CodebaseToSpecAgent {
    /// Create a new builder.
    pub fn builder() -> CodebaseToSpecAgentBuilder {
        CodebaseToSpecAgentBuilder::new()
    }

    /// Generate an initial SDD specification from a codebase context artifact.
    pub async fn generate_spec(&self, input: &CodebaseToSpecInput) -> NovaResult<String> {
        let (system_msg, user_msg) = build_generate_prompt(input);
        self.complete_text(vec![system_msg, user_msg]).await
    }

    /// Revise an existing spec to address review issues.
    pub async fn revise_spec(&self, spec: &str, issues: &[ReviewIssue]) -> NovaResult<String> {
        let user_content = build_revise_prompt(spec, issues);
        self.complete_text(vec![
            Message::system(SYSTEM_PROMPT),
            Message::user(user_content),
        ])
        .await
    }

    // ---- private ----

    /// Call the LLM and return the text response, retrying on empty responses.
    async fn complete_text(&self, messages: Vec<Message>) -> NovaResult<String> {
        let mut request = CompletionRequest::new(messages, &self.config.model);
        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        let mut last_error = String::new();

        for attempt in 0..=self.config.max_retries {
            match self.provider.complete(request.clone()).await {
                Ok(response) => {
                    let content = response.content.trim().to_string();
                    if !content.is_empty() {
                        return Ok(content);
                    }
                    last_error = "empty response from LLM".to_string();
                }
                Err(e) => {
                    last_error = e.to_string();
                }
            }

            if attempt < self.config.max_retries {
                request.messages.push(Message::user(format!(
                    "The previous response was invalid ({}). \
                     Please generate the complete specification again.",
                    last_error
                )));
            }
        }

        Err(NovaError::Other(anyhow::anyhow!(
            "CodebaseToSpecAgent: failed after {} retries. Last error: {}",
            self.config.max_retries,
            last_error
        )))
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
impl Agent for CodebaseToSpecAgent {
    /// Run in creator or reviser role.
    ///
    /// - **Creator**: input is JSON-encoded [`CodebaseToSpecInput`] →
    ///   [`generate_spec`][CodebaseToSpecAgent::generate_spec].
    /// - **Reviser**: input is a CRR revision prompt → `complete_text` with system prompt.
    async fn run(&self, input: &str) -> NovaResult<String> {
        match serde_json::from_str::<CodebaseToSpecInput>(input) {
            Ok(spec_input) => self.generate_spec(&spec_input).await,
            Err(_) => {
                // Reviser role: CRR built the revision prompt; prepend the SDD system prompt.
                self.complete_text(vec![
                    Message::system(SYSTEM_PROMPT),
                    Message::user(input.to_string()),
                ])
                .await
            }
        }
    }

    async fn run_with_handler(
        &self,
        input: &str,
        _handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        self.run(input).await
    }
}

// ============================================================
// System prompt
// ============================================================

const SYSTEM_PROMPT: &str = r#"You are an expert specification author for spec-driven development (SDD).

Your task is to reverse-engineer a formal technical specification from analyzed codebase context.

Generate formal technical specifications following these strict rules.

## Format Priority (highest to lowest)

1. OpenRPC JSON — for MCP/API tool definitions
2. JSON Schema — for data models, state, payloads
3. Mermaid diagrams — for structure visualization
4. YAML — for config schemas, CLI command trees
5. Markdown tables — for mappings, enums, checklists
6. Prose — ONLY for context that cannot be expressed above

## Diagram Selection

| Structure                    | Diagram           |
|------------------------------|-------------------|
| State machine / lifecycle    | `stateDiagram-v2` |
| Algorithm / DAG / decisions  | `flowchart`       |
| Object relations / models    | `classDiagram`    |
| Actor interactions / APIs    | `sequenceDiagram` |

## Section Structure

Create only sections that drive codegen, handwrite, or verification artifacts.
Use `unit-test` for generated unit test design and `e2e-test` for product
journey and side-effect verification. Do not add legacy `changes`; TD-to-codebase
tooling infers implementation targets from existing spec refs.

## Quality Gates

- Natural language MUST NOT exceed 10% of total spec content
- NEVER include real Rust, Python, or TypeScript implementation code
- Use pseudocode blocks if behavior logic must be described:
  ```
  FUNCTION name(param: Type) -> Result
    INPUT: ...
    OUTPUT: ...
    ERRORS: ...
  ```
- FSMs MUST use `stateDiagram-v2` — NEVER use flowchart for state machines
- Do not add legacy changes sections for new TDs"#;

// ============================================================
// Prompt builders
// ============================================================

fn build_generate_prompt(input: &CodebaseToSpecInput) -> (Message, Message) {
    let system_msg = Message::system(SYSTEM_PROMPT);
    let ctx = &input.codebase_context;

    let mut content = format!(
        "## Codebase Analysis\n\n\
         **Target**: {}\n\n\
         **Summary**: {}\n\n",
        ctx.target, ctx.summary
    );

    // Key files
    if !ctx.key_files.is_empty() {
        content.push_str("### Key Files\n\n");
        for file in &ctx.key_files {
            let exports = if file.key_exports.is_empty() {
                String::new()
            } else {
                format!(" — exports: `{}`", file.key_exports.join("`, `"))
            };
            content.push_str(&format!("- `{}`: {}{}\n", file.path, file.purpose, exports));
        }
        content.push('\n');
    }

    // Architectural patterns
    if !ctx.architectural_patterns.is_empty() {
        content.push_str("### Architectural Patterns\n\n");
        for pattern in &ctx.architectural_patterns {
            content.push_str(&format!("- {}\n", pattern));
        }
        content.push('\n');
    }

    // Dependencies
    if !ctx.dependencies.is_empty() {
        content.push_str("### Dependencies\n\n");
        for dep in &ctx.dependencies {
            content.push_str(&format!(
                "- `{}` ({}): {}\n",
                dep.name, dep.dependency_type, dep.purpose
            ));
        }
        content.push('\n');
    }

    // Relationships
    if !ctx.relationships.is_empty() {
        content.push_str("### Component Relationships\n\n");
        for rel in &ctx.relationships {
            content.push_str(&format!(
                "- `{}` → `{}`: {}\n",
                rel.from, rel.to, rel.relationship_type
            ));
        }
        content.push('\n');
    }

    // Optional target spec path
    if let Some(ref path) = input.target_spec_path {
        content.push_str(&format!("**Target Spec Path**: `{}`\n\n", path));
    }

    // Additional context
    if let Some(ref ctx_note) = input.additional_context {
        content.push_str(&format!("### Additional Context\n\n{}\n\n", ctx_note));
    }

    content.push_str(
        "Generate a complete technical specification that accurately captures the \
         functionality, data models, and logic present in the analyzed codebase. \
         Follow all SDD format rules and artifact section guidance.",
    );

    (system_msg, Message::user(content))
}

fn build_revise_prompt(spec: &str, issues: &[ReviewIssue]) -> String {
    let mut prompt = format!(
        "Revise the following specification to address all review issues listed below.\n\n\
         ## Original Spec\n\n{}\n\n\
         ## Review Issues\n",
        spec
    );

    for (i, issue) in issues.iter().enumerate() {
        prompt.push_str(&format!(
            "\n{}. [{}] {}\n   Suggestion: {}\n",
            i + 1,
            issue.severity,
            issue.description,
            issue.suggestion,
        ));
        if let Some(ref loc) = issue.location {
            prompt.push_str(&format!("   Location: {}\n", loc));
        }
    }

    prompt.push_str(
        "\nAddress every issue above. Return the fully revised specification, \
         maintaining the artifact section structure and all SDD format rules.",
    );
    prompt
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
impl CodebaseToSpecAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: CodebaseToSpecAgentConfig::default(),
            provider: None,
        }
    }

    pub fn with_provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
        self.provider = Some(Arc::new(provider));
        self
    }

    pub fn with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.config.max_retries = n;
        self
    }

    pub fn build(self) -> NovaResult<CodebaseToSpecAgent> {
        let provider = self.provider.ok_or_else(|| {
            NovaError::ConfigError("CodebaseToSpecAgent: provider is required".to_string())
        })?;
        Ok(CodebaseToSpecAgent {
            config: self.config,
            provider,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/codebase_to_spec.md#source
impl Default for CodebaseToSpecAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::reference_codebase_context::{
        CodebaseDependency, ComponentRelationship, KeyFile, ReferenceCodebaseArtifact,
    };
    use crate::agents::review::{ReviewIssue, Severity};
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response: String,
    }

    #[async_trait]
    impl agent::llm::LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            "openai"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            Ok(CompletionResponse {
                content: self.response.clone(),
                tool_calls: None,
                finish_reason: "stop".to_string(),
                usage: TokenUsage::default(),
                model: "mock-model".to_string(),
                metadata: HashMap::new(),
            })
        }
        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    /// Provider that returns responses from a scripted sequence.
    struct SequenceProvider {
        responses: Mutex<Vec<String>>,
        fallback: String,
    }

    impl SequenceProvider {
        fn new(responses: Vec<String>, fallback: &str) -> Self {
            Self {
                responses: Mutex::new(responses),
                fallback: fallback.to_string(),
            }
        }
    }

    #[async_trait]
    impl agent::llm::LLMProvider for SequenceProvider {
        fn provider_name(&self) -> &str {
            "openai"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            let mut responses = self.responses.lock().unwrap();
            let content = if responses.is_empty() {
                self.fallback.clone()
            } else {
                responses.remove(0)
            };
            Ok(CompletionResponse {
                content,
                tool_calls: None,
                finish_reason: "stop".to_string(),
                usage: TokenUsage::default(),
                model: "mock-model".to_string(),
                metadata: HashMap::new(),
            })
        }
        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    // ---- Helpers ----

    const DRAFT_SPEC: &str = "\
## Overview\n\
\n\
Draft spec overview.\n\
\n\
## Requirements\n\
\n\
R1: The agent MUST generate specs.\n\
\n\
## Changes\n\
\n\
- `crates/foo/src/lib.rs` (Create): Initial implementation.";

    fn make_artifact() -> ReferenceCodebaseArtifact {
        ReferenceCodebaseArtifact {
            target: "crates/cclab-agent/src/agents".to_string(),
            key_files: vec![KeyFile {
                path: "src/agents/mod.rs".to_string(),
                purpose: "Module entry, Agent trait definition".to_string(),
                key_exports: vec!["Agent".to_string(), "ApprovalHandler".to_string()],
            }],
            architectural_patterns: vec![
                "trait objects".to_string(),
                "builder pattern".to_string(),
            ],
            dependencies: vec![CodebaseDependency {
                name: "async_trait".to_string(),
                dependency_type: "external".to_string(),
                purpose: "Async trait support".to_string(),
            }],
            relationships: vec![ComponentRelationship {
                from: "CodingAgent".to_string(),
                to: "Agent".to_string(),
                relationship_type: "implements".to_string(),
            }],
            summary: "The agents module defines the Agent trait and multiple implementations."
                .to_string(),
        }
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        ReviewIssue {
            severity: Severity::High,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_generate_spec_returns_llm_content() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: None,
            additional_context: None,
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_generate_spec_with_optional_fields() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: Some("cclab-agent/agents.md".to_string()),
            additional_context: Some("Focus on the Agent trait contract.".to_string()),
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_revise_spec_returns_revised_content() {
        let revised = "## Overview\n\nRevised overview.\n\n## Requirements\n\nR1: Updated.";
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: revised.to_string(),
            })
            .build()
            .unwrap();

        let issues = vec![make_review_issue("Missing diagram section")];
        let result = agent.revise_spec(DRAFT_SPEC, &issues).await.unwrap();
        assert_eq!(result, revised.trim());
    }

    #[tokio::test]
    async fn test_run_creator_role_via_json_input() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: None,
            additional_context: None,
        };
        let json = serde_json::to_string(&input).unwrap();
        let result = agent.run(&json).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_run_reviser_role_via_text_prompt() {
        let revised = "## Overview\n\nRevised.\n\n## Requirements\n\nR1: Fixed.";
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: revised.to_string(),
            })
            .build()
            .unwrap();

        let prompt = format!(
            "Revise the following artifact based on the review issues listed below.\n\n\
             ## Original Artifact\n\n{}\n\n\
             ## Review Issues\n\n\
             1. [High] Missing diagrams\n   Suggestion: Add them",
            DRAFT_SPEC
        );
        let result = agent.run(&prompt).await.unwrap();
        assert_eq!(result, revised.trim());
    }

    #[tokio::test]
    async fn test_empty_response_triggers_retry() {
        let provider = SequenceProvider::new(vec!["   ".to_string()], DRAFT_SPEC);

        let agent = CodebaseToSpecAgent::builder()
            .with_provider(provider)
            .with_max_retries(1)
            .build()
            .unwrap();

        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: None,
            additional_context: None,
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_all_retries_exhausted_returns_error() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: "   ".to_string(),
            })
            .with_max_retries(1)
            .build()
            .unwrap();

        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: None,
            additional_context: None,
        };
        let err = agent.generate_spec(&input).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("CodebaseToSpecAgent"),
            "expected agent error, got: {}",
            msg
        );
    }

    #[tokio::test]
    async fn test_revise_spec_with_location_in_issue() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let issues = vec![ReviewIssue {
            severity: Severity::High,
            description: "Missing classDiagram".to_string(),
            suggestion: "Add a classDiagram for Agent hierarchy".to_string(),
            location: Some("spec.md:Diagrams".to_string()),
        }];

        let result = agent.revise_spec(DRAFT_SPEC, &issues).await.unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_builder_missing_provider_returns_config_error() {
        let err = CodebaseToSpecAgent::builder().build().unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_input_round_trips_json() {
        let input = CodebaseToSpecInput {
            codebase_context: make_artifact(),
            target_spec_path: Some("cclab-agent/agents.md".to_string()),
            additional_context: Some("Focus on public API.".to_string()),
        };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: CodebaseToSpecInput = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.codebase_context.target,
            input.codebase_context.target
        );
        assert_eq!(parsed.target_spec_path, input.target_spec_path);
        assert_eq!(parsed.additional_context, input.additional_context);
    }

    #[test]
    fn test_config_defaults() {
        let config = CodebaseToSpecAgentConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, Some(8192));
        assert_eq!(config.temperature, Some(0.3));
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_builder_with_overrides() {
        let agent = CodebaseToSpecAgent::builder()
            .with_provider(MockProvider {
                response: String::new(),
            })
            .with_model("gemini-2.0-flash")
            .with_max_tokens(4096)
            .with_temperature(0.5)
            .with_max_retries(1)
            .build()
            .unwrap();

        assert_eq!(agent.config.model, "gemini-2.0-flash");
        assert_eq!(agent.config.max_tokens, Some(4096));
        assert_eq!(agent.config.temperature, Some(0.5));
        assert_eq!(agent.config.max_retries, 1);
    }
}

// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
// CODEGEN-BEGIN
//! ChangeSpecAgent — generates formal technical specifications from structured issues.
//!
//! Operates during SDD phase 6 (change spec). Takes a [`ChangeSpecInput`] containing
//! a [`StructuredIssue`] and [`ReferenceContextOutput`], generates a complete sdd
//! spec document, and integrates with the CRR cycle as both creator and reviser.
//!
//! # CRR Integration
//!
//! The agent implements [`Agent`] so it can fill both creator and reviser roles in a
//! [`CRRCycle`]:
//!
//! - **Creator**: `run()` receives a JSON-encoded [`ChangeSpecInput`] → calls
//!   [`generate_spec`][ChangeSpecAgent::generate_spec].
//! - **Reviser**: `run()` receives the CRR revision prompt (artifact + issues text) →
//!   calls the LLM directly with the SDD system prompt prepended.
//!
//! # SDD Format Rules Enforced
//!
//! 1. Format priority: OpenRPC > JSON Schema > Mermaid > YAML > Markdown table > Prose
//! 2. Diagram selection: FSM → `stateDiagram-v2`, DAG → `flowchart`,
//!    actors → `sequenceDiagram`, objects → `classDiagram`
//! 3. Artifact sections: codegen/handwrite sections, `unit-test`, and `e2e-test`
//! 4. No real Rust/Python/TypeScript implementation code in output
//! 5. Natural language ≤ 10% of spec content
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::{ChangeSpecAgent, ChangeSpecInput, ClaudeProvider};
//!
//! let agent = ChangeSpecAgent::builder()
//!     .with_provider(provider)
//!     .build()?;
//!
//! let input = ChangeSpecInput { issue, context };
//! let spec = agent.generate_spec(&input).await?;
//! println!("{}", spec);
//! ```

use crate::agents::reference_spec_context::ReferenceContextOutput;
use crate::agents::restructure::StructuredIssue;
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

/// Input for ChangeSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSpecInput {
    /// The structured issue to resolve.
    pub issue: StructuredIssue,
    /// Synthesized reference context from existing project specs.
    pub context: ReferenceContextOutput,
}

/// Configuration for ChangeSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSpecAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Maximum response tokens.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Number of LLM completion retries.
    pub max_retries: u32,
}

/// ChangeSpec agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#schema
pub struct ChangeSpecAgent {
    /// Agent configuration.
    config: ChangeSpecAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
}

/// Builder for ChangeSpecAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#schema
pub struct ChangeSpecAgentBuilder {
    /// Agent configuration.
    config: ChangeSpecAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#changes
// ============================================================
// Agent config
// ============================================================

impl Default for ChangeSpecAgentConfig {
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

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
impl std::fmt::Debug for ChangeSpecAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSpecAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
impl ChangeSpecAgent {
    /// Create a new builder.
    pub fn builder() -> ChangeSpecAgentBuilder {
        ChangeSpecAgentBuilder::new()
    }

    /// Generate an initial formal spec from a structured issue and reference context.
    pub async fn generate_spec(&self, input: &ChangeSpecInput) -> NovaResult<String> {
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
            "ChangeSpecAgent: failed after {} retries. Last error: {}",
            self.config.max_retries,
            last_error
        )))
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
impl Agent for ChangeSpecAgent {
    /// Run in creator or reviser role.
    ///
    /// - **Creator**: input is JSON-encoded [`ChangeSpecInput`] → [`generate_spec`][ChangeSpecAgent::generate_spec].
    /// - **Reviser**: input is a CRR revision prompt → `complete_text` with system prompt.
    async fn run(&self, input: &str) -> NovaResult<String> {
        match serde_json::from_str::<ChangeSpecInput>(input) {
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

fn build_generate_prompt(input: &ChangeSpecInput) -> (Message, Message) {
    let system_msg = Message::system(SYSTEM_PROMPT);

    let issue = &input.issue;
    let mut content = format!(
        "## Structured Issue\n\n\
         - **Title**: {}\n\
         - **Type**: {}\n\
         - **Priority**: {}\n\
         - **Scope**: {}\n\n\
         **Description**:\n{}\n\n\
         **Acceptance Criteria**:\n",
        issue.title, issue.issue_type, issue.priority, issue.scope, issue.description
    );

    for (i, criterion) in issue.acceptance_criteria.iter().enumerate() {
        content.push_str(&format!("{}. {}\n", i + 1, criterion));
    }

    if !issue.labels.is_empty() {
        content.push_str("\n**Labels**: ");
        content.push_str(&issue.labels.join(", "));
        content.push('\n');
    }

    if !issue.depends_on.is_empty() {
        content.push_str("\n**Depends On**: ");
        content.push_str(&issue.depends_on.join(", "));
        content.push('\n');
    }

    content.push_str("\n## Reference Context\n\n");

    if input.context.specs.is_empty() {
        content.push_str("(No relevant specs found)\n\n");
    } else {
        for spec in &input.context.specs {
            let relevance = format!("{:?}", spec.relevance).to_lowercase();
            if spec.key_requirements.is_empty() {
                content.push_str(&format!("- `{}` ({})\n", spec.spec_id, relevance));
            } else {
                content.push_str(&format!(
                    "- `{}` ({}): {}\n",
                    spec.spec_id,
                    relevance,
                    spec.key_requirements.join("; ")
                ));
            }
        }
        content.push('\n');
    }

    if !input.context.contradictions.is_empty() {
        content.push_str("### Contradictions to Resolve\n\n");
        for c in &input.context.contradictions {
            content.push_str(&format!(
                "- `{}`: `{}` — Conflict: {} — Resolution: {}\n",
                c.spec_id, c.requirement, c.conflict, c.resolution
            ));
        }
        content.push('\n');
    }

    content.push_str(
        "Generate a complete technical specification for this issue \
         following all SDD format rules and artifact section guidance.",
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

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
impl ChangeSpecAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ChangeSpecAgentConfig::default(),
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

    pub fn build(self) -> NovaResult<ChangeSpecAgent> {
        let provider = self.provider.ok_or_else(|| {
            NovaError::ConfigError("ChangeSpecAgent: provider is required".to_string())
        })?;
        Ok(ChangeSpecAgent {
            config: self.config,
            provider,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/change_spec.md#source
impl Default for ChangeSpecAgentBuilder {
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
    use crate::agents::reference_spec_context::{
        Contradiction, ReferenceContextOutput, RelevanceLevel, SpecReferenceEntry,
    };
    use crate::agents::restructure::StructuredIssue;
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
    impl LLMProvider for MockProvider {
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
    impl LLMProvider for SequenceProvider {
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

    fn make_issue() -> StructuredIssue {
        StructuredIssue {
            title: "feat(agent): add ChangeSpecAgent".to_string(),
            description: "Implement ChangeSpecAgent for SDD phase 6.".to_string(),
            issue_type: "feature".to_string(),
            priority: "P1".to_string(),
            labels: vec!["crate:agent".to_string()],
            acceptance_criteria: vec![
                "Agent generates spec from StructuredIssue and ReferenceContext".to_string(),
                "Agent revises spec given ReviewIssues".to_string(),
            ],
            depends_on: vec![],
            scope: "medium".to_string(),
        }
    }

    fn make_context() -> ReferenceContextOutput {
        ReferenceContextOutput {
            specs: vec![SpecReferenceEntry {
                spec_id: "cclab-agent/agents.md".to_string(),
                spec_group: "cclab-agent".to_string(),
                relevance: RelevanceLevel::High,
                key_requirements: vec!["Agent trait must be implemented".to_string()],
            }],
            contradictions: vec![],
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
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let input = ChangeSpecInput {
            issue: make_issue(),
            context: make_context(),
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_revise_spec_returns_revised_content() {
        let revised = "## Overview\n\nRevised overview.\n\n## Requirements\n\nR1: Updated.";
        let agent = ChangeSpecAgent::builder()
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
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let input = ChangeSpecInput {
            issue: make_issue(),
            context: make_context(),
        };
        let json = serde_json::to_string(&input).unwrap();
        let result = agent.run(&json).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_run_reviser_role_via_text_prompt() {
        let revised = "## Overview\n\nRevised.\n\n## Requirements\n\nR1: Fixed.";
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: revised.to_string(),
            })
            .build()
            .unwrap();

        // CRR revision prompt — plain text, not JSON
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
        // First call returns empty; second call returns valid content.
        let provider = SequenceProvider::new(vec!["   ".to_string()], DRAFT_SPEC);

        let agent = ChangeSpecAgent::builder()
            .with_provider(provider)
            .with_max_retries(1)
            .build()
            .unwrap();

        let input = ChangeSpecInput {
            issue: make_issue(),
            context: make_context(),
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert_eq!(result, DRAFT_SPEC.trim());
    }

    #[tokio::test]
    async fn test_all_retries_exhausted_returns_error() {
        // Always returns empty.
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: "   ".to_string(),
            })
            .with_max_retries(1)
            .build()
            .unwrap();

        let input = ChangeSpecInput {
            issue: make_issue(),
            context: make_context(),
        };
        let err = agent.generate_spec(&input).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("ChangeSpecAgent"),
            "expected agent error, got: {}",
            msg
        );
    }

    #[tokio::test]
    async fn test_generate_spec_with_contradictions() {
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let context = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![Contradiction {
                spec_id: "agent.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing is sync-only".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let input = ChangeSpecInput {
            issue: make_issue(),
            context,
        };
        let result = agent.generate_spec(&input).await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_revise_spec_with_location_in_issue() {
        let agent = ChangeSpecAgent::builder()
            .with_provider(MockProvider {
                response: DRAFT_SPEC.to_string(),
            })
            .build()
            .unwrap();

        let issues = vec![ReviewIssue {
            severity: Severity::High,
            description: "Missing classDiagram".to_string(),
            suggestion: "Add a classDiagram for the agent".to_string(),
            location: Some("spec.md:Diagrams".to_string()),
        }];

        let result = agent.revise_spec(DRAFT_SPEC, &issues).await.unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_builder_missing_provider_returns_config_error() {
        let err = ChangeSpecAgent::builder().build().unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_change_spec_input_round_trips_json() {
        let input = ChangeSpecInput {
            issue: make_issue(),
            context: make_context(),
        };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: ChangeSpecInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.issue.title, input.issue.title);
        assert_eq!(parsed.context.specs.len(), 1);
    }

    #[test]
    fn test_config_defaults() {
        let config = ChangeSpecAgentConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, Some(8192));
        assert_eq!(config.temperature, Some(0.3));
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_builder_with_overrides() {
        let provider = MockProvider {
            response: String::new(),
        };
        let agent = ChangeSpecAgent::builder()
            .with_provider(provider)
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

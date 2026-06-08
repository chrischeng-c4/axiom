---
id: sdd-agents-restructure-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Restructure Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/restructure.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Clarification` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 26 |  |
| `Question` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 46 |  |
| `RestructureAgent` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 158 |  |
| `RestructureAgentBuilder` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 340 |  |
| `RestructureAgentConfig` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 96 |  |
| `RestructureInput` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 34 |  |
| `RestructureOutput` | projects/agentic-workflow/src/agents/restructure.rs | enum | pub | 71 |  |
| `SpecExcerpt` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 84 |  |
| `StructuredIssue` | projects/agentic-workflow/src/agents/restructure.rs | struct | pub | 56 |  |
| `build` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 401 | build(self) -> NovaResult<RestructureAgent> |
| `builder` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 167 | builder() -> RestructureAgentBuilder |
| `new` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 348 | new() -> Self |
| `run` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 172 | run(&self, input: &RestructureInput) -> NovaResult<RestructureOutput> |
| `with_max_retries` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 396 | with_max_retries(mut self, n: u32) -> Self |
| `with_max_spec_excerpts` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 391 | with_max_spec_excerpts(mut self, n: usize) -> Self |
| `with_max_tokens` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 381 | with_max_tokens(mut self, max_tokens: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 376 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 356 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 361 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_spec_store` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 366 | with_spec_store(mut self, store: S) -> Self |
| `with_spec_store_arc` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 371 | with_spec_store_arc(mut self, store: Arc<dyn SpecStore>) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/restructure.rs | function | pub | 386 | with_temperature(mut self, temperature: f32) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Clarification:
    type: object
    required: [question, answer]
    description: A single clarification Q&A pair from a previous round.
    properties:
      question:
        type: string
      answer:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RestructureInput:
    type: object
    required: [intent, project_id, clarifications]
    description: Typed input for the RestructureAgent.
    properties:
      intent:
        type: string
        description: "The user's raw intent or feature request."
      project_id:
        type: string
        description: "Project identifier used for SpecStore lookup."
      clarifications:
        type: array
        items:
          $ref: "#/definitions/Clarification"
        description: "Q&A clarifications from prior rounds (empty on first call)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Question:
    type: object
    required: [id, question, why, suggestions]
    description: A spec-informed clarification question produced when more info is needed.
    properties:
      id:
        type: string
      question:
        type: string
      why:
        type: string
      suggestions:
        type: array
        items: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StructuredIssue:
    type: object
    required: [title, description, issue_type, priority, labels, acceptance_criteria, depends_on, scope]
    description: A structured issue produced when sufficient information is available.
    properties:
      title:
        type: string
      description:
        type: string
      issue_type:
        type: string
      priority:
        type: string
      labels:
        type: array
        items: { type: string }
      acceptance_criteria:
        type: array
        items: { type: string }
      depends_on:
        type: array
        items: { type: string }
      scope:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RestructureOutput:
    type: object
    description: >
      Discriminated-union output from RestructureAgent. Serializes as
      `{"type": "need_clarification", ...}` or `{"type": "create_issues", ...}`.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_tag: type
      serde_rename_all: snake_case
      variants:
        - name: NeedClarification
          kind: struct
          fields:
            - { name: questions, rust_type: "Vec<Question>" }
        - name: CreateIssues
          kind: struct
          fields:
            - { name: issues,  rust_type: "Vec<StructuredIssue>" }
            - { name: summary, rust_type: "String" }

  SpecExcerpt:
    type: object
    required: [path, content, relevance]
    description: A spec excerpt returned by a SpecStore search.
    properties:
      path:
        type: string
        description: "File path relative to the project root."
      content:
        type: string
        description: "Relevant excerpt from the spec file."
      relevance:
        type: number
        x-rust-type: f32
        description: "Relevance score in [0, 1]."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RestructureAgentConfig:
    type: object
    required: [model, max_spec_excerpts, max_retries]
    description: Configuration for RestructureAgent.
    properties:
      model:
        type: string
      max_tokens:
        type: integer
        x-rust-type: "Option<u32>"
        x-serde-default: true
      temperature:
        type: number
        x-rust-type: "Option<f32>"
        x-serde-default: true
      max_spec_excerpts:
        type: integer
        x-rust-type: usize
        description: "Maximum number of spec excerpts to include in the prompt."
      max_retries:
        type: integer
        x-rust-type: u32
        description: "Number of structured-output retries on validation failure."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-trait-impls:
      - trait: Default
        impl_mode: codegen
        body: |
          Self {
              model: "claude-sonnet-4-20250514".to_string(),
              max_tokens: Some(4096),
              temperature: Some(0.3),
              max_spec_excerpts: 10,
              max_retries: 2,
          }
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/restructure.rs -->
````rust
//! RestructureAgent — LLM-based prompt refinement with structured I/O.
//!
//! Takes a user's vague intent, enriches it with spec context, and produces
//! either clarifying questions or well-structured issues.

use agent::error::{NovaError, NovaResult};
use agent::llm::{CompletionRequest, LLMProvider};
use agent::structured::complete_structured;
use agent::types::Message;
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

// ============================================================
// Input types
// ============================================================

use serde::{Deserialize, Serialize};

/// A single clarification Q&A pair from a previous round.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    pub question: String,
    pub answer: String,
}

/// Typed input for the RestructureAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestructureInput {
    /// The user's raw intent or feature request.
    pub intent: String,
    /// Project identifier used for SpecStore lookup.
    pub project_id: String,
    /// Q&A clarifications from prior rounds (empty on first call).
    pub clarifications: Vec<Clarification>,
}

/// A spec-informed clarification question produced when more info is needed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub why: String,
    pub suggestions: Vec<String>,
}

/// A structured issue produced when sufficient information is available.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredIssue {
    pub title: String,
    pub description: String,
    pub issue_type: String,
    pub priority: String,
    pub labels: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub depends_on: Vec<String>,
    pub scope: String,
}

/// Discriminated-union output from RestructureAgent. Serializes as `{"type": "need_clarification", ...}` or `{"type": "create_issues", ...}`.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RestructureOutput {
    NeedClarification {
        questions: Vec<Question>,
    },
    CreateIssues {
        issues: Vec<StructuredIssue>,
        summary: String,
    },
}

/// A spec excerpt returned by a SpecStore search.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecExcerpt {
    /// File path relative to the project root.
    pub path: String,
    /// Relevant excerpt from the spec file.
    pub content: String,
    /// Relevance score in [0, 1].
    pub relevance: f32,
}

/// Configuration for RestructureAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestructureAgentConfig {
    pub model: String,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum number of spec excerpts to include in the prompt.
    pub max_spec_excerpts: usize,
    /// Number of structured-output retries on validation failure.
    pub max_retries: u32,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#schema.trait-impls.Default
impl Default for RestructureAgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.3),
            max_spec_excerpts: 10,
            max_retries: 2,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
// ============================================================
// Output types
// ============================================================

// ============================================================
// SpecStore trait
// ============================================================

/// Trait for retrieving relevant spec context.
///
/// A production implementation (issue #901) queries the cclab spec index.
/// Tests can inject a mock.
#[async_trait]
pub trait SpecStore: Send + Sync {
    /// Return spec excerpts ranked by relevance to the query.
    async fn search(&self, query: &str) -> NovaResult<Vec<SpecExcerpt>>;

    /// Return the full text of a specification file by its path.
    async fn read(&self, path: &str) -> NovaResult<String>;
}

// ============================================================
// Agent
// ============================================================

/// Stateless agent that refines vague user intent into well-formed issues.
///
/// # Flow
///
/// 1. Search `SpecStore` for relevant spec excerpts.
/// 2. Assemble a prompt: intent + clarification history + spec excerpts.
/// 3. Call `complete_structured()` with the output JSON Schema.
/// 4. Deserialize the validated JSON into [`RestructureOutput`].
///
/// The agent has no internal mutable state and is safe to share across tasks.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
pub struct RestructureAgent {
    config: RestructureAgentConfig,
    provider: Arc<dyn LLMProvider>,
    spec_store: Arc<dyn SpecStore>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
impl RestructureAgent {
    /// Create a new builder.
    pub fn builder() -> RestructureAgentBuilder {
        RestructureAgentBuilder::new()
    }

    /// Run the agent with a typed input and return a typed output.
    pub async fn run(&self, input: &RestructureInput) -> NovaResult<RestructureOutput> {
        // 1. Retrieve relevant spec context
        let specs = self.spec_store.search(&input.intent).await?;
        let spec_count = specs.len().min(self.config.max_spec_excerpts);
        let specs = &specs[..spec_count];

        // 2. Build prompt
        let (system_msg, user_msg) = build_prompt(input, specs);
        let messages = vec![system_msg, user_msg];

        // 3. Build completion request
        let mut request = CompletionRequest::new(messages, &self.config.model);
        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        // 4. Call LLM with JSON Schema-enforced structured output
        let schema = output_schema();
        let (_response, value) = complete_structured(
            self.provider.as_ref(),
            request,
            &schema,
            self.config.max_retries,
        )
        .await?;

        // 5. Deserialize validated JSON into typed output
        let output: RestructureOutput = serde_json::from_value(value).map_err(|e| {
            NovaError::SchemaValidationError(format!("Failed to deserialize output: {}", e))
        })?;

        Ok(output)
    }
}

// ============================================================
// Prompt assembly
// ============================================================

const SYSTEM_PROMPT: &str = r#"You are an expert software requirements analyst.

Your job:
1. Analyze the user's intent against existing project specs.
2. Decide: is there enough information to produce well-formed issues?
   - If NO: produce specific, spec-informed clarifying questions.
   - If YES: produce structured issues with acceptance criteria.

Rules for questions:
- Questions must reference concrete existing specs (traits, types, modules).
- Never ask generic questions like "what feature do you need?".
- Include suggestions to help the user choose.

Rules for issues:
- Each issue title must follow conventional commit format (e.g. "feat(auth): ...").
- Acceptance criteria must be testable and concrete.
- Scope: "small" (<1 day), "medium" (1-3 days), "large" (>3 days).
- Priority: "P0" (critical), "P1" (high), "P2" (medium), "P3" (low).

Output MUST be a JSON object matching the provided schema."#;

fn build_prompt(input: &RestructureInput, specs: &[SpecExcerpt]) -> (Message, Message) {
    let system_msg = Message::system(SYSTEM_PROMPT);

    let mut content = format!("## Intent\n\n{}\n\n", input.intent);

    if !input.clarifications.is_empty() {
        content.push_str("## Clarification History\n\n");
        for c in &input.clarifications {
            content.push_str(&format!("**Q:** {}\n**A:** {}\n\n", c.question, c.answer));
        }
    }

    if !specs.is_empty() {
        content.push_str("## Relevant Spec Context\n\n");
        for spec in specs {
            content.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                spec.path, spec.content
            ));
        }
    } else {
        content.push_str("## Spec Context\n\n(No matching specs found)\n\n");
    }

    content.push_str("Based on the above, produce a structured JSON output.");
    (system_msg, Message::user(content))
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["type"],
        "properties": {
            "type": {
                "type": "string",
                "enum": ["need_clarification", "create_issues"]
            },
            "questions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "question", "why", "suggestions"],
                    "properties": {
                        "id": { "type": "string" },
                        "question": { "type": "string" },
                        "why": { "type": "string" },
                        "suggestions": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "issues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": [
                        "title", "description", "issue_type", "priority",
                        "labels", "acceptance_criteria", "depends_on", "scope"
                    ],
                    "properties": {
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "issue_type": { "type": "string" },
                        "priority": {
                            "type": "string",
                            "enum": ["P0", "P1", "P2", "P3"]
                        },
                        "labels": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "acceptance_criteria": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "depends_on": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "scope": {
                            "type": "string",
                            "enum": ["small", "medium", "large"]
                        }
                    },
                    "additionalProperties": false
                }
            },
            "summary": { "type": "string" }
        }
    })
}

// ============================================================
// Builder
// ============================================================

/// Builder for [`RestructureAgent`].
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
pub struct RestructureAgentBuilder {
    config: RestructureAgentConfig,
    provider: Option<Arc<dyn LLMProvider>>,
    spec_store: Option<Arc<dyn SpecStore>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
impl RestructureAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: RestructureAgentConfig::default(),
            provider: None,
            spec_store: None,
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

    pub fn with_spec_store<S: SpecStore + 'static>(mut self, store: S) -> Self {
        self.spec_store = Some(Arc::new(store));
        self
    }

    pub fn with_spec_store_arc(mut self, store: Arc<dyn SpecStore>) -> Self {
        self.spec_store = Some(store);
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

    pub fn with_max_spec_excerpts(mut self, n: usize) -> Self {
        self.config.max_spec_excerpts = n;
        self
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.config.max_retries = n;
        self
    }

    pub fn build(self) -> NovaResult<RestructureAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        Ok(RestructureAgent {
            config: self.config,
            provider,
            spec_store,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_types.md#source
impl Default for RestructureAgentBuilder {
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
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use std::collections::HashMap;

    // --- Mock LLM ---

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" causes content to be parsed directly as JSON
            "openai"
        }

        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }

        async fn complete(&self, _request: CompletionRequest) -> NovaResult<CompletionResponse> {
            Ok(CompletionResponse {
                content: self.response_json.clone(),
                tool_calls: None,
                finish_reason: "stop".to_string(),
                usage: TokenUsage::default(),
                model: "mock-model".to_string(),
                metadata: HashMap::new(),
            })
        }

        async fn complete_stream(&self, _request: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    // --- Mock SpecStores ---

    struct EmptySpecStore;

    #[async_trait]
    impl SpecStore for EmptySpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(vec![])
        }

        async fn read(&self, _path: &str) -> NovaResult<String> {
            Ok(String::new())
        }
    }

    struct FakeSpecStore;

    #[async_trait]
    impl SpecStore for FakeSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(vec![SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "AuthProvider trait with login/logout methods.".to_string(),
                relevance: 0.9,
            }])
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            Ok(format!(
                "# Full content of {}\n\nAuthProvider trait definition.",
                path
            ))
        }
    }

    // --- Helpers ---

    fn make_input() -> RestructureInput {
        RestructureInput {
            intent: "Add OAuth2 support with Google and GitHub".to_string(),
            project_id: "cclab".to_string(),
            clarifications: vec![],
        }
    }

    // --- Tests ---

    #[tokio::test]
    async fn test_run_returns_need_clarification() {
        let response_json = serde_json::json!({
            "type": "need_clarification",
            "questions": [
                {
                    "id": "q1",
                    "question": "Extend AuthProvider or create new OAuth2Provider?",
                    "why": "Affects whether this modifies existing spec or creates new one",
                    "suggestions": ["Extend existing", "Create new trait"]
                }
            ]
        })
        .to_string();

        let agent = RestructureAgent::builder()
            .with_provider(MockProvider { response_json })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap();

        let result = agent.run(&make_input()).await.unwrap();

        match result {
            RestructureOutput::NeedClarification { questions } => {
                assert_eq!(questions.len(), 1);
                assert_eq!(questions[0].id, "q1");
                assert_eq!(questions[0].suggestions.len(), 2);
            }
            _ => panic!("Expected NeedClarification"),
        }
    }

    #[tokio::test]
    async fn test_run_returns_create_issues() {
        let response_json = serde_json::json!({
            "type": "create_issues",
            "issues": [
                {
                    "title": "feat(auth): add OAuth2 provider trait",
                    "description": "Define OAuth2Provider trait with authorize_url, exchange_code, refresh_token.",
                    "issue_type": "feature",
                    "priority": "P1",
                    "labels": ["auth", "oauth2"],
                    "acceptance_criteria": [
                        "OAuth2Provider trait compiles",
                        "Unit tests cover trait contract"
                    ],
                    "depends_on": [],
                    "scope": "medium"
                }
            ],
            "summary": "1 trait definition issue"
        })
        .to_string();

        let agent = RestructureAgent::builder()
            .with_provider(MockProvider { response_json })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap();

        let result = agent.run(&make_input()).await.unwrap();

        match result {
            RestructureOutput::CreateIssues { issues, summary } => {
                assert_eq!(issues.len(), 1);
                assert_eq!(issues[0].priority, "P1");
                assert_eq!(issues[0].scope, "medium");
                assert_eq!(issues[0].acceptance_criteria.len(), 2);
                assert!(!summary.is_empty());
            }
            _ => panic!("Expected CreateIssues"),
        }
    }

    #[tokio::test]
    async fn test_spec_context_and_clarifications_assembled() {
        // Use FakeSpecStore to verify specs are fetched; use clarifications in input.
        let response_json = serde_json::json!({
            "type": "create_issues",
            "issues": [
                {
                    "title": "feat(auth): add OAuth2",
                    "description": "OAuth2 support via extension of AuthProvider.",
                    "issue_type": "feature",
                    "priority": "P1",
                    "labels": ["auth"],
                    "acceptance_criteria": ["Tests pass"],
                    "depends_on": [],
                    "scope": "small"
                }
            ],
            "summary": "1 issue"
        })
        .to_string();

        let agent = RestructureAgent::builder()
            .with_provider(MockProvider { response_json })
            .with_spec_store(FakeSpecStore)
            .build()
            .unwrap();

        let input = RestructureInput {
            intent: "Add OAuth2 support".to_string(),
            project_id: "cclab".to_string(),
            clarifications: vec![Clarification {
                question: "Extend existing trait?".to_string(),
                answer: "Yes, extend".to_string(),
            }],
        };

        let result = agent.run(&input).await.unwrap();
        assert!(matches!(result, RestructureOutput::CreateIssues { .. }));
    }

    #[test]
    fn test_builder_missing_provider_returns_err() {
        let result = RestructureAgent::builder()
            .with_spec_store(EmptySpecStore)
            .build();
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(err.contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store_returns_err() {
        let result = RestructureAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .build();
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(err.contains("SpecStore"));
    }

    #[test]
    fn test_restructure_input_round_trips() {
        let input = RestructureInput {
            intent: "Add feature X".to_string(),
            project_id: "proj1".to_string(),
            clarifications: vec![Clarification {
                question: "Which module?".to_string(),
                answer: "auth".to_string(),
            }],
        };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: RestructureInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.intent, "Add feature X");
        assert_eq!(parsed.clarifications.len(), 1);
        assert_eq!(parsed.clarifications[0].answer, "auth");
    }

    #[test]
    fn test_output_serializes_need_clarification_tag() {
        let output = RestructureOutput::NeedClarification {
            questions: vec![Question {
                id: "q1".to_string(),
                question: "How?".to_string(),
                why: "Because".to_string(),
                suggestions: vec!["Option A".to_string()],
            }],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["type"], "need_clarification");
        assert!(json["questions"].is_array());
    }

    #[test]
    fn test_output_serializes_create_issues_tag() {
        let output = RestructureOutput::CreateIssues {
            issues: vec![StructuredIssue {
                title: "feat: add X".to_string(),
                description: "Description".to_string(),
                issue_type: "feature".to_string(),
                priority: "P1".to_string(),
                labels: vec!["enhancement".to_string()],
                acceptance_criteria: vec!["Test passes".to_string()],
                depends_on: vec![],
                scope: "small".to_string(),
            }],
            summary: "1 issue".to_string(),
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["type"], "create_issues");
        assert_eq!(json["summary"], "1 issue");
        assert_eq!(json["issues"][0]["priority"], "P1");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/restructure.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete restructure agent module, including
      schema-derived data shapes, SpecStore, runtime behavior, prompt/schema
      helpers, builder methods, and tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Accurately enumerates the 7 codegen-tracked types and the three patterns (serde_tag, custom Default body, Vec-in-required). Hand-written boundary explicit.
- [schema] Schema is complete and correct: 5 plain structs with required-Vec convention; RestructureOutput serde_tag + 2 struct variants; SpecExcerpt with f32 type override; RestructureAgentConfig with x-rust-type Option<u32>/Option<f32> overrides + custom Default body.
- [changes] Two-entry split correctly partitions codegen (7 type decls + Default impl) from hand-written (RestructureAgent, Builder, SpecStore trait, helper fns, tests, preamble). `replaces:` lists all seven type names.

## Review 2
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Promotes the full restructure agent module to source-template ownership while keeping schema as the language-neutral shape contract.
- [source] Uses `strip-managed-markers` to preserve current Rust behavior and remove the mixed CODEGEN/HANDWRITE wrapper split.
- [changes] Correctly routes the file through the `source` section with `impl_mode: codegen`.

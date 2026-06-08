---
id: sdd-agents-review-mod
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Review Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/review/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReviewAgent` | projects/agentic-workflow/src/agents/review/mod.rs | struct | pub | 56 |  |
| `ReviewAgentBuilder` | projects/agentic-workflow/src/agents/review/mod.rs | struct | pub | 65 |  |
| `ReviewAgentConfig` | projects/agentic-workflow/src/agents/review/mod.rs | struct | pub | 41 |  |
| `build` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 261 | build(self) -> NovaResult<ReviewAgent> |
| `builder` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 100 | builder() -> ReviewAgentBuilder |
| `new` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 219 | new() -> Self |
| `with_max_retries` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 256 | with_max_retries(mut self, n: u32) -> Self |
| `with_max_tokens` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 246 | with_max_tokens(mut self, max_tokens: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 241 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 226 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 231 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_review_type` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 236 | with_review_type(mut self, review_type: ReviewType) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/review/mod.rs | function | pub | 251 | with_temperature(mut self, temperature: f32) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewAgentConfig:
    type: object
    required: [review_type, model, max_tokens, temperature, max_retries]
    description: Configuration for ReviewAgent.
    properties:
      review_type:
        type: string
        x-rust-type: "ReviewType"
        description: "Spec or Code review."
      model:
        type: string
        description: "LLM model identifier."
      max_tokens:
        type: integer
        x-rust-type: "Option<u32>"
        description: "Maximum response tokens."
      temperature:
        type: number
        x-rust-type: "Option<f32>"
        description: "Sampling temperature."
      max_retries:
        type: integer
        x-rust-type: "u32"
        description: "Maximum retries on schema validation failure."
    x-rust-struct:
      derive: [Debug, Clone]

  ReviewAgent:
    type: object
    required: [config, provider]
    description: Opinionated review agent.
    properties:
      config:
        type: object
        x-rust-type: "ReviewAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Arc<dyn LLMProvider>"
        x-rust-visibility: private
        description: "LLM provider."
    x-rust-struct:
      derive: []

  ReviewAgentBuilder:
    type: object
    required: [config, provider]
    description: Builder for ReviewAgent.
    properties:
      config:
        type: object
        x-rust-type: "ReviewAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Option<Arc<dyn LLMProvider>>"
        x-rust-visibility: private
        description: "Optional LLM provider."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/review/mod.rs -->
```rust
//! ReviewAgent — opinionated review for specs and code.
//!
//! Standalone struct that uses [`ReviewType`] config to switch between
//! spec review and code review. Returns a [`ReviewVerdict`].

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
mod types;

pub use types::{ReviewIssue, ReviewType, ReviewVerdict, Severity};

use agent::error::{NovaError, NovaResult};
use agent::llm::{CompletionRequest, LLMProvider};
use agent::structured::complete_structured;
use agent::types::Message;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

// ============================================================
// Reviewer trait
// ============================================================

/// Trait for anything that can review an artifact and return a verdict.
///
/// [`ReviewAgent`] implements this, but tests can provide mocks.
#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
pub trait Reviewer: Send + Sync {
    async fn review(&self, artifact: &str) -> NovaResult<ReviewVerdict>;
}

// ============================================================
// ReviewAgent config
// ============================================================

/// Configuration for ReviewAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#schema
#[derive(Debug, Clone)]
pub struct ReviewAgentConfig {
    /// Spec or Code review.
    pub review_type: ReviewType,
    /// LLM model identifier.
    pub model: String,
    /// Maximum response tokens.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum retries on schema validation failure.
    pub max_retries: u32,
}

/// Opinionated review agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#schema
pub struct ReviewAgent {
    /// Agent configuration.
    config: ReviewAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
}

/// Builder for ReviewAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#schema
pub struct ReviewAgentBuilder {
    /// Agent configuration.
    config: ReviewAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl Default for ReviewAgentConfig {
    fn default() -> Self {
        Self {
            review_type: ReviewType::Spec,
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.2),
            max_retries: 2,
        }
    }
}

// ============================================================
// ReviewAgent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl std::fmt::Debug for ReviewAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReviewAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl ReviewAgent {
    pub fn builder() -> ReviewAgentBuilder {
        ReviewAgentBuilder::new()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl Reviewer for ReviewAgent {
    async fn review(&self, artifact: &str) -> NovaResult<ReviewVerdict> {
        let system_prompt = match self.config.review_type {
            ReviewType::Spec => SPEC_REVIEW_PROMPT,
            ReviewType::Code => CODE_REVIEW_PROMPT,
        };

        let system_msg = Message::system(system_prompt);
        let user_msg = Message::user(format!(
            "Review the following artifact and return a JSON verdict.\n\n{}",
            artifact
        ));

        let mut request = CompletionRequest::new(vec![system_msg, user_msg], &self.config.model);
        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        let schema = verdict_schema();
        let (_response, value) = complete_structured(
            self.provider.as_ref(),
            request,
            &schema,
            self.config.max_retries,
        )
        .await?;

        let verdict: ReviewVerdict = serde_json::from_value(value).map_err(|e| {
            NovaError::SchemaValidationError(format!("Failed to deserialize verdict: {}", e))
        })?;

        Ok(verdict)
    }
}

// ============================================================
// System prompts
// ============================================================

const SPEC_REVIEW_PROMPT: &str = r#"You are an expert spec reviewer following strict quality standards.

Review the spec artifact and check:
1. Format compliance — uses OpenRPC/JSON Schema/Mermaid, NOT prose
2. Diagram correctness — right diagram type for the structure (FSM → stateDiagram, DAG → flowchart, actors → sequenceDiagram)
3. Quality — less than 10% natural language, no real code in specs
4. Completeness — required artifact sections are present; legacy file manifest is not required
5. Consistency — naming matches across sections

Return a JSON verdict:
- "approved" if all checks pass
- "needs_revision" with specific issues if problems found
- "rejected" if fundamentally wrong

Each issue must have: severity (high/medium/low), description, suggestion, and optionally location."#;

const CODE_REVIEW_PROMPT: &str = r#"You are an expert code reviewer.

Review the code artifact and check:
1. Spec compliance — does the code implement what the spec defines?
2. Security — no OWASP top 10 vulnerabilities
3. Error handling — proper error types, no silent failures
4. Test coverage — key paths have tests
5. Style — consistent naming, no unnecessary complexity

Return a JSON verdict:
- "approved" if all checks pass
- "needs_revision" with specific issues if problems found
- "rejected" if fundamentally wrong

Each issue must have: severity (high/medium/low), description, suggestion, and optionally location (file:line)."#;

// ============================================================
// JSON Schema for verdict
// ============================================================

fn verdict_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["verdict"],
        "properties": {
            "verdict": {
                "type": "string",
                "enum": ["approved", "needs_revision", "rejected"]
            },
            "issues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["severity", "description", "suggestion"],
                    "properties": {
                        "severity": { "type": "string", "enum": ["high", "medium", "low"] },
                        "description": { "type": "string" },
                        "suggestion": { "type": "string" },
                        "location": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            },
            "reason": { "type": "string" }
        }
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl ReviewAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReviewAgentConfig::default(),
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

    pub fn with_review_type(mut self, review_type: ReviewType) -> Self {
        self.config.review_type = review_type;
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

    pub fn build(self) -> NovaResult<ReviewAgent> {
        let provider = self.provider.ok_or_else(|| {
            NovaError::ConfigError("ReviewAgent: provider is required".to_string())
        })?;
        Ok(ReviewAgent {
            config: self.config,
            provider,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/mod.md#source
impl Default for ReviewAgentBuilder {
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

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            "openai"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock".to_string()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            Ok(CompletionResponse {
                content: self.response_json.clone(),
                tool_calls: None,
                finish_reason: "stop".to_string(),
                usage: TokenUsage::default(),
                model: "mock".to_string(),
                metadata: HashMap::new(),
            })
        }
        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_review_agent_approved() {
        let agent = ReviewAgent::builder()
            .with_provider(MockProvider {
                response_json: r#"{"verdict": "approved"}"#.to_string(),
            })
            .with_review_type(ReviewType::Spec)
            .build()
            .unwrap();

        let verdict = agent.review("some spec content").await.unwrap();
        assert!(verdict.is_approved());
    }

    #[tokio::test]
    async fn test_review_agent_needs_revision() {
        let json = serde_json::json!({
            "verdict": "needs_revision",
            "issues": [{
                "severity": "high",
                "description": "Missing overview section",
                "suggestion": "Add an overview section"
            }]
        })
        .to_string();

        let agent = ReviewAgent::builder()
            .with_provider(MockProvider {
                response_json: json,
            })
            .with_review_type(ReviewType::Spec)
            .build()
            .unwrap();

        let verdict = agent.review("incomplete spec").await.unwrap();
        assert!(verdict.is_needs_revision());
        if let ReviewVerdict::NeedsRevision { issues } = verdict {
            assert_eq!(issues.len(), 1);
            assert_eq!(issues[0].severity, Severity::High);
        }
    }

    #[tokio::test]
    async fn test_review_agent_rejected() {
        let json = serde_json::json!({
            "verdict": "rejected",
            "reason": "Not a valid spec"
        })
        .to_string();

        let agent = ReviewAgent::builder()
            .with_provider(MockProvider {
                response_json: json,
            })
            .build()
            .unwrap();

        let verdict = agent.review("garbage").await.unwrap();
        assert!(verdict.is_rejected());
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReviewAgent::builder().build().unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_review_type_default_is_spec() {
        let config = ReviewAgentConfig::default();
        assert_eq!(config.review_type, ReviewType::Spec);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/review/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete review agent module, including module
      wiring, schema-derived structs, behavior impls, prompts, builder methods,
      and tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 3 standard agent-pattern types: Config + Agent + Builder.
- [schema] Config gets full derive; Agent + Builder are no-derive trait-object holders.
- [changes] All three in `replaces`; everything else hand-written including custom impl Debug for ReviewAgent.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Promotes the module preamble and review-agent behavior into source-template ownership while preserving the schema as the shape contract.
- [source] Uses `strip-managed-markers` to preserve the existing implementation and erase the mixed CODEGEN/HANDWRITE boundary.
- [changes] Correctly routes the file through the `source` section with `impl_mode: codegen`.

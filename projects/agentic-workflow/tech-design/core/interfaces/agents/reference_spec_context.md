---
id: sdd-agents-reference-spec-context
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Reference Spec Context Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/reference_spec_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Contradiction` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 55 |  |
| `ReferenceContextOutput` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 69 |  |
| `ReferenceSpecContextAgent` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 94 |  |
| `ReferenceSpecContextAgentBuilder` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 107 |  |
| `ReferenceSpecContextAgentConfig` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 79 |  |
| `RelevanceLevel` | projects/agentic-workflow/src/agents/reference_spec_context.rs | enum | pub | 32 |  |
| `SpecReferenceEntry` | projects/agentic-workflow/src/agents/reference_spec_context.rs | struct | pub | 41 |  |
| `build` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 466 | build(self) -> NovaResult<ReferenceSpecContextAgent> |
| `builder` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 185 | builder() -> ReferenceSpecContextAgentBuilder |
| `new` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 402 | new() -> Self |
| `with_max_retries` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 461 | with_max_retries(mut self, n: u32) -> Self |
| `with_max_spec_excerpts` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 456 | with_max_spec_excerpts(mut self, n: usize) -> Self |
| `with_max_tokens` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 446 | with_max_tokens(mut self, max_tokens: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 441 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 411 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 416 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_reviewer` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 431 | with_reviewer(mut self, reviewer: R) -> Self |
| `with_reviewer_arc` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 436 | with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self |
| `with_spec_store` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 421 | with_spec_store(mut self, store: S) -> Self |
| `with_spec_store_arc` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 426 | with_spec_store_arc(mut self, store: Arc<dyn SpecStore>) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/reference_spec_context.rs | function | pub | 451 | with_temperature(mut self, temperature: f32) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RelevanceLevel:
    type: string
    enum: [High, Medium, Low]
    description: Relevance level.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]
      serde_rename_all: lowercase

  SpecReferenceEntry:
    type: object
    required: [spec_id, spec_group, relevance, key_requirements]
    description: A single spec entry in the reference context artifact.
    properties:
      spec_id:
        type: string
        description: "Path relative to .aw/tech-design/."
      spec_group:
        type: string
        description: "Logical group name."
      relevance:
        type: string
        x-rust-type: "RelevanceLevel"
        description: "Relevance of this spec to the change."
      key_requirements:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Key requirements from this spec that apply."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  Contradiction:
    type: object
    required: [spec_id, requirement, conflict, resolution]
    description: A contradiction detected between an existing spec and the change.
    properties:
      spec_id:
        type: string
        description: "Spec where the contradiction was found."
      requirement:
        type: string
        description: "The change requirement that conflicts."
      conflict:
        type: string
        description: "Description of the conflict."
      resolution:
        type: string
        description: "Suggested resolution."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  ReferenceContextOutput:
    type: object
    required: [specs, contradictions]
    description: Output of the reference spec context agent.
    properties:
      specs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecReferenceEntry>"
        description: "Spec references."
      contradictions:
        type: array
        items: { type: object }
        x-rust-type: "Vec<Contradiction>"
        description: "Contradictions between specs and the change."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  ReferenceSpecContextAgentConfig:
    type: object
    required: [model, max_tokens, temperature, max_spec_excerpts, max_retries]
    description: Configuration for ReferenceSpecContextAgent.
    properties:
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
      max_spec_excerpts:
        type: integer
        x-rust-type: "usize"
        description: "Maximum specs to fetch and read in full."
      max_retries:
        type: integer
        x-rust-type: "u32"
        description: "Structured-output retries on validation failure."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReferenceSpecContextAgent:
    type: object
    required: [config, provider, spec_store, reviewer]
    description: Spec reference context agent.
    properties:
      config:
        type: object
        x-rust-type: "ReferenceSpecContextAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Arc<dyn LLMProvider>"
        x-rust-visibility: private
        description: "LLM provider."
      spec_store:
        type: object
        x-rust-type: "Arc<dyn SpecStore>"
        x-rust-visibility: private
        description: "Spec store."
      reviewer:
        type: object
        x-rust-type: "Arc<dyn Reviewer>"
        x-rust-visibility: private
        description: "Reviewer for output validation."
    x-rust-struct:
      derive: []

  ReferenceSpecContextAgentBuilder:
    type: object
    required: [config, provider, spec_store, reviewer]
    description: Builder for ReferenceSpecContextAgent.
    properties:
      config:
        type: object
        x-rust-type: "ReferenceSpecContextAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Option<Arc<dyn LLMProvider>>"
        x-rust-visibility: private
        description: "Optional LLM provider."
      spec_store:
        type: object
        x-rust-type: "Option<Arc<dyn SpecStore>>"
        x-rust-visibility: private
        description: "Optional spec store."
      reviewer:
        type: object
        x-rust-type: "Option<Arc<dyn Reviewer>>"
        x-rust-visibility: private
        description: "Optional reviewer."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/reference_spec_context.rs -->
````rust
//! ReferenceSpecContextAgent — discovers and synthesizes spec context for SDD.
//!
//! Operates during SDD phases 4 (reference-context) and 5 (post-clarification).
//! Searches the `SpecStore`, reads full spec content, scores each spec's relevance,
//! extracts key requirements, detects contradictions, and runs an internal CRR cycle
//! (max_revisions=1, auto-approve semantics) via `ReviewAgent` before finalizing.

use crate::agents::restructure::{SpecExcerpt, SpecStore};
use crate::agents::review::{ReviewIssue, ReviewVerdict, Reviewer};
use crate::agents::Agent;
use agent::error::{NovaError, NovaResult};
use agent::llm::{CompletionRequest, LLMProvider};
use agent::stream::StreamHandler;
use agent::structured::complete_structured;
use agent::types::Message;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

// ============================================================
// Output types
// ============================================================

use serde::{Deserialize, Serialize};

/// Relevance level.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelevanceLevel {
    High,
    Medium,
    Low,
}

/// A single spec entry in the reference context artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecReferenceEntry {
    /// Path relative to .aw/tech-design/.
    pub spec_id: String,
    /// Logical group name.
    pub spec_group: String,
    /// Relevance of this spec to the change.
    pub relevance: RelevanceLevel,
    /// Key requirements from this spec that apply.
    pub key_requirements: Vec<String>,
}

/// A contradiction detected between an existing spec and the change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contradiction {
    /// Spec where the contradiction was found.
    pub spec_id: String,
    /// The change requirement that conflicts.
    pub requirement: String,
    /// Description of the conflict.
    pub conflict: String,
    /// Suggested resolution.
    pub resolution: String,
}

/// Output of the reference spec context agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceContextOutput {
    /// Spec references.
    pub specs: Vec<SpecReferenceEntry>,
    /// Contradictions between specs and the change.
    pub contradictions: Vec<Contradiction>,
}

/// Configuration for ReferenceSpecContextAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSpecContextAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Maximum response tokens.
    pub max_tokens: Option<u32>,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum specs to fetch and read in full.
    pub max_spec_excerpts: usize,
    /// Structured-output retries on validation failure.
    pub max_retries: u32,
}

/// Spec reference context agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
pub struct ReferenceSpecContextAgent {
    /// Agent configuration.
    config: ReferenceSpecContextAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
    /// Spec store.
    spec_store: Arc<dyn SpecStore>,
    /// Reviewer for output validation.
    reviewer: Arc<dyn Reviewer>,
}

/// Builder for ReferenceSpecContextAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#schema
pub struct ReferenceSpecContextAgentBuilder {
    /// Agent configuration.
    config: ReferenceSpecContextAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
    /// Optional spec store.
    spec_store: Option<Arc<dyn SpecStore>>,
    /// Optional reviewer.
    reviewer: Option<Arc<dyn Reviewer>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#changes
// ============================================================
// Agent config
// ============================================================

impl Default for ReferenceSpecContextAgentConfig {
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

// ============================================================
// Agent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl std::fmt::Debug for ReferenceSpecContextAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceSpecContextAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Agent for ReferenceSpecContextAgent {
    async fn run(&self, input: &str) -> NovaResult<String> {
        // 1. Search for relevant specs
        let excerpts = self.spec_store.search(input).await?;
        let excerpt_count = excerpts.len().min(self.config.max_spec_excerpts);
        let excerpts = &excerpts[..excerpt_count];

        // 2. Read full text of each spec
        let mut full_specs: Vec<(SpecExcerpt, String)> = Vec::new();
        for excerpt in excerpts {
            let content = self.spec_store.read(&excerpt.path).await?;
            full_specs.push((excerpt.clone(), content));
        }

        // 3. Generate initial reference context
        let artifact = self.generate_context(input, &full_specs).await?;

        // 4. CRR cycle (max_revisions=1, auto-approve)
        let final_artifact = self.crr_cycle(input, &full_specs, artifact).await?;

        Ok(final_artifact)
    }

    async fn run_with_handler(
        &self,
        input: &str,
        _handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        self.run(input).await
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgent {
    /// Create a new builder.
    pub fn builder() -> ReferenceSpecContextAgentBuilder {
        ReferenceSpecContextAgentBuilder::new()
    }

    /// Generate the initial reference context JSON from the change input + spec contents.
    async fn generate_context(
        &self,
        input: &str,
        full_specs: &[(SpecExcerpt, String)],
    ) -> NovaResult<String> {
        let (system_msg, user_msg) = build_prompt(input, full_specs);
        let artifact_json = self
            .complete_structured_output(vec![system_msg, user_msg])
            .await?;
        Ok(artifact_json)
    }

    /// Run an internal CRR cycle with max_revisions=1 and auto-approve semantics.
    ///
    /// If the reviewer flags issues, we revise once and accept the result regardless
    /// of the second verdict (auto-approve). If the artifact is `Rejected`, we still
    /// return it rather than failing.
    async fn crr_cycle(
        &self,
        input: &str,
        full_specs: &[(SpecExcerpt, String)],
        initial_artifact: String,
    ) -> NovaResult<String> {
        let verdict = self.reviewer.review(&initial_artifact).await?;

        match verdict {
            // Accept immediately — approved or auto-approve on rejection
            ReviewVerdict::Approved | ReviewVerdict::Rejected { .. } => Ok(initial_artifact),

            // Revise once, then auto-approve regardless of the second verdict
            ReviewVerdict::NeedsRevision { issues } => {
                let revision_prompt =
                    build_revision_prompt(input, full_specs, &initial_artifact, &issues);
                let revised = self
                    .complete_structured_output(vec![
                        Message::system(SYSTEM_PROMPT),
                        Message::user(revision_prompt),
                    ])
                    .await?;
                Ok(revised)
            }
        }
    }

    /// Call the LLM with structured output and return the pretty-printed JSON string.
    async fn complete_structured_output(&self, messages: Vec<Message>) -> NovaResult<String> {
        let mut request = CompletionRequest::new(messages, &self.config.model);
        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }
        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        let schema = output_schema();
        let (_response, value) = complete_structured(
            self.provider.as_ref(),
            request,
            &schema,
            self.config.max_retries,
        )
        .await?;

        // Validate round-trip to ReferenceContextOutput
        let _: ReferenceContextOutput = serde_json::from_value(value.clone()).map_err(|e| {
            NovaError::SchemaValidationError(format!("Failed to deserialize output: {}", e))
        })?;

        serde_json::to_string_pretty(&value)
            .map_err(|e| NovaError::Other(anyhow::anyhow!("Failed to serialize output: {}", e)))
    }
}

// ============================================================
// Prompt assembly
// ============================================================

const SYSTEM_PROMPT: &str = r#"You are an expert specification analyst for spec-driven development (SDD).

Your task:
1. Analyze the provided change requirements against the existing project specifications.
2. For each spec assign a relevance score:
   - "high"   — directly impacts the change (same module, same interface)
   - "medium" — adjacent context (related module, shared type)
   - "low"    — tangentially related (general framework, distant dependency)
3. Extract concise key requirements from high/medium-relevance specs applicable to the change.
4. Identify contradictions between existing specs and the proposed change requirements.

Output MUST be a JSON object matching the provided schema exactly."#;

fn build_prompt(input: &str, full_specs: &[(SpecExcerpt, String)]) -> (Message, Message) {
    let system_msg = Message::system(SYSTEM_PROMPT);

    let mut content = format!("## Change Requirements\n\n{}\n\n", input);

    if !full_specs.is_empty() {
        content.push_str("## Existing Specifications\n\n");
        for (excerpt, full_content) in full_specs {
            content.push_str(&format!(
                "### `{}` (search relevance: {:.2})\n\n```\n{}\n```\n\n",
                excerpt.path, excerpt.relevance, full_content
            ));
        }
    } else {
        content.push_str("## Existing Specifications\n\n(No matching specs found)\n\n");
    }

    content.push_str(
        "Analyze the specifications against the change requirements \
         and produce a JSON reference context.",
    );

    (system_msg, Message::user(content))
}

fn build_revision_prompt(
    input: &str,
    full_specs: &[(SpecExcerpt, String)],
    artifact: &str,
    issues: &[ReviewIssue],
) -> String {
    let mut prompt = format!(
        "## Change Requirements\n\n{}\n\n\
         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

         ## Original Reference Context\n\n{}\n\n\
         ## Review Issues\n",
        input, artifact
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

    if !full_specs.is_empty() {
        prompt.push_str("\n## Existing Specifications (for reference)\n\n");
        for (excerpt, full_content) in full_specs {
            prompt.push_str(&format!(
                "### `{}`\n\n```\n{}\n```\n\n",
                excerpt.path, full_content
            ));
        }
    }

    prompt.push_str(
        "\nAddress every review issue above and return \
         the fully revised reference context JSON.",
    );
    prompt
}

// ============================================================
// JSON Schema for structured output
// ============================================================

fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "required": ["specs", "contradictions"],
        "properties": {
            "specs": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "spec_group", "relevance", "key_requirements"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "spec_group": { "type": "string" },
                        "relevance": {
                            "type": "string",
                            "enum": ["high", "medium", "low"]
                        },
                        "key_requirements": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    },
                    "additionalProperties": false
                }
            },
            "contradictions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["spec_id", "requirement", "conflict", "resolution"],
                    "properties": {
                        "spec_id": { "type": "string" },
                        "requirement": { "type": "string" },
                        "conflict": { "type": "string" },
                        "resolution": { "type": "string" }
                    },
                    "additionalProperties": false
                }
            }
        },
        "additionalProperties": false
    })
}

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl ReferenceSpecContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceSpecContextAgentConfig::default(),
            provider: None,
            spec_store: None,
            reviewer: None,
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

    pub fn with_reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    pub fn with_reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
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

    pub fn build(self) -> NovaResult<ReferenceSpecContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let spec_store = self
            .spec_store
            .ok_or_else(|| NovaError::ConfigError("SpecStore is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("Reviewer is required".to_string()))?;
        Ok(ReferenceSpecContextAgent {
            config: self.config,
            provider,
            spec_store,
            reviewer,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_spec_context.md#source
impl Default for ReferenceSpecContextAgentBuilder {
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
    use crate::agents::restructure::SpecExcerpt;
    use agent::llm::{CompletionRequest, CompletionResponse, StreamResponse};
    use agent::types::TokenUsage;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ---- Mock LLM ----

    struct MockProvider {
        response_json: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        fn provider_name(&self) -> &str {
            // "openai" triggers direct JSON parsing in complete_structured
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

    // ---- Mock SpecStore ----

    struct MockSpecStore {
        excerpts: Vec<SpecExcerpt>,
        read_content: String,
        read_paths: Arc<Mutex<Vec<String>>>,
    }

    impl MockSpecStore {
        fn new(excerpts: Vec<SpecExcerpt>, read_content: &str) -> Self {
            Self {
                excerpts,
                read_content: read_content.to_string(),
                read_paths: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl SpecStore for MockSpecStore {
        async fn search(&self, _query: &str) -> NovaResult<Vec<SpecExcerpt>> {
            Ok(self.excerpts.clone())
        }

        async fn read(&self, path: &str) -> NovaResult<String> {
            self.read_paths.lock().unwrap().push(path.to_string());
            Ok(self.read_content.clone())
        }
    }

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

    // ---- Mock Reviewer ----

    struct ScriptedReviewer {
        verdicts: Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn always_approve() -> Self {
            Self {
                verdicts: Mutex::new(vec![]),
            }
        }

        fn sequence(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    // ---- Helpers ----

    fn valid_context_json() -> String {
        serde_json::json!({
            "specs": [
                {
                    "spec_id": "cclab-agent/spec.md",
                    "spec_group": "cclab-agent",
                    "relevance": "high",
                    "key_requirements": ["SpecStore must implement read()", "Agent trait must be implemented"]
                }
            ],
            "contradictions": []
        })
        .to_string()
    }

    fn make_review_issue(msg: &str) -> ReviewIssue {
        use crate::agents::review::Severity;
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "Fix it.".to_string(),
            location: None,
        }
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_approved_on_first_review() {
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
        assert_eq!(output.specs[0].relevance, RelevanceLevel::High);
        assert!(output.contradictions.is_empty());
    }

    #[tokio::test]
    async fn test_run_triggers_revision_on_needs_revision() {
        // Reviewer returns NeedsRevision once, then auto-approve kicks in.
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_review_issue("Missing medium-relevance spec")],
                },
            ]))
            .build()
            .unwrap();

        // Should succeed (auto-approve after one revision attempt)
        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_auto_approves_on_rejection() {
        // Rejected verdict → auto-approve, returns initial artifact
        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::sequence(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .build()
            .unwrap();

        let result = agent.run("Add OAuth2 support").await.unwrap();
        let output: ReferenceContextOutput = serde_json::from_str(&result).unwrap();
        assert_eq!(output.specs.len(), 1);
    }

    #[tokio::test]
    async fn test_spec_store_read_called_for_each_excerpt() {
        let excerpts = vec![
            SpecExcerpt {
                path: "specs/auth.md".to_string(),
                content: "Auth excerpt".to_string(),
                relevance: 0.9,
            },
            SpecExcerpt {
                path: "specs/oauth.md".to_string(),
                content: "OAuth excerpt".to_string(),
                relevance: 0.7,
            },
        ];
        let store = MockSpecStore::new(excerpts, "Full spec content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap();

        agent.run("Add OAuth2 support").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"specs/auth.md".to_string()));
        assert!(paths.contains(&"specs/oauth.md".to_string()));
    }

    #[tokio::test]
    async fn test_max_spec_excerpts_limit() {
        let excerpts: Vec<SpecExcerpt> = (0..5)
            .map(|i| SpecExcerpt {
                path: format!("specs/spec{}.md", i),
                content: format!("content {}", i),
                relevance: 0.5,
            })
            .collect();
        let store = MockSpecStore::new(excerpts, "Full content");
        let read_paths = store.read_paths.clone();

        let agent = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: valid_context_json(),
            })
            .with_spec_store(store)
            .with_reviewer(ScriptedReviewer::always_approve())
            .with_max_spec_excerpts(3) // limit to 3
            .build()
            .unwrap();

        agent.run("Some change").await.unwrap();

        let paths = read_paths.lock().unwrap();
        assert_eq!(paths.len(), 3, "should only read up to max_spec_excerpts");
    }

    #[tokio::test]
    async fn test_output_round_trips_all_relevance_levels() {
        let output = ReferenceContextOutput {
            specs: vec![
                SpecReferenceEntry {
                    spec_id: "a.md".to_string(),
                    spec_group: "g1".to_string(),
                    relevance: RelevanceLevel::High,
                    key_requirements: vec!["req1".to_string()],
                },
                SpecReferenceEntry {
                    spec_id: "b.md".to_string(),
                    spec_group: "g2".to_string(),
                    relevance: RelevanceLevel::Medium,
                    key_requirements: vec![],
                },
                SpecReferenceEntry {
                    spec_id: "c.md".to_string(),
                    spec_group: "g3".to_string(),
                    relevance: RelevanceLevel::Low,
                    key_requirements: vec![],
                },
            ],
            contradictions: vec![Contradiction {
                spec_id: "a.md".to_string(),
                requirement: "must use async".to_string(),
                conflict: "existing spec is sync".to_string(),
                resolution: "migrate to async_trait".to_string(),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["specs"][0]["relevance"], "high");
        assert_eq!(json["specs"][1]["relevance"], "medium");
        assert_eq!(json["specs"][2]["relevance"], "low");
        assert_eq!(json["contradictions"][0]["spec_id"], "a.md");

        let parsed: ReferenceContextOutput = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, output);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceSpecContextAgent::builder()
            .with_spec_store(EmptySpecStore)
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_missing_spec_store() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_reviewer(ScriptedReviewer::always_approve())
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("SpecStore"));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = ReferenceSpecContextAgent::builder()
            .with_provider(MockProvider {
                response_json: "{}".to_string(),
            })
            .with_spec_store(EmptySpecStore)
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("Reviewer"));
    }

    #[test]
    fn test_relevance_level_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(RelevanceLevel::High).unwrap(),
            serde_json::json!("high")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Medium).unwrap(),
            serde_json::json!("medium")
        );
        assert_eq!(
            serde_json::to_value(RelevanceLevel::Low).unwrap(),
            serde_json::json!("low")
        );
    }

    #[test]
    fn test_empty_context_output_is_valid() {
        let output = ReferenceContextOutput {
            specs: vec![],
            contradictions: vec![],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert!(json["specs"].is_array());
        assert!(json["contradictions"].is_array());
    }
}
````

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/reference_spec_context.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete reference-spec-context agent module,
      including generated data shapes, defaults, SpecStore search/read flow,
      structured output validation, internal CRR loop, builder methods, and
      unit tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 7 types: enum + 3 data carriers + Config + Agent + Builder.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split.

## Traceability Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
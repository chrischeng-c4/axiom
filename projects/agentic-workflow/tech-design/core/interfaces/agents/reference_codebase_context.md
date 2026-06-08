---
id: sdd-agents-reference-codebase-context
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Reference Codebase Context Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/reference_codebase_context.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodebaseDependency` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 53 |  |
| `ComponentRelationship` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 65 |  |
| `KeyFile` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 41 |  |
| `ReferenceCodebaseArtifact` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 77 |  |
| `ReferenceCodebaseContextAgent` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 108 |  |
| `ReferenceCodebaseContextAgentBuilder` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 119 |  |
| `ReferenceCodebaseContextAgentConfig` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | struct | pub | 95 |  |
| `build` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 407 | build(self) -> NovaResult<ReferenceCodebaseContextAgent> |
| `builder` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 216 | builder() -> ReferenceCodebaseContextAgentBuilder |
| `new` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 359 | new() -> Self |
| `with_max_retries` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 402 | with_max_retries(mut self, n: u32) -> Self |
| `with_max_turns` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 397 | with_max_turns(mut self, max_turns: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 387 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 367 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 372 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_registry` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 377 | with_registry(mut self, registry: ToolRegistry) -> Self |
| `with_registry_arc` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 382 | with_registry_arc(mut self, registry: Arc<ToolRegistry>) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/reference_codebase_context.rs | function | pub | 392 | with_temperature(mut self, temperature: f32) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  KeyFile:
    type: object
    required: [path, purpose, key_exports]
    description: A key file identified in the codebase.
    properties:
      path:
        type: string
        description: "Relative path."
      purpose:
        type: string
        description: "Purpose or role."
      key_exports:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Key exports."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  CodebaseDependency:
    type: object
    required: [name, dependency_type, purpose]
    description: A codebase dependency.
    properties:
      name:
        type: string
        description: "Dependency name."
      dependency_type:
        type: string
        description: "internal or external."
      purpose:
        type: string
        description: "Purpose."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  ComponentRelationship:
    type: object
    required: [from, to, relationship_type]
    description: A directed relationship between components.
    properties:
      from:
        type: string
        description: "Source component."
      to:
        type: string
        description: "Target component."
      relationship_type:
        type: string
        description: "Relationship kind."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  ReferenceCodebaseArtifact:
    type: object
    required: [target, key_files, architectural_patterns, dependencies, relationships, summary]
    description: Structured codebase context artifact.
    properties:
      target:
        type: string
        description: "The target path or component analyzed."
      key_files:
        type: array
        items: { type: object }
        x-rust-type: "Vec<KeyFile>"
        description: "Key files identified."
      architectural_patterns:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Architectural patterns observed."
      dependencies:
        type: array
        items: { type: object }
        x-rust-type: "Vec<CodebaseDependency>"
        description: "Dependencies identified."
      relationships:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ComponentRelationship>"
        description: "Component relationships."
      summary:
        type: string
        description: "Concise summary."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize]

  ReferenceCodebaseContextAgentConfig:
    type: object
    required: [model, temperature, max_turns, max_retries]
    description: Configuration for ReferenceCodebaseContextAgent.
    properties:
      model:
        type: string
        description: "LLM model identifier."
      temperature:
        type: number
        x-rust-type: "Option<f32>"
        description: "Sampling temperature."
      max_turns:
        type: integer
        x-rust-type: "u32"
        description: "Maximum exploration turns."
      max_retries:
        type: integer
        x-rust-type: "u32"
        description: "Retries on JSON validation failure."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ReferenceCodebaseContextAgent:
    type: object
    required: [config, provider, registry]
    description: Agent that explores codebase.
    properties:
      config:
        type: object
        x-rust-type: "ReferenceCodebaseContextAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Arc<dyn LLMProvider>"
        x-rust-visibility: private
        description: "LLM provider."
      registry:
        type: object
        x-rust-type: "Arc<ToolRegistry>"
        x-rust-visibility: private
        description: "Tool registry."
    x-rust-struct:
      derive: []

  ReferenceCodebaseContextAgentBuilder:
    type: object
    required: [config, provider, registry]
    description: Builder for ReferenceCodebaseContextAgent.
    properties:
      config:
        type: object
        x-rust-type: "ReferenceCodebaseContextAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Option<Arc<dyn LLMProvider>>"
        x-rust-visibility: private
        description: "Optional LLM provider."
      registry:
        type: object
        x-rust-type: "Option<Arc<ToolRegistry>>"
        x-rust-visibility: private
        description: "Optional tool registry."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/reference_codebase_context.rs -->
~~~rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
// CODEGEN-BEGIN
//! ReferenceCodebaseContextAgent — explores the codebase and extracts structured context.
//!
//! Operates during the SDD fillback flow, parallel to what `ReferenceSpecContextAgent`
//! does for specs. Uses coding tools (ReadFile, Glob, Grep, Bash) to autonomously
//! explore a target codebase path, then produces a structured [`ReferenceCodebaseArtifact`]
//! describing key files, architectural patterns, dependencies, and component relationships.
//!
//! # Flow
//!
//! 1. An inner [`CodingAgent`] is instantiated with the provided tool registry and a
//!    specialized exploration system prompt.
//! 2. The inner agent explores the codebase using its tools, then emits a JSON summary
//!    as its final response.
//! 3. JSON is extracted from the response (handling markdown code fences), validated
//!    against [`ReferenceCodebaseArtifact`], and returned.
//! 4. On validation failure the inner agent is re-invoked with a corrective prompt,
//!    up to `max_retries` times.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
use crate::agents::coding::CodingAgent;
use crate::agents::Agent;
use agent::error::{NovaError, NovaResult};
use agent::stream::StreamHandler;
use agent::tools::ToolRegistry;
use async_trait::async_trait;
use std::sync::Arc;

use agent::llm::LLMProvider;

// ============================================================
// Output types
// ============================================================

use serde::{Deserialize, Serialize};

/// A key file identified in the codebase.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyFile {
    /// Relative path.
    pub path: String,
    /// Purpose or role.
    pub purpose: String,
    /// Key exports.
    pub key_exports: Vec<String>,
}

/// A codebase dependency.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodebaseDependency {
    /// Dependency name.
    pub name: String,
    /// internal or external.
    pub dependency_type: String,
    /// Purpose.
    pub purpose: String,
}

/// A directed relationship between components.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentRelationship {
    /// Source component.
    pub from: String,
    /// Target component.
    pub to: String,
    /// Relationship kind.
    pub relationship_type: String,
}

/// Structured codebase context artifact.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceCodebaseArtifact {
    /// The target path or component analyzed.
    pub target: String,
    /// Key files identified.
    pub key_files: Vec<KeyFile>,
    /// Architectural patterns observed.
    pub architectural_patterns: Vec<String>,
    /// Dependencies identified.
    pub dependencies: Vec<CodebaseDependency>,
    /// Component relationships.
    pub relationships: Vec<ComponentRelationship>,
    /// Concise summary.
    pub summary: String,
}

/// Configuration for ReferenceCodebaseContextAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCodebaseContextAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum exploration turns.
    pub max_turns: u32,
    /// Retries on JSON validation failure.
    pub max_retries: u32,
}

/// Agent that explores codebase.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
pub struct ReferenceCodebaseContextAgent {
    /// Agent configuration.
    config: ReferenceCodebaseContextAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
    /// Tool registry.
    registry: Arc<ToolRegistry>,
}

/// Builder for ReferenceCodebaseContextAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#schema
pub struct ReferenceCodebaseContextAgentBuilder {
    /// Agent configuration.
    config: ReferenceCodebaseContextAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
    /// Optional tool registry.
    registry: Option<Arc<ToolRegistry>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
// ============================================================
// Agent config
// ============================================================

impl Default for ReferenceCodebaseContextAgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.0),
            max_turns: 30,
            max_retries: 2,
        }
    }
}

// ============================================================
// Agent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
impl std::fmt::Debug for ReferenceCodebaseContextAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceCodebaseContextAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
impl Agent for ReferenceCodebaseContextAgent {
    async fn run(&self, input: &str) -> NovaResult<String> {
        let mut prompt = format!(
            "Explore the following target and produce a structured codebase context artifact.\n\n\
             Target: {}\n\n\
             Use your tools to read files, glob patterns, grep for symbols, and inspect \
             the codebase. When you have gathered sufficient context, output ONLY the JSON \
             artifact as described in your system prompt — no other text.",
            input
        );

        for attempt in 0..=self.config.max_retries {
            let inner = self.build_inner_agent()?;
            let raw_output = inner.run(&prompt).await?;

            match Self::extract_and_validate(&raw_output) {
                Ok(artifact_json) => return Ok(artifact_json),
                Err(e) => {
                    if attempt < self.config.max_retries {
                        // Build a corrective prompt that includes the invalid output
                        prompt = format!(
                            "Your previous response was not valid JSON conforming to \
                             ReferenceCodebaseArtifact (error: {}).\n\n\
                             Previous response:\n{}\n\n\
                             Please explore the target again and output ONLY the JSON artifact.",
                            e, raw_output
                        );
                    } else {
                        return Err(NovaError::Other(anyhow::anyhow!(
                            "ReferenceCodebaseContextAgent: failed after {} retries. \
                             Last validation error: {}",
                            self.config.max_retries,
                            e
                        )));
                    }
                }
            }
        }

        // Unreachable, but satisfies the compiler
        Err(NovaError::Other(anyhow::anyhow!(
            "ReferenceCodebaseContextAgent: exhausted retries"
        )))
    }

    async fn run_with_handler(
        &self,
        input: &str,
        _handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        self.run(input).await
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
impl ReferenceCodebaseContextAgent {
    /// Create a new builder.
    pub fn builder() -> ReferenceCodebaseContextAgentBuilder {
        ReferenceCodebaseContextAgentBuilder::new()
    }

    /// Build the inner [`CodingAgent`] used for codebase exploration.
    fn build_inner_agent(&self) -> NovaResult<CodingAgent> {
        CodingAgent::builder()
            .with_provider_arc(self.provider.clone())
            .with_registry_arc(self.registry.clone())
            .with_system_prompt(EXPLORATION_SYSTEM_PROMPT)
            .with_max_turns(self.config.max_turns)
            .with_temperature(0.0)
            .with_model(&self.config.model)
            .build()
    }

    /// Extract a JSON object from the agent's raw response text and validate it as
    /// [`ReferenceCodebaseArtifact`]. Returns pretty-printed JSON on success.
    fn extract_and_validate(text: &str) -> NovaResult<String> {
        let json_str = Self::extract_json_object(text).ok_or_else(|| {
            NovaError::Other(anyhow::anyhow!("No JSON object found in agent response"))
        })?;

        let artifact: ReferenceCodebaseArtifact = serde_json::from_str(&json_str).map_err(|e| {
            NovaError::SchemaValidationError(format!(
                "ReferenceCodebaseArtifact deserialization failed: {}",
                e
            ))
        })?;

        serde_json::to_string_pretty(&artifact)
            .map_err(|e| NovaError::Other(anyhow::anyhow!("Serialization error: {}", e)))
    }

    /// Extract the first JSON object from text, handling markdown code fences.
    fn extract_json_object(text: &str) -> Option<String> {
        // 1. Try ```json ... ``` fence
        if let Some(fence_start) = text.find("```json") {
            let after_fence = &text[fence_start + 7..];
            // Skip optional newline after fence marker
            let content_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
            let content = &after_fence[content_start..];
            if let Some(end) = content.find("```") {
                return Some(content[..end].trim().to_string());
            }
        }

        // 2. Try generic ``` ... ``` fence that starts with '{'
        if let Some(fence_start) = text.find("```\n") {
            let content = &text[fence_start + 4..];
            if content.trim_start().starts_with('{') {
                if let Some(end) = content.find("\n```") {
                    return Some(content[..end].trim().to_string());
                }
            }
        }

        // 3. Find the outermost { ... } by matching braces
        let bytes = text.as_bytes();
        let mut brace_start = None;
        let mut depth = 0usize;
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'{' => {
                    if depth == 0 {
                        brace_start = Some(i);
                    }
                    depth += 1;
                }
                b'}' if depth > 0 => {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(start) = brace_start {
                            return Some(text[start..=i].to_string());
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }
}

// ============================================================
// System prompt
// ============================================================

const EXPLORATION_SYSTEM_PROMPT: &str = r#"You are an expert software architect performing codebase exploration for spec-driven development (SDD).

Your goal is to systematically explore the target codebase path using your tools, then output a structured JSON context artifact.

## Exploration Strategy

1. Start with directory listing and file discovery (glob patterns).
2. Read key files: entry points (lib.rs, mod.rs, main.rs), trait definitions, public API files.
3. Use grep to locate important symbols: trait implementations, public types, re-exports.
4. Identify architectural patterns: builder pattern, trait objects, actor model, etc.
5. Map dependencies: Cargo.toml for external crates; use statements for internal modules.
6. Trace component relationships: which modules depend on which, how types flow.

## Output Format

After exploration, output ONLY this JSON object — no other text before or after:

```json
{
  "target": "<analyzed path or component name>",
  "key_files": [
    {
      "path": "<relative path from crate root>",
      "purpose": "<role of this file>",
      "key_exports": ["<type or function name>", ...]
    }
  ],
  "architectural_patterns": ["<pattern name>", ...],
  "dependencies": [
    {
      "name": "<crate, package, or module>",
      "dependency_type": "internal | external",
      "purpose": "<why it is used>"
    }
  ],
  "relationships": [
    {
      "from": "<module or type>",
      "to": "<module or type>",
      "relationship_type": "implements | uses | depends_on | extends | wraps"
    }
  ],
  "summary": "<2-3 sentence architectural summary>"
}
```

Do NOT output any explanation, prose, or markdown outside of the JSON block."#;

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
impl ReferenceCodebaseContextAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: ReferenceCodebaseContextAgentConfig::default(),
            provider: None,
            registry: None,
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

    pub fn with_registry(mut self, registry: ToolRegistry) -> Self {
        self.registry = Some(Arc::new(registry));
        self
    }

    pub fn with_registry_arc(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.config.max_turns = max_turns;
        self
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.config.max_retries = n;
        self
    }

    pub fn build(self) -> NovaResult<ReferenceCodebaseContextAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
        let registry = self
            .registry
            .unwrap_or_else(|| Arc::new(ToolRegistry::new()));
        Ok(ReferenceCodebaseContextAgent {
            config: self.config,
            provider,
            registry,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/reference_codebase_context.md#source
impl Default for ReferenceCodebaseContextAgentBuilder {
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
    use async_trait::async_trait;
    use std::collections::HashMap;

    // ---- Mock LLM that returns JSON without any tool calls ----

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
                tool_calls: None, // No tool calls → CodingAgent finishes immediately
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

    fn valid_artifact_json() -> String {
        serde_json::json!({
            "target": "crates/cclab-agent/src/agents",
            "key_files": [
                {
                    "path": "crates/cclab-agent/src/agents/mod.rs",
                    "purpose": "Module entry point, Agent trait definition",
                    "key_exports": ["Agent", "ApprovalHandler"]
                }
            ],
            "architectural_patterns": ["trait objects", "builder pattern"],
            "dependencies": [
                {
                    "name": "async_trait",
                    "dependency_type": "external",
                    "purpose": "Async trait support"
                }
            ],
            "relationships": [
                {
                    "from": "CodingAgent",
                    "to": "Agent",
                    "relationship_type": "implements"
                }
            ],
            "summary": "The agents module defines the Agent trait and provides multiple implementations for different SDD workflow phases."
        })
        .to_string()
    }

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_returns_valid_artifact() {
        let agent = ReferenceCodebaseContextAgent::builder()
            .with_provider(MockProvider {
                response: valid_artifact_json(),
            })
            .build()
            .unwrap();

        let result = agent.run("crates/cclab-agent/src/agents").await.unwrap();
        let artifact: ReferenceCodebaseArtifact = serde_json::from_str(&result).unwrap();
        assert_eq!(artifact.target, "crates/cclab-agent/src/agents");
        assert_eq!(artifact.key_files.len(), 1);
        assert_eq!(artifact.architectural_patterns.len(), 2);
    }

    #[tokio::test]
    async fn test_run_extracts_json_from_markdown_fence() {
        let response = format!(
            "I explored the codebase. Here is the artifact:\n\n```json\n{}\n```\n",
            valid_artifact_json()
        );
        let agent = ReferenceCodebaseContextAgent::builder()
            .with_provider(MockProvider { response })
            .build()
            .unwrap();

        let result = agent.run("target").await.unwrap();
        let artifact: ReferenceCodebaseArtifact = serde_json::from_str(&result).unwrap();
        assert_eq!(artifact.target, "crates/cclab-agent/src/agents");
    }

    #[tokio::test]
    async fn test_run_fails_when_no_valid_json() {
        let agent = ReferenceCodebaseContextAgent::builder()
            .with_provider(MockProvider {
                response: "No JSON here at all.".to_string(),
            })
            .with_max_retries(0)
            .build()
            .unwrap();

        let err = agent.run("target").await.unwrap_err();
        assert!(err.to_string().contains("ReferenceCodebaseContextAgent"));
    }

    #[test]
    fn test_extract_json_object_plain() {
        let text = r#"{"target":"x","key_files":[],"architectural_patterns":[],"dependencies":[],"relationships":[],"summary":"s"}"#;
        let extracted = ReferenceCodebaseContextAgent::extract_json_object(text).unwrap();
        assert!(extracted.contains("\"target\""));
    }

    #[test]
    fn test_extract_json_object_from_markdown_fence() {
        let text = "```json\n{\"target\":\"x\"}\n```";
        let extracted = ReferenceCodebaseContextAgent::extract_json_object(text).unwrap();
        assert!(extracted.contains("\"target\""));
    }

    #[test]
    fn test_extract_json_object_with_nested_braces() {
        let text = r#"Some text {"outer": {"inner": 1}} more text"#;
        let extracted = ReferenceCodebaseContextAgent::extract_json_object(text).unwrap();
        assert_eq!(extracted, r#"{"outer": {"inner": 1}}"#);
    }

    #[test]
    fn test_extract_json_object_none_when_no_braces() {
        assert!(ReferenceCodebaseContextAgent::extract_json_object("no braces here").is_none());
    }

    #[test]
    fn test_artifact_round_trip_serialization() {
        let artifact = ReferenceCodebaseArtifact {
            target: "crates/foo".to_string(),
            key_files: vec![KeyFile {
                path: "src/lib.rs".to_string(),
                purpose: "Entry point".to_string(),
                key_exports: vec!["Foo".to_string(), "Bar".to_string()],
            }],
            architectural_patterns: vec!["builder pattern".to_string()],
            dependencies: vec![CodebaseDependency {
                name: "serde".to_string(),
                dependency_type: "external".to_string(),
                purpose: "Serialization".to_string(),
            }],
            relationships: vec![ComponentRelationship {
                from: "Foo".to_string(),
                to: "Bar".to_string(),
                relationship_type: "uses".to_string(),
            }],
            summary: "A test crate.".to_string(),
        };

        let json = serde_json::to_value(&artifact).unwrap();
        let parsed: ReferenceCodebaseArtifact = serde_json::from_value(json).unwrap();
        assert_eq!(parsed, artifact);
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = ReferenceCodebaseContextAgent::builder()
            .build()
            .unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_builder_defaults_registry_when_not_provided() {
        // Should succeed — registry defaults to empty ToolRegistry
        let _agent = ReferenceCodebaseContextAgent::builder()
            .with_provider(MockProvider {
                response: "{}".to_string(),
            })
            .build()
            .unwrap();
    }

    #[test]
    fn test_config_defaults() {
        let config = ReferenceCodebaseContextAgentConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_turns, 30);
        assert_eq!(config.max_retries, 2);
    }
}

// CODEGEN-END
~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/reference_codebase_context.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete reference-codebase-context agent
      module, including schema-derived shapes, defaults, inner CodingAgent
      orchestration, JSON extraction, prompt text, builder methods, and tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

## Reviews
<!-- type: doc lang: markdown -->

### Review 1
**Verdict:** approved

- [overview] 7 types: 4 data carriers + Config + Agent + Builder.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split.

### Review 2
**Verdict:** approved

- [overview] Promotes behavior and tests into full source-template ownership while retaining the schema shape contract.
- [source] Uses `strip-managed-markers` to preserve existing Rust behavior and remove mixed CODEGEN/HANDWRITE boundaries.
- [changes] Correctly routes the target file through the `source` section with `impl_mode: codegen`.

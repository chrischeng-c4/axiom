// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
// CODEGEN-BEGIN
//! RestructureCodebaseAgent — decomposes a codebase into budget-safe spec groups.
//!
//! Operates during the SDD fillback flow before [`ReferenceCodebaseContextAgent`]
//! and [`CodebaseToSpecAgent`]. It reads workspace manifests, summarizes directory
//! structure, estimates token counts, and iteratively drills down into large
//! directories until every component fits within the configured token budget.
//!
//! # Flow
//!
//! 1. An inner [`CodingAgent`] is instantiated with four specialized tools:
//!    - `read_manifest` — discover workspace members
//!    - `list_folder_summary` — structural tree + file/line counts
//!    - `estimate_tokens` — heuristic token count (lines × 3)
//!    - `set_grouping` — finalize groups (terminal action)
//! 2. The inner agent loops, calling tools until it invokes `set_grouping`.
//! 3. [`SetGroupingTool`] stores the groups in shared [`GroupingState`].
//! 4. After the inner agent finishes, the outer agent extracts and returns the
//!    groups as a JSON-serialized [`Vec<SpecGroup>`].
//!
//! # Failure Modes
//!
//! - If the inner agent exhausts `max_turns` without calling `set_grouping`,
//!   an error is returned.
//! - On validation failure (no groups stored) after the inner agent completes,
//!   the agent retries up to `max_retries` times.
//!
//! [`ReferenceCodebaseContextAgent`]: crate::agents::reference_codebase_context::ReferenceCodebaseContextAgent
//! [`CodebaseToSpecAgent`]: crate::agents::codebase_to_spec::CodebaseToSpecAgent
//! [`SetGroupingTool`]: agent::tools::set_grouping::SetGroupingTool
//! [`GroupingState`]: agent::tools::set_grouping::GroupingState

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
use crate::agents::coding::CodingAgent;
use crate::agents::Agent;
use agent::error::{NovaError, NovaResult};
use agent::llm::LLMProvider;
use agent::stream::StreamHandler;
use agent::tools::set_grouping::{GroupingState, SetGroupingTool};
use agent::tools::{EstimateTokensTool, ListFolderSummaryTool, ReadManifestTool, ToolRegistry};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================
// Configuration
// ============================================================

use serde::{Deserialize, Serialize};

/// Configuration for RestructureCodebaseAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestructureCodebaseAgentConfig {
    /// LLM model identifier.
    pub model: String,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum exploration turns for the inner CodingAgent.
    pub max_turns: u32,
    /// Retries when set_grouping was never called.
    pub max_retries: u32,
    /// Token budget per group.
    pub token_budget: u64,
}

/// Codebase restructuring agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#schema
pub struct RestructureCodebaseAgent {
    /// Agent configuration.
    config: RestructureCodebaseAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
}

/// Builder for RestructureCodebaseAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#schema
pub struct RestructureCodebaseAgentBuilder {
    /// Agent configuration.
    config: RestructureCodebaseAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl Default for RestructureCodebaseAgentConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.0),
            max_turns: 40,
            max_retries: 2,
            token_budget: 50_000,
        }
    }
}

// ============================================================
// Agent
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl std::fmt::Debug for RestructureCodebaseAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestructureCodebaseAgent")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl Agent for RestructureCodebaseAgent {
    /// Run the agent on the given codebase path.
    ///
    /// `input` should be the root directory of the codebase to analyse.
    ///
    /// Returns a JSON-encoded `Vec<SpecGroup>` on success.
    async fn run(&self, input: &str) -> NovaResult<String> {
        let mut prompt = build_prompt(input, self.config.token_budget);

        for attempt in 0..=self.config.max_retries {
            let state = Arc::new(Mutex::new(GroupingState::default()));
            let inner = self.build_inner_agent(state.clone())?;

            // Run the inner coding agent; ignore its text response — we only
            // care about whether set_grouping was called.
            let _text = inner.run(&prompt).await;

            let stored = state.lock().await;
            match &stored.groups {
                Some(groups) if !groups.is_empty() => {
                    return serde_json::to_string_pretty(groups).map_err(|e| {
                        NovaError::Other(anyhow::anyhow!("Serialization error: {}", e))
                    });
                }
                Some(_) => {
                    // set_grouping was called with an empty list
                    if attempt < self.config.max_retries {
                        prompt = format!(
                            "Your previous grouping was empty. Please re-analyse the codebase \
                             at '{}' and call set_grouping with at least one non-empty group. \
                             Token budget per group: {} tokens.",
                            input, self.config.token_budget
                        );
                    } else {
                        return Err(NovaError::Other(anyhow::anyhow!(
                            "RestructureCodebaseAgent: set_grouping called with empty groups \
                             after {} retries",
                            self.config.max_retries
                        )));
                    }
                }
                None => {
                    // set_grouping was never called
                    if attempt < self.config.max_retries {
                        prompt = format!(
                            "You did not call set_grouping. Please re-analyse the codebase at \
                             '{}' and finalize the grouping by calling set_grouping. \
                             Token budget per group: {} tokens.",
                            input, self.config.token_budget
                        );
                    } else {
                        return Err(NovaError::Other(anyhow::anyhow!(
                            "RestructureCodebaseAgent: set_grouping was never called after \
                             {} retries",
                            self.config.max_retries
                        )));
                    }
                }
            }
        }

        Err(NovaError::Other(anyhow::anyhow!(
            "RestructureCodebaseAgent: exhausted retries"
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

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl RestructureCodebaseAgent {
    /// Create a new builder.
    pub fn builder() -> RestructureCodebaseAgentBuilder {
        RestructureCodebaseAgentBuilder::new()
    }

    /// Build the inner [`CodingAgent`] with the four specialized tools.
    fn build_inner_agent(&self, state: Arc<Mutex<GroupingState>>) -> NovaResult<CodingAgent> {
        let registry = ToolRegistry::new();
        registry.register(Arc::new(ReadManifestTool::new()))?;
        registry.register(Arc::new(ListFolderSummaryTool::new()))?;
        registry.register(Arc::new(EstimateTokensTool::new()))?;
        registry.register(Arc::new(SetGroupingTool::new(state)))?;

        CodingAgent::builder()
            .with_provider_arc(self.provider.clone())
            .with_registry(registry)
            .with_system_prompt(SYSTEM_PROMPT)
            .with_max_turns(self.config.max_turns)
            .with_temperature(self.config.temperature.unwrap_or(0.0))
            .with_model(&self.config.model)
            .build()
    }
}

// ============================================================
// Prompt builder
// ============================================================

fn build_prompt(codebase_path: &str, token_budget: u64) -> String {
    format!(
        "Analyse the codebase at '{}' and decompose it into spec groups.\n\n\
         Token budget per group: {} tokens.\n\n\
         Follow this workflow:\n\
         1. Call read_manifest to discover workspace members.\n\
         2. For each top-level component, call list_folder_summary and estimate_tokens.\n\
         3. If a component's estimated_tokens exceeds the budget, drill down into its \
            subdirectories and repeat steps 2–3 until all sub-components fit.\n\
         4. Once every component is mapped to a budget-safe group, call set_grouping \
            with the complete list of groups. This MUST be your last tool call.\n\n\
         Group naming: use the relative path of the dominant directory (e.g., \
         'crates/cclab-agent', 'src/frontend').",
        codebase_path, token_budget
    )
}

// ============================================================
// System prompt
// ============================================================

const SYSTEM_PROMPT: &str = r#"You are a software architect performing codebase decomposition for spec-driven development (SDD) fillback.

Your goal is to partition a codebase into spec groups where each group's token estimate fits within a given budget, preventing context window overflows for downstream agents.

## Tools Available

- `read_manifest` — parse Cargo.toml/package.json/pyproject.toml to find workspace members
- `list_folder_summary` — get folder tree, file count, and line count for a path
- `estimate_tokens` — compute heuristic token count (lines × 3) for a path
- `set_grouping` — MANDATORY terminal tool: store the final groups array and exit

## Algorithm

1. Call `read_manifest` at the repo root to discover workspace members as initial components.
2. For each component, call `list_folder_summary` (depth=2) and `estimate_tokens`.
3. If `estimated_tokens > budget`, subdivide: call `list_folder_summary` at a deeper depth on that component to find sub-directories, then estimate each sub-directory separately.
4. Repeat until every candidate path fits within the budget.
5. Assemble the final groups array and call `set_grouping`. This is the LAST tool call.

## Rules

- NEVER include paths whose estimated tokens exceed the budget as a single group.
- A group MAY contain multiple small paths if their combined estimate fits the budget.
- After `set_grouping` succeeds, output a brief one-line confirmation and STOP."#;

// ============================================================
// Builder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl RestructureCodebaseAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: RestructureCodebaseAgentConfig::default(),
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

    pub fn with_token_budget(mut self, budget: u64) -> Self {
        self.config.token_budget = budget;
        self
    }

    pub fn build(self) -> NovaResult<RestructureCodebaseAgent> {
        let provider = self.provider.ok_or_else(|| {
            NovaError::ConfigError("RestructureCodebaseAgent: LLM provider is required".to_string())
        })?;
        Ok(RestructureCodebaseAgent {
            config: self.config,
            provider,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/restructure_codebase.md#source
impl Default for RestructureCodebaseAgentBuilder {
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
    use agent::tools::set_grouping::SpecGroup;
    use agent::types::{TokenUsage, ToolCall};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex as StdMutex;

    // ---- Mock provider that simulates the set_grouping tool call ----

    /// A mock provider that first emits a `set_grouping` tool call, then
    /// returns a plain text response (no tool calls) so the CodingAgent exits.
    struct SetGroupingMockProvider {
        /// Responses to return in sequence; after all consumed, returns an
        /// empty tool_calls response to signal stop.
        calls: StdMutex<u32>,
    }

    impl SetGroupingMockProvider {
        fn new() -> Self {
            Self {
                calls: StdMutex::new(0),
            }
        }
    }

    #[async_trait]
    impl agent::llm::LLMProvider for SetGroupingMockProvider {
        fn provider_name(&self) -> &str {
            "mock"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }

        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;

            if *calls == 1 {
                // First call: emit set_grouping tool call
                let args = serde_json::json!({
                    "groups": [
                        {
                            "name": "crates/agent",
                            "paths": ["crates/cclab-agent/src"],
                            "description": "Agent crate",
                            "estimated_tokens": 30000
                        }
                    ]
                });
                Ok(CompletionResponse {
                    content: String::new(),
                    tool_calls: Some(vec![ToolCall {
                        id: "tc-1".to_string(),
                        name: "set_grouping".to_string(),
                        arguments: args,
                    }]),
                    finish_reason: "tool_use".to_string(),
                    usage: TokenUsage::default(),
                    model: "mock-model".to_string(),
                    metadata: HashMap::new(),
                })
            } else {
                // Second call: no more tool calls → CodingAgent exits
                Ok(CompletionResponse {
                    content: "Grouping complete.".to_string(),
                    tool_calls: None,
                    finish_reason: "stop".to_string(),
                    usage: TokenUsage::default(),
                    model: "mock-model".to_string(),
                    metadata: HashMap::new(),
                })
            }
        }

        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            unimplemented!()
        }
    }

    /// Mock that never calls set_grouping (always returns stop immediately).
    struct NoGroupingMockProvider;

    #[async_trait]
    impl agent::llm::LLMProvider for NoGroupingMockProvider {
        fn provider_name(&self) -> &str {
            "mock"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["mock".to_string()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            Ok(CompletionResponse {
                content: "I analysed the codebase.".to_string(),
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

    // ---- Tests ----

    #[tokio::test]
    async fn test_run_returns_groups_when_set_grouping_called() {
        let agent = RestructureCodebaseAgent::builder()
            .with_provider(SetGroupingMockProvider::new())
            .with_max_retries(0)
            .build()
            .unwrap();

        let result = agent.run("/tmp/test-repo").await.unwrap();
        let groups: Vec<SpecGroup> = serde_json::from_str(&result).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "crates/agent");
        assert_eq!(groups[0].estimated_tokens, Some(30000));
    }

    #[tokio::test]
    async fn test_run_fails_when_set_grouping_never_called() {
        let agent = RestructureCodebaseAgent::builder()
            .with_provider(NoGroupingMockProvider)
            .with_max_retries(0)
            .build()
            .unwrap();

        let err = agent.run("/tmp/test-repo").await.unwrap_err();
        assert!(err.to_string().contains("set_grouping was never called"));
    }

    #[tokio::test]
    async fn test_run_retries_on_missing_grouping() {
        // max_retries=1: first attempt fails (NoGrouping), second also fails,
        // then error is returned.
        let agent = RestructureCodebaseAgent::builder()
            .with_provider(NoGroupingMockProvider)
            .with_max_retries(1)
            .build()
            .unwrap();

        let err = agent.run("/tmp/repo").await.unwrap_err();
        assert!(err.to_string().contains("set_grouping was never called"));
    }

    #[test]
    fn test_builder_missing_provider() {
        let err = RestructureCodebaseAgent::builder().build().unwrap_err();
        assert!(err.to_string().contains("provider"));
    }

    #[test]
    fn test_config_defaults() {
        let config = RestructureCodebaseAgentConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_turns, 40);
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.token_budget, 50_000);
    }

    #[test]
    fn test_builder_with_overrides() {
        let agent = RestructureCodebaseAgent::builder()
            .with_provider(NoGroupingMockProvider)
            .with_model("gemini-2.0-flash")
            .with_max_turns(20)
            .with_max_retries(1)
            .with_token_budget(100_000)
            .with_temperature(0.5)
            .build()
            .unwrap();

        assert_eq!(agent.config.model, "gemini-2.0-flash");
        assert_eq!(agent.config.max_turns, 20);
        assert_eq!(agent.config.max_retries, 1);
        assert_eq!(agent.config.token_budget, 100_000);
        assert_eq!(agent.config.temperature, Some(0.5));
    }
}

// CODEGEN-END

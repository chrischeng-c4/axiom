---
id: sdd-agents-coding-config
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# CodingAgentConfig

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/coding.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CodingAgent` | projects/agentic-workflow/src/agents/coding.rs | struct | pub | 62 |  |
| `CodingAgentBuilder` | projects/agentic-workflow/src/agents/coding.rs | struct | pub | 377 |  |
| `CodingAgentConfig` | projects/agentic-workflow/src/agents/coding.rs | struct | pub | 23 |  |
| `build` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 457 | build(self) -> NovaResult<CodingAgent> |
| `builder` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 97 | builder() -> CodingAgentBuilder |
| `create_context` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 119 | create_context(&self) -> ContextManager |
| `new` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 387 | new() -> Self |
| `registry` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 131 | registry(&self) -> &ToolRegistry |
| `run_conversation` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 108 | run_conversation(         &self,         context: &mut ContextManager,         input: &str,         handler: &dyn StreamHandler,     ) -> NovaResult<String> |
| `run_streaming` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 102 | run_streaming(&self, input: &str) -> NovaResult<String> |
| `security` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 126 | security(&self) -> &SecurityPolicy |
| `with_approval_handler` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 422 | with_approval_handler(mut self, handler: H) -> Self |
| `with_compact_model` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 452 | with_compact_model(mut self, model: impl Into<String>) -> Self |
| `with_max_context_tokens` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 442 | with_max_context_tokens(mut self, max_tokens: u32) -> Self |
| `with_max_turns` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 437 | with_max_turns(mut self, max_turns: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 432 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 397 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 402 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_registry` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 407 | with_registry(mut self, registry: ToolRegistry) -> Self |
| `with_registry_arc` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 412 | with_registry_arc(mut self, registry: Arc<ToolRegistry>) -> Self |
| `with_security` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 417 | with_security(mut self, security: SecurityPolicy) -> Self |
| `with_system_prompt` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 427 | with_system_prompt(mut self, prompt: impl Into<String>) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/coding.rs | function | pub | 447 | with_temperature(mut self, temperature: f32) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CodingAgentConfig:
    type: object
    required: [system_prompt, max_turns, model, temperature, max_tokens, max_context_tokens, compact_model]
    description: Configuration for the coding agent.
    properties:
      system_prompt:
        type: string
      max_turns:
        type: integer
        x-rust-type: u32
      model:
        type: string
      temperature:
        type: number
        x-rust-type: "Option<f32>"
      max_tokens:
        type: integer
        x-rust-type: "Option<u32>"
      max_context_tokens:
        type: integer
        x-rust-type: u32
      compact_model:
        type: string
        description: "Model used for LLM-based context summarization (default: claude-3-haiku-20240307)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
    x-trait-impls:
      - trait: Default
        impl_mode: codegen
        body: |
          Self {
              system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
              max_turns: 20,
              model: "claude-sonnet-4-20250514".to_string(),
              temperature: Some(0.0),
              max_tokens: Some(8192),
              max_context_tokens: 128_000,
              compact_model: "claude-3-haiku-20240307".to_string(),
          }
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/coding.rs -->
```rust
//! Coding agent implementation for code-related tasks.

use crate::agents::{Agent, ApprovalHandler, AutoApproveHandler};
use agent::context::ContextManager;
use agent::error::{NovaError, NovaResult};
use agent::llm::{CompletionRequest, LLMProvider, ToolDefinition};
use agent::security::{ApprovalRequest, ApprovalResponse, SecurityPolicy};
use agent::stream::{NoOpHandler, PrintHandler, StreamEvent, StreamHandler};
use agent::tools::{ToolExecutor, ToolRegistry};
use agent::types::{Message, ToolCall};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

use serde::{Deserialize, Serialize};

/// Configuration for the coding agent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingAgentConfig {
    pub system_prompt: String,
    pub max_turns: u32,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub max_context_tokens: u32,
    /// Model used for LLM-based context summarization (default: claude-3-haiku-20240307).
    pub compact_model: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#schema.trait-impls.Default
impl Default for CodingAgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            max_turns: 20,
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.0),
            max_tokens: Some(8192),
            max_context_tokens: 128_000,
            compact_model: "claude-3-haiku-20240307".to_string(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#changes
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an expert coding assistant with access to tools for reading, writing, and editing files, searching codebases, and executing shell commands.

When helping with coding tasks:
1. First understand the codebase structure by exploring relevant files
2. Plan your approach before making changes
3. Make precise, minimal changes to accomplish the task
4. Verify your changes work as expected

Always explain what you're doing and why."#;

/// Main coding agent orchestrator.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
pub struct CodingAgent {
    config: CodingAgentConfig,
    provider: Arc<dyn LLMProvider>,
    registry: Arc<ToolRegistry>,
    security: SecurityPolicy,
    approval_handler: Arc<dyn ApprovalHandler>,
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
impl Agent for CodingAgent {
    async fn run(&self, input: &str) -> NovaResult<String> {
        let handler = NoOpHandler;
        self.run_with_handler(input, &handler).await
    }

    async fn run_with_handler(
        &self,
        input: &str,
        handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        let mut context = ContextManager::new(self.config.max_context_tokens);
        context.set_system_prompt(&self.config.system_prompt);
        // Wire smart auto-compact: use the same provider with the compact model
        context.set_compact_provider(self.provider.clone());
        context.set_compact_model(&self.config.compact_model);
        context.add_user_message(input);

        self.run_loop(&mut context, handler).await
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
impl CodingAgent {
    /// Create a new coding agent builder.
    pub fn builder() -> CodingAgentBuilder {
        CodingAgentBuilder::new()
    }

    /// Run the agent with streaming output to stdout.
    pub async fn run_streaming(&self, input: &str) -> NovaResult<String> {
        let handler = PrintHandler::new(true);
        self.run_with_handler(input, &handler).await
    }

    /// Run a multi-turn conversation.
    pub async fn run_conversation(
        &self,
        context: &mut ContextManager,
        input: &str,
        handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        context.add_user_message(input);
        self.run_loop(context, handler).await
    }

    /// Create a new context manager for multi-turn conversations.
    pub fn create_context(&self) -> ContextManager {
        let mut context = ContextManager::new(self.config.max_context_tokens);
        context.set_system_prompt(&self.config.system_prompt);
        context
    }

    /// Get the security policy.
    pub fn security(&self) -> &SecurityPolicy {
        &self.security
    }

    /// Get the tool registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    async fn run_loop(
        &self,
        context: &mut ContextManager,
        handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        handler.on_event(StreamEvent::Started).await?;

        let executor =
            ToolExecutor::new(self.registry.clone()).with_timeout(self.security.shell_timeout);

        let mut turn = 0;
        let final_response;

        loop {
            if turn >= self.config.max_turns {
                return Err(NovaError::MaxTurnsReached(self.config.max_turns));
            }

            if handler.should_cancel() {
                return Err(NovaError::Other(anyhow::anyhow!("Execution cancelled")));
            }

            turn += 1;
            debug!("Starting turn {}", turn);

            let request = self.build_request(context)?;

            handler
                .on_event(StreamEvent::Thinking {
                    model: self.config.model.clone(),
                })
                .await?;

            let response = self.provider.complete(request).await?;

            let mut assistant_msg = Message::assistant(&response.content);
            if let Some(ref tool_calls) = response.tool_calls {
                assistant_msg = assistant_msg.with_tool_calls(tool_calls.clone());
            }
            context.add_message(assistant_msg);

            if !response.content.is_empty() {
                handler
                    .on_event(StreamEvent::TextChunk {
                        content: response.content.clone(),
                    })
                    .await?;
            }

            if let Some(tool_calls) = response.tool_calls {
                if !tool_calls.is_empty() {
                    self.execute_tool_calls(context, &tool_calls, &executor, handler)
                        .await?;
                    continue;
                }
            }

            final_response = response.content;
            break;
        }

        handler
            .on_event(StreamEvent::TurnCompleted { turn_number: turn })
            .await?;
        handler
            .on_event(StreamEvent::Completed {
                content: final_response.clone(),
            })
            .await?;

        Ok(final_response)
    }

    fn build_request(&self, context: &ContextManager) -> NovaResult<CompletionRequest> {
        let messages = context.get_messages();
        let tools = self.get_tool_definitions();

        let mut request = CompletionRequest::new(messages, &self.config.model);

        if !tools.is_empty() {
            request = request.with_tools(tools);
        }

        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }

        if let Some(max_tokens) = self.config.max_tokens {
            request = request.with_max_tokens(max_tokens);
        }

        Ok(request)
    }

    fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.registry
            .tool_names()
            .iter()
            .filter_map(|name| {
                self.registry.get(name).map(|tool| {
                    let def = tool.definition();
                    ToolDefinition {
                        name: def.name,
                        description: def.description,
                        parameters: self.params_to_json_schema(&def.parameters),
                    }
                })
            })
            .collect()
    }

    fn params_to_json_schema(&self, params: &[agent::tools::ToolParameter]) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in params {
            properties.insert(
                param.name.clone(),
                serde_json::json!({
                    "type": param.parameter_type,
                    "description": param.description
                }),
            );
            if param.required {
                required.push(param.name.clone());
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    async fn execute_tool_calls(
        &self,
        context: &mut ContextManager,
        tool_calls: &[ToolCall],
        executor: &ToolExecutor,
        handler: &dyn StreamHandler,
    ) -> NovaResult<()> {
        for tool_call in tool_calls {
            handler
                .on_event(StreamEvent::ToolCallRequested {
                    tool_name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                })
                .await?;

            if self.security.requires_approval(&tool_call.name) {
                let approval_request = ApprovalRequest {
                    tool_name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                    description: format!("Execute tool '{}'", tool_call.name),
                    risks: vec!["This tool may modify files or execute commands.".to_string()],
                };

                handler
                    .on_event(StreamEvent::ApprovalRequested {
                        tool_name: tool_call.name.clone(),
                        description: approval_request.description.clone(),
                    })
                    .await?;

                let response = self
                    .approval_handler
                    .request_approval(approval_request)
                    .await?;

                let approved = matches!(
                    response,
                    ApprovalResponse::Approved | ApprovalResponse::AlwaysApprove
                );
                handler
                    .on_event(StreamEvent::ApprovalReceived { approved })
                    .await?;

                if !approved {
                    context.add_tool_result(
                        &tool_call.id,
                        serde_json::json!({
                            "error": "Execution denied by user",
                            "tool": tool_call.name
                        })
                        .to_string(),
                    );
                    continue;
                }
            }

            handler
                .on_event(StreamEvent::ToolExecutionStarted {
                    tool_name: tool_call.name.clone(),
                })
                .await?;

            let start = Instant::now();
            let result = executor
                .execute(&tool_call.name, tool_call.arguments.clone())
                .await;
            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(output) => {
                    handler
                        .on_event(StreamEvent::ToolExecutionCompleted {
                            tool_name: tool_call.name.clone(),
                            result: output.clone(),
                            duration_ms,
                        })
                        .await?;

                    context.add_tool_result(&tool_call.id, output.to_string());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    handler
                        .on_event(StreamEvent::ToolExecutionFailed {
                            tool_name: tool_call.name.clone(),
                            error: error_msg.clone(),
                        })
                        .await?;

                    context.add_tool_result(
                        &tool_call.id,
                        serde_json::json!({
                            "error": error_msg,
                            "tool": tool_call.name
                        })
                        .to_string(),
                    );
                }
            }
        }

        Ok(())
    }
}

/// Builder for CodingAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
pub struct CodingAgentBuilder {
    config: CodingAgentConfig,
    provider: Option<Arc<dyn LLMProvider>>,
    registry: Option<Arc<ToolRegistry>>,
    security: SecurityPolicy,
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
impl CodingAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: CodingAgentConfig::default(),
            provider: None,
            registry: None,
            security: SecurityPolicy::default(),
            approval_handler: None,
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

    pub fn with_security(mut self, security: SecurityPolicy) -> Self {
        self.security = security;
        self
    }

    pub fn with_approval_handler<H: ApprovalHandler + 'static>(mut self, handler: H) -> Self {
        self.approval_handler = Some(Arc::new(handler));
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = prompt.into();
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.config.max_turns = max_turns;
        self
    }

    pub fn with_max_context_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_context_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    pub fn with_compact_model(mut self, model: impl Into<String>) -> Self {
        self.config.compact_model = model.into();
        self
    }

    pub fn build(self) -> NovaResult<CodingAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;

        let registry = self
            .registry
            .unwrap_or_else(|| Arc::new(ToolRegistry::new()));
        let approval_handler = self
            .approval_handler
            .unwrap_or_else(|| Arc::new(AutoApproveHandler {}));

        Ok(CodingAgent {
            config: self.config,
            provider,
            registry,
            security: self.security,
            approval_handler,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/coding_config.md#source
impl Default for CodingAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CodingAgentConfig::default();
        assert!(!config.system_prompt.is_empty());
        assert!(config.max_context_tokens > 0);
    }

    #[test]
    fn test_builder_without_provider() {
        let result = CodingAgentBuilder::new().build();
        assert!(result.is_err());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/coding.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete coding agent module, including
      CodingAgentConfig, default config, tool-backed runtime loop, approval
      flow, builder methods, and unit tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single-struct + custom Default body scope clean.
- [schema] Derive list + Option<u32>/Option<f32> overrides + custom Default body all match source.
- [changes] Two-entry split correct.

---
id: sdd-agents-analyst
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# Analyst Agent Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/analyst.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AnalystAgent` | projects/agentic-workflow/src/agents/analyst.rs | struct | pub | 59 |  |
| `AnalystAgentBuilder` | projects/agentic-workflow/src/agents/analyst.rs | struct | pub | 78 |  |
| `AnalystAgentConfig` | projects/agentic-workflow/src/agents/analyst.rs | struct | pub | 40 |  |
| `build` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 636 | build(self) -> NovaResult<AnalystAgent> |
| `builder` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 162 | builder() -> AnalystAgentBuilder |
| `create_context` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 184 | create_context(&self) -> ContextManager |
| `is_paused` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 209 | is_paused(&self) -> bool |
| `new` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 512 | new() -> Self |
| `pending_clarification` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 215 | pending_clarification(&self) -> Option<agent::storage::PendingClarification> |
| `registry` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 288 | registry(&self) -> &ToolRegistry |
| `resume_with_response` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 225 | resume_with_response(         &self,         user_response: &str,         handler: &dyn StreamHandler,     ) -> NovaResult<String> |
| `run_conversation` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 173 | run_conversation(         &self,         context: &mut ContextManager,         input: &str,         handler: &dyn StreamHandler,     ) -> NovaResult<String> |
| `run_streaming` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 167 | run_streaming(&self, input: &str) -> NovaResult<String> |
| `save_session` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 196 | save_session(&self) -> NovaResult<()> |
| `security` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 283 | security(&self) -> &SecurityPolicy |
| `session` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 191 | session(&self) -> SessionState |
| `with_approval_handler` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 540 | with_approval_handler(mut self, handler: H) -> Self |
| `with_compact_model` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 570 | with_compact_model(mut self, model: impl Into<String>) -> Self |
| `with_github` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 594 | with_github(         mut self,         token: impl Into<String>,         owner: impl Into<String>,         repo: impl Into<String>,     ) -> NovaResult<Self> |
| `with_gitlab` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 606 | with_gitlab(         mut self,         token: impl Into<String>,         base_url: impl Into<String>,         project_id: impl Into<String>,     ) -> NovaResult<Self> |
| `with_jira` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 618 | with_jira(         mut self,         base_url: impl Into<String>,         email: impl Into<String>,         api_token: impl Into<String>,         project_key: Option<String>,     ) -> NovaResult<Self> |
| `with_max_context_tokens` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 560 | with_max_context_tokens(mut self, max_tokens: u32) -> Self |
| `with_max_turns` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 555 | with_max_turns(mut self, max_turns: u32) -> Self |
| `with_model` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 550 | with_model(mut self, model: impl Into<String>) -> Self |
| `with_provider` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 525 | with_provider(mut self, provider: P) -> Self |
| `with_provider_arc` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 530 | with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self |
| `with_security` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 535 | with_security(mut self, security: SecurityPolicy) -> Self |
| `with_session_id` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 588 | with_session_id(mut self, id: impl Into<String>) -> Self |
| `with_storage` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 576 | with_storage(mut self, storage: S) -> Self |
| `with_storage_arc` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 582 | with_storage_arc(mut self, storage: Arc<dyn Storage>) -> Self |
| `with_system_prompt` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 545 | with_system_prompt(mut self, prompt: impl Into<String>) -> Self |
| `with_temperature` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 565 | with_temperature(mut self, temperature: f32) -> Self |
| `with_tool` | projects/agentic-workflow/src/agents/analyst.rs | function | pub | 631 | with_tool(self, tool: T) -> NovaResult<Self> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  AnalystAgentConfig:
    type: object
    required: [system_prompt, max_turns, model, temperature, max_tokens, max_context_tokens, compact_model]
    description: Configuration for AnalystAgent.
    properties:
      system_prompt:
        type: string
        description: "System prompt template."
      max_turns:
        type: integer
        x-rust-type: "u32"
        description: "Maximum agent turns."
      model:
        type: string
        description: "LLM model identifier."
      temperature:
        type: number
        x-rust-type: "Option<f32>"
        description: "Sampling temperature."
      max_tokens:
        type: integer
        x-rust-type: "Option<u32>"
        description: "Maximum response tokens."
      max_context_tokens:
        type: integer
        x-rust-type: "u32"
        description: "Maximum context tokens before compaction."
      compact_model:
        type: string
        description: "Model used for context summarisation."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AnalystAgent:
    type: object
    required: [config, provider, registry, security, _approval_handler, storage, session]
    description: Analyst agent for requirements analysis.
    properties:
      config:
        type: object
        x-rust-type: "AnalystAgentConfig"
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
      security:
        type: object
        x-rust-type: "SecurityPolicy"
        x-rust-visibility: private
        description: "Security policy."
      _approval_handler:
        type: object
        x-rust-type: "Arc<dyn ApprovalHandler>"
        x-rust-visibility: private
        description: "Approval handler."
      storage:
        type: object
        x-rust-type: "Arc<dyn Storage>"
        x-rust-visibility: private
        description: "Artifact storage."
      session:
        type: object
        x-rust-type: "Arc<RwLock<SessionState>>"
        x-rust-visibility: private
        description: "Session state."
    x-rust-struct:
      derive: []

  AnalystAgentBuilder:
    type: object
    required: [config, provider, registry, security, approval_handler, storage, session_id, integrations]
    description: Builder for AnalystAgent.
    properties:
      config:
        type: object
        x-rust-type: "AnalystAgentConfig"
        x-rust-visibility: private
        description: "Agent configuration."
      provider:
        type: object
        x-rust-type: "Option<Arc<dyn LLMProvider>>"
        x-rust-visibility: private
        description: "Optional LLM provider."
      registry:
        type: object
        x-rust-type: "ToolRegistry"
        x-rust-visibility: private
        description: "Tool registry."
      security:
        type: object
        x-rust-type: "SecurityPolicy"
        x-rust-visibility: private
        description: "Security policy."
      approval_handler:
        type: object
        x-rust-type: "Option<Arc<dyn ApprovalHandler>>"
        x-rust-visibility: private
        description: "Optional approval handler."
      storage:
        type: object
        x-rust-type: "Option<Arc<dyn Storage>>"
        x-rust-visibility: private
        description: "Optional storage."
      session_id:
        type: string
        x-rust-type: "Option<String>"
        x-rust-visibility: private
        description: "Optional session id."
      integrations:
        type: array
        x-rust-type: "Vec<Box<dyn PlatformIntegration>>"
        x-rust-visibility: private
        description: "Platform integrations."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/analyst.rs -->
```rust
//! Analyst agent implementation for requirements analysis.
//!
//! The AnalystAgent is designed for analyzing requirements, investigating issues,
//! and producing structured analysis reports. It supports:
//!
//! - Session persistence via storage backends
//! - Platform integrations (GitHub, GitLab, Jira)
//! - Analysis-specific tools (notes, findings, web search)

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
use crate::agents::{Agent, ApprovalHandler, AutoApproveHandler};
use agent::context::ContextManager;
use agent::error::{NovaError, NovaResult};
use agent::integrations::{
    parse_clarification_response, GitHubIntegration, GitLabIntegration, JiraIntegration,
    PlatformIntegration,
};
use agent::llm::{CompletionRequest, LLMProvider, ToolDefinition};
use agent::security::SecurityPolicy;
use agent::storage::{MemoryStorage, SessionState, SessionStatus, Storage};
use agent::stream::{NoOpHandler, PrintHandler, StreamEvent, StreamHandler};
use agent::tools::{
    AskUserTool, RecordFindingTool, TakeNoteTool, Tool, ToolExecutor, ToolParameter, ToolRegistry,
    WebFetchTool, WebSearchTool,
};
use agent::types::{Message, ToolCall};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

use serde::{Deserialize, Serialize};

/// Configuration for AnalystAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalystAgentConfig {
    /// System prompt template.
    pub system_prompt: String,
    /// Maximum agent turns.
    pub max_turns: u32,
    /// LLM model identifier.
    pub model: String,
    /// Sampling temperature.
    pub temperature: Option<f32>,
    /// Maximum response tokens.
    pub max_tokens: Option<u32>,
    /// Maximum context tokens before compaction.
    pub max_context_tokens: u32,
    /// Model used for context summarisation.
    pub compact_model: String,
}

/// Analyst agent for requirements analysis.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#schema
pub struct AnalystAgent {
    /// Agent configuration.
    config: AnalystAgentConfig,
    /// LLM provider.
    provider: Arc<dyn LLMProvider>,
    /// Tool registry.
    registry: Arc<ToolRegistry>,
    /// Security policy.
    security: SecurityPolicy,
    /// Approval handler.
    _approval_handler: Arc<dyn ApprovalHandler>,
    /// Artifact storage.
    storage: Arc<dyn Storage>,
    /// Session state.
    session: Arc<RwLock<SessionState>>,
}

/// Builder for AnalystAgent.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#schema
pub struct AnalystAgentBuilder {
    /// Agent configuration.
    config: AnalystAgentConfig,
    /// Optional LLM provider.
    provider: Option<Arc<dyn LLMProvider>>,
    /// Tool registry.
    registry: ToolRegistry,
    /// Security policy.
    security: SecurityPolicy,
    /// Optional approval handler.
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
    /// Optional storage.
    storage: Option<Arc<dyn Storage>>,
    /// Optional session id.
    session_id: Option<String>,
    /// Platform integrations.
    integrations: Vec<Box<dyn PlatformIntegration>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
impl Default for AnalystAgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            max_turns: 30,
            model: "claude-sonnet-4-20250514".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(8192),
            max_context_tokens: 128_000,
            compact_model: "claude-3-haiku-20240307".to_string(),
        }
    }
}

const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an expert requirements analyst and business analyst. Your role is to:

1. Analyze requirements, user stories, and issues from various sources
2. Identify gaps, ambiguities, and potential risks
3. Ask clarifying questions when needed
4. Document findings and insights using structured notes
5. Produce clear, actionable analysis reports

When analyzing:
- Start by understanding the context and scope
- Identify stakeholders and their needs
- Look for missing acceptance criteria
- Note dependencies and technical constraints
- Consider edge cases and error scenarios

Use the available tools to:
- take_note: Record observations and insights during analysis
- record_finding: Document important conclusions with severity levels
- ask_user: Request clarification when requirements are ambiguous
- web_search/web_fetch: Research technical concepts or standards

Always explain your reasoning and provide structured, actionable recommendations."#;

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
impl Agent for AnalystAgent {
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

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
impl AnalystAgent {
    /// Create a new analyst agent builder.
    pub fn builder() -> AnalystAgentBuilder {
        AnalystAgentBuilder::new()
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

    /// Get the current session state.
    pub async fn session(&self) -> SessionState {
        self.session.read().await.clone()
    }

    /// Save the current session to storage.
    pub async fn save_session(&self) -> NovaResult<()> {
        let session = self.session.read().await;
        self.storage.save_session(&session).await
    }

    /// Save session with the current context messages.
    async fn save_session_with_context(&self, context: &ContextManager) -> NovaResult<()> {
        let mut session = self.session.write().await;
        session.set_messages(context.get_messages());
        self.storage.save_session(&session).await
    }

    /// Check if the session is paused and waiting for clarification.
    pub async fn is_paused(&self) -> bool {
        let session = self.session.read().await;
        session.status == SessionStatus::Paused && session.pending_clarification.is_some()
    }

    /// Get the pending clarification info if session is paused.
    pub async fn pending_clarification(&self) -> Option<agent::storage::PendingClarification> {
        let session = self.session.read().await;
        session.pending_clarification.clone()
    }

    /// Resume a paused session with a user response.
    ///
    /// This method is used when the user has responded to a clarification question.
    /// The response can be provided directly, or if `None`, the method will attempt
    /// to fetch new comments from the platform to find the response.
    pub async fn resume_with_response(
        &self,
        user_response: &str,
        handler: &dyn StreamHandler,
    ) -> NovaResult<String> {
        // Load session and verify it's paused
        let mut session = self.session.write().await;
        if session.status != SessionStatus::Paused {
            return Err(NovaError::Other(anyhow::anyhow!(
                "Session is not paused, cannot resume"
            )));
        }

        // Parse the user response
        let parsed = parse_clarification_response(user_response);

        // Format the user input for the LLM context
        let user_input = if !parsed.selected_options.is_empty() {
            if let Some(ref text) = parsed.reply_text {
                format!(
                    "User selected: {}\nAdditional comment: {}",
                    parsed.selected_options.join(", "),
                    text
                )
            } else {
                format!("User selected: {}", parsed.selected_options.join(", "))
            }
        } else if let Some(ref text) = parsed.reply_text {
            format!("User replied: {}", text)
        } else {
            user_response.to_string()
        };

        info!("Resuming session {} with user input", session.id);

        // Resume the session
        session.resume();

        // Restore context from saved messages
        let mut context = ContextManager::new(self.config.max_context_tokens);
        context.set_system_prompt(&self.config.system_prompt);

        // Restore previous messages
        for msg in &session.messages {
            context.add_message(msg.clone());
        }

        // Add the user's response as a new message
        context.add_user_message(&user_input);

        // Drop the lock before running the loop
        drop(session);

        // Continue execution
        self.run_loop(&mut context, handler).await
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
                    let should_pause = self
                        .execute_tool_calls(context, &tool_calls, &executor, handler)
                        .await?;

                    if should_pause {
                        // User input required - save session with context and return
                        self.save_session_with_context(context).await?;
                        final_response = response.content;
                        break;
                    }

                    continue;
                }
            }

            final_response = response.content;
            break;
        }

        // Save session at the end
        self.save_session().await?;

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

    fn params_to_json_schema(&self, params: &[ToolParameter]) -> serde_json::Value {
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
    ) -> NovaResult<bool> {
        let mut should_pause = false;

        for tool_call in tool_calls {
            handler
                .on_event(StreamEvent::ToolCallRequested {
                    tool_name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                })
                .await?;

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
                    // Check if this is a user input request
                    if output.get("type") == Some(&serde_json::json!("user_input_required")) {
                        should_pause = true;
                    }

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

        Ok(should_pause)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
impl AnalystAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: AnalystAgentConfig::default(),
            provider: None,
            registry: ToolRegistry::new(),
            security: SecurityPolicy::default(),
            approval_handler: None,
            storage: None,
            session_id: None,
            integrations: Vec::new(),
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

    /// Set the storage backend for session persistence.
    pub fn with_storage<S: Storage + 'static>(mut self, storage: S) -> Self {
        self.storage = Some(Arc::new(storage));
        self
    }

    /// Set the storage backend from an Arc.
    pub fn with_storage_arc(mut self, storage: Arc<dyn Storage>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set the session ID (creates a new session or loads existing).
    pub fn with_session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Add a GitHub integration.
    pub fn with_github(
        mut self,
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> NovaResult<Self> {
        let integration = GitHubIntegration::new(token, owner, repo)?;
        self.integrations.push(Box::new(integration));
        Ok(self)
    }

    /// Add a GitLab integration.
    pub fn with_gitlab(
        mut self,
        token: impl Into<String>,
        base_url: impl Into<String>,
        project_id: impl Into<String>,
    ) -> NovaResult<Self> {
        let integration = GitLabIntegration::new(token, base_url, project_id)?;
        self.integrations.push(Box::new(integration));
        Ok(self)
    }

    /// Add a Jira integration.
    pub fn with_jira(
        mut self,
        base_url: impl Into<String>,
        email: impl Into<String>,
        api_token: impl Into<String>,
        project_key: Option<String>,
    ) -> NovaResult<Self> {
        let integration = JiraIntegration::new(base_url, email, api_token, project_key)?;
        self.integrations.push(Box::new(integration));
        Ok(self)
    }

    /// Add a custom tool.
    pub fn with_tool<T: Tool + 'static>(self, tool: T) -> NovaResult<Self> {
        self.registry.register(Arc::new(tool))?;
        Ok(self)
    }

    pub async fn build(self) -> NovaResult<AnalystAgent> {
        let provider = self
            .provider
            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;

        let approval_handler = self
            .approval_handler
            .unwrap_or_else(|| Arc::new(AutoApproveHandler {}));

        let storage = self
            .storage
            .unwrap_or_else(|| Arc::new(MemoryStorage::new()));

        // Load or create session
        let session_id = self
            .session_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let session = match storage.load_session(&session_id).await? {
            Some(existing) => existing,
            None => SessionState::new(&session_id),
        };

        let session = Arc::new(RwLock::new(session));

        // Register analysis tools
        self.registry.register(Arc::new(AskUserTool::new()))?;
        self.registry
            .register(Arc::new(TakeNoteTool::new(session.clone())))?;
        self.registry
            .register(Arc::new(RecordFindingTool::new(session.clone())))?;

        // Register web tools (may fail if HTTP client creation fails)
        if let Ok(web_search) = WebSearchTool::new() {
            let _ = self.registry.register(Arc::new(web_search));
        }
        if let Ok(web_fetch) = WebFetchTool::new() {
            let _ = self.registry.register(Arc::new(web_fetch));
        }

        // Register platform integration tools
        for integration in self.integrations {
            let tools = integration.into_tools();
            for tool in tools {
                let _ = self.registry.register(Arc::from(tool));
            }
        }

        Ok(AnalystAgent {
            config: self.config,
            provider,
            registry: Arc::new(self.registry),
            security: self.security,
            _approval_handler: approval_handler,
            storage,
            session,
        })
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/analyst.md#source
impl Default for AnalystAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent::storage::PendingClarification;

    #[test]
    fn test_default_config() {
        let config = AnalystAgentConfig::default();
        assert!(!config.system_prompt.is_empty());
        assert!(config.max_context_tokens > 0);
        assert_eq!(config.max_turns, 30);
    }

    #[test]
    fn test_builder_without_provider() {
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { AnalystAgentBuilder::new().build().await });

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_creation() {
        let session = SessionState::new("test-session");
        let session = Arc::new(RwLock::new(session));

        let take_note = TakeNoteTool::new(session.clone());
        let result = take_note
            .execute(serde_json::json!({
                "content": "Test note",
                "category": "test"
            }))
            .await
            .unwrap();

        assert!(result["success"].as_bool().unwrap());

        let session = session.read().await;
        assert_eq!(session.notes.len(), 1);
    }

    #[tokio::test]
    async fn test_session_pause_detection() {
        let session = SessionState::new("test-session");
        let session = Arc::new(RwLock::new(session));

        // Initially not paused
        {
            let s = session.read().await;
            assert_eq!(s.status, SessionStatus::Active);
            assert!(s.pending_clarification.is_none());
        }

        // Pause with clarification
        {
            let mut s = session.write().await;
            s.pause_for_clarification(PendingClarification {
                platform: "github".to_string(),
                issue_id: "123".to_string(),
                comment_id: "456".to_string(),
                question: "Which option?".to_string(),
                options: vec![],
                multi_select: false,
                requested_at: chrono::Utc::now(),
            });
        }

        // Now paused
        {
            let s = session.read().await;
            assert_eq!(s.status, SessionStatus::Paused);
            assert!(s.pending_clarification.is_some());
        }
    }

    #[tokio::test]
    async fn test_session_message_persistence() {
        let session = SessionState::new("test-session");
        let session = Arc::new(RwLock::new(session));

        // Add some messages
        {
            let mut s = session.write().await;
            s.add_message(Message::user("Hello"));
            s.add_message(Message::assistant("Hi there!"));
            s.add_message(Message::user("Can you help?"));
        }

        // Verify messages are stored
        {
            let s = session.read().await;
            assert_eq!(s.messages.len(), 3);
            assert_eq!(s.messages[0].content, "Hello");
            assert_eq!(s.messages[1].content, "Hi there!");
            assert_eq!(s.messages[2].content, "Can you help?");
        }
    }

    #[test]
    fn test_parse_clarification_response_checkbox() {
        let response = r#"- [x] Option A
- [ ] Option B
- [x] Option C"#;

        let parsed = parse_clarification_response(response);
        assert_eq!(parsed.selected_options.len(), 2);
        assert!(parsed.selected_options.contains(&"Option A".to_string()));
        assert!(parsed.selected_options.contains(&"Option C".to_string()));
    }

    #[test]
    fn test_parse_clarification_response_with_text() {
        let response = "I think we should go with the first approach because it's simpler.";

        let parsed = parse_clarification_response(response);
        assert!(parsed.selected_options.is_empty());
        assert!(parsed.reply_text.is_some());
        assert!(parsed.reply_text.unwrap().contains("first approach"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/analyst.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete analyst agent module, including
      schema-derived structs, runtime behavior, tool/platform integrations,
      builder methods, and tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] 3 standard agent-pattern types.
- [schema] All in `required:`; foreign types via x-rust-type incl trait-object Arcs and Vec<Box<dyn ...>>.
- [changes] Standard split.

## Review 2
<!-- type: doc lang: markdown -->

**Verdict:** approved

- [overview] Promotes analyst behavior and tests into full source-template ownership while preserving schema as the type contract.
- [source] Uses `strip-managed-markers` to preserve existing Rust behavior and remove mixed CODEGEN/HANDWRITE boundaries.
- [changes] Correctly routes the target file through the `source` section with `impl_mode: codegen`.

//! `Agent<Deps, Output>` — generic typed agent runtime with a tool-dispatch
//! run loop. Implements the PydanticAI-style surface defined in
//! `.aw/tech-design/projects/agentkit/specs/agent-deps-output-run-loop.md`
//! (#2030).
//!
//! The agent is parameterised over a `Deps` bag (typed dependency injection,
//! #2034) and an `Output` type (typed structured result, #2032). One call to
//! `Agent::run(input)` drives the loop:
//!
//! 1. build the message list (system prompt + user input + accumulated tool
//!    turns)
//! 2. call the configured `LLMProvider`
//! 3. if the reply carries `tool_calls`, dispatch each handler with the
//!    shared `Deps` and append the result; loop back to step 2
//! 4. otherwise parse the reply as JSON, validate against `Output`'s schema,
//!    deserialize into `Output`, and return
//!
//! The validator-retry loop (#2033), structured-output coercion (#2032), and
//! cancellation/timeout (#2070) are deliberately layered on top of this base
//! in their own issues.

// HANDWRITE-BEGIN reason: codegen has no Rust generator for `changes`-section
// targets yet (the `cb gen-code` pipeline emits marker-only stubs for Rust
// modules). Once a `rust-runtime` section type + emitter lands that can
// reproduce this struct + impl from the lifecycle/dependency/test-plan
// graphs, swap the markers below to CODEGEN-BEGIN / CODEGEN-END and let the
// generator own the file. Until then this file is the temporary state.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use tracing::{info_span, Instrument};

use crate::error::{NovaError, NovaResult};
use crate::events::{AgentEvent, EventEmitter};
use crate::llm::{CompletionRequest, CompletionResponse, LLMProvider, ToolDefinition};
use crate::schema::Schema;
use crate::types::{Message, ToolCall};

/// Boxed async handler returned by [`ToolSpec`] handlers. The handler is
/// invoked once per LLM `tool_call`, with the shared [`Deps`](Agent) bag and
/// the parsed JSON arguments.
pub type ToolHandlerFuture = Pin<Box<dyn Future<Output = NovaResult<serde_json::Value>> + Send>>;

/// Function-pointer handler over `Deps`. Built via [`ToolSpec::new`] from a
/// closure that receives the deps bag plus the JSON arguments.
pub type ToolHandler<Deps> =
    Arc<dyn Fn(Deps, serde_json::Value) -> ToolHandlerFuture + Send + Sync>;

/// Tool descriptor bundled with its handler — the unit the [`Agent`] builder
/// stores per registered tool.
pub struct ToolSpec<Deps> {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub handler: ToolHandler<Deps>,
}

impl<Deps> ToolSpec<Deps>
where
    Deps: Clone + Send + Sync + 'static,
{
    /// Build a tool spec from a closure. The closure receives a clone of the
    /// agent's `Deps` bag and the JSON-shaped tool arguments.
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
        handler: F,
    ) -> Self
    where
        F: Fn(Deps, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NovaResult<serde_json::Value>> + Send + 'static,
    {
        let handler: ToolHandler<Deps> = Arc::new(move |deps, args| {
            let fut = handler(deps, args);
            Box::pin(fut) as ToolHandlerFuture
        });
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            handler,
        }
    }
}

/// Read-only context handed to dynamic system-prompt builders and other
/// per-run helpers (#2034 — typed dependency injection). Carries a borrow of
/// the agent's `Deps` bag plus a snapshot of the running counters so tool /
/// prompt code can branch on "first turn vs revision N" without keeping its
/// own state.
pub struct RunContext<'a, Deps> {
    pub deps: &'a Deps,
    pub revision: u32,
    pub turn: u32,
}

/// Closure type for dynamic system prompts (#2034). The closure is invoked
/// once at the start of `run` with a borrow of the agent's `Deps` bag, so
/// callers can interpolate per-instance configuration (user id, locale, …)
/// into the system message.
pub type SystemPromptFn<Deps> = Arc<dyn Fn(&Deps) -> String + Send + Sync + 'static>;

enum SystemPrompt<Deps> {
    Static(String),
    Dynamic(SystemPromptFn<Deps>),
}

/// Typed agent runtime — generic over a dependency bag and a structured
/// output type. Construct via [`Agent::builder`].
pub struct Agent<Deps, Output> {
    provider: Arc<dyn LLMProvider>,
    model: String,
    system_prompt: Option<SystemPrompt<Deps>>,
    deps: Deps,
    tools: HashMap<String, ToolSpec<Deps>>,
    output_schema: Option<Schema>,
    max_turns: u32,
    max_revisions: u32,
    events: Option<EventEmitter<AgentEvent>>,
    _output: std::marker::PhantomData<fn() -> Output>,
}

/// Builder for [`Agent`].
pub struct AgentBuilder<Deps, Output> {
    provider: Option<Arc<dyn LLMProvider>>,
    model: Option<String>,
    system_prompt: Option<SystemPrompt<Deps>>,
    deps: Option<Deps>,
    tools: HashMap<String, ToolSpec<Deps>>,
    output_schema: Option<Schema>,
    max_turns: u32,
    max_revisions: u32,
    events: Option<EventEmitter<AgentEvent>>,
    _output: std::marker::PhantomData<fn() -> Output>,
}

impl<Deps, Output> Default for AgentBuilder<Deps, Output> {
    fn default() -> Self {
        Self {
            provider: None,
            model: None,
            system_prompt: None,
            deps: None,
            tools: HashMap::new(),
            output_schema: None,
            max_turns: 10,
            max_revisions: 2,
            events: None,
            _output: std::marker::PhantomData,
        }
    }
}

impl<Deps, Output> Agent<Deps, Output>
where
    Deps: Clone + Send + Sync + 'static,
    Output: DeserializeOwned + Send + 'static,
{
    pub fn builder() -> AgentBuilder<Deps, Output> {
        AgentBuilder::default()
    }

    /// Drive the run loop for a single user turn. Returns the typed
    /// `Output` on success, or a `NovaError` on the first hard failure
    /// (max turns reached, max revisions reached, tool error, LLM error).
    ///
    /// On validation failure, the agent feeds the error back to the LLM and
    /// asks for a corrected reply, up to [`max_revisions`](AgentBuilder::max_revisions)
    /// attempts (#2033 — validator + retry-with-feedback).
    pub async fn run(&self, input: impl Into<String>) -> NovaResult<Output> {
        let span = info_span!(
            "agent.run",
            agent.model = %self.model,
            agent.tools = self.tools.len(),
            agent.max_turns = self.max_turns,
            agent.max_revisions = self.max_revisions,
            turns_used = tracing::field::Empty,
            revisions = tracing::field::Empty,
            outcome = tracing::field::Empty,
        );
        self.run_inner(input.into()).instrument(span).await
    }

    fn emit(&self, event: AgentEvent) {
        if let Some(bus) = &self.events {
            let _ = bus.emit(event);
        }
    }

    async fn run_inner(&self, input: String) -> NovaResult<Output> {
        let span = tracing::Span::current();
        self.emit(AgentEvent::run_started(self.model.clone()));
        let mut messages: Vec<Message> = Vec::new();
        if let Some(prompt) = &self.system_prompt {
            let rendered = match prompt {
                SystemPrompt::Static(s) => s.clone(),
                SystemPrompt::Dynamic(f) => f(&self.deps),
            };
            messages.push(Message::system(rendered));
        }
        messages.push(Message::user(input));

        let mut revisions: u32 = 0;
        let mut turns_used: u32 = 0;
        let result: NovaResult<Output> = 'outer: loop {
            // Tool-dispatch loop: keep calling the LLM until it returns a
            // reply with no tool_calls. `turns_used` is cumulative across
            // revision rounds — total LLM calls are bounded by max_turns.
            let content = loop {
                if turns_used >= self.max_turns {
                    break 'outer Err(NovaError::MaxTurnsReached(self.max_turns));
                }
                turns_used += 1;

                self.emit(AgentEvent::LlmCallStarted {
                    turn: turns_used,
                    message_count: messages.len(),
                });
                let response = match self.call_llm(&messages).await {
                    Ok(r) => r,
                    Err(e) => break 'outer Err(e),
                };
                let tool_calls = response.tool_calls.clone().unwrap_or_default();
                self.emit(AgentEvent::LlmCallCompleted {
                    turn: turns_used,
                    tool_calls: tool_calls.len(),
                });
                if tool_calls.is_empty() {
                    break response.content;
                }

                messages.push(
                    Message::assistant(response.content.clone())
                        .with_tool_calls(tool_calls.clone()),
                );
                for call in tool_calls {
                    self.emit(AgentEvent::ToolCallStarted {
                        tool: call.name.clone(),
                        call_id: call.id.clone(),
                    });
                    let output = match self.dispatch_tool(&call).await {
                        Ok(v) => {
                            self.emit(AgentEvent::ToolCallCompleted {
                                tool: call.name.clone(),
                                call_id: call.id.clone(),
                            });
                            v
                        }
                        Err(e) => {
                            self.emit(AgentEvent::ToolCallFailed {
                                tool: call.name.clone(),
                                call_id: call.id.clone(),
                                error: e.to_string(),
                            });
                            break 'outer Err(e);
                        }
                    };
                    let serialized = match serde_json::to_string(&output) {
                        Ok(s) => s,
                        Err(e) => break 'outer Err(NovaError::SerializationError(e)),
                    };
                    messages.push(Message::tool(call.id.clone(), serialized));
                }
            };

            match self.finalize(&content) {
                Ok(out) => break Ok(out),
                Err(err) if is_validation_error(&err) => {
                    // max_revisions = 0 means "no retry budget" — surface the
                    // original error instead of a synthetic budget-exceeded.
                    if self.max_revisions == 0 {
                        self.emit(AgentEvent::ValidationFailed {
                            revision: revisions,
                            error: err.to_string(),
                        });
                        break Err(err);
                    }
                    if revisions >= self.max_revisions {
                        break Err(NovaError::MaxRevisionsExceeded(self.max_revisions));
                    }
                    self.emit(AgentEvent::ValidationFailed {
                        revision: revisions,
                        error: err.to_string(),
                    });
                    revisions += 1;
                    let feedback = format!(
                        "Your previous reply could not be used: {err}. Reply with JSON that matches the response schema exactly — no prose, no code fences."
                    );
                    messages.push(Message::assistant(content));
                    messages.push(Message::user(feedback));
                }
                Err(err) => break Err(err),
            }
        };

        span.record("turns_used", turns_used);
        span.record("revisions", revisions);
        match &result {
            Ok(_) => {
                span.record("outcome", "ok");
                self.emit(AgentEvent::RunCompleted {
                    turns_used,
                    revisions,
                });
            }
            Err(e) => {
                span.record("outcome", tracing::field::display(e));
                self.emit(AgentEvent::RunFailed {
                    turns_used,
                    revisions,
                    error: e.to_string(),
                });
            }
        }
        result
    }

    async fn call_llm(&self, messages: &[Message]) -> NovaResult<CompletionResponse> {
        let span = info_span!(
            "llm.call",
            llm.model = %self.model,
            llm.message_count = messages.len(),
            llm.tool_count = self.tools.len(),
        );
        async move {
            let mut request = CompletionRequest::new(messages.to_vec(), self.model.clone());

            if !self.tools.is_empty() {
                let defs: Vec<ToolDefinition> = self
                    .tools
                    .values()
                    .map(|t| ToolDefinition {
                        name: t.name.clone(),
                        description: t.description.clone(),
                        parameters: t.parameters.clone(),
                    })
                    .collect();
                request = request.with_tools(defs);
            }

            if let Some(schema) = &self.output_schema {
                request = request.with_response_schema(schema.to_json_schema());
            }

            self.provider.complete(request).await
        }
        .instrument(span)
        .await
    }

    async fn dispatch_tool(&self, call: &ToolCall) -> NovaResult<serde_json::Value> {
        let span = info_span!(
            "tool.call",
            tool.name = %call.name,
            tool.call_id = %call.id,
        );
        async move {
            let tool = self
                .tools
                .get(&call.name)
                .ok_or_else(|| NovaError::ToolNotFound(call.name.clone()))?;
            (tool.handler)(self.deps.clone(), call.arguments.clone()).await
        }
        .instrument(span)
        .await
    }

    fn finalize(&self, raw: &str) -> NovaResult<Output> {
        let value: serde_json::Value = if raw.trim().is_empty() {
            return Err(NovaError::MalformedLLMResponse(
                "empty final reply".to_string(),
            ));
        } else {
            serde_json::from_str(raw).map_err(|e| {
                NovaError::MalformedLLMResponse(format!("final reply is not valid JSON: {e}"))
            })?
        };

        if let Some(schema) = &self.output_schema {
            schema
                .validate(&value)
                .map_err(|e| NovaError::ValidationFailed(e.to_string()))?;
        }

        serde_json::from_value(value).map_err(|e| {
            NovaError::ValidationFailed(format!(
                "structured output failed to deserialize into target type: {e}"
            ))
        })
    }
}

impl<Deps, Output> AgentBuilder<Deps, Output>
where
    Deps: Clone + Send + Sync + 'static,
    Output: DeserializeOwned + Send + 'static,
{
    pub fn provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(SystemPrompt::Static(prompt.into()));
        self
    }

    /// Dynamic system prompt — the closure runs once per `run` call with a
    /// borrow of the agent's `Deps` bag, so callers can interpolate
    /// per-instance state (user id, locale, …) into the system message.
    /// Replaces any previous static or dynamic prompt (#2034).
    pub fn system_prompt_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&Deps) -> String + Send + Sync + 'static,
    {
        self.system_prompt = Some(SystemPrompt::Dynamic(Arc::new(f)));
        self
    }

    pub fn deps(mut self, deps: Deps) -> Self {
        self.deps = Some(deps);
        self
    }

    pub fn tool(mut self, spec: ToolSpec<Deps>) -> Self {
        self.tools.insert(spec.name.clone(), spec);
        self
    }

    pub fn output_schema(mut self, schema: Schema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    pub fn max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = max_turns;
        self
    }

    /// Cap on validator-feedback rounds (#2033). When the LLM's final reply
    /// fails schema validation, the agent feeds the error back and asks for a
    /// corrected reply up to this many times before failing with
    /// `NovaError::MaxRevisionsExceeded`. Default: 2.
    pub fn max_revisions(mut self, max_revisions: u32) -> Self {
        self.max_revisions = max_revisions;
        self
    }

    /// Attach a typed event bus (#2058). Every state change during
    /// `Agent::run` is published to the bus; subscribers receive
    /// [`AgentEvent`]s through their own queues. Publishing is
    /// fire-and-forget — if no subscriber is listening the events are
    /// dropped without error.
    pub fn events(mut self, bus: EventEmitter<AgentEvent>) -> Self {
        self.events = Some(bus);
        self
    }

    pub fn build(self) -> NovaResult<Agent<Deps, Output>> {
        Ok(Agent {
            provider: self
                .provider
                .ok_or_else(|| NovaError::ConfigError("Agent: provider missing".into()))?,
            model: self
                .model
                .ok_or_else(|| NovaError::ConfigError("Agent: model missing".into()))?,
            system_prompt: self.system_prompt,
            deps: self
                .deps
                .ok_or_else(|| NovaError::ConfigError("Agent: deps missing".into()))?,
            tools: self.tools,
            output_schema: self.output_schema,
            max_turns: self.max_turns,
            max_revisions: self.max_revisions,
            events: self.events,
            _output: std::marker::PhantomData,
        })
    }
}

/// Errors that the validator-retry loop (#2033) should bounce back to the
/// LLM rather than surface immediately. Everything else — tool error,
/// LLM-provider error, etc. — fails fast.
fn is_validation_error(err: &NovaError) -> bool {
    matches!(
        err,
        NovaError::ValidationFailed(_) | NovaError::MalformedLLMResponse(_)
    )
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::StreamResponse;
    use crate::types::{TokenUsage, ToolCall};
    use async_trait::async_trait;
    use serde::Deserialize;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct ScriptedProvider {
        replies: Arc<Mutex<Vec<CompletionResponse>>>,
    }

    impl ScriptedProvider {
        fn new(replies: Vec<CompletionResponse>) -> Self {
            Self {
                replies: Arc::new(Mutex::new(replies)),
            }
        }
    }

    #[async_trait]
    impl LLMProvider for ScriptedProvider {
        fn provider_name(&self) -> &str {
            "scripted"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["test-model".into()]
        }
        async fn complete(&self, _req: CompletionRequest) -> NovaResult<CompletionResponse> {
            let mut q = self.replies.lock().unwrap();
            if q.is_empty() {
                return Err(NovaError::LLMError("no scripted reply left".into()));
            }
            Ok(q.remove(0))
        }
        async fn complete_stream(&self, _req: CompletionRequest) -> NovaResult<StreamResponse> {
            Err(NovaError::NotSupported("stream".into()))
        }
        fn validate_model(&self, _model: &str) -> NovaResult<()> {
            Ok(())
        }
    }

    fn reply(content: &str) -> CompletionResponse {
        CompletionResponse {
            content: content.to_string(),
            tool_calls: None,
            finish_reason: "stop".into(),
            usage: TokenUsage::default(),
            model: "test-model".into(),
            metadata: Default::default(),
        }
    }

    fn reply_with_tool(name: &str, args: serde_json::Value, id: &str) -> CompletionResponse {
        CompletionResponse {
            content: String::new(),
            tool_calls: Some(vec![ToolCall {
                id: id.to_string(),
                name: name.to_string(),
                arguments: args,
            }]),
            finish_reason: "tool_use".into(),
            usage: TokenUsage::default(),
            model: "test-model".into(),
            metadata: Default::default(),
        }
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct WeatherReport {
        city: String,
        temp_c: i32,
    }

    fn weather_schema() -> Schema {
        Schema::object()
            .field("city", Schema::string())
            .field("temp_c", Schema::integer())
            .required(&["city", "temp_c"])
            .build()
    }

    #[tokio::test]
    async fn happy_path_direct_reply_matches_schema() {
        let provider = Arc::new(ScriptedProvider::new(vec![reply(
            r#"{"city":"Taipei","temp_c":24}"#,
        )]));

        let agent: Agent<(), WeatherReport> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(())
            .output_schema(weather_schema())
            .build()
            .unwrap();

        let out = agent.run("how's Taipei?").await.unwrap();
        assert_eq!(
            out,
            WeatherReport {
                city: "Taipei".into(),
                temp_c: 24
            }
        );
    }

    #[derive(Clone)]
    struct CounterDeps {
        seen: Arc<Mutex<Vec<String>>>,
    }

    #[tokio::test]
    async fn tool_call_path_deps_reachable_inside_handler() {
        let provider = Arc::new(ScriptedProvider::new(vec![
            reply_with_tool(
                "lookup_city",
                serde_json::json!({"city": "Taipei"}),
                "call_1",
            ),
            reply(r#"{"city":"Taipei","temp_c":24}"#),
        ]));

        let deps = CounterDeps {
            seen: Arc::new(Mutex::new(Vec::new())),
        };

        let tool = ToolSpec::new(
            "lookup_city",
            "Look up a city",
            serde_json::json!({
                "type": "object",
                "properties": {"city": {"type": "string"}},
                "required": ["city"],
            }),
            |deps: CounterDeps, args: serde_json::Value| async move {
                let city = args["city"].as_str().unwrap_or("").to_string();
                deps.seen.lock().unwrap().push(city.clone());
                Ok(serde_json::json!({"found": city}))
            },
        );

        let agent: Agent<CounterDeps, WeatherReport> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(deps.clone())
            .tool(tool)
            .output_schema(weather_schema())
            .build()
            .unwrap();

        let out = agent.run("weather?").await.unwrap();
        assert_eq!(out.city, "Taipei");
        assert_eq!(
            deps.seen.lock().unwrap().as_slice(),
            &["Taipei".to_string()]
        );
    }

    #[tokio::test]
    async fn validation_failure_returns_typed_error() {
        let provider = Arc::new(ScriptedProvider::new(vec![reply(
            r#"{"city":"Taipei","temp_c":"twenty-four"}"#,
        )]));

        let agent: Agent<(), WeatherReport> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(())
            .output_schema(weather_schema())
            .max_revisions(0) // disable retry loop so first failure surfaces
            .build()
            .unwrap();

        let err = agent.run("?").await.unwrap_err();
        assert!(
            matches!(err, NovaError::ValidationFailed(_)),
            "expected ValidationFailed, got {err:?}"
        );
    }

    #[tokio::test]
    async fn dynamic_system_prompt_sees_deps_at_run_time() {
        // Provider that records the messages it received so we can assert
        // the rendered system prompt was interpolated from Deps.
        #[derive(Clone, Default)]
        struct Recorder(Arc<Mutex<Vec<Message>>>);
        #[async_trait]
        impl LLMProvider for Recorder {
            fn provider_name(&self) -> &str {
                "recorder"
            }
            fn supported_models(&self) -> Vec<String> {
                vec!["m".into()]
            }
            async fn complete(&self, req: CompletionRequest) -> NovaResult<CompletionResponse> {
                *self.0.lock().unwrap() = req.messages.clone();
                Ok(reply(r#"{"city":"X","temp_c":0}"#))
            }
            async fn complete_stream(&self, _r: CompletionRequest) -> NovaResult<StreamResponse> {
                Err(NovaError::NotSupported("stream".into()))
            }
        }

        #[derive(Clone)]
        struct UserDeps {
            user_id: String,
        }

        let recorder = Arc::new(Recorder::default());
        let agent: Agent<UserDeps, WeatherReport> = Agent::builder()
            .provider(recorder.clone())
            .model("m")
            .deps(UserDeps {
                user_id: "u42".into(),
            })
            .system_prompt_fn(|d: &UserDeps| format!("You are assisting user {}.", d.user_id))
            .output_schema(weather_schema())
            .build()
            .unwrap();

        agent.run("hi").await.unwrap();

        let msgs = recorder.0.lock().unwrap().clone();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, crate::types::Role::System);
        assert_eq!(msgs[0].content, "You are assisting user u42.");
    }

    #[tokio::test]
    async fn validator_retry_recovers_after_one_bad_reply() {
        // First reply fails validation (temp_c is a string). Second reply
        // is well-formed → should succeed without surfacing the first error.
        let provider = Arc::new(ScriptedProvider::new(vec![
            reply(r#"{"city":"Taipei","temp_c":"twenty"}"#),
            reply(r#"{"city":"Taipei","temp_c":24}"#),
        ]));

        let agent: Agent<(), WeatherReport> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(())
            .output_schema(weather_schema())
            .max_revisions(2)
            .build()
            .unwrap();

        let out = agent.run("weather?").await.unwrap();
        assert_eq!(out.temp_c, 24);
    }

    #[tokio::test]
    async fn validator_retry_gives_up_after_max_revisions() {
        // Every reply fails validation — the loop should bail with
        // MaxRevisionsExceeded after exhausting the budget, not with the
        // last ValidationFailed (so callers can distinguish the two paths).
        let provider = Arc::new(ScriptedProvider::new(vec![
            reply(r#"{"city":"Taipei","temp_c":"a"}"#),
            reply(r#"{"city":"Taipei","temp_c":"b"}"#),
            reply(r#"{"city":"Taipei","temp_c":"c"}"#),
        ]));

        let agent: Agent<(), WeatherReport> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(())
            .output_schema(weather_schema())
            .max_revisions(2)
            .build()
            .unwrap();

        let err = agent.run("?").await.unwrap_err();
        assert!(
            matches!(err, NovaError::MaxRevisionsExceeded(2)),
            "expected MaxRevisionsExceeded(2), got {err:?}"
        );
    }

    #[tokio::test]
    async fn max_turns_reached_when_tool_loop_does_not_settle() {
        let provider = Arc::new(ScriptedProvider::new(vec![
            reply_with_tool("noop", serde_json::json!({}), "a"),
            reply_with_tool("noop", serde_json::json!({}), "b"),
            reply_with_tool("noop", serde_json::json!({}), "c"),
        ]));

        let tool = ToolSpec::new(
            "noop",
            "Noop tool",
            serde_json::json!({"type": "object", "properties": {}}),
            |_d: (), _a: serde_json::Value| async move { Ok(serde_json::json!({})) },
        );

        let agent: Agent<(), serde_json::Value> = Agent::builder()
            .provider(provider)
            .model("test-model")
            .deps(())
            .tool(tool)
            .max_turns(2)
            .build()
            .unwrap();

        let err = agent.run("loop").await.unwrap_err();
        assert!(matches!(err, NovaError::MaxTurnsReached(2)));
    }

    // Tracing-span emission is verified in `tests/tracing_spans.rs`
    // (separate test binary so its callsite cache isn't shared with
    // the parallel unit tests in this binary, which is what kept
    // `set_default` from re-enabling cached callsites).
}

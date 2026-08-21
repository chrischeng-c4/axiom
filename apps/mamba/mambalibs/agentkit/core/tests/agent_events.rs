//! End-to-end check that `Agent::run` publishes a typed event stream
//! to every subscriber attached via `AgentBuilder::events` (#2058).

use std::sync::{Arc, Mutex};

use agent::{
    Agent, AgentEvent, CompletionRequest, CompletionResponse, EventBus, LLMProvider, NovaError,
    NovaResult, Schema, StreamResponse, ToolCall, ToolSpec,
};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone)]
struct ScriptedProvider {
    replies: Arc<Mutex<Vec<CompletionResponse>>>,
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
        usage: Default::default(),
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
        usage: Default::default(),
        model: "test-model".into(),
        metadata: Default::default(),
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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

#[tokio::test(flavor = "current_thread")]
async fn happy_run_emits_typed_event_stream() {
    let bus: Arc<EventBus<AgentEvent>> = Arc::new(EventBus::new());
    let mut sub = bus.subscribe();

    let provider = Arc::new(ScriptedProvider {
        replies: Arc::new(Mutex::new(vec![
            reply_with_tool("echo", serde_json::json!({"msg": "hi"}), "call_1"),
            reply(r#"{"city":"Taipei","temp_c":24}"#),
        ])),
    });

    let tool = ToolSpec::new(
        "echo",
        "Echo a message",
        serde_json::json!({
            "type": "object",
            "properties": {"msg": {"type": "string"}},
            "required": ["msg"],
        }),
        |_: (), args: serde_json::Value| async move { Ok(serde_json::json!({"echoed": args})) },
    );

    let agent: Agent<(), WeatherReport> = Agent::builder()
        .provider(provider)
        .model("test-model")
        .deps(())
        .tool(tool)
        .output_schema(weather_schema())
        .events(bus.clone())
        .build()
        .unwrap();

    agent.run("trace me").await.unwrap();
    let events = sub.try_drain();

    // We expect: RunStarted, LlmCallStarted, LlmCallCompleted (with one
    // tool call), ToolCallStarted, ToolCallCompleted, LlmCallStarted,
    // LlmCallCompleted (zero tool calls), RunCompleted.
    let names: Vec<&'static str> = events
        .iter()
        .map(|e| match e {
            AgentEvent::RunStarted { .. } => "run_started",
            AgentEvent::LlmCallStarted { .. } => "llm_call_started",
            AgentEvent::LlmCallCompleted { .. } => "llm_call_completed",
            AgentEvent::ToolCallStarted { .. } => "tool_call_started",
            AgentEvent::ToolCallCompleted { .. } => "tool_call_completed",
            AgentEvent::ToolCallFailed { .. } => "tool_call_failed",
            AgentEvent::ValidationFailed { .. } => "validation_failed",
            AgentEvent::RunCompleted { .. } => "run_completed",
            AgentEvent::RunFailed { .. } => "run_failed",
        })
        .collect();

    assert_eq!(
        names,
        vec![
            "run_started",
            "llm_call_started",
            "llm_call_completed",
            "tool_call_started",
            "tool_call_completed",
            "llm_call_started",
            "llm_call_completed",
            "run_completed",
        ],
        "got: {events:#?}"
    );

    // RunCompleted should report turns_used = 2 (one tool + one final).
    let Some(AgentEvent::RunCompleted {
        turns_used,
        revisions,
    }) = events.last()
    else {
        panic!("last event must be RunCompleted, got: {events:#?}");
    };
    assert_eq!(*turns_used, 2);
    assert_eq!(*revisions, 0);
}

#[tokio::test(flavor = "current_thread")]
async fn validation_failure_emits_run_failed_event() {
    let bus: Arc<EventBus<AgentEvent>> = Arc::new(EventBus::new());
    let mut sub = bus.subscribe();

    let provider = Arc::new(ScriptedProvider {
        replies: Arc::new(Mutex::new(vec![reply(
            r#"{"city":"Taipei","temp_c":"twenty-four"}"#,
        )])),
    });

    let agent: Agent<(), WeatherReport> = Agent::builder()
        .provider(provider)
        .model("test-model")
        .deps(())
        .output_schema(weather_schema())
        .max_revisions(0)
        .events(bus.clone())
        .build()
        .unwrap();

    let err = agent.run("?").await.unwrap_err();
    assert!(matches!(err, NovaError::ValidationFailed(_)));

    let events = sub.try_drain();
    assert!(events
        .iter()
        .any(|e| matches!(e, AgentEvent::ValidationFailed { revision: 0, .. })));
    assert!(matches!(events.last(), Some(AgentEvent::RunFailed { .. })));
}

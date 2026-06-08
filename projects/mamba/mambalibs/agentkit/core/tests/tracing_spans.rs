//! Tracing span emission for `Agent::run` (#2057).
//!
//! This lives as its own integration test binary so its `tracing`
//! callsite cache is isolated from the unit-test binary. Unit tests
//! run in parallel and the first one to touch a callsite under a
//! `NoSubscriber` thread caches it as `Interest::never`, which then
//! starves a sibling test that installs a real subscriber via
//! `set_default`. In a fresh binary the cache starts empty so the
//! `set_default` subscriber sees every span we declare.

use std::sync::{Arc, Mutex};

use agent::{
    Agent, CompletionRequest, CompletionResponse, LLMProvider, NovaError, NovaResult, Schema,
    StreamResponse, ToolCall, ToolSpec,
};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::span::{Attributes, Id};
use tracing::Subscriber;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Registry;

#[derive(Default, Clone)]
struct SpanCapture {
    names: Arc<Mutex<Vec<String>>>,
}

impl<S> Layer<S> for SpanCapture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        self.names
            .lock()
            .unwrap()
            .push(attrs.metadata().name().to_string());
    }
}

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
async fn run_emits_agent_llm_and_tool_spans() {
    let capture = SpanCapture::default();
    let subscriber = Registry::default().with(capture.clone());
    let _default = tracing::subscriber::set_default(subscriber);

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
        .build()
        .unwrap();

    let _ = agent.run("trace me").await.unwrap();

    let names = capture.names.lock().unwrap().clone();

    assert!(
        names.iter().any(|n| n == "agent.run"),
        "expected agent.run span, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "llm.call"),
        "expected llm.call span, got: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "tool.call"),
        "expected tool.call span, got: {names:?}"
    );
    // Two LLM round trips: one to invoke the tool, one to return final.
    assert_eq!(
        names.iter().filter(|n| *n == "llm.call").count(),
        2,
        "expected exactly two llm.call spans, got: {names:?}"
    );
    assert_eq!(
        names.iter().filter(|n| *n == "tool.call").count(),
        1,
        "expected exactly one tool.call span, got: {names:?}"
    );
}

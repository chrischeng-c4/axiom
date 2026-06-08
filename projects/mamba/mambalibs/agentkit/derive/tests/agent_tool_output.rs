//! Smoke tests for `#[derive(AgentTool)]` (#2031) and
//! `#[derive(AgentOutput)]` (#2032). Both layer on top of `AgentSchema`,
//! so each test type derives all three.

use agent::{Agent, AgentSchema, NovaError, ToolSpec};
use agent_derive::{AgentOutput, AgentTool};
use serde::{Deserialize, Serialize};

// ── AgentTool ─────────────────────────────────────────────────────────────

#[derive(AgentSchema, AgentTool, Deserialize, Serialize, Debug)]
#[tool(name = "lookup_city", description = "Look up a city by name")]
#[allow(dead_code)]
struct LookupCity {
    #[schema(description = "City name to look up", min_length = 1, max_length = 80)]
    city: String,
}

#[test]
fn agent_tool_emits_name_description_parameters() {
    assert_eq!(LookupCity::TOOL_NAME, "lookup_city");
    assert_eq!(LookupCity::TOOL_DESCRIPTION, "Look up a city by name");

    let params = LookupCity::tool_parameters_json();
    assert_eq!(params["type"], "object");
    assert_eq!(params["properties"]["city"]["type"], "string");
    assert_eq!(
        params["properties"]["city"]["description"],
        "City name to look up"
    );
    assert_eq!(params["properties"]["city"]["minLength"], 1);
    assert_eq!(params["properties"]["city"]["maxLength"], 80);
    assert_eq!(
        params["required"].as_array().map(|a| a.len()),
        Some(1),
        "city must be in `required`"
    );
}

#[tokio::test]
async fn agent_tool_into_tool_spec_dispatches_typed_handler() {
    #[derive(Clone, Default)]
    struct Deps {
        last_seen: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    }

    let deps = Deps::default();

    let spec: ToolSpec<Deps> = LookupCity::into_tool_spec(|d: Deps, args: LookupCity| async move {
        *d.last_seen.lock().unwrap() = Some(args.city.clone());
        Ok(serde_json::json!({"found": args.city}))
    });

    assert_eq!(spec.name, "lookup_city");

    let result = (spec.handler)(deps.clone(), serde_json::json!({"city": "Taipei"}))
        .await
        .unwrap();
    assert_eq!(result["found"], "Taipei");
    assert_eq!(deps.last_seen.lock().unwrap().as_deref(), Some("Taipei"));
}

#[tokio::test]
async fn agent_tool_invalid_args_returns_typed_error() {
    let spec: ToolSpec<()> =
        LookupCity::into_tool_spec(|_: (), _: LookupCity| async move { Ok(serde_json::json!({})) });

    // Missing required `city` field — serde_json::from_value should fail and
    // `into_tool_spec` should surface it as `InvalidArguments`.
    let err = (spec.handler)((), serde_json::json!({})).await.unwrap_err();
    assert!(
        matches!(err, NovaError::InvalidArguments(_)),
        "expected InvalidArguments, got {err:?}"
    );
}

// ── AgentOutput ────────────────────────────────────────────────────────────

#[derive(AgentSchema, AgentOutput, Deserialize, Debug, PartialEq)]
#[allow(dead_code)]
struct WeatherReport {
    city: String,
    temp_c: i32,
}

#[test]
fn agent_output_parse_response_happy_path() {
    let out = WeatherReport::parse_response(r#"{"city":"Taipei","temp_c":24}"#).unwrap();
    assert_eq!(
        out,
        WeatherReport {
            city: "Taipei".into(),
            temp_c: 24
        }
    );
}

#[test]
fn agent_output_parse_response_rejects_non_json() {
    let err = WeatherReport::parse_response("not json").unwrap_err();
    assert!(
        matches!(err, NovaError::MalformedLLMResponse(_)),
        "expected MalformedLLMResponse, got {err:?}"
    );
}

#[test]
fn agent_output_parse_response_rejects_shape_mismatch() {
    let err = WeatherReport::parse_response(r#"{"city":"Taipei","temp_c":"twenty"}"#).unwrap_err();
    assert!(
        matches!(err, NovaError::ValidationFailed(_)),
        "expected ValidationFailed, got {err:?}"
    );
}

#[test]
fn agent_output_parse_response_rejects_empty() {
    let err = WeatherReport::parse_response("").unwrap_err();
    assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
}

// ── Integration: derived tool + output drive the Agent runtime ─────────────

#[tokio::test]
async fn derived_tool_drives_agent_run_loop_end_to_end() {
    use agent::llm::{CompletionRequest, CompletionResponse, LLMProvider, StreamResponse};
    use agent::types::{TokenUsage, ToolCall};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct Scripted(Arc<Mutex<Vec<CompletionResponse>>>);
    #[async_trait]
    impl LLMProvider for Scripted {
        fn provider_name(&self) -> &str {
            "scripted"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["m".into()]
        }
        async fn complete(&self, _r: CompletionRequest) -> agent::NovaResult<CompletionResponse> {
            Ok(self.0.lock().unwrap().remove(0))
        }
        async fn complete_stream(
            &self,
            _r: CompletionRequest,
        ) -> agent::NovaResult<StreamResponse> {
            Err(NovaError::NotSupported("stream".into()))
        }
    }

    let provider = Arc::new(Scripted(Arc::new(Mutex::new(vec![
        CompletionResponse {
            content: String::new(),
            tool_calls: Some(vec![ToolCall {
                id: "1".into(),
                name: "lookup_city".into(),
                arguments: serde_json::json!({"city": "Taipei"}),
            }]),
            finish_reason: "tool_use".into(),
            usage: TokenUsage::default(),
            model: "m".into(),
            metadata: Default::default(),
        },
        CompletionResponse {
            content: r#"{"city":"Taipei","temp_c":24}"#.into(),
            tool_calls: None,
            finish_reason: "stop".into(),
            usage: TokenUsage::default(),
            model: "m".into(),
            metadata: Default::default(),
        },
    ]))));

    let tool = LookupCity::into_tool_spec(|_d: (), args: LookupCity| async move {
        Ok(serde_json::json!({"city": args.city, "temp_c": 24}))
    });

    let agent: Agent<(), WeatherReport> = Agent::builder()
        .provider(provider)
        .model("m")
        .deps(())
        .tool(tool)
        .output_schema(WeatherReport::output_schema())
        .build()
        .unwrap();

    let out = agent.run("weather?").await.unwrap();
    assert_eq!(out.city, "Taipei");
    assert_eq!(out.temp_c, 24);
}

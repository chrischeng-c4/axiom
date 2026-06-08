//! Tracing/observability surface for agentkit (#2057).
//!
//! Wires the `tracing` crate into the agent runtime so that every
//! `Agent::run`, LLM call, and tool dispatch emits a structured span.
//! Span attributes mirror PydanticAI's Logfire fields (`agent.model`,
//! `agent.tools`, `tool.name`, `tool.call_id`, `llm.model`,
//! `llm.message_count`) so downstream OTel collectors can be configured
//! once across both ecosystems.
//!
//! ## Subscribers
//!
//! This module ships two helpers:
//!
//! * [`init_stdout_subscriber`] — installs a `tracing-subscriber` fmt
//!   layer for local development. Idempotent; safe to call from `main`
//!   or test setup.
//!
//! * OTLP / production export is left to the caller: depend on
//!   `tracing-opentelemetry` and attach the exporter layer to the
//!   `tracing_subscriber::Registry`. We do not pull the OTLP crates
//!   into the core graph; this keeps the dependency surface tight and
//!   lets the host process choose the transport (gRPC, HTTP, batch
//!   vs. simple, etc.).
//!
//! The instrumentation in `Agent` / `Graph` is unconditional — spans
//! are recorded regardless of which subscriber is installed. When no
//! subscriber is attached, `tracing` discards the events at zero cost.

// HANDWRITE-BEGIN reason: same Epic-3 gap — no rust-runtime generator
// for a structured-observability bootstrap module. Once the codegen
// pipeline learns to emit `tracing` spans from the lifecycle graph,
// this module can be regenerated from the same source.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{NovaError, NovaResult};
use crate::events::AgentEvent;

/// Shared identifiers and labels attached to exported trace records.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceContext {
    pub trace_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
}

impl TraceContext {
    pub fn new(trace_id: impl Into<String>, run_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            run_id: run_id.into(),
            user_id: None,
            session_id: None,
            metadata: Map::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Observation family for a trace export record.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceRecordKind {
    Trace,
    Generation,
    Tool,
    Event,
}

/// Langfuse-style neutral trace envelope for agent events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TraceRecord {
    pub trace_id: String,
    pub run_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub kind: TraceRecordKind,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Value>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Convert one typed agent lifecycle event into a serializable trace record.
pub fn agent_event_to_trace_record(context: &TraceContext, event: &AgentEvent) -> TraceRecord {
    match event {
        AgentEvent::RunStarted {
            model,
            timestamp_ms,
        } => record(
            context,
            TraceRecordKind::Trace,
            "agent.run",
            Some(*timestamp_ms),
            event_metadata([("model", Value::String(model.clone()))]),
            None,
        ),
        AgentEvent::LlmCallStarted {
            turn,
            message_count,
        } => record(
            context,
            TraceRecordKind::Generation,
            "llm.call",
            None,
            event_metadata([
                ("turn", Value::from(*turn)),
                ("message_count", Value::from(*message_count as u64)),
            ]),
            None,
        ),
        AgentEvent::LlmCallCompleted { turn, tool_calls } => record(
            context,
            TraceRecordKind::Generation,
            "llm.call.completed",
            None,
            event_metadata([
                ("turn", Value::from(*turn)),
                ("tool_calls", Value::from(*tool_calls as u64)),
            ]),
            None,
        ),
        AgentEvent::ToolCallStarted { tool, call_id } => record(
            context,
            TraceRecordKind::Tool,
            format!("tool.{tool}.started"),
            None,
            event_metadata([
                ("tool", Value::String(tool.clone())),
                ("call_id", Value::String(call_id.clone())),
            ]),
            None,
        ),
        AgentEvent::ToolCallCompleted { tool, call_id } => record(
            context,
            TraceRecordKind::Tool,
            format!("tool.{tool}.completed"),
            None,
            event_metadata([
                ("tool", Value::String(tool.clone())),
                ("call_id", Value::String(call_id.clone())),
            ]),
            None,
        ),
        AgentEvent::ToolCallFailed {
            tool,
            call_id,
            error,
        } => record(
            context,
            TraceRecordKind::Tool,
            format!("tool.{tool}.failed"),
            None,
            event_metadata([
                ("tool", Value::String(tool.clone())),
                ("call_id", Value::String(call_id.clone())),
            ]),
            Some(error.clone()),
        ),
        AgentEvent::ValidationFailed { revision, error } => record(
            context,
            TraceRecordKind::Event,
            "validation.failed",
            None,
            event_metadata([("revision", Value::from(*revision))]),
            Some(error.clone()),
        ),
        AgentEvent::RunCompleted {
            turns_used,
            revisions,
        } => record(
            context,
            TraceRecordKind::Trace,
            "agent.run.completed",
            None,
            event_metadata([
                ("turns_used", Value::from(*turns_used)),
                ("revisions", Value::from(*revisions)),
            ]),
            None,
        ),
        AgentEvent::RunFailed {
            turns_used,
            revisions,
            error,
        } => record(
            context,
            TraceRecordKind::Trace,
            "agent.run.failed",
            None,
            event_metadata([
                ("turns_used", Value::from(*turns_used)),
                ("revisions", Value::from(*revisions)),
            ]),
            Some(error.clone()),
        ),
    }
}

/// Convert a batch of agent events into trace records sharing one context.
pub fn agent_events_to_trace_records(
    context: &TraceContext,
    events: &[AgentEvent],
) -> Vec<TraceRecord> {
    events
        .iter()
        .map(|event| agent_event_to_trace_record(context, event))
        .collect()
}

fn record(
    context: &TraceContext,
    kind: TraceRecordKind,
    name: impl Into<String>,
    timestamp_ms: Option<u64>,
    event_metadata: Map<String, Value>,
    error: Option<String>,
) -> TraceRecord {
    let mut metadata = context.metadata.clone();
    metadata.extend(event_metadata);
    TraceRecord {
        trace_id: context.trace_id.clone(),
        run_id: context.run_id.clone(),
        user_id: context.user_id.clone(),
        session_id: context.session_id.clone(),
        kind,
        name: name.into(),
        timestamp_ms,
        input: None,
        output: None,
        metadata,
        error,
    }
}

fn event_metadata<const N: usize>(entries: [(&str, Value); N]) -> Map<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

/// Install a `tracing-subscriber` fmt layer that writes to stdout.
///
/// Filters by the `RUST_LOG` environment variable when set, otherwise
/// defaults to `info`. Returns `Ok(())` if this call installed the
/// global subscriber, or wraps the subscriber-already-set error as
/// [`NovaError::ConfigError`] so the caller can decide whether a
/// pre-existing subscriber is fatal.
pub fn init_stdout_subscriber() -> NovaResult<()> {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(|e| NovaError::ConfigError(format!("tracing subscriber already set: {e}")))
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_stdout_subscriber_is_idempotent_on_double_call() {
        // First call may succeed or fail depending on whether another
        // test in the same binary already installed a subscriber; the
        // contract under test is that the *second* call returns a typed
        // ConfigError rather than panicking.
        let _ = init_stdout_subscriber();
        let result = init_stdout_subscriber();
        assert!(matches!(result, Err(NovaError::ConfigError(_))));
    }

    #[test]
    fn llm_event_maps_to_generation_trace_record() {
        let context = TraceContext::new("trace-1", "run-1")
            .with_user_id("user-1")
            .with_session_id("session-1")
            .with_metadata("env", "test");

        let record = agent_event_to_trace_record(
            &context,
            &AgentEvent::LlmCallStarted {
                turn: 2,
                message_count: 4,
            },
        );

        assert_eq!(record.trace_id, "trace-1");
        assert_eq!(record.run_id, "run-1");
        assert_eq!(record.user_id.as_deref(), Some("user-1"));
        assert_eq!(record.session_id.as_deref(), Some("session-1"));
        assert_eq!(record.kind, TraceRecordKind::Generation);
        assert_eq!(record.name, "llm.call");
        assert_eq!(
            record.metadata.get("env"),
            Some(&Value::String("test".into()))
        );
        assert_eq!(record.metadata.get("turn"), Some(&Value::from(2)));
        assert_eq!(record.metadata.get("message_count"), Some(&Value::from(4)));
        assert_eq!(record.error, None);
    }

    #[test]
    fn trace_record_batch_serializes_failure_envelope() {
        let context = TraceContext::new("trace-1", "run-1");
        let records = agent_events_to_trace_records(
            &context,
            &[AgentEvent::ToolCallFailed {
                tool: "search".into(),
                call_id: "call-1".into(),
                error: "timeout".into(),
            }],
        );

        let json = serde_json::to_value(&records).unwrap();
        assert_eq!(json[0]["trace_id"], "trace-1");
        assert_eq!(json[0]["run_id"], "run-1");
        assert_eq!(json[0]["kind"], "tool");
        assert_eq!(json[0]["name"], "tool.search.failed");
        assert_eq!(json[0]["metadata"]["tool"], "search");
        assert_eq!(json[0]["metadata"]["call_id"], "call-1");
        assert_eq!(json[0]["error"], "timeout");
        assert!(json[0].get("user_id").is_none());
    }
}

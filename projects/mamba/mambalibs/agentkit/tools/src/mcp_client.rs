//! agentkit MCP client (#2063).
//!
//! Implements the client half of the Model Context Protocol so agentkit
//! agents can consume tools published by an external MCP server as if
//! they were native `AgentTool`s.
//!
//! The wire format is JSON-RPC 2.0 over a pluggable, line-delimited
//! transport. We implement the three MCP methods the agent loop actually
//! needs in a first slice:
//!
//! | Method          | Used for                                                    |
//! |-----------------|-------------------------------------------------------------|
//! | `initialize`    | Negotiate protocol version + advertise client capabilities  |
//! | `tools/list`    | Discover the server's tool catalog                          |
//! | `tools/call`    | Invoke a discovered tool by name with JSON args             |
//!
//! Higher-level features (resources, prompts, sampling, notifications)
//! are deferred to follow-up issues; the transport trait is designed to
//! grow without breaking callers.
//
// HANDWRITE-BEGIN reason: no rust-runtime generator emits a JSON-RPC
// state machine + per-method typed wrapper yet. Closing this gap
// requires a `jsonrpc-client` section type (planned alongside the
// `mcp-server` issue #2064 — both can share the same generator once it
// lands).

use std::sync::Mutex;

use agent::{NovaError, NovaResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// JSON-RPC 2.0 protocol literal — every request/response we emit
/// carries this exact string in the `jsonrpc` field.
pub const JSONRPC_VERSION: &str = "2.0";

/// Protocol version we advertise during `initialize`. Servers that
/// don't recognize it should fall back to their newest supported
/// version, which we accept verbatim on the `initialize` response.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Transport for an MCP session.
///
/// Implementations move framed JSON-RPC envelopes between the client
/// and the server. The contract is intentionally minimal so a real
/// stdio child-process transport, a unit-test in-memory transport, or
/// a future SSE transport can all satisfy it.
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send one JSON-RPC envelope to the server.
    async fn send(&self, message: Value) -> NovaResult<()>;

    /// Block until the server emits the next JSON-RPC envelope.
    async fn recv(&self) -> NovaResult<Value>;
}

/// Tool advertised by an MCP server (subset of the MCP `Tool` schema
/// — only the fields a client needs to dispatch a call).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTool {
    /// Stable name used as the `tools/call` `name` argument.
    pub name: String,
    /// Human-readable description; surfaced to the LLM as the tool
    /// description when re-published into an agentkit `ToolRegistry`.
    #[serde(default)]
    pub description: Option<String>,
    /// JSON Schema for the tool's input.
    #[serde(rename = "inputSchema", default = "default_schema")]
    pub input_schema: Value,
}

fn default_schema() -> Value {
    json!({ "type": "object" })
}

/// Result of a `tools/call` invocation. MCP returns an array of
/// `content` blocks (text, image, resource, ...); for the first slice
/// we expose the raw array verbatim so callers can render any
/// content type, and surface the `isError` flag separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallResult {
    /// Content blocks returned by the tool. Each item is a typed MCP
    /// content envelope (`{"type":"text","text":"..."}` etc.).
    #[serde(default)]
    pub content: Vec<Value>,
    /// Server-set flag indicating the tool itself returned an error
    /// (as opposed to a transport / protocol error, which is mapped
    /// to `NovaError`).
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// MCP client over a pluggable transport.
pub struct McpClient {
    transport: Box<dyn McpTransport>,
    next_id: Mutex<u64>,
}

impl McpClient {
    /// Build a new client bound to `transport`. Does not yet send any
    /// bytes; the caller must call [`McpClient::initialize`] before
    /// any other method.
    pub fn new<T: McpTransport + 'static>(transport: T) -> Self {
        Self {
            transport: Box::new(transport),
            next_id: Mutex::new(1),
        }
    }

    fn next_request_id(&self) -> u64 {
        let mut g = self.next_id.lock().expect("next_id mutex poisoned");
        let id = *g;
        *g += 1;
        id
    }

    async fn request(&self, method: &str, params: Value) -> NovaResult<Value> {
        let id = self.next_request_id();
        let envelope = json!({
            "jsonrpc": JSONRPC_VERSION,
            "id": id,
            "method": method,
            "params": params,
        });
        self.transport.send(envelope).await?;

        let response = self.transport.recv().await?;

        if response.get("jsonrpc").and_then(Value::as_str) != Some(JSONRPC_VERSION) {
            return Err(NovaError::InvalidRequest(format!(
                "MCP response missing jsonrpc=\"{JSONRPC_VERSION}\""
            )));
        }
        if response.get("id").and_then(Value::as_u64) != Some(id) {
            return Err(NovaError::InvalidRequest(format!(
                "MCP response id mismatch for method {method}"
            )));
        }
        if let Some(err) = response.get("error") {
            let message = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("<no message>");
            let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
            return Err(NovaError::ToolError(format!(
                "MCP error {code} on {method}: {message}"
            )));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| NovaError::MalformedLLMResponse(format!("{method} missing result")))
    }

    /// Negotiate protocol version + capabilities with the server.
    ///
    /// Returns the protocol version the server selected (typically
    /// `MCP_PROTOCOL_VERSION`, but a polite server may downgrade).
    pub async fn initialize(&self) -> NovaResult<String> {
        let params = json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "clientInfo": { "name": "agentkit", "version": env!("CARGO_PKG_VERSION") },
        });
        let result = self.request("initialize", params).await?;
        result
            .get("protocolVersion")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| {
                NovaError::MalformedLLMResponse(
                    "initialize result missing protocolVersion".to_string(),
                )
            })
    }

    /// Fetch the server's tool catalog.
    pub async fn list_tools(&self) -> NovaResult<Vec<McpTool>> {
        let result = self.request("tools/list", json!({})).await?;
        let tools = result
            .get("tools")
            .ok_or_else(|| {
                NovaError::MalformedLLMResponse("tools/list missing tools array".to_string())
            })?
            .clone();
        serde_json::from_value(tools).map_err(|e| {
            NovaError::MalformedLLMResponse(format!("tools/list: bad tool entry: {e}"))
        })
    }

    /// Invoke a tool by name with `arguments` as the input payload.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> NovaResult<McpToolCallResult> {
        let result = self
            .request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await?;
        serde_json::from_value(result)
            .map_err(|e| NovaError::MalformedLLMResponse(format!("tools/call: bad result: {e}")))
    }
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex as StdMutex;

    /// Scripted in-memory transport. Each `send()` records the
    /// envelope it observed; each `recv()` returns the next canned
    /// response. Lets tests drive the full JSON-RPC roundtrip
    /// without spawning a child process.
    struct ScriptedTransport {
        sent: StdMutex<Vec<Value>>,
        responses: StdMutex<VecDeque<Value>>,
    }

    impl ScriptedTransport {
        fn new(responses: Vec<Value>) -> Self {
            Self {
                sent: StdMutex::new(Vec::new()),
                responses: StdMutex::new(VecDeque::from(responses)),
            }
        }
    }

    #[async_trait]
    impl McpTransport for ScriptedTransport {
        async fn send(&self, message: Value) -> NovaResult<()> {
            self.sent.lock().unwrap().push(message);
            Ok(())
        }

        async fn recv(&self) -> NovaResult<Value> {
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| NovaError::ConfigError("script exhausted".into()))
        }
    }

    fn ok_response(id: u64, result: Value) -> Value {
        json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "result": result })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn initialize_list_call_drives_full_handshake() {
        let transport = ScriptedTransport::new(vec![
            ok_response(1, json!({ "protocolVersion": MCP_PROTOCOL_VERSION })),
            ok_response(
                2,
                json!({
                    "tools": [
                        {
                            "name": "echo",
                            "description": "echo the input back",
                            "inputSchema": { "type": "object" }
                        }
                    ]
                }),
            ),
            ok_response(
                3,
                json!({
                    "content": [{ "type": "text", "text": "hello" }],
                    "isError": false
                }),
            ),
        ]);

        let client = McpClient::new(transport);

        let version = client.initialize().await.unwrap();
        assert_eq!(version, MCP_PROTOCOL_VERSION);

        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");
        assert_eq!(tools[0].description.as_deref(), Some("echo the input back"));

        let result = client
            .call_tool("echo", json!({ "msg": "hello" }))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0]["text"], "hello");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn jsonrpc_error_response_is_mapped_to_nova_tool_error() {
        let transport = ScriptedTransport::new(vec![json!({
            "jsonrpc": JSONRPC_VERSION,
            "id": 1,
            "error": { "code": -32601, "message": "method not found" }
        })]);
        let client = McpClient::new(transport);
        let err = client.initialize().await.unwrap_err();
        match err {
            NovaError::ToolError(msg) => {
                assert!(msg.contains("-32601"), "got: {msg}");
                assert!(msg.contains("method not found"), "got: {msg}");
            }
            other => panic!("expected ToolError, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn response_id_mismatch_is_rejected() {
        let transport = ScriptedTransport::new(vec![ok_response(
            42,
            json!({ "protocolVersion": MCP_PROTOCOL_VERSION }),
        )]);
        let client = McpClient::new(transport);
        let err = client.initialize().await.unwrap_err();
        assert!(matches!(err, NovaError::InvalidRequest(_)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn list_tools_parses_minimal_entry_without_description() {
        let transport = ScriptedTransport::new(vec![
            ok_response(1, json!({ "protocolVersion": MCP_PROTOCOL_VERSION })),
            ok_response(2, json!({ "tools": [ { "name": "ping" } ] })),
        ]);
        let client = McpClient::new(transport);
        client.initialize().await.unwrap();
        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools[0].name, "ping");
        assert!(tools[0].description.is_none());
        assert_eq!(tools[0].input_schema, json!({ "type": "object" }));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn round_trip_preserves_mcp_tool_call_result_serde() {
        let result = McpToolCallResult {
            content: vec![json!({ "type": "text", "text": "ok" })],
            is_error: true,
        };
        let s = serde_json::to_string(&result).unwrap();
        assert!(s.contains("\"isError\":true"));
        let back: McpToolCallResult = serde_json::from_str(&s).unwrap();
        assert_eq!(back, result);
    }
}

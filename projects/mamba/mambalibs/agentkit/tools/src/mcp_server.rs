//! agentkit MCP server (#2064).
//!
//! Symmetric counterpart to [`crate::mcp_client`]. Lets agentkit
//! publish its own tools to any MCP-aware client (Claude Desktop,
//! Cursor, etc.).
//!
//! The server is built around a single entry point — [`McpServer::handle_request`] —
//! which takes one JSON-RPC envelope and returns the response envelope.
//! Wiring this to a transport (stdio, SSE, websocket) is the caller's
//! concern; a thin `serve_stdio_loop` driver can be added later
//! without touching the protocol implementation.
//!
//! Methods implemented in this first slice:
//!
//! | Method        | Semantics                                                 |
//! |---------------|-----------------------------------------------------------|
//! | `initialize`  | Echo server info + agreed protocol version                |
//! | `tools/list`  | Return the registered tool catalog                        |
//! | `tools/call`  | Dispatch to the handler registered under `params.name`    |
//
// HANDWRITE-BEGIN reason: shares the JSON-RPC server gap with #2063 —
// no rust-runtime generator emits a typed JSON-RPC method router yet.
// Both will move into CODEGEN markers once the `jsonrpc-server`
// section type lands.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use agent::{NovaError, NovaResult};
use serde_json::{json, Value};

use crate::mcp_client::{McpTool, McpToolCallResult, JSONRPC_VERSION, MCP_PROTOCOL_VERSION};

/// JSON-RPC error codes we emit (matching the JSON-RPC 2.0 spec for
/// the cases that have a standard code).
pub const ERR_METHOD_NOT_FOUND: i64 = -32601;
pub const ERR_INVALID_PARAMS: i64 = -32602;
pub const ERR_INTERNAL: i64 = -32603;

/// Future returned by an [`McpToolHandler`].
pub type McpHandlerFuture =
    Pin<Box<dyn Future<Output = NovaResult<McpToolCallResult>> + Send + 'static>>;

/// Server-side counterpart to a single MCP tool. The handler receives
/// the JSON arguments passed in `tools/call` and returns either a
/// successful [`McpToolCallResult`] or a typed [`NovaError`].
pub trait McpToolHandler: Send + Sync {
    fn invoke(&self, arguments: Value) -> McpHandlerFuture;
}

/// Adapter that turns any `Fn(Value) -> Future` into an
/// [`McpToolHandler`].
pub struct FnHandler<F>(pub F);

impl<F, Fut> McpToolHandler for FnHandler<F>
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = NovaResult<McpToolCallResult>> + Send + 'static,
{
    fn invoke(&self, arguments: Value) -> McpHandlerFuture {
        Box::pin((self.0)(arguments))
    }
}

struct Registered {
    spec: McpTool,
    handler: Arc<dyn McpToolHandler>,
}

/// MCP server. Holds a tool catalog and dispatches JSON-RPC envelopes.
pub struct McpServer {
    name: String,
    version: String,
    tools: HashMap<String, Registered>,
}

impl McpServer {
    /// Build a server that identifies itself as `name @ version` in
    /// the `initialize` response.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tools: HashMap::new(),
        }
    }

    /// Register a tool spec + handler. Replaces any prior registration
    /// with the same name; returns the previous spec for inspection.
    pub fn register<H: McpToolHandler + 'static>(
        &mut self,
        spec: McpTool,
        handler: H,
    ) -> Option<McpTool> {
        let prev = self.tools.remove(&spec.name).map(|r| r.spec);
        self.tools.insert(
            spec.name.clone(),
            Registered {
                spec,
                handler: Arc::new(handler),
            },
        );
        prev
    }

    /// Convenience overload: register a free async function as a
    /// handler without the caller having to wrap it in [`FnHandler`].
    pub fn register_fn<F, Fut>(&mut self, spec: McpTool, f: F) -> Option<McpTool>
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NovaResult<McpToolCallResult>> + Send + 'static,
    {
        self.register(spec, FnHandler(f))
    }

    /// Handle one JSON-RPC envelope. Always returns a JSON-RPC
    /// response envelope (success or error); transport errors raised
    /// from the carrier are the caller's problem.
    pub async fn handle_request(&self, request: Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = match request.get("method").and_then(Value::as_str) {
            Some(m) => m,
            None => return jsonrpc_error(id, ERR_INVALID_PARAMS, "missing method"),
        };
        let params = request.get("params").cloned().unwrap_or(json!({}));

        let result = match method {
            "initialize" => Ok(self.handle_initialize()),
            "tools/list" => Ok(self.handle_tools_list()),
            "tools/call" => self.handle_tools_call(params).await,
            other => {
                return jsonrpc_error(
                    id,
                    ERR_METHOD_NOT_FOUND,
                    &format!("unknown method: {other}"),
                );
            }
        };

        match result {
            Ok(value) => json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "result": value }),
            Err(NovaError::InvalidArguments(msg))
            | Err(NovaError::ValidationFailed(msg))
            | Err(NovaError::InvalidRequest(msg)) => jsonrpc_error(id, ERR_INVALID_PARAMS, &msg),
            Err(NovaError::ToolNotFound(msg)) => jsonrpc_error(id, ERR_METHOD_NOT_FOUND, &msg),
            Err(err) => jsonrpc_error(id, ERR_INTERNAL, &err.to_string()),
        }
    }

    fn handle_initialize(&self) -> Value {
        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": self.name, "version": self.version },
        })
    }

    fn handle_tools_list(&self) -> Value {
        let tools: Vec<&McpTool> = self.tools.values().map(|r| &r.spec).collect();
        json!({ "tools": tools })
    }

    async fn handle_tools_call(&self, params: Value) -> NovaResult<Value> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| NovaError::InvalidArguments("tools/call: missing name".into()))?;
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let registered = self
            .tools
            .get(name)
            .ok_or_else(|| NovaError::ToolNotFound(format!("tools/call: {name}")))?;

        let result = registered.handler.invoke(arguments).await?;
        serde_json::to_value(&result)
            .map_err(|e| NovaError::ConfigError(format!("failed to serialize tool result: {e}")))
    }
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": { "code": code, "message": message },
    })
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn echo_tool() -> McpTool {
        McpTool {
            name: "echo".into(),
            description: Some("echo args verbatim".into()),
            input_schema: json!({ "type": "object" }),
        }
    }

    fn req(id: u64, method: &str, params: Value) -> Value {
        json!({ "jsonrpc": JSONRPC_VERSION, "id": id, "method": method, "params": params })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn initialize_returns_protocol_and_server_info() {
        let server = McpServer::new("agentkit", "1.2.3");
        let resp = server.handle_request(req(1, "initialize", json!({}))).await;
        assert_eq!(resp["jsonrpc"], JSONRPC_VERSION);
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "agentkit");
        assert_eq!(resp["result"]["serverInfo"]["version"], "1.2.3");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tools_list_returns_registered_catalog() {
        let mut server = McpServer::new("agentkit", "0.1.0");
        server.register_fn(echo_tool(), |args| async move {
            Ok(McpToolCallResult {
                content: vec![json!({ "type": "text", "text": args.to_string() })],
                is_error: false,
            })
        });

        let resp = server.handle_request(req(2, "tools/list", json!({}))).await;
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "echo");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tools_call_dispatches_to_handler() {
        let mut server = McpServer::new("agentkit", "0.1.0");
        server.register_fn(echo_tool(), |args| async move {
            Ok(McpToolCallResult {
                content: vec![json!({ "type": "text", "text": args["msg"].clone() })],
                is_error: false,
            })
        });

        let resp = server
            .handle_request(req(
                3,
                "tools/call",
                json!({ "name": "echo", "arguments": { "msg": "hi" } }),
            ))
            .await;
        assert_eq!(resp["result"]["isError"], false);
        assert_eq!(resp["result"]["content"][0]["text"], "hi");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tools_call_unknown_tool_yields_method_not_found() {
        let server = McpServer::new("agentkit", "0.1.0");
        let resp = server
            .handle_request(req(
                4,
                "tools/call",
                json!({ "name": "missing", "arguments": {} }),
            ))
            .await;
        assert_eq!(resp["error"]["code"], ERR_METHOD_NOT_FOUND);
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("missing"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn handler_error_is_mapped_to_jsonrpc_internal() {
        let mut server = McpServer::new("agentkit", "0.1.0");
        server.register_fn(echo_tool(), |_args| async move {
            Err(NovaError::ToolError("the tool exploded".into()))
        });
        let resp = server
            .handle_request(req(
                5,
                "tools/call",
                json!({ "name": "echo", "arguments": {} }),
            ))
            .await;
        assert_eq!(resp["error"]["code"], ERR_INTERNAL);
        assert!(resp["error"]["message"]
            .as_str()
            .unwrap()
            .contains("the tool exploded"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unknown_method_yields_method_not_found() {
        let server = McpServer::new("agentkit", "0.1.0");
        let resp = server
            .handle_request(req(6, "nope/please", json!({})))
            .await;
        assert_eq!(resp["error"]["code"], ERR_METHOD_NOT_FOUND);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tools_call_without_name_yields_invalid_params() {
        let server = McpServer::new("agentkit", "0.1.0");
        let resp = server.handle_request(req(7, "tools/call", json!({}))).await;
        assert_eq!(resp["error"]["code"], ERR_INVALID_PARAMS);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn register_replaces_existing_tool() {
        let mut server = McpServer::new("agentkit", "0.1.0");
        let prev = server.register_fn(echo_tool(), |_| async move {
            Ok(McpToolCallResult {
                content: vec![],
                is_error: false,
            })
        });
        assert!(prev.is_none());

        let mut updated = echo_tool();
        updated.description = Some("v2".into());
        let prev = server.register_fn(updated, |_| async move {
            Ok(McpToolCallResult {
                content: vec![],
                is_error: false,
            })
        });
        assert_eq!(
            prev.unwrap().description.as_deref(),
            Some("echo args verbatim")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn client_and_server_round_trip_through_in_memory_pipe() {
        // Sanity check: the server's response envelope is exactly
        // what the client expects to parse. We feed handle_request
        // output back through the McpClient's transport layer.
        use crate::mcp_client::{McpClient, McpTransport};
        use async_trait::async_trait;
        use std::sync::Mutex as StdMutex;

        struct LoopbackTransport {
            server: Arc<McpServer>,
            pending: StdMutex<Option<Value>>,
        }

        #[async_trait]
        impl McpTransport for LoopbackTransport {
            async fn send(&self, message: Value) -> NovaResult<()> {
                let response = self.server.handle_request(message).await;
                *self.pending.lock().unwrap() = Some(response);
                Ok(())
            }
            async fn recv(&self) -> NovaResult<Value> {
                self.pending
                    .lock()
                    .unwrap()
                    .take()
                    .ok_or_else(|| NovaError::ConfigError("no pending response".into()))
            }
        }

        let mut server = McpServer::new("agentkit", "1.0.0");
        server.register_fn(echo_tool(), |args| async move {
            Ok(McpToolCallResult {
                content: vec![json!({ "type": "text", "text": args["msg"].clone() })],
                is_error: false,
            })
        });

        let transport = LoopbackTransport {
            server: Arc::new(server),
            pending: StdMutex::new(None),
        };
        let client = McpClient::new(transport);

        let v = client.initialize().await.unwrap();
        assert_eq!(v, MCP_PROTOCOL_VERSION);

        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools[0].name, "echo");

        let r = client
            .call_tool("echo", json!({ "msg": "yo" }))
            .await
            .unwrap();
        assert_eq!(r.content[0]["text"], "yo");
    }
}

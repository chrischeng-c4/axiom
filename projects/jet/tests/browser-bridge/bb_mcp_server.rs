// <HANDWRITE gap="codegen:jsonrpc-server-loop" tracker="jet-bb-mcp-server" reason="Companion integration test for the handwritten MCP stdio server; becomes CODEGEN with the server module.">
//! `jet bb mcp` MCP stdio protocol contract.
//!
//! Spawns the real `jet` binary and speaks newline-delimited JSON-RPC
//! over stdin/stdout, the way an MCP client (agent harness) does:
//! initialize → initialized → tools/list → tools/call. No browser is
//! launched: tool calls that need a session must fail as MCP tool
//! errors (`isError: true`), never as protocol errors or stdout noise.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl McpClient {
    fn spawn(root_dir: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_jet"))
            .args(["bb", "mcp"])
            .current_dir(root_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawning `jet bb mcp`");
        let stdin = child.stdin.take().expect("child stdin");
        let stdout = BufReader::new(child.stdout.take().expect("child stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, msg: &Value) {
        let mut line = serde_json::to_string(msg).expect("serializing request");
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .expect("writing JSON-RPC frame to jet bb mcp stdin");
        self.stdin.flush().expect("flushing stdin");
    }

    fn recv(&mut self) -> Value {
        let mut line = String::new();
        let n = self
            .stdout
            .read_line(&mut line)
            .expect("reading JSON-RPC frame from jet bb mcp stdout");
        assert!(n > 0, "jet bb mcp closed stdout before responding");
        serde_json::from_str(line.trim()).unwrap_or_else(|err| {
            panic!("stdout must carry only JSON-RPC frames; got {line:?} ({err})")
        })
    }

    fn request(&mut self, msg: &Value) -> Value {
        self.send(msg);
        self.recv()
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn handshake(client: &mut McpClient) -> Value {
    let init = client.request(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "bb-mcp-contract-test", "version": "0" },
        },
    }));
    client.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
    init
}

#[test]
fn mcp_handshake_advertises_bb_tool_surface() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = McpClient::spawn(tmp.path());

    let init = handshake(&mut client);
    assert_eq!(init["jsonrpc"], "2.0");
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "jet-bb");
    assert!(
        init["result"]["capabilities"]["tools"].is_object(),
        "server must advertise the tools capability: {init}"
    );

    let listed = client.request(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
    }));
    let tools = listed["result"]["tools"]
        .as_array()
        .expect("tools/list result must contain a tools array");
    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().expect("tool name"))
        .collect();
    for required in [
        "bb_launch",
        "bb_shutdown",
        "bb_tree",
        "bb_hooks",
        "bb_eval",
        "bb_capture",
        "bb_screenshot",
        "bb_mouse",
        "bb_drag",
        "bb_wheel",
        "bb_key",
        "bb_highlight",
        // Semantic surface (playwright-mcp-shaped): snapshot refs +
        // element-targeted actions + navigation + observability.
        "bb_snapshot",
        "bb_click",
        "bb_fill",
        "bb_type",
        "bb_hover",
        "bb_select",
        "bb_check",
        "bb_goto",
        "bb_back",
        "bb_forward",
        "bb_reload",
        "bb_resize",
        "bb_wait_for",
        "bb_console",
        "bb_requests",
    ] {
        assert!(names.contains(&required), "tools/list missing {required}: {names:?}");
    }
    for tool in tools {
        assert_eq!(
            tool["inputSchema"]["type"], "object",
            "every tool needs a JSON-schema object inputSchema: {tool}"
        );
    }
}

#[test]
fn tool_failures_are_mcp_tool_errors_not_protocol_errors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut client = McpClient::spawn(tmp.path());
    handshake(&mut client);

    // No Browser Bridge session exists in this tempdir, so the call must
    // surface the attach failure inside the tool result.
    let resp = client.request(&json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": { "name": "bb_eval", "arguments": { "expression": "1+1" } },
    }));
    assert!(resp.get("error").is_none(), "tool failure leaked as protocol error: {resp}");
    assert_eq!(resp["result"]["isError"], true);
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("tool error content text");
    assert!(!text.is_empty());

    // Unknown tool: same contract.
    let resp = client.request(&json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": { "name": "bb_does_not_exist", "arguments": {} },
    }));
    assert_eq!(resp["result"]["isError"], true);

    // Unknown method: that one IS a protocol error.
    let resp = client.request(&json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "resources/list",
    }));
    assert_eq!(resp["error"]["code"], -32601);

    // ping keeps working after errors.
    let resp = client.request(&json!({ "jsonrpc": "2.0", "id": 6, "method": "ping" }));
    assert!(resp["result"].is_object());
}
// </HANDWRITE>

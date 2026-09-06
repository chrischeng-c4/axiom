pub mod http;
pub mod protocol;
pub mod resources;
pub mod tools;

use anyhow::Result;
use protocol::{PROTOCOL_VERSION, SERVER_NAME, SERVER_VERSION};
use resources::{list_resources, read_resource};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tools::{call_tool, tool_definitions};

use crate::store::PmStore;

pub async fn serve(store: PmStore) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    eprintln!(
        "pm: MCP server started (server: {}, version: {}, store: {})",
        SERVER_NAME,
        SERVER_VERSION,
        store.path().display()
    );

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(err) => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("parse error: {err}") }
                });
                write_frame(&mut stdout, &resp).await?;
                continue;
            }
        };

        if let Some(resp) = handle_message(&store, &msg).await {
            write_frame(&mut stdout, &resp).await?;
        }
    }

    eprintln!("pm: MCP server shutting down (stdin closed).");
    Ok(())
}

pub async fn handle_message(store: &PmStore, msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let id = msg.get("id").cloned();

    // Notifications (no id) never get a response frame in JSON-RPC 2.0
    let id = match id {
        Some(id) if !id.is_null() => id,
        _ => return None,
    };

    let result = match method {
        "initialize" => Ok(initialize_result(msg)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_definitions() })),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

            match call_tool(store, name, &args).await {
                Ok(content) => Ok(json!({ "content": content, "isError": false })),
                Err(err) => Ok(json!({
                    "content": [ { "type": "text", "text": format!("{err:#}") } ],
                    "isError": true,
                })),
            }
        }
        "resources/list" => {
            let resources = list_resources(store).await;
            Ok(json!({ "resources": resources }))
        }
        "resources/read" => {
            let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
            let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");

            match read_resource(store, uri).await {
                Ok(res) => Ok(res),
                Err(err) => Err(json!({
                    "code": -32602,
                    "message": format!("{err:#}"),
                })),
            }
        }
        other => Err(json!({
            "code": -32601,
            "message": format!("method not found: {other}"),
        })),
    };

    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

fn initialize_result(msg: &Value) -> Value {
    let requested = msg
        .pointer("/params/protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION);

    let version = match requested {
        "2024-11-05" | "2025-03-26" | "2025-06-18" => requested,
        _ => PROTOCOL_VERSION,
    };

    json!({
        "protocolVersion": version,
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "listChanged": false }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "version": SERVER_VERSION
        }
    })
}

async fn write_frame(stdout: &mut tokio::io::Stdout, msg: &Value) -> Result<()> {
    let mut body = serde_json::to_vec(msg)?;
    body.push(b'\n');
    stdout.write_all(&body).await?;
    stdout.flush().await?;
    Ok(())
}

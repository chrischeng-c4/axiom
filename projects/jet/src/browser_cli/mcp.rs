// <HANDWRITE gap="codegen:jsonrpc-server-loop" tracker="jet-bb-mcp-server" reason="MCP stdio JSON-RPC server loop has no deterministic generator primitive yet; feed back into Agentic Workflow until it can become CODEGEN.">
//! `jet bb mcp` — serve the Browser Bridge as an MCP stdio server.
//!
//! Agents speak MCP (JSON-RPC 2.0 over newline-delimited stdio) and get
//! the same agent-first surface the `jet bb` CLI exposes: launch a
//! detached headless session, drive it (mouse/drag/wheel/key), observe
//! it (tree/hooks/eval/capture/screenshot), and shut it down.
//!
//! Invariant: stdout carries only JSON-RPC frames. Every handler returns
//! its result as a string; nothing in this module may `println!`.
//! Diagnostics go to stderr via `tracing`/`eprintln!` like the rest of
//! the Browser Bridge.

use anyhow::{Context, Result};
use base64::Engine;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::{pretty, session};

/// Latest MCP protocol revision this server understands. Echoed back
/// when the client requests an unknown revision.
const PROTOCOL_VERSION: &str = "2025-06-18";

const SERVER_NAME: &str = "jet-bb";

/// Run the MCP stdio server until stdin closes.
pub async fn serve(root_dir: &Path) -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

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
                    "error": { "code": -32700, "message": format!("parse error: {err}") },
                });
                write_frame(&mut stdout, &resp).await?;
                continue;
            }
        };
        if let Some(resp) = handle_message(root_dir, &msg).await {
            write_frame(&mut stdout, &resp).await?;
        }
    }
    Ok(())
}

async fn write_frame(stdout: &mut tokio::io::Stdout, msg: &Value) -> Result<()> {
    let mut body = serde_json::to_vec(msg)?;
    body.push(b'\n');
    stdout.write_all(&body).await?;
    stdout.flush().await?;
    Ok(())
}

/// Handle one JSON-RPC message. Returns `None` for notifications.
async fn handle_message(root_dir: &Path, msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
    let id = msg.get("id").cloned();

    // Notifications (no id) never get a response.
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
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            match call_tool(root_dir, name, &args).await {
                Ok(content) => Ok(json!({ "content": content, "isError": false })),
                Err(err) => Ok(json!({
                    "content": [ { "type": "text", "text": format!("{err:#}") } ],
                    "isError": true,
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
    // Echo the client's revision when it is one we can serve; the tool
    // surface here only depends on baseline tools/list + tools/call.
    let version = match requested {
        "2024-11-05" | "2025-03-26" | "2025-06-18" => requested,
        _ => PROTOCOL_VERSION,
    };
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
    })
}

fn tool_definitions() -> Vec<Value> {
    fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
        json!({
            "name": name,
            "description": description,
            "inputSchema": {
                "type": "object",
                "properties": props,
                "required": required,
            },
        })
    }
    vec![
        tool(
            "bb_launch",
            "Launch headless Chromium as a detached Browser Bridge session and return session metadata.",
            json!({ "url": { "type": "string", "description": "URL to open (typically the jet serve address)" } }),
            &["url"],
        ),
        tool(
            "bb_shutdown",
            "Request the active Browser Bridge session to close.",
            json!({}),
            &[],
        ),
        tool(
            "bb_tree",
            "Print the element, layout, or fiber tree from the attached session (requires a jet-wasm debug bridge page).",
            json!({ "which": { "type": "string", "enum": ["element", "layout", "fiber"], "default": "element" } }),
            &[],
        ),
        tool(
            "bb_hooks",
            "Print hook values for a fiber id (jet-wasm debug bridge).",
            json!({ "fiber_id": { "type": "integer" } }),
            &["fiber_id"],
        ),
        tool(
            "bb_eval",
            "Runtime.evaluate escape hatch; returns the JSON value of the expression.",
            json!({ "expression": { "type": "string" } }),
            &["expression"],
        ),
        tool(
            "bb_capture",
            "Capture a parity-ready JSON observation bundle from the attached session.",
            json!({
                "surface": { "type": "string", "enum": ["wasm", "dom"], "default": "wasm" },
                "root_selector": { "type": "string", "default": "body", "description": "DOM root selector for surface=dom" },
                "hook_ids": { "type": "array", "items": { "type": "integer" }, "description": "Fiber ids whose hook values to include (surface=wasm only)" },
            }),
            &[],
        ),
        tool(
            "bb_screenshot",
            "Capture a PNG of the current page. Returns the image inline, or writes to `out` when given.",
            json!({ "out": { "type": "string", "description": "Optional output file path" } }),
            &[],
        ),
        tool(
            "bb_mouse",
            "Dispatch one CDP mouse event at viewport CSS-pixel coordinates.",
            json!({
                "type": { "type": "string", "enum": ["mouseMoved", "mousePressed", "mouseReleased"] },
                "x": { "type": "number" },
                "y": { "type": "number" },
                "button": { "type": "string", "enum": ["left", "right", "middle", "none"] },
                "buttons": { "type": "integer", "description": "CDP buttons bitfield, e.g. 1 while dragging" },
                "click_count": { "type": "integer" },
            }),
            &["type", "x", "y"],
        ),
        tool(
            "bb_drag",
            "Drag between two viewport coordinates using interpolated CDP mouse events.",
            json!({
                "from_x": { "type": "number" },
                "from_y": { "type": "number" },
                "to_x": { "type": "number" },
                "to_y": { "type": "number" },
                "steps": { "type": "integer", "default": 8 },
            }),
            &["from_x", "from_y", "to_x", "to_y"],
        ),
        tool(
            "bb_wheel",
            "Dispatch one CDP mouse wheel event.",
            json!({
                "x": { "type": "number" },
                "y": { "type": "number" },
                "delta_x": { "type": "number", "default": 0 },
                "delta_y": { "type": "number", "default": 0 },
            }),
            &["x", "y"],
        ),
        tool(
            "bb_key",
            "Press one key (keyDown+keyUp) with optional modifiers.",
            json!({
                "key": { "type": "string", "description": "Key value such as c, Enter, or ArrowDown" },
                "ctrl": { "type": "boolean", "default": false },
                "meta": { "type": "boolean", "default": false },
                "shift": { "type": "boolean", "default": false },
                "alt": { "type": "boolean", "default": false },
            }),
            &["key"],
        ),
        tool(
            "bb_highlight",
            "Overlay a highlight rect on a layout-node index, or clear it.",
            json!({
                "index": { "type": "integer" },
                "clear": { "type": "boolean", "default": false },
            }),
            &[],
        ),
        tool(
            "bb_snapshot",
            "Capture a ref-annotated semantic snapshot of the live DOM. Each interactable element gets a ref (e1, e2, …) that bb_click/bb_fill/bb_type/bb_hover/bb_select accept; refs stay valid until the next snapshot or navigation.",
            json!({}),
            &[],
        ),
        tool(
            "bb_click",
            "Click an element by snapshot ref or selector (CSS, text=…, role=…[name=\"…\"]). Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string", "description": "Snapshot ref such as e12" },
                "selector": { "type": "string", "description": "CSS, text=…, or role=… selector" },
                "dblclick": { "type": "boolean", "default": false },
            }),
            &[],
        ),
        tool(
            "bb_fill",
            "Replace an input/textarea value (native setter + input/change events). Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string" },
                "selector": { "type": "string" },
                "text": { "type": "string" },
            }),
            &["text"],
        ),
        tool(
            "bb_type",
            "Focus an element and type text through the real CDP input pipeline (appends; use bb_fill to replace). Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string" },
                "selector": { "type": "string" },
                "text": { "type": "string" },
            }),
            &["text"],
        ),
        tool(
            "bb_hover",
            "Hover an element (mouseenter + mousemove at its center). Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string" },
                "selector": { "type": "string" },
            }),
            &[],
        ),
        tool(
            "bb_select",
            "Choose a <select> option by value or label. Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string" },
                "selector": { "type": "string" },
                "option": { "type": "string", "description": "Option value or label" },
            }),
            &["option"],
        ),
        tool(
            "bb_check",
            "Check or uncheck a checkbox idempotently. Pass exactly one of ref/selector.",
            json!({
                "ref": { "type": "string" },
                "selector": { "type": "string" },
                "checked": { "type": "boolean", "default": true },
            }),
            &[],
        ),
        tool(
            "bb_goto",
            "Navigate the attached session to a URL and wait for load.",
            json!({ "url": { "type": "string" } }),
            &["url"],
        ),
        tool(
            "bb_back",
            "Go back one entry in session history.",
            json!({}),
            &[],
        ),
        tool(
            "bb_forward",
            "Go forward one entry in session history.",
            json!({}),
            &[],
        ),
        tool(
            "bb_reload",
            "Reload the current document and wait for it to become ready.",
            json!({}),
            &[],
        ),
        tool(
            "bb_resize",
            "Resize the viewport (CDP device-metrics override).",
            json!({
                "width": { "type": "integer" },
                "height": { "type": "integer" },
            }),
            &["width", "height"],
        ),
        tool(
            "bb_wait_for",
            "Wait for a selector to attach, text to appear, or a fixed delay. Pass exactly one of selector/text/ms.",
            json!({
                "selector": { "type": "string" },
                "text": { "type": "string" },
                "ms": { "type": "integer" },
                "timeout_ms": { "type": "integer", "default": 10000 },
            }),
            &[],
        ),
        tool(
            "bb_console",
            "Console messages, page errors, and unhandled rejections captured since launch (init-script ring buffer).",
            json!({
                "level": { "type": "string", "enum": ["log", "info", "warn", "error", "debug"] },
                "limit": { "type": "integer", "default": 100 },
                "clear": { "type": "boolean", "default": false, "description": "Drain the buffer after reading" },
            }),
            &[],
        ),
        tool(
            "bb_requests",
            "fetch/XHR activity captured since launch (init-script ring buffer).",
            json!({
                "limit": { "type": "integer", "default": 100 },
                "clear": { "type": "boolean", "default": false, "description": "Drain the buffer after reading" },
            }),
            &[],
        ),
    ]
}

/// Resolve the `ref`/`selector` argument pair every element-targeted
/// tool shares. Exactly one must be present.
fn arg_target(args: &Value) -> Result<super::interact::Target> {
    let r = args.get("ref").and_then(Value::as_str);
    let sel = args.get("selector").and_then(Value::as_str);
    match (r, sel) {
        (Some(r), None) => super::interact::parse_target(&format!("ref={r}")),
        (None, Some(sel)) => Ok(super::interact::Target::Selector(sel.to_string())),
        (Some(_), Some(_)) => anyhow::bail!("pass ref or selector, not both"),
        (None, None) => anyhow::bail!("missing argument: ref or selector"),
    }
}

fn text_content(text: String) -> Vec<Value> {
    vec![json!({ "type": "text", "text": text })]
}

fn arg_f64(args: &Value, key: &str) -> Result<f64> {
    args.get(key)
        .and_then(Value::as_f64)
        .with_context(|| format!("missing or non-numeric argument: {key}"))
}

async fn call_tool(root_dir: &Path, name: &str, args: &Value) -> Result<Vec<Value>> {
    match name {
        "bb_launch" => {
            let url = args
                .get("url")
                .and_then(Value::as_str)
                .context("missing argument: url")?;
            session::clear_shutdown_request(root_dir);
            let browser =
                super::prepare_session_with_mode(root_dir, url, &[], session::MODE_DETACHED)
                    .await?;
            let s = session::read(root_dir).context("reading just-written browser session")?;
            browser.detach();
            Ok(text_content(serde_json::to_string_pretty(&json!({
                "schema_version": "jet.bb.session.v1",
                "mode": "detached",
                "url": s.url,
                "ws_endpoint": s.ws_endpoint,
                "target_id": s.target_id,
                "pid": s.pid,
                "session_file": session::session_path(root_dir).display().to_string(),
            }))?))
        }
        "bb_shutdown" => {
            super::shutdown(root_dir).await?;
            Ok(text_content(
                json!({ "ok": true, "action": "shutdown_requested" }).to_string(),
            ))
        }
        "bb_tree" => {
            let which = args
                .get("which")
                .and_then(Value::as_str)
                .unwrap_or("element");
            let page = super::attach(root_dir).await?;
            super::assert_debug_bridge(&page).await?;
            let (method, printer): (&str, fn(&Value) -> String) = match which {
                "element" => ("elementTree", pretty::element_tree),
                "layout" => ("layoutTree", pretty::layout_tree),
                "fiber" => ("fiberTree", pretty::fiber_tree),
                other => {
                    anyhow::bail!("unknown tree kind {other:?} — use element | layout | fiber")
                }
            };
            let v = page.evaluate(&super::expr(method, "")).await?;
            Ok(text_content(printer(&v)))
        }
        "bb_hooks" => {
            let fiber_id = args
                .get("fiber_id")
                .and_then(Value::as_u64)
                .context("missing or non-integer argument: fiber_id")?;
            let page = super::attach(root_dir).await?;
            super::assert_debug_bridge(&page).await?;
            let v = page
                .evaluate(&super::expr("hookValues", &fiber_id.to_string()))
                .await?;
            Ok(text_content(pretty::hook_values(&v)))
        }
        "bb_eval" => {
            let expression = args
                .get("expression")
                .and_then(Value::as_str)
                .context("missing argument: expression")?;
            let page = super::attach(root_dir).await?;
            let v = page.evaluate(expression).await?;
            Ok(text_content(serde_json::to_string_pretty(&v)?))
        }
        "bb_capture" => {
            let surface = args
                .get("surface")
                .and_then(Value::as_str)
                .unwrap_or("wasm");
            let bundle = match surface {
                "wasm" => {
                    let hook_ids: Vec<u64> = args
                        .get("hook_ids")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_u64).collect())
                        .unwrap_or_default();
                    super::observation_bundle(root_dir, &hook_ids).await?
                }
                "dom" => {
                    let root_selector = args
                        .get("root_selector")
                        .and_then(Value::as_str)
                        .unwrap_or("body");
                    super::dom_observation_bundle(root_dir, root_selector).await?
                }
                other => {
                    anyhow::bail!("unknown browser capture surface {other:?}; expected wasm or dom")
                }
            };
            Ok(text_content(serde_json::to_string(&bundle)?))
        }
        "bb_screenshot" => {
            let page = super::attach(root_dir).await?;
            let bytes = page.screenshot().await.context("capturing screenshot")?;
            match args.get("out").and_then(Value::as_str) {
                Some(out) => {
                    let path = PathBuf::from(out);
                    std::fs::write(&path, &bytes)
                        .with_context(|| format!("writing {}", path.display()))?;
                    Ok(text_content(
                        json!({ "ok": true, "path": path.display().to_string(), "bytes": bytes.len() })
                            .to_string(),
                    ))
                }
                None => Ok(vec![json!({
                    "type": "image",
                    "data": base64::engine::general_purpose::STANDARD.encode(&bytes),
                    "mimeType": "image/png",
                })]),
            }
        }
        "bb_mouse" => {
            let event_type = args
                .get("type")
                .and_then(Value::as_str)
                .context("missing argument: type")?;
            let x = arg_f64(args, "x")?;
            let y = arg_f64(args, "y")?;
            let button = args.get("button").and_then(Value::as_str);
            let buttons = args.get("buttons").and_then(Value::as_u64);
            let click_count = args.get("click_count").and_then(Value::as_u64);
            let page = super::attach(root_dir).await?;
            super::dispatch_mouse_event(&page, event_type, x, y, button, buttons, click_count)
                .await?;
            Ok(text_content(
                json!({ "ok": true, "type": event_type, "x": x, "y": y }).to_string(),
            ))
        }
        "bb_drag" => {
            let from_x = arg_f64(args, "from_x")?;
            let from_y = arg_f64(args, "from_y")?;
            let to_x = arg_f64(args, "to_x")?;
            let to_y = arg_f64(args, "to_y")?;
            let steps = args
                .get("steps")
                .and_then(Value::as_u64)
                .unwrap_or(8)
                .max(1);
            let page = super::attach(root_dir).await?;
            super::dispatch_mouse_event(&page, "mouseMoved", from_x, from_y, None, Some(0), None)
                .await?;
            super::dispatch_mouse_event(
                &page,
                "mousePressed",
                from_x,
                from_y,
                Some("left"),
                Some(1),
                Some(1),
            )
            .await?;
            for step in 1..=steps {
                let t = step as f64 / steps as f64;
                let x = from_x + (to_x - from_x) * t;
                let y = from_y + (to_y - from_y) * t;
                super::dispatch_mouse_event(&page, "mouseMoved", x, y, Some("left"), Some(1), None)
                    .await?;
                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
            }
            super::dispatch_mouse_event(
                &page,
                "mouseReleased",
                to_x,
                to_y,
                Some("left"),
                Some(0),
                Some(1),
            )
            .await?;
            Ok(text_content(
                json!({
                    "ok": true,
                    "from": { "x": from_x, "y": from_y },
                    "to": { "x": to_x, "y": to_y },
                    "steps": steps,
                })
                .to_string(),
            ))
        }
        "bb_wheel" => {
            let x = arg_f64(args, "x")?;
            let y = arg_f64(args, "y")?;
            let delta_x = args.get("delta_x").and_then(Value::as_f64).unwrap_or(0.0);
            let delta_y = args.get("delta_y").and_then(Value::as_f64).unwrap_or(0.0);
            let page = super::attach(root_dir).await?;
            page.session()
                .send(
                    "Input.dispatchMouseEvent",
                    json!({
                        "type": "mouseWheel",
                        "x": x,
                        "y": y,
                        "deltaX": delta_x,
                        "deltaY": delta_y,
                    }),
                )
                .await
                .context("dispatching CDP mouse wheel event")?;
            Ok(text_content(
                json!({ "ok": true, "type": "mouseWheel", "x": x, "y": y, "deltaX": delta_x, "deltaY": delta_y })
                    .to_string(),
            ))
        }
        "bb_key" => {
            let key = args
                .get("key")
                .and_then(Value::as_str)
                .context("missing argument: key")?;
            let mut modifiers = 0u64;
            // Same CDP modifier bitfield as the `jet bb key` CLI flags.
            if args.get("alt").and_then(Value::as_bool).unwrap_or(false) {
                modifiers |= 1;
            }
            if args.get("ctrl").and_then(Value::as_bool).unwrap_or(false) {
                modifiers |= 2;
            }
            if args.get("meta").and_then(Value::as_bool).unwrap_or(false) {
                modifiers |= 4;
            }
            if args.get("shift").and_then(Value::as_bool).unwrap_or(false) {
                modifiers |= 8;
            }
            let page = super::attach(root_dir).await?;
            super::dispatch_key_event(&page, "keyDown", key, modifiers).await?;
            super::dispatch_key_event(&page, "keyUp", key, modifiers).await?;
            Ok(text_content(
                json!({ "ok": true, "key": key, "modifiers": modifiers }).to_string(),
            ))
        }
        "bb_highlight" => {
            let page = super::attach(root_dir).await?;
            super::assert_debug_bridge(&page).await?;
            let arg = if args.get("clear").and_then(Value::as_bool).unwrap_or(false) {
                "undefined".to_string()
            } else {
                args.get("index")
                    .and_then(Value::as_u64)
                    .context("highlight requires index or clear=true")?
                    .to_string()
            };
            page.evaluate(&super::expr("highlight", &arg)).await?;
            Ok(text_content(json!({ "ok": true }).to_string()))
        }
        "bb_snapshot" => {
            let v = super::interact::snapshot(root_dir).await?;
            let mut out = format!(
                "# {} — {}\n",
                v["title"].as_str().unwrap_or(""),
                v["url"].as_str().unwrap_or("")
            );
            if v["truncated"].as_bool() == Some(true) {
                out.push_str("# (truncated at element cap)\n");
            }
            out.push_str(v["snapshot"].as_str().unwrap_or(""));
            Ok(text_content(out))
        }
        "bb_click" => {
            let target = arg_target(args)?;
            let dblclick = args
                .get("dblclick")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let v = super::interact::click(root_dir, &target, dblclick).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_fill" => {
            let target = arg_target(args)?;
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .context("missing argument: text")?;
            let v = super::interact::fill(root_dir, &target, text).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_type" => {
            let target = arg_target(args)?;
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .context("missing argument: text")?;
            let v = super::interact::type_text(root_dir, &target, text).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_hover" => {
            let target = arg_target(args)?;
            let v = super::interact::hover(root_dir, &target).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_select" => {
            let target = arg_target(args)?;
            let option = args
                .get("option")
                .and_then(Value::as_str)
                .context("missing argument: option")?;
            let v = super::interact::select(root_dir, &target, option).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_check" => {
            let target = arg_target(args)?;
            let checked = args.get("checked").and_then(Value::as_bool).unwrap_or(true);
            let v = super::interact::set_checked(root_dir, &target, checked).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_goto" => {
            let url = args
                .get("url")
                .and_then(Value::as_str)
                .context("missing argument: url")?;
            let v = super::interact::goto(root_dir, url).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_back" => {
            let v = super::interact::history_step(root_dir, -1).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_forward" => {
            let v = super::interact::history_step(root_dir, 1).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_reload" => {
            let v = super::interact::reload(root_dir).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_resize" => {
            let width = args
                .get("width")
                .and_then(Value::as_u64)
                .context("missing or non-integer argument: width")?;
            let height = args
                .get("height")
                .and_then(Value::as_u64)
                .context("missing or non-integer argument: height")?;
            let v = super::interact::resize(root_dir, width, height).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_wait_for" => {
            let selector = args.get("selector").and_then(Value::as_str);
            let text = args.get("text").and_then(Value::as_str);
            let ms = args.get("ms").and_then(Value::as_u64);
            let timeout_ms = args
                .get("timeout_ms")
                .and_then(Value::as_u64)
                .unwrap_or(10_000);
            let v = super::interact::wait(root_dir, selector, text, ms, timeout_ms).await?;
            Ok(text_content(v.to_string()))
        }
        "bb_console" => {
            let level = args.get("level").and_then(Value::as_str);
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize;
            let clear = args.get("clear").and_then(Value::as_bool).unwrap_or(false);
            let v = super::interact::console(root_dir, level, limit, clear).await?;
            Ok(text_content(serde_json::to_string_pretty(&v)?))
        }
        "bb_requests" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(100) as usize;
            let clear = args.get("clear").and_then(Value::as_bool).unwrap_or(false);
            let v = super::interact::requests(root_dir, limit, clear).await?;
            Ok(text_content(serde_json::to_string_pretty(&v)?))
        }
        other => anyhow::bail!("unknown tool: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definitions_cover_the_agent_surface() {
        let names: Vec<String> = tool_definitions()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
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
            assert!(
                names.contains(&required.to_string()),
                "missing tool {required}"
            );
        }
        for tool in tool_definitions() {
            assert!(tool["description"].as_str().unwrap().len() > 10);
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn element_targeted_tools_resolve_ref_xor_selector() {
        let t = arg_target(&json!({ "ref": "e3" })).unwrap();
        assert_eq!(t, super::super::interact::Target::Ref("e3".into()));
        let t = arg_target(&json!({ "selector": "text=Save" })).unwrap();
        assert_eq!(
            t,
            super::super::interact::Target::Selector("text=Save".into())
        );
        assert!(arg_target(&json!({})).is_err());
        assert!(arg_target(&json!({ "ref": "e3", "selector": "#x" })).is_err());
        assert!(arg_target(&json!({ "ref": "not-a-ref" })).is_err());
    }

    #[tokio::test]
    async fn initialize_echoes_known_protocol_versions() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": "2025-03-26" },
        });
        let resp = handle_message(Path::new("."), &msg).await.unwrap();
        assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(resp["result"]["serverInfo"]["name"], "jet-bb");
    }

    #[tokio::test]
    async fn unknown_protocol_version_falls_back_to_latest() {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": "1999-01-01" },
        });
        let resp = handle_message(Path::new("."), &msg).await.unwrap();
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn notifications_get_no_response() {
        let msg = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
        assert!(handle_message(Path::new("."), &msg).await.is_none());
    }

    #[tokio::test]
    async fn unknown_method_is_a_jsonrpc_error() {
        let msg = json!({ "jsonrpc": "2.0", "id": 7, "method": "resources/list" });
        let resp = handle_message(Path::new("."), &msg).await.unwrap();
        assert_eq!(resp["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn tool_call_without_session_reports_is_error_not_protocol_error() {
        let tmp = std::env::temp_dir().join("jet-bb-mcp-no-session");
        let _ = std::fs::create_dir_all(&tmp);
        let msg = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "bb_eval", "arguments": { "expression": "1+1" } },
        });
        let resp = handle_message(&tmp, &msg).await.unwrap();
        assert_eq!(resp["result"]["isError"], true);
        assert!(
            resp.get("error").is_none(),
            "tool failure must not be a protocol error"
        );
    }
}
// </HANDWRITE>

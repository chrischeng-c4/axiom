// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
// CODEGEN-BEGIN
//! Pretty-print helpers for the JSON shapes `window.__jet_debug`
//! returns.
//!
//! Keep these dead simple and deterministic — snapshot-testable.
//! The shapes are defined in `jet_wasm::debug` but we avoid importing
//! the type machinery here (the JSON comes over the CDP wire anyway)
//! and just walk `serde_json::Value`.

use serde_json::Value;
use std::fmt::Write;

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn element_tree(v: &Value) -> String {
    let mut out = String::new();
    render_element(v, 0, &mut out);
    out
}

fn render_element(v: &Value, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    match v.get("kind").and_then(|k| k.as_str()) {
        Some("intrinsic") => {
            let tag = v.get("tag").and_then(|t| t.as_str()).unwrap_or("?");
            let id = v
                .get("props")
                .and_then(|p| p.get("id"))
                .and_then(|i| i.as_str());
            let has_click = v
                .get("props")
                .and_then(|p| p.get("has_on_click"))
                .and_then(|b| b.as_bool())
                .unwrap_or(false);
            let tag_extras = match (id, has_click) {
                (Some(id), true) => format!(" #{id} (onClick)"),
                (Some(id), false) => format!(" #{id}"),
                (None, true) => " (onClick)".to_string(),
                (None, false) => String::new(),
            };
            let _ = writeln!(out, "{indent}<{tag}>{tag_extras}");
            if let Some(children) = v.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    render_element(child, depth + 1, out);
                }
            }
        }
        Some("text") => {
            let t = v.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let _ = writeln!(out, "{indent}\"{t}\"");
        }
        Some("component") => {
            let name = v.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            let _ = writeln!(out, "{indent}<{name} />  (not yet expanded)");
        }
        Some("empty") => {
            let _ = writeln!(out, "{indent}<empty/>");
        }
        _ => {
            let _ = writeln!(out, "{indent}<unknown> {v}");
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn layout_tree(v: &Value) -> String {
    let mut out = String::new();
    if let Some(root) = v.get("root_rect") {
        let _ = writeln!(out, "root: {}", render_rect(root));
    }
    if let Some(nodes) = v.get("nodes").and_then(|n| n.as_array()) {
        for (i, n) in nodes.iter().enumerate() {
            let rect = n.get("rect").map(render_rect).unwrap_or_default();
            let kind = n.get("kind");
            let label = match kind.and_then(|k| k.get("kind")).and_then(|k| k.as_str()) {
                Some("intrinsic") => {
                    let tag = kind
                        .and_then(|k| k.get("tag"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("?");
                    let id = kind.and_then(|k| k.get("id")).and_then(|i| i.as_str());
                    match id {
                        Some(id) => format!("<{tag}> #{id}"),
                        None => format!("<{tag}>"),
                    }
                }
                Some("text") => {
                    let t = kind
                        .and_then(|k| k.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    format!("\"{t}\"")
                }
                _ => "?".to_string(),
            };
            let _ = writeln!(out, "  [{i}] {rect}  {label}");
        }
    }
    out
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn fiber_tree(v: &Value) -> String {
    let mut out = String::new();
    if let Some(arr) = v.as_array() {
        let _ = writeln!(out, "id  hooks  dirty");
        for f in arr {
            let id = f.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let hc = f.get("hook_count").and_then(|v| v.as_u64()).unwrap_or(0);
            let dirty = f.get("dirty").and_then(|v| v.as_bool()).unwrap_or(false);
            let _ = writeln!(out, "{id:<3} {hc:<6} {dirty}");
        }
    }
    out
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn hook_values(v: &Value) -> String {
    let mut out = String::new();
    if let Some(arr) = v.as_array() {
        for (i, h) in arr.iter().enumerate() {
            let kind = h.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let ty = h.get("type_name").and_then(|v| v.as_str()).unwrap_or("");
            let val = h.get("value").cloned().unwrap_or(Value::Null);
            let val_str = if val.is_null() {
                "<unknown>".to_string()
            } else {
                val.to_string()
            };
            let _ = writeln!(out, "[{i}] {kind}<{ty}> = {val_str}");
        }
    }
    out
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn paint_ops(v: &Value) -> String {
    if v.is_null() {
        return "(no frame recorded yet)\n".to_string();
    }
    let mut out = String::new();
    if let Some(arr) = v.as_array() {
        for (i, op) in arr.iter().enumerate() {
            let name = op.get("op").and_then(|v| v.as_str()).unwrap_or("?");
            let _ = writeln!(out, "[{i:>3}] {name}  {op}");
        }
    }
    out
}

fn render_rect(r: &Value) -> String {
    let x = r.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = r.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let w = r.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let h = r.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);
    format!("x={x:>4.0} y={y:>4.0} w={w:>4.0} h={h:>4.0}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn element_tree_renders_button_with_text_child() {
        let v = json!({
            "kind": "intrinsic",
            "tag": "button",
            "props": {
                "id": "inc",
                "has_on_click": true,
                "has_on_change": false,
                "class_name": null,
                "style": null,
            },
            "children": [
                { "kind": "text", "text": "count: " },
                { "kind": "text", "text": "0" },
            ],
        });
        let out = element_tree(&v);
        assert!(out.contains("<button> #inc (onClick)"));
        assert!(out.contains("\"count: \""));
        assert!(out.contains("\"0\""));
    }

    #[test]
    fn fiber_tree_shows_header_plus_row() {
        let v = json!([{ "id": 0, "hook_count": 1, "dirty": false }]);
        let out = fiber_tree(&v);
        assert!(out.contains("id"));
        assert!(out.contains("0   1      false"));
    }

    #[test]
    fn paint_ops_null_means_no_frame() {
        assert_eq!(paint_ops(&Value::Null), "(no frame recorded yet)\n");
    }
}
// CODEGEN-END

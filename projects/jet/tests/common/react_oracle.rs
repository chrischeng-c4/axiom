// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
// CODEGEN-BEGIN
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub const LAYOUT_TOLERANCE_CSS_PX: f64 = 1.0;
pub const SCREENSHOT_BOUNDS_TOLERANCE_PX: i64 = 8;
pub const SCREENSHOT_FOREGROUND_RATIO_TOLERANCE: f64 = 0.50;
pub const SCREENSHOT_PHASH_HAMMING_TOLERANCE: u32 = 16;
const SCREENSHOT_FOREGROUND_COLOR_DISTANCE: i16 = 24;

enum ReactDomAssets {
    Umd {
        react: PathBuf,
        react_dom: PathBuf,
    },
    Cjs {
        react: PathBuf,
        react_dom: PathBuf,
        react_dom_client: PathBuf,
        scheduler: PathBuf,
    },
}

fn react_node_modules_root(workspace: &Path) -> PathBuf {
    std::env::var("JET_REACT_ORACLE_NODE_MODULES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            workspace
                .join("examples")
                .join("react-bench")
                .join("node_modules")
        })
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
fn react_dom_assets(workspace: &Path) -> Option<ReactDomAssets> {
    let root = react_node_modules_root(workspace);
    let umd_react = root.join("react").join("umd").join("react.development.js");
    let umd_react_dom = root
        .join("react-dom")
        .join("umd")
        .join("react-dom.development.js");
    if umd_react.exists() && umd_react_dom.exists() {
        return Some(ReactDomAssets::Umd {
            react: umd_react,
            react_dom: umd_react_dom,
        });
    }

    let cjs_react = root.join("react").join("cjs").join("react.development.js");
    let cjs_react_dom = root
        .join("react-dom")
        .join("cjs")
        .join("react-dom.development.js");
    let cjs_react_dom_client = root
        .join("react-dom")
        .join("cjs")
        .join("react-dom-client.development.js");
    let cjs_scheduler = root
        .join("scheduler")
        .join("cjs")
        .join("scheduler.development.js");
    if cjs_react.exists()
        && cjs_react_dom.exists()
        && cjs_react_dom_client.exists()
        && cjs_scheduler.exists()
    {
        return Some(ReactDomAssets::Cjs {
            react: cjs_react,
            react_dom: cjs_react_dom,
            react_dom_client: cjs_react_dom_client,
            scheduler: cjs_scheduler,
        });
    }

    None
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn react_dom_available(workspace: &Path) -> bool {
    react_dom_assets(workspace).is_some()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn require_react_oracle_env(workspace: &Path) {
    if !super::node_available() || !super::chromium_available() || !react_dom_available(workspace) {
        super::fail_missing_prerequisite(format!(
            "need node + Chromium + local React/ReactDOM node_modules \
             (node={} chromium={} react_dom={})",
            super::node_available(),
            super::chromium_available(),
            react_dom_available(workspace),
        ));
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn dom_serializer_expr(root_selector: &str) -> String {
    let selector = serde_json::to_string(root_selector).expect("selector serializes");
    format!(
        r#"
(() => {{
  const root = document.querySelector({selector});
  const stableAttrs = new Set(['id', 'class', 'style', 'role', 'aria-label', 'data-testid']);
  const normalizeText = (text) => text.replace(/\s+/g, ' ').trim();
  const mergeText = (children) => {{
    const out = [];
    for (const child of children) {{
      if (child.kind === 'text' && normalizeText(child.text) === '') continue;
      const prev = out[out.length - 1];
      if (prev && prev.kind === 'text' && child.kind === 'text') {{
        prev.text = `${{prev.text}}${{child.text}}`;
      }} else {{
        out.push(child);
      }}
    }}
    return out;
  }};
  const finalizeText = (node) => {{
    if (!node) return null;
    if (node.kind === 'text') return {{ kind: 'text', text: normalizeText(node.text || '') }};
    if (node.kind === 'element') {{
      return {{ ...node, children: node.children.map(finalizeText).filter(Boolean) }};
    }}
    return node;
  }};
  const walk = (node) => {{
    if (!node) return null;
    if (node.nodeType === Node.TEXT_NODE) {{
      return {{ kind: 'text', text: node.textContent || '' }};
    }}
    if (node.nodeType !== Node.ELEMENT_NODE) return null;
    const attrs = {{}};
    for (const attr of Array.from(node.attributes)) {{
      if (stableAttrs.has(attr.name) || attr.name.startsWith('data-') || attr.name.startsWith('aria-')) {{
        attrs[attr.name] = attr.value;
      }}
    }}
    const children = mergeText(Array.from(node.childNodes).map(walk).filter(Boolean));
    return {{
      kind: 'element',
      tag: node.tagName.toLowerCase(),
      attrs,
      children
    }};
  }};
  return finalizeText(walk(root));
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#unit-test
pub fn dom_layout_boxes_expr(root_selector: &str) -> String {
    let selector = serde_json::to_string(root_selector).expect("selector serializes");
    format!(
        r#"
(() => {{
  const root = document.querySelector({selector});
  if (!root) {{
    return {{
      schema_version: 'jet.layout_boxes.v1',
      root_selector: {selector},
      boxes: []
    }};
  }}
  const rootRect = root.getBoundingClientRect();
  const normalizeNumber = (value) => Math.round(Number(value || 0) * 10) / 10;
  const rectFor = (node) => {{
    const rect = node.getBoundingClientRect();
    return {{
      x: normalizeNumber(rect.left - rootRect.left),
      y: normalizeNumber(rect.top - rootRect.top),
      w: normalizeNumber(rect.width),
      h: normalizeNumber(rect.height)
    }};
  }};
  const boxes = [];
  const walk = (node, path) => {{
    if (!node || node.nodeType !== Node.ELEMENT_NODE) return;
    boxes.push({{
      key: String(boxes.length),
      path,
      kind: 'element',
      tag: node.tagName.toLowerCase(),
      id: node.getAttribute('id') || '',
      rect: rectFor(node)
    }});
    let elementIndex = 0;
    for (const child of Array.from(node.childNodes)) {{
      if (child.nodeType !== Node.ELEMENT_NODE) continue;
      walk(child, `${{path}}/${{elementIndex}}`);
      elementIndex += 1;
    }}
  }};
  walk(root, '0');
  return {{
    schema_version: 'jet.layout_boxes.v1',
    root_selector: {selector},
    boxes
  }};
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4004.md#schema
pub fn controlled_input_dom_state_expr(input_selector: &str, label_selector: &str) -> String {
    let input_selector = serde_json::to_string(input_selector).expect("selector serializes");
    let label_selector = serde_json::to_string(label_selector).expect("selector serializes");
    format!(
        r#"
(() => {{
  const input = document.querySelector({input_selector});
  const label = document.querySelector({label_selector});
  return {{
    schema_version: 'jet.controlled_input_dom_state.v1',
    input_value: input && typeof input.value === 'string' ? input.value : '',
    placeholder: input ? (input.getAttribute('placeholder') || '') : '',
    label_text: label ? (label.textContent || '').replace(/\s+/g, ' ').trim() : ''
  }};
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4015.md#schema
pub fn controlled_textarea_dom_state_expr(textarea_selector: &str, label_selector: &str) -> String {
    let textarea_selector = serde_json::to_string(textarea_selector).expect("selector serializes");
    let label_selector = serde_json::to_string(label_selector).expect("selector serializes");
    format!(
        r#"
(() => {{
  const textarea = document.querySelector({textarea_selector});
  const label = document.querySelector({label_selector});
  return {{
    schema_version: 'jet.controlled_textarea_dom_state.v1',
    textarea_value: textarea && typeof textarea.value === 'string' ? textarea.value : '',
    placeholder: textarea ? (textarea.getAttribute('placeholder') || '') : '',
    label_text: label ? (label.textContent || '').replace(/\s+/g, ' ').trim() : ''
  }};
}})()
"#
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn counter_fixture_html(workspace: &Path) -> Option<String> {
    fixture_html(
        workspace,
        r#"
      const e = React.createElement;
      function Counter(props) {
        const [n, setN] = React.useState(props.start);
        return e('button', { id: 'inc', onClick: () => setN(n + 1) }, 'count: ', String(n));
      }
      render(e(Counter, { start: 0 }));
"#,
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3943.md#changes
pub fn fixture_html(workspace: &Path, render_script: &str) -> Option<String> {
    let react_runtime = react_runtime_html(workspace)?;
    let render_script = render_script.replace("</script", "<\\/script");
    Some(format!(
        r#"<!doctype html>
<html>
  <head>
    <style>
      html, body {{
        margin: 0;
        padding: 0;
        background: #fafafa;
        font-family: -apple-system, Segoe UI, system-ui, sans-serif;
      }}
    </style>
  </head>
  <body>
    <div id="root"></div>
    {react_runtime}
    <script>
      function render(app) {{
        const root = document.getElementById('root');
        if (ReactDOMClient.createRoot) {{
          ReactDOMClient.createRoot(root).render(app);
        }} else {{
          ReactDOM.render(app, root);
        }}
      }}
{render_script}
    </script>
  </body>
</html>
"#,
    ))
}

fn react_runtime_html(workspace: &Path) -> Option<String> {
    Some(match react_dom_assets(workspace)? {
        ReactDomAssets::Umd { react, react_dom } => {
            let react_src = serde_json::to_string(&file_url(&react)).expect("React URL serializes");
            let react_dom_src =
                serde_json::to_string(&file_url(&react_dom)).expect("ReactDOM URL serializes");
            format!(
                r#"<script src={react_src}></script>
    <script src={react_dom_src}></script>
    <script>window.ReactDOMClient = window.ReactDOM;</script>"#
            )
        }
        ReactDomAssets::Cjs {
            react,
            react_dom,
            react_dom_client,
            scheduler,
        } => {
            let react = read_script(&react)?;
            let react_dom = read_script(&react_dom)?;
            let react_dom_client = read_script(&react_dom_client)?;
            let scheduler = read_script(&scheduler)?;
            format!(
                r#"<script>
      window.process = {{ env: {{ NODE_ENV: 'development' }} }};
      var process = window.process;
      (function () {{
        const modules = {{}};
        function require(id) {{
          if (!modules[id]) throw new Error('missing React oracle module: ' + id);
          return modules[id];
        }}
        function define(id, factory) {{
          const module = {{ exports: {{}} }};
          factory(module.exports, module, require);
          modules[id] = module.exports;
        }}
        define('react', function (exports, module, require) {{
{react}
        }});
        define('scheduler', function (exports, module, require) {{
{scheduler}
        }});
        define('react-dom', function (exports, module, require) {{
{react_dom}
        }});
        define('react-dom/client', function (exports, module, require) {{
{react_dom_client}
        }});
        window.React = require('react');
        window.ReactDOM = require('react-dom');
        window.ReactDOMClient = require('react-dom/client');
      }})();
    </script>"#
            )
        }
    })
}

fn read_script(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| content.replace("</script", "<\\/script"))
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn normalize_jet_element_tree(value: &Value) -> Value {
    let mut normalized =
        normalize_jet_node(value).unwrap_or_else(|| json!({ "kind": "text", "text": "" }));
    finalize_text_nodes(&mut normalized);
    normalized
}

/// @spec .aw/tech-design/projects/jet/specs/3941.md#unit-test
pub fn dom_tree_from_observation(bundle: &Value) -> Value {
    bundle.get("dom_tree").cloned().unwrap_or(Value::Null)
}

/// @spec .aw/tech-design/projects/jet/specs/3941.md#unit-test
pub fn normalize_wasm_observation_element_tree(bundle: &Value) -> Value {
    normalize_jet_element_tree(bundle.get("element_tree").unwrap_or(&Value::Null))
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#unit-test
pub fn normalize_wasm_layout_boxes(layout_tree: &Value) -> Value {
    let boxes = layout_tree
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|node| normalize_wasm_layout_box(node))
        .enumerate()
        .map(|(index, mut box_value)| {
            box_value["key"] = Value::String(index.to_string());
            box_value
        })
        .collect::<Vec<_>>();

    json!({
        "schema_version": "jet.layout_boxes.v1",
        "root_selector": "window.__jet_debug.layoutTree().nodes",
        "boxes": boxes,
    })
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#unit-test
pub fn layout_boxes_match(expected: &Value, actual: &Value) -> bool {
    let Some(expected_boxes) = expected.get("boxes").and_then(Value::as_array) else {
        return false;
    };
    let Some(actual_boxes) = actual.get("boxes").and_then(Value::as_array) else {
        return false;
    };
    if expected_boxes.len() != actual_boxes.len() {
        return false;
    }
    expected_boxes
        .iter()
        .zip(actual_boxes.iter())
        .all(|(expected_box, actual_box)| layout_box_matches(expected_box, actual_box))
}

/// @spec .aw/tech-design/projects/jet/specs/3941.md#unit-test
pub fn wasm_observation_has_hook_i64(bundle: &Value, expected: i64) -> bool {
    bundle
        .get("hook_values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            entry
                .get("values")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .any(|hook| hook.get("value").and_then(Value::as_i64) == Some(expected))
}

fn normalize_wasm_layout_box(node: &Value) -> Option<Value> {
    let kind = node.get("kind")?;
    if kind.get("kind").and_then(Value::as_str) != Some("intrinsic") {
        return None;
    }
    let rect = node.get("rect").unwrap_or(&Value::Null);
    Some(json!({
        "path": "",
        "kind": "element",
        "tag": kind.get("tag").and_then(Value::as_str).unwrap_or(""),
        "id": kind.get("id").and_then(Value::as_str).unwrap_or(""),
        "rect": normalize_layout_rect(rect),
    }))
}

fn normalize_layout_rect(rect: &Value) -> Value {
    json!({
        "x": round_layout_number(rect.get("x").and_then(Value::as_f64).unwrap_or(0.0)),
        "y": round_layout_number(rect.get("y").and_then(Value::as_f64).unwrap_or(0.0)),
        "w": round_layout_number(rect.get("w").and_then(Value::as_f64).unwrap_or(0.0)),
        "h": round_layout_number(rect.get("h").and_then(Value::as_f64).unwrap_or(0.0)),
    })
}

fn round_layout_number(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn layout_box_matches(expected: &Value, actual: &Value) -> bool {
    for field in ["key", "kind", "tag", "id"] {
        if expected.get(field) != actual.get(field) {
            return false;
        }
    }
    let Some(expected_rect) = expected.get("rect") else {
        return false;
    };
    let Some(actual_rect) = actual.get("rect") else {
        return false;
    };
    ["x", "y", "w", "h"].iter().all(|field| {
        let expected = expected_rect
            .get(*field)
            .and_then(Value::as_f64)
            .unwrap_or(f64::NAN);
        let actual = actual_rect
            .get(*field)
            .and_then(Value::as_f64)
            .unwrap_or(f64::NAN);
        expected.is_finite()
            && actual.is_finite()
            && (expected - actual).abs() <= LAYOUT_TOLERANCE_CSS_PX
    })
}

fn normalize_jet_node(value: &Value) -> Option<Value> {
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "intrinsic" => {
            let tag = value.get("tag").and_then(|v| v.as_str()).unwrap_or("");
            let attrs = normalize_jet_attrs(value.get("props").unwrap_or(&Value::Null));
            let children = normalize_children(value.get("children").and_then(|v| v.as_array()));
            Some(json!({
                "kind": "element",
                "tag": tag,
                "attrs": attrs,
                "children": children,
            }))
        }
        "text" => Some(json!({
            "kind": "text",
            "text": value.get("text").and_then(|v| v.as_str()).unwrap_or(""),
        })),
        "fragment" => {
            let children = normalize_children(value.get("children").and_then(|v| v.as_array()));
            Some(json!({ "kind": "fragment", "children": children }))
        }
        "empty" | "component" => None,
        _ => None,
    }
}

fn normalize_children(children: Option<&Vec<Value>>) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for child in children.into_iter().flatten() {
        let Some(normalized) = normalize_jet_node(child) else {
            continue;
        };
        if normalized.get("kind").and_then(|v| v.as_str()) == Some("fragment") {
            if let Some(fragment_children) = normalized.get("children").and_then(|v| v.as_array()) {
                for fragment_child in fragment_children {
                    push_child(&mut out, fragment_child.clone());
                }
            }
        } else {
            push_child(&mut out, normalized);
        }
    }
    out
}

fn push_child(out: &mut Vec<Value>, child: Value) {
    if child.get("kind").and_then(|v| v.as_str()) == Some("text")
        && child
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
    {
        return;
    }

    if let Some(prev) = out.last_mut() {
        if prev.get("kind").and_then(|v| v.as_str()) == Some("text")
            && child.get("kind").and_then(|v| v.as_str()) == Some("text")
        {
            let prev_text = prev.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let child_text = child.get("text").and_then(|v| v.as_str()).unwrap_or("");
            *prev = json!({ "kind": "text", "text": format!("{prev_text}{child_text}") });
            return;
        }
    }

    out.push(child);
}

fn normalize_jet_attrs(props: &Value) -> Value {
    let mut attrs = Map::new();
    if let Some(id) = props.get("id").and_then(|v| v.as_str()) {
        attrs.insert("id".to_string(), Value::String(id.to_string()));
    }
    if let Some(class_name) = props.get("class_name").and_then(|v| v.as_str()) {
        attrs.insert("class".to_string(), Value::String(class_name.to_string()));
    }
    if let Some(style) = props.get("style").and_then(|v| v.as_str()) {
        attrs.insert("style".to_string(), Value::String(style.to_string()));
    }
    if let Some(input_type) = props.get("input_type").and_then(|v| v.as_str()) {
        attrs.insert("type".to_string(), Value::String(input_type.to_string()));
    }
    if let Some(placeholder) = props.get("placeholder").and_then(|v| v.as_str()) {
        attrs.insert(
            "placeholder".to_string(),
            Value::String(placeholder.to_string()),
        );
    }
    if let Some(checked) = props.get("checked").and_then(|v| v.as_bool()) {
        attrs.insert("checked".to_string(), Value::String(checked.to_string()));
    }
    if let Some(aria_label) = props.get("aria_label").and_then(|v| v.as_str()) {
        attrs.insert(
            "aria-label".to_string(),
            Value::String(aria_label.to_string()),
        );
    }
    Value::Object(attrs)
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn finalize_text_nodes(value: &mut Value) {
    match value.get("kind").and_then(|v| v.as_str()) {
        Some("text") => {
            let text = value.get("text").and_then(|v| v.as_str()).unwrap_or("");
            value["text"] = Value::String(normalize_text(text));
        }
        Some("element") | Some("fragment") => {
            if let Some(children) = value.get_mut("children").and_then(|v| v.as_array_mut()) {
                for child in children {
                    finalize_text_nodes(child);
                }
            }
        }
        _ => {}
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn diff_message(step: &str, react: &Value, jet: &Value) -> String {
    format!(
        "React DOM oracle mismatch at {step}\nreact:\n{}\njet:\n{}",
        serde_json::to_string_pretty(react).unwrap_or_else(|_| react.to_string()),
        serde_json::to_string_pretty(jet).unwrap_or_else(|_| jet.to_string()),
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3943.md#unit-test
pub fn fixture_diff_message(
    fixture_id: &str,
    phase: &str,
    expected: &Value,
    actual: &Value,
) -> String {
    let payload = json!({
        "failure_kind": "dom_wasm_parity_mismatch",
        "expected_source": "react_dom",
        "actual_source": "jet_wasm",
        "fixture_id": fixture_id,
        "phase": phase,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM oracle fixture mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4041.md#unit-test
pub fn library_fixture_diff_message(
    library_id: &str,
    fixture_id: &str,
    phase: &str,
    expected: &Value,
    actual: &Value,
) -> String {
    let payload = json!({
        "failure_kind": "library_dom_wasm_parity_mismatch",
        "expected_source": "react_dom",
        "actual_source": "jet_wasm",
        "library_id": library_id,
        "fixture_id": fixture_id,
        "phase": phase,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React library DOM/WASM parity mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4004.md#schema
pub fn controlled_input_diff_message(phase: &str, expected: &Value, actual: &Value) -> String {
    let payload = json!({
        "phase": phase,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM controlled input mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/4015.md#schema
pub fn controlled_textarea_diff_message(phase: &str, expected: &Value, actual: &Value) -> String {
    let payload = json!({
        "phase": phase,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM controlled textarea mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#unit-test
pub fn layout_diff_message(
    fixture_id: &str,
    phase: &str,
    expected: &Value,
    actual: &Value,
) -> String {
    let payload = json!({
        "failure_kind": "layout_dom_wasm_parity_mismatch",
        "fixture_id": fixture_id,
        "phase": phase,
        "tolerance_css_px": LAYOUT_TOLERANCE_CSS_PX,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM oracle layout mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_summary_from_png(bytes: &[u8]) -> Value {
    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|err| panic!("decode screenshot PNG: {err}"))
        .to_rgba8();
    let (width, height) = image.dimensions();
    let background = screenshot_background_color(&image);
    let mut foreground_count = 0_u64;
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0_u32;
    let mut max_y = 0_u32;

    for (x, y, pixel) in image.enumerate_pixels() {
        if screenshot_pixel_is_foreground(pixel.0, background) {
            foreground_count += 1;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    let bounds = if foreground_count == 0 {
        json!({ "x": 0, "y": 0, "w": 0, "h": 0 })
    } else {
        json!({
            "x": min_x,
            "y": min_y,
            "w": max_x - min_x + 1,
            "h": max_y - min_y + 1,
        })
    };

    json!({
        "schema_version": "jet.screenshot_summary.v1",
        "width": width,
        "height": height,
        "foreground_count": foreground_count,
        "bounds": bounds,
    })
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_visual_probe_from_png(bytes: &[u8]) -> Value {
    let image = image::load_from_memory(bytes)
        .unwrap_or_else(|err| panic!("decode screenshot PNG for visual probe: {err}"))
        .to_rgba8();
    let (width, height) = image.dimensions();
    let mut non_transparent = 0_u64;
    let mut non_white = 0_u64;
    let mut non_black = 0_u64;
    let mut buckets = HashSet::new();

    for pixel in image.pixels() {
        let [r, g, b, a] = pixel.0;
        if a > 0 {
            non_transparent += 1;
        }
        if a > 0 && (r < 250 || g < 250 || b < 250) {
            non_white += 1;
        }
        if a > 0 && (r > 5 || g > 5 || b > 5) {
            non_black += 1;
        }
        buckets.insert((r >> 5, g >> 5, b >> 5, a >> 5));
    }

    let summary = screenshot_summary_from_png(bytes);
    let full_hash = screenshot_phash_for_image(&image, ScreenshotPHashMode::RgbLuma);
    let (crop_image, crop_bounds, hash_scope) = screenshot_foreground_crop(&image, &summary)
        .unwrap_or_else(|| {
            (
                image.clone(),
                json!({ "x": 0, "y": 0, "w": width, "h": height }),
                "full",
            )
        });
    let cropped_hash = screenshot_phash_for_image(&crop_image, ScreenshotPHashMode::ForegroundMask);

    json!({
        "schema_version": "jet.screenshot_visual_probe.v1",
        "width": width,
        "height": height,
        "nonTransparent": non_transparent,
        "nonWhite": non_white,
        "nonBlack": non_black,
        "uniqueBuckets": buckets.len(),
        "foregroundCount": summary
            .get("foreground_count")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        "foregroundBounds": summary.get("bounds").cloned().unwrap_or(Value::Null),
        "hashScope": hash_scope,
        "hashBounds": crop_bounds,
        "hashMode": "foreground-mask-phash",
        "averageLuma": cropped_hash.average_luma,
        "averageForegroundMask": cropped_hash.average_luma,
        "hash": cropped_hash.hash,
        "hashOnes": cropped_hash.ones,
        "fullHashMode": "rgb-luma-phash",
        "fullAverageLuma": full_hash.average_luma,
        "fullHash": full_hash.hash,
        "fullHashOnes": full_hash.ones,
    })
}

struct ScreenshotPHash {
    hash: String,
    ones: u64,
    average_luma: f64,
}

enum ScreenshotPHashMode {
    RgbLuma,
    ForegroundMask,
}

fn screenshot_phash_for_image(
    image: &image::RgbaImage,
    mode: ScreenshotPHashMode,
) -> ScreenshotPHash {
    let background = match mode {
        ScreenshotPHashMode::RgbLuma => None,
        ScreenshotPHashMode::ForegroundMask => Some(screenshot_background_color(image)),
    };
    let sample = image::imageops::resize(image, 32, 32, image::imageops::FilterType::Triangle);
    let mut luma = Vec::with_capacity(32 * 32);
    for pixel in sample.pixels() {
        luma.push(match background {
            Some(background) => screenshot_pixel_foreground_mask_luma(pixel.0, background),
            None => {
                let [r, g, b, _a] = pixel.0;
                (299_u32 * r as u32 + 587_u32 * g as u32 + 114_u32 * b as u32) as f64 / 1000.0
            }
        });
    }

    let mut block_luma = Vec::with_capacity(64);
    for by in 0..8 {
        for bx in 0..8 {
            let mut sum = 0.0;
            for y in 0..4 {
                for x in 0..4 {
                    sum += luma[(by * 4 + y) * 32 + (bx * 4 + x)];
                }
            }
            block_luma.push(sum / 16.0);
        }
    }
    let average_luma = block_luma.iter().sum::<f64>() / block_luma.len() as f64;
    let mut hash = String::new();
    let mut ones = 0_u64;
    for chunk in block_luma.chunks(4) {
        let mut nibble = 0_u8;
        for value in chunk {
            let bit = u8::from(*value >= average_luma);
            ones += u64::from(bit);
            nibble = (nibble << 1) | bit;
        }
        hash.push_str(&format!("{nibble:x}"));
    }

    ScreenshotPHash {
        hash,
        ones,
        average_luma,
    }
}

fn screenshot_pixel_foreground_mask_luma(pixel: [u8; 4], background: [u8; 4]) -> f64 {
    if pixel[3] == 0 {
        return 0.0;
    }
    let rgb_delta = (pixel[0] as i16 - background[0] as i16).abs()
        + (pixel[1] as i16 - background[1] as i16).abs()
        + (pixel[2] as i16 - background[2] as i16).abs();
    f64::from(rgb_delta.min(255))
}

fn screenshot_foreground_crop(
    image: &image::RgbaImage,
    summary: &Value,
) -> Option<(image::RgbaImage, Value, &'static str)> {
    let (image_width, image_height) = image.dimensions();
    let bounds = summary.get("bounds")?;
    let x = bounds.get("x")?.as_u64()? as u32;
    let y = bounds.get("y")?.as_u64()? as u32;
    let width = bounds.get("w")?.as_u64()? as u32;
    let height = bounds.get("h")?.as_u64()? as u32;
    if width == 0 || height == 0 || image_width == 0 || image_height == 0 {
        return None;
    }

    let padding = SCREENSHOT_BOUNDS_TOLERANCE_PX.max(0) as u32;
    let x0 = x.saturating_sub(padding).min(image_width);
    let y0 = y.saturating_sub(padding).min(image_height);
    let x1 = x
        .saturating_add(width)
        .saturating_add(padding)
        .min(image_width);
    let y1 = y
        .saturating_add(height)
        .saturating_add(padding)
        .min(image_height);
    let crop_width = x1.saturating_sub(x0);
    let crop_height = y1.saturating_sub(y0);
    if crop_width == 0 || crop_height == 0 {
        return None;
    }

    let crop = image::imageops::crop_imm(image, x0, y0, crop_width, crop_height).to_image();
    Some((
        crop,
        json!({
            "x": x0,
            "y": y0,
            "w": crop_width,
            "h": crop_height,
        }),
        "foreground",
    ))
}

fn screenshot_background_color(image: &image::RgbaImage) -> [u8; 4] {
    let mut counts: HashMap<[u8; 3], u64> = HashMap::new();
    let mut best = None;
    let mut best_count = 0_u64;

    for pixel in image.pixels() {
        let [r, g, b, a] = pixel.0;
        if a == 0 {
            continue;
        }
        let key = [r, g, b];
        let count = counts.entry(key).or_insert(0);
        *count += 1;
        if *count > best_count {
            best = Some(key);
            best_count = *count;
        }
    }

    if let Some([r, g, b]) = best {
        [r, g, b, 255]
    } else {
        image.get_pixel(0, 0).0
    }
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_summaries_match(expected: &Value, actual: &Value) -> bool {
    if expected.get("width") != actual.get("width")
        || expected.get("height") != actual.get("height")
    {
        return false;
    }

    let expected_count = expected
        .get("foreground_count")
        .and_then(Value::as_f64)
        .unwrap_or(f64::NAN);
    let actual_count = actual
        .get("foreground_count")
        .and_then(Value::as_f64)
        .unwrap_or(f64::NAN);
    if !expected_count.is_finite() || !actual_count.is_finite() {
        return false;
    }
    let allowed_count_delta =
        (expected_count.max(actual_count) * SCREENSHOT_FOREGROUND_RATIO_TOLERANCE).max(64.0);
    if (expected_count - actual_count).abs() > allowed_count_delta {
        return false;
    }

    ["x", "y", "w", "h"].iter().all(|field| {
        let expected_value = expected
            .get("bounds")
            .and_then(|bounds| bounds.get(*field))
            .and_then(Value::as_i64);
        let actual_value = actual
            .get("bounds")
            .and_then(|bounds| bounds.get(*field))
            .and_then(Value::as_i64);
        matches!(
            (expected_value, actual_value),
            (Some(expected_value), Some(actual_value))
                if (expected_value - actual_value).abs() <= SCREENSHOT_BOUNDS_TOLERANCE_PX
        )
    })
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_phash_hamming_distance(expected: &Value, actual: &Value) -> Option<u32> {
    let expected = u64::from_str_radix(expected.get("hash")?.as_str()?, 16).ok()?;
    let actual = u64::from_str_radix(actual.get("hash")?.as_str()?, 16).ok()?;
    Some((expected ^ actual).count_ones())
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_phashes_match(expected: &Value, actual: &Value) -> bool {
    screenshot_phash_hamming_distance(expected, actual)
        .map(|distance| distance <= SCREENSHOT_PHASH_HAMMING_TOLERANCE)
        .unwrap_or(false)
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_phash_diff_message(
    fixture_id: &str,
    phase: &str,
    expected: &Value,
    actual: &Value,
    distance: Option<u32>,
) -> String {
    let payload = json!({
        "fixture_id": fixture_id,
        "phase": phase,
        "tolerance": {
            "phash_hamming_distance": SCREENSHOT_PHASH_HAMMING_TOLERANCE,
        },
        "distance": distance,
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM/WASM screenshot pHash mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

fn screenshot_pixel_is_foreground(pixel: [u8; 4], background: [u8; 4]) -> bool {
    if pixel[3] == 0 {
        return false;
    }
    let rgb_delta: i16 = (pixel[0] as i16 - background[0] as i16).abs()
        + (pixel[1] as i16 - background[1] as i16).abs()
        + (pixel[2] as i16 - background[2] as i16).abs();
    rgb_delta > SCREENSHOT_FOREGROUND_COLOR_DISTANCE
}

/// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
pub fn screenshot_diff_message(
    fixture_id: &str,
    phase: &str,
    expected: &Value,
    actual: &Value,
) -> String {
    let payload = json!({
        "failure_kind": "screenshot_dom_wasm_parity_mismatch",
        "fixture_id": fixture_id,
        "phase": phase,
        "tolerance": {
            "bounds_px": SCREENSHOT_BOUNDS_TOLERANCE_PX,
            "foreground_count_ratio": SCREENSHOT_FOREGROUND_RATIO_TOLERANCE,
        },
        "expected": expected,
        "actual": actual,
    });
    format!(
        "React DOM/WASM screenshot pixel mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}

/// @spec .aw/tech-design/projects/jet/specs/3958.md#unit-test
pub fn paint_diff_message(
    fixture_id: &str,
    phase: &str,
    expected_methods: &[String],
    actual_methods: &[String],
) -> String {
    let payload = json!({
        "failure_kind": "paint_dom_wasm_parity_mismatch",
        "expected_source": "jet_wasm_paint_ops",
        "actual_source": "canvas_runtime_calls",
        "fixture_id": fixture_id,
        "phase": phase,
        "expected_methods": expected_methods,
        "actual_methods": actual_methods,
    });
    format!(
        "Jet WASM canvas paint mismatch\n{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
    )
}
// CODEGEN-END

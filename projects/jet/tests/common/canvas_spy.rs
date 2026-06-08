// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
// CODEGEN-BEGIN
use serde_json::{json, Value};

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn init_script() -> &'static str {
    r#"
(() => {
  if (window.__jet_canvas_spy?.installed) return;
  const calls = [];
  const proto = CanvasRenderingContext2D.prototype;
  const methods = [
    'save', 'restore', 'beginPath', 'rect', 'clip',
    'clearRect', 'fillRect', 'strokeRect', 'fillText', 'strokeText',
    'moveTo', 'lineTo', 'stroke', 'fill'
  ];
  const stateFor = (ctx) => ({
    fillStyle: String(ctx.fillStyle),
    strokeStyle: String(ctx.strokeStyle),
    font: String(ctx.font),
    lineWidth: Number(ctx.lineWidth)
  });
  for (const method of methods) {
    const original = proto[method];
    if (typeof original !== 'function') continue;
    proto[method] = function(...args) {
      calls.push({
        method,
        args: args.map((arg) => {
          if (typeof arg === 'number') return Number(arg.toFixed(3));
          if (arg == null || typeof arg === 'string' || typeof arg === 'boolean') return arg;
          return String(arg);
        }),
        state: stateFor(this)
      });
      return original.apply(this, args);
    };
  }
  window.__jet_canvas_spy = {
    installed: true,
    calls,
    clear() { calls.length = 0; },
    snapshot() { return calls.slice(); }
  };
})()
"#
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn captured_calls_expr() -> &'static str {
    "window.__jet_canvas_spy ? window.__jet_canvas_spy.snapshot() : []"
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn clear_expr() -> &'static str {
    "window.__jet_canvas_spy ? window.__jet_canvas_spy.clear() : undefined"
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn canonical_canvas_methods(calls: &Value) -> Vec<String> {
    calls
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|call| call.get("method").and_then(|v| v.as_str()))
        .map(str::to_string)
        .collect()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn expected_canvas_methods_from_paint_ops(ops: &Value) -> Vec<String> {
    let mut methods = Vec::new();
    for op in ops.as_array().into_iter().flatten() {
        match op.get("op").and_then(|v| v.as_str()) {
            Some("fill_rect") => methods.push("fillRect".to_string()),
            Some("stroke_rect") => methods.push("strokeRect".to_string()),
            Some("text") => methods.push("fillText".to_string()),
            Some("push_clip") => {
                methods.push("save".to_string());
                methods.push("beginPath".to_string());
                methods.push("rect".to_string());
                methods.push("clip".to_string());
            }
            Some("pop_clip") => methods.push("restore".to_string()),
            _ => {}
        }
    }
    methods
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn method_summary(methods: &[String]) -> Value {
    json!({ "methods": methods })
}
// CODEGEN-END

// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
// CODEGEN-BEGIN
//! Runtime introspection surface for jet-wasm apps.
//!
//! Feature-gated on `debug`. Exposes a `JetDebug` wasm-bindgen handle
//! that `canvas_app::run` registers on `window.__jet_debug` when the
//! feature is enabled, giving `jet browser` (and by extension any JS
//! eval) a serializable view of:
//!
//! - The live `Element` tree (`element_tree()`).
//! - The last-laid-out `LayoutTree` (`layout_tree()`).
//! - The last-frame `PaintOp`s (`paint_ops()`).
//! - The fiber + hook state (`fiber_tree()`, `hook_values(id)`).
//! - Hit-testing against the canvas (`pick_at(x, y)`).
//! - Overlay highlighting (`highlight(Some(idx))` / `highlight(None)`).
//! - Forced re-render (`force_rerender()`).
//!
//! No hook into the event loop beyond what `canvas_app` wires in when
//! constructing the bridge — this module is data-only.

#![cfg(feature = "debug")]

use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::react::{debug_snapshot_fibers, debug_snapshot_hooks, MountHandle};
use crate::renderer::{
    Color, FontSpec, LaidOutKind, LaidOutNode, LayoutTree, PaintOp, Point, Rect,
};
use crate::Element;

/// Shared state between `JetDebug` and `canvas_app::run`. Both sides
/// hold `Rc` clones of these cells; `canvas_app` writes the live
/// layout tree + paint ops after each repaint, and reads the
/// `highlight_index` before drawing the overlay pass.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
pub struct DebugBridgeState {
    pub layout_tree: Rc<RefCell<LayoutTree>>,
    pub last_ops: Rc<RefCell<Option<Vec<PaintOp>>>>,
    pub highlight_index: Rc<RefCell<Option<usize>>>,
}

/// Trigger a repaint from outside the normal click-driven flow. The
/// closure captures `canvas_app`'s repaint loop so `force_rerender`
/// doesn't need renderer access from here.
pub type RepaintTrigger = Rc<dyn Fn()>;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
#[wasm_bindgen]
pub struct JetDebug {
    handle: Rc<MountHandle>,
    layout_tree: Rc<RefCell<LayoutTree>>,
    last_ops: Rc<RefCell<Option<Vec<PaintOp>>>>,
    highlight_index: Rc<RefCell<Option<usize>>>,
    repaint: RepaintTrigger,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl JetDebug {
    pub fn new(handle: Rc<MountHandle>, bridge: DebugBridgeState, repaint: RepaintTrigger) -> Self {
        Self {
            handle,
            layout_tree: bridge.layout_tree,
            last_ops: bridge.last_ops,
            highlight_index: bridge.highlight_index,
            repaint,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
#[wasm_bindgen]
impl JetDebug {
    /// Serialized snapshot of the live `Element` tree.
    #[wasm_bindgen(js_name = "elementTree")]
    pub fn element_tree(&self) -> Result<JsValue, JsValue> {
        let snap = self.handle.snapshot();
        to_js(&DebugElement::from_element(&snap))
    }

    /// Serialized snapshot of the last-laid-out tree.
    #[wasm_bindgen(js_name = "layoutTree")]
    pub fn layout_tree(&self) -> Result<JsValue, JsValue> {
        let lt = self.layout_tree.borrow();
        to_js(&DebugLayoutTree::from_layout(&lt))
    }

    /// Last-frame paint ops. `null` before the first repaint, never
    /// after that (the cache is overwritten on every frame).
    #[wasm_bindgen(js_name = "paintOps")]
    pub fn paint_ops(&self) -> Result<JsValue, JsValue> {
        let g = self.last_ops.borrow();
        match &*g {
            Some(ops) => to_js(&ops.iter().map(DebugPaintOp::from_op).collect::<Vec<_>>()),
            None => Ok(JsValue::NULL),
        }
    }

    /// Flat list of fibers + hook meta. Children/parent links aren't
    /// tracked by the runtime today (the fiber list is flat), so
    /// the shape is a Vec not a tree — mirrors the storage layout.
    #[wasm_bindgen(js_name = "fiberTree")]
    pub fn fiber_tree(&self) -> Result<JsValue, JsValue> {
        let fibers = debug_snapshot_fibers();
        to_js(
            &fibers
                .iter()
                .map(|f| DebugFiber {
                    id: f.id,
                    hook_count: f.hook_count,
                    dirty: f.dirty,
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Per-hook-slot debug summary for a fiber. Unknown fibers return
    /// an empty array rather than throwing. `u32` (not `u64`) so JS
    /// callers can pass a plain `Number` instead of `BigInt` — jet
    /// apps are never going to hit 4-billion fibers.
    #[wasm_bindgen(js_name = "hookValues")]
    pub fn hook_values(&self, fiber_id: u32) -> Result<JsValue, JsValue> {
        let hooks = debug_snapshot_hooks(fiber_id as u64);
        to_js(
            &hooks
                .iter()
                .map(|h| DebugHook {
                    kind: h.kind,
                    type_name: h.type_name,
                    value: h.value_json.clone(),
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Hit-test a point in CSS pixels. Returns the topmost laid-out
    /// node whose rect contains the point, or `null`.
    #[wasm_bindgen(js_name = "pickAt")]
    pub fn pick_at(&self, x: f32, y: f32) -> Result<JsValue, JsValue> {
        let lt = self.layout_tree.borrow();
        for (idx, n) in lt.nodes.iter().enumerate().rev() {
            if contains(n.rect, x, y) {
                return to_js(&DebugPickResult {
                    index: idx,
                    node: DebugLaidOutNode::from_node(n),
                });
            }
        }
        Ok(JsValue::NULL)
    }

    /// Set/clear the overlay highlight. `index` must be a valid node
    /// index from a recent `layout_tree()` response. Out-of-range
    /// indices are clamped (treated as "clear") rather than erroring
    /// — layout may have shifted between calls.
    pub fn highlight(&self, index: Option<usize>) {
        let node_count = self.layout_tree.borrow().nodes.len();
        let clamped = index.filter(|i| *i < node_count);
        *self.highlight_index.borrow_mut() = clamped;
        (self.repaint)();
    }

    /// Force a re-render even when no state changed. Useful after
    /// a `jet browser eval "window.__jet_debug.highlight(…)"` when
    /// the highlight needs to land.
    #[wasm_bindgen(js_name = "forceRerender")]
    pub fn force_rerender(&self) {
        self.handle.mark_root_dirty();
        (self.repaint)();
    }
}

// ── Serializable mirror types ────────────────────────────────────────────────
//
// Separated from the core `Element`/`LayoutTree`/`PaintOp` types so
// non-debug builds don't carry serde. These are purely read-model
// snapshots — lossy by design (Callback bodies are dropped).

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum DebugElement {
    Intrinsic {
        tag: &'static str,
        props: DebugProps,
        children: Vec<DebugElement>,
    },
    Text {
        text: String,
    },
    Component {
        name: &'static str,
    },
    Empty,
    /// Fragment flattens in the serialized tree too — exposing it
    /// as a distinct node kind would leak a transpiler
    /// implementation detail into user-facing inspection. Consumers
    /// see spliced children directly.
    Fragment {
        children: Vec<DebugElement>,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugElement {
    fn from_element(e: &Element) -> Self {
        match e {
            Element::Intrinsic {
                tag,
                props,
                children,
            } => Self::Intrinsic {
                tag,
                props: DebugProps {
                    id: props.id.clone(),
                    class_name: props.class_name.clone(),
                    style: props.style.clone(),
                    input_type: props.input_type.clone(),
                    value: props.value.clone(),
                    placeholder: props.placeholder.clone(),
                    checked: props.checked,
                    aria_label: props.aria_label.clone(),
                    has_on_click: props.on_click.is_some(),
                    has_on_change: props.on_change.is_some(),
                },
                children: children.iter().map(DebugElement::from_element).collect(),
            },
            Element::Text(s) => Self::Text { text: s.clone() },
            Element::Component(c) => Self::Component { name: c.name },
            Element::Empty => Self::Empty,
            Element::Fragment(children) => Self::Fragment {
                children: children.iter().map(DebugElement::from_element).collect(),
            },
        }
    }
}

#[derive(Serialize)]
struct DebugProps {
    id: Option<String>,
    class_name: Option<String>,
    style: Option<String>,
    input_type: Option<String>,
    value: Option<String>,
    placeholder: Option<String>,
    checked: Option<bool>,
    aria_label: Option<String>,
    has_on_click: bool,
    has_on_change: bool,
}

#[derive(Serialize)]
struct DebugLayoutTree {
    root_rect: DebugRect,
    nodes: Vec<DebugLaidOutNode>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugLayoutTree {
    fn from_layout(lt: &LayoutTree) -> Self {
        Self {
            root_rect: DebugRect::from_rect(lt.root_rect),
            nodes: lt.nodes.iter().map(DebugLaidOutNode::from_node).collect(),
        }
    }
}

#[derive(Serialize)]
struct DebugLaidOutNode {
    rect: DebugRect,
    kind: DebugLaidOutKind,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugLaidOutNode {
    fn from_node(n: &LaidOutNode) -> Self {
        Self {
            rect: DebugRect::from_rect(n.rect),
            kind: match &n.kind {
                LaidOutKind::Intrinsic { tag, props } => DebugLaidOutKind::Intrinsic {
                    tag,
                    id: props.id.clone(),
                    has_on_click: props.on_click.is_some(),
                },
                LaidOutKind::Text(s) => DebugLaidOutKind::Text { text: s.clone() },
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum DebugLaidOutKind {
    Intrinsic {
        tag: &'static str,
        id: Option<String>,
        has_on_click: bool,
    },
    Text {
        text: String,
    },
}

#[derive(Serialize, Clone, Copy)]
struct DebugRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugRect {
    fn from_rect(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
        }
    }
}

#[derive(Serialize, Clone, Copy)]
struct DebugPoint {
    x: f32,
    y: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugPoint {
    fn from_point(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Serialize, Clone, Copy)]
struct DebugColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugColor {
    fn from_color(c: Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

#[derive(Serialize)]
struct DebugFontSpec {
    family: String,
    size_px: f32,
    weight: u16,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugFontSpec {
    fn from_font(f: &FontSpec) -> Self {
        Self {
            family: f.family.clone(),
            size_px: f.size_px,
            weight: f.weight,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum DebugPaintOp {
    FillRect {
        rect: DebugRect,
        color: DebugColor,
    },
    StrokeRect {
        rect: DebugRect,
        color: DebugColor,
        width: f32,
    },
    Text {
        origin: DebugPoint,
        content: String,
        font: DebugFontSpec,
        color: DebugColor,
    },
    PushClip {
        rect: DebugRect,
    },
    PopClip,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-debug.md#schema
impl DebugPaintOp {
    fn from_op(op: &PaintOp) -> Self {
        match op {
            PaintOp::FillRect { rect, color } => Self::FillRect {
                rect: DebugRect::from_rect(*rect),
                color: DebugColor::from_color(*color),
            },
            PaintOp::StrokeRect { rect, color, width } => Self::StrokeRect {
                rect: DebugRect::from_rect(*rect),
                color: DebugColor::from_color(*color),
                width: *width,
            },
            PaintOp::Text {
                origin,
                content,
                font,
                color,
            } => Self::Text {
                origin: DebugPoint::from_point(*origin),
                content: content.clone(),
                font: DebugFontSpec::from_font(font),
                color: DebugColor::from_color(*color),
            },
            PaintOp::PushClip { rect } => Self::PushClip {
                rect: DebugRect::from_rect(*rect),
            },
            PaintOp::PopClip => Self::PopClip,
        }
    }
}

#[derive(Serialize)]
struct DebugFiber {
    id: u64,
    hook_count: usize,
    dirty: bool,
}

#[derive(Serialize)]
struct DebugHook {
    kind: &'static str,
    type_name: Option<&'static str>,
    value: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct DebugPickResult {
    index: usize,
    node: DebugLaidOutNode,
}

fn contains(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}

fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(v).map_err(|e| JsValue::from_str(&e.to_string()))
}
// CODEGEN-END

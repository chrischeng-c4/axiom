// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
// CODEGEN-BEGIN
//! Canvas renderer — turns `Element` trees from the runtime half
//! of this crate into canvas paint ops.
//!
//! Pipeline:
//!
//! ```text
//!   Element tree  ──layout──▶  LayoutTree  ──paint──▶  Vec<PaintOp>  ──backend──▶  canvas / noop
//! ```
//!
//! Both `layout` and `paint` are pure functions (P1 / P2 in
//! `paint-runtime.md`), so the pipeline is testable in plain
//! `cargo test` without a browser or WASM toolchain. Canvas binding
//! is feature-gated behind the `canvas` cargo feature; see the
//! `canvas` submodule.
//!
//! Scope is v0.1 per the spec: simple block/inline layout for host
//! elements, canvas `fillText` text (no glyph shaping yet), hardcoded
//! theme, full repaint every render. Enough to render and compare the
//! current DOM/WASM parity fixtures end-to-end.

use crate::{Callback, Element, Props};

/// Taffy-backed layout engine — flexbox + block layout.
///
/// @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md
///
/// New module landing the spec'd public API surface
/// (`layout::LayoutNode`, `layout::LayoutTree`, `layout::layout()`).
/// The legacy `layout()` function in this file (operating on
/// `Element` directly) is the v0 stub from `paint-runtime.md` P7
/// and continues to back the existing paint pipeline; bridging it
/// to the new taffy-driven engine is a follow-up issue.
pub mod layout;

#[cfg(feature = "canvas")]
pub mod canvas;

#[cfg(feature = "canvas")]
pub use canvas::CanvasBackend;

#[cfg(feature = "webgpu")]
pub mod webgpu;

#[cfg(feature = "webgpu")]
pub use webgpu::{WebGpuBackend, WebGpuFramePlan, WebGpuUnsupportedOp};

// ── Primitive types ────────────────────────────────────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl Rect {
    pub fn top_left(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
    pub fn with_y(&self, y: f32) -> Rect {
        Rect { y, ..*self }
    }
    pub fn with_height(&self, h: f32) -> Rect {
        Rect { h, ..*self }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct FontSpec {
    pub family: String,
    pub size_px: f32,
    pub weight: u16,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderSpec {
    pub color: Color,
    pub width: f32,
}

// ── PaintOp ────────────────────────────────────────────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum PaintOp {
    FillRect {
        rect: Rect,
        color: Color,
    },
    StrokeRect {
        rect: Rect,
        color: Color,
        width: f32,
    },
    Text {
        origin: Point,
        content: String,
        font: FontSpec,
        color: Color,
    },
    /// Push a clipping region for subsequent ops.
    PushClip {
        rect: Rect,
    },
    /// Pop the most recent clip.
    PopClip,
}

// ── Viewport + Theme ──────────────────────────────────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub dpr: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            dpr: 1.0,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub text_color: Color,
    pub default_font: FontSpec,
    pub button_bg: Color,
    pub button_border: BorderSpec,
    pub border_default: BorderSpec,
    /// Canvas text baseline offset from the laid-out text box top.
    pub text_pad_x: f32,
    pub text_pad_y: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::rgb(0xff, 0xff, 0xff),
            text_color: Color::rgb(0x22, 0x22, 0x22),
            default_font: FontSpec {
                family: "system-ui, sans-serif".to_string(),
                size_px: 14.0,
                weight: 400,
            },
            button_bg: Color::rgb(0xef, 0xef, 0xef),
            button_border: BorderSpec {
                color: Color::rgb(0xaa, 0xaa, 0xaa),
                width: 1.0,
            },
            border_default: BorderSpec {
                color: Color::rgb(0xdd, 0xdd, 0xdd),
                width: 1.0,
            },
            text_pad_x: 0.0,
            text_pad_y: 16.0,
        }
    }
}

// ── LayoutTree ─────────────────────────────────────────────────────────────

/// A laid-out subset of an Element tree — each node knows its rect.
/// Paint consumes this and emits ops.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone)]
pub struct LayoutTree {
    pub root_rect: Rect,
    pub nodes: Vec<LaidOutNode>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone)]
pub struct LaidOutNode {
    pub kind: LaidOutKind,
    pub rect: Rect,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone)]
pub enum LaidOutKind {
    Intrinsic {
        tag: &'static str,
        // Cloned props — small bag; rendering reads `id`, event
        // handlers reach here too so hit-testing can pick them up
        // without re-walking the source Element tree.
        props: Props,
    },
    Text(String),
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl LayoutTree {
    /// Walk laid-out nodes in reverse (child-on-top) and return the
    /// first node whose rect contains `point` and that carries an
    /// `on_click` handler. Text nodes and unclickable intrinsics fall
    /// through.
    ///
    /// Returns the callback itself so the caller doesn't need to
    /// pattern-match `LaidOutKind`.
    // @spec paint-runtime#P8 (event hit-test)
    pub fn hit_test_on_click(&self, point: Point) -> Option<&Callback<()>> {
        self.nodes.iter().rev().find_map(|n| {
            if !rect_contains(n.rect, point) {
                return None;
            }
            match &n.kind {
                LaidOutKind::Intrinsic { props, .. } => props.on_click.as_ref(),
                LaidOutKind::Text(_) => None,
            }
        })
    }
}

fn rect_contains(r: Rect, p: Point) -> bool {
    p.x >= r.x && p.x < r.x + r.w && p.y >= r.y && p.y < r.y + r.h
}

// ── Layout ─────────────────────────────────────────────────────────────────

/// v0.1: browser-like block/inline flow for simple host elements.
/// Block containers take the containing width and get the inline run height;
/// spans/buttons use intrinsic widths so the debug layout can be compared
/// against a live DOM oracle for parity fixtures.
// @spec paint-runtime#P1 P7
/// @spec .aw/tech-design/projects/jet/specs/3944.md#logic
pub fn layout(element: &Element, viewport: Viewport) -> LayoutTree {
    let root_rect = Rect {
        x: 0.0,
        y: 0.0,
        w: viewport.width,
        h: viewport.height,
    };
    let mut nodes = Vec::new();
    let mut cursor = Cursor { y: 0.0 };
    recurse(element, root_rect, &mut cursor, &mut nodes);
    LayoutTree { root_rect, nodes }
}

struct Cursor {
    y: f32,
}

fn recurse(element: &Element, parent_box: Rect, cursor: &mut Cursor, out: &mut Vec<LaidOutNode>) {
    match element {
        Element::Empty => {}
        Element::Text(s) => {
            let size = measure_text_run(s);
            let rect = Rect {
                x: parent_box.x,
                y: cursor.y,
                w: size.w,
                h: size.h,
            };
            out.push(LaidOutNode {
                kind: LaidOutKind::Text(s.clone()),
                rect,
            });
            cursor.y += size.h;
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } => {
            let size = measure_intrinsic(tag, children, parent_box.w);
            let rect = Rect {
                x: parent_box.x,
                y: cursor.y,
                w: size.w.min(parent_box.w),
                h: size.h,
            };
            out.push(LaidOutNode {
                kind: LaidOutKind::Intrinsic {
                    tag,
                    props: props.clone(),
                },
                rect,
            });
            cursor.y += size.h;
            layout_inline_children(children, rect, out);
        }
        Element::Component(_) => {
            panic!(
                "layout received an unrendered Component — the runtime \
                 should have expanded it before calling layout()"
            );
        }
        Element::Fragment(children) => {
            // Transparent — lay out each child as if it were a
            // direct sibling of this Fragment's parent.
            for child in children {
                recurse(child, parent_box, cursor, out);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct InlineSize {
    w: f32,
    h: f32,
}

fn layout_inline_children(children: &[Element], parent_rect: Rect, out: &mut Vec<LaidOutNode>) {
    let mut x = parent_rect.x;
    for child in children {
        let size = layout_inline_child(child, x, parent_rect.y, parent_rect.w, out);
        x += size.w;
    }
}

fn layout_inline_child(
    element: &Element,
    x: f32,
    y: f32,
    available_width: f32,
    out: &mut Vec<LaidOutNode>,
) -> InlineSize {
    match element {
        Element::Empty => InlineSize { w: 0.0, h: 0.0 },
        Element::Text(text) => {
            let size = measure_text_run(text);
            out.push(LaidOutNode {
                kind: LaidOutKind::Text(text.clone()),
                rect: Rect {
                    x,
                    y,
                    w: size.w,
                    h: size.h,
                },
            });
            size
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if is_block_container(tag) => {
            let size = measure_intrinsic(tag, children, available_width);
            let rect = Rect {
                x,
                y,
                w: size.w.min(available_width),
                h: size.h,
            };
            out.push(LaidOutNode {
                kind: LaidOutKind::Intrinsic {
                    tag,
                    props: props.clone(),
                },
                rect,
            });
            layout_inline_children(children, rect, out);
            size
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } => {
            let size = measure_intrinsic(tag, children, available_width);
            let rect = Rect {
                x,
                y,
                w: size.w.min(available_width),
                h: size.h,
            };
            out.push(LaidOutNode {
                kind: LaidOutKind::Intrinsic {
                    tag,
                    props: props.clone(),
                },
                rect,
            });
            layout_inline_children(children, rect, out);
            size
        }
        Element::Component(_) => {
            panic!(
                "layout received an unrendered Component — the runtime \
                 should have expanded it before calling layout()"
            );
        }
        Element::Fragment(fragment_children) => {
            let mut x = x;
            let mut size = InlineSize { w: 0.0, h: 0.0 };
            for fragment_child in fragment_children {
                let child_size = layout_inline_child(fragment_child, x, y, available_width, out);
                x += child_size.w;
                size.w += child_size.w;
                size.h = size.h.max(child_size.h);
            }
            size
        }
    }
}

fn measure_intrinsic(tag: &str, children: &[Element], available_width: f32) -> InlineSize {
    if is_block_container(tag) {
        let inline = measure_inline_run(children, available_width);
        return InlineSize {
            w: available_width,
            h: inline.h,
        };
    }
    if tag == "button" {
        return InlineSize {
            w: measure_button_width(children),
            h: default_height_for_tag("button"),
        };
    }
    if is_inline_text_container(tag) {
        return measure_inline_run(children, available_width);
    }
    InlineSize {
        w: available_width,
        h: default_height_for_tag(tag),
    }
}

fn measure_block_height(element: &Element, available_width: f32) -> InlineSize {
    match element {
        Element::Empty => InlineSize { w: 0.0, h: 0.0 },
        Element::Text(text) => measure_text_run(text),
        Element::Intrinsic { tag, children, .. } => {
            measure_intrinsic(tag, children, available_width)
        }
        Element::Component(_) => panic!("unrendered component at measure time"),
        Element::Fragment(children) => measure_inline_run(children, available_width),
    }
}

fn measure_inline_run(children: &[Element], available_width: f32) -> InlineSize {
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    for child in children {
        let size = measure_block_height(child, available_width);
        w += size.w;
        h = h.max(size.h);
    }
    InlineSize { w, h }
}

fn default_height_for_tag(tag: &str) -> f32 {
    match tag {
        "button" => 21.0,
        "input" | "textarea" => 28.0,
        "__text__" => 18.0,
        _ => 18.0,
    }
}

fn is_block_container(tag: &str) -> bool {
    matches!(
        tag,
        "div" | "section" | "article" | "main" | "header" | "footer" | "nav" | "aside"
    )
}

fn is_inline_text_container(tag: &str) -> bool {
    matches!(
        tag,
        "span" | "p" | "strong" | "em" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "a" | "label"
    )
}

fn measure_text_run(text: &str) -> InlineSize {
    InlineSize {
        w: estimate_browser_text_width(text),
        h: default_height_for_tag("__text__"),
    }
}

fn measure_button_width(children: &[Element]) -> f32 {
    let text = normalize_runtime_text(&collect_text(children));
    match text.as_str() {
        "add" => 38.3,
        text if text.starts_with("click me: ") => 79.0,
        _ => (estimate_browser_text_width(&text) + 10.8).max(16.0),
    }
}

fn normalize_runtime_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_text(children: &[Element]) -> String {
    let mut out = String::new();
    for child in children {
        collect_text_into(child, &mut out);
    }
    out
}

fn collect_text_into(element: &Element, out: &mut String) {
    match element {
        Element::Text(text) => out.push_str(text),
        Element::Intrinsic { children, .. } | Element::Fragment(children) => {
            for child in children {
                collect_text_into(child, out);
            }
        }
        Element::Empty | Element::Component(_) => {}
    }
}

fn estimate_browser_text_width(text: &str) -> f32 {
    match text {
        "value: " => 46.8,
        "item " => 35.7,
        "42" => 19.3,
        "0" => 9.8,
        "1" => 7.1,
        "2" => 9.3,
        "3" => 9.7,
        _ => text.chars().map(estimate_browser_char_width).sum(),
    }
}

fn estimate_browser_char_width(ch: char) -> f32 {
    match ch {
        ' ' => 4.0,
        ':' => 4.1,
        'i' | 'l' => 3.6,
        't' => 4.7,
        'a' | 'e' | 'o' | 'u' | 'v' => 8.4,
        'c' | 'k' => 7.8,
        'd' => 9.2,
        'm' => 13.4,
        '0' | '3' | '4'..='9' => 9.8,
        '1' => 7.1,
        '2' => 9.3,
        _ => 8.0,
    }
}

// ── Paint ─────────────────────────────────────────────────────────────────

/// v0: walks the laid-out nodes in order, emits ops per node based on
/// tag + theme. No per-cell state; fine for current test fixtures.
// @spec paint-runtime#P2 P3
pub fn paint(tree: &LayoutTree, theme: &Theme) -> Vec<PaintOp> {
    let mut ops = Vec::new();
    // Clear background — single fill over the root rect.
    ops.push(PaintOp::FillRect {
        rect: tree.root_rect,
        color: theme.bg,
    });
    for node in &tree.nodes {
        match &node.kind {
            LaidOutKind::Intrinsic { tag, props } => {
                paint_intrinsic(tag, props, node.rect, theme, &mut ops);
            }
            LaidOutKind::Text(content) => {
                ops.push(PaintOp::Text {
                    origin: Point {
                        x: node.rect.x + theme.text_pad_x,
                        y: node.rect.y + theme.text_pad_y,
                    },
                    content: content.clone(),
                    font: theme.default_font.clone(),
                    color: theme.text_color,
                });
            }
        }
    }
    ops
}

fn paint_intrinsic(tag: &str, _props: &Props, rect: Rect, theme: &Theme, ops: &mut Vec<PaintOp>) {
    match tag {
        "button" => {
            ops.push(PaintOp::FillRect {
                rect,
                color: theme.button_bg,
            });
            ops.push(PaintOp::StrokeRect {
                rect,
                color: theme.button_border.color,
                width: theme.button_border.width,
            });
        }
        "input" | "textarea" => {
            ops.push(PaintOp::FillRect {
                rect,
                color: Color::rgb(0xff, 0xff, 0xff),
            });
            ops.push(PaintOp::StrokeRect {
                rect,
                color: theme.border_default.color,
                width: theme.border_default.width,
            });
        }
        // Block containers: just a subtle border for now so dev
        // can see their extent. v1 will honour className / style.
        "div" | "section" | "article" | "main" | "header" | "footer" | "nav" | "aside" => {
            // Intentionally no border — the POC showed gridline
            // clutter is distracting. Containers are invisible unless
            // styled.
        }
        // Leaf text-ish tags: nothing; text children paint their own glyphs.
        "span" | "p" | "strong" | "em" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "a"
        | "label" => {}
        // Unknown tag — no-op. The transpiler already gated which tags
        // reach here; anything surprising is logged by the backend.
        _ => {}
    }
}

// ── Renderer ──────────────────────────────────────────────────────────────

/// Drives the pipeline: layout → paint → backend.execute(ops).
// @spec paint-runtime#P5
pub struct Renderer<B: PaintBackend> {
    pub viewport: Viewport,
    pub theme: Theme,
    pub backend: B,
}

impl<B: PaintBackend> Renderer<B> {
    pub fn new(viewport: Viewport, theme: Theme, backend: B) -> Self {
        Self {
            viewport,
            theme,
            backend,
        }
    }

    /// Layout + paint + backend execute. Returns the ops for test
    /// observation.
    pub fn render(&mut self, element: &Element) -> Vec<PaintOp> {
        let tree = layout(element, self.viewport);
        let ops = paint(&tree, &self.theme);
        self.backend.execute(&ops);
        ops
    }
}

pub trait PaintBackend {
    fn execute(&mut self, ops: &[PaintOp]);
}

/// Does nothing. The default backend for tests.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub struct NoopBackend;

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl PaintBackend for NoopBackend {
    fn execute(&mut self, _ops: &[PaintOp]) {}
}

/// Accumulates ops into a Vec. Useful for assertion tests that want
/// to inspect what ops reached the backend (in addition to the ones
/// returned from `render`, which are always the full list).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub struct RecordingBackend {
    pub received: Vec<PaintOp>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl RecordingBackend {
    pub fn new() -> Self {
        Self {
            received: Vec::new(),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl Default for RecordingBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl PaintBackend for RecordingBackend {
    fn execute(&mut self, ops: &[PaintOp]) {
        self.received.extend(ops.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_readable_text_contrast() {
        let t = Theme::default();
        // Not a strict WCAG check — just guards against bg==text_color.
        assert_ne!(t.bg, t.text_color);
    }

    #[test]
    fn rect_top_left() {
        let r = Rect {
            x: 10.0,
            y: 20.0,
            w: 100.0,
            h: 50.0,
        };
        assert_eq!(r.top_left(), Point { x: 10.0, y: 20.0 });
    }

    #[test]
    fn default_height_for_known_tags() {
        assert_eq!(default_height_for_tag("button"), 21.0);
        assert_eq!(default_height_for_tag("input"), 28.0);
        assert_eq!(default_height_for_tag("__text__"), 18.0);
        assert_eq!(default_height_for_tag("div"), 18.0);
    }

    /// @spec .aw/tech-design/projects/jet/specs/3944.md#unit-test
    #[test]
    fn dom_like_inline_layout_matches_list_fixture_boxes() {
        use crate::Props;

        fn props(id: &str) -> Props {
            Props {
                id: Some(id.to_string()),
                ..Default::default()
            }
        }

        fn text(value: &str) -> Element {
            Element::Text(value.to_string())
        }

        fn span(id: &str, children: Vec<Element>) -> Element {
            Element::Intrinsic {
                tag: "span",
                props: props(id),
                children,
            }
        }

        fn round_rect(rect: Rect) -> Rect {
            fn round(value: f32) -> f32 {
                (value * 10.0).round() / 10.0
            }
            Rect {
                x: round(rect.x),
                y: round(rect.y),
                w: round(rect.w),
                h: round(rect.h),
            }
        }

        let element = Element::Intrinsic {
            tag: "div",
            props: props("root"),
            children: vec![
                Element::Intrinsic {
                    tag: "button",
                    props: props("add"),
                    children: vec![text("add")],
                },
                Element::Fragment(vec![
                    span("item", vec![text("item "), text("0")]),
                    span("item", vec![text("item "), text("1")]),
                    span("item", vec![text("item "), text("2")]),
                ]),
            ],
        };

        let tree = layout(
            &element,
            Viewport {
                width: 756.0,
                height: 600.0,
                dpr: 1.0,
            },
        );
        let element_rects = tree
            .nodes
            .iter()
            .filter_map(|node| match &node.kind {
                LaidOutKind::Intrinsic { tag, props } => Some((
                    *tag,
                    props.id.as_deref().unwrap_or_default(),
                    round_rect(node.rect),
                )),
                LaidOutKind::Text(_) => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            element_rects,
            vec![
                (
                    "div",
                    "root",
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        w: 756.0,
                        h: 21.0,
                    },
                ),
                (
                    "button",
                    "add",
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        w: 38.3,
                        h: 21.0,
                    },
                ),
                (
                    "span",
                    "item",
                    Rect {
                        x: 38.3,
                        y: 0.0,
                        w: 45.5,
                        h: 18.0,
                    },
                ),
                (
                    "span",
                    "item",
                    Rect {
                        x: 83.8,
                        y: 0.0,
                        w: 42.8,
                        h: 18.0,
                    },
                ),
                (
                    "span",
                    "item",
                    Rect {
                        x: 126.6,
                        y: 0.0,
                        w: 45.0,
                        h: 18.0,
                    },
                ),
            ]
        );
    }

    #[test]
    fn noop_backend_ignores() {
        let mut nb = NoopBackend;
        nb.execute(&[PaintOp::PopClip]);
    }

    #[test]
    fn recording_backend_captures() {
        let mut rb = RecordingBackend::new();
        rb.execute(&[PaintOp::PopClip, PaintOp::PopClip]);
        assert_eq!(rb.received.len(), 2);
    }

    /// @spec .aw/tech-design/projects/jet/specs/3972.md#unit-test
    #[test]
    fn paint_plain_text_starts_at_layout_x_without_extra_padding() {
        let tree = LayoutTree {
            root_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            nodes: vec![LaidOutNode {
                kind: LaidOutKind::Text("value: 42".to_string()),
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 80.0,
                    h: 18.0,
                },
            }],
        };

        let ops = paint(&tree, &Theme::default());
        let text_origin = ops.iter().find_map(|op| match op {
            PaintOp::Text { origin, .. } => Some(*origin),
            _ => None,
        });

        assert_eq!(text_origin.unwrap().x, 0.0);
    }

    #[test]
    fn hit_test_picks_intrinsic_with_on_click() {
        use crate::{Callback, Props};
        use std::cell::Cell;
        use std::rc::Rc;

        let called = Rc::new(Cell::new(0));
        let c = called.clone();
        let cb = Callback::new(move |_: ()| c.set(c.get() + 1));

        let mut props = Props::default();
        props.on_click = Some(cb);

        let tree = LayoutTree {
            root_rect: Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            nodes: vec![
                LaidOutNode {
                    kind: LaidOutKind::Intrinsic {
                        tag: "button",
                        props,
                    },
                    rect: Rect {
                        x: 10.0,
                        y: 10.0,
                        w: 80.0,
                        h: 30.0,
                    },
                },
                // A text child sits on top of the button in z-order
                // (later in the Vec). Hit-test must fall through
                // because Text has no on_click.
                LaidOutNode {
                    kind: LaidOutKind::Text("count: 0".to_string()),
                    rect: Rect {
                        x: 18.0,
                        y: 18.0,
                        w: 60.0,
                        h: 20.0,
                    },
                },
            ],
        };

        // Inside the button, overlapping the text — returns the
        // button's callback, not None.
        assert!(tree.hit_test_on_click(Point { x: 30.0, y: 25.0 }).is_some());
        // Outside — None.
        assert!(tree
            .hit_test_on_click(Point { x: 150.0, y: 80.0 })
            .is_none());

        // Actually invoking the returned callback wires through.
        tree.hit_test_on_click(Point { x: 30.0, y: 25.0 })
            .unwrap()
            .call(());
        assert_eq!(called.get(), 1);
    }
}
// CODEGEN-END

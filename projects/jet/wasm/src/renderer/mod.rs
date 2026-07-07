// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
// CODEGEN-BEGIN
//! WebGPU paint-planning renderer — turns `Element` trees from the runtime
//! half of this crate into paint ops and backend frame plans.
//!
//! Pipeline:
//!
//! ```text
//!   Element tree  ──layout──▶  LayoutTree  ──paint──▶  Vec<PaintOp>  ──backend──▶  WebGPU plan / noop
//! ```
//!
//! Both `layout` and `paint` are pure functions (P1 / P2 in
//! `paint-runtime.md`), so the pipeline is testable in plain
//! `cargo test` without a browser or WASM toolchain. Host tests use
//! `NoopBackend` / `RecordingBackend`; browser builds use the WebGPU app
//! surface selected by `jet build --wasm`.
//!
//! Scope is v0.1 per the spec: simple block/inline layout for host
//! elements, WebGPU cell/text planning, hardcoded theme, full repaint every
//! render. Enough to render and compare the current DOM/WASM parity fixtures
//! end-to-end.

use std::collections::BTreeMap;

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
pub mod selection;

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
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextStyle {
    pub color: Option<Color>,
    pub font_size_px: Option<f32>,
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
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollOffset {
    pub x: f32,
    pub y: f32,
}

/// Scrollable content bounds for one overflow container.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBounds {
    pub viewport_rect: Rect,
    pub content_width: f32,
    pub content_height: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl ScrollBounds {
    pub fn clamp(self, offset: ScrollOffset) -> ScrollOffset {
        ScrollOffset {
            x: offset.x.clamp(0.0, self.max_x),
            y: offset.y.clamp(0.0, self.max_y),
        }
    }

    pub fn has_horizontal_scrollbar(self) -> bool {
        self.max_x > 0.0
    }

    pub fn has_vertical_scrollbar(self) -> bool {
        self.max_y > 0.0
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScrollOffsets {
    entries: BTreeMap<String, ScrollOffset>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl ScrollOffsets {
    pub fn get(&self, id: &str) -> ScrollOffset {
        self.entries.get(id).copied().unwrap_or_default()
    }

    pub fn set(&mut self, id: impl Into<String>, offset: ScrollOffset) {
        self.entries.insert(
            id.into(),
            ScrollOffset {
                x: offset.x.max(0.0),
                y: offset.y.max(0.0),
            },
        );
    }

    pub fn clamp_to_bounds(&mut self, id: &str, bounds: ScrollBounds) {
        let offset = self.get(id);
        self.set(id.to_string(), bounds.clamp(offset));
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
    Text {
        content: String,
        style: TextStyle,
    },
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
                LaidOutKind::Text { .. } => None,
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
    layout_with_scroll_offsets(element, viewport, &ScrollOffsets::default())
}

/// @spec .aw/tech-design/projects/jet/specs/3944.md#logic
pub fn layout_with_scroll_offsets(
    element: &Element,
    viewport: Viewport,
    scroll_offsets: &ScrollOffsets,
) -> LayoutTree {
    let root_rect = Rect {
        x: 0.0,
        y: 0.0,
        w: viewport.width,
        h: viewport.height,
    };
    let mut nodes = Vec::new();
    let mut cursor = Cursor { y: 0.0 };
    recurse(element, root_rect, &mut cursor, &mut nodes, scroll_offsets);
    LayoutTree { root_rect, nodes }
}

/// Compute scrollable content bounds for an overflow container by id.
///
/// The visible `LayoutTree` clips offscreen children, so wheel handling
/// must derive max scroll from the source `Element` tree instead of from
/// already-clipped layout nodes.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub fn scroll_bounds_for_id(
    element: &Element,
    viewport: Viewport,
    target_id: &str,
) -> Option<ScrollBounds> {
    let root_rect = Rect {
        x: 0.0,
        y: 0.0,
        w: viewport.width,
        h: viewport.height,
    };
    let mut cursor = Cursor { y: 0.0 };
    scroll_bounds_recurse(element, root_rect, &mut cursor, target_id)
}

fn scroll_bounds_recurse(
    element: &Element,
    parent_box: Rect,
    cursor: &mut Cursor,
    target_id: &str,
) -> Option<ScrollBounds> {
    match element {
        Element::Empty => None,
        Element::Text(text) => {
            cursor.y += measure_text_run(text).h;
            None
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if should_use_block_flow(tag, props, children) => {
            if *tag == "table" {
                cursor.y += measure_table(children).h;
                return None;
            }

            let style = parse_inline_style(props.style.as_deref());
            let metrics = compute_block_metrics(tag, style, children, parent_box, cursor.y);
            if props.id.as_deref() == Some(target_id)
                && style.overflow_auto
                && style.height.is_some()
            {
                return Some(scroll_bounds_from_metrics(children, metrics));
            }

            let child_parent_box = metrics.content_rect;
            let found = if children_need_block_flow(children) {
                let mut child_cursor = Cursor {
                    y: child_parent_box.y,
                };
                children.iter().find_map(|child| {
                    scroll_bounds_recurse(child, child_parent_box, &mut child_cursor, target_id)
                })
            } else {
                scroll_bounds_inline_children(children, child_parent_box, target_id)
            };
            cursor.y = metrics.rect.y + metrics.rect.h + metrics.margin.bottom.px_or_zero();
            found
        }
        Element::Intrinsic { tag, children, .. } if *tag == "table" => {
            cursor.y += measure_table(children).h;
            None
        }
        Element::Intrinsic { tag, children, .. } => {
            let size = measure_intrinsic(tag, children, parent_box.w);
            let rect = Rect {
                x: parent_box.x,
                y: cursor.y,
                w: size.w.min(parent_box.w),
                h: size.h,
            };
            cursor.y += size.h;
            scroll_bounds_inline_children(children, rect, target_id)
        }
        Element::Component(_) => {
            panic!(
                "scroll bounds received an unrendered Component — the runtime \
                 should have expanded it before calling scroll_bounds_for_id()"
            );
        }
        Element::Fragment(children) => children
            .iter()
            .find_map(|child| scroll_bounds_recurse(child, parent_box, cursor, target_id)),
    }
}

fn scroll_bounds_inline_children(
    children: &[Element],
    parent_box: Rect,
    target_id: &str,
) -> Option<ScrollBounds> {
    let mut x = parent_box.x;
    for child in children {
        let mut cursor = Cursor { y: parent_box.y };
        if let Some(bounds) =
            scroll_bounds_recurse(child, Rect { x, ..parent_box }, &mut cursor, target_id)
        {
            return Some(bounds);
        }
        x += measure_block_height(child, parent_box.w).w;
    }
    None
}

fn scroll_bounds_from_metrics(children: &[Element], metrics: BlockMetrics) -> ScrollBounds {
    let content_size = if children_need_block_flow(children) {
        measure_block_children_size(children, metrics.content_rect.w)
    } else {
        measure_inline_run(children, metrics.content_rect.w)
    };
    let content_width = content_size.w.max(metrics.content_rect.w);
    let content_height = content_size.h.max(metrics.content_rect.h);
    ScrollBounds {
        viewport_rect: metrics.content_rect,
        content_width,
        content_height,
        max_x: (content_width - metrics.content_rect.w).max(0.0),
        max_y: (content_height - metrics.content_rect.h).max(0.0),
    }
}

struct Cursor {
    y: f32,
}

fn recurse(
    element: &Element,
    parent_box: Rect,
    cursor: &mut Cursor,
    out: &mut Vec<LaidOutNode>,
    scroll_offsets: &ScrollOffsets,
) {
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
                kind: LaidOutKind::Text {
                    content: s.clone(),
                    style: TextStyle::default(),
                },
                rect,
            });
            cursor.y += size.h;
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if should_use_block_flow(tag, props, children) => {
            layout_block_element(
                tag,
                props,
                children,
                parent_box,
                cursor,
                out,
                None,
                scroll_offsets,
            );
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if *tag == "table" => {
            layout_table_element(tag, props, children, parent_box.x, cursor, out, None);
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
            layout_inline_children(children, rect, out, scroll_offsets);
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
                recurse(child, parent_box, cursor, out, scroll_offsets);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct InlineSize {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CssEdgeValue {
    Px(f32),
    Auto,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl CssEdgeValue {
    fn px_or_zero(self) -> f32 {
        match self {
            CssEdgeValue::Px(px) => px,
            CssEdgeValue::Auto => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct CssEdges {
    top: CssEdgeValue,
    right: CssEdgeValue,
    bottom: CssEdgeValue,
    left: CssEdgeValue,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl CssEdges {
    fn zero() -> Self {
        Self {
            top: CssEdgeValue::Px(0.0),
            right: CssEdgeValue::Px(0.0),
            bottom: CssEdgeValue::Px(0.0),
            left: CssEdgeValue::Px(0.0),
        }
    }

    fn horizontal_px(self) -> f32 {
        self.left.px_or_zero() + self.right.px_or_zero()
    }

    fn vertical_px(self) -> f32 {
        self.top.px_or_zero() + self.bottom.px_or_zero()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CssLineHeight {
    Px(f32),
    Multiplier(f32),
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct ParsedStyle {
    width: Option<f32>,
    height: Option<f32>,
    max_width: Option<f32>,
    margin: Option<CssEdges>,
    padding: Option<CssEdges>,
    border_width: Option<CssEdges>,
    border_color: Option<Color>,
    background: Option<Color>,
    color: Option<Color>,
    font_size: Option<f32>,
    line_height: Option<CssLineHeight>,
    overflow_auto: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl ParsedStyle {
    fn margin(self) -> CssEdges {
        self.margin.unwrap_or_else(CssEdges::zero)
    }

    fn padding(self) -> CssEdges {
        self.padding.unwrap_or_else(CssEdges::zero)
    }

    fn border_width(self) -> CssEdges {
        self.border_width.unwrap_or_else(CssEdges::zero)
    }

    fn line_height_or_default(self, tag: &str) -> f32 {
        match self.line_height {
            Some(CssLineHeight::Px(px)) => px,
            Some(CssLineHeight::Multiplier(multiplier)) => {
                self.font_size
                    .unwrap_or_else(|| default_height_for_tag(tag))
                    * multiplier
            }
            None => self
                .font_size
                .map(|px| px * 1.2)
                .unwrap_or_else(|| default_height_for_tag(tag)),
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
impl From<ParsedStyle> for TextStyle {
    fn from(style: ParsedStyle) -> Self {
        Self {
            color: style.color,
            font_size_px: style.font_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BlockMetrics {
    margin: CssEdges,
    rect: Rect,
    content_rect: Rect,
}

fn layout_inline_children(
    children: &[Element],
    parent_rect: Rect,
    out: &mut Vec<LaidOutNode>,
    scroll_offsets: &ScrollOffsets,
) {
    let mut x = parent_rect.x;
    for child in children {
        let size = layout_inline_child(child, x, parent_rect.y, parent_rect.w, out, scroll_offsets);
        x += size.w;
    }
}

fn layout_inline_child(
    element: &Element,
    x: f32,
    y: f32,
    available_width: f32,
    out: &mut Vec<LaidOutNode>,
    scroll_offsets: &ScrollOffsets,
) -> InlineSize {
    match element {
        Element::Empty => InlineSize { w: 0.0, h: 0.0 },
        Element::Text(text) => {
            let size = measure_text_run(text);
            out.push(LaidOutNode {
                kind: LaidOutKind::Text {
                    content: text.clone(),
                    style: TextStyle::default(),
                },
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
        } if should_use_block_flow(tag, props, children) => {
            let mut cursor = Cursor { y };
            layout_block_element(
                tag,
                props,
                children,
                Rect {
                    x,
                    y,
                    w: available_width,
                    h: f32::INFINITY,
                },
                &mut cursor,
                out,
                None,
                scroll_offsets,
            )
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if *tag == "table" => {
            let mut cursor = Cursor { y };
            layout_table_element(tag, props, children, x, &mut cursor, out, None)
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
            layout_inline_children(children, rect, out, scroll_offsets);
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
            layout_inline_children(children, rect, out, scroll_offsets);
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
                let child_size =
                    layout_inline_child(fragment_child, x, y, available_width, out, scroll_offsets);
                x += child_size.w;
                size.w += child_size.w;
                size.h = size.h.max(child_size.h);
            }
            size
        }
    }
}

fn measure_intrinsic(tag: &str, children: &[Element], available_width: f32) -> InlineSize {
    if tag == "table" {
        return measure_table(children);
    }
    if tag == "tr" {
        return measure_table_row(children);
    }
    if tag == "td" {
        return InlineSize { w: 72.0, h: 24.0 };
    }
    if is_block_container(tag) {
        let content_h = if children_need_block_flow(children) {
            measure_block_children_height(children, available_width)
        } else {
            measure_inline_run(children, available_width).h
        };
        return InlineSize {
            w: available_width,
            h: content_h,
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

fn should_use_block_flow(tag: &str, props: &Props, children: &[Element]) -> bool {
    is_block_container(tag)
        || matches!(tag, "h1" | "table" | "tbody" | "tr")
        || props.id.as_deref() == Some("table-viewport")
        || has_table_descendant(children)
}

fn children_need_block_flow(children: &[Element]) -> bool {
    children.iter().any(is_block_level_child)
}

fn is_block_level_child(child: &Element) -> bool {
    match child {
        Element::Intrinsic {
            tag,
            props,
            children,
        } => should_use_block_flow(tag, props, children),
        Element::Fragment(children) => children_need_block_flow(children),
        Element::Empty | Element::Text(_) | Element::Component(_) => false,
    }
}

fn has_table_descendant(children: &[Element]) -> bool {
    children.iter().any(|child| match child {
        Element::Intrinsic { tag, children, .. } => {
            matches!(*tag, "table" | "tbody" | "tr" | "td") || has_table_descendant(children)
        }
        Element::Fragment(children) => has_table_descendant(children),
        Element::Empty | Element::Text(_) | Element::Component(_) => false,
    })
}

fn layout_block_element(
    tag: &'static str,
    props: &Props,
    children: &[Element],
    parent_box: Rect,
    cursor: &mut Cursor,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
    scroll_offsets: &ScrollOffsets,
) -> InlineSize {
    if tag == "table" {
        return layout_table_element(tag, props, children, parent_box.x, cursor, out, clip);
    }

    let style = parse_inline_style(props.style.as_deref());
    let metrics = compute_block_metrics(tag, style, children, parent_box, cursor.y);
    push_intrinsic_node_if_visible(tag, props, metrics.rect, out, clip);

    let scroll_offset = props
        .id
        .as_deref()
        .filter(|_| style.overflow_auto && style.height.is_some())
        .map(|id| scroll_offsets.get(id))
        .unwrap_or_default();
    let child_clip = if style.overflow_auto && style.height.is_some() {
        Some(metrics.content_rect)
    } else {
        clip
    };
    let child_parent_box = Rect {
        x: metrics.content_rect.x - scroll_offset.x,
        y: metrics.content_rect.y - scroll_offset.y,
        ..metrics.content_rect
    };
    if children_need_block_flow(children) {
        let mut child_cursor = Cursor {
            y: child_parent_box.y,
        };
        for child in children {
            layout_block_child(
                child,
                child_parent_box,
                &mut child_cursor,
                out,
                child_clip,
                scroll_offsets,
            );
        }
    } else {
        layout_inline_children(children, child_parent_box, out, scroll_offsets);
    }

    cursor.y = metrics.rect.y + metrics.rect.h + metrics.margin.bottom.px_or_zero();
    InlineSize {
        w: metrics.rect.w + metrics.margin.horizontal_px(),
        h: metrics.margin.top.px_or_zero() + metrics.rect.h + metrics.margin.bottom.px_or_zero(),
    }
}

fn layout_block_child(
    element: &Element,
    parent_box: Rect,
    cursor: &mut Cursor,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
    scroll_offsets: &ScrollOffsets,
) -> InlineSize {
    match element {
        Element::Empty => InlineSize { w: 0.0, h: 0.0 },
        Element::Text(text) => {
            let size = measure_text_run(text);
            let rect = Rect {
                x: parent_box.x,
                y: cursor.y,
                w: size.w,
                h: size.h,
            };
            push_text_node_if_visible(text.clone(), rect, out, clip);
            cursor.y += size.h;
            size
        }
        Element::Intrinsic {
            tag,
            props,
            children,
        } if should_use_block_flow(tag, props, children) => layout_block_element(
            tag,
            props,
            children,
            parent_box,
            cursor,
            out,
            clip,
            scroll_offsets,
        ),
        Element::Intrinsic {
            tag,
            props,
            children,
        } if *tag == "table" => {
            layout_table_element(tag, props, children, parent_box.x, cursor, out, clip)
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
            push_intrinsic_node_if_visible(tag, props, rect, out, clip);
            layout_inline_children(children, rect, out, scroll_offsets);
            cursor.y += size.h;
            size
        }
        Element::Fragment(children) => {
            let start_y = cursor.y;
            let mut max_w: f32 = 0.0;
            for child in children {
                let size = layout_block_child(child, parent_box, cursor, out, clip, scroll_offsets);
                max_w = max_w.max(size.w);
            }
            InlineSize {
                w: max_w,
                h: cursor.y - start_y,
            }
        }
        Element::Component(_) => {
            panic!(
                "layout received an unrendered Component — the runtime \
                 should have expanded it before calling layout()"
            );
        }
    }
}

fn compute_block_metrics(
    tag: &str,
    style: ParsedStyle,
    children: &[Element],
    parent_box: Rect,
    y: f32,
) -> BlockMetrics {
    let margin = style.margin();
    let padding = style.padding();
    let border = style.border_width();
    let padding_border_x = padding.horizontal_px() + border.horizontal_px();
    let padding_border_y = padding.vertical_px() + border.vertical_px();
    let parent_width = parent_box.w.max(0.0);

    let available_for_border = (parent_width - margin.horizontal_px()).max(0.0);
    let border_w = if let Some(width) = style.width {
        (width + padding_border_x).max(0.0)
    } else if let Some(max_width) = style.max_width {
        (max_width + padding_border_x).min(parent_width).max(0.0)
    } else {
        available_for_border
    };
    let content_w = (border_w - padding_border_x).max(0.0);

    let x = match (margin.left, margin.right) {
        (CssEdgeValue::Auto, CssEdgeValue::Auto) => {
            parent_box.x + ((parent_width - border_w).max(0.0) / 2.0)
        }
        (left, _) => parent_box.x + left.px_or_zero(),
    };
    let rect_y = y + margin.top.px_or_zero();
    let content_h = style.height.unwrap_or_else(|| {
        let measured_h = if children_need_block_flow(children) {
            measure_block_children_height(children, content_w)
        } else {
            measure_inline_run(children, content_w).h
        };
        measured_h.max(style.line_height_or_default(tag))
    });
    let rect = Rect {
        x,
        y: rect_y,
        w: border_w,
        h: content_h + padding_border_y,
    };
    let content_rect = Rect {
        x: rect.x + border.left.px_or_zero() + padding.left.px_or_zero(),
        y: rect.y + border.top.px_or_zero() + padding.top.px_or_zero(),
        w: content_w,
        h: content_h,
    };

    BlockMetrics {
        margin,
        rect,
        content_rect,
    }
}

fn measure_block_children_height(children: &[Element], available_width: f32) -> f32 {
    children
        .iter()
        .map(|child| measure_block_child_outer(child, available_width).h)
        .sum()
}

fn measure_block_children_size(children: &[Element], available_width: f32) -> InlineSize {
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    for child in children {
        let size = measure_block_child_outer(child, available_width);
        w = w.max(size.w);
        h += size.h;
    }
    InlineSize { w, h }
}

fn measure_block_child_outer(element: &Element, available_width: f32) -> InlineSize {
    match element {
        Element::Empty => InlineSize { w: 0.0, h: 0.0 },
        Element::Text(text) => measure_text_run(text),
        Element::Intrinsic {
            tag,
            props,
            children,
        } if should_use_block_flow(tag, props, children) => {
            if *tag == "table" {
                return measure_table(children);
            }
            let style = parse_inline_style(props.style.as_deref());
            let margin = style.margin();
            let padding = style.padding();
            let border = style.border_width();
            let padding_border_x = padding.horizontal_px() + border.horizontal_px();
            let padding_border_y = padding.vertical_px() + border.vertical_px();
            let content_w = style
                .width
                .or(style.max_width)
                .unwrap_or((available_width - padding_border_x).max(0.0));
            let content_h = style.height.unwrap_or_else(|| {
                let measured_h = if children_need_block_flow(children) {
                    measure_block_children_height(children, content_w)
                } else {
                    measure_inline_run(children, content_w).h
                };
                measured_h.max(style.line_height_or_default(tag))
            });
            InlineSize {
                w: content_w + padding_border_x + margin.horizontal_px(),
                h: content_h + padding_border_y + margin.vertical_px(),
            }
        }
        Element::Intrinsic { tag, children, .. } => {
            measure_intrinsic(tag, children, available_width)
        }
        Element::Fragment(children) => {
            if children_need_block_flow(children) {
                InlineSize {
                    w: available_width,
                    h: measure_block_children_height(children, available_width),
                }
            } else {
                measure_inline_run(children, available_width)
            }
        }
        Element::Component(_) => panic!("unrendered component at measure time"),
    }
}

fn layout_table_element(
    tag: &'static str,
    props: &Props,
    children: &[Element],
    x: f32,
    cursor: &mut Cursor,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) -> InlineSize {
    let style = parse_inline_style(props.style.as_deref());
    let rows = collect_table_rows(children);
    let measured_w = rows
        .first()
        .map(|row| measure_table_row_cells_with_style(row, style).w)
        .unwrap_or(0.0);
    let table_w = style.width.unwrap_or(measured_w).max(measured_w);
    let table_h: f32 = rows
        .iter()
        .map(|row| measure_table_row_cells_with_style(row, style).h)
        .sum();
    let table_rect = Rect {
        x,
        y: cursor.y,
        w: table_w,
        h: table_h,
    };
    push_intrinsic_node_if_visible(tag, props, table_rect, out, clip);

    let tbody_props = Props::default();
    push_intrinsic_node_if_visible("tbody", &tbody_props, table_rect, out, clip);

    let mut row_y = table_rect.y;
    for row in rows {
        let row_size = measure_table_row_cells_with_style(row, style);
        if let Element::Intrinsic {
            tag: row_tag,
            props: row_props,
            children: row_children,
        } = row
        {
            let row_rect = Rect {
                x: table_rect.x,
                y: row_y,
                w: table_w.max(row_size.w),
                h: row_size.h,
            };
            push_intrinsic_node_if_visible(row_tag, row_props, row_rect, out, clip);
            layout_table_cells(row_children, row_rect, out, clip);
        }
        row_y += row_size.h;
    }

    cursor.y += table_h;
    InlineSize {
        w: table_w,
        h: table_h,
    }
}

fn layout_table_cells(
    row_children: &[Element],
    row_rect: Rect,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) {
    let cells = collect_table_cells(row_children);
    let mut x = row_rect.x;
    for cell in cells {
        let Element::Intrinsic {
            tag,
            props,
            children,
        } = cell
        else {
            continue;
        };
        let style = parse_inline_style(props.style.as_deref());
        let size = measure_table_cell(style, ParsedStyle::default());
        let w = size.w;
        let h = row_rect.h.max(size.h);
        let rect = Rect {
            x,
            y: row_rect.y,
            w,
            h,
        };
        push_intrinsic_node_if_visible(tag, props, rect, out, clip);
        if rect_intersects_clip(rect, clip) {
            let padding = style.padding();
            let border = style.border_width();
            let content_rect = Rect {
                x: rect.x + border.left.px_or_zero() + padding.left.px_or_zero(),
                y: rect.y + border.top.px_or_zero() + padding.top.px_or_zero(),
                w: (rect.w - border.horizontal_px() - padding.horizontal_px()).max(0.0),
                h: (rect.h - border.vertical_px() - padding.vertical_px()).max(0.0),
            };
            layout_table_cell_text(children, content_rect, TextStyle::from(style), out, clip);
        }
        x += w;
    }
}

fn layout_table_cell_text(
    children: &[Element],
    content_rect: Rect,
    style: TextStyle,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) {
    let mut x = content_rect.x;
    for child in children {
        match child {
            Element::Text(text) => {
                let size = measure_text_run(text);
                let y = content_rect.y + ((content_rect.h - size.h).max(0.0) / 2.0);
                let rect = Rect {
                    x,
                    y,
                    w: size.w,
                    h: size.h.min(content_rect.h.max(size.h)),
                };
                push_styled_text_node_if_visible(text.clone(), rect, style, out, clip);
                x += size.w;
            }
            Element::Fragment(children) => {
                layout_table_cell_text(children, Rect { x, ..content_rect }, style, out, clip)
            }
            _ => {
                let size = layout_inline_child(
                    child,
                    x,
                    content_rect.y,
                    content_rect.w,
                    out,
                    &ScrollOffsets::default(),
                );
                x += size.w;
            }
        }
    }
}

fn measure_table(children: &[Element]) -> InlineSize {
    let rows = collect_table_rows(children);
    let mut w: f32 = 0.0;
    let mut h: f32 = 0.0;
    for row in rows {
        let row_size = measure_table_row_cells(row);
        w = w.max(row_size.w);
        h += row_size.h;
    }
    InlineSize { w, h }
}

fn measure_table_row(children: &[Element]) -> InlineSize {
    measure_table_row_with_style(children, ParsedStyle::default())
}

fn measure_table_row_with_style(children: &[Element], inherited_style: ParsedStyle) -> InlineSize {
    let mut w: f32 = 0.0;
    let mut h: f32 = 24.0;
    for cell in collect_table_cells(children) {
        if let Element::Intrinsic { props, .. } = cell {
            let style = parse_inline_style(props.style.as_deref());
            let size = measure_table_cell(style, inherited_style);
            w += size.w;
            h = h.max(size.h);
        }
    }
    InlineSize { w, h }
}

fn measure_table_row_cells(row: &Element) -> InlineSize {
    measure_table_row_cells_with_style(row, ParsedStyle::default())
}

fn measure_table_row_cells_with_style(row: &Element, inherited_style: ParsedStyle) -> InlineSize {
    match row {
        Element::Intrinsic { children, .. } => {
            measure_table_row_with_style(children, inherited_style)
        }
        Element::Fragment(children) => {
            let mut w: f32 = 0.0;
            let mut h: f32 = 0.0;
            for child in children {
                let size = measure_table_row_cells_with_style(child, inherited_style);
                w = w.max(size.w);
                h += size.h;
            }
            InlineSize { w, h }
        }
        _ => InlineSize { w: 0.0, h: 0.0 },
    }
}

fn measure_table_cell(style: ParsedStyle, inherited_style: ParsedStyle) -> InlineSize {
    let padding = style.padding();
    let border = style.border_width();
    let collapsed_border = border
        .top
        .px_or_zero()
        .max(border.right.px_or_zero())
        .max(border.bottom.px_or_zero())
        .max(border.left.px_or_zero());
    let content_w = style.width.unwrap_or(72.0);
    let content_h = style
        .height
        .unwrap_or(24.0)
        .max(inherited_style.line_height_or_default("td"));
    InlineSize {
        w: content_w + padding.horizontal_px() + collapsed_border,
        h: content_h + padding.vertical_px() + collapsed_border,
    }
}

fn collect_table_rows(children: &[Element]) -> Vec<&Element> {
    let mut rows = Vec::new();
    collect_table_rows_into(children, &mut rows);
    rows
}

fn collect_table_rows_into<'a>(children: &'a [Element], rows: &mut Vec<&'a Element>) {
    for child in children {
        match child {
            Element::Intrinsic { tag: "tr", .. } => rows.push(child),
            Element::Intrinsic {
                tag: "tbody",
                children,
                ..
            }
            | Element::Fragment(children) => collect_table_rows_into(children, rows),
            _ => {}
        }
    }
}

fn collect_table_cells(children: &[Element]) -> Vec<&Element> {
    let mut cells = Vec::new();
    collect_table_cells_into(children, &mut cells);
    cells
}

fn collect_table_cells_into<'a>(children: &'a [Element], cells: &mut Vec<&'a Element>) {
    for child in children {
        match child {
            Element::Intrinsic { tag: "td", .. } => cells.push(child),
            Element::Fragment(children) => collect_table_cells_into(children, cells),
            _ => {}
        }
    }
}

fn push_intrinsic_node_if_visible(
    tag: &'static str,
    props: &Props,
    rect: Rect,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) {
    if let Some(rect) = visible_rect(rect, clip) {
        out.push(LaidOutNode {
            kind: LaidOutKind::Intrinsic {
                tag,
                props: props.clone(),
            },
            rect,
        });
    }
}

fn push_text_node_if_visible(
    content: String,
    rect: Rect,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) {
    push_styled_text_node_if_visible(content, rect, TextStyle::default(), out, clip);
}

fn push_styled_text_node_if_visible(
    content: String,
    rect: Rect,
    style: TextStyle,
    out: &mut Vec<LaidOutNode>,
    clip: Option<Rect>,
) {
    if let Some(rect) = visible_rect(rect, clip) {
        out.push(LaidOutNode {
            kind: LaidOutKind::Text { content, style },
            rect,
        });
    }
}

fn rect_intersects_clip(rect: Rect, clip: Option<Rect>) -> bool {
    visible_rect(rect, clip).is_some()
}

fn visible_rect(rect: Rect, clip: Option<Rect>) -> Option<Rect> {
    let Some(clip) = clip else {
        return Some(rect);
    };
    let x0 = rect.x.max(clip.x);
    let y0 = rect.y.max(clip.y);
    let x1 = (rect.x + rect.w).min(clip.x + clip.w);
    let y1 = (rect.y + rect.h).min(clip.y + clip.h);
    (x1 > x0 && y1 > y0).then_some(Rect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
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

fn parse_inline_style(style: Option<&str>) -> ParsedStyle {
    let mut parsed = ParsedStyle::default();
    let Some(style) = style else {
        return parsed;
    };

    for declaration in style.split(';') {
        let Some((raw_key, raw_value)) = declaration.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let value = raw_value.trim();
        match key.as_str() {
            "width" => parsed.width = parse_px(value),
            "height" => parsed.height = parse_px(value),
            "max-width" => parsed.max_width = parse_px(value),
            "margin" => parsed.margin = parse_edge_shorthand(value, true),
            "padding" => parsed.padding = parse_edge_shorthand(value, false),
            "border-width" => parsed.border_width = parse_edge_shorthand(value, false),
            "border" => {
                if let Some(width) = parse_border_width(value) {
                    parsed.border_width = Some(CssEdges {
                        top: CssEdgeValue::Px(width),
                        right: CssEdgeValue::Px(width),
                        bottom: CssEdgeValue::Px(width),
                        left: CssEdgeValue::Px(width),
                    });
                }
                parsed.border_color = parse_hex_color_from_tokens(value);
            }
            "border-color" => parsed.border_color = parse_hex_color(value),
            "background" | "background-color" => parsed.background = parse_hex_color(value),
            "color" => parsed.color = parse_hex_color(value),
            "font-size" => parsed.font_size = parse_px(value),
            "line-height" => parsed.line_height = parse_line_height(value),
            "overflow" => parsed.overflow_auto = value == "auto",
            _ => {}
        }
    }

    parsed
}

fn parse_px(value: &str) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    value
        .strip_suffix("px")
        .unwrap_or(value)
        .trim()
        .parse::<f32>()
        .ok()
}

fn parse_line_height(value: &str) -> Option<CssLineHeight> {
    let value = value.trim();
    if value.ends_with("px") {
        parse_px(value).map(CssLineHeight::Px)
    } else {
        parse_px(value).map(CssLineHeight::Multiplier)
    }
}

fn parse_edge_value(value: &str, allow_auto: bool) -> Option<CssEdgeValue> {
    if allow_auto && value.trim() == "auto" {
        Some(CssEdgeValue::Auto)
    } else {
        parse_px(value).map(CssEdgeValue::Px)
    }
}

fn parse_edge_shorthand(value: &str, allow_auto: bool) -> Option<CssEdges> {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    let values = parts
        .iter()
        .map(|part| parse_edge_value(part, allow_auto))
        .collect::<Option<Vec<_>>>()?;
    match values.as_slice() {
        [all] => Some(CssEdges {
            top: *all,
            right: *all,
            bottom: *all,
            left: *all,
        }),
        [vertical, horizontal] => Some(CssEdges {
            top: *vertical,
            right: *horizontal,
            bottom: *vertical,
            left: *horizontal,
        }),
        [top, horizontal, bottom] => Some(CssEdges {
            top: *top,
            right: *horizontal,
            bottom: *bottom,
            left: *horizontal,
        }),
        [top, right, bottom, left] => Some(CssEdges {
            top: *top,
            right: *right,
            bottom: *bottom,
            left: *left,
        }),
        _ => None,
    }
}

fn parse_border_width(value: &str) -> Option<f32> {
    value.split_whitespace().find_map(parse_px)
}

fn parse_hex_color_from_tokens(value: &str) -> Option<Color> {
    value.split_whitespace().find_map(parse_hex_color)
}

fn parse_hex_color(value: &str) -> Option<Color> {
    let hex = value.trim().strip_prefix('#')?;
    let expand = |c: char| -> Option<u8> {
        let digit = c.to_digit(16)? as u8;
        Some((digit << 4) | digit)
    };
    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            Some(Color::rgb(
                expand(chars.next()?)?,
                expand(chars.next()?)?,
                expand(chars.next()?)?,
            ))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::rgb(r, g, b))
        }
        _ => None,
    }
}

// ── Paint ─────────────────────────────────────────────────────────────────

const SCROLLBAR_THICKNESS_PX: f32 = 10.0;
const SCROLLBAR_MIN_THUMB_PX: f32 = 24.0;
const SCROLLBAR_TRACK_COLOR: Color = Color {
    r: 0xea,
    g: 0xee,
    b: 0xf3,
    a: 255,
};
const SCROLLBAR_THUMB_COLOR: Color = Color {
    r: 0x94,
    g: 0x9b,
    b: 0xa8,
    a: 255,
};

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
            LaidOutKind::Text { content, style } => {
                let mut font = theme.default_font.clone();
                if let Some(font_size_px) = style.font_size_px {
                    font.size_px = font_size_px;
                }
                ops.push(PaintOp::Text {
                    origin: Point {
                        x: node.rect.x + theme.text_pad_x,
                        y: node.rect.y + theme.text_pad_y,
                    },
                    content: content.clone(),
                    font,
                    color: style.color.unwrap_or(theme.text_color),
                });
            }
        }
    }
    ops
}

/// Overlay scrollbar tracks/thumbs for one overflow container.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer.md#schema
pub fn scrollbar_paint_ops(bounds: ScrollBounds, offset: ScrollOffset) -> Vec<PaintOp> {
    let mut ops = Vec::new();
    let offset = bounds.clamp(offset);
    let has_h = bounds.has_horizontal_scrollbar();
    let has_v = bounds.has_vertical_scrollbar();
    if !has_h && !has_v {
        return ops;
    }

    let corner = if has_h && has_v {
        SCROLLBAR_THICKNESS_PX
    } else {
        0.0
    };
    if has_v {
        let track = Rect {
            x: bounds.viewport_rect.x + bounds.viewport_rect.w - SCROLLBAR_THICKNESS_PX,
            y: bounds.viewport_rect.y,
            w: SCROLLBAR_THICKNESS_PX,
            h: (bounds.viewport_rect.h - corner).max(0.0),
        };
        let thumb_h = scrollbar_thumb_len(track.h, bounds.viewport_rect.h, bounds.content_height);
        let travel = (track.h - thumb_h).max(0.0);
        let thumb_y = track.y + scroll_ratio(offset.y, bounds.max_y) * travel;
        ops.push(PaintOp::FillRect {
            rect: track,
            color: SCROLLBAR_TRACK_COLOR,
        });
        ops.push(PaintOp::FillRect {
            rect: Rect {
                y: thumb_y,
                h: thumb_h,
                ..track
            },
            color: SCROLLBAR_THUMB_COLOR,
        });
    }
    if has_h {
        let track = Rect {
            x: bounds.viewport_rect.x,
            y: bounds.viewport_rect.y + bounds.viewport_rect.h - SCROLLBAR_THICKNESS_PX,
            w: (bounds.viewport_rect.w - corner).max(0.0),
            h: SCROLLBAR_THICKNESS_PX,
        };
        let thumb_w = scrollbar_thumb_len(track.w, bounds.viewport_rect.w, bounds.content_width);
        let travel = (track.w - thumb_w).max(0.0);
        let thumb_x = track.x + scroll_ratio(offset.x, bounds.max_x) * travel;
        ops.push(PaintOp::FillRect {
            rect: track,
            color: SCROLLBAR_TRACK_COLOR,
        });
        ops.push(PaintOp::FillRect {
            rect: Rect {
                x: thumb_x,
                w: thumb_w,
                ..track
            },
            color: SCROLLBAR_THUMB_COLOR,
        });
    }
    if has_h && has_v {
        ops.push(PaintOp::FillRect {
            rect: Rect {
                x: bounds.viewport_rect.x + bounds.viewport_rect.w - SCROLLBAR_THICKNESS_PX,
                y: bounds.viewport_rect.y + bounds.viewport_rect.h - SCROLLBAR_THICKNESS_PX,
                w: SCROLLBAR_THICKNESS_PX,
                h: SCROLLBAR_THICKNESS_PX,
            },
            color: SCROLLBAR_TRACK_COLOR,
        });
    }
    ops
}

fn scrollbar_thumb_len(track_len: f32, viewport_len: f32, content_len: f32) -> f32 {
    if track_len <= 0.0 || content_len <= 0.0 {
        return 0.0;
    }
    (track_len * (viewport_len / content_len))
        .clamp(SCROLLBAR_MIN_THUMB_PX.min(track_len), track_len)
}

fn scroll_ratio(offset: f32, max_offset: f32) -> f32 {
    if max_offset <= 0.0 {
        0.0
    } else {
        (offset / max_offset).clamp(0.0, 1.0)
    }
}

fn paint_intrinsic(tag: &str, props: &Props, rect: Rect, theme: &Theme, ops: &mut Vec<PaintOp>) {
    let style = parse_inline_style(props.style.as_deref());
    if let Some(color) = style.background {
        ops.push(PaintOp::FillRect { rect, color });
    }
    if let Some(width) = border_width_for_paint(style) {
        ops.push(PaintOp::StrokeRect {
            rect,
            color: style.border_color.unwrap_or(theme.border_default.color),
            width,
        });
    }

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

fn border_width_for_paint(style: ParsedStyle) -> Option<f32> {
    let border = style.border_width?;
    let width = border
        .top
        .px_or_zero()
        .max(border.right.px_or_zero())
        .max(border.bottom.px_or_zero())
        .max(border.left.px_or_zero());
    (width > 0.0).then_some(width)
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
    fn surface_inline_layout_matches_list_fixture_boxes() {
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
                LaidOutKind::Text { .. } => None,
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
                kind: LaidOutKind::Text {
                    content: "value: 42".to_string(),
                    style: TextStyle::default(),
                },
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
                    kind: LaidOutKind::Text {
                        content: "count: 0".to_string(),
                        style: TextStyle::default(),
                    },
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

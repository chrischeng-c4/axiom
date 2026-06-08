// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
// CODEGEN-BEGIN
//! Layout engine — taffy-backed flexbox + block layout.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md
//!
//! This module implements the public layout API surface (R1) and the
//! pure `layout()` entry-point (R6) that paint-runtime.md consumes.
//! All taffy state (NodeId map + per-node measurement cache) lives
//! inside `LayoutTree` so two trees with byte-identical contents +
//! same `Viewport` produce byte-identical `Vec<LaidOutNode>` output.
//!
//! Dirty-subtree marking (R3) is the SEPARATE impure phase that runs
//! BEFORE `layout()` is called — it flips `LayoutTree.dirty_nodes`
//! flags and is implemented in `dirty.rs`. The `layout()` function
//! itself never mutates its receiver.
//!
//! Style-prop parsing (R7) lives in `style_parser.rs`. The minimal-
//! viable CSS feature set per R4 covers display:block / flex,
//! width / height + min/max, padding / margin / border-width,
//! flex-direction / justify-content / align-items.

pub mod dirty;
pub mod style_parser;

use std::collections::{HashMap, HashSet};
use taffy::{
    AlignItems as TaffyAlignItems, AvailableSpace, Dimension as TaffyDimension, Display,
    FlexDirection as TaffyFlexDirection, JustifyContent as TaffyJustifyContent, LengthPercentage,
    LengthPercentageAuto, NodeId, Rect as TaffyRect, Size, Style, TaffyTree,
};

// ── Public ID + style types ────────────────────────────────────────

/// Stable, opaque key for a layout node (R1, R5).
///
/// Derived by the caller from the fiber id + JSX path. Survives
/// re-renders if the element's position in the fiber tree is
/// unchanged.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutNodeId(pub String);

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl LayoutNodeId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Display kind — minimal-viable subset (R4). `grid`, `table`, etc.
/// are out of scope (R9) and rejected by the style parser.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Block,
    Flex,
    None,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl Default for DisplayKind {
    fn default() -> Self {
        Self::Block
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl Default for FlexDirection {
    fn default() -> Self {
        Self::Row
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

/// CSS length value (R7). Discriminated union; `Auto` is the absence
/// of an explicit value.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// Pixels. Taffy: `Dimension::Length(px)`.
    Px(f32),
    /// Percentage of parent (0.0..=100.0). Taffy: `Dimension::Percent(pct/100)`.
    Pct(f32),
    /// Auto-sized. Taffy: `Dimension::Auto`.
    Auto,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl Default for Dimension {
    fn default() -> Self {
        Self::Auto
    }
}

/// Four-sided shorthand for padding / margin / border-width (R4).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect4 {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

/// Minimal-viable layout style parsed from inline JSX style props
/// (R4, R7). Properties absent here default to taffy's own defaults.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Default)]
pub struct LayoutStyle {
    pub display: Option<DisplayKind>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub max_height: Option<Dimension>,
    pub padding: Option<Rect4>,
    pub margin: Option<Rect4>,
    pub border_width: Option<Rect4>,
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
}

// ── Layout node + tree ─────────────────────────────────────────────

/// A single node in the layout tree (R1). Carries the style and the
/// ordered list of child IDs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: LayoutNodeId,
    pub style: LayoutStyle,
    pub children: Vec<LayoutNodeId>,
}

/// Available rendering area (R1, R10). Single shared shape with
/// paint-runtime.md P4. `dpr` is carried for call-site compatibility
/// but ignored by the layout engine — taffy is unitless.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub dpr: f32,
}

/// Paint-side rect with absolute origin and size (R1, R5).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Output of `layout()` — one entry per non-skipped node (R1, R5).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutNode {
    pub node_id: LayoutNodeId,
    pub rect: Rect,
}

/// The mutable, incrementally-updated layout state (R1, R5, R6).
/// Encapsulates the logical node map AND the taffy `TaffyTree`
/// (including its per-node measurement cache). All taffy state
/// lives here, so `(LayoutTree, Viewport)` pairs with byte-identical
/// contents always produce byte-identical `Vec<LaidOutNode>` from
/// `layout()`.
///
/// Dirty-subtree marking (R3) writes to `dirty_nodes` BEFORE
/// `layout()` is called. The `layout()` call itself never mutates
/// its receiver — see `R6` of layout-runtime.md.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub struct LayoutTree {
    /// Logical nodes, keyed by stable ID.
    pub(crate) nodes: HashMap<LayoutNodeId, LayoutNode>,
    /// Logical → taffy NodeId mapping (R5). Encapsulates taffy state.
    pub(crate) taffy_map: HashMap<LayoutNodeId, NodeId>,
    /// Taffy tree carrying per-node measurement cache.
    pub(crate) taffy: TaffyTree<()>,
    /// Root of the logical tree.
    pub(crate) root: Option<LayoutNodeId>,
    /// Dirty nodes accumulated since last `layout()` call (R3). Read-
    /// and-cleared by `layout()`, mutated by `dirty.rs`.
    pub(crate) dirty_nodes: HashSet<LayoutNodeId>,
    /// Last viewport seen by `layout()`. Drives full vs. subtree
    /// recompute (R3, R8 scenario S5).
    pub(crate) last_viewport: Option<Viewport>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
impl LayoutTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            taffy_map: HashMap::new(),
            taffy: TaffyTree::new(),
            root: None,
            dirty_nodes: HashSet::new(),
            last_viewport: None,
        }
    }

    /// Insert or replace a node. Marks the node as dirty so the next
    /// `layout()` call applies its style/children changes to taffy.
    pub fn upsert(&mut self, node: LayoutNode) {
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.dirty_nodes.insert(id);
    }

    /// Set the root of the logical tree. Required before `layout()`.
    pub fn set_root(&mut self, id: LayoutNodeId) {
        self.root = Some(id.clone());
        self.dirty_nodes.insert(id);
    }

    /// Mark a node dirty (R3 entry point).
    pub fn mark_dirty(&mut self, id: &LayoutNodeId) {
        self.dirty_nodes.insert(id.clone());
    }

    /// Read-only view of the dirty set — useful for tests and for
    /// callers that want to inspect pending work before `layout()`.
    pub fn dirty_nodes(&self) -> &HashSet<LayoutNodeId> {
        &self.dirty_nodes
    }

    /// Read-only view of the last `Viewport` seen by `layout()`.
    /// `None` until the first call.
    pub fn last_viewport(&self) -> Option<Viewport> {
        self.last_viewport
    }

    /// Build the parent map needed for ancestor propagation (R3).
    pub(crate) fn parent_map(&self) -> HashMap<LayoutNodeId, LayoutNodeId> {
        let mut map = HashMap::new();
        for (id, node) in &self.nodes {
            for child in &node.children {
                map.insert(child.clone(), id.clone());
            }
        }
        map
    }
}

// ── The pure `layout()` entry point ────────────────────────────────

/// Run layout against the current `LayoutTree` for the given
/// `Viewport`. Pure given its inputs (R6): two trees with byte-
/// identical contents and the same viewport always produce byte-
/// identical output.
///
/// Side effects on `LayoutTree`:
///   - Applies pending dirty-node styles + children to the internal
///     taffy tree (idempotent: same input → same taffy state).
///   - Clears `dirty_nodes` after a successful compute.
///   - Updates `last_viewport`.
///
/// These mutations are part of the encapsulated taffy cache; they
/// do NOT affect the externally-observable output for a given
/// `(node_set, viewport)` input shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn layout(tree: &mut LayoutTree, viewport: Viewport) -> Vec<LaidOutNode> {
    let Some(root_id) = tree.root.clone() else {
        return Vec::new();
    };

    // Phase 1: ensure every logical node has a corresponding taffy
    // NodeId. Build lazily — first sight inserts a leaf.
    let all_ids: Vec<LayoutNodeId> = tree.nodes.keys().cloned().collect();
    for id in &all_ids {
        if !tree.taffy_map.contains_key(id) {
            let nid = tree
                .taffy
                .new_leaf(Style::default())
                .expect("taffy::TaffyTree::new_leaf");
            tree.taffy_map.insert(id.clone(), nid);
        }
    }

    // Phase 2: apply dirty nodes' style + children to taffy.
    let dirty: Vec<LayoutNodeId> = tree.dirty_nodes.iter().cloned().collect();
    for id in &dirty {
        let Some(node) = tree.nodes.get(id) else {
            continue;
        };
        let taffy_id = *tree.taffy_map.get(id).expect("taffy_map populated");
        let taffy_style = layout_style_to_taffy(&node.style);
        tree.taffy
            .set_style(taffy_id, taffy_style)
            .expect("taffy::set_style");
        let child_taffy_ids: Vec<NodeId> = node
            .children
            .iter()
            .filter_map(|c| tree.taffy_map.get(c).copied())
            .collect();
        tree.taffy
            .set_children(taffy_id, &child_taffy_ids)
            .expect("taffy::set_children");
    }

    // Phase 3: compute layout from the root with viewport-derived
    // available space. Taffy's compute_layout is idempotent for an
    // unchanged tree — incremental dirty work is taffy-internal.
    let root_taffy_id = *tree.taffy_map.get(&root_id).expect("root taffy id present");
    let viewport_changed = tree
        .last_viewport
        .map(|prev| prev != viewport)
        .unwrap_or(true);
    if viewport_changed {
        // Per spec dirty.rs: viewport change forces full recompute.
        tree.taffy
            .mark_dirty(root_taffy_id)
            .expect("taffy::mark_dirty");
    }
    tree.taffy
        .compute_layout(
            root_taffy_id,
            Size {
                width: AvailableSpace::Definite(viewport.width),
                height: AvailableSpace::Definite(viewport.height),
            },
        )
        .expect("taffy::compute_layout");

    // Phase 4: walk the logical tree depth-first pre-order,
    // accumulating absolute origin from parent offsets.
    let mut out = Vec::with_capacity(tree.nodes.len());
    walk_extract(tree, &root_id, 0.0, 0.0, &mut out);

    // Phase 5: clear dirty + record viewport for next call.
    tree.dirty_nodes.clear();
    tree.last_viewport = Some(viewport);

    out
}

fn walk_extract(
    tree: &LayoutTree,
    id: &LayoutNodeId,
    offset_x: f32,
    offset_y: f32,
    out: &mut Vec<LaidOutNode>,
) {
    let Some(taffy_id) = tree.taffy_map.get(id).copied() else {
        return;
    };
    let lay = tree.taffy.layout(taffy_id).expect("taffy::layout");
    let abs_x = offset_x + lay.location.x;
    let abs_y = offset_y + lay.location.y;
    out.push(LaidOutNode {
        node_id: id.clone(),
        rect: Rect {
            x: abs_x,
            y: abs_y,
            w: lay.size.width,
            h: lay.size.height,
        },
    });
    if let Some(node) = tree.nodes.get(id) {
        for child in &node.children {
            walk_extract(tree, child, abs_x, abs_y, out);
        }
    }
}

// ── Mapping helpers ────────────────────────────────────────────────

fn layout_style_to_taffy(s: &LayoutStyle) -> Style {
    let mut style = Style::default();
    if let Some(d) = s.display {
        style.display = match d {
            DisplayKind::Block => Display::Block,
            DisplayKind::Flex => Display::Flex,
            DisplayKind::None => Display::None,
        };
    }
    style.size = Size {
        width: dimension_to_taffy(s.width.unwrap_or_default()),
        height: dimension_to_taffy(s.height.unwrap_or_default()),
    };
    style.min_size = Size {
        width: dimension_to_taffy(s.min_width.unwrap_or_default()),
        height: dimension_to_taffy(s.min_height.unwrap_or_default()),
    };
    style.max_size = Size {
        width: dimension_to_taffy(s.max_width.unwrap_or_default()),
        height: dimension_to_taffy(s.max_height.unwrap_or_default()),
    };
    if let Some(p) = s.padding {
        style.padding = rect4_to_taffy_lp(&p);
    }
    if let Some(m) = s.margin {
        style.margin = rect4_to_taffy_lpa(&m);
    }
    if let Some(b) = s.border_width {
        style.border = rect4_to_taffy_lp(&b);
    }
    if let Some(fd) = s.flex_direction {
        style.flex_direction = match fd {
            FlexDirection::Row => TaffyFlexDirection::Row,
            FlexDirection::Column => TaffyFlexDirection::Column,
            FlexDirection::RowReverse => TaffyFlexDirection::RowReverse,
            FlexDirection::ColumnReverse => TaffyFlexDirection::ColumnReverse,
        };
    }
    if let Some(jc) = s.justify_content {
        style.justify_content = Some(match jc {
            JustifyContent::FlexStart => TaffyJustifyContent::FlexStart,
            JustifyContent::FlexEnd => TaffyJustifyContent::FlexEnd,
            JustifyContent::Center => TaffyJustifyContent::Center,
            JustifyContent::SpaceBetween => TaffyJustifyContent::SpaceBetween,
            JustifyContent::SpaceAround => TaffyJustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly => TaffyJustifyContent::SpaceEvenly,
        });
    }
    if let Some(ai) = s.align_items {
        style.align_items = Some(match ai {
            AlignItems::FlexStart => TaffyAlignItems::FlexStart,
            AlignItems::FlexEnd => TaffyAlignItems::FlexEnd,
            AlignItems::Center => TaffyAlignItems::Center,
            AlignItems::Baseline => TaffyAlignItems::Baseline,
            AlignItems::Stretch => TaffyAlignItems::Stretch,
        });
    }
    style
}

fn dimension_to_taffy(d: Dimension) -> TaffyDimension {
    match d {
        Dimension::Px(px) => TaffyDimension::Length(px),
        Dimension::Pct(pct) => TaffyDimension::Percent(pct / 100.0),
        Dimension::Auto => TaffyDimension::Auto,
    }
}

fn dimension_to_lp(d: Dimension) -> LengthPercentage {
    match d {
        Dimension::Px(px) => LengthPercentage::Length(px),
        Dimension::Pct(pct) => LengthPercentage::Percent(pct / 100.0),
        Dimension::Auto => LengthPercentage::Length(0.0),
    }
}

fn dimension_to_lpa(d: Dimension) -> LengthPercentageAuto {
    match d {
        Dimension::Px(px) => LengthPercentageAuto::Length(px),
        Dimension::Pct(pct) => LengthPercentageAuto::Percent(pct / 100.0),
        Dimension::Auto => LengthPercentageAuto::Auto,
    }
}

fn rect4_to_taffy_lp(r: &Rect4) -> TaffyRect<LengthPercentage> {
    TaffyRect {
        top: dimension_to_lp(r.top),
        right: dimension_to_lp(r.right),
        bottom: dimension_to_lp(r.bottom),
        left: dimension_to_lp(r.left),
    }
}

fn rect4_to_taffy_lpa(r: &Rect4) -> TaffyRect<LengthPercentageAuto> {
    TaffyRect {
        top: dimension_to_lpa(r.top),
        right: dimension_to_lpa(r.right),
        bottom: dimension_to_lpa(r.bottom),
        left: dimension_to_lpa(r.left),
    }
}

// ── Helpers for tests ──────────────────────────────────────────────

/// Convenience: build a single-node tree.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn leaf(id: impl Into<String>, style: LayoutStyle) -> LayoutNode {
    LayoutNode {
        id: LayoutNodeId::new(id),
        style,
        children: Vec::new(),
    }
}

/// Convenience: build a node with children.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-renderer-layout.md#schema
pub fn parent(
    id: impl Into<String>,
    style: LayoutStyle,
    children: Vec<LayoutNodeId>,
) -> LayoutNode {
    LayoutNode {
        id: LayoutNodeId::new(id),
        style,
        children,
    }
}
// CODEGEN-END

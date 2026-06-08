// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Layout-engine S4 + S5: dirty-mark on a single node triggers only
//! the subtree re-layout; viewport resize triggers a full re-layout.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md#scenarios
//!
//! L0 pure-Rust tier.

use jet_wasm::renderer::layout::{
    dirty::propagate_ancestors, layout, leaf, parent, Dimension, DisplayKind, FlexDirection,
    LayoutNodeId, LayoutStyle, LayoutTree, Viewport,
};

fn vp(w: f32, h: f32) -> Viewport {
    Viewport {
        width: w,
        height: h,
        dpr: 1.0,
    }
}

fn build_three_row_tree() -> LayoutTree {
    let mut tree = LayoutTree::new();
    // Root deliberately wider than 3*200 = 600 so the dirty-mark
    // test (which grows b from 200 → 300) does not trigger taffy's
    // default flex-shrink. flex-shrink as a parsed property is
    // out of scope per R4/R9.
    tree.upsert(parent(
        "root",
        LayoutStyle {
            display: Some(DisplayKind::Flex),
            flex_direction: Some(FlexDirection::Row),
            width: Some(Dimension::Px(900.0)),
            height: Some(Dimension::Px(100.0)),
            ..Default::default()
        },
        vec![
            LayoutNodeId::new("a"),
            LayoutNodeId::new("b"),
            LayoutNodeId::new("c"),
        ],
    ));
    for id in ["a", "b", "c"] {
        tree.upsert(leaf(
            id,
            LayoutStyle {
                width: Some(Dimension::Px(200.0)),
                height: Some(Dimension::Px(100.0)),
                ..Default::default()
            },
        ));
    }
    tree.set_root(LayoutNodeId::new("root"));
    tree
}

#[test]
fn s4_dirty_single_node_recomputes_consistently() {
    let mut tree = build_three_row_tree();
    let viewport = vp(900.0, 100.0);

    // First call: full build.
    let out1 = layout(&mut tree, viewport);
    let b1 = out1
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("b"))
        .unwrap()
        .rect;

    // Mutate `b`'s width then mark it dirty (R3 entry point).
    tree.upsert(leaf(
        "b",
        LayoutStyle {
            width: Some(Dimension::Px(300.0)),
            height: Some(Dimension::Px(100.0)),
            ..Default::default()
        },
    ));
    // Even though only `b` is in dirty_nodes, taffy's compute_layout
    // re-flexes the row and `c` shifts. Propagating ancestors ensures
    // the row's set_children is applied (idempotent here, but the
    // contract still requires it).
    propagate_ancestors(&mut tree);

    let out2 = layout(&mut tree, viewport);
    let b2 = out2
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("b"))
        .unwrap()
        .rect;
    let c2 = out2
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("c"))
        .unwrap()
        .rect;

    assert_eq!(b1.w, 200.0);
    assert_eq!(b2.w, 300.0, "dirty node's new width is reflected");
    assert_eq!(
        c2.x, 500.0,
        "row reflowed: a(0..200) + b(200..500) → c at 500"
    );
    // dirty_nodes is cleared after each layout() call.
    assert!(tree.dirty_nodes_is_empty());
}

#[test]
fn s5_viewport_resize_triggers_full_relayout() {
    let mut tree = build_three_row_tree();

    let out1 = layout(&mut tree, vp(900.0, 100.0));
    let c1 = out1
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("c"))
        .unwrap()
        .rect;

    // Dirty set is empty after first layout; viewport changes.
    let out2 = layout(&mut tree, vp(1200.0, 100.0));
    let root2 = out2
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("root"))
        .unwrap()
        .rect;
    let c2 = out2
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("c"))
        .unwrap()
        .rect;

    // Root width is fixed at 900 — viewport widening doesn't change
    // root.w because the root's own style pins it. Children layout is
    // identical across viewport changes when the root has fixed size.
    assert_eq!(root2.w, 900.0);
    assert_eq!(
        c2, c1,
        "fixed-size root: viewport resize is a no-op for inner layout"
    );

    // But the spec requires a full recompute cycle to RUN. Verify the
    // last_viewport was updated (proves layout() touched taffy state).
    let lv = tree.last_viewport_for_test();
    assert_eq!(lv.unwrap().width, 1200.0);
}

#[test]
fn s5_viewport_resize_with_pct_root_actually_reflows() {
    // Distinct from the fixed-size case above: root sized in % of
    // viewport — children should reflow on resize.
    let mut tree = LayoutTree::new();
    tree.upsert(parent(
        "root",
        LayoutStyle {
            display: Some(DisplayKind::Block),
            width: Some(Dimension::Pct(100.0)),
            height: Some(Dimension::Pct(100.0)),
            ..Default::default()
        },
        vec![LayoutNodeId::new("child")],
    ));
    tree.upsert(leaf(
        "child",
        LayoutStyle {
            display: Some(DisplayKind::Block),
            width: Some(Dimension::Pct(100.0)),
            height: Some(Dimension::Px(100.0)),
            ..Default::default()
        },
    ));
    tree.set_root(LayoutNodeId::new("root"));

    let small = layout(&mut tree, vp(400.0, 300.0));
    let big = layout(&mut tree, vp(800.0, 600.0));
    let s = small
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("child"))
        .unwrap();
    let b = big
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("child"))
        .unwrap();

    assert_eq!(s.rect.w, 400.0);
    assert_eq!(
        b.rect.w, 800.0,
        "% sized root reflows children on viewport resize"
    );
}

// Test-only accessors on LayoutTree to assert internal state.
trait LayoutTreeTestExt {
    fn dirty_nodes_is_empty(&self) -> bool;
    fn last_viewport_for_test(&self) -> Option<Viewport>;
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
impl LayoutTreeTestExt for LayoutTree {
    fn dirty_nodes_is_empty(&self) -> bool {
        self.dirty_nodes().is_empty()
    }
    fn last_viewport_for_test(&self) -> Option<Viewport> {
        self.last_viewport()
    }
}
// CODEGEN-END

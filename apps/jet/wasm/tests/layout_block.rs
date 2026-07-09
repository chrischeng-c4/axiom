// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Layout-engine S1: single block child fills viewport width.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md#scenarios
//!
//! L0 pure-Rust tier — no browser, no wasm-pack. Exercises
//! `jet_wasm::renderer::layout::layout()` directly.

use jet_wasm::renderer::layout::{
    layout, leaf, parent, AlignItems, Dimension, DisplayKind, FlexDirection, JustifyContent,
    LayoutNodeId, LayoutStyle, LayoutTree, Rect4, Viewport,
};

fn vp(w: f32, h: f32) -> Viewport {
    Viewport {
        width: w,
        height: h,
        dpr: 1.0,
    }
}

#[test]
fn s1_block_child_fills_viewport_width() {
    let mut tree = LayoutTree::new();
    let root_style = LayoutStyle {
        display: Some(DisplayKind::Block),
        width: Some(Dimension::Pct(100.0)),
        height: Some(Dimension::Pct(100.0)),
        ..Default::default()
    };
    let child_style = LayoutStyle {
        display: Some(DisplayKind::Block),
        width: Some(Dimension::Pct(100.0)),
        height: Some(Dimension::Px(100.0)),
        ..Default::default()
    };
    tree.upsert(parent("root", root_style, vec![LayoutNodeId::new("child")]));
    tree.upsert(leaf("child", child_style));
    tree.set_root(LayoutNodeId::new("root"));

    let out = layout(&mut tree, vp(800.0, 600.0));
    let child = out
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("child"))
        .expect("child must be laid out");
    assert_eq!(child.rect.w, 800.0, "block child fills viewport width");
    assert_eq!(child.rect.h, 100.0, "block child honors fixed height");
    assert_eq!(child.rect.x, 0.0);
    assert_eq!(child.rect.y, 0.0);
}

#[test]
fn s1_block_with_padding_inflates_inner() {
    let mut tree = LayoutTree::new();
    let root_style = LayoutStyle {
        display: Some(DisplayKind::Block),
        width: Some(Dimension::Px(400.0)),
        height: Some(Dimension::Px(300.0)),
        padding: Some(Rect4 {
            top: Dimension::Px(10.0),
            right: Dimension::Px(20.0),
            bottom: Dimension::Px(10.0),
            left: Dimension::Px(20.0),
        }),
        ..Default::default()
    };
    let child_style = LayoutStyle {
        display: Some(DisplayKind::Block),
        width: Some(Dimension::Pct(100.0)),
        height: Some(Dimension::Px(50.0)),
        ..Default::default()
    };
    tree.upsert(parent("root", root_style, vec![LayoutNodeId::new("child")]));
    tree.upsert(leaf("child", child_style));
    tree.set_root(LayoutNodeId::new("root"));

    let out = layout(&mut tree, vp(400.0, 300.0));
    let child = out
        .iter()
        .find(|n| n.node_id == LayoutNodeId::new("child"))
        .unwrap();
    // child width = root.content-box.width = 400 - 20 - 20 = 360
    assert_eq!(child.rect.w, 360.0);
    // child x is 20 (the left padding) inside root at x=0
    assert_eq!(child.rect.x, 20.0);
    assert_eq!(child.rect.y, 10.0);
}

// Touch all imports to keep them honest across the suite.
#[allow(dead_code)]
fn _touch_imports() {
    let _ = (
        FlexDirection::Row,
        JustifyContent::Center,
        AlignItems::Center,
    );
}
// CODEGEN-END

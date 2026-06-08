// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Layout-engine S2 + S3: flex row distributes children, flex column
//! stacks children.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/layout-runtime.md#scenarios
//!
//! L0 pure-Rust tier.

use jet_wasm::renderer::layout::{
    layout, leaf, parent, Dimension, DisplayKind, FlexDirection, LayoutNodeId, LayoutStyle,
    LayoutTree, Viewport,
};

fn vp(w: f32, h: f32) -> Viewport {
    Viewport {
        width: w,
        height: h,
        dpr: 1.0,
    }
}

#[test]
fn s2_flex_row_distributes_children() {
    let mut tree = LayoutTree::new();
    let root_style = LayoutStyle {
        display: Some(DisplayKind::Flex),
        flex_direction: Some(FlexDirection::Row),
        width: Some(Dimension::Px(600.0)),
        height: Some(Dimension::Px(100.0)),
        ..Default::default()
    };
    let kid = |id: &str| {
        leaf(
            id,
            LayoutStyle {
                width: Some(Dimension::Px(200.0)),
                height: Some(Dimension::Px(100.0)),
                ..Default::default()
            },
        )
    };
    tree.upsert(parent(
        "root",
        root_style,
        vec![
            LayoutNodeId::new("a"),
            LayoutNodeId::new("b"),
            LayoutNodeId::new("c"),
        ],
    ));
    tree.upsert(kid("a"));
    tree.upsert(kid("b"));
    tree.upsert(kid("c"));
    tree.set_root(LayoutNodeId::new("root"));

    let out = layout(&mut tree, vp(600.0, 100.0));
    let find = |id: &str| {
        out.iter()
            .find(|n| n.node_id == LayoutNodeId::new(id))
            .cloned()
            .unwrap_or_else(|| panic!("{id} laid out"))
    };

    // Three 200px-wide children laid out side-by-side from x=0.
    assert_eq!(find("a").rect.x, 0.0);
    assert_eq!(find("a").rect.w, 200.0);
    assert_eq!(find("b").rect.x, 200.0);
    assert_eq!(find("b").rect.w, 200.0);
    assert_eq!(find("c").rect.x, 400.0);
    assert_eq!(find("c").rect.w, 200.0);
}

#[test]
fn s3_flex_column_stacks_children() {
    let mut tree = LayoutTree::new();
    let root_style = LayoutStyle {
        display: Some(DisplayKind::Flex),
        flex_direction: Some(FlexDirection::Column),
        width: Some(Dimension::Px(400.0)),
        height: Some(Dimension::Px(600.0)),
        ..Default::default()
    };
    let kid = |id: &str, h: f32| {
        leaf(
            id,
            LayoutStyle {
                width: Some(Dimension::Pct(100.0)),
                height: Some(Dimension::Px(h)),
                ..Default::default()
            },
        )
    };
    tree.upsert(parent(
        "root",
        root_style,
        vec![
            LayoutNodeId::new("hdr"),
            LayoutNodeId::new("body"),
            LayoutNodeId::new("ftr"),
        ],
    ));
    tree.upsert(kid("hdr", 60.0));
    tree.upsert(kid("body", 400.0));
    tree.upsert(kid("ftr", 40.0));
    tree.set_root(LayoutNodeId::new("root"));

    let out = layout(&mut tree, vp(400.0, 600.0));
    let find = |id: &str| {
        out.iter()
            .find(|n| n.node_id == LayoutNodeId::new(id))
            .cloned()
            .unwrap_or_else(|| panic!("{id} laid out"))
    };

    assert_eq!(find("hdr").rect.y, 0.0);
    assert_eq!(find("hdr").rect.h, 60.0);
    assert_eq!(find("body").rect.y, 60.0);
    assert_eq!(find("body").rect.h, 400.0);
    assert_eq!(find("ftr").rect.y, 460.0);
    assert_eq!(find("ftr").rect.h, 40.0);
    // All children fill width.
    assert_eq!(find("hdr").rect.w, 400.0);
    assert_eq!(find("body").rect.w, 400.0);
    assert_eq!(find("ftr").rect.w, 400.0);
}
// CODEGEN-END

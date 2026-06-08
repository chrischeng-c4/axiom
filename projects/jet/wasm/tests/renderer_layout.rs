// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Layout v0.1: simple block/inline positioning, tag-height
//! overrides, Empty skip. Element trees constructed by hand so these
//! tests don't depend on the transpiler or the runtime's mount flow.

use jet_wasm::renderer::{
    layout, layout_with_scroll_offsets, scroll_bounds_for_id, scrollbar_paint_ops, LaidOutKind,
    PaintOp, Rect, ScrollOffset, ScrollOffsets, Viewport,
};
use jet_wasm::{Element, Props};

fn button_vp() -> Viewport {
    Viewport {
        width: 400.0,
        height: 300.0,
        dpr: 1.0,
    }
}

fn scroll_td(text: &str) -> Element {
    Element::intrinsic(
        "td",
        Props {
            style: Some(
                "width: 72px; height: 24px; padding: 4px; border: 1px solid #d7dde8;".to_string(),
            ),
            ..Default::default()
        },
        vec![Element::text(text)],
    )
}

fn scroll_table_fixture(row_count: usize, col_count: usize, viewport_height: f32) -> Element {
    let rows = (0..row_count)
        .map(|row| {
            Element::intrinsic(
                "tr",
                Props::default(),
                (0..col_count)
                    .map(|col| scroll_td(&format!("cell {}", row * col_count + col)))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    Element::intrinsic(
        "div",
        Props {
            id: Some("table-viewport".to_string()),
            style: Some(format!("overflow: auto; height: {viewport_height}px;")),
            ..Default::default()
        },
        vec![Element::intrinsic(
            "table",
            Props {
                style: Some(format!("width: {}px;", col_count as f32 * 81.0)),
                ..Default::default()
            },
            vec![Element::intrinsic("tbody", Props::default(), rows)],
        )],
    )
}

#[test]
fn empty_element_produces_empty_tree() {
    let tree = layout(&Element::Empty, button_vp());
    assert_eq!(tree.nodes.len(), 0);
    assert_eq!(
        tree.root_rect,
        Rect {
            x: 0.0,
            y: 0.0,
            w: 400.0,
            h: 300.0
        }
    );
}

#[test]
fn text_element_gets_text_height() {
    let tree = layout(&Element::Text("hello".into()), button_vp());
    assert_eq!(tree.nodes.len(), 1);
    match &tree.nodes[0].kind {
        LaidOutKind::Text { content, .. } => assert_eq!(content, "hello"),
        _ => panic!("expected Text kind"),
    }
    assert_eq!(tree.nodes[0].rect.h, 18.0); // __text__ default
    assert_eq!(tree.nodes[0].rect.y, 0.0);
}

#[test]
fn button_uses_button_default_height() {
    let el = Element::intrinsic("button", Props::default(), vec![]);
    let tree = layout(&el, button_vp());
    assert_eq!(tree.nodes.len(), 1);
    assert_eq!(tree.nodes[0].rect.h, 21.0); // button default
    assert_eq!(tree.nodes[0].rect.w, 16.0); // empty intrinsic min width
}

/// @spec .aw/tech-design/projects/jet/specs/3945.md#unit-test
#[test]
fn inline_flow_positions_children_horizontally() {
    let el = Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::intrinsic("button", Props::default(), vec![]),
            Element::intrinsic("button", Props::default(), vec![]),
            Element::intrinsic("button", Props::default(), vec![]),
        ],
    );
    let tree = layout(&el, button_vp());
    // 1 div + 3 buttons = 4 nodes.
    assert_eq!(tree.nodes.len(), 4);
    // First node is the div, spanning its own measured height.
    let div_rect = tree.nodes[0].rect;
    assert_eq!(div_rect.y, 0.0);
    assert_eq!(div_rect.w, 400.0);
    // Div measures as max inline child height = 21.
    assert_eq!(div_rect.h, 21.0);

    // Each button sits inside the div on the same inline row.
    let b1 = tree.nodes[1].rect;
    let b2 = tree.nodes[2].rect;
    let b3 = tree.nodes[3].rect;
    assert_eq!(b1.x, 0.0);
    assert_eq!(b2.x, 16.0);
    assert_eq!(b3.x, 32.0);
    assert_eq!(b1.y, 0.0);
    assert_eq!(b2.y, 0.0);
    assert_eq!(b3.y, 0.0);
}

/// @spec .aw/tech-design/projects/jet/specs/3945.md#unit-test
#[test]
fn empty_child_contributes_zero_inline_width() {
    let el = Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::intrinsic("button", Props::default(), vec![]),
            Element::Empty,
            Element::intrinsic("button", Props::default(), vec![]),
        ],
    );
    let tree = layout(&el, button_vp());
    // 1 div + 2 buttons (Empty skipped).
    assert_eq!(tree.nodes.len(), 3);
    let [_, b1, b2] = [&tree.nodes[0], &tree.nodes[1], &tree.nodes[2]];
    assert_eq!(b1.rect.x, 0.0);
    assert_eq!(b2.rect.x, 16.0);
    assert_eq!(b1.rect.y, 0.0);
    assert_eq!(b2.rect.y, 0.0);
}

/// @spec .aw/tech-design/projects/jet/specs/3945.md#unit-test
#[test]
fn button_with_text_child_reports_button_height() {
    // <button>hi</button> — button keeps its 21 px; text lays out
    // inside the same box (at y = 0, h = 18 text-height).
    let el = Element::intrinsic("button", Props::default(), vec![Element::text("hi")]);
    let tree = layout(&el, button_vp());
    assert_eq!(tree.nodes.len(), 2);
    assert_eq!(tree.nodes[0].rect.h, 21.0);
    match &tree.nodes[1].kind {
        LaidOutKind::Text { content, .. } => assert_eq!(content, "hi"),
        _ => panic!("expected text child"),
    }
    // Text is laid out at the start of the button's box.
    assert_eq!(tree.nodes[1].rect.y, 0.0);
    assert_eq!(tree.nodes[1].rect.h, 18.0);
}

#[test]
#[should_panic(expected = "unrendered Component")]
fn unrendered_component_panics() {
    use jet_wasm::Component;
    use std::rc::Rc;
    fn noop(_: &Rc<dyn std::any::Any>) -> Element {
        Element::Empty
    }
    let comp = Component {
        name: "X",
        render: noop,
        props: Rc::new(()) as Rc<dyn std::any::Any>,
    };
    let _ = layout(&Element::Component(comp), button_vp());
}

/// @spec .aw/tech-design/projects/jet/specs/3945.md#unit-test
#[test]
fn block_container_width_propagates_to_root() {
    let vp = Viewport {
        width: 1000.0,
        height: 100.0,
        dpr: 1.0,
    };
    let el = Element::intrinsic("div", Props::default(), vec![]);
    let tree = layout(&el, vp);
    assert_eq!(tree.nodes[0].rect.w, 1000.0);
}

#[test]
fn styled_table_rows_stack_vertically_and_cells_keep_fixed_size() {
    fn td(text: &str) -> Element {
        Element::intrinsic(
            "td",
            Props {
                style: Some(
                    "width: 72px; height: 24px; padding: 4px; border: 1px solid #d7dde8;"
                        .to_string(),
                ),
                ..Default::default()
            },
            vec![Element::text(text)],
        )
    }

    let table = Element::intrinsic(
        "table",
        Props {
            style: Some("width: 144px;".to_string()),
            ..Default::default()
        },
        vec![Element::intrinsic(
            "tbody",
            Props::default(),
            vec![
                Element::intrinsic("tr", Props::default(), vec![td("cell 0"), td("cell 1")]),
                Element::intrinsic("tr", Props::default(), vec![td("cell 2"), td("cell 3")]),
            ],
        )],
    );

    let tree = layout(&table, button_vp());
    let rows = tree
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            LaidOutKind::Intrinsic { tag: "tr", .. } => Some(node.rect),
            _ => None,
        })
        .collect::<Vec<_>>();
    let cells = tree
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            LaidOutKind::Intrinsic { tag: "td", .. } => Some(node.rect),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].y, 0.0);
    assert_eq!(rows[1].y, 33.0);
    assert_eq!(cells.len(), 4);
    assert_eq!(cells[0].rect_tuple(), (0.0, 0.0, 81.0, 33.0));
    assert_eq!(cells[1].rect_tuple(), (81.0, 0.0, 81.0, 33.0));
    assert_eq!(cells[2].rect_tuple(), (0.0, 33.0, 81.0, 33.0));
    assert_eq!(cells[3].rect_tuple(), (81.0, 33.0, 81.0, 33.0));

    let text_rects = tree
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            LaidOutKind::Text { content, .. } if content == "cell 0" => Some(node.rect),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(text_rects.len(), 1);
    assert_eq!(text_rects[0].y, 7.5);
}

#[test]
fn overflow_auto_container_scroll_offset_brings_later_rows_into_view() {
    fn td(text: &str) -> Element {
        Element::intrinsic(
            "td",
            Props {
                style: Some(
                    "width: 72px; height: 24px; padding: 4px; border: 1px solid #d7dde8;"
                        .to_string(),
                ),
                ..Default::default()
            },
            vec![Element::text(text)],
        )
    }

    let rows = (0..4)
        .map(|row| {
            Element::intrinsic(
                "tr",
                Props::default(),
                vec![
                    td(&format!("cell {}", row * 2)),
                    td(&format!("cell {}", row * 2 + 1)),
                ],
            )
        })
        .collect::<Vec<_>>();
    let viewport = Element::intrinsic(
        "div",
        Props {
            id: Some("table-viewport".to_string()),
            style: Some("overflow: auto; height: 66px;".to_string()),
            ..Default::default()
        },
        vec![Element::intrinsic(
            "table",
            Props {
                style: Some("width: 162px;".to_string()),
                ..Default::default()
            },
            vec![Element::intrinsic("tbody", Props::default(), rows)],
        )],
    );

    let mut scroll_offsets = ScrollOffsets::default();
    scroll_offsets.set("table-viewport", ScrollOffset { x: 0.0, y: 66.0 });
    let tree = layout_with_scroll_offsets(&viewport, button_vp(), &scroll_offsets);
    let visible_text = tree
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            LaidOutKind::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        visible_text.contains(&"cell 4") && visible_text.contains(&"cell 7"),
        "scrolled viewport should expose later rows, got {visible_text:?}"
    );
    assert!(
        !visible_text.contains(&"cell 0") && !visible_text.contains(&"cell 1"),
        "first row should be clipped after scrolling, got {visible_text:?}"
    );
}

#[test]
fn overflow_auto_scroll_bounds_clamp_offsets_to_content_extent() {
    let viewport = scroll_table_fixture(4, 2, 66.0);
    let bounds = scroll_bounds_for_id(&viewport, button_vp(), "table-viewport")
        .expect("table viewport should expose scroll bounds");

    assert_eq!(bounds.viewport_rect.h, 66.0);
    assert_eq!(bounds.content_height, 132.0);
    assert_eq!(bounds.max_y, 66.0);
    assert_eq!(bounds.max_x, 0.0);
    assert_eq!(
        bounds.clamp(ScrollOffset { x: 12.0, y: 999.0 }),
        ScrollOffset { x: 0.0, y: 66.0 }
    );
}

#[test]
fn overflow_auto_scrollbar_paint_ops_include_vertical_thumb() {
    let viewport = scroll_table_fixture(4, 2, 66.0);
    let bounds = scroll_bounds_for_id(&viewport, button_vp(), "table-viewport")
        .expect("table viewport should expose scroll bounds");
    let ops = scrollbar_paint_ops(bounds, ScrollOffset { x: 0.0, y: 33.0 });

    let rects = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::FillRect { rect, .. } => Some(*rect),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(rects.len(), 2, "expected vertical track + thumb: {ops:?}");
    assert_eq!(rects[0].rect_tuple(), (390.0, 0.0, 10.0, 66.0));
    assert_eq!(rects[1].rect_tuple(), (390.0, 16.5, 10.0, 33.0));
}

trait RectTuple {
    fn rect_tuple(self) -> (f32, f32, f32, f32);
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#unit-test
impl RectTuple for Rect {
    fn rect_tuple(self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.w, self.h)
    }
}
// CODEGEN-END

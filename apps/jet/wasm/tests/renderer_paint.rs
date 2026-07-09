// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Paint v0: layout-tree → PaintOp list. Snapshot-style tests over
//! hand-built LayoutTrees so the expected ops live as Rust arrays
//! in the test file (reviewable as normal code).

use jet_wasm::renderer::{layout, paint, Color, FontSpec, PaintOp, Rect, Theme, Viewport};
use jet_wasm::{Element, Props};

fn vp() -> Viewport {
    Viewport {
        width: 320.0,
        height: 100.0,
        dpr: 1.0,
    }
}

#[test]
fn empty_tree_yields_background_only() {
    let tree = layout(&Element::Empty, vp());
    let ops = paint(&tree, &Theme::default());
    // Background fill only.
    assert_eq!(ops.len(), 1);
    match &ops[0] {
        PaintOp::FillRect { rect, color } => {
            assert_eq!(
                *rect,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    w: 320.0,
                    h: 100.0
                }
            );
            assert_eq!(*color, Theme::default().bg);
        }
        other => panic!("unexpected op: {other:?}"),
    }
}

#[test]
fn button_emits_fill_plus_stroke_plus_background() {
    let el = Element::intrinsic("button", Props::default(), vec![]);
    let tree = layout(&el, vp());
    let ops = paint(&tree, &Theme::default());
    // Expected ops (in order):
    // 0: FillRect bg (root)
    // 1: FillRect button bg
    // 2: StrokeRect button border
    assert_eq!(ops.len(), 3);
    matches!(ops[0], PaintOp::FillRect { .. });
    matches!(ops[1], PaintOp::FillRect { .. });
    matches!(ops[2], PaintOp::StrokeRect { .. });
}

#[test]
fn text_emits_text_op_with_theme_font() {
    let el = Element::text("hi");
    let tree = layout(&el, vp());
    let ops = paint(&tree, &Theme::default());
    assert_eq!(ops.len(), 2);
    match &ops[1] {
        PaintOp::Text {
            content,
            font,
            color,
            ..
        } => {
            assert_eq!(content, "hi");
            assert_eq!(font.size_px, 14.0);
            assert_eq!(font.weight, 400);
            assert_eq!(*color, Theme::default().text_color);
        }
        other => panic!("expected Text op, got {other:?}"),
    }
}

#[test]
fn button_with_text_child_paints_button_chrome_then_text() {
    let el = Element::intrinsic("button", Props::default(), vec![Element::text("click me")]);
    let tree = layout(&el, vp());
    let ops = paint(&tree, &Theme::default());
    // Expected:
    // 0: bg
    // 1: button fill
    // 2: button stroke
    // 3: text
    assert_eq!(ops.len(), 4);
    match &ops[3] {
        PaintOp::Text { content, .. } => assert_eq!(content, "click me"),
        other => panic!("expected Text op, got {other:?}"),
    }
}

#[test]
fn custom_theme_changes_button_color() {
    let el = Element::intrinsic("button", Props::default(), vec![]);
    let tree = layout(&el, vp());
    let mut theme = Theme::default();
    theme.button_bg = Color::rgb(0xff, 0x00, 0x00);
    let ops = paint(&tree, &theme);
    match &ops[1] {
        PaintOp::FillRect { color, .. } => {
            assert_eq!(*color, Color::rgb(0xff, 0x00, 0x00));
        }
        other => panic!("expected FillRect, got {other:?}"),
    }
}

#[test]
fn container_div_emits_only_children_not_its_own_chrome() {
    // Divs are invisible unless styled — no fill / stroke for the
    // container itself. Child button still paints its chrome.
    let el = Element::intrinsic(
        "div",
        Props::default(),
        vec![Element::intrinsic("button", Props::default(), vec![])],
    );
    let tree = layout(&el, vp());
    let ops = paint(&tree, &Theme::default());
    // 0: bg, 1: button fill, 2: button stroke.
    assert_eq!(ops.len(), 3);
    let div_ops: Vec<_> = ops
        .iter()
        .filter(|op| matches!(op, PaintOp::FillRect { .. } | PaintOp::StrokeRect { .. }))
        .collect();
    // Exactly: 1 bg + 1 button fill + 1 button stroke = 3.
    assert_eq!(div_ops.len(), 3);
}

#[test]
fn font_spec_defaults_stable_under_theme_default() {
    // Regression guard: changing Theme::default in a way that shifts
    // font specs would blow this snapshot and force review.
    let theme = Theme::default();
    assert_eq!(
        theme.default_font,
        FontSpec {
            family: "system-ui, sans-serif".to_string(),
            size_px: 14.0,
            weight: 400,
        }
    );
}

#[test]
fn styled_table_cell_paints_background_and_border() {
    let cell = Element::intrinsic(
        "td",
        Props {
            style: Some(
                "width: 72px; height: 24px; background: #ffffff; border: 1px solid #d7dde8;"
                    .to_string(),
            ),
            ..Default::default()
        },
        vec![Element::text("cell 0")],
    );
    let table = Element::intrinsic(
        "table",
        Props {
            style: Some("width: 72px;".to_string()),
            ..Default::default()
        },
        vec![Element::intrinsic(
            "tbody",
            Props::default(),
            vec![Element::intrinsic("tr", Props::default(), vec![cell])],
        )],
    );

    let tree = layout(&table, vp());
    let ops = paint(&tree, &Theme::default());
    let cell_fill = ops.iter().any(|op| {
        matches!(
            op,
            PaintOp::FillRect {
                rect,
                color
            } if *rect == Rect { x: 0.0, y: 0.0, w: 73.0, h: 25.0 }
                && *color == Color::rgb(0xff, 0xff, 0xff)
        )
    });
    let cell_border = ops.iter().any(|op| {
        matches!(
            op,
            PaintOp::StrokeRect {
                rect,
                color,
                width
            } if *rect == Rect { x: 0.0, y: 0.0, w: 73.0, h: 25.0 }
                && *color == Color::rgb(0xd7, 0xdd, 0xe8)
                && (*width - 1.0).abs() < f32::EPSILON
        )
    });

    assert!(cell_fill, "styled td should paint its background");
    assert!(cell_border, "styled td should paint its border");
}

#[test]
fn styled_table_cell_text_uses_cell_font_size_and_color() {
    let cell = Element::intrinsic(
        "td",
        Props {
            style: Some("width: 72px; height: 24px; color: #1f2937; font-size: 12px;".to_string()),
            ..Default::default()
        },
        vec![Element::text("cell 0")],
    );
    let table = Element::intrinsic(
        "table",
        Props {
            style: Some("width: 72px;".to_string()),
            ..Default::default()
        },
        vec![Element::intrinsic(
            "tbody",
            Props::default(),
            vec![Element::intrinsic("tr", Props::default(), vec![cell])],
        )],
    );

    let tree = layout(&table, vp());
    let ops = paint(&tree, &Theme::default());
    let text = ops.iter().find_map(|op| match op {
        PaintOp::Text {
            content,
            font,
            color,
            ..
        } if content == "cell 0" => Some((font, color)),
        _ => None,
    });

    let (font, color) = text.expect("styled table cell should paint text");
    assert_eq!(font.size_px, 12.0);
    assert_eq!(*color, Color::rgb(0x1f, 0x29, 0x37));
}
// CODEGEN-END

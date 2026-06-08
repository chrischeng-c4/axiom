// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Layout v0.1: simple block/inline positioning, tag-height
//! overrides, Empty skip. Element trees constructed by hand so these
//! tests don't depend on the transpiler or the runtime's mount flow.

use jet_wasm::renderer::{layout, LaidOutKind, Rect, Viewport};
use jet_wasm::{Element, Props};

fn button_vp() -> Viewport {
    Viewport {
        width: 400.0,
        height: 300.0,
        dpr: 1.0,
    }
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
        LaidOutKind::Text(s) => assert_eq!(s, "hello"),
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
        LaidOutKind::Text(s) => assert_eq!(s, "hi"),
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
// CODEGEN-END

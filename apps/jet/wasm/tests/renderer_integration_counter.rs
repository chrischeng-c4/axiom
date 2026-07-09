// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! End-to-end integration: mount Counter from jet-react-wasm-runtime,
//! run the renderer, assert the op list shape + key ops present.
//! This is the first test proving the full pipeline:
//!
//!   Component → Element tree (runtime) → LayoutTree → PaintOps (renderer)
//!
//! The Component is hand-authored to mirror what jet-tsx-to-rust
//! emits for Counter.tsx. A future test will run the actual transpiler
//! output through this same harness.

use jet_wasm::react::{mount, use_state};
use jet_wasm::renderer::{NoopBackend, PaintOp, RecordingBackend, Renderer, Theme, Viewport};
use jet_wasm::{Callback, Component, Element, Props};
use std::any::Any;
use std::rc::Rc;

// ── Hand-authored Counter (mirrors Counter.tsx transpile output) ─────────────

#[derive(Clone, Debug)]
struct CounterProps {
    start: i64,
}

fn counter_render(props: &Rc<dyn Any>) -> Element {
    let props: &CounterProps = props.downcast_ref().expect("CounterProps");
    let (n, set_n) = use_state::<i64>(props.start);
    let handler = {
        let set_n = set_n.clone();
        let n_cap = n;
        Callback::new(move |_| set_n.set(n_cap + 1))
    };
    Element::intrinsic(
        "button",
        Props {
            id: Some("inc".to_string()),
            on_click: Some(handler),
            ..Default::default()
        },
        vec![Element::text("count: "), Element::from_number(n)],
    )
}

fn counter(start: i64) -> Component {
    Component {
        name: "Counter",
        render: counter_render,
        props: Rc::new(CounterProps { start }),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn counter_renders_expected_op_shape() {
    let handle = mount(counter(0));
    let tree = handle.snapshot();
    let mut renderer = Renderer::new(
        Viewport {
            width: 300.0,
            height: 60.0,
            dpr: 1.0,
        },
        Theme::default(),
        NoopBackend,
    );
    let ops = renderer.render(&tree);

    // Expected:
    // 0: FillRect  (root bg)
    // 1: FillRect  (button bg)
    // 2: StrokeRect (button border)
    // 3: Text "count: "
    // 4: Text "0"
    assert!(
        ops.len() >= 5,
        "expected ≥5 ops, got {}: {ops:#?}",
        ops.len()
    );
    let text_ops: Vec<_> = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();
    assert!(
        text_ops.iter().any(|t| t.contains("count")),
        "no 'count' text op: {text_ops:?}"
    );
    assert!(
        text_ops.iter().any(|t| t == "0"),
        "no '0' text op: {text_ops:?}"
    );
}

#[test]
fn counter_after_click_rerenders_with_new_text() {
    let handle = mount(counter(5));
    let mut renderer = Renderer::new(
        Viewport {
            width: 300.0,
            height: 60.0,
            dpr: 1.0,
        },
        Theme::default(),
        NoopBackend,
    );

    let ops_before = renderer.render(&handle.snapshot());
    let text_before: Vec<String> = ops_before
        .iter()
        .filter_map(|op| match op {
            PaintOp::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();
    assert!(text_before.iter().any(|t| t == "5"));

    // Click.
    let cb = handle.snapshot().find_on_click("inc").expect("on_click");
    cb.call(());
    assert!(handle.flush());

    let ops_after = renderer.render(&handle.snapshot());
    let text_after: Vec<String> = ops_after
        .iter()
        .filter_map(|op| match op {
            PaintOp::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect();
    assert!(text_after.iter().any(|t| t == "6"));
    assert!(!text_after.iter().any(|t| t == "5"));
}

#[test]
fn recording_backend_receives_same_ops_as_return_value() {
    let handle = mount(counter(0));
    let mut renderer = Renderer::new(
        Viewport::default(),
        Theme::default(),
        RecordingBackend::new(),
    );
    let returned_ops = renderer.render(&handle.snapshot());
    assert_eq!(renderer.backend.received, returned_ops);
}

#[test]
fn multiple_renders_are_independent() {
    // The renderer has no state carried across render() calls — same
    // Element tree produces the same ops.
    let tree = Element::intrinsic("button", Props::default(), vec![Element::text("static")]);
    let mut renderer = Renderer::new(Viewport::default(), Theme::default(), NoopBackend);
    let a = renderer.render(&tree);
    let b = renderer.render(&tree);
    assert_eq!(a, b);
}
// CODEGEN-END

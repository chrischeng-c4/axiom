// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Counter component: hand-written Rust equivalent of the TSX
//! that `jet-tsx-to-rust` will eventually produce. This file is the
//! **target shape** — when the transpiler lands, its output for the
//! TSX below should be byte-for-byte equivalent to the Rust in this
//! test (modulo formatting).
//!
//! TSX source (for reference):
//!
//! ```tsx
//! interface CounterProps { start: number }
//!
//! export function Counter({ start }: CounterProps) {
//!   const [n, setN] = useState(start);
//!   return (
//!     <button id="inc" onClick={() => setN(n + 1)}>
//!       count: {n}
//!     </button>
//!   );
//! }
//! ```
//!
//! The tests below verify that:
//! 1. Initial render produces the expected tree.
//! 2. Hooks persist state across renders (cursor reset works).
//! 3. Invoking `on_click` marks the fiber dirty.
//! 4. Flush re-runs the component and the new tree reflects new state.
//! 5. Multiple rounds of click + flush behave monotonically.

use jet_wasm::react::{mount, use_state};
use jet_wasm::{Callback, Component, Element, Props};
use std::rc::Rc;

// ── Generated-shaped Counter ────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct CounterProps {
    start: i64,
}

fn counter_render(props: &Rc<dyn std::any::Any>) -> Element {
    // Transpiler will emit: let props = props.downcast_ref::<CounterProps>().unwrap();
    let props: &CounterProps = props.downcast_ref().expect("CounterProps");
    // Transpiler will emit the useState call with the correct generic type
    // inferred from the TSX annotation.
    let (n, set_n) = use_state::<i64>(props.start);
    // Transpiler-emitted event handler — the closure captures the current
    // `n` (by-value clone because i64: Copy) and the setter (Clone).
    let handler = {
        let set_n = set_n.clone();
        let n_captured = n;
        Callback::new(move |_| set_n.set(n_captured + 1))
    };
    // Transpiler-emitted JSX:
    // <button id="inc" onClick={...}>count: {n}</button>
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn initial_render_matches_props() {
    let handle = mount(counter(0));
    let tree = handle.snapshot();
    assert_eq!(tree.text_content(), "count: 0");
}

#[test]
fn initial_state_overrides_start_after_update() {
    let handle = mount(counter(7));
    assert_eq!(handle.snapshot().text_content(), "count: 7");
    let cb = handle.snapshot().find_on_click("inc").expect("on_click");
    cb.call(());
    assert!(handle.flush());
    assert_eq!(handle.snapshot().text_content(), "count: 8");
}

#[test]
fn multiple_clicks_accumulate() {
    let handle = mount(counter(0));
    for expected in 1..=5 {
        let cb = handle
            .snapshot()
            .find_on_click("inc")
            .expect("on_click missing");
        cb.call(());
        assert!(handle.flush(), "flush should re-render on every click");
        assert_eq!(
            handle.snapshot().text_content(),
            format!("count: {expected}"),
            "after click {expected}"
        );
    }
}

#[test]
fn flush_without_dirty_is_noop() {
    let handle = mount(counter(3));
    assert!(!handle.flush(), "no state change → no re-render");
    assert_eq!(handle.snapshot().text_content(), "count: 3");
}

#[test]
fn hook_cursor_resets_between_renders() {
    // If cursor didn't reset, subsequent renders would append NEW
    // hook slots instead of reusing the existing one — state would
    // reset to the initial value on every render.
    let handle = mount(counter(100));
    let cb = handle.snapshot().find_on_click("inc").unwrap();
    cb.call(());
    assert!(handle.flush());
    assert_eq!(handle.snapshot().text_content(), "count: 101");
    // If cursor leaked, this would jump back to 100.
    let cb = handle.snapshot().find_on_click("inc").unwrap();
    cb.call(());
    assert!(handle.flush());
    assert_eq!(handle.snapshot().text_content(), "count: 102");
}
// CODEGEN-END

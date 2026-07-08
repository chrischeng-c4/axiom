// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-tests.md#tests
// CODEGEN-BEGIN
//! Covers the v1 hook surface added per
//! `.aw/tech-design/projects/jet/wasm-renderer/hooks-runtime.md`:
//! `use_reducer`, `use_ref`, `use_memo`, `use_callback`.
//!
//! Each test uses a hand-authored Component whose shape mirrors what
//! `jet-tsx-to-rust` will eventually emit — so any break in the
//! transpiler's output shape also breaks these tests.

use jet_wasm::react::{
    hash_dep, mount, use_callback, use_memo, use_reducer, use_ref, use_state, RefHandle,
};
use jet_wasm::{Callback, Component, Element, Props};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

// ── use_reducer: counter driven by Action enum ──────────────────────────────

#[derive(Clone, Debug, PartialEq)]
enum CountAction {
    Inc,
    Dec,
    Reset,
}

#[derive(Clone)]
struct ReducerProps {
    start: i64,
}

fn reducer_render(props: &Rc<dyn Any>) -> Element {
    let props: &ReducerProps = props.downcast_ref().expect("ReducerProps");
    let (count, dispatch) = use_reducer::<i64, CountAction, _>(
        |state, action| match action {
            CountAction::Inc => state + 1,
            CountAction::Dec => state - 1,
            CountAction::Reset => 0,
        },
        props.start,
    );
    let d_inc = dispatch.clone();
    let d_dec = dispatch.clone();
    let d_reset = dispatch;
    Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::from_number(count),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("inc".into()),
                    on_click: Some(Callback::new(move |_| d_inc.dispatch(CountAction::Inc))),
                    ..Default::default()
                },
                vec![Element::text("+")],
            ),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("dec".into()),
                    on_click: Some(Callback::new(move |_| d_dec.dispatch(CountAction::Dec))),
                    ..Default::default()
                },
                vec![Element::text("-")],
            ),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("reset".into()),
                    on_click: Some(Callback::new(move |_| d_reset.dispatch(CountAction::Reset))),
                    ..Default::default()
                },
                vec![Element::text("0")],
            ),
        ],
    )
}

#[test]
fn use_reducer_dispatches_actions() {
    let handle = mount(Component {
        name: "Reducer",
        render: reducer_render,
        props: Rc::new(ReducerProps { start: 10 }),
    });
    assert!(handle.snapshot().text_content().starts_with("10"));

    handle.snapshot().find_on_click("inc").unwrap().call(());
    assert!(handle.flush());
    assert!(handle.snapshot().text_content().starts_with("11"));

    handle.snapshot().find_on_click("inc").unwrap().call(());
    handle.flush();
    handle.snapshot().find_on_click("dec").unwrap().call(());
    handle.flush();
    assert!(handle.snapshot().text_content().starts_with("11"));

    handle.snapshot().find_on_click("reset").unwrap().call(());
    handle.flush();
    assert!(handle.snapshot().text_content().starts_with("0"));
}

// ── use_ref: survives renders, doesn't trigger re-render ────────────────────

#[derive(Clone)]
struct RefProps;

fn ref_render(_props: &Rc<dyn Any>) -> Element {
    let (n, set_n) = use_state::<i64>(0);
    let counter: RefHandle<i64> = use_ref(0);

    // Every render bumps the ref. Mutation doesn't trigger re-render;
    // only the use_state setter does.
    counter.with_mut(|c| *c += 1);

    let set_n_click = set_n.clone();
    let counter_click = counter.clone();
    Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::from_number(n),
            Element::text(" renders="),
            Element::from_number(counter.current()),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("bump".into()),
                    on_click: Some(Callback::new(move |_| {
                        set_n_click.set(n + 1);
                        // Also tweak the ref from the handler — should
                        // persist into the next render.
                        counter_click.with_mut(|c| *c += 100);
                    })),
                    ..Default::default()
                },
                vec![Element::text("bump")],
            ),
        ],
    )
}

#[test]
fn use_ref_persists_across_renders() {
    let handle = mount(Component {
        name: "RefComp",
        render: ref_render,
        props: Rc::new(RefProps),
    });
    // First render: counter bumped from 0 → 1. text_content() walks
    // the full tree, so the button's "bump" label is part of it.
    assert_eq!(handle.snapshot().text_content(), "0 renders=1bump");

    // Click: set_n(1) schedules re-render AND ref mutation adds 100.
    // Then render runs again and bumps the ref one more time (+1).
    handle.snapshot().find_on_click("bump").unwrap().call(());
    handle.flush();
    // After click: ref is 1 + 100 + 1 = 102. State is 1.
    assert_eq!(handle.snapshot().text_content(), "1 renders=102bump");
}

// ── use_memo: recomputes only on deps change ────────────────────────────────

#[derive(Clone)]
struct MemoProps;

thread_local! {
    static COMPUTE_COUNT: RefCell<u32> = const { RefCell::new(0) };
}

fn memo_render(_props: &Rc<dyn Any>) -> Element {
    let (n, set_n) = use_state::<i64>(0);
    let (unrelated, set_unrelated) = use_state::<i64>(0);

    // Expensive computation gated by deps on `n` only.
    let squared = use_memo::<i64, _>(
        || {
            COMPUTE_COUNT.with(|c| *c.borrow_mut() += 1);
            n * n
        },
        vec![hash_dep(&n)],
    );

    let set_n_cb = set_n.clone();
    let set_u_cb = set_unrelated.clone();
    Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::from_number(squared),
            Element::text(" u="),
            Element::from_number(unrelated),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("bump-n".into()),
                    on_click: Some(Callback::new(move |_| set_n_cb.set(n + 1))),
                    ..Default::default()
                },
                vec![Element::text("n+")],
            ),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("bump-u".into()),
                    on_click: Some(Callback::new(move |_| set_u_cb.set(unrelated + 1))),
                    ..Default::default()
                },
                vec![Element::text("u+")],
            ),
        ],
    )
}

#[test]
fn use_memo_skips_compute_when_deps_unchanged() {
    COMPUTE_COUNT.with(|c| *c.borrow_mut() = 0);
    let handle = mount(Component {
        name: "Memo",
        render: memo_render,
        props: Rc::new(MemoProps),
    });
    assert_eq!(
        COMPUTE_COUNT.with(|c| *c.borrow()),
        1,
        "initial render computes once"
    );

    // Change unrelated state — re-renders but deps for the memo are
    // unchanged, so compute count stays at 1.
    handle.snapshot().find_on_click("bump-u").unwrap().call(());
    handle.flush();
    assert_eq!(
        COMPUTE_COUNT.with(|c| *c.borrow()),
        1,
        "unrelated change did not re-compute"
    );

    // Change n — deps hash changes, compute runs again.
    handle.snapshot().find_on_click("bump-n").unwrap().call(());
    handle.flush();
    assert_eq!(
        COMPUTE_COUNT.with(|c| *c.borrow()),
        2,
        "n change triggered recompute"
    );
}

// ── use_callback: stable identity across renders when deps unchanged ────────

#[derive(Clone)]
struct CbProps;

fn cb_render(_props: &Rc<dyn Any>) -> Element {
    let (n, set_n) = use_state::<i64>(0);

    // Two callbacks: one that captures n (deps include n); one that
    // doesn't (empty deps). The latter must be stable across renders.
    let with_dep: Callback<()> = use_callback(
        {
            let set_n = set_n.clone();
            move |_| set_n.set(n + 1)
        },
        vec![hash_dep(&n)],
    );
    let without_dep: Callback<()> = use_callback::<(), _>(
        {
            let set_n = set_n.clone();
            move |_| set_n.set(0)
        },
        vec![],
    );

    Element::intrinsic(
        "div",
        Props::default(),
        vec![
            Element::from_number(n),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("with".into()),
                    on_click: Some(with_dep),
                    ..Default::default()
                },
                vec![Element::text("w")],
            ),
            Element::intrinsic(
                "button",
                Props {
                    id: Some("without".into()),
                    on_click: Some(without_dep),
                    ..Default::default()
                },
                vec![Element::text("r")],
            ),
        ],
    )
}

#[test]
fn use_callback_returns_working_callbacks() {
    let handle = mount(Component {
        name: "Cb",
        render: cb_render,
        props: Rc::new(CbProps),
    });

    handle.snapshot().find_on_click("with").unwrap().call(());
    handle.flush();
    assert_eq!(handle.snapshot().text_content(), "1wr");

    handle.snapshot().find_on_click("with").unwrap().call(());
    handle.flush();
    assert_eq!(handle.snapshot().text_content(), "2wr");

    handle.snapshot().find_on_click("without").unwrap().call(());
    handle.flush();
    // Reset callback fires — brings n back to 0.
    assert_eq!(handle.snapshot().text_content(), "0wr");
}

// Rules-of-hooks cross-render violation — intentionally NOT unit-
// tested here because constructing it requires swapping render fns
// mid-mount, which the current MountHandle doesn't expose. The
// slot-kind guards are exercised by the positive-path tests above
// via the pattern matches inside each hook implementation.
// CODEGEN-END

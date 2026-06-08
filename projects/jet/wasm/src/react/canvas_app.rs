// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
// CODEGEN-BEGIN
//! Canvas event loop — wires a mounted React component to a canvas
//! element end-to-end.
//!
//! Responsibilities:
//!
//! 1. Size the canvas (DPR-aware) and build a `CanvasBackend`.
//! 2. Run the first layout + paint; cache the `LayoutTree` for
//!    hit-testing.
//! 3. Install a `click` listener. On click: hit-test the cached
//!    layout, invoke any matched `on_click`, flush pending state
//!    updates, and — if anything changed — re-layout + repaint.
//!
//! Feature-gated on `canvas-app` (which pulls in `canvas` + `react`).
//! Deliberately kept to a single entry point (`run`) so the emitted
//! `#[wasm_bindgen(start)]` collapses to one line.

#![cfg(feature = "canvas-app")]

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::renderer::{
    self, CanvasBackend, LayoutTree, PaintBackend, PaintOp, Point, Renderer, Theme, Viewport,
};
use crate::Component;

use super::{mount, MountHandle};

#[cfg(feature = "debug")]
use crate::debug::{DebugBridgeState, JetDebug};

/// Mount `component` on `#<canvas_id>`, install event listeners, and
/// run the commit loop. Returns once everything is wired — the loop
/// itself is callback-driven (click → repaint) so this fn does not
/// block.
///
/// Expects the canvas to exist in the document already (see
/// `examples/counter-demo/dist/index.html` for the minimal host page).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn run(canvas_id: &str, component: Component) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas element not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("element is not an HTMLCanvasElement"))?;

    // DPR-aware sizing: back the CSS box with a higher-density pixel
    // buffer so text + strokes stay crisp on retina displays.
    let dpr = window.device_pixel_ratio() as f32;
    let css_w = canvas.client_width().max(1) as f32;
    let css_h = canvas.client_height().max(1) as f32;
    canvas.set_width((css_w * dpr) as u32);
    canvas.set_height((css_h * dpr) as u32);

    let ctx = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("no 2d context"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| JsValue::from_str("not a CanvasRenderingContext2d"))?;
    ctx.scale(dpr as f64, dpr as f64)?;

    let backend = CanvasBackend::new(ctx);
    let viewport = Viewport {
        width: css_w,
        height: css_h,
        dpr,
    };
    let renderer = Rc::new(RefCell::new(Renderer::new(
        viewport,
        Theme::default(),
        backend,
    )));
    let handle = Rc::new(mount(component));

    // Debug plumbing (no-op when feature is off): shared handles that
    // let `JetDebug` read the last-frame layout/ops + nudge a repaint.
    let last_ops: Rc<RefCell<Option<Vec<PaintOp>>>> = Rc::new(RefCell::new(None));
    let highlight_index: Rc<RefCell<Option<usize>>> = Rc::new(RefCell::new(None));

    // First paint + cache the layout tree for hit-testing.
    let layout_tree: Rc<RefCell<LayoutTree>> = Rc::new(RefCell::new(repaint(
        &handle,
        &renderer,
        &last_ops,
        &highlight_index,
    )));

    #[cfg(feature = "debug")]
    {
        // Build a repaint trigger that closes over the same state
        // the click listener touches — so `force_rerender` and
        // `highlight` go through the same codepath as user clicks.
        let handle_for_trigger = handle.clone();
        let renderer_for_trigger = renderer.clone();
        let layout_tree_for_trigger = layout_tree.clone();
        let last_ops_for_trigger = last_ops.clone();
        let highlight_for_trigger = highlight_index.clone();
        let repaint_trigger: crate::debug::RepaintTrigger = Rc::new(move || {
            let _ = handle_for_trigger.flush();
            *layout_tree_for_trigger.borrow_mut() = repaint(
                &handle_for_trigger,
                &renderer_for_trigger,
                &last_ops_for_trigger,
                &highlight_for_trigger,
            );
        });

        let bridge = DebugBridgeState {
            layout_tree: layout_tree.clone(),
            last_ops: last_ops.clone(),
            highlight_index: highlight_index.clone(),
        };
        let debug_handle = JetDebug::new(handle.clone(), bridge, repaint_trigger);
        // Register on window.__jet_debug so `jet browser ...` and any
        // DevTools Console user can reach the bridge.
        js_sys::Reflect::set(
            &window,
            &JsValue::from_str("__jet_debug"),
            &JsValue::from(debug_handle),
        )?;
    }

    install_click_listener(
        &canvas,
        handle,
        renderer,
        layout_tree,
        last_ops,
        highlight_index,
    )?;
    Ok(())
}

/// Run the component, layout it against the current viewport, paint
/// and execute the resulting ops. Returns the new `LayoutTree` so the
/// caller can cache it for subsequent hit-testing. Also caches the
/// emitted ops (for `JetDebug::paint_ops`) and — if a highlight
/// index is set — draws a 2px red stroke over that node.
fn repaint(
    handle: &MountHandle,
    renderer: &Rc<RefCell<Renderer<CanvasBackend>>>,
    last_ops: &Rc<RefCell<Option<Vec<PaintOp>>>>,
    highlight_index: &Rc<RefCell<Option<usize>>>,
) -> LayoutTree {
    let (new_lt, mut ops) = {
        let r = renderer.borrow();
        let snapshot = handle.snapshot();
        let new_lt = renderer::layout(&snapshot, r.viewport);
        let ops = renderer::paint(&new_lt, &r.theme);
        (new_lt, ops)
    };
    if let Some(idx) = *highlight_index.borrow() {
        if let Some(node) = new_lt.nodes.get(idx) {
            ops.push(PaintOp::StrokeRect {
                rect: node.rect,
                color: crate::renderer::Color {
                    r: 0xff,
                    g: 0x33,
                    b: 0x33,
                    a: 0xff,
                },
                width: 2.0,
            });
        }
    }
    renderer.borrow_mut().backend.execute(&ops);
    *last_ops.borrow_mut() = Some(ops);
    new_lt
}

fn install_click_listener(
    canvas: &web_sys::HtmlCanvasElement,
    handle: Rc<MountHandle>,
    renderer: Rc<RefCell<Renderer<CanvasBackend>>>,
    layout_tree: Rc<RefCell<LayoutTree>>,
    last_ops: Rc<RefCell<Option<Vec<PaintOp>>>>,
    highlight_index: Rc<RefCell<Option<usize>>>,
) -> Result<(), JsValue> {
    let canvas_for_rect = canvas.clone();
    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        // Convert viewport-space mouse coords into canvas-space
        // (CSS pixels — the Renderer works in CSS pixels after the
        // `ctx.scale(dpr, dpr)` at mount time).
        let rect = canvas_for_rect.get_bounding_client_rect();
        let x = e.client_x() as f32 - rect.left() as f32;
        let y = e.client_y() as f32 - rect.top() as f32;

        // Extract + clone the callback first so we're not holding the
        // layout borrow when it fires (the callback mutates runtime
        // state, and the repaint below needs an exclusive borrow).
        let cb = layout_tree
            .borrow()
            .hit_test_on_click(Point { x, y })
            .cloned();
        let Some(cb) = cb else { return };
        cb.call(());

        if handle.flush() {
            *layout_tree.borrow_mut() = repaint(&handle, &renderer, &last_ops, &highlight_index);
        }
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref())?;
    // The closure is owned by JS from now on — forget the Rust-side
    // handle so it doesn't drop at the end of this function.
    click_cb.forget();
    Ok(())
}
// CODEGEN-END

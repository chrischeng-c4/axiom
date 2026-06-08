// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
// CODEGEN-BEGIN
//! WebGPU event loop — wires a mounted React component to the shared
//! `cclab-grid-wasm` renderer handle.
//!
//! The layout/paint path mirrors `canvas_app`: React produces an `Element`
//! tree, Jet lays it out and emits `PaintOp`s, then `WebGpuBackend` lowers
//! rect fills into the packed `Float32Array` consumed by
//! `RendererHandle.renderFrame`.

#![cfg(all(feature = "webgpu-app", target_arch = "wasm32"))]

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::renderer::{
    self, LayoutTree, PaintBackend, PaintOp, Point, Renderer, Theme, Viewport, WebGpuBackend,
};
use crate::Component;

use super::{mount, MountHandle};

/// JS-owned WebGPU app handle. Dropping the Rust-side handle would release
/// the JS value, but an explicit `destroy()` gives browser callers the same
/// lifecycle shape as `cclab-grid-wasm::RendererHandle`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[wasm_bindgen]
pub struct JetWebGpuApp {
    grid_handle: JsValue,
    status: js_sys::Object,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
#[wasm_bindgen]
impl JetWebGpuApp {
    pub fn destroy(self) -> Result<(), JsValue> {
        set_status_str(&self.status, "phase", "destroyed");
        call_method0(&self.grid_handle, "destroy")
    }

    pub fn status(&self) -> JsValue {
        self.status.clone().into()
    }
}

/// Mount `component` on `#<canvas_id>` and initialize the shared WebGPU
/// renderer asynchronously. The returned Promise resolves to `JetWebGpuApp`
/// after the adapter/device is ready and the first frame has rendered.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-react.md#schema
pub fn run(canvas_id: &str, component: Component) -> Result<js_sys::Promise, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas element not found"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("element is not an HTMLCanvasElement"))?;

    let dpr = window.device_pixel_ratio() as f32;
    let css_w = canvas.client_width().max(1) as f32;
    let css_h = canvas.client_height().max(1) as f32;
    canvas.set_width((css_w * dpr) as u32);
    canvas.set_height((css_h * dpr) as u32);
    let status = init_status(&window);
    set_status_str(&status, "phase", "initializing");
    set_status_num(&status, "frames", 0.0);
    set_status_num(&status, "lastCellCount", 0.0);
    set_status_num(&status, "lastTextRunCount", 0.0);
    set_status_num(&status, "lastTextGlyphCount", 0.0);
    set_status_num(&status, "lastUnsupportedCount", 0.0);
    set_status_str(&status, "bridgeMode", "uninitialized");

    let renderer = Rc::new(RefCell::new(Renderer::new(
        Viewport {
            width: css_w,
            height: css_h,
            dpr,
        },
        Theme::default(),
        WebGpuBackend::new(),
    )));
    let handle = Rc::new(mount(component));
    let canvas_for_init = canvas.clone();

    Ok(wasm_bindgen_futures::future_to_promise(async move {
        let grid_handle =
            match JsFuture::from(cclab_grid_wasm::init_renderer(canvas_for_init, dpr)).await {
                Ok(handle) => handle,
                Err(e) => {
                    record_error(&status, &e);
                    return Err(e);
                }
            };
        set_status_str(&status, "phase", "renderer-ready");
        let last_ops: Rc<RefCell<Option<Vec<PaintOp>>>> = Rc::new(RefCell::new(None));
        let layout_tree: Rc<RefCell<LayoutTree>> = Rc::new(RefCell::new(repaint(
            &grid_handle,
            &handle,
            &renderer,
            &last_ops,
            &status,
        )?));

        install_click_listener(
            &canvas,
            grid_handle.clone(),
            handle,
            renderer,
            layout_tree,
            last_ops,
            status.clone(),
        )?;
        set_status_str(&status, "phase", "mounted");

        Ok(JsValue::from(JetWebGpuApp {
            grid_handle,
            status,
        }))
    }))
}

fn repaint(
    grid_handle: &JsValue,
    handle: &MountHandle,
    renderer: &Rc<RefCell<Renderer<WebGpuBackend>>>,
    last_ops: &Rc<RefCell<Option<Vec<PaintOp>>>>,
    status: &js_sys::Object,
) -> Result<LayoutTree, JsValue> {
    let (new_lt, ops) = {
        let r = renderer.borrow();
        let snapshot = handle.snapshot();
        let new_lt = renderer::layout(&snapshot, r.viewport);
        let ops = renderer::paint(&new_lt, &r.theme);
        (new_lt, ops)
    };

    let (cells, text_runs, cell_count, text_run_count, unsupported_count) = {
        let mut r = renderer.borrow_mut();
        r.backend.execute(&ops);
        let frame = r.backend.last_frame().clone();
        (
            frame.to_float32_array(),
            frame.to_js_text_runs(),
            frame.cells.len(),
            frame.text_runs.len(),
            frame.unsupported.len(),
        )
    };
    let bridge_mode = call_render_frame(grid_handle, &cells, &text_runs)?;
    let text_glyph_count = read_text_glyph_count(grid_handle);
    record_frame(
        status,
        bridge_mode,
        cell_count,
        text_run_count,
        text_glyph_count,
        unsupported_count,
    );
    *last_ops.borrow_mut() = Some(ops);
    Ok(new_lt)
}

/// Read `RendererHandle.lastTextGlyphCount()` via JS reflection. Used
/// to mirror the wasm-side glyph count into `window.__jet_webgpu_status`
/// so the browser e2e (T8) can distinguish encode-fired-empty from
/// encode-fired-with-glyphs. Returns 0 if the method is missing
/// (e.g. cells-only fallback path or older bridge build). Slice #2191.
fn read_text_glyph_count(grid_handle: &JsValue) -> u32 {
    let Ok(method_value) =
        js_sys::Reflect::get(grid_handle, &JsValue::from_str("lastTextGlyphCount"))
    else {
        return 0;
    };
    let Ok(method) = method_value.dyn_into::<js_sys::Function>() else {
        return 0;
    };
    let Ok(result) = method.call0(grid_handle) else {
        return 0;
    };
    result.as_f64().map(|n| n as u32).unwrap_or(0)
}

fn install_click_listener(
    canvas: &web_sys::HtmlCanvasElement,
    grid_handle: JsValue,
    handle: Rc<MountHandle>,
    renderer: Rc<RefCell<Renderer<WebGpuBackend>>>,
    layout_tree: Rc<RefCell<LayoutTree>>,
    last_ops: Rc<RefCell<Option<Vec<PaintOp>>>>,
    status: js_sys::Object,
) -> Result<(), JsValue> {
    let canvas_for_rect = canvas.clone();
    let click_cb = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        let rect = canvas_for_rect.get_bounding_client_rect();
        let x = e.client_x() as f32 - rect.left() as f32;
        let y = e.client_y() as f32 - rect.top() as f32;

        let cb = layout_tree
            .borrow()
            .hit_test_on_click(Point { x, y })
            .cloned();
        let Some(cb) = cb else { return };
        cb.call(());

        if handle.flush() {
            match repaint(&grid_handle, &handle, &renderer, &last_ops, &status) {
                Ok(new_tree) => *layout_tree.borrow_mut() = new_tree,
                Err(e) => record_error(&status, &e),
            }
        }
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("click", click_cb.as_ref().unchecked_ref())?;
    click_cb.forget();
    Ok(())
}

fn call_render_frame(
    grid_handle: &JsValue,
    cells: &js_sys::Float32Array,
    text_runs: &js_sys::Array,
) -> Result<&'static str, JsValue> {
    if let Ok(method) = js_sys::Reflect::get(grid_handle, &JsValue::from_str("renderFrameWithText"))
        .and_then(|m| {
            m.dyn_into::<js_sys::Function>().map_err(|_| {
                JsValue::from_str("RendererHandle.renderFrameWithText is not a function")
            })
        })
    {
        method.call2(grid_handle, cells, text_runs)?;
        return Ok("text");
    }

    let method = js_sys::Reflect::get(grid_handle, &JsValue::from_str("renderFrame"))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("RendererHandle.renderFrame is not a function"))?;
    method.call1(grid_handle, cells)?;
    Ok("cells")
}

fn call_method0(target: &JsValue, name: &str) -> Result<(), JsValue> {
    let method = js_sys::Reflect::get(target, &JsValue::from_str(name))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("method is not a function"))?;
    method.call0(target)?;
    Ok(())
}

const STATUS_KEY: &str = "__jet_webgpu_status";

fn init_status(window: &web_sys::Window) -> js_sys::Object {
    let status = js_sys::Object::new();
    let _ = js_sys::Reflect::set(window, &JsValue::from_str(STATUS_KEY), status.as_ref());
    status
}

fn record_frame(
    status: &js_sys::Object,
    bridge_mode: &str,
    cell_count: usize,
    text_run_count: usize,
    text_glyph_count: u32,
    unsupported_count: usize,
) {
    let frames = js_sys::Reflect::get(status, &JsValue::from_str("frames"))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    set_status_str(status, "phase", "rendered");
    set_status_str(status, "bridgeMode", bridge_mode);
    set_status_num(status, "frames", frames + 1.0);
    set_status_num(status, "lastCellCount", cell_count as f64);
    set_status_num(status, "lastTextRunCount", text_run_count as f64);
    set_status_num(status, "lastTextGlyphCount", text_glyph_count as f64);
    set_status_num(status, "lastUnsupportedCount", unsupported_count as f64);
}

fn record_error(status: &js_sys::Object, error: &JsValue) {
    set_status_str(status, "phase", "error");
    set_status_value(status, "error", error);
    web_sys::console::error_1(error);
}

fn set_status_str(status: &js_sys::Object, key: &str, value: &str) {
    set_status_value(status, key, &JsValue::from_str(value));
}

fn set_status_num(status: &js_sys::Object, key: &str, value: f64) {
    set_status_value(status, key, &JsValue::from_f64(value));
}

fn set_status_value(status: &js_sys::Object, key: &str, value: &JsValue) {
    let _ = js_sys::Reflect::set(status, &JsValue::from_str(key), value);
}
// CODEGEN-END

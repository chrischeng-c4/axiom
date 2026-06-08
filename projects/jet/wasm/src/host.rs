// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
// CODEGEN-BEGIN
//! Thin browser host-capability bridge for Jet WASM.
//!
//! This module is the Rust side of `dist/jet-host.js`. It gives the
//! TSX -> Rust lowering target a stable API for browser capabilities
//! that WebAssembly cannot invoke directly. The JS side stays thin:
//! bootstrap + host adapters only. App/domain/render state belongs in
//! Rust/WASM.

#![cfg(feature = "host-bridge")]

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(raw_module = "./jet-host.js")]
extern "C" {
    #[wasm_bindgen(js_name = jet_bridge_fetch)]
    fn js_fetch(input: &str, init: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_name = jet_bridge_console_log)]
    fn js_console_log(value: JsValue);
    #[wasm_bindgen(js_name = jet_bridge_console_warn)]
    fn js_console_warn(value: JsValue);
    #[wasm_bindgen(js_name = jet_bridge_console_error)]
    fn js_console_error(value: JsValue);
    #[wasm_bindgen(js_name = jet_bridge_console_info)]
    fn js_console_info(value: JsValue);

    #[wasm_bindgen(js_name = jet_bridge_local_storage_get_item)]
    fn js_local_storage_get_item(key: &str) -> Option<String>;
    #[wasm_bindgen(js_name = jet_bridge_local_storage_set_item)]
    fn js_local_storage_set_item(key: &str, value: &str);
    #[wasm_bindgen(js_name = jet_bridge_local_storage_remove_item)]
    fn js_local_storage_remove_item(key: &str);
    #[wasm_bindgen(js_name = jet_bridge_local_storage_clear)]
    fn js_local_storage_clear();
}

/// Start a browser fetch through the host adapter.
///
/// The returned `Promise` intentionally stays a JS promise at this
/// layer. Typed API clients can wrap it with `wasm_bindgen_futures`
/// and decode JSON/bytes into compact Rust structs at the edge.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn fetch(input: &str) -> js_sys::Promise {
    js_fetch(input, JsValue::UNDEFINED)
}

/// Start a browser fetch with a JS `RequestInit` object.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn fetch_with_init(input: &str, init: JsValue) -> js_sys::Promise {
    js_fetch(input, init)
}

/// Fetch a URL and decode the response body with `Response.json()`.
///
/// The browser capability still enters through `jet-host.js`, but
/// response validation and JSON promise handling stay in Rust/WASM.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[cfg(target_arch = "wasm32")]
pub async fn fetch_json(input: &str) -> Result<JsValue, JsValue> {
    fetch_json_with_init(input, JsValue::UNDEFINED).await
}

/// Fetch with a caller-provided `RequestInit` object and decode JSON.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[cfg(target_arch = "wasm32")]
pub async fn fetch_json_with_init(input: &str, init: JsValue) -> Result<JsValue, JsValue> {
    let response = fetch_response(input, init).await?;
    let json = response.json()?;
    JsFuture::from(json).await
}

/// Fetch a URL and decode the response body with `Response.text()`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[cfg(target_arch = "wasm32")]
pub async fn fetch_text(input: &str) -> Result<String, JsValue> {
    fetch_text_with_init(input, JsValue::UNDEFINED).await
}

/// Fetch with a caller-provided `RequestInit` object and decode text.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
#[cfg(target_arch = "wasm32")]
pub async fn fetch_text_with_init(input: &str, init: JsValue) -> Result<String, JsValue> {
    let response = fetch_response(input, init).await?;
    let text = JsFuture::from(response.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_response(input: &str, init: JsValue) -> Result<web_sys::Response, JsValue> {
    let value = JsFuture::from(fetch_with_init(input, init)).await?;
    let response = value
        .dyn_into::<web_sys::Response>()
        .map_err(|_| JsValue::from_str("jet host fetch did not return a Response"))?;
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "jet host fetch failed: HTTP {}",
            response.status()
        )));
    }
    Ok(response)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn console_log(message: &str) {
    js_console_log(JsValue::from_str(message));
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn console_warn(message: &str) {
    js_console_warn(JsValue::from_str(message));
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn console_error(message: &str) {
    js_console_error(JsValue::from_str(message));
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn console_info(message: &str) {
    js_console_info(JsValue::from_str(message));
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn local_storage_get_item(key: &str) -> Option<String> {
    js_local_storage_get_item(key)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn local_storage_set_item(key: &str, value: &str) {
    js_local_storage_set_item(key, value);
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn local_storage_remove_item(key: &str) {
    js_local_storage_remove_item(key);
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src.md#schema
pub fn local_storage_clear() {
    js_local_storage_clear();
}
// CODEGEN-END

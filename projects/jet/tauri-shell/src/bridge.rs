// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
// CODEGEN-BEGIN
//! Local-process IPC bridge between the Tauri shell and the Cue
//! runtime. Wraps the exact JSON-RPC shape that the web HTTP
//! backend serves so app code can call `useBackend().query("...")`
//! without caring whether the response came from `fetch()` or
//! `tauri::ipc::invoke`.
//!
//! @spec `.score/tech_design/projects/jet/logic/multi-target/desktop-runtime.md`
//!     §"Slice 3 — local backend bridge"
//! @issue #1242 — Slice 2 (trait + error enum); Slice 3 wires the
//!     `tauri::ipc::Invoke` adapter behind the `tauri` feature.

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

/// Pinned, boxed future the trait returns. We avoid pulling in
/// `futures` here so the public surface stays minimal and
/// substrate-side users can implement the trait without an extra
/// dep.
pub type BridgeFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// IPC bridge contract. The substrate calls `handle("entity.query",
/// {...params...})` and gets back the same JSON shape the web
/// backend produces over HTTP. Cue's concrete impl wraps the
/// in-process Cue runtime; tests can supply a `MockBridge`.
pub trait BackendBridge: Send + Sync {
    fn handle<'a>(
        &'a self,
        method: &'a str,
        params: serde_json::Value,
    ) -> BridgeFuture<'a, Result<serde_json::Value, BridgeError>>;
}

/// Errors the bridge can surface back to the app.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("method {0:?} not registered")]
    NotFound(String),
    #[error("invalid params for {method:?}: {message}")]
    InvalidParams { method: String, message: String },
    #[error("internal error: {0}")]
    Internal(String),
}

// ── Wire envelope (Slice 3a) ────────────────────────────────────────
//
// JSON-RPC-2.0-shaped request/response envelope. Both the web HTTP
// backend and the Tauri IPC adapter (Slice 3b) decode wire bytes
// into `RpcRequest`, hand it to `dispatch_envelope`, and serialize
// the resulting `RpcResponse` back out. The envelope IS the
// transport-compatibility contract from #1242 AC: web `fetch()`
// and tauri `invoke()` cross the same JSON shape.

/// JSON-RPC-shaped request envelope. The outer `jsonrpc` / `id`
/// fields the spec defines are optional on the wire and the
/// dispatcher does not interpret them — keeping the on-wire
/// shape minimal lets both transports add their own framing
/// (HTTP correlates by URL path; tauri correlates by invoke
/// promise) without fighting an envelope-level id.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcRequest {
    pub method: String,
    /// Parameters object. Defaults to `null` when the wire payload
    /// omits the field — matches JSON-RPC's "no-params" convention.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl RpcRequest {
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC-shaped response envelope. Exactly one of `result` /
/// `error` is populated; the constructors enforce that, and
/// `Serialize` skips the unset field so the wire shape stays
/// canonical.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl RpcResponse {
    pub fn ok(result: serde_json::Value) -> Self {
        Self {
            result: Some(result),
            error: None,
        }
    }

    pub fn err(error: RpcError) -> Self {
        Self {
            result: None,
            error: Some(error),
        }
    }
}

/// Reserved JSON-RPC-2.0 error codes used by `BridgeError`'s
/// `From` impl. Kept as `pub const`s so callers (test fixtures,
/// future HTTP adapter) can match on them without re-deriving.
pub const RPC_METHOD_NOT_FOUND: i32 = -32601;
pub const RPC_INVALID_PARAMS: i32 = -32602;
pub const RPC_INTERNAL: i32 = -32603;

/// Stable JSON-RPC error envelope. The `code` field follows the
/// JSON-RPC-2.0 reserved range; the `message` field carries the
/// `BridgeError::Display` rendering verbatim so the substrate-side
/// log + the app-side error toast see the same string.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
impl From<BridgeError> for RpcError {
    fn from(e: BridgeError) -> Self {
        let code = match &e {
            BridgeError::NotFound(_) => RPC_METHOD_NOT_FOUND,
            BridgeError::InvalidParams { .. } => RPC_INVALID_PARAMS,
            BridgeError::Internal(_) => RPC_INTERNAL,
        };
        Self {
            code,
            message: e.to_string(),
        }
    }
}

/// Drive a `BackendBridge` from a wire envelope. Both the future
/// `tauri::ipc::Invoke` adapter (Slice 3b) and the future HTTP
/// adapter wrap this function — they own only the wire-bytes
/// codec; the dispatch shape is identical, which is what makes
/// the bridge transport-compatible per #1242's third AC.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tauri-shell-src.md#schema
pub async fn dispatch_envelope(bridge: &dyn BackendBridge, request: RpcRequest) -> RpcResponse {
    match bridge.handle(&request.method, request.params).await {
        Ok(v) => RpcResponse::ok(v),
        Err(e) => RpcResponse::err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// In-memory bridge fixture. Maps method names to JSON
    /// responses so tests can drive the substrate side without
    /// any real Cue runtime.
    struct MapBridge {
        routes: HashMap<String, serde_json::Value>,
    }

    impl MapBridge {
        fn new(routes: &[(&str, serde_json::Value)]) -> Self {
            Self {
                routes: routes
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), v.clone()))
                    .collect(),
            }
        }
    }

    impl BackendBridge for MapBridge {
        fn handle<'a>(
            &'a self,
            method: &'a str,
            _params: serde_json::Value,
        ) -> BridgeFuture<'a, Result<serde_json::Value, BridgeError>> {
            let resp = self
                .routes
                .get(method)
                .cloned()
                .ok_or_else(|| BridgeError::NotFound(method.to_string()));
            Box::pin(async move { resp })
        }
    }

    fn block_on<F: Future>(f: F) -> F::Output {
        // Hand-rolled tiny executor — no tokio dep needed for this
        // unit test. Adapted from the std library's `pending!`
        // doc-test pattern.
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::task::{Context, Poll, Wake, Waker};

        struct Noop(AtomicBool);
        impl Wake for Noop {
            fn wake(self: Arc<Self>) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let noop = Arc::new(Noop(AtomicBool::new(false)));
        let waker = Waker::from(noop);
        let mut cx = Context::from_waker(&waker);
        let mut fut = Box::pin(f);
        loop {
            if let Poll::Ready(v) = fut.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    #[test]
    fn map_bridge_returns_registered_response() {
        let b = MapBridge::new(&[("issues.list", serde_json::json!([{"id": 1}]))]);
        let v = block_on(b.handle("issues.list", serde_json::json!({}))).unwrap();
        assert_eq!(v, serde_json::json!([{"id": 1}]));
    }

    #[test]
    fn map_bridge_yields_not_found_for_unknown_method() {
        let b = MapBridge::new(&[]);
        let err = block_on(b.handle("unknown.method", serde_json::json!({}))).unwrap_err();
        match err {
            BridgeError::NotFound(m) => assert_eq!(m, "unknown.method"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn bridge_error_variants_format_with_method_context() {
        let e = BridgeError::InvalidParams {
            method: "x".into(),
            message: "missing field foo".into(),
        };
        assert!(format!("{e}").contains("missing field foo"));
        assert!(format!("{e}").contains("\"x\""));
    }

    // ── envelope tests (Slice 3a) ───────────────────────────────────

    #[test]
    fn rpc_request_deserializes_with_default_params() {
        let req: RpcRequest = serde_json::from_str(r#"{"method":"x"}"#).unwrap();
        assert_eq!(req.method, "x");
        assert_eq!(req.params, serde_json::Value::Null);
    }

    #[test]
    fn rpc_request_round_trip_preserves_params() {
        let req = RpcRequest::new("issues.list", serde_json::json!({"state":"open"}));
        let s = serde_json::to_string(&req).unwrap();
        let back: RpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn rpc_response_ok_omits_error_field() {
        let resp = RpcResponse::ok(serde_json::json!(42));
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(s, r#"{"result":42}"#);
    }

    #[test]
    fn rpc_response_err_omits_result_field() {
        let resp = RpcResponse::err(RpcError {
            code: RPC_METHOD_NOT_FOUND,
            message: "method \"x\" not registered".into(),
        });
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(
            s,
            r#"{"error":{"code":-32601,"message":"method \"x\" not registered"}}"#
        );
    }

    #[test]
    fn bridge_error_not_found_maps_to_method_not_found_code() {
        let rpc: RpcError = BridgeError::NotFound("issues.list".into()).into();
        assert_eq!(rpc.code, RPC_METHOD_NOT_FOUND);
        assert!(rpc.message.contains("issues.list"));
    }

    #[test]
    fn bridge_error_invalid_params_maps_to_invalid_params_code() {
        let rpc: RpcError = BridgeError::InvalidParams {
            method: "issues.create".into(),
            message: "missing title".into(),
        }
        .into();
        assert_eq!(rpc.code, RPC_INVALID_PARAMS);
        assert!(rpc.message.contains("missing title"));
        assert!(rpc.message.contains("issues.create"));
    }

    #[test]
    fn bridge_error_internal_maps_to_internal_code() {
        let rpc: RpcError = BridgeError::Internal("db down".into()).into();
        assert_eq!(rpc.code, RPC_INTERNAL);
        assert!(rpc.message.contains("db down"));
    }

    #[test]
    fn dispatch_envelope_ok_path_returns_result() {
        let b = MapBridge::new(&[("ping", serde_json::json!("pong"))]);
        let req = RpcRequest::new("ping", serde_json::json!({}));
        let resp = block_on(dispatch_envelope(&b, req));
        assert_eq!(resp.result, Some(serde_json::json!("pong")));
        assert!(resp.error.is_none());
    }

    #[test]
    fn dispatch_envelope_unknown_method_returns_error_envelope() {
        let b = MapBridge::new(&[]);
        let req = RpcRequest::new("unknown", serde_json::json!({}));
        let resp = block_on(dispatch_envelope(&b, req));
        assert!(resp.result.is_none());
        let err = resp.error.unwrap();
        assert_eq!(err.code, RPC_METHOD_NOT_FOUND);
        assert!(err.message.contains("unknown"));
    }

    #[test]
    fn dispatch_envelope_response_serializes_canonically() {
        let b = MapBridge::new(&[("ping", serde_json::json!({"pong":true}))]);
        let req = RpcRequest::new("ping", serde_json::json!(null));
        let resp = block_on(dispatch_envelope(&b, req));
        let s = serde_json::to_string(&resp).unwrap();
        assert_eq!(s, r#"{"result":{"pong":true}}"#);
    }
}
// CODEGEN-END

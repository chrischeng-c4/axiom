// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! HTTP request handler for the embedded trace viewer.
//!
//! Routes:
//! - `GET /`          — serves embedded viewer HTML (with inlined JS + CSS)
//! - `GET /trace.json` — manifest JSON
//! - `GET /assets/:id` — zip asset bytes with correct Content-Type
//! - everything else  — 404
//!
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R6
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R12

use crate::trace::manifest::TraceManifest;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;

// ── Embedded assets ──────────────────────────────────────────────────────────

/// Raw HTML template.
const VIEWER_HTML: &str = include_str!("../../assets/trace-viewer/viewer.html");
/// Viewer JavaScript.
const VIEWER_JS: &str = include_str!("../../assets/trace-viewer/viewer.js");
/// Viewer CSS.
const VIEWER_CSS: &str = include_str!("../../assets/trace-viewer/viewer.css");

/// Lazily-built HTML with JS/CSS inlined.
fn build_viewer_html() -> String {
    VIEWER_HTML
        .replace("/* VIEWER_CSS_PLACEHOLDER */", VIEWER_CSS)
        .replace("/* VIEWER_JS_PLACEHOLDER */", VIEWER_JS)
}

// ── App state ────────────────────────────────────────────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
#[derive(Clone)]
pub struct ViewerState {
    /// Path to the `.jet-trace` / `trace.zip` file.
    pub zip_path: Arc<PathBuf>,
    /// Parsed manifest — serves the /trace.json endpoint directly.
    pub manifest: Arc<TraceManifest>,
    /// Pre-built HTML (CSS + JS inlined).
    pub viewer_html: Arc<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
impl ViewerState {
    pub fn new(zip_path: PathBuf, manifest: TraceManifest) -> Self {
        let viewer_html = build_viewer_html();
        Self {
            zip_path: Arc::new(zip_path),
            manifest: Arc::new(manifest),
            viewer_html: Arc::new(viewer_html),
        }
    }
}

// ── Router ───────────────────────────────────────────────────────────────────

/// Build the axum `Router` for the trace viewer HTTP server.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
pub fn build_router(state: ViewerState) -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/trace.json", get(handle_trace_json))
        .route("/assets/{id}", get(handle_asset))
        .fallback(handle_not_found)
        .with_state(state)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Serve the embedded viewer HTML with JS and CSS inlined.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R6
async fn handle_root(State(s): State<ViewerState>) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(s.viewer_html.as_ref().clone()))
        .unwrap()
}

/// Serve the parsed `TraceManifest` as JSON.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
async fn handle_trace_json(State(s): State<ViewerState>) -> impl IntoResponse {
    match serde_json::to_string(s.manifest.as_ref()) {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json; charset=utf-8")
            .body(Body::from(json))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Failed to serialise manifest: {e}")))
            .unwrap(),
    }
}

/// Serve a binary asset from the zip archive.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
async fn handle_asset(
    Path(asset_id): Path<String>,
    State(s): State<ViewerState>,
) -> impl IntoResponse {
    // Look up asset_id in the manifest assets map.
    let zip_entry = match s.manifest.assets.get(&asset_id) {
        Some(e) => e.clone(),
        None => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Asset not found: {asset_id}")))
                .unwrap();
        }
    };

    // Read the asset bytes from the zip file.
    let zip_path = s.zip_path.clone();
    let bytes_result = tokio::task::spawn_blocking(move || {
        crate::trace::archive::read_asset_from_zip(&zip_path, &zip_entry)
    })
    .await;

    match bytes_result {
        Ok(Ok(bytes)) => {
            let content_type = content_type_for_asset_id(&asset_id);
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", content_type)
                .body(Body::from(bytes))
                .unwrap()
        }
        Ok(Err(e)) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Failed to read asset: {e}")))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Task join error: {e}")))
            .unwrap(),
    }
}

async fn handle_not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not found")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Map an asset id to a MIME type based on prefix convention:
/// - `screenshot-*` or `*-screenshot-*` → `image/png`
/// - `dom-*` → `text/html; charset=utf-8`
fn content_type_for_asset_id(id: &str) -> &'static str {
    if id.starts_with("screenshot") || id.contains("-screenshot-") {
        "image/png"
    } else if id.starts_with("dom") {
        "text/html; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}
// CODEGEN-END

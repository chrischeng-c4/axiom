---
id: sdd-ui-viewer
fill_sections: [source, overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ReviewResult Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/ui/viewer/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `start_viewer` | projects/agentic-workflow/src/ui/viewer/mod.rs | function | pub | 30 | start_viewer(change_id: &str, project_root: &StdPath) -> anyhow::Result<ReviewResult> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewResult:
    type: string
    enum: [Approved, ChangesRequested, Cancelled]
    description: |
      Result of a review session.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq]
      variants:
        - { name: Approved, doc: "User approved the proposal." }
        - { name: ChangesRequested, doc: "User requested changes (comments saved)." }
        - { name: Cancelled, doc: "User closed without taking action." }
```

## Source
<!-- type: source lang: rust -->

```rust
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::oneshot;

/// Application state shared across handlers
struct AppState {
    manager: ViewerManager,
    change_id: String,
    project_root: PathBuf,
    shutdown_tx: Option<oneshot::Sender<()>>,
    review_result: ReviewResult,
}

/// Start the plan viewer web server
///
/// This function starts an HTTP server and opens the browser.
/// It blocks until the server is shut down (via close_window API).
/// Returns the review result indicating what action the user took.
pub fn start_viewer(change_id: &str, project_root: &StdPath) -> anyhow::Result<ReviewResult> {
    let manager = ViewerManager::new(change_id, project_root);

    if !manager.change_exists() {
        anyhow::bail!("Change '{}' not found", change_id);
    }

    // Build tokio runtime and run async server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_server(
        manager,
        change_id.to_string(),
        project_root.to_path_buf(),
    ))
}

async fn run_server(
    manager: ViewerManager,
    change_id: String,
    project_root: PathBuf,
) -> anyhow::Result<ReviewResult> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let state = Arc::new(tokio::sync::Mutex::new(AppState {
        manager,
        change_id: change_id.clone(),
        project_root,
        shutdown_tx: Some(shutdown_tx),
        review_result: ReviewResult::Cancelled, // Default if window closed without action
    }));

    let app = Router::new()
        // Serve React SPA at root
        .route("/", get(serve_index))
        // Serve bundled assets (React build output)
        .route("/assets/{*path}", get(serve_assets))
        .route("/static/{*path}", get(serve_assets))
        // Legacy viewer API endpoints
        .route("/api/info", get(api_info))
        .route("/api/files", get(api_list_files))
        .route("/api/files/*path", get(api_load_file))
        .route("/api/annotations", post(api_save_annotation))
        .route(
            "/api/annotations/{id}/resolve",
            post(api_resolve_annotation),
        )
        .route("/api/review/approve", post(api_approve_review))
        .route("/api/review/request-changes", post(api_request_changes))
        .route("/api/review/submit-comments", post(api_submit_comments))
        .route("/api/close", post(api_close_window))
        // SddDataSource API endpoints (R7)
        .route("/api/issues", get(api::api_list_issues))
        .route("/api/issues/{id}", get(api::api_get_issue))
        .route("/api/tech-designs", get(api::api_list_tech_designs))
        .route("/api/tech-designs/{id}", get(api::api_get_tech_design))
        .route("/api/changes", get(api::api_list_changes))
        .route("/api/changes/{id}", get(api::api_get_change))
        .route("/api/lineage/{id}", get(api::api_get_lineage))
        .route("/api/project-info", get(api::api_project_info))
        // SPA fallback — any non-API route returns index.html for client-side routing
        .fallback(get(serve_index))
        .with_state(state.clone());

    // Find available port
    let addr = find_available_port().await?;
    let url = format!("http://{}", addr);

    println!("Starting Plan Viewer at {}", url);
    println!("Press Ctrl+C to stop");

    // Open browser
    if let Err(e) = open::that(&url) {
        eprintln!(
            "Failed to open browser: {}. Please open {} manually.",
            e, url
        );
    }

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
            println!("\nShutting down viewer...");
        })
        .await?;

    // Get the final review result
    let result = state.lock().await.review_result;
    Ok(result)
}

async fn find_available_port() -> anyhow::Result<SocketAddr> {
    // Try ports 3000-3100
    for port in 3000..3100 {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        if tokio::net::TcpListener::bind(addr).await.is_ok() {
            return Ok(addr);
        }
    }
    anyhow::bail!("No available port found in range 3000-3100")
}

// ============================================================================
// Static file handlers — React SPA build output
// ============================================================================

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
}

/// Serve bundled static assets from the checked-in viewer bundle.
async fn serve_assets(Path(path): Path<String>) -> impl IntoResponse {
    match path.as_str() {
        "app.js" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            include_str!("assets/app.js"),
        )
            .into_response(),
        "styles.css" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/css")],
            include_str!("assets/styles.css"),
        )
            .into_response(),
        "highlight.min.js" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            include_str!("assets/highlight.min.js"),
        )
            .into_response(),
        "highlight.min.css" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/css")],
            include_str!("assets/highlight.min.css"),
        )
            .into_response(),
        "mermaid.min.js" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            include_str!("assets/mermaid.min.js"),
        )
            .into_response(),
        _ => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

// ============================================================================
// API handlers
// ============================================================================

#[derive(serde::Serialize)]
struct InfoResponse {
    change_id: String,
    files: Vec<manager::FileInfo>,
}

async fn api_info(State(state): State<Arc<tokio::sync::Mutex<AppState>>>) -> Json<InfoResponse> {
    let state = state.lock().await;
    Json(InfoResponse {
        change_id: state.change_id.clone(),
        files: state.manager.list_files(),
    })
}

async fn api_list_files(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> Json<Vec<manager::FileInfo>> {
    let state = state.lock().await;
    Json(state.manager.list_files())
}

async fn api_load_file(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Path(path): Path<String>,
) -> Response {
    // Remove leading slash if present (wildcard captures include it)
    let filename = path.trim_start_matches('/');
    let state = state.lock().await;
    match state.manager.load_file(filename) {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct SaveAnnotationRequest {
    file: String,
    section_id: String,
    content: String,
}

async fn api_save_annotation(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(req): Json<SaveAnnotationRequest>,
) -> Response {
    use crate::models::{get_author_name, Annotation};

    let state = state.lock().await;
    let author = get_author_name();
    let annotation = Annotation::new(&req.file, &req.section_id, &req.content, author);

    match state.manager.load_annotations() {
        Ok(mut store) => {
            let annotation_clone = annotation.clone();
            store.add(annotation);
            match state.manager.save_annotations(&store) {
                Ok(_) => Json(annotation_clone).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_resolve_annotation(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Path(id): Path<String>,
) -> Response {
    let state = state.lock().await;

    match state.manager.load_annotations() {
        Ok(mut store) => {
            if let Err(e) = store.resolve(&id) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response();
            }

            match state.manager.save_annotations(&store) {
                Ok(_) => {
                    let annotation = store.find(&id).cloned();
                    Json(annotation).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn api_approve_review(State(state): State<Arc<tokio::sync::Mutex<AppState>>>) -> Response {
    let mut state = state.lock().await;

    match state.manager.update_phase("complete") {
        Ok(_) => {
            // Set review result and trigger shutdown
            state.review_result = ReviewResult::Approved;
            if let Some(tx) = state.shutdown_tx.take() {
                let _ = tx.send(());
            }
            Json(serde_json::json!({
                "action": "approve_review",
                "status": "success",
                "message": "Review approved. Phase updated to complete."
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "approve_review",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

async fn api_request_changes(State(state): State<Arc<tokio::sync::Mutex<AppState>>>) -> Response {
    let mut state = state.lock().await;

    // Update phase to indicate changes were requested
    if let Err(e) = state.manager.update_phase("changes_requested") {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response();
    }

    match state.manager.load_annotations() {
        Ok(store) => {
            let unresolved_count = store.unresolved_count();
            // Set review result and trigger shutdown
            state.review_result = ReviewResult::ChangesRequested;
            if let Some(tx) = state.shutdown_tx.take() {
                let _ = tx.send(());
            }
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "success",
                "message": format!("Changes requested with {} comment(s). Phase updated.", unresolved_count)
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "action": "request_changes",
                "status": "error",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

async fn api_submit_comments() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "action": "submit_comments",
        "status": "success",
        "message": "Comments saved."
    }))
}

async fn api_close_window(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> Json<serde_json::Value> {
    // Trigger shutdown
    let mut state = state.lock().await;
    if let Some(tx) = state.shutdown_tx.take() {
        let _ = tx.send(());
    }

    Json(serde_json::json!({
        "action": "close_window",
        "status": "success"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_exported() {
        // Test that slugify is accessible from this module
        assert_eq!(slugify("Test Heading"), "test-heading");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/ui/viewer/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - ReviewResult
    description: |
      Codegen replaces the enum declaration only.
  - path: projects/agentic-workflow/src/ui/viewer/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate viewer server wiring, route handlers, browser launch, and
      tests from the source section.
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->
**Verdict:** approved

- [overview] Single unit-variant enum.
- [schema] Standard 3-variant enum, no serde derives.
- [changes] Standard split.

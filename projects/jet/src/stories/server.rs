// HANDWRITE-BEGIN gap="missing-generator:logic:d0ce83ef" tracker="pending-tracker" reason="start_stories_workbench(root, host, port): discover StoryIndex, build a dev-server variant (reuse dev_server substrate) with routes for the manager, isolated preview, and module serving; build per-story entry via module graph."
//! The `jet stories` native workbench server (B2).
//!
//! A small, focused axum server — deliberately *not* a fork of
//! [`crate::dev_server`] — that serves three things:
//!
//! 1. `GET /` (and `/__jet_stories_manager`) → the manager shell
//!    ([`manager::render_manager_html`]): a sidebar tree of discovered stories,
//!    a toolbar, and an `<iframe>` showing the selected story's preview.
//! 2. `GET /__jet_stories_preview/{story_id}` → the *isolated* preview document
//!    ([`manager::render_preview_html`]) for one story — it mounts only that
//!    story's component, with no app router/shell. An unknown id → 404.
//! 3. `GET /{module path}` → on-demand transform of a `.ts/.tsx/.js/.jsx`
//!    source file to browser JS so the preview's `import` of the story module
//!    resolves. This reuses the same `crate::transform::*` pipeline the dev
//!    server uses for on-demand module serving.
//!
//! HMR is out of scope (B2b / #176): navigation does a full preview reload.
//! Controls are out of scope (B3).
//!
//! ## Bare-import resolution (deferred)
//! Local *relative* imports (`./Button`) are resolved + transformed by the
//! module route. The React runtime itself is provided to the preview via an
//! importmap to esm.sh (see [`manager::render_preview_html`]). Full bare-import
//! resolution against local `node_modules` (arbitrary third-party packages)
//! is intentionally NOT wired here.
//! TODO(#174 follow-up): reuse the dev server's `node_modules` resolution +
//! prebundle path so bare specifiers other than React resolve locally.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use axum::{
    extract::{ws::WebSocket, Path as AxumPath, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};

use super::hmr::{self, StoriesHmrManager, STORIES_HMR_ROUTE};
use super::manager;
use super::{discover, StoryEntry, StoryIndex};
use crate::dev_server::module_graph::ModuleGraph;
use crate::dev_server::watcher::FileWatcher;

/// Manager shell route (alias of `/`).
pub const MANAGER_PREFIX: &str = "/__jet_stories_manager";

/// Shared router state: the discovered index + the project root (for resolving
/// + transforming module sources on demand), plus the HMR broadcast hub and the
/// served-module import graph (B2b/#176).
#[derive(Clone)]
struct WorkbenchState {
    index: Arc<StoryIndex>,
    root: Arc<PathBuf>,
    /// Broadcast hub the preview-frame HMR clients subscribe to.
    hmr: StoriesHmrManager,
    /// Import graph the module route populates lazily as it serves modules, so
    /// [`super::hmr::affected_modules`] can walk a changed module's importers.
    graph: Arc<RwLock<ModuleGraph>>,
}

/// Discover stories under `root`, build the router, bind `host:port`, and serve
/// until the process is stopped.
pub async fn start_stories_workbench(root: &Path, host: String, port: u16) -> Result<()> {
    let root = root.to_path_buf();
    let index = discover(&root);

    eprintln!(
        "[jet stories] discovered {} stories across {} files",
        index.stories.len(),
        index.metas.len()
    );
    for diag in &index.diagnostics {
        eprintln!("[jet stories] {diag}");
    }

    // B2b/#176: a shared HMR hub + import graph, wired to a file watcher so a
    // story/component edit hot-updates ONLY the preview frame.
    let hmr = StoriesHmrManager::new();
    let graph = Arc::new(RwLock::new(ModuleGraph::new()));

    // Hold the watcher for the server's lifetime — dropping it stops the notify
    // backend. A failed watcher must NOT abort the workbench: the manager +
    // preview still serve, just without live reload.
    let _watcher: Option<FileWatcher> = match hmr::spawn_watcher(&root, graph.clone(), hmr.clone())
    {
        Ok(w) => Some(w),
        Err(err) => {
            eprintln!("[jet stories] file watcher unavailable, HMR disabled: {err}");
            None
        }
    };

    let app = build_router_with(index, root, hmr, graph);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .with_context(|| format!("invalid host:port {host}:{port}"))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    let actual = listener.local_addr()?;
    eprintln!("[jet stories] workbench listening on http://{actual}");

    axum::serve(listener, app)
        .await
        .context("jet stories server error")?;
    // Keep the watcher alive until the server exits.
    drop(_watcher);
    Ok(())
}

/// Build the workbench router (factored out so tests can drive routes without
/// binding a port — via `tower::ServiceExt::oneshot` or a `127.0.0.1:0` bind).
///
/// Constructs a fresh HMR hub + import graph; for the live workbench
/// [`start_stories_workbench`] uses [`build_router_with`] to share the hub with
/// its file watcher.
pub fn build_router(index: StoryIndex, root: PathBuf) -> Router {
    build_router_with(
        index,
        root,
        StoriesHmrManager::new(),
        Arc::new(RwLock::new(ModuleGraph::new())),
    )
}

/// Build the router over an explicit HMR hub + import graph (so the watcher and
/// the WS route share one broadcast channel).
fn build_router_with(
    index: StoryIndex,
    root: PathBuf,
    hmr: StoriesHmrManager,
    graph: Arc<RwLock<ModuleGraph>>,
) -> Router {
    let state = WorkbenchState {
        index: Arc::new(index),
        root: Arc::new(root),
        hmr,
        graph,
    };

    Router::new()
        .route("/", get(manager_handler))
        .route(MANAGER_PREFIX, get(manager_handler))
        .route("/__jet_stories_preview/{story_id}", get(preview_handler))
        // Preview-frame HMR WebSocket (B2b/#176).
        .route(STORIES_HMR_ROUTE, get(stories_hmr_handler))
        // Catch-all for module + static requests the preview imports.
        .route("/{*path}", get(module_handler))
        .with_state(state)
}

/// `GET /` / `GET /__jet_stories_manager` → the manager shell.
async fn manager_handler(State(state): State<WorkbenchState>) -> Response {
    let html = manager::render_manager_html(&state.index, None);
    html_response(html)
}

/// `GET /__jet_stories_preview/{story_id}` → isolated single-story preview.
async fn preview_handler(
    State(state): State<WorkbenchState>,
    AxumPath(story_id): AxumPath<String>,
) -> Response {
    // Empty id (the `/__jet_stories_preview/` empty-state link) → empty preview.
    if story_id.is_empty() {
        return html_response(manager::render_empty_preview_html());
    }

    let Some(story) = state.index.stories.iter().find(|s| s.id == story_id) else {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: unknown story id '{story_id}'"),
        )
            .into_response();
    };

    let module_url = module_url_for(&state.root, &story.file);
    let html = manager::render_preview_html(story, &module_url);
    html_response(html)
}

/// `GET /__jet_stories_hmr` → upgrade to the preview-frame HMR WebSocket.
///
/// Each connected preview frame subscribes to the shared [`StoriesHmrManager`];
/// the file watcher broadcasts [`super::hmr::StoriesHmrMessage`]s which this
/// handler forwards as JSON. The manager shell never connects here, so it never
/// reloads (B2b/#176).
async fn stories_hmr_handler(ws: WebSocketUpgrade, State(state): State<WorkbenchState>) -> Response {
    ws.on_upgrade(move |socket| stories_hmr_socket(socket, state.hmr.clone()))
}

/// Pump broadcast HMR messages to one connected preview frame until it closes.
async fn stories_hmr_socket(socket: WebSocket, hmr: StoriesHmrManager) {
    use axum::extract::ws::Message;

    let (mut sender, mut receiver) = socket.split();
    let mut rx = hmr.subscribe();

    // Greet the client so it can confirm the channel before any edits arrive.
    let _ = sender
        .send(Message::Text(
            super::hmr::StoriesHmrMessage::Connected.to_json().into(),
        ))
        .await;

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(Message::Text(msg.to_json().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Drain inbound frames so the socket stays healthy; close ends the loop.
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

/// `GET /{path}` → transform + serve a local `.ts/.tsx/.js/.jsx` module (so the
/// preview's `import` of the story file resolves), or 404.
async fn module_handler(
    State(state): State<WorkbenchState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    // Reject `..` traversal so a request can't escape the project root.
    if path.split('/').any(|seg| seg == "..") {
        return (StatusCode::BAD_REQUEST, "jet stories: invalid path").into_response();
    }

    let file_path = state.root.join(&path);
    let ext = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext {
        "ts" | "tsx" | "js" | "jsx" => {
            // B2b/#176: record this module's relative-import edges in the shared
            // graph BEFORE serving, so a later edit to an imported component can
            // walk back to the importing story (`affected_modules`). Best-effort:
            // failure to read/parse just means a thinner graph, never a 500.
            register_module_imports(&state, &file_path, &path);
            serve_module(&file_path, &path).await
        }
        _ => (
            StatusCode::NOT_FOUND,
            format!("jet stories: not found '{path}'"),
        )
            .into_response(),
    }
}

/// B2b/#176: record `request_path`'s relative-import edges in the shared graph.
///
/// Reads the (untransformed) source, extracts import specifiers via the dev
/// server's [`crate::dev_server::source_analysis::extract_imports_from_source`],
/// resolves the *relative* ones (`./`, `../`) against the module's own URL to
/// root-relative URLs, and registers the edges. Bare specifiers (`react`, etc.)
/// are skipped — they're not part of the served-module invalidation graph.
///
/// Best-effort: any read/parse failure leaves the graph thinner but never
/// affects serving (the caller ignores the outcome).
fn register_module_imports(state: &WorkbenchState, file_path: &Path, request_path: &str) {
    let Ok(source) = std::fs::read_to_string(file_path) else {
        return;
    };
    let module_url = {
        let mut u = String::from("/");
        u.push_str(request_path.trim_start_matches('/'));
        u
    };
    let specifiers = crate::dev_server::source_analysis::extract_imports_from_source(&source);
    let resolved: Vec<String> = specifiers
        .iter()
        .filter_map(|spec| resolve_relative_import(&module_url, spec))
        .collect();

    hmr::register_served_module(&state.graph, &module_url, file_path, &resolved);
}

/// Resolve a relative import specifier (`./Button`, `../lib/x`) against the
/// importing module's root-relative URL, yielding a root-relative URL. Returns
/// `None` for bare specifiers (no leading `.`).
///
/// Extensionless relative imports are left extensionless here; the invalidation
/// walk keys on whatever URL the preview actually requests, and the watcher
/// emits the on-disk path's URL, so a follow-up could normalize extensions. For
/// the common case (stories import a sibling `./Button` and the watcher fires on
/// `Button.tsx`) this thin resolution is enough to link the two when the story
/// imports with the explicit extension; without it, `affected_modules` falls
/// back to the changed module alone (still a correct, if narrower, update).
/// TODO(#176 follow-up): probe `.tsx/.ts/.jsx/.js/index.*` like the module route
/// so extensionless relative imports resolve to the served URL.
fn resolve_relative_import(importer_url: &str, spec: &str) -> Option<String> {
    if !spec.starts_with('.') {
        return None;
    }
    // Base directory = importer URL minus its filename.
    let base_dir = match importer_url.rsplit_once('/') {
        Some((dir, _file)) => dir,
        None => "",
    };
    let mut segments: Vec<&str> = base_dir.split('/').filter(|s| !s.is_empty()).collect();
    for part in spec.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }
    Some(format!("/{}", segments.join("/")))
}

/// Transform a single source file to browser JS and serve it.
///
/// Reuses the same per-extension transform entrypoints the dev server uses for
/// on-demand module serving (`transform_tsx` / `transform_typescript` /
/// `transform_jsx`; `.js` is served as-is).
async fn serve_module(file_path: &Path, request_path: &str) -> Response {
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(err) => {
            // A missing module is a 404; any other read failure is surfaced as
            // a 500 with the path so the failure isn't silently swallowed.
            if err.kind() == std::io::ErrorKind::NotFound {
                return (
                    StatusCode::NOT_FOUND,
                    format!("jet stories: not found '{request_path}'"),
                )
                    .into_response();
            }
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("jet stories: failed to read '{request_path}': {err}"),
            )
                .into_response();
        }
    };

    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let options = crate::transform::TransformOptions::default();
    let result = match ext {
        "tsx" => crate::transform::transform_tsx::transform_tsx(&source, &options),
        "ts" => crate::transform::typescript::transform_typescript(&source, &options),
        "jsx" => crate::transform::jsx::transform_jsx(&source, &options),
        "js" => Ok(crate::transform::TransformResult {
            code: source.clone(),
            source_map: None,
        }),
        _ => Ok(crate::transform::TransformResult {
            code: source.clone(),
            source_map: None,
        }),
    };

    match result {
        Ok(transformed) => js_response(transformed.code),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: transform error for '{request_path}': {err}"),
        )
            .into_response(),
    }
}

/// The browser-facing URL of a story's source file: root-relative, slashed.
fn module_url_for(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let mut url = String::from("/");
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    url.push_str(rel_str.trim_start_matches('/'));
    url
}

fn html_response(html: String) -> Response {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

fn js_response(code: String) -> Response {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        code,
    )
        .into_response()
}

/// Re-exported helper so callers (and tests) can resolve a story's module URL
/// without reaching into private internals.
pub fn story_module_url(root: &Path, story: &StoryEntry) -> String {
    module_url_for(root, &story.file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn module_url_is_root_relative() {
        let root = Path::new("/proj");
        let file = Path::new("/proj/src/Button.stories.tsx");
        assert_eq!(module_url_for(root, file), "/src/Button.stories.tsx");
    }

    #[test]
    fn build_router_constructs() {
        // Smoke: the router builds with an empty index without panicking.
        let _router = build_router(StoryIndex::default(), PathBuf::from("/tmp"));
    }

    #[test]
    fn story_module_url_matches_module_route() {
        let story = StoryEntry {
            id: "x--y".into(),
            name: "Y".into(),
            export_name: "Y".into(),
            args: BTreeMap::new(),
            has_render: false,
            file: PathBuf::from("/proj/a/B.stories.tsx"),
            title_path: vec!["X".into()],
        };
        assert_eq!(story_module_url(Path::new("/proj"), &story), "/a/B.stories.tsx");
    }
}
// HANDWRITE-END

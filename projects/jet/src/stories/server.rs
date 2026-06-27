// <HANDWRITE gap="missing-generator:logic:d0ce83ef" tracker="standardize-gap-projects-jet-src-stories-server-rs" reason="start_stories_workbench(root, host, port): discover StoryIndex, build a dev-server variant (reuse dev_server substrate) with routes for the manager, isolated preview, and module serving; build per-story entry via module graph.">
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
//! ## Bare-import resolution (#197)
//! Local *relative* imports (`./Button`) are resolved + transformed by the
//! module route. Bare specifiers (`import x from "clsx"`) that resolve to a real
//! file in the project's `node_modules` are now resolved via the shared
//! [`super::deps`] helper (which reuses the project
//! [`crate::resolver::ModuleResolver`]), rewritten in the served JS to a
//! `/@dep/<node_modules-relative-path>` route ([`DEP_PREFIX`]), and served —
//! transformed if TS/JSX — by [`dep_handler`], **recursively** for the dep's own
//! bare + relative imports. Specifiers that do NOT resolve on disk (e.g. `react`
//! with no local install) are left as-authored so the esm.sh importmap baked
//! into [`manager::render_preview_html`] still satisfies them.
//! TODO(#197 follow-up): advanced conditional-`exports` edge cases and CommonJS
//! interop are out of scope — see [`super::deps`].

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

use super::controls::{resolve_controls, Control};
use super::hmr::{self, StoriesHmrManager, STORIES_HMR_ROUTE};
use super::prop_extractor::extract_props_at;
use super::{discover, manager, StoryEntry, StoryIndex};
use crate::dev_server::module_graph::ModuleGraph;
use crate::dev_server::watcher::FileWatcher;

/// Manager shell route (alias of `/`).
pub const MANAGER_PREFIX: &str = "/__jet_stories_manager";

/// Route prefix for a resolved `node_modules` dependency module (#197).
///
/// A served/emitted module's bare import that resolves to a real file under the
/// project's `node_modules` is rewritten to `/@dep/<node_modules-relative-path>`
/// (e.g. `/@dep/clsx/dist/clsx.mjs`); [`dep_handler`] maps that path back to the
/// on-disk file, transforms it if needed, and recursively rewrites ITS imports.
pub const DEP_PREFIX: &str = "/@dep/";

/// React Fast Refresh runtime endpoint (#196). Must match the import specifier
/// the transform's [`crate::transform::react_refresh::inject_react_fast_refresh`]
/// preamble emits (`import RefreshRuntime from '/@react-refresh'`) so the
/// preview-served modules' refresh registration resolves. Reuses the dev
/// server's runtime shim.
pub const REACT_REFRESH_ROUTE: &str = "/@react-refresh";

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
        // React Fast Refresh runtime (#196): the preview-served `.tsx`/`.jsx`
        // modules carry an `import RefreshRuntime from '/@react-refresh'`
        // preamble (injected by the transform), so the preview must serve that
        // runtime — reusing the dev server's shim — for state-preserving refresh.
        .route(REACT_REFRESH_ROUTE, get(react_refresh_handler))
        // Resolved node_modules dependency modules (#197): the module route
        // rewrites a served module's bare imports to `/@dep/<key>`, served here.
        .route("/@dep/{*dep}", get(dep_handler))
        // Catch-all for module + static requests the preview imports.
        .route("/{*path}", get(module_handler))
        .with_state(state)
}

/// `GET /` / `GET /__jet_stories_manager` → the manager shell.
///
/// B3: the manager embeds the resolved controls for the initially-selected
/// story (the first in the id-sorted index) so the Controls panel renders
/// server-side, seeded with that story's current arg values.
async fn manager_handler(State(state): State<WorkbenchState>) -> Response {
    let selected = state.index.stories.first();
    let controls = selected
        .map(|story| controls_for_story(&state.root, &state.index, story))
        .unwrap_or_default();
    let html = manager::render_manager_html(&state.index, None, &controls);
    html_response(html)
}

/// Resolve the Controls panel descriptors for one story (B3).
///
/// Pipeline: find the story's meta → resolve the component's source file (the
/// relative import that brings in `meta.component`) → extract the component's
/// props → infer/override controls and seed each with the story's current args.
///
/// Every step degrades gracefully to an empty control list (no meta, no
/// component, unreadable file, no props) so the manager always renders.
fn controls_for_story(root: &Path, index: &StoryIndex, story: &StoryEntry) -> Vec<Control> {
    let Some(meta) = index.metas.iter().find(|m| m.file == story.file) else {
        return Vec::new();
    };
    let Some(component_name) = meta.component.as_deref() else {
        return Vec::new();
    };
    let Some((component_path, component_source)) =
        read_component_source(root, &story.file, component_name)
    else {
        return Vec::new();
    };
    let props = extract_props_at(&component_source, component_name, Some(&component_path));
    resolve_controls(&props, &meta.arg_types, &story.args)
}

/// Locate + read the source of the component named `component_name`, imported by
/// the story file at `story_file`.
///
/// Finds the story file's relative import that brings in `component_name`,
/// resolves it against the story file's directory (trying `.tsx/.ts/.jsx/.js`
/// and `index.*`), and returns the resolved file path plus its source. Returns
/// `None` for bare (node_modules) imports or unresolvable paths.
///
/// The path is returned (not just the source) so prop extraction can follow the
/// component file's own relative imports for cross-file prop types (#198).
///
/// TODO(#198 follow-up): cross-package / aliased component imports and barrel
/// re-exports (`export { Button } from './Button'`) are not followed.
fn read_component_source(
    root: &Path,
    story_file: &Path,
    component_name: &str,
) -> Option<(PathBuf, String)> {
    let story_source = std::fs::read_to_string(story_file).ok()?;
    let specifier = component_import_specifier(&story_source, component_name)?;
    // Only relative imports are resolvable to a local file here.
    if !specifier.starts_with('.') {
        return None;
    }
    let base_dir = story_file.parent().unwrap_or(root);
    let resolved = resolve_module_file(base_dir, &specifier)?;
    let source = std::fs::read_to_string(&resolved).ok()?;
    Some((resolved, source))
}

/// Find the import specifier (`./Button`) that imports `component_name` in the
/// story source. Matches `import { Button } ...`, `import Button ...`, and
/// `import { Foo as Button } ...` (the *local* binding is what the meta uses).
fn component_import_specifier(story_source: &str, component_name: &str) -> Option<String> {
    use tree_sitter::Parser;

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
        .ok()?;
    let tree = parser.parse(story_source, None)?;
    let root = tree.root_node();

    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        if child.kind() != "import_statement" {
            continue;
        }
        // The import's source string (last `string` child).
        let source_node = {
            let mut c = child.walk();
            child
                .named_children(&mut c)
                .filter(|n| n.kind() == "string")
                .last()
        };
        let Some(source_node) = source_node else {
            continue;
        };
        let specifier = strip_quotes(&story_source[source_node.byte_range()]);

        // Does this import bind `component_name`? Scan the import clause text for
        // the identifier as a default import or a named (possibly aliased) one.
        let clause_text = &story_source[child.byte_range()];
        if import_binds(clause_text, component_name) {
            return Some(specifier);
        }
    }
    None
}

/// True when an import statement's source text binds the local name `name`
/// (default import, namespace import, or named/aliased import).
fn import_binds(import_text: &str, name: &str) -> bool {
    // Named/aliased: `{ Foo as Button }` or `{ Button }`. The local binding is
    // the token after `as`, or the token itself.
    if let Some(open) = import_text.find('{') {
        if let Some(close) = import_text[open..].find('}') {
            let inner = &import_text[open + 1..open + close];
            for spec in inner.split(',') {
                let local = spec
                    .rsplit(" as ")
                    .next()
                    .unwrap_or(spec)
                    .trim()
                    .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '$');
                let local = local.trim();
                if local == name {
                    return true;
                }
            }
        }
    }
    // Default / namespace: `import Button from ...` / `import * as Button ...`.
    // Match the binding token between `import` and `from`.
    if let Some(after_import) = import_text.strip_prefix("import") {
        if let Some(from_idx) = after_import.find(" from ") {
            let head = &after_import[..from_idx];
            // Skip a leading `type` keyword and `* as`.
            let head = head.trim();
            let head = head.strip_prefix("type ").unwrap_or(head).trim();
            if let Some(ns) = head.strip_prefix("* as ") {
                if ns.trim() == name {
                    return true;
                }
            } else if !head.starts_with('{') {
                // `Button` or `Button, { ... }` — take the first token.
                let first = head.split(',').next().unwrap_or(head).trim();
                if first == name {
                    return true;
                }
            }
        }
    }
    false
}

/// Resolve a relative module specifier to an existing file under `base_dir`,
/// probing the common TS/JS extensions and an `index.*` barrel.
fn resolve_module_file(base_dir: &Path, specifier: &str) -> Option<PathBuf> {
    let joined = base_dir.join(specifier);
    // Exact path (specifier already had an extension).
    if joined.is_file() {
        return Some(joined);
    }
    const EXTS: &[&str] = &["tsx", "ts", "jsx", "js"];
    for ext in EXTS {
        let candidate = joined.with_extension(ext);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    // `./components/Button` → `./components/Button/index.tsx`.
    for ext in EXTS {
        let candidate = joined.join(format!("index.{ext}"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Strip surrounding quotes from a string-literal source slice.
fn strip_quotes(raw: &str) -> String {
    raw.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
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

/// `GET /@react-refresh` → the React Fast Refresh runtime shim (#196).
///
/// Serves the *same* runtime source the dev server serves
/// ([`crate::dev_server::react_refresh::react_refresh_runtime_source`]), so the
/// preview-served modules' injected `import RefreshRuntime from '/@react-refresh'`
/// (+ `$RefreshReg$` / `$RefreshSig$` registration) resolves and the preview's
/// HMR client can drive `performReactRefresh()` for state-preserving updates.
async fn react_refresh_handler() -> Response {
    js_response(crate::dev_server::react_refresh::react_refresh_runtime_source().to_string())
}

/// `GET /__jet_stories_hmr` → upgrade to the preview-frame HMR WebSocket.
///
/// Each connected preview frame subscribes to the shared [`StoriesHmrManager`];
/// the file watcher broadcasts [`super::hmr::StoriesHmrMessage`]s which this
/// handler forwards as JSON. The manager shell never connects here, so it never
/// reloads (B2b/#176).
async fn stories_hmr_handler(
    ws: WebSocketUpgrade,
    State(state): State<WorkbenchState>,
) -> Response {
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
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "ts" | "tsx" | "js" | "jsx" => {
            // B2b/#176: record this module's relative-import edges in the shared
            // graph BEFORE serving, so a later edit to an imported component can
            // walk back to the importing story (`affected_modules`). Best-effort:
            // failure to read/parse just means a thinner graph, never a 500.
            register_module_imports(&state, &file_path, &path);
            serve_module(&state.root, &file_path, &path).await
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

/// Transform a single project source file to browser JS, rewrite its resolvable
/// `node_modules` bare imports to `/@dep/<key>` routes (#197), and serve it.
///
/// Reuses the same per-extension transform entrypoints the dev server uses for
/// on-demand module serving (`transform_tsx` / `transform_typescript` /
/// `transform_jsx`; `.js` is served as-is). After transforming, every bare
/// import that resolves to a real file under the project's `node_modules` is
/// rewritten to the `/@dep/<node_modules-relative-path>` route ([`dep_handler`]
/// serves it). Unresolvable specifiers (e.g. `react` with no local install) are
/// left as-authored for the esm.sh importmap.
async fn serve_module(root: &Path, file_path: &Path, request_path: &str) -> Response {
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

    match transform_to_js(&source, file_path) {
        Ok(code) => js_response(rewrite_bare_imports_to_dep_routes(&code, root, file_path)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: transform error for '{request_path}': {err}"),
        )
            .into_response(),
    }
}

/// `GET /@dep/{key}` → transform + serve a resolved `node_modules` dependency
/// module (#197), recursively rewriting ITS own bare imports to `/@dep/<key>`.
///
/// `dep` is the `node_modules`-relative key the module route rewrote a bare
/// import to (`clsx/dist/clsx.mjs`). We map it back to the on-disk file under
/// `<root>/node_modules/<key>`, transform it if TS/JSX (pass through `.js`/
/// `.mjs`/`.cjs`), and rewrite the dep's own bare imports to further `/@dep/`
/// routes — so a dep's transitive deps load too. The dep's RELATIVE imports
/// (`./chunk.js`) resolve browser-side against this same `/@dep/<dir>/` URL, so
/// they need no rewriting.
async fn dep_handler(
    State(state): State<WorkbenchState>,
    AxumPath(dep): AxumPath<String>,
) -> Response {
    // Reject traversal so a `/@dep/../..` can't escape node_modules.
    if dep.split('/').any(|seg| seg == "..") {
        return (StatusCode::BAD_REQUEST, "jet stories: invalid dep path").into_response();
    }

    let file_path = state.root.join("node_modules").join(&dep);
    if !file_path.is_file() {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: dep not found '@dep/{dep}'"),
        )
            .into_response();
    }

    let source = match std::fs::read_to_string(&file_path) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("jet stories: failed to read dep '@dep/{dep}': {err}"),
            )
                .into_response();
        }
    };

    match transform_to_js(&source, &file_path) {
        Ok(code) => js_response(rewrite_bare_imports_to_dep_routes(
            &code,
            &state.root,
            &file_path,
        )),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: transform error for dep '@dep/{dep}': {err}"),
        )
            .into_response(),
    }
}

/// Transform a source file to browser JS using the same per-extension
/// entrypoints the dev server's module route uses. `.js`/`.mjs`/`.cjs` and any
/// other extension pass through unchanged.
fn transform_to_js(source: &str, file_path: &Path) -> Result<String> {
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let options = crate::transform::TransformOptions::default();
    let result = match ext {
        "tsx" => crate::transform::transform_tsx::transform_tsx(source, &options),
        "ts" => crate::transform::typescript::transform_typescript(source, &options),
        "jsx" => crate::transform::jsx::transform_jsx(source, &options),
        _ => Ok(crate::transform::TransformResult {
            code: source.to_string(),
            source_map: None,
        }),
    };
    result.map(|r| r.code).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Rewrite every bare import in `code` that resolves to a real file under the
/// project's `node_modules` to its `/@dep/<key>` route (#197).
///
/// `importer_file` is the on-disk file the code came from (the resolution base).
/// Bare specifiers that don't resolve on disk are left untouched so the esm.sh
/// importmap still satisfies them (React etc.). Only quoted specifier forms are
/// replaced, so an identifier sharing the spelling is never touched.
fn rewrite_bare_imports_to_dep_routes(code: &str, root: &Path, importer_file: &Path) -> String {
    let mut out = code.to_string();
    for spec in super::deps::extract_all_import_specifiers(code) {
        let Some(resolved) = super::deps::resolve_bare_specifier(root, importer_file, &spec) else {
            continue; // relative, or unresolved → leave for the importmap
        };
        let route = format!("{DEP_PREFIX}{}", super::deps::dep_key(&resolved));
        out = out
            .replace(&format!("\"{spec}\""), &format!("\"{route}\""))
            .replace(&format!("'{spec}'"), &format!("'{route}'"));
    }
    out
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
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
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
        assert_eq!(
            story_module_url(Path::new("/proj"), &story),
            "/a/B.stories.tsx"
        );
    }
}
// </HANDWRITE>

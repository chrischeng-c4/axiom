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

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{Context, Result};
use axum::{
    extract::{ws::WebSocket, Path as AxumPath, Query, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};

use super::controls::{resolve_controls, Control};
use super::hmr::{self, StoriesHmrManager, STORIES_HMR_ROUTE};
use super::prop_extractor::{extract_component_description, extract_props_at};
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
        .route("/index.json", get(index_json_handler))
        .route("/favicon.ico", get(favicon_handler))
        .route("/iframe.html", get(iframe_handler))
        .route("/__jet_stories_preview/{story_id}", get(preview_handler))
        .route("/__jet_stories_controls/{story_id}", get(controls_handler))
        .route(
            "/storybook-server-channel",
            get(storybook_server_channel_handler),
        )
        // Preview-frame HMR WebSocket (B2b/#176).
        .route(STORIES_HMR_ROUTE, get(stories_hmr_handler))
        // React Fast Refresh runtime (#196): the preview-served `.tsx`/`.jsx`
        // modules carry an `import RefreshRuntime from '/@react-refresh'`
        // preamble (injected by the transform), so the preview must serve that
        // runtime — reusing the dev server's shim — for state-preserving refresh.
        .route(REACT_REFRESH_ROUTE, get(react_refresh_handler))
        // Optimized third-party dependency bundles for heavy Storybook preview deps.
        .route(
            "/__jet_stories_optimized/{*specifier}",
            get(optimized_dep_handler),
        )
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
async fn manager_handler(
    State(state): State<WorkbenchState>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Response {
    let selected_path = selected_storybook_path_from_query(&query);
    let selected = state.index.stories.first();
    let controls = selected
        .map(|story| controls_for_story(&state.root, &state.index, story))
        .unwrap_or_default();
    let docs_pages = docs_pages_for_index(&state.root, &state.index);
    let html = if selected_path.is_some() {
        manager::render_official_storybook_manager_html()
    } else {
        manager::render_manager_html_with_docs(&state.index, None, &controls, &docs_pages)
    };
    html_response(html)
}

async fn index_json_handler(State(state): State<WorkbenchState>) -> Response {
    json_response(story_index_json(&state.root, &state.index))
}

async fn favicon_handler() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

async fn iframe_handler(
    State(state): State<WorkbenchState>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Response {
    let id = query.get("id").map(String::as_str).unwrap_or_default();
    let view_mode = query.get("viewMode").map(String::as_str).unwrap_or("story");
    if view_mode == "docs" {
        return docs_response_for_docs_id(&state.root, &state.index, id);
    }
    preview_response_for_story_id(&state.root, &state.index, id)
}

/// `GET /__jet_stories_controls/{story_id}` → current Controls markup + args.
///
/// The manager calls this on sidebar selection so the Controls panel is
/// re-derived from the selected story instead of keeping the initially-selected
/// story's args globally (#987).
async fn controls_handler(
    State(state): State<WorkbenchState>,
    AxumPath(story_id): AxumPath<String>,
) -> Response {
    let Some(story) = state.index.stories.iter().find(|s| s.id == story_id) else {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: unknown story id '{story_id}'"),
        )
            .into_response();
    };
    let controls = controls_for_story(&state.root, &state.index, story);
    json_response(manager::render_controls_payload_json(&controls))
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

pub(crate) fn docs_pages_for_index(root: &Path, index: &StoryIndex) -> Vec<manager::DocsPage> {
    let mut pages = Vec::new();
    for meta in &index.metas {
        if meta.tags.iter().any(|tag| tag == "!autodocs") {
            continue;
        }
        let title = meta.title_path.join(" / ");
        let stories: Vec<manager::DocsStory> = index
            .stories
            .iter()
            .filter(|story| story.file == meta.file && story.title_path == meta.title_path)
            .map(|story| manager::DocsStory {
                id: story.id.clone(),
                name: story.name.clone(),
                description: story.description.clone(),
            })
            .collect();
        if stories.is_empty() {
            continue;
        }
        let mut description = String::new();
        let mut arg_types = Vec::new();
        if let Some(component_name) = meta.component.as_deref() {
            if let Some((component_path, component_source)) =
                read_component_source(root, &meta.file, component_name)
            {
                description = extract_component_description(&component_source, component_name);
                arg_types =
                    extract_props_at(&component_source, component_name, Some(&component_path))
                        .into_iter()
                        .map(|prop| manager::DocsArgType {
                            name: prop.name,
                            type_text: prop.type_text,
                            default_value: prop.default_value,
                            description: prop.description,
                            control_kind: None,
                            control_options: Vec::new(),
                            control_current: None,
                        })
                        .collect();
            }
        }
        pages.push(manager::DocsPage {
            id: format!("docs-{}", slug_for_docs_id(&title)),
            title,
            description,
            primary_story_id: stories
                .first()
                .map(|story| story.id.clone())
                .unwrap_or_default(),
            stories,
            arg_types,
            content_html: None,
        });
    }
    let mut mdx_pages = super::mdx::docs_pages(root, index, &pages);
    mdx_pages.extend(pages);
    mdx_pages
}

pub(crate) fn story_index_json(root: &Path, index: &StoryIndex) -> String {
    let docs_pages = docs_pages_for_index(root, index);
    let mut entries = Vec::new();
    let mut emitted = BTreeSet::new();
    for meta in &index.metas {
        push_storybook_docs_entry(root, &mut emitted, &mut entries, meta);
        let component_path = meta.component.as_deref().and_then(|component_name| {
            read_component_source(root, &meta.file, component_name).map(|(path, _)| path)
        });
        for story in index
            .stories
            .iter()
            .filter(|story| story.file == meta.file && story.title_path == meta.title_path)
        {
            push_storybook_story_entry(
                root,
                &mut emitted,
                &mut entries,
                story,
                component_path.as_deref(),
            );
        }
    }
    for story in &index.stories {
        push_storybook_story_entry(root, &mut emitted, &mut entries, story, None);
    }

    let mut legacy_stories = String::new();
    for (idx, story) in index.stories.iter().enumerate() {
        if idx > 0 {
            legacy_stories.push(',');
        }
        let title = story.title_path.join("/");
        let import_path = storybook_import_path(root, &story.file);
        let tags = index
            .metas
            .iter()
            .find(|meta| meta.file == story.file)
            .map(|meta| meta.tags.as_slice())
            .unwrap_or(&[]);
        legacy_stories.push_str(&format!(
            "{{\"id\":{},\"title\":{},\"name\":{},\"importPath\":{},\"tags\":{}}}",
            json_string(&story.id),
            json_string(&title),
            json_string(&story.name),
            json_string(&import_path),
            json_string_array(tags),
        ));
    }

    let mut legacy_docs = String::new();
    for (idx, docs) in docs_pages.iter().enumerate() {
        if idx > 0 {
            legacy_docs.push(',');
        }
        legacy_docs.push_str(&format!(
            "{{\"id\":{},\"title\":{},\"primaryStoryId\":{}}}",
            json_string(&docs.id),
            json_string(&docs.title),
            json_string(&docs.primary_story_id),
        ));
    }

    format!(
        "{{\"v\":5,\"entries\":{{{}}},\"schemaVersion\":1,\"stories\":[{}],\"docs\":[{}]}}",
        entries.join(","),
        legacy_stories,
        legacy_docs,
    )
}

fn push_storybook_docs_entry(
    root: &Path,
    emitted: &mut BTreeSet<String>,
    entries: &mut Vec<String>,
    meta: &super::StoryMeta,
) {
    let title = storybook_title(&meta.title_path);
    let id = storybook_docs_id(&title);
    if !emitted.insert(id.clone()) {
        return;
    }
    let import_path = storybook_import_path(root, &meta.file);
    entries.push(format!(
        "{}:{{\"id\":{},\"title\":{},\"name\":\"Docs\",\"importPath\":{},\"type\":\"docs\",\"tags\":[\"dev\",\"test\",\"autodocs\"],\"storiesImports\":[]}}",
        json_string(&id),
        json_string(&id),
        json_string(&title),
        json_string(&import_path),
    ));
}

fn push_storybook_story_entry(
    root: &Path,
    emitted: &mut BTreeSet<String>,
    entries: &mut Vec<String>,
    story: &StoryEntry,
    component_path: Option<&Path>,
) {
    if !emitted.insert(story.id.clone()) {
        return;
    }
    let title = storybook_title(&story.title_path);
    let import_path = storybook_import_path(root, &story.file);
    let component_path_field = component_path
        .map(|path| {
            format!(
                ",\"componentPath\":{}",
                json_string(&storybook_import_path(root, path))
            )
        })
        .unwrap_or_default();
    entries.push(format!(
        "{}:{{\"id\":{},\"title\":{},\"name\":{},\"importPath\":{},\"type\":\"story\"{},\"tags\":[\"dev\",\"test\",\"autodocs\"]}}",
        json_string(&story.id),
        json_string(&story.id),
        json_string(&title),
        json_string(&storybook_story_name(&story.name)),
        json_string(&import_path),
        component_path_field,
    ));
}

fn storybook_import_path(root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let path = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".to_string()),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    if path.starts_with("./") {
        path
    } else {
        format!("./{path}")
    }
}

fn storybook_docs_id(title: &str) -> String {
    format!("{}--docs", slug_for_docs_id(title))
}

fn storybook_title(title_path: &[String]) -> String {
    title_path.join("/")
}

fn storybook_story_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower_or_digit = false;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
            prev_lower_or_digit = false;
            continue;
        }
        if ch.is_ascii_uppercase() && prev_lower_or_digit && !out.ends_with(' ') {
            out.push(' ');
        }
        out.push(ch);
        prev_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }
    let mut chars = out.trim().chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + chars.as_str()
}

fn json_string_array(values: &[String]) -> String {
    let mut out = String::from("[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&json_string(value));
    }
    out.push(']');
    out
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn slug_for_docs_id(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
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
pub(crate) fn read_component_source(
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
        let candidate = probe_with_extension(&joined, ext);
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

fn probe_with_extension(path: &Path, ext: &str) -> PathBuf {
    if path.extension().is_some() {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return path.with_extension(ext);
        };
        return path.with_file_name(format!("{file_name}.{ext}"));
    }
    path.with_extension(ext)
}

/// Strip surrounding quotes from a string-literal source slice.
fn strip_quotes(raw: &str) -> String {
    raw.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

fn selected_storybook_path_from_query(query: &BTreeMap<String, String>) -> Option<String> {
    query
        .get("path")
        .filter(|path| path.starts_with("/story/") || path.starts_with("/docs/"))
        .cloned()
}

fn preview_response_for_story_id(root: &Path, index: &StoryIndex, story_id: &str) -> Response {
    if story_id.is_empty() {
        return html_response(manager::render_empty_preview_html());
    }
    let Some(story) = index.stories.iter().find(|s| s.id == story_id) else {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: unknown story id '{story_id}'"),
        )
            .into_response();
    };
    let module_url = module_url_for(root, &story.file);
    let project_preview_url = project_preview_module_url(root);
    let controls = controls_for_story(root, index, story);
    let html = manager::render_preview_html_with_project_preview_actions_and_controls(
        story,
        &module_url,
        manager::UrlMode::Dev,
        project_preview_url.as_deref(),
        &[],
        &controls,
    );
    html_response(html)
}

fn docs_response_for_docs_id(root: &Path, index: &StoryIndex, docs_id: &str) -> Response {
    let docs_pages = docs_pages_for_index(root, index);
    let Some(page) = docs_pages
        .iter()
        .find(|page| docs_page_matches_id(page, docs_id))
    else {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: unknown docs id '{docs_id}'"),
        )
            .into_response();
    };
    html_response(manager::render_docs_preview_html(
        page,
        manager::UrlMode::Dev,
    ))
}

fn docs_page_matches_id(page: &manager::DocsPage, id: &str) -> bool {
    page.id == id || storybook_docs_id(&page.title.replace(" / ", "/")) == id
}

/// `GET /__jet_stories_preview/{story_id}` → isolated single-story preview.
async fn preview_handler(
    State(state): State<WorkbenchState>,
    AxumPath(story_id): AxumPath<String>,
) -> Response {
    preview_response_for_story_id(&state.root, &state.index, &story_id)
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

async fn storybook_server_channel_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(storybook_server_channel_socket)
}

async fn storybook_server_channel_socket(socket: WebSocket) {
    use axum::extract::ws::Message;
    use std::time::Duration;

    let (mut sender, mut receiver) = socket.split();
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if sender
                    .send(Message::Text(r#"{"type":"ping"}"#.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if text.contains(r#""type":"requestWhatsNewData""#)
                            && sender
                                .send(Message::Text(storybook_whats_new_response_json().into()))
                                .await
                                .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

fn storybook_whats_new_response_json() -> &'static str {
    r#"{"type":"resultWhatsNewData","args":[{"data":{"title":"Storybook 10.4","url":"https://storybook.js.org/blog/whats-new/storybook-10-4","blogUrl":"https://storybook.js.org/blog/storybook-10-4","publishedAt":"2026-05-18T20:38:16.000+00:00","excerpt":"Storybook 10.4","blogExcerpt":"Automatic setup with agents, review filters, TanStack React, and more","status":"SUCCESS","postIsRead":false,"showNotification":true,"disableWhatsNewNotifications":false}}],"from":"jet"}"#
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
    if let Some(asset_path) = resolve_storybook_manager_asset(&state.root, &path) {
        return serve_storybook_manager_asset(&asset_path).await;
    }

    let file_path =
        resolve_module_file(&state.root, &path).unwrap_or_else(|| state.root.join(&path));
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
        "css" | "scss" | "sass" => serve_style_module(&file_path, &path).await,
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" => {
            serve_raw_asset(&file_path, &path).await
        }
        _ => (
            StatusCode::NOT_FOUND,
            format!("jet stories: not found '{path}'"),
        )
            .into_response(),
    }
}

fn resolve_storybook_manager_asset(root: &Path, request_path: &str) -> Option<PathBuf> {
    if request_path == "favicon.svg" {
        return storybook_core_package_dir(root)
            .map(|dir| dir.join("assets/browser/favicon.svg"))
            .filter(|path| path.is_file());
    }
    if let Some(rel) = request_path.strip_prefix("sb-common-assets/") {
        return storybook_core_package_dir(root)
            .map(|dir| dir.join("assets/browser").join(rel))
            .filter(|path| path.is_file());
    }
    if let Some(rel) = request_path.strip_prefix("sb-manager/") {
        return storybook_core_package_dir(root)
            .map(|dir| dir.join("dist/manager").join(rel))
            .filter(|path| path.is_file());
    }
    if request_path.starts_with("sb-addons/") {
        return storybook_cache_public_dir(root)
            .map(|dir| dir.join(request_path))
            .filter(|path| path.is_file());
    }
    None
}

fn storybook_core_package_dir(root: &Path) -> Option<PathBuf> {
    node_package_dir(root, "@storybook/core")
}

fn node_package_dir(root: &Path, package_name: &str) -> Option<PathBuf> {
    let direct = root.join("node_modules").join(package_name);
    if direct.is_dir() {
        return Some(direct);
    }
    let pnpm_hoist = root
        .join("node_modules/.pnpm/node_modules")
        .join(package_name);
    if pnpm_hoist.is_dir() {
        return Some(pnpm_hoist);
    }

    let pnpm_dir = root.join("node_modules/.pnpm");
    let entries = std::fs::read_dir(pnpm_dir).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("node_modules").join(package_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn storybook_cache_public_dir(root: &Path) -> Option<PathBuf> {
    let cache_dir = root.join("node_modules/.cache/storybook");
    let entries = std::fs::read_dir(cache_dir).ok()?;
    for entry in entries.flatten() {
        let public = entry.path().join("public");
        if public.join("sb-addons").is_dir() {
            return Some(public);
        }
    }
    None
}

async fn serve_storybook_manager_asset(path: &Path) -> Response {
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if ext == "js" {
        return match std::fs::read_to_string(path) {
            Ok(code) => js_response(code),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "jet stories: failed to read Storybook manager asset '{}': {err}",
                    path.display()
                ),
            )
                .into_response(),
        };
    }
    match std::fs::read(path) {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, storybook_asset_content_type(path))],
            bytes,
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "jet stories: failed to read Storybook manager asset '{}': {err}",
                path.display()
            ),
        )
            .into_response(),
    }
}

fn storybook_asset_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "css" => "text/css; charset=utf-8",
        _ => "application/octet-stream",
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

async fn serve_style_module(file_path: &Path, request_path: &str) -> Response {
    let css = if crate::css::scss::is_sass_family_path(file_path) {
        match crate::css::scss::compile_sass_file(file_path) {
            Ok(css) => css,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("jet stories: style transform error for '{request_path}': {err}"),
                )
                    .into_response();
            }
        }
    } else {
        match std::fs::read_to_string(file_path) {
            Ok(css) => css,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("jet stories: failed to read style '{request_path}': {err}"),
                )
                    .into_response();
            }
        }
    };
    match crate::transform::css::transform_css(&css, &crate::transform::TransformOptions::default())
    {
        Ok(result) => js_response(result.code),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: style transform error for '{request_path}': {err}"),
        )
            .into_response(),
    }
}

async fn serve_raw_asset(file_path: &Path, request_path: &str) -> Response {
    match std::fs::read(file_path) {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, content_type_for_asset(file_path))],
            bytes,
        )
            .into_response(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            format!("jet stories: not found '{request_path}'"),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: failed to read asset '{request_path}': {err}"),
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
async fn optimized_dep_handler(
    State(state): State<WorkbenchState>,
    AxumPath(specifier): AxumPath<String>,
) -> Response {
    if specifier.split('/').any(|seg| seg == "..") {
        return (
            StatusCode::BAD_REQUEST,
            "jet stories: invalid optimized dep path",
        )
            .into_response();
    }

    match super::optimizer::optimized_dep_source(&state.root, &specifier) {
        Ok(code) => js_response(rewrite_optimized_external_imports(&code)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("jet stories: failed to optimize dependency '{specifier}': {err}"),
        )
            .into_response(),
    }
}

async fn dep_handler(
    State(state): State<WorkbenchState>,
    AxumPath(dep): AxumPath<String>,
) -> Response {
    // Reject traversal so a `/@dep/../..` can't escape node_modules.
    if dep.split('/').any(|seg| seg == "..") {
        return (StatusCode::BAD_REQUEST, "jet stories: invalid dep path").into_response();
    }

    let file_path = resolve_dep_route_file(&state.root, &dep)
        .unwrap_or_else(|| state.root.join("node_modules").join(&dep));
    if !file_path.is_file() {
        return (
            StatusCode::NOT_FOUND,
            format!("jet stories: dep not found '@dep/{dep}'"),
        )
            .into_response();
    }

    let ext = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    if matches!(ext, "css" | "scss" | "sass") {
        return serve_style_module(&file_path, &format!("@dep/{dep}")).await;
    }
    if is_raw_asset_path(&file_path) {
        return serve_raw_asset(&file_path, &format!("@dep/{dep}")).await;
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
        "js" | "cjs" => Ok(crate::transform::TransformResult {
            code: crate::dev_server::prebundle::convert_cjs_file_to_esm_with_import_prefix(
                source, file_path, DEP_PREFIX,
            ),
            source_map: None,
        }),
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
fn rewrite_optimized_external_imports(code: &str) -> String {
    let mut specs = vec!["dayjs".to_string()];
    for spec in super::deps::extract_all_import_specifiers(code) {
        if spec.starts_with("dayjs/") && !specs.contains(&spec) {
            specs.push(spec);
        }
    }
    let mut out = code.to_string();
    for spec in specs {
        let route = format!("{DEP_PREFIX}{spec}");
        out = rewrite_import_source_literal(&out, &spec, &route);
    }
    out
}

fn rewrite_import_source_literal(code: &str, spec: &str, route: &str) -> String {
    code.replace(&format!(" from \"{spec}\""), &format!(" from \"{route}\""))
        .replace(&format!(" from '{spec}'"), &format!(" from '{route}'"))
        .replace(
            &format!("import \"{spec}\""),
            &format!("import \"{route}\""),
        )
        .replace(&format!("import '{spec}'"), &format!("import '{route}'"))
}

fn rewrite_bare_imports_to_dep_routes(code: &str, root: &Path, importer_file: &Path) -> String {
    let mut out = code.to_string();
    for spec in super::deps::extract_all_import_specifiers(code) {
        if spec.starts_with('.') && path_has_node_modules(importer_file) {
            if let Some(resolved) = resolve_relative_import_file(importer_file, &spec) {
                let route = if path_has_node_modules(&resolved) {
                    format!("{DEP_PREFIX}{}", super::deps::dep_key(&resolved))
                } else {
                    module_url_for(root, &resolved)
                };
                if is_raw_asset_path(&resolved) {
                    out = rewrite_asset_import_for_spec(&out, &spec, &route);
                } else {
                    out = out
                        .replace(&format!("\"{spec}\""), &format!("\"{route}\""))
                        .replace(&format!("'{spec}'"), &format!("'{route}'"));
                }
                continue;
            }
        }
        if let Some(route) = super::optimizer::optimized_route_for_specifier(root, &spec) {
            out = out
                .replace(&format!("\"{spec}\""), &format!("\"{route}\""))
                .replace(&format!("'{spec}'"), &format!("'{route}'"));
            continue;
        }
        let Some(resolved) = super::deps::resolve_bare_specifier(root, importer_file, &spec) else {
            continue; // relative, or unresolved -> leave for the importmap
        };
        if is_raw_asset_path(&resolved) {
            let route = if path_has_node_modules(&resolved) {
                format!("{DEP_PREFIX}{}", super::deps::dep_key(&resolved))
            } else {
                module_url_for(root, &resolved)
            };
            out = rewrite_asset_import_for_spec(&out, &spec, &route);
            continue;
        }
        let route = if spec == "dayjs" {
            format!("{DEP_PREFIX}dayjs")
        } else {
            format!("{DEP_PREFIX}{}", super::deps::dep_key(&resolved))
        };
        out = out
            .replace(&format!("\"{spec}\""), &format!("\"{route}\""))
            .replace(&format!("'{spec}'"), &format!("'{route}'"));
    }
    out
}

fn resolve_relative_import_file(importer_file: &Path, spec: &str) -> Option<PathBuf> {
    let base = importer_file.parent()?;
    resolve_file_with_extension_fallback(base.join(spec))
}

fn rewrite_asset_import_for_spec(code: &str, spec: &str, new_spec: &str) -> String {
    let mut out = String::with_capacity(code.len());
    for line in code.split_inclusive('\n') {
        let newline = if line.ends_with('\n') { "\n" } else { "" };
        let body = line.trim_end_matches('\n');
        let mut rewritten_body = String::with_capacity(body.len());
        for segment in body.split_inclusive(';') {
            let leading_len = segment.len() - segment.trim_start().len();
            let leading = &segment[..leading_len];
            let trimmed = segment.trim_start();
            if let Some(replacement) = asset_import_replacement(trimmed, spec, new_spec) {
                rewritten_body.push_str(leading);
                rewritten_body.push_str(&replacement);
            } else {
                rewritten_body.push_str(segment);
            }
        }
        out.push_str(&rewritten_body);
        out.push_str(newline);
    }
    out
}

fn asset_import_replacement(trimmed_line: &str, spec: &str, new_spec: &str) -> Option<String> {
    if !(trimmed_line.contains(&format!("\"{spec}\""))
        || trimmed_line.contains(&format!("'{spec}'")))
    {
        return None;
    }
    if !trimmed_line.starts_with("import ") {
        return None;
    }
    let before_from = trimmed_line.split(" from ").next()?.trim();
    let binding = before_from.trim_start_matches("import").trim();
    if binding.is_empty() || binding.starts_with('{') || binding.starts_with('*') {
        return Some(String::new());
    }
    let default_binding = binding.split(',').next()?.trim();
    if !is_js_identifier(default_binding) {
        return None;
    }
    Some(format!("const {default_binding} = {new_spec:?};"))
}

fn is_js_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn is_raw_asset_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(ext)
            if ext.eq_ignore_ascii_case("svg")
                || ext.eq_ignore_ascii_case("png")
                || ext.eq_ignore_ascii_case("jpg")
                || ext.eq_ignore_ascii_case("jpeg")
                || ext.eq_ignore_ascii_case("gif")
                || ext.eq_ignore_ascii_case("webp")
                || ext.eq_ignore_ascii_case("avif")
    )
}

fn path_has_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == std::ffi::OsStr::new("node_modules"))
}

fn content_type_for_asset(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

fn resolve_dep_route_file(root: &Path, dep: &str) -> Option<PathBuf> {
    resolve_file_with_extension_fallback(root.join("node_modules").join(dep))
        .or_else(|| resolve_pnpm_dep_file(root, dep))
        .or_else(|| resolve_workspace_dep_file(root, dep))
}

fn resolve_pnpm_dep_file(root: &Path, dep: &str) -> Option<PathBuf> {
    let (package_name, subpath) = split_dep_package_path(dep)?;
    let entries = std::fs::read_dir(root.join("node_modules/.pnpm")).ok()?;
    for entry in entries.flatten() {
        let package_dir = entry.path().join("node_modules").join(&package_name);
        let candidate = package_dir.join(&subpath);
        if let Some(file) = resolve_file_with_extension_fallback(candidate) {
            return Some(file);
        }
        if let Some(file) = resolve_package_json_route(&package_dir, &subpath) {
            return Some(file);
        }
    }
    None
}

fn resolve_workspace_dep_file(root: &Path, dep: &str) -> Option<PathBuf> {
    let (package_name, subpath) = split_dep_package_path(dep)?;
    let package_dir = workspace_package_dir(root, &package_name)?;
    let direct = package_dir.join(&subpath);
    if let Some(file) = resolve_file_with_extension_fallback(direct) {
        return Some(file);
    }
    if let Some(file) = resolve_package_json_route(&package_dir, &subpath) {
        return Some(file);
    }
    if let Some(rest) = subpath.strip_prefix("dist/") {
        for source_root in ["src/lib", "src"] {
            let source = package_dir.join(source_root).join(rest);
            if let Some(file) = resolve_file_with_extension_fallback(source) {
                return Some(file);
            }
        }
    }
    None
}

fn resolve_package_json_route(package_dir: &Path, subpath: &str) -> Option<PathBuf> {
    let package = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
    let package = serde_json::from_str::<serde_json::Value>(&package).ok()?;
    if let Some(exports) = package.get("exports") {
        let key = if subpath.is_empty() {
            ".".to_string()
        } else {
            format!("./{subpath}")
        };
        if let Some(target) = export_target_for_route(exports, &key) {
            let target = target.trim_start_matches("./");
            if let Some(file) = resolve_file_with_extension_fallback(package_dir.join(target)) {
                return Some(file);
            }
        }
    }
    if subpath.is_empty() {
        for field in ["browser", "module", "main"] {
            if let Some(target) = package.get(field).and_then(|value| value.as_str()) {
                let target = target.trim_start_matches("./");
                if let Some(file) = resolve_file_with_extension_fallback(package_dir.join(target)) {
                    return Some(file);
                }
            }
        }
    }
    None
}

fn export_target_for_route(exports: &serde_json::Value, key: &str) -> Option<String> {
    match exports {
        serde_json::Value::String(target) if key == "." => Some(target.clone()),
        serde_json::Value::Array(items) => items
            .iter()
            .find_map(|item| export_target_for_route(item, key)),
        serde_json::Value::Object(map) => {
            if let Some(value) = map.get(key) {
                return export_target_value(value);
            }
            for (pattern, value) in map {
                let Some((prefix, suffix)) = pattern.split_once('*') else {
                    continue;
                };
                if key.starts_with(prefix) && key.ends_with(suffix) {
                    let matched = &key[prefix.len()..key.len() - suffix.len()];
                    return export_target_value(value).map(|target| target.replace('*', matched));
                }
            }
            if key == "." {
                return export_target_value(exports);
            }
            None
        }
        _ => None,
    }
}

fn export_target_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(target) => Some(target.clone()),
        serde_json::Value::Array(items) => items.iter().find_map(export_target_value),
        serde_json::Value::Object(map) => {
            for key in ["browser", "import", "module", "default", "require"] {
                if let Some(value) = map.get(key).and_then(export_target_value) {
                    return Some(value);
                }
            }
            None
        }
        _ => None,
    }
}

fn resolve_file_with_extension_fallback(path: PathBuf) -> Option<PathBuf> {
    if path.is_file() {
        return Some(path);
    }
    if path.extension().is_none() {
        for ext in ["js", "mjs", "cjs", "ts", "tsx", "jsx"] {
            let candidate = path.with_extension(ext);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    if path.is_dir() {
        for name in [
            "index.js",
            "index.mjs",
            "index.cjs",
            "index.ts",
            "index.tsx",
            "index.jsx",
        ] {
            let candidate = path.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn split_dep_package_path(dep: &str) -> Option<(String, String)> {
    if dep.starts_with('@') {
        let mut parts = dep.splitn(4, '/');
        let scope = parts.next()?;
        let name = parts.next()?;
        let rest = parts.next().unwrap_or("");
        let tail = parts
            .next()
            .map(|tail| format!("{rest}/{tail}"))
            .unwrap_or_else(|| rest.to_string());
        return Some((format!("{scope}/{name}"), tail));
    }
    let (package_name, rest) = dep.split_once('/').unwrap_or((dep, ""));
    Some((package_name.to_string(), rest.to_string()))
}

fn workspace_package_dir(root: &Path, package_name: &str) -> Option<PathBuf> {
    for parent in ["packages", "libs"] {
        let dir = root.join(parent);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let package_dir = entry.path();
            let package_json = package_dir.join("package.json");
            let Ok(body) = std::fs::read_to_string(package_json) else {
                continue;
            };
            let Ok(package) = serde_json::from_str::<serde_json::Value>(&body) else {
                continue;
            };
            if package.get("name").and_then(|name| name.as_str()) == Some(package_name) {
                return Some(package_dir);
            }
        }
    }
    None
}

/// The browser-facing URL of a story's source file: root-relative, slashed.
fn module_url_for(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let mut url = String::from("/");
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    url.push_str(rel_str.trim_start_matches('/'));
    url
}

fn project_preview_file(root: &Path) -> Option<PathBuf> {
    for ext in ["ts", "tsx", "js", "jsx"] {
        let path = root.join(".storybook").join(format!("preview.{ext}"));
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn project_preview_module_url(root: &Path) -> Option<String> {
    project_preview_file(root).map(|file| module_url_for(root, &file))
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

fn json_response(json: String) -> Response {
    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        json,
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
            description: String::new(),
            args: BTreeMap::new(),
            parameters: BTreeMap::new(),
            source: None,
            has_render: false,
            file: PathBuf::from("/proj/a/B.stories.tsx"),
            title_path: vec!["X".into()],
        };
        assert_eq!(
            story_module_url(Path::new("/proj"), &story),
            "/a/B.stories.tsx"
        );
    }

    #[test]
    fn resolve_module_file_probes_extensionless_and_story_suffix_paths() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::write(root.join("src/sp-empty-box.tsx"), "export const X = 1;")
            .expect("write component");
        std::fs::write(
            root.join("src/form-wrapper.stories.tsx"),
            "export const Default = {};",
        )
        .expect("write story");

        assert_eq!(
            resolve_module_file(root, "src/sp-empty-box"),
            Some(root.join("src/sp-empty-box.tsx"))
        );
        assert_eq!(
            resolve_module_file(root, "src/form-wrapper.stories"),
            Some(root.join("src/form-wrapper.stories.tsx"))
        );
    }

    #[test]
    fn rewrites_bare_asset_default_import_to_url_const() {
        let code = "import Icon from '@scope/assets/icon.svg';\nexport const value = Icon;\n";
        let rewritten =
            rewrite_asset_import_for_spec(code, "@scope/assets/icon.svg", "/assets/icon.svg");
        assert!(
            rewritten.contains("const Icon = \"/assets/icon.svg\";"),
            "asset default import should become a URL const: {rewritten}"
        );
        assert!(
            !rewritten.contains("@scope/assets/icon.svg"),
            "asset import specifier should be removed: {rewritten}"
        );
    }

    #[test]
    fn rewrites_node_modules_relative_imports_to_dep_routes() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let dep_dir =
            root.join("node_modules/.pnpm/dom-helpers@6.0.1/node_modules/dom-helpers/esm");
        std::fs::create_dir_all(&dep_dir).expect("mkdir dep");
        let index = dep_dir.join("index.js");
        let animate = dep_dir.join("animate.js");
        std::fs::write(&animate, "export default null;").expect("write dep");
        let rewritten = rewrite_bare_imports_to_dep_routes(
            "export { default as animate } from './animate';",
            root,
            &index,
        );
        assert!(
            rewritten.contains("/@dep/dom-helpers/esm/animate.js"),
            "relative dep import should use package-relative dep route: {rewritten}"
        );
    }

    #[test]
    fn asset_content_type_uses_image_mime() {
        assert_eq!(content_type_for_asset(Path::new("x.svg")), "image/svg+xml");
        assert_eq!(content_type_for_asset(Path::new("x.png")), "image/png");
    }

    #[test]
    fn style_paths_are_not_raw_assets() {
        assert!(!is_raw_asset_path(Path::new("x.css")));
        assert!(!is_raw_asset_path(Path::new("x.scss")));
    }

    #[test]
    fn resolves_workspace_dep_asset_from_dist_or_source_fallback() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir = root.join("packages/assets");
        std::fs::create_dir_all(package_dir.join("src/lib/icons")).expect("mkdir package");
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"@tw-tech/shared-assets"}"#,
        )
        .expect("write package");
        std::fs::write(package_dir.join("src/lib/icons/list.svg"), "<svg />").expect("write svg");

        assert_eq!(
            resolve_workspace_dep_file(root, "@tw-tech/shared-assets/dist/icons/list.svg"),
            Some(package_dir.join("src/lib/icons/list.svg"))
        );
    }

    #[test]
    fn resolves_workspace_dep_js_from_dist() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir = root.join("packages/ui-styles");
        std::fs::create_dir_all(package_dir.join("dist")).expect("mkdir package");
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"@tw-tech/shared-ui-styles"}"#,
        )
        .expect("write package");
        std::fs::write(package_dir.join("dist/index.js"), "export const x = 1;").expect("write js");

        assert_eq!(
            resolve_workspace_dep_file(root, "@tw-tech/shared-ui-styles/dist/index.js"),
            Some(package_dir.join("dist/index.js"))
        );
    }

    #[test]
    fn resolves_pnpm_dep_route_file_from_canonical_dep_key() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir =
            root.join("node_modules/.pnpm/react-router-dom@6.30.4/node_modules/react-router-dom");
        std::fs::create_dir_all(package_dir.join("dist")).expect("mkdir package");
        std::fs::write(package_dir.join("dist/index.js"), "export const Link = 1;")
            .expect("write js");

        assert_eq!(
            resolve_dep_route_file(root, "react-router-dom/dist/index.js"),
            Some(package_dir.join("dist/index.js"))
        );
    }

    #[test]
    fn resolves_pnpm_dep_route_file_with_extensionless_fallback() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir =
            root.join("node_modules/.pnpm/dom-helpers@6.0.1/node_modules/dom-helpers");
        std::fs::create_dir_all(package_dir.join("esm")).expect("mkdir package");
        std::fs::write(package_dir.join("esm/canUseDOM.js"), "export default true;")
            .expect("write js");

        assert_eq!(
            resolve_dep_route_file(root, "dom-helpers/esm/canUseDOM"),
            Some(package_dir.join("esm/canUseDOM.js"))
        );
    }

    #[test]
    fn cjs_dep_transform_exports_default() {
        let file = Path::new("/proj/node_modules/classnames/index.js");
        let transformed =
            transform_to_js("module.exports = function classNames() {};", file).expect("transform");
        assert!(transformed.contains("export default __cjs_default__"));
        assert!(!transformed.contains("/node_modules/.jet/"));
    }

    #[test]
    fn resolves_package_root_via_package_json_module_or_main() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir = root.join("node_modules/.pnpm/dayjs@1.0.0/node_modules/dayjs");
        std::fs::create_dir_all(&package_dir).expect("mkdir package");
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"dayjs","main":"dayjs.min.js"}"#,
        )
        .expect("write package");
        std::fs::write(package_dir.join("dayjs.min.js"), "module.exports = {};").expect("write js");

        assert_eq!(
            resolve_dep_route_file(root, "dayjs"),
            Some(package_dir.join("dayjs.min.js"))
        );
    }

    #[test]
    fn resolves_package_subpath_via_exports_wildcard() {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        let package_dir =
            root.join("node_modules/.pnpm/dom-helpers@6.0.1/node_modules/dom-helpers");
        std::fs::create_dir_all(package_dir.join("esm")).expect("mkdir package");
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"dom-helpers","exports":{"./*":{"import":{"default":"./esm/*.js"}}}}"#,
        )
        .expect("write package");
        std::fs::write(
            package_dir.join("esm/querySelectorAll.js"),
            "export default null;",
        )
        .expect("write js");

        assert_eq!(
            resolve_dep_route_file(root, "dom-helpers/querySelectorAll"),
            Some(package_dir.join("esm/querySelectorAll.js"))
        );
    }
}
// </HANDWRITE>

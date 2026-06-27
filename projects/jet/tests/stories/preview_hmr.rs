// <HANDWRITE gap="missing-generator:unit-test:41fd209d" tracker="standardize-gap-projects-jet-tests-stories-preview-hmr-rs" reason="Tests: a changed module yields the correct invalidation set; the preview HTML includes the HMR client; a react-refresh-compatible vs incompatible edit routes to patch vs full reload; the manager shell is not reloaded.">
//! Integration tests for B2b (#176): HMR + state-preserving React refresh for
//! the `jet stories` preview frame.
//!
//! These exercise the public stories HMR surface
//! ([`jet::stories::hmr`]) plus the preview/manager HTML rendering, covering:
//! (a) `render_preview_html` injects the HMR client script + WS route, while the
//!     manager shell does NOT connect to that WS (manager untouched),
//! (b) the affected-module computation returns the changed module + its
//!     importers for a small fixture graph,
//! (c) classification routes a react-refresh-compatible (`.tsx`) edit to a
//!     state-preserving patch and an incompatible (`.ts`) edit to a full reload,
//! (d) the live WS broadcast → preview-frame message flow actually delivers
//!     (the loop that drives live reload on edit).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use jet::dev_server::module_graph::ModuleGraph;
use jet::stories::hmr::{
    self, affected_modules, classify_update, message_for_change, StoriesHmrManager,
    StoriesHmrMessage, UpdateKind, STORIES_HMR_ROUTE,
};
use jet::stories::manager::{render_manager_html, render_preview_html};
use jet::stories::{StoryEntry, StoryIndex};

fn entry(id: &str, name: &str, file: &str) -> StoryEntry {
    StoryEntry {
        id: id.to_string(),
        name: name.to_string(),
        export_name: name.to_string(),
        args: BTreeMap::new(),
        has_render: false,
        file: PathBuf::from(file),
        title_path: vec!["Components".to_string(), "Button".to_string()],
    }
}

// ── (a) The preview frame carries the HMR client + WS route ──────────────────

#[test]
fn preview_html_includes_hmr_client_and_ws_route() {
    let story = entry(
        "components-button--primary",
        "Primary",
        "/x/Button.stories.tsx",
    );
    let html = render_preview_html(&story, "/src/components/Button.stories.tsx");

    // The HMR client connects to the stories-scoped WS route...
    assert!(
        html.contains(STORIES_HMR_ROUTE),
        "preview must reference the stories HMR WS route ({STORIES_HMR_ROUTE}): {html}"
    );
    assert_eq!(STORIES_HMR_ROUTE, "/__jet_stories_hmr");
    // ...via a real WebSocket connection...
    assert!(
        html.contains("new WebSocket"),
        "preview must open a WebSocket for HMR"
    );
    // ...and exposes the in-place re-render hook the client drives on `update`.
    assert!(
        html.contains("__jetStoriesRender"),
        "preview must expose the render hook for state-preserving updates"
    );
    // The reload fallback reloads only THIS frame (location.reload inside iframe).
    assert!(
        html.contains("location.reload()"),
        "preview must have a reload fallback"
    );
}

// ── (a') The MANAGER shell is untouched: no preview HMR hook, no whole-page reload ──

#[test]
fn manager_shell_does_not_connect_to_preview_hmr() {
    let mut index = StoryIndex::default();
    index.stories.push(entry(
        "components-button--primary",
        "Primary",
        "/x/Button.stories.tsx",
    ));
    let html = render_manager_html(&index, None, &[]);

    // The manager must NOT open the preview HMR WebSocket — only the iframe does.
    assert!(
        !html.contains(STORIES_HMR_ROUTE),
        "manager shell must not connect to the preview HMR WS: {html}"
    );
    assert!(
        !html.contains("new WebSocket"),
        "manager shell must not open any HMR WebSocket"
    );
    // And it must not carry a whole-page reload hook — the manager stays put.
    assert!(
        !html.contains("location.reload()"),
        "manager shell must never reload the whole page on edit"
    );
}

// ── (b) Affected-module computation: changed module + its importers ──────────

#[test]
fn affected_modules_returns_changed_plus_importers() {
    // Story imports the component; editing the component must invalidate both.
    let mut graph = ModuleGraph::new();
    graph.add_module(
        "/src/Button.stories.tsx",
        "/abs/Button.stories.tsx",
        &["/src/Button.tsx".to_string()],
    );
    graph.add_module("/src/Button.tsx", "/abs/Button.tsx", &[]);

    let affected = affected_modules(&graph, "/src/Button.tsx");
    assert_eq!(
        affected[0], "/src/Button.tsx",
        "the changed module is always first"
    );
    assert!(
        affected.contains(&"/src/Button.stories.tsx".to_string()),
        "the importing story must be in the affected set: {affected:?}"
    );
}

#[test]
fn affected_modules_walks_barrel_cascade() {
    // story -> barrel -> leaf: editing the leaf invalidates barrel AND story.
    let mut graph = ModuleGraph::new();
    graph.add_module(
        "/src/Foo.stories.tsx",
        "/abs/Foo.stories.tsx",
        &["/src/index.ts".to_string()],
    );
    graph.add_module(
        "/src/index.ts",
        "/abs/index.ts",
        &["/src/Foo.tsx".to_string()],
    );
    graph.add_module("/src/Foo.tsx", "/abs/Foo.tsx", &[]);

    let mut affected = affected_modules(&graph, "/src/Foo.tsx");
    affected.sort();
    assert_eq!(
        affected,
        vec![
            "/src/Foo.stories.tsx".to_string(),
            "/src/Foo.tsx".to_string(),
            "/src/index.ts".to_string(),
        ]
    );
}

// ── (c) react-refresh-compatible vs incompatible routes patch vs reload ─────

#[test]
fn component_edit_classifies_as_patch() {
    assert_eq!(classify_update("/src/Button.tsx"), UpdateKind::Patch);
    assert_eq!(classify_update("/src/Card.jsx"), UpdateKind::Patch);
}

#[test]
fn non_component_edit_classifies_as_reload() {
    assert_eq!(classify_update("/src/theme.ts"), UpdateKind::Reload);
    assert_eq!(classify_update("/src/data.js"), UpdateKind::Reload);
}

#[test]
fn message_routes_patch_to_update_and_reload_to_reload() {
    let mut graph = ModuleGraph::new();
    graph.add_module(
        "/src/Button.stories.tsx",
        "/abs/Button.stories.tsx",
        &["/src/Button.tsx".to_string()],
    );
    graph.add_module("/src/Button.tsx", "/abs/Button.tsx", &[]);

    // A `.tsx` component edit → state-preserving `update` carrying the affected set.
    match message_for_change(&graph, "/src/Button.tsx", 1234) {
        StoriesHmrMessage::Update {
            path,
            timestamp,
            affected,
        } => {
            assert_eq!(path, "/src/Button.tsx");
            assert_eq!(timestamp, 1234);
            assert!(
                affected.contains(&"/src/Button.stories.tsx".to_string()),
                "update carries the importing story: {affected:?}"
            );
        }
        other => panic!("expected Update for a .tsx edit, got {other:?}"),
    }

    // A `.ts` helper edit → full preview reload (safe fallback).
    match message_for_change(&graph, "/src/theme.ts", 1234) {
        StoriesHmrMessage::Reload { reason } => {
            assert!(
                reason.contains("/src/theme.ts"),
                "reason names the file: {reason}"
            );
        }
        other => panic!("expected Reload for a .ts edit, got {other:?}"),
    }
}

#[test]
fn wire_messages_serialize_to_kebab_tagged_json() {
    let update = StoriesHmrMessage::Update {
        path: "/src/Button.tsx".into(),
        timestamp: 9,
        affected: vec!["/src/Button.stories.tsx".into()],
    };
    let v: serde_json::Value = serde_json::from_str(&update.to_json()).unwrap();
    assert_eq!(v["type"], "update");
    assert_eq!(v["path"], "/src/Button.tsx");
    assert_eq!(v["affected"][0], "/src/Button.stories.tsx");

    let reload = StoriesHmrMessage::Reload {
        reason: "non-component".into(),
    };
    let v: serde_json::Value = serde_json::from_str(&reload.to_json()).unwrap();
    assert_eq!(v["type"], "reload");
}

// ── (d) The live broadcast → preview-frame delivery loop actually works ──────

#[tokio::test]
async fn broadcast_delivers_update_to_subscribed_preview() {
    // This is the heart of live-reload-on-edit: a watcher broadcast must reach
    // the connected preview frame's subscriber.
    let hmr = StoriesHmrManager::new();
    let mut frame_a = hmr.subscribe();
    let mut frame_b = hmr.subscribe();
    assert_eq!(hmr.subscriber_count(), 2);

    let mut graph = ModuleGraph::new();
    graph.add_module(
        "/src/Button.stories.tsx",
        "/abs/Button.stories.tsx",
        &["/src/Button.tsx".to_string()],
    );
    graph.add_module("/src/Button.tsx", "/abs/Button.tsx", &[]);

    let msg = message_for_change(&graph, "/src/Button.tsx", 555);
    hmr.broadcast(msg.clone());

    // Every connected preview frame receives the same hot-update.
    assert_eq!(frame_a.recv().await.expect("frame a receives"), msg);
    assert_eq!(frame_b.recv().await.expect("frame b receives"), msg);
}

// ── register_served_module wires the importer edge the route relies on ──────

#[test]
fn register_served_module_links_story_to_component() {
    use std::sync::{Arc, RwLock};

    let graph = Arc::new(RwLock::new(ModuleGraph::new()));
    // Serving the story registers its `./Button` -> /src/Button.tsx edge.
    hmr::register_served_module(
        &graph,
        "/src/Button.stories.tsx",
        Path::new("/abs/Button.stories.tsx"),
        &["/src/Button.tsx".to_string()],
    );

    let g = graph.read().unwrap();
    let affected = affected_modules(&g, "/src/Button.tsx");
    assert!(
        affected.contains(&"/src/Button.stories.tsx".to_string()),
        "editing the served component must reach the story that imported it: {affected:?}"
    );
}

// ════════════════════════════════════════════════════════════════════════════
// #196: state-preserving React Fast Refresh wiring for the preview.
//
// A real browser hook-state assertion is out of reach in a Rust test, so these
// assert the WIRING that makes hook-state preservation work:
//   (a) a preview-SERVED `.tsx` module carries the transform's $RefreshReg$ /
//       $RefreshSig$ registration (so component families register at import),
//   (b) the server serves the `/@react-refresh` runtime the modules import,
//   (c) the dev preview HTML wires that runtime + calls performReactRefresh on
//       an `update`, and full-reloads only on an incompatible `reload`,
//   (d) the manager shell carries NONE of the refresh runtime / reload wiring.
// ════════════════════════════════════════════════════════════════════════════

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use jet::stories::discover;
use jet::stories::server::{self, REACT_REFRESH_ROUTE};
use tempfile::TempDir;
use tower::ServiceExt; // for `oneshot`

/// A genuine React component module: an uppercase function returning JSX and
/// using a hook — so the transform injects BOTH `$RefreshReg$` (component
/// family registration) and `$RefreshSig$` (hook-order signature).
const COUNTER_TSX: &str = r#"
import React, { useState } from 'react';

export function Counter() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
"#;

const COUNTER_STORIES: &str = r#"
import { Counter } from './Counter';

export default {
  title: 'Components/Counter',
  component: Counter,
};

export const Default = { args: {} };
"#;

fn write_counter_fixtures() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    let root = dir.path();
    write_file(root.join("src/Counter.tsx"), COUNTER_TSX);
    write_file(root.join("src/Counter.stories.tsx"), COUNTER_STORIES);
    dir
}

fn write_file(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, contents).expect("write fixture");
}

fn router_for(root: &Path) -> axum::Router {
    let index = discover(root);
    server::build_router(index, root.to_path_buf())
}

async fn get(router: &axum::Router, path: &str) -> (StatusCode, String) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .expect("body bytes");
    (status, String::from_utf8_lossy(&bytes).to_string())
}

// ── (a) preview-served `.tsx` module carries $RefreshReg$/$RefreshSig$ ───────

#[tokio::test]
async fn served_module_is_instrumented_with_react_refresh() {
    let dir = write_counter_fixtures();
    let router = router_for(dir.path());

    let (status, js) = get(&router, "/src/Counter.tsx").await;
    assert_eq!(
        status,
        StatusCode::OK,
        "module route serves the component JS"
    );

    // The transform's React Fast Refresh preamble + registration are present, so
    // importing this module registers the `Counter` family with the runtime.
    assert!(
        js.contains("RefreshRuntime"),
        "served module imports the refresh runtime: {js}"
    );
    assert!(
        js.contains("$RefreshReg$(Counter, \"Counter\")"),
        "served module registers the Counter component family: {js}"
    );
    assert!(
        js.contains("$RefreshSig$()"),
        "served module emits the hook signature for state-order stability: {js}"
    );
    assert!(
        js.contains("RefreshRuntime.enqueueUpdate()"),
        "served module schedules a refresh after registration: {js}"
    );
    // The runtime is imported from the same endpoint the preview serves.
    assert!(
        js.contains("'/@react-refresh'") || js.contains("\"/@react-refresh\""),
        "refresh runtime import targets the served endpoint: {js}"
    );
}

// ── (b) the server serves the react-refresh runtime the modules import ───────

#[tokio::test]
async fn react_refresh_runtime_endpoint_is_served() {
    let dir = write_counter_fixtures();
    let router = router_for(dir.path());

    assert_eq!(REACT_REFRESH_ROUTE, "/@react-refresh");
    let (status, js) = get(&router, REACT_REFRESH_ROUTE).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the /@react-refresh runtime is served"
    );

    // It exposes the API the preview wiring drives.
    assert!(
        js.contains("performReactRefresh"),
        "runtime exposes performReactRefresh"
    );
    assert!(
        js.contains("onPerformReactRefresh"),
        "runtime exposes the host refresh hook"
    );
    assert!(
        js.contains("createSignatureFunctionForTransform"),
        "runtime exposes the signature factory the modules use"
    );
}

// ── (c) the dev preview wires the runtime + performReactRefresh on update ────

#[test]
fn preview_html_wires_react_refresh_runtime_and_performs_refresh_on_update() {
    let story = entry(
        "components-counter--default",
        "Default",
        "/x/Counter.stories.tsx",
    );
    let html = render_preview_html(&story, "/src/Counter.stories.tsx");

    // Loads the refresh runtime BEFORE the story module and installs the
    // $RefreshReg$/$RefreshSig$ globals the served modules expect.
    assert!(
        html.contains("import RefreshRuntime from \"/@react-refresh\""),
        "preview loads the react-refresh runtime: {html}"
    );
    assert!(
        html.contains("window.$RefreshReg$ = RefreshRuntime.register"),
        "preview installs the global $RefreshReg$ hook"
    );
    assert!(
        html.contains("RefreshRuntime.createSignatureFunctionForTransform"),
        "preview installs the global $RefreshSig$ factory"
    );
    // The runtime is loaded before the story module import (registry-first).
    let runtime_at = html
        .find("import RefreshRuntime")
        .expect("runtime import present");
    let story_at = html
        .find("import * as Story")
        .expect("story import present");
    assert!(
        runtime_at < story_at,
        "the refresh runtime must be set up BEFORE the story module imports"
    );
    // Registers the in-place refresh callback the runtime drives.
    assert!(
        html.contains("RefreshRuntime.onPerformReactRefresh"),
        "preview registers an in-place refresh callback"
    );
    // On an `update`, the HMR client drives performReactRefresh (state-preserving),
    // NOT a blind remount.
    assert!(
        html.contains("performReactRefresh"),
        "preview HMR client calls performReactRefresh on update: {html}"
    );
    // Incompatible edits still full-reload THIS frame only.
    assert!(
        html.contains("location.reload()"),
        "preview keeps a full-reload fallback for incompatible edits"
    );
    assert!(
        html.contains("case \"reload\":"),
        "reload message branch present"
    );
    assert!(
        html.contains("case \"update\":"),
        "update message branch present"
    );
}

// ── (d) the manager shell carries NONE of the refresh wiring ─────────────────

#[test]
fn manager_shell_has_no_react_refresh_runtime_or_reload() {
    let mut index = StoryIndex::default();
    index.stories.push(entry(
        "components-counter--default",
        "Default",
        "/x/Counter.stories.tsx",
    ));
    let html = render_manager_html(&index, None, &[]);

    assert!(
        !html.contains("/@react-refresh"),
        "manager shell must not load the react-refresh runtime: {html}"
    );
    assert!(
        !html.contains("RefreshRuntime"),
        "manager shell must not reference the refresh runtime"
    );
    assert!(
        !html.contains("performReactRefresh"),
        "manager shell must not perform react refresh"
    );
    assert!(
        !html.contains("location.reload()"),
        "manager shell must never reload the whole page"
    );
}
// </HANDWRITE>

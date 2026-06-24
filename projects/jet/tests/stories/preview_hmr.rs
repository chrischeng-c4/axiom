// HANDWRITE-BEGIN gap="missing-generator:unit-test:41fd209d" tracker="pending-tracker" reason="Tests: a changed module yields the correct invalidation set; the preview HTML includes the HMR client; a react-refresh-compatible vs incompatible edit routes to patch vs full reload; the manager shell is not reloaded."
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
    let story = entry("components-button--primary", "Primary", "/x/Button.stories.tsx");
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
            assert!(reason.contains("/src/theme.ts"), "reason names the file: {reason}");
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
// HANDWRITE-END

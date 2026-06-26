// HANDWRITE-BEGIN gap="missing-generator:logic:071383a9" tracker="standardize-gap-projects-jet-src-stories-hmr-rs" reason="Preview HMR: a file watcher + WS endpoint that, on a story/component edit, computes the affected module set (dependents_of), invalidates the module cache, and pushes an HMR message; reuses the dev_server hmr_client + react_refresh runtime so compatible edits are state-preserving and incompatible edits fall back to a full preview reload."
//! HMR + state-preserving React refresh for the `jet stories` preview (B2b/#176).
//!
//! This module is the stories-scoped twin of [`crate::dev_server::hmr`], but
//! deliberately *small*: the stories workbench only needs to hot-update the
//! isolated **preview frame**, never the manager shell. It owns:
//!
//! 1. [`StoriesHmrMessage`] — the server→client wire messages (`connected`,
//!    `update`, `reload`) sent over the `/__jet_stories_hmr` WebSocket.
//! 2. [`StoriesHmrManager`] — a thin broadcast hub (one [`tokio::sync::broadcast`]
//!    channel) the watcher writes to and each connected preview frame reads from.
//! 3. [`affected_modules`] — given a changed module URL and the served-module
//!    import graph, the set of module URLs whose preview output is now stale
//!    (the changed module + its transitive importers). Reuses
//!    [`crate::dev_server::module_graph::ModuleGraph::dependents_of`].
//! 4. [`classify_update`] — routes a changed module to a state-preserving
//!    react-refresh **patch** vs a full preview **reload**, based on whether the
//!    changed module is a React component module (`.tsx`/`.jsx`-shaped; the
//!    isolated single-component preview needs no other boundary).
//! 5. [`spawn_watcher`] — wires a [`crate::dev_server::watcher::FileWatcher`]
//!    over the project root to the broadcaster, translating each file event into
//!    the affected-module set + a classified message.
//!
//! The pure functions ([`affected_modules`], [`classify_update`],
//! [`message_for_change`], message serialization) are factored out so tests need
//! no real file events.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::dev_server::module_graph::ModuleGraph;
use crate::dev_server::watcher::FileWatcher;

/// WebSocket route the preview frame's HMR client connects to. Deliberately
/// distinct from the dev server's `/__jet_hmr` so the two never collide.
pub const STORIES_HMR_ROUTE: &str = "/__jet_stories_hmr";

/// How a changed module should be applied to the live preview frame.
///
/// Distinct from the wire message: this is the *decision*, computed server-side
/// from the changed module, that selects which [`StoriesHmrMessage`] to send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateKind {
    /// State-preserving React Fast Refresh: re-import the module and let the
    /// react-refresh runtime swap the component family in place, keeping hook
    /// state. Chosen for React component modules (`.tsx` / `.jsx`).
    Patch,
    /// Full preview-frame reload (`location.reload()` *inside the iframe*). The
    /// safe fallback for non-component edits or when react-refresh can't apply.
    Reload,
}

impl UpdateKind {
    /// `true` when this is the state-preserving react-refresh path.
    pub fn is_patch(self) -> bool {
        matches!(self, UpdateKind::Patch)
    }
}

/// Server → preview-frame messages over `/__jet_stories_hmr`.
///
/// Mirrors the shape of [`crate::dev_server::hmr::HmrMessage`] but trimmed to
/// what the isolated preview needs. Tagged `type` with kebab-case names so the
/// browser client can `switch (msg.type)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StoriesHmrMessage {
    /// Initial connection acknowledgement (no-op on the client).
    Connected,
    /// State-preserving react-refresh update (#196): re-import `path`
    /// (cache-busted by `timestamp`) — which re-runs the module's
    /// transform-injected `$RefreshReg$(...)` registration — then drive
    /// `RefreshRuntime.performReactRefresh()` so the preview re-renders the
    /// EXISTING root in place, preserving component hook state.
    /// `affected` is the changed module plus its transitive importers, so the
    /// client can also re-import re-exporting barrels that feed the story.
    Update {
        path: String,
        timestamp: u64,
        #[serde(default)]
        affected: Vec<String>,
    },
    /// Full preview-frame reload — the safe fallback. `reason` is for logging.
    Reload { reason: String },
}

impl StoriesHmrMessage {
    /// Serialize to the compact JSON the WS sends. Infallible for these
    /// variants (no non-string map keys), so callers don't need to thread a
    /// `Result` through the broadcast path.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Unreachable for these variants; degrade to a reload so the
            // preview still recovers rather than silently dropping the event.
            r#"{"type":"reload","reason":"serialization-failed"}"#.to_string()
        })
    }
}

/// Broadcast hub: the watcher task pushes [`StoriesHmrMessage`]s in, each
/// connected preview-frame WebSocket subscribes and forwards them to its client.
///
/// A clone-cheap wrapper over a [`broadcast::Sender`]; cloning shares the
/// channel (so the watcher and the router hold the same hub).
#[derive(Clone)]
pub struct StoriesHmrManager {
    tx: broadcast::Sender<StoriesHmrMessage>,
}

impl StoriesHmrManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    /// Push a message to every connected preview frame. A lagging/absent
    /// subscriber is not an error (the send result is intentionally ignored).
    pub fn broadcast(&self, message: StoriesHmrMessage) {
        let _ = self.tx.send(message);
    }

    /// Subscribe a new preview-frame WebSocket to the broadcast stream.
    pub fn subscribe(&self) -> broadcast::Receiver<StoriesHmrMessage> {
        self.tx.subscribe()
    }

    /// Number of currently-connected preview frames (test/diagnostic aid).
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for StoriesHmrManager {
    fn default() -> Self {
        Self::new()
    }
}

/// The set of module URLs whose preview output is stale after `changed_url`
/// is edited: the changed module itself **plus its transitive importers**.
///
/// Reuses [`ModuleGraph::dependents_of`] (which excludes `changed_url`) and
/// prepends the changed module so the result is the full invalidation set the
/// preview frame must re-import. The changed module is always first; importers
/// follow in BFS order.
///
/// When the graph has no record of `changed_url` (the preview imported it but
/// we never registered an edge — e.g. the very first edit before any import was
/// observed) the result is just `[changed_url]`, which still drives a correct
/// single-module update.
///
/// TODO(#176 follow-up): the stories server currently feeds this a graph it
/// builds lazily from served module requests; a story that imports a component
/// which is edited *before* the component was ever served will only see the
/// changed module, not the importing story. Building the graph eagerly at
/// discovery time (parsing each story file's relative imports) would close that
/// gap.
pub fn affected_modules(graph: &ModuleGraph, changed_url: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(1);
    out.push(changed_url.to_string());
    for importer in graph.dependents_of(changed_url) {
        if importer != changed_url {
            out.push(importer);
        }
    }
    out
}

/// Decide whether a changed module hot-patches via react-refresh or forces a
/// full preview reload.
///
/// For the isolated single-component preview the rule is simple: a React
/// component module (`.tsx` / `.jsx`) is react-refresh-compatible and patches;
/// anything else (`.ts`/`.js` utilities, JSON, config) forces a reload because
/// it has no component family the refresh runtime can swap in place.
///
/// This is intentionally coarser than the dev server's full
/// [`crate::dev_server::module_graph::ModuleGraph::find_hmr_boundary`] walk:
/// the preview has exactly one mounted component, so there is no multi-boundary
/// graph to honor.
///
/// TODO(#176 follow-up): inspect the transformed module for actual
/// `$RefreshReg$` instrumentation (a module can be `.tsx` yet export only
/// non-component values) instead of trusting the extension, and consult the
/// changed module's importer chain so editing a shared `.ts` helper that only
/// feeds component modules can still patch rather than reload.
pub fn classify_update(changed_url: &str) -> UpdateKind {
    if is_react_component_module(changed_url) {
        UpdateKind::Patch
    } else {
        UpdateKind::Reload
    }
}

/// `true` for `.tsx` / `.jsx` module URLs — the react-refresh-eligible shapes.
/// Tolerates a trailing `?t=...` cache-bust query.
fn is_react_component_module(url: &str) -> bool {
    let path = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    path.ends_with(".tsx") || path.ends_with(".jsx")
}

/// Build the [`StoriesHmrMessage`] for a changed module, given the served-module
/// graph. Pure: no I/O, no clock — `timestamp` is injected so tests are
/// deterministic.
///
/// `Patch` edits emit an `update` carrying the affected-module set; `Reload`
/// edits emit a `reload`. This is the single place the classification + the
/// affected-set computation are combined into a wire message.
pub fn message_for_change(
    graph: &ModuleGraph,
    changed_url: &str,
    timestamp: u64,
) -> StoriesHmrMessage {
    match classify_update(changed_url) {
        UpdateKind::Patch => StoriesHmrMessage::Update {
            path: changed_url.to_string(),
            timestamp,
            affected: affected_modules(graph, changed_url),
        },
        UpdateKind::Reload => StoriesHmrMessage::Reload {
            reason: format!("non-component module changed: {changed_url}"),
        },
    }
}

/// Map a changed filesystem path to a root-relative module URL (`/src/Foo.tsx`),
/// matching the URL space the module route + preview imports use.
///
/// Mirrors [`crate::stories::server`]'s `module_url_for`, kept here so the
/// watcher task can translate `notify` paths without reaching into the server's
/// private helper.
pub fn module_url_for(root: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let mut url = String::from("/");
    url.push_str(rel_str.trim_start_matches('/'));
    url
}

/// `true` for the source extensions the preview can hot-update at all
/// (everything else — images, JSON, lockfiles — is ignored by the watcher).
fn is_watchable_module(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("ts" | "tsx" | "js" | "jsx")
    )
}

/// Current wall-clock as a millisecond cache-busting timestamp.
///
/// Falls back to `0` if the host clock is before the UNIX epoch (matching the
/// dev server's clock-skew survival, GH #3680) rather than panicking inside the
/// watcher task.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Wire a file watcher over `root` to `hmr`: each watched source change becomes
/// an affected-module computation + a broadcast message that updates ONLY the
/// preview frame.
///
/// `graph` is the shared served-module import graph (the server populates it as
/// it transforms modules on demand). The returned [`FileWatcher`] must be kept
/// alive for watching to continue (dropping it stops the `notify` backend), so
/// the caller holds it for the server's lifetime.
///
/// The spawned task lives as long as the broadcast channel has the watcher's
/// sender; it exits cleanly when the watcher is dropped (the `recv` returns
/// `Closed`).
pub fn spawn_watcher(
    root: &Path,
    graph: Arc<RwLock<ModuleGraph>>,
    hmr: StoriesHmrManager,
) -> anyhow::Result<FileWatcher> {
    let watcher = FileWatcher::new(root.to_path_buf())?;
    let mut rx = watcher.subscribe();
    let root: PathBuf = root.to_path_buf();

    tokio::spawn(async move {
        while let Ok(path) = rx.recv().await {
            if !is_watchable_module(&path) {
                continue;
            }
            let module_url = module_url_for(&root, &path);
            let timestamp = now_ms();

            let message = {
                let g = match graph.read() {
                    Ok(g) => g,
                    // A poisoned lock means another task panicked; recover the
                    // inner graph rather than poisoning the watcher loop too.
                    Err(poisoned) => poisoned.into_inner(),
                };
                message_for_change(&g, &module_url, timestamp)
            };

            tracing::info!(
                target: "jet::stories::hmr",
                "preview change {module_url} -> {message:?}"
            );
            hmr.broadcast(message);
        }
    });

    Ok(watcher)
}

/// Register/refresh a served module's import edges in the shared graph so
/// [`affected_modules`] can walk importers. Called by the server's module route
/// after it transforms a module and extracts its relative imports.
///
/// `imports` are already root-relative module URLs (the caller resolves them).
pub fn register_served_module(
    graph: &Arc<RwLock<ModuleGraph>>,
    url: &str,
    file: &Path,
    imports: &[String],
) {
    let mut g = match graph.write() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let file_str = file.to_string_lossy().to_string();
    g.update_module(url, &file_str, imports);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_with(edges: &[(&str, &[&str])]) -> ModuleGraph {
        let mut g = ModuleGraph::new();
        for (url, imports) in edges {
            let owned: Vec<String> = imports.iter().map(|s| s.to_string()).collect();
            g.add_module(url, &format!("/abs{url}"), &owned);
        }
        g
    }

    #[test]
    fn affected_includes_changed_module_first() {
        // story imports the component; editing the component affects the story.
        let g = graph_with(&[
            ("/Button.stories.tsx", &["/Button.tsx"]),
            ("/Button.tsx", &[]),
        ]);
        let affected = affected_modules(&g, "/Button.tsx");
        assert_eq!(affected[0], "/Button.tsx", "changed module is first");
        assert!(
            affected.contains(&"/Button.stories.tsx".to_string()),
            "the importing story is in the affected set: {affected:?}"
        );
    }

    #[test]
    fn affected_unknown_module_is_just_itself() {
        let g = ModuleGraph::new();
        let affected = affected_modules(&g, "/Lonely.tsx");
        assert_eq!(affected, vec!["/Lonely.tsx".to_string()]);
    }

    #[test]
    fn affected_walks_barrel_cascade() {
        // story -> barrel -> leaf. Editing the leaf must affect barrel + story.
        let g = graph_with(&[
            ("/Foo.stories.tsx", &["/index.ts"]),
            ("/index.ts", &["/Foo.tsx"]),
            ("/Foo.tsx", &[]),
        ]);
        let mut affected = affected_modules(&g, "/Foo.tsx");
        affected.sort();
        assert_eq!(
            affected,
            vec![
                "/Foo.stories.tsx".to_string(),
                "/Foo.tsx".to_string(),
                "/index.ts".to_string(),
            ]
        );
    }

    #[test]
    fn classify_component_module_patches() {
        assert_eq!(classify_update("/Button.tsx"), UpdateKind::Patch);
        assert_eq!(classify_update("/Card.jsx"), UpdateKind::Patch);
        assert!(classify_update("/Button.tsx").is_patch());
    }

    #[test]
    fn classify_non_component_module_reloads() {
        assert_eq!(classify_update("/utils.ts"), UpdateKind::Reload);
        assert_eq!(classify_update("/data.js"), UpdateKind::Reload);
        assert!(!classify_update("/utils.ts").is_patch());
    }

    #[test]
    fn classify_ignores_cache_bust_query() {
        // A cache-busted URL still classifies by its real extension.
        assert_eq!(classify_update("/Button.tsx?t=123"), UpdateKind::Patch);
        assert_eq!(classify_update("/utils.ts?t=123"), UpdateKind::Reload);
    }

    #[test]
    fn message_for_patch_emits_update_with_affected() {
        let g = graph_with(&[
            ("/Button.stories.tsx", &["/Button.tsx"]),
            ("/Button.tsx", &[]),
        ]);
        let msg = message_for_change(&g, "/Button.tsx", 42);
        match msg {
            StoriesHmrMessage::Update {
                path,
                timestamp,
                affected,
            } => {
                assert_eq!(path, "/Button.tsx");
                assert_eq!(timestamp, 42);
                assert!(affected.contains(&"/Button.stories.tsx".to_string()));
            }
            other => panic!("expected Update, got {other:?}"),
        }
    }

    #[test]
    fn message_for_non_component_emits_reload() {
        let g = ModuleGraph::new();
        let msg = message_for_change(&g, "/theme.ts", 7);
        match msg {
            StoriesHmrMessage::Reload { reason } => {
                assert!(
                    reason.contains("/theme.ts"),
                    "reason names the file: {reason}"
                );
            }
            other => panic!("expected Reload, got {other:?}"),
        }
    }

    #[test]
    fn message_serializes_to_tagged_json() {
        let update = StoriesHmrMessage::Update {
            path: "/Button.tsx".into(),
            timestamp: 99,
            affected: vec!["/Button.stories.tsx".into()],
        };
        let json = update.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "update");
        assert_eq!(parsed["path"], "/Button.tsx");
        assert_eq!(parsed["timestamp"], 99);
        assert_eq!(parsed["affected"][0], "/Button.stories.tsx");

        let reload = StoriesHmrMessage::Reload { reason: "x".into() };
        let parsed: serde_json::Value = serde_json::from_str(&reload.to_json()).unwrap();
        assert_eq!(parsed["type"], "reload");
        assert_eq!(parsed["reason"], "x");

        let connected = StoriesHmrMessage::Connected;
        let parsed: serde_json::Value = serde_json::from_str(&connected.to_json()).unwrap();
        assert_eq!(parsed["type"], "connected");
    }

    #[test]
    fn module_url_for_is_root_relative() {
        assert_eq!(
            module_url_for(Path::new("/proj"), Path::new("/proj/src/Button.tsx")),
            "/src/Button.tsx"
        );
    }

    #[test]
    fn is_watchable_module_filters_non_source() {
        assert!(is_watchable_module(Path::new("/x/Button.tsx")));
        assert!(is_watchable_module(Path::new("/x/util.ts")));
        assert!(!is_watchable_module(Path::new("/x/logo.png")));
        assert!(!is_watchable_module(Path::new("/x/data.json")));
    }

    #[tokio::test]
    async fn manager_broadcast_reaches_subscriber() {
        let hmr = StoriesHmrManager::new();
        let mut rx = hmr.subscribe();
        hmr.broadcast(StoriesHmrMessage::Connected);
        let got = rx.recv().await.expect("subscriber receives");
        assert_eq!(got, StoriesHmrMessage::Connected);
    }

    #[test]
    fn register_served_module_builds_importer_edges() {
        let graph = Arc::new(RwLock::new(ModuleGraph::new()));
        register_served_module(
            &graph,
            "/Button.stories.tsx",
            Path::new("/abs/Button.stories.tsx"),
            &["/Button.tsx".to_string()],
        );
        let g = graph.read().unwrap();
        let affected = affected_modules(&g, "/Button.tsx");
        assert!(affected.contains(&"/Button.stories.tsx".to_string()));
    }
}
// HANDWRITE-END

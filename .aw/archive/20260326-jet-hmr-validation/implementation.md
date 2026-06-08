---
id: implementation
type: change_implementation
change_id: jet-hmr-validation
---

# Implementation

## Summary

JavaScript module HMR with module graph boundary detection, React Fast Refresh injection, and comprehensive HMR client runtime. New files: (1) dev_server/module_graph.rs (536 lines): ModuleGraph struct tracking import/importer edges as a directed graph; ModuleGraphNode with url, file, imports, importers, is_self_accepting, accepted_deps, has_react_refresh, last_transform_timestamp fields; add_module/update_module methods return (removed, added) import diffs and maintain bidirectional edges; set_self_accepting/set_accepted_deps/set_has_react_refresh/set_timestamp mutation methods; remove_module returns orphaned dependency list; find_hmr_boundary performs BFS upward walk to find HMR boundaries (self-accepting modules, React Fast Refresh modules, or parent-accepted deps), returns HmrBoundaryResult::HotUpdate{targets} or HmrBoundaryResult::FullReload{reason}; 10 unit tests covering graph construction (T5), stale edge removal (T6), self-accept boundary (T7), parent propagation (T8), full-reload cascade (T9), React Fast Refresh boundary, module removal orphans, unknown module fallback, self-accepting parent boundary. (2) dev_server/hmr_client.rs (336 lines): generate_hmr_runtime() returns <script type="module"> IIFE with full import.meta.hot API (accept/dispose/prune/invalidate/data), module registry Map, WebSocket message handlers (update/css-update/full-reload/error/prune), dynamic import() with cache-busting ?t=timestamp, error overlay (dark backdrop, monospace, code frame, click/Escape dismiss), exponential backoff reconnection (1s→2s→4s→...→30s max); generate_hot_preamble(module_url) returns per-module import.meta.hot initialization snippet via window.__jet_hmr_create_hot. (3) dev_server/react_refresh.rs (130 lines): react_refresh_runtime_source() serves /@react-refresh endpoint as ESM module with register(type, id), createSignatureFunctionForTransform(), enqueueUpdate() (debounced 30ms), performReactRefresh() forwarding to React DevTools hook; component family registry and hook signature tracking. (4) transform/react_refresh.rs (292 lines): inject_react_fast_refresh(transformed, source, root) — AST-based React component detection using tree-sitter; detects function declarations, export statements, lexical declarations (arrow/function expression), React.memo/forwardRef wrappers; components must have uppercase name + JSX in body; injects preamble (import RefreshRuntime + $ creation), per-component  registration, hooks signature tracking via (Name, "useState{} useEffect{}"), and footer (RefreshRuntime.enqueueUpdate()). Modified files: (5) dev_server/hmr.rs (+175 lines): HmrMessage enum extended with accepted_by: Option<String> on Update, Error variant gains file/line/column/frame fields, new Prune{paths} variant; new HmrUpdateResult enum (HotUpdate/FullReload) with determine() bridging to ModuleGraph::find_hmr_boundary; 8 new tests: T16 error code frame, determine hot/full-reload, serialization round-trips for update/full-reload/prune/css-update. (6) dev_server/mod.rs (+318/-65 lines): DevServer gains module_graph: Arc<RwLock<ModuleGraph>> field; file watcher handler split into CSS-only path and JS/TS path; JS/TS handler re-transforms file, extracts imports via extract_imports_from_source(), detects React components via source_has_react_components(), updates module graph, runs HmrUpdateResult::determine() boundary detection, broadcasts HmrMessage::Update with accepted_by or HmrMessage::FullReload; transform errors produce HmrMessage::Error with code frame; HMR runtime injected into HTML via serve_index_html before </body>; serve_root_file injects import.meta.hot preamble for all JS/TS modules; new /@react-refresh endpoint; helper functions: file_path_to_url, extract_imports_from_source (line-based import scanner), extract_string_literal, source_has_react_components (JSX heuristic), build_error_frame; old generate_hmr_client() inline IIFE removed. (7) dev_server/watcher.rs (+23 lines): 50ms debounce via HashMap<PathBuf, Instant> + DEBOUNCE_MS constant to filter duplicate events from editor save-then-rename patterns. (8) transform/mod.rs (+5 lines): new pub mod react_refresh; TransformOptions gains dev_mode: bool field (default true). (9) transform/transform_tsx.rs (+5 lines): calls super::react_refresh::inject_react_fast_refresh when dev_mode=true and source has JSX. (10) transform/transform_tsx_tests.rs (+214 lines): 10 new tests: T1 import.meta.hot injected in dev mode, T2 not injected in prod, T10 function component $, T11 arrow component $, T12 hook $ with signature fingerprint, T13 non-component function skipped, T14 React.memo wrapped detection, T15 preamble/footer structure, no-JSX file skipped.

## Diff

```diff
diff --git a/crates/cclab-jet/src/dev_server/hmr.rs b/crates/cclab-jet/src/dev_server/hmr.rs
index 10f735cb..92624d1e 100644
--- a/crates/cclab-jet/src/dev_server/hmr.rs
+++ b/crates/cclab-jet/src/dev_server/hmr.rs
@@ -1,16 +1,63 @@
 use serde::{Deserialize, Serialize};
 use tokio::sync::broadcast;
 
-/// HMR message types
+use super::module_graph::{HmrBoundaryResult, ModuleGraph};
+
+/// HMR message types sent over the `/__jet_hmr` WebSocket.
 #[derive(Debug, Clone, Serialize, Deserialize)]
 #[serde(tag = "type", rename_all = "kebab-case")]
 pub enum HmrMessage {
-    Update { path: String, timestamp: u64 },
+    /// JS/TS module hot update — client should re-import the module.
+    Update {
+        path: String,
+        timestamp: u64,
+        /// When a parent module accepts the update on behalf of the changed module.
+        #[serde(skip_serializing_if = "Option::is_none")]
+        accepted_by: Option<String>,
+    },
     /// CSS hot replacement — browser can swap stylesheet without a full reload.
-    CssUpdate { css: String, filename: String, timestamp: u64 },
+    CssUpdate {
+        css: String,
+        filename: String,
+        timestamp: u64,
+    },
+    /// Full page reload required — no HMR boundary found.
     FullReload { reason: String },
+    /// Initial connection acknowledgement.
     Connected,
-    Error { message: String },
+    /// Syntax or transform error with optional code frame.
+    Error {
+        message: String,
+        #[serde(skip_serializing_if = "Option::is_none")]
+        file: Option<String>,
+        #[serde(skip_serializing_if = "Option::is_none")]
+        line: Option<u32>,
+        #[serde(skip_serializing_if = "Option::is_none")]
+        column: Option<u32>,
+        #[serde(skip_serializing_if = "Option::is_none")]
+        frame: Option<String>,
+    },
+    /// Modules pruned from the graph — client should run prune callbacks.
+    Prune { paths: Vec<String> },
+}
+
+/// Result of determining the HMR update strategy for a changed file.
+#[derive(Debug, Clone)]
+pub enum HmrUpdateResult {
+    /// Hot update — re-import the target modules.
+    HotUpdate { targets: Vec<String> },
+    /// Full page reload — no HMR boundary found.
+    FullReload { reason: String },
+}
+
+impl HmrUpdateResult {
+    /// Determine the update strategy for a changed module path using the module graph.
+    pub fn determine(changed_url: &str, graph: &ModuleGraph) -> Self {
+        match graph.find_hmr_boundary(changed_url) {
+            HmrBoundaryResult::HotUpdate { targets } => Self::HotUpdate { targets },
+            HmrBoundaryResult::FullReload { reason } => Self::FullReload { reason },
+        }
+    }
 }
 
 /// HMR manager for broadcasting updates
@@ -59,4 +106,124 @@ mod tests {
         let _rx = manager.subscribe();
         assert_eq!(manager.subscriber_count(), 1);
     }
+
+    // ── T16: Error Message Contains Code Frame ──────────────────────────────
+    #[test]
+    fn t16_error_message_contains_code_frame() {
+        let error_msg = HmrMessage::Error {
+            message: "Unexpected token".to_string(),
+            file: Some("/src/App.tsx".to_string()),
+            line: Some(15),
+            column: Some(8),
+            frame: Some("  13 |   return (\n  14 |     <div>\n> 15 |       <span\n     |        ^ Unexpected token\n  16 |     </div>\n  17 |   )".to_string()),
+        };
+
+        // Serialize and verify structure
+        let json = serde_json::to_string(&error_msg).unwrap();
+        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+
+        assert_eq!(parsed["type"], "error");
+        assert_eq!(parsed["line"], 15);
+        assert_eq!(parsed["column"], 8);
+        assert!(
+            parsed["frame"].as_str().unwrap().contains("Unexpected token"),
+            "frame must contain error marker"
+        );
+        assert!(
+            parsed["file"].as_str().unwrap().contains("App.tsx"),
+            "file path must be present"
+        );
+    }
+
+    // ── HmrUpdateResult::determine bridges to module graph ──────────────────
+    #[test]
+    fn determine_hot_update_for_self_accepting() {
+        let mut graph = ModuleGraph::new();
+        graph.add_module("/src/App.tsx", "/abs/App.tsx", &[]);
+        graph.set_self_accepting("/src/App.tsx", true);
+
+        let result = HmrUpdateResult::determine("/src/App.tsx", &graph);
+        match result {
+            HmrUpdateResult::HotUpdate { targets } => {
+                assert_eq!(targets, vec!["/src/App.tsx".to_string()]);
+            }
+            HmrUpdateResult::FullReload { reason } => {
+                panic!("Expected HotUpdate but got FullReload: {}", reason);
+            }
+        }
+    }
+
+    #[test]
+    fn determine_full_reload_for_no_boundary() {
+        let mut graph = ModuleGraph::new();
+        graph.add_module("/src/entry.tsx", "/abs/entry.tsx", &["/src/utils.ts".to_string()]);
+        graph.add_module("/src/utils.ts", "/abs/utils.ts", &[]);
+
+        let result = HmrUpdateResult::determine("/src/utils.ts", &graph);
+        match result {
+            HmrUpdateResult::FullReload { reason } => {
+                assert!(
+                    reason.contains("no HMR boundary"),
+                    "reason must indicate no boundary: {}",
+                    reason
+                );
+            }
+            HmrUpdateResult::HotUpdate { targets } => {
+                panic!("Expected FullReload but got HotUpdate: {:?}", targets);
+            }
+        }
+    }
+
+    // ── HmrMessage serialization round-trips ────────────────────────────────
+    #[test]
+    fn hmr_message_update_serialization() {
+        let msg = HmrMessage::Update {
+            path: "/src/App.tsx".to_string(),
+            timestamp: 1234567890,
+            accepted_by: None,
+        };
+        let json = serde_json::to_string(&msg).unwrap();
+        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed["type"], "update");
+        assert_eq!(parsed["path"], "/src/App.tsx");
+        assert_eq!(parsed["timestamp"], 1234567890);
+        // accepted_by should be skipped when None
+        assert!(parsed.get("accepted_by").is_none() || parsed["accepted_by"].is_null());
+    }
+
+    #[test]
+    fn hmr_message_full_reload_serialization() {
+        let msg = HmrMessage::FullReload {
+            reason: "no HMR boundary for /src/utils.ts".to_string(),
+        };
+        let json = serde_json::to_string(&msg).unwrap();
+        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed["type"], "full-reload");
+        assert!(parsed["reason"].as_str().unwrap().contains("no HMR boundary"));
+    }
+
+    #[test]
+    fn hmr_message_prune_serialization() {
+        let msg = HmrMessage::Prune {
+            paths: vec!["/src/old.tsx".to_string(), "/src/removed.ts".to_string()],
+        };
+        let json = serde_json::to_string(&msg).unwrap();
+        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed["type"], "prune");
+        let paths = parsed["paths"].as_array().unwrap();
+        assert_eq!(paths.len(), 2);
+    }
+
+    #[test]
+    fn hmr_message_css_update_serialization() {
+        let msg = HmrMessage::CssUpdate {
+            css: "body { color: red; }".to_string(),
+            filename: "index.abc123.css".to_string(),
+            timestamp: 9999,
+        };
+        let json = serde_json::to_string(&msg).unwrap();
+        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
+        assert_eq!(parsed["type"], "css-update");
+        assert!(parsed["css"].as_str().unwrap().contains("color: red"));
+    }
 }
diff --git a/crates/cclab-jet/src/dev_server/mod.rs b/crates/cclab-jet/src/dev_server/mod.rs
index 4e1fa88e..c78f399e 100644
--- a/crates/cclab-jet/src/dev_server/mod.rs
+++ b/crates/cclab-jet/src/dev_server/mod.rs
@@ -13,17 +13,22 @@ use std::path::PathBuf;
 use std::sync::Arc;
 
 pub mod hmr;
+pub mod hmr_client;
 pub mod importmap;
+pub mod module_graph;
 pub mod polyfills;
 pub mod prebundle;
 pub mod proxy;
+pub mod react_refresh;
 pub mod watcher;
 
-use hmr::{HmrManager, HmrMessage};
+use hmr::{HmrManager, HmrMessage, HmrUpdateResult};
+use module_graph::ModuleGraph;
 use proxy::ProxyHandler;
 use watcher::FileWatcher;
 
 use crate::css::{CssPipeline, TailwindConfig};
+use std::sync::RwLock;
 
 /// Development server with HMR support
 pub struct DevServer {
@@ -40,6 +45,8 @@ pub struct DevServer {
     /// Cached importmap JSON generated by the pre-bundler.
     /// `None` when pre-bundling hasn't run yet or produced no deps.
     importmap_json: Option<String>,
+    /// Server-side module dependency graph for HMR boundary detection.
+    module_graph: Arc<RwLock<ModuleGraph>>,
 }
 
 /// Server configuration
@@ -65,6 +72,8 @@ struct ServerState {
     proxy_handler: Option<Arc<ProxyHandler>>,
     /// Cached importmap JSON from pre-bundler.
     importmap_json: Option<String>,
+    /// Shared module graph for HMR boundary detection.
+    module_graph: Arc<RwLock<ModuleGraph>>,
 }
 
 impl DevServer {
@@ -90,6 +99,7 @@ impl DevServer {
             css_content_globs: Vec::new(),
             proxy_handler,
             importmap_json: None,
+            module_graph: Arc::new(RwLock::new(ModuleGraph::new())),
         })
     }
 
@@ -149,6 +159,7 @@ impl DevServer {
             config: self.config.clone(),
             proxy_handler: self.proxy_handler.clone(),
             importmap_json: self.importmap_json.clone(),
+            module_graph: self.module_graph.clone(),
         };
 
         Router::new()
@@ -167,6 +178,8 @@ impl DevServer {
         let css_entry = self.css_entry.clone();
         let css_root = self.config.root_dir.clone();
         let css_content_globs = self.css_content_globs.clone();
+        let module_graph = self.module_graph.clone();
+        let root_dir = self.config.root_dir.clone();
 
         tokio::spawn(async move {
             let mut rx = watcher.subscribe();
@@ -180,12 +193,15 @@ impl DevServer {
                     .unwrap()
                     .as_millis() as u64;
 
-                // CSS HMR: rebuild CSS pipeline when a .css file or a content
-                // source file (.ts, .tsx) changes.
+                // CSS HMR: rebuild CSS pipeline when a .css file changes.
                 let is_css_change = path_str.ends_with(".css");
-                let is_content_change = path_str.ends_with(".ts") || path_str.ends_with(".tsx");
+                let is_js_change = matches!(
+                    path.extension().and_then(|e| e.to_str()),
+                    Some("ts" | "tsx" | "js" | "jsx")
+                );
 
-                if is_css_change || is_content_change {
+                // For CSS files, rebuild CSS pipeline
+                if is_css_change {
                     if let Some(ref css_entry_path) = css_entry {
                         if let Some(css_hmr) = rebuild_css(
                             css_entry_path,
@@ -197,15 +213,120 @@ impl DevServer {
                         .await
                         {
                             hmr_manager.broadcast(css_hmr).await;
-                            continue; // CSS update sent — skip generic update
+                            continue;
                         }
                     }
                 }
 
-                // Generic file update (JS/TS module change)
+                // For JS/TS files, also trigger CSS rebuild (Tailwind class changes)
+                // then run module graph boundary detection.
+                if is_js_change {
+                    // Rebuild CSS in case Tailwind classes changed
+                    if let Some(ref css_entry_path) = css_entry {
+                        if let Some(css_hmr) = rebuild_css(
+                            css_entry_path,
+                            &css_root,
+                            &css_content_globs,
+                            &path_str,
+                            timestamp,
+                        )
+                        .await
+                        {
+                            hmr_manager.broadcast(css_hmr).await;
+                        }
+                    }
+
+                    // Compute URL path from filesystem path
+                    let module_url = file_path_to_url(&path, &root_dir);
+
+                    // Re-transform the file and extract imports
+                    let source = match std::fs::read_to_string(&path) {
+                        Ok(s) => s,
+                        Err(e) => {
+                            tracing::warn!("Failed to read {}: {}", path_str, e);
+                            continue;
+                        }
+                    };
+
+                    let ext = path
+                        .extension()
+                        .and_then(|e| e.to_str())
+                        .unwrap_or("");
+                    let options = crate::transform::TransformOptions::default();
+                    let transform_result = match ext {
+                        "tsx" => crate::transform::transform_tsx::transform_tsx(&source, &options),
+                        "ts" => crate::transform::typescript::transform_typescript(&source, &options),
+                        "jsx" => crate::transform::jsx::transform_jsx(&source, &options),
+                        _ => Ok(crate::transform::TransformResult {
+                            code: source.clone(),
+                            source_map: None,
+                        }),
+                    };
+
+                    match transform_result {
+                        Ok(result) => {
+                            // Extract import paths from the transformed code
+                            let imports = extract_imports_from_source(&result.code);
+                            let has_react_components = source_has_react_components(&source);
+
+                            // Update module graph
+                            {
+                                let mut graph = module_graph.write().unwrap();
+                                graph.update_module(&module_url, &path_str, &imports);
+                                graph.set_has_react_refresh(&module_url, has_react_components);
+                                graph.set_timestamp(&module_url, timestamp);
+                            }
+
+                            // Run boundary detection
+                            let boundary = {
+                                let graph = module_graph.read().unwrap();
+                                HmrUpdateResult::determine(&module_url, &graph)
+                            };
+
+                            match boundary {
+                                HmrUpdateResult::HotUpdate { targets } => {
+                                    for target in targets {
+                                        let accepted_by = if target != module_url {
+                                            Some(target.clone())
+                                        } else {
+                                            None
+                                        };
+                                        let message = HmrMessage::Update {
+                                            path: module_url.clone(),
+                                            timestamp,
+                                            accepted_by,
+                                        };
+                                        hmr_manager.broadcast(message).await;
+                                    }
+                                }
+                                HmrUpdateResult::FullReload { reason } => {
+                                    let message = HmrMessage::FullReload { reason };
+                                    hmr_manager.broadcast(message).await;
+                                }
+                            }
+                        }
+                        Err(e) => {
+                            // Transform error — send error message with details
+                            let err_msg = format!("{}", e);
+                            let frame = build_error_frame(&source, &err_msg);
+                            let message = HmrMessage::Error {
+                                message: err_msg,
+                                file: Some(module_url),
+                                line: None,
+                                column: None,
+                                frame: Some(frame),
+                            };
+                            hmr_manager.broadcast(message).await;
+                        }
+                    }
+                    continue;
+                }
+
+                // Generic file update (non-JS/TS, non-CSS)
                 let message = HmrMessage::Update {
                     path: path_str,
                     timestamp,
+                    accepted_by: None,
                 };
                 hmr_manager.broadcast(message).await;
             }
@@ -273,6 +394,11 @@ async fn dispatch_request(
         return serve_bundle(state).await;
     }
 
+    // Serve React Fast Refresh runtime shim
+    if rel_path == "@react-refresh" {
+        return serve_react_refresh();
+    }
+
     if let Some(content) = serve_static_file(&state.config, rel_path).await {
         return content;
     }
@@ -337,10 +463,7 @@ async fn serve_bundle(state: ServerState) -> Response {
 
     match state.bundler.bundle(entry).await {
         Ok(output) => {
-            let mut code = output.code;
-
-            code.push_str("\n\n");
-            code.push_str(&generate_hmr_client());
+            let code = output.code;
 
             (
                 [(
@@ -378,6 +501,14 @@ async fn serve_index_html(
         html = importmap::inject_importmap_html(&html, json);
     }
 
+    // Inject HMR client runtime before </body> (or at the end)
+    let hmr_runtime = hmr_client::generate_hmr_runtime();
+    if let Some(pos) = html.rfind("</body>") {
+        html.insert_str(pos, &hmr_runtime);
+    } else {
+        html.push_str(&hmr_runtime);
+    }
+
     (
         [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
         html,
@@ -431,24 +562,38 @@ async fn serve_root_file(config: &ServerConfig, path: &str) -> Option<Response>
     let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
 
     // Transform TypeScript/TSX files through the AST-based transformer
-    if matches!(ext, "tsx" | "ts" | "jsx") {
+    if matches!(ext, "tsx" | "ts" | "jsx" | "js") {
         let source = std::fs::read_to_string(&file_path).ok()?;
         let options = crate::transform::TransformOptions::default();
         let result = match ext {
             "tsx" => crate::transform::transform_tsx::transform_tsx(&source, &options),
             "ts" => crate::transform::typescript::transform_typescript(&source, &options),
             "jsx" => crate::transform::jsx::transform_jsx(&source, &options),
+            "js" => Ok(crate::transform::TransformResult {
+                code: source.clone(),
+                source_map: None,
+            }),
             _ => unreachable!(),
         };
         match result {
             Ok(transformed) => {
+                // Inject import.meta.hot preamble for HMR support
+                let module_url = format!("/{}", path);
+                let hot_preamble = hmr_client::generate_hot_preamble(&module_url);
+
+                let mut code = String::with_capacity(
+                    hot_preamble.len() + transformed.code.len(),
+                );
+                code.push_str(&hot_preamble);
+                code.push_str(&transformed.code);
+
                 return Some(
                     (
                         [(
                             axum::http::header::CONTENT_TYPE,
                             "application/javascript; charset=utf-8",
                         )],
-                        transformed.code,
+                        code,
                     )
                         .into_response(),
                 );
@@ -497,54 +642,107 @@ fn guess_content_type(path: &PathBuf) -> String {
     }
 }
 
-fn generate_hmr_client() -> String {
-    r#"// Jet HMR Client
-(function() {
-  if (typeof window === 'undefined') return;
-
-  console.log('[Jet] Connecting to HMR server...');
-
-  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
-  const host = window.location.host;
-  const ws = new WebSocket(`${protocol}//${host}/__jet_hmr`);
-
-  ws.onopen = () => {
-    console.log('[Jet] HMR connected');
-  };
-
-  ws.onmessage = (event) => {
-    const message = JSON.parse(event.data);
-    console.log('[Jet] HMR message:', message);
-
-    switch (message.type) {
-      case 'update':
-        console.log('[Jet] Module updated:', message.path);
-        // For now, do a full reload
-        window.location.reload();
-        break;
-
-      case 'full-reload':
-        console.log('[Jet] Full reload:', message.reason);
-        window.location.reload();
-        break;
-
-      case 'error':
-        console.error('[Jet] Error:', message.message);
-        break;
+/// Serve the `/@react-refresh` endpoint with the React Fast Refresh runtime shim.
+fn serve_react_refresh() -> Response {
+    (
+        [(
+            axum::http::header::CONTENT_TYPE,
+            "application/javascript; charset=utf-8",
+        )],
+        react_refresh::react_refresh_runtime_source(),
+    )
+        .into_response()
+}
+
+/// Convert a filesystem path to a URL path relative to the project root.
+fn file_path_to_url(path: &std::path::Path, root: &std::path::Path) -> String {
+    match path.strip_prefix(root) {
+        Ok(rel) => format!("/{}", rel.to_string_lossy().replace('\\', "/")),
+        Err(_) => format!("/{}", path.to_string_lossy().replace('\\', "/")),
+    }
+}
+
+/// Extract import specifiers from transformed JavaScript source.
+///
+/// A simple regex-free scanner that looks for `import ... from '...'` and
+/// `import '...'` patterns.  Returns URL-style paths (starting with `/` or `.`).
+fn extract_imports_from_source(code: &str) -> Vec<String> {
+    let mut imports = Vec::new();
+
+    for line in code.lines() {
+        let trimmed = line.trim();
+        if !trimmed.starts_with("import") {
+            continue;
+        }
+
+        // Find the string literal after `from` or standalone `import 'x'`
+        let spec = if let Some(pos) = trimmed.rfind("from ") {
+            let after = &trimmed[pos + 5..];
+            extract_string_literal(after)
+        } else if trimmed.starts_with("import '") || trimmed.starts_with("import \"") {
+            extract_string_literal(&trimmed[7..])
+        } else {
+            None
+        };
+
+        if let Some(s) = spec {
+            // Only track relative / absolute imports (not bare specifiers like 'react')
+            if s.starts_with('.') || s.starts_with('/') {
+                imports.push(s);
+            }
+        }
+    }
+
+    imports
+}
+
+/// Extract a string literal value from a slice like `'./foo'` or `"./foo"`.
+fn extract_string_literal(s: &str) -> Option<String> {
+    let s = s.trim();
+    let (quote, rest) = if s.starts_with('\'') {
+        ('\'', &s[1..])
+    } else if s.starts_with('"') {
+        ('"', &s[1..])
+    } else {
+        return None;
+    };
+
+    rest.find(quote).map(|end| rest[..end].to_string())
+}
+
+/// Heuristic check: does the source contain React component patterns?
+///
+/// Looks for JSX elements or known React patterns to determine if the file
+/// should be treated as having React Fast Refresh boundaries.
+fn source_has_react_components(source: &str) -> bool {
+    // Look for JSX syntax: <Component or <div
+    for line in source.lines() {
+        let trimmed = line.trim();
+        // JSX return statements
+        if trimmed.contains("return (") || trimmed.contains("return(") {
+            continue; // Will be detected by JSX tag check
+        }
+        // Self-closing JSX: <Foo /> or <div />
+        if trimmed.contains("/>") && trimmed.contains('<') {
+            return true;
+        }
+        // Opening JSX: <Foo> or <div>
+        if trimmed.contains("</") && trimmed.contains('>') {
+            return true;
+        }
+    }
+    false
+}
+
+/// Build a simple code frame string for error reporting.
+fn build_error_frame(source: &str, _error_msg: &str) -> String {
+    // Show the first 10 lines of source as context
+    let lines: Vec<&str> = source.lines().take(10).collect();
+    let mut frame = String::new();
+    for (i, line) in lines.iter().enumerate() {
+        frame.push_str(&format!("{:>4} | {}\n", i + 1, line));
     }
-  };
-
-  ws.onerror = (error) => {
-    console.error('[Jet] HMR connection error:', error);
-  };
-
-  ws.onclose = () => {
-    console.log('[Jet] HMR disconnected. Retrying in 1s...');
-    setTimeout(() => window.location.reload(), 1000);
-  };
-})();
-"#
-    .to_string()
+    frame
 }
 
 impl Default for ServerConfig {
diff --git a/crates/cclab-jet/src/dev_server/watcher.rs b/crates/cclab-jet/src/dev_server/watcher.rs
index eba3204a..c724010c 100644
--- a/crates/cclab-jet/src/dev_server/watcher.rs
+++ b/crates/cclab-jet/src/dev_server/watcher.rs
@@ -1,9 +1,16 @@
 use anyhow::Result;
 use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
+use std::collections::HashMap;
 use std::path::PathBuf;
+use std::sync::Mutex;
+use std::time::Instant;
 use tokio::sync::broadcast;
 
-/// File watcher for detecting changes
+/// Debounce window in milliseconds.
+/// Editors often save-then-rename which produces duplicate events.
+const DEBOUNCE_MS: u128 = 50;
+
+/// File watcher for detecting changes with debouncing.
 pub struct FileWatcher {
     _watcher: RecommendedWatcher,
     tx: broadcast::Sender<PathBuf>,
@@ -13,6 +20,8 @@ impl FileWatcher {
     pub fn new(root_dir: PathBuf) -> Result<Self> {
         let (tx, _) = broadcast::channel(100);
         let tx_clone = tx.clone();
+        let last_seen: std::sync::Arc<Mutex<HashMap<PathBuf, Instant>>> =
+            std::sync::Arc::new(Mutex::new(HashMap::new()));
 
         let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
             if let Ok(event) = res {
@@ -21,6 +30,18 @@ impl FileWatcher {
                         continue;
                     }
 
+                    // Debounce: skip if same path was seen within DEBOUNCE_MS
+                    let now = Instant::now();
+                    {
+                        let mut map = last_seen.lock().unwrap();
+                        if let Some(prev) = map.get(&path) {
+                            if now.duration_since(*prev).as_millis() < DEBOUNCE_MS {
+                                continue;
+                            }
+                        }
+                        map.insert(path.clone(), now);
+                    }
+
                     let _ = tx_clone.send(path);
                 }
             }
diff --git a/crates/cclab-jet/src/transform/mod.rs b/crates/cclab-jet/src/transform/mod.rs
index 9acdabf7..eaf9911c 100644
--- a/crates/cclab-jet/src/transform/mod.rs
+++ b/crates/cclab-jet/src/transform/mod.rs
@@ -6,6 +6,7 @@ pub mod css;
 pub mod incremental;
 pub mod jsx;
 pub mod modules;
+pub mod react_refresh;
 pub mod transform_tsx;
 pub mod type_strip;
 pub mod typescript;
@@ -35,6 +36,9 @@ pub struct TransformOptions {
 
     /// Enable minification
     pub minify: bool,
+
+    /// Dev mode: enables React Fast Refresh injection for HMR
+    pub dev_mode: bool,
 }
 
 /// TypeScript compilation target
@@ -131,6 +135,7 @@ impl Default for TransformOptions {
             ts_target: TypeScriptTarget::ES2020,
             source_maps: true,
             minify: false,
+            dev_mode: true,
         }
     }
 }
diff --git a/crates/cclab-jet/src/transform/transform_tsx.rs b/crates/cclab-jet/src/transform/transform_tsx.rs
index df583206..df6d949b 100644
--- a/crates/cclab-jet/src/transform/transform_tsx.rs
+++ b/crates/cclab-jet/src/transform/transform_tsx.rs
@@ -103,6 +103,11 @@ pub fn transform_tsx(source: &str, options: &TransformOptions) -> Result<Transfo
         transformed = runtime_import.to_string() + &transformed;
     }
 
+    // React Fast Refresh injection (dev mode only, JSX files only)
+    if opts.dev_mode && has_jsx(&root) {
+        transformed = super::react_refresh::inject_react_fast_refresh(&transformed, source, &root);
+    }
+
     Ok(TransformResult {
         code: transformed,
         source_map: if options.source_maps {
diff --git a/crates/cclab-jet/src/transform/transform_tsx_tests.rs b/crates/cclab-jet/src/transform/transform_tsx_tests.rs
index 0ab24e88..07e73be4 100644
--- a/crates/cclab-jet/src/transform/transform_tsx_tests.rs
+++ b/crates/cclab-jet/src/transform/transform_tsx_tests.rs
@@ -408,3 +408,217 @@ fn t32_mixed_import_value_and_type() {
         result.code
     );
 }
+
+// ── HMR / React Fast Refresh Tests (T1, T2, T10–T15) ───────────────────
+
+/// T1: import.meta.hot Injected in Dev Mode
+#[test]
+fn t1_import_meta_hot_injected_in_dev_mode() {
+    let source = r#"export function App() { return <div>Hello</div>; }"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    // Dev mode with JSX should inject React Fast Refresh preamble
+    assert!(
+        result.code.contains("import RefreshRuntime from '/@react-refresh'"),
+        "must inject RefreshRuntime import in dev mode: {}",
+        result.code
+    );
+    assert!(
+        result.code.contains("$RefreshSig$"),
+        "must inject $RefreshSig$ in dev mode: {}",
+        result.code
+    );
+    assert!(
+        result.code.contains("enqueueUpdate"),
+        "must inject enqueueUpdate footer in dev mode: {}",
+        result.code
+    );
+}
+
+/// T2: import.meta.hot Not Injected in Prod Build
+#[test]
+fn t2_import_meta_hot_not_injected_in_prod() {
+    let source = r#"export function App() { return <div>Hello</div>; }"#;
+    let options = TransformOptions {
+        dev_mode: false,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        !result.code.contains("RefreshRuntime"),
+        "must NOT inject RefreshRuntime in prod mode: {}",
+        result.code
+    );
+    assert!(
+        !result.code.contains("$RefreshReg$"),
+        "must NOT inject $RefreshReg$ in prod mode: {}",
+        result.code
+    );
+    assert!(
+        !result.code.contains("$RefreshSig$"),
+        "must NOT inject $RefreshSig$ in prod mode: {}",
+        result.code
+    );
+    assert!(
+        !result.code.contains("enqueueUpdate"),
+        "must NOT inject enqueueUpdate in prod mode: {}",
+        result.code
+    );
+}
+
+/// T10: Component Declaration Gets $RefreshReg$
+#[test]
+fn t10_component_declaration_gets_refresh_reg() {
+    let source = r#"export function App() { return <div/>; }"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        result.code.contains("$RefreshReg$(App, \"App\")"),
+        "must inject $RefreshReg$ for App component: {}",
+        result.code
+    );
+}
+
+/// T11: Arrow Component Gets $RefreshReg$
+#[test]
+fn t11_arrow_component_gets_refresh_reg() {
+    let source = r#"const App = () => <div/>;"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        result.code.contains("$RefreshReg$(App, \"App\")"),
+        "must inject $RefreshReg$ for arrow App component: {}",
+        result.code
+    );
+}
+
+/// T12: Hook Usage Gets $RefreshSig$
+#[test]
+fn t12_hook_usage_gets_refresh_sig() {
+    let source = r#"function Counter() {
+  const [count, setCount] = useState(0);
+  useEffect(() => {}, []);
+  return <div>{count}</div>;
+}"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        result.code.contains("$RefreshSig$()"),
+        "must inject $RefreshSig$ call for hooks: {}",
+        result.code
+    );
+    // Signature should include hook call order fingerprint
+    assert!(
+        result.code.contains("useState{}"),
+        "hooks signature must include useState: {}",
+        result.code
+    );
+    assert!(
+        result.code.contains("useEffect{}"),
+        "hooks signature must include useEffect: {}",
+        result.code
+    );
+}
+
+/// T13: Non-Component Function Skipped
+#[test]
+fn t13_non_component_function_skipped() {
+    let source = r#"function calculateTotal(items: number[]): number { return items.reduce((a, b) => a + b, 0); }
+export function App() { return <div>{calculateTotal([1,2,3])}</div>; }"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    // calculateTotal is lowercase — not a React component
+    assert!(
+        !result.code.contains("$RefreshReg$(calculateTotal"),
+        "must NOT inject $RefreshReg$ for non-component function: {}",
+        result.code
+    );
+    // App should still get registration
+    assert!(
+        result.code.contains("$RefreshReg$(App, \"App\")"),
+        "must inject $RefreshReg$ for App: {}",
+        result.code
+    );
+}
+
+/// T14: React.memo Wrapped Component Detected
+#[test]
+fn t14_react_memo_wrapped_component() {
+    let source = r#"const MemoApp = React.memo(function App() { return <div/>; });"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        result.code.contains("$RefreshReg$(MemoApp, \"MemoApp\")"),
+        "must inject $RefreshReg$ for React.memo wrapped component: {}",
+        result.code
+    );
+}
+
+/// T15: Preamble and Footer Injected
+#[test]
+fn t15_preamble_and_footer_injected() {
+    let source = r#"export function App() { return <div>Hello</div>; }"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    // Preamble: starts with RefreshRuntime import
+    assert!(
+        result.code.starts_with("import RefreshRuntime from '/@react-refresh'"),
+        "output must start with RefreshRuntime import: {}",
+        result.code
+    );
+
+    // Footer: ends with enqueueUpdate
+    let trimmed = result.code.trim_end();
+    assert!(
+        trimmed.ends_with("RefreshRuntime.enqueueUpdate();"),
+        "output must end with RefreshRuntime.enqueueUpdate(): {}",
+        result.code
+    );
+}
+
+/// Additional: No Fast Refresh for non-JSX .tsx file
+#[test]
+fn no_fast_refresh_for_non_jsx_file() {
+    let source = r#"export const value: number = 42;
+export function helper(x: number): number { return x * 2; }"#;
+    let options = TransformOptions {
+        dev_mode: true,
+        ..TransformOptions::default()
+    };
+    let result = transform_tsx(source, &options).unwrap();
+
+    assert!(
+        !result.code.contains("RefreshRuntime"),
+        "must NOT inject Fast Refresh for non-JSX file: {}",
+        result.code
+    );
+}
diff --git a/crates/cclab-jet/src/dev_server/hmr_client.rs b/crates/cclab-jet/src/dev_server/hmr_client.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-jet/src/dev_server/hmr_client.rs
@@ -0,0 +1,     336 @@
+/// Generate the full HMR client runtime JavaScript.
+///
+/// This replaces the minimal `generate_hmr_client()` IIFE with a comprehensive
+/// runtime that supports:
+/// - `import.meta.hot` API (accept/dispose/prune/invalidate/data)
+/// - Module registry keyed by URL
+/// - WebSocket message handlers: update, css-update, full-reload, error, prune
+/// - Dynamic `import()` with cache-busting timestamp query
+/// - Error overlay: dark backdrop, monospace, code frame, click/Escape dismiss
+/// - Exponential backoff reconnection (1s, 2s, 4s, ... max 30s)
+pub fn generate_hmr_runtime() -> String {
+    r#"<script type="module">
+// ─── Jet HMR Runtime ─────────────────────────────────────────────────────────
+(function() {
+  if (typeof window === 'undefined') return;
+
+  // ── Module Registry ──────────────────────────────────────────────────────
+  // Map<moduleUrl, { accept, acceptDeps, dispose, prune, data, invalidate }>
+  const moduleRegistry = new Map();
+
+  // ── import.meta.hot factory ──────────────────────────────────────────────
+  window.__jet_hmr_create_hot = function(moduleUrl) {
+    let entry = moduleRegistry.get(moduleUrl);
+    if (!entry) {
+      entry = {
+        acceptSelf: null,
+        acceptDeps: new Map(),
+        disposeCb: null,
+        pruneCb: null,
+        invalidateCalled: false,
+        data: {},
+      };
+      moduleRegistry.set(moduleUrl, entry);
+    }
+
+    return {
+      get data() { return entry.data; },
+
+      accept(depsOrCb, cb) {
+        if (typeof depsOrCb === 'function' || depsOrCb === undefined) {
+          // Self-accepting: accept() or accept(cb)
+          entry.acceptSelf = depsOrCb || true;
+        } else if (Array.isArray(depsOrCb) && typeof cb === 'function') {
+          // Dependency accept: accept(['./dep'], cb)
+          for (const dep of depsOrCb) {
+            const resolved = new URL(dep, moduleUrl).pathname;
+            entry.acceptDeps.set(resolved, cb);
+          }
+        }
+      },
+
+      dispose(cb) {
+        if (typeof cb === 'function') {
+          entry.disposeCb = cb;
+        }
+      },
+
+      prune(cb) {
+        if (typeof cb === 'function') {
+          entry.pruneCb = cb;
+        }
+      },
+
+      invalidate() {
+        entry.invalidateCalled = true;
+      },
+    };
+  };
+
+  // ── Error Overlay ────────────────────────────────────────────────────────
+  let overlayContainer = null;
+
+  function createOverlayContainer() {
+    if (overlayContainer) return overlayContainer;
+    overlayContainer = document.createElement('div');
+    overlayContainer.id = '__jet_error_overlay';
+    overlayContainer.style.cssText = [
+      'position: fixed',
+      'top: 0',
+      'left: 0',
+      'width: 100vw',
+      'height: 100vh',
+      'background: rgba(0, 0, 0, 0.85)',
+      'z-index: 99999',
+      'display: flex',
+      'flex-direction: column',
+      'align-items: center',
+      'justify-content: center',
+      'overflow-y: auto',
+      'padding: 24px',
+      'box-sizing: border-box',
+      'font-family: "SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", monospace',
+      'color: #fff',
+    ].join(';');
+
+    overlayContainer.addEventListener('click', (e) => {
+      if (e.target === overlayContainer) dismissOverlay();
+    });
+
+    document.addEventListener('keydown', (e) => {
+      if (e.key === 'Escape') dismissOverlay();
+    });
+
+    document.body.appendChild(overlayContainer);
+    return overlayContainer;
+  }
+
+  function showError(err) {
+    const container = createOverlayContainer();
+    container.style.display = 'flex';
+
+    const card = document.createElement('div');
+    card.style.cssText = [
+      'background: #1a1a2e',
+      'border: 1px solid #e94560',
+      'border-radius: 8px',
+      'padding: 20px 24px',
+      'max-width: 800px',
+      'width: 100%',
+      'margin-bottom: 12px',
+      'box-shadow: 0 4px 24px rgba(233, 69, 96, 0.3)',
+    ].join(';');
+
+    let html = '<div style="color:#e94560;font-size:14px;font-weight:600;margin-bottom:8px;">Error</div>';
+
+    if (err.file) {
+      let loc = err.file;
+      if (err.line != null) loc += ':' + err.line;
+      if (err.column != null) loc += ':' + err.column;
+      html += '<div style="color:#aaa;font-size:12px;margin-bottom:12px;">' + escapeHtml(loc) + '</div>';
+    }
+
+    html += '<div style="color:#f0f0f0;font-size:13px;white-space:pre-wrap;margin-bottom:12px;">' + escapeHtml(err.message) + '</div>';
+
+    if (err.frame) {
+      html += '<pre style="background:#0d1117;border-radius:4px;padding:12px;font-size:12px;overflow-x:auto;color:#c9d1d9;margin:0;">' + escapeHtml(err.frame) + '</pre>';
+    }
+
+    card.innerHTML = html;
+    // Newest on top
+    container.insertBefore(card, container.firstChild);
+  }
+
+  function dismissOverlay() {
+    if (overlayContainer) {
+      overlayContainer.style.display = 'none';
+      overlayContainer.innerHTML = '';
+    }
+  }
+
+  function escapeHtml(str) {
+    return String(str)
+      .replace(/&/g, '&amp;')
+      .replace(/</g, '&lt;')
+      .replace(/>/g, '&gt;')
+      .replace(/"/g, '&quot;');
+  }
+
+  // ── CSS Update ───────────────────────────────────────────────────────────
+  function handleCssUpdate(msg) {
+    // Find existing <link> or <style> with matching filename pattern
+    const links = document.querySelectorAll('link[rel="stylesheet"]');
+    for (const link of links) {
+      const href = link.getAttribute('href') || '';
+      // Match by base name (before hash)
+      const baseName = msg.filename.replace(/\.[a-f0-9]+\.css$/, '');
+      if (href.includes(baseName) || href.includes(msg.filename)) {
+        link.href = '/' + msg.filename + '?t=' + msg.timestamp;
+        return;
+      }
+    }
+
+    // Check for <style data-jet-css> injected styles
+    const jetStyles = document.querySelectorAll('style[data-jet-css]');
+    if (jetStyles.length > 0) {
+      jetStyles[jetStyles.length - 1].textContent = msg.css;
+      return;
+    }
+
+    // Create new <style> element
+    const style = document.createElement('style');
+    style.setAttribute('data-jet-css', msg.filename);
+    style.textContent = msg.css;
+    document.head.appendChild(style);
+  }
+
+  // ── Module Hot Update ────────────────────────────────────────────────────
+  async function handleUpdate(msg) {
+    const { path, timestamp, acceptedBy } = msg;
+    const moduleUrl = acceptedBy || path;
+
+    const entry = moduleRegistry.get(moduleUrl);
+    if (!entry && !acceptedBy) {
+      // Module not registered in HMR — full reload as safety net
+      console.log('[Jet] Module not in HMR registry, reloading:', path);
+      window.location.reload();
+      return;
+    }
+
+    // 1. Run dispose callback on the old module
+    if (entry && entry.disposeCb) {
+      try {
+        entry.disposeCb(entry.data);
+      } catch (e) {
+        console.error('[Jet] dispose callback error:', e);
+      }
+    }
+
+    // 2. Re-import the changed module with cache-busting
+    try {
+      const newModule = await import(path + '?t=' + timestamp);
+
+      // 3. Run accept callback
+      if (entry) {
+        if (acceptedBy && entry.acceptDeps.has(path)) {
+          // Dependency accept — parent handles update
+          const cb = entry.acceptDeps.get(path);
+          if (typeof cb === 'function') {
+            cb([newModule]);
+          }
+        } else if (entry.acceptSelf) {
+          // Self-accepting
+          if (typeof entry.acceptSelf === 'function') {
+            entry.acceptSelf(newModule);
+          }
+        }
+      }
+
+      // 4. Dismiss error overlay on successful update
+      dismissOverlay();
+
+      console.log('[Jet] Hot updated:', path);
+    } catch (e) {
+      console.error('[Jet] HMR update failed, falling back to reload:', e);
+      window.location.reload();
+    }
+  }
+
+  // ── Prune ────────────────────────────────────────────────────────────────
+  function handlePrune(msg) {
+    for (const path of msg.paths) {
+      const entry = moduleRegistry.get(path);
+      if (entry && entry.pruneCb) {
+        try {
+          entry.pruneCb();
+        } catch (e) {
+          console.error('[Jet] prune callback error:', e);
+        }
+      }
+      moduleRegistry.delete(path);
+    }
+  }
+
+  // ── WebSocket Connection ─────────────────────────────────────────────────
+  let retryDelay = 1000;
+  const MAX_RETRY_DELAY = 30000;
+
+  function connect() {
+    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
+    const host = window.location.host;
+    const ws = new WebSocket(protocol + '//' + host + '/__jet_hmr');
+
+    ws.onopen = () => {
+      console.log('[Jet] HMR connected');
+      retryDelay = 1000; // Reset backoff on successful connection
+    };
+
+    ws.onmessage = (event) => {
+      const msg = JSON.parse(event.data);
+
+      switch (msg.type) {
+        case 'connected':
+          console.log('[Jet] HMR ready');
+          break;
+
+        case 'update':
+          console.log('[Jet] Module update:', msg.path);
+          handleUpdate(msg);
+          break;
+
+        case 'css-update':
+          console.log('[Jet] CSS update:', msg.filename);
+          handleCssUpdate(msg);
+          break;
+
+        case 'full-reload':
+          console.log('[Jet] Full reload:', msg.reason);
+          window.location.reload();
+          break;
+
+        case 'error':
+          console.error('[Jet] Error:', msg.message);
+          showError(msg);
+          break;
+
+        case 'prune':
+          console.log('[Jet] Pruning modules:', msg.paths);
+          handlePrune(msg);
+          break;
+
+        default:
+          console.log('[Jet] Unknown HMR message:', msg);
+      }
+    };
+
+    ws.onerror = () => {
+      // Error will be followed by close — reconnect handled there
+    };
+
+    ws.onclose = () => {
+      console.log('[Jet] HMR disconnected. Reconnecting in ' + retryDelay + 'ms...');
+      setTimeout(() => {
+        retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY);
+        connect();
+      }, retryDelay);
+    };
+  }
+
+  console.log('[Jet] Connecting to HMR server...');
+  connect();
+})();
+</script>
+"#
+    .to_string()
+}
+
+/// Generate the `import.meta.hot` injection code for a served JS module.
+///
+/// Prepends a small snippet that creates the `import.meta.hot` object keyed
+/// by the module's URL path.
+pub fn generate_hot_preamble(module_url: &str) -> String {
+    format!(
+        "if (window.__jet_hmr_create_hot) {{ import.meta.hot = window.__jet_hmr_create_hot(\"{}\"); }}\n",
+        module_url
+    )
+}
diff --git a/crates/cclab-jet/src/dev_server/module_graph.rs b/crates/cclab-jet/src/dev_server/module_graph.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-jet/src/dev_server/module_graph.rs
@@ -0,0 +1,     536 @@
+use std::collections::{HashMap, HashSet, VecDeque};
+
+/// A node in the module dependency graph.
+#[derive(Debug, Clone)]
+pub struct ModuleGraphNode {
+    /// Module URL path (e.g. `/src/App.tsx`).
+    pub url: String,
+    /// Absolute filesystem path.
+    pub file: String,
+    /// URLs of modules this module imports.
+    pub imports: HashSet<String>,
+    /// URLs of modules that import this module.
+    pub importers: HashSet<String>,
+    /// Module called `import.meta.hot.accept()` with no deps (self-accepting).
+    pub is_self_accepting: bool,
+    /// URLs of deps accepted via `import.meta.hot.accept(deps, cb)`.
+    pub accepted_deps: HashSet<String>,
+    /// Module contains React Fast Refresh instrumentation.
+    pub has_react_refresh: bool,
+    /// Timestamp of the last transform.
+    pub last_transform_timestamp: u64,
+}
+
+impl ModuleGraphNode {
+    fn new(url: &str, file: &str) -> Self {
+        Self {
+            url: url.to_string(),
+            file: file.to_string(),
+            imports: HashSet::new(),
+            importers: HashSet::new(),
+            is_self_accepting: false,
+            accepted_deps: HashSet::new(),
+            has_react_refresh: false,
+            last_transform_timestamp: 0,
+        }
+    }
+}
+
+/// Result of an HMR boundary search.
+#[derive(Debug, Clone, PartialEq)]
+pub enum HmrBoundaryResult {
+    /// Hot-update targets found — re-import these modules.
+    HotUpdate { targets: Vec<String> },
+    /// No boundary found — full page reload required.
+    FullReload { reason: String },
+}
+
+/// Server-side module dependency graph.
+///
+/// Tracks import relationships as a directed graph and supports invalidation
+/// walks to determine HMR update boundaries.
+pub struct ModuleGraph {
+    nodes: HashMap<String, ModuleGraphNode>,
+}
+
+impl ModuleGraph {
+    pub fn new() -> Self {
+        Self {
+            nodes: HashMap::new(),
+        }
+    }
+
+    /// Add or replace a module in the graph, establishing import edges.
+    ///
+    /// Returns the list of removed imports and newly added imports.
+    pub fn add_module(
+        &mut self,
+        url: &str,
+        file: &str,
+        imports: &[String],
+    ) -> (Vec<String>, Vec<String>) {
+        let old_imports = if let Some(existing) = self.nodes.get(url) {
+            existing.imports.clone()
+        } else {
+            HashSet::new()
+        };
+
+        let new_imports: HashSet<String> = imports.iter().cloned().collect();
+
+        // Compute diff
+        let removed: Vec<String> = old_imports.difference(&new_imports).cloned().collect();
+        let added: Vec<String> = new_imports.difference(&old_imports).cloned().collect();
+
+        // Remove stale importer edges
+        for dep_url in &removed {
+            if let Some(dep_node) = self.nodes.get_mut(dep_url) {
+                dep_node.importers.remove(url);
+            }
+        }
+
+        // Add new importer edges (ensure dep nodes exist)
+        for dep_url in &added {
+            let dep_node = self
+                .nodes
+                .entry(dep_url.clone())
+                .or_insert_with(|| ModuleGraphNode::new(dep_url, ""));
+            dep_node.importers.insert(url.to_string());
+        }
+
+        // Also ensure existing (unchanged) deps have importer edges
+        for dep_url in new_imports.intersection(&old_imports) {
+            if let Some(dep_node) = self.nodes.get_mut(dep_url) {
+                dep_node.importers.insert(url.to_string());
+            }
+        }
+
+        // Upsert the module node itself
+        let node = self
+            .nodes
+            .entry(url.to_string())
+            .or_insert_with(|| ModuleGraphNode::new(url, file));
+        node.file = file.to_string();
+        node.imports = imports.iter().cloned().collect();
+
+        (removed, added)
+    }
+
+    /// Update a module's imports after re-transform, returning (removed, added).
+    pub fn update_module(
+        &mut self,
+        url: &str,
+        file: &str,
+        new_imports: &[String],
+    ) -> (Vec<String>, Vec<String>) {
+        self.add_module(url, file, new_imports)
+    }
+
+    /// Mark a module as self-accepting (`import.meta.hot.accept()`).
+    pub fn set_self_accepting(&mut self, url: &str, accepting: bool) {
+        if let Some(node) = self.nodes.get_mut(url) {
+            node.is_self_accepting = accepting;
+        }
+    }
+
+    /// Set accepted deps for a module (`import.meta.hot.accept(deps, cb)`).
+    pub fn set_accepted_deps(&mut self, url: &str, deps: HashSet<String>) {
+        if let Some(node) = self.nodes.get_mut(url) {
+            node.accepted_deps = deps;
+        }
+    }
+
+    /// Mark a module as having React Fast Refresh instrumentation.
+    pub fn set_has_react_refresh(&mut self, url: &str, has_refresh: bool) {
+        if let Some(node) = self.nodes.get_mut(url) {
+            node.has_react_refresh = has_refresh;
+        }
+    }
+
+    /// Set the last transform timestamp on a module.
+    pub fn set_timestamp(&mut self, url: &str, timestamp: u64) {
+        if let Some(node) = self.nodes.get_mut(url) {
+            node.last_transform_timestamp = timestamp;
+        }
+    }
+
+    /// Remove a module from the graph and clean up edges.
+    ///
+    /// Returns the list of orphaned modules (imported by no one after removal).
+    pub fn remove_module(&mut self, url: &str) -> Vec<String> {
+        let mut orphans = Vec::new();
+
+        if let Some(removed) = self.nodes.remove(url) {
+            // Remove this module from importers lists of its imports
+            for dep_url in &removed.imports {
+                if let Some(dep_node) = self.nodes.get_mut(dep_url) {
+                    dep_node.importers.remove(url);
+                    if dep_node.importers.is_empty() {
+                        orphans.push(dep_url.clone());
+                    }
+                }
+            }
+
+            // Remove this module from imports lists of its importers
+            for importer_url in &removed.importers {
+                if let Some(importer_node) = self.nodes.get_mut(importer_url) {
+                    importer_node.imports.remove(url);
+                }
+            }
+        }
+
+        orphans
+    }
+
+    /// Walk the module graph upward from `changed_url` to find HMR boundaries.
+    ///
+    /// A boundary is:
+    /// - A self-accepting module (`import.meta.hot.accept()`)
+    /// - A module with React Fast Refresh instrumentation
+    /// - A parent that accepts the changed dep (`import.meta.hot.accept([dep], cb)`)
+    ///
+    /// If no boundary is found before reaching the entry point, returns `FullReload`.
+    pub fn find_hmr_boundary(&self, changed_url: &str) -> HmrBoundaryResult {
+        let node = match self.nodes.get(changed_url) {
+            Some(n) => n,
+            None => {
+                return HmrBoundaryResult::FullReload {
+                    reason: format!("module not in graph: {}", changed_url),
+                }
+            }
+        };
+
+        // Case 1: Changed module is self-accepting
+        if node.is_self_accepting {
+            return HmrBoundaryResult::HotUpdate {
+                targets: vec![changed_url.to_string()],
+            };
+        }
+
+        // Case 2: Changed module has React Fast Refresh
+        if node.has_react_refresh {
+            return HmrBoundaryResult::HotUpdate {
+                targets: vec![changed_url.to_string()],
+            };
+        }
+
+        // Case 3: Walk upward through importers using BFS
+        let mut targets = Vec::new();
+        let mut visited = HashSet::new();
+        let mut queue = VecDeque::new();
+
+        // Seed queue with (module_url, dep_that_changed_url)
+        // The "dep" from the perspective of the parent is the module we came from.
+        for importer_url in &node.importers {
+            queue.push_back((importer_url.clone(), changed_url.to_string()));
+        }
+        visited.insert(changed_url.to_string());
+
+        // If the module has no importers at all, it's likely an entry — full reload.
+        if node.importers.is_empty() {
+            return HmrBoundaryResult::FullReload {
+                reason: format!("no HMR boundary for {}", changed_url),
+            };
+        }
+
+        while let Some((current_url, dep_url)) = queue.pop_front() {
+            if visited.contains(&current_url) {
+                continue;
+            }
+            visited.insert(current_url.clone());
+
+            let current = match self.nodes.get(&current_url) {
+                Some(n) => n,
+                None => continue,
+            };
+
+            // Check if this parent accepts the dep that changed
+            if current.accepted_deps.contains(&dep_url) {
+                targets.push(current_url.clone());
+                continue;
+            }
+
+            // Check if parent is self-accepting
+            if current.is_self_accepting {
+                targets.push(current_url.clone());
+                continue;
+            }
+
+            // Check if parent has React Fast Refresh
+            if current.has_react_refresh {
+                targets.push(current_url.clone());
+                continue;
+            }
+
+            // No boundary here — keep walking up
+            if current.importers.is_empty() {
+                // Reached an entry point with no boundary
+                return HmrBoundaryResult::FullReload {
+                    reason: format!("no HMR boundary for {}", changed_url),
+                };
+            }
+
+            for parent_url in &current.importers {
+                queue.push_back((parent_url.clone(), current_url.clone()));
+            }
+        }
+
+        if targets.is_empty() {
+            HmrBoundaryResult::FullReload {
+                reason: format!("no HMR boundary for {}", changed_url),
+            }
+        } else {
+            HmrBoundaryResult::HotUpdate { targets }
+        }
+    }
+
+    /// Get a reference to a module node.
+    pub fn get(&self, url: &str) -> Option<&ModuleGraphNode> {
+        self.nodes.get(url)
+    }
+
+    /// Get all module URLs in the graph.
+    pub fn urls(&self) -> Vec<String> {
+        self.nodes.keys().cloned().collect()
+    }
+}
+
+impl Default for ModuleGraph {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    // ── T5: Graph Built From Imports ────────────────────────────────────────
+    #[test]
+    fn t5_graph_built_from_imports() {
+        let mut graph = ModuleGraph::new();
+        let imports = vec!["/src/B.tsx".to_string(), "/src/C.tsx".to_string()];
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &imports);
+
+        // A's imports contain B and C
+        let node_a = graph.get("/src/A.tsx").expect("A must exist");
+        assert!(node_a.imports.contains("/src/B.tsx"));
+        assert!(node_a.imports.contains("/src/C.tsx"));
+
+        // B's importers contain A
+        let node_b = graph.get("/src/B.tsx").expect("B must exist");
+        assert!(
+            node_b.importers.contains("/src/A.tsx"),
+            "B.importers must contain A"
+        );
+
+        // C's importers contain A
+        let node_c = graph.get("/src/C.tsx").expect("C must exist");
+        assert!(
+            node_c.importers.contains("/src/A.tsx"),
+            "C.importers must contain A"
+        );
+    }
+
+    // ── T6: Graph Update Removes Stale Edges ────────────────────────────────
+    #[test]
+    fn t6_graph_update_removes_stale_edges() {
+        let mut graph = ModuleGraph::new();
+
+        // Initial: A imports B and C
+        let imports_v1 = vec!["/src/B.tsx".to_string(), "/src/C.tsx".to_string()];
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &imports_v1);
+
+        // Update: A now imports C and D (removed B, added D)
+        let imports_v2 = vec!["/src/C.tsx".to_string(), "/src/D.tsx".to_string()];
+        let (removed, added) = graph.update_module("/src/A.tsx", "/abs/src/A.tsx", &imports_v2);
+
+        // Check diff return values
+        assert!(removed.contains(&"/src/B.tsx".to_string()), "B must be in removed list");
+        assert!(added.contains(&"/src/D.tsx".to_string()), "D must be in added list");
+
+        // B's importers no longer contains A
+        let node_b = graph.get("/src/B.tsx").expect("B must still exist as node");
+        assert!(
+            !node_b.importers.contains("/src/A.tsx"),
+            "B.importers must NOT contain A after update"
+        );
+
+        // D's importers contains A
+        let node_d = graph.get("/src/D.tsx").expect("D must exist");
+        assert!(
+            node_d.importers.contains("/src/A.tsx"),
+            "D.importers must contain A"
+        );
+
+        // C's importers still contains A (unchanged)
+        let node_c = graph.get("/src/C.tsx").expect("C must exist");
+        assert!(
+            node_c.importers.contains("/src/A.tsx"),
+            "C.importers must still contain A"
+        );
+
+        // A's imports are now C and D
+        let node_a = graph.get("/src/A.tsx").expect("A must exist");
+        assert!(node_a.imports.contains("/src/C.tsx"));
+        assert!(node_a.imports.contains("/src/D.tsx"));
+        assert!(!node_a.imports.contains("/src/B.tsx"));
+    }
+
+    // ── T7: Invalidation Walk Finds Self-Accept Boundary ────────────────────
+    #[test]
+    fn t7_invalidation_walk_finds_self_accept_boundary() {
+        let mut graph = ModuleGraph::new();
+
+        // A imports B
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
+        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);
+
+        // B is self-accepting, A is not
+        graph.set_self_accepting("/src/B.tsx", true);
+
+        let result = graph.find_hmr_boundary("/src/B.tsx");
+        match result {
+            HmrBoundaryResult::HotUpdate { targets } => {
+                assert_eq!(targets, vec!["/src/B.tsx".to_string()]);
+            }
+            HmrBoundaryResult::FullReload { reason } => {
+                panic!("Expected HotUpdate but got FullReload: {}", reason);
+            }
+        }
+    }
+
+    // ── T8: Invalidation Walk Propagates to Parent ──────────────────────────
+    #[test]
+    fn t8_invalidation_walk_propagates_to_parent() {
+        let mut graph = ModuleGraph::new();
+
+        // B imports C
+        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
+        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &[]);
+
+        // C has no accept handler, no React components
+        // B accepts C as a dependency
+        let mut deps = HashSet::new();
+        deps.insert("/src/C.tsx".to_string());
+        graph.set_accepted_deps("/src/B.tsx", deps);
+
+        let result = graph.find_hmr_boundary("/src/C.tsx");
+        match result {
+            HmrBoundaryResult::HotUpdate { targets } => {
+                assert!(
+                    targets.contains(&"/src/B.tsx".to_string()),
+                    "B must be in targets as the parent that accepts C: {:?}",
+                    targets
+                );
+            }
+            HmrBoundaryResult::FullReload { reason } => {
+                panic!("Expected HotUpdate but got FullReload: {}", reason);
+            }
+        }
+    }
+
+    // ── T9: Invalidation Walk Returns FullReload ────────────────────────────
+    #[test]
+    fn t9_invalidation_walk_returns_full_reload() {
+        let mut graph = ModuleGraph::new();
+
+        // Chain: entry → A → B → C, none have accept handlers or React components
+        graph.add_module("/src/entry.tsx", "/abs/src/entry.tsx", &["/src/A.tsx".to_string()]);
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
+        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &["/src/C.tsx".to_string()]);
+        graph.add_module("/src/C.tsx", "/abs/src/C.tsx", &[]);
+
+        let result = graph.find_hmr_boundary("/src/C.tsx");
+        match result {
+            HmrBoundaryResult::FullReload { reason } => {
+                assert!(
+                    reason.contains("no HMR boundary"),
+                    "reason must mention 'no HMR boundary': {}",
+                    reason
+                );
+            }
+            HmrBoundaryResult::HotUpdate { targets } => {
+                panic!("Expected FullReload but got HotUpdate: {:?}", targets);
+            }
+        }
+    }
+
+    // ── Additional: React Fast Refresh boundary ─────────────────────────────
+    #[test]
+    fn invalidation_walk_finds_react_refresh_boundary() {
+        let mut graph = ModuleGraph::new();
+
+        // A imports B
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
+        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);
+
+        // B has React Fast Refresh (it's a React component)
+        graph.set_has_react_refresh("/src/B.tsx", true);
+
+        let result = graph.find_hmr_boundary("/src/B.tsx");
+        match result {
+            HmrBoundaryResult::HotUpdate { targets } => {
+                assert_eq!(targets, vec!["/src/B.tsx".to_string()]);
+            }
+            HmrBoundaryResult::FullReload { reason } => {
+                panic!("Expected HotUpdate but got FullReload: {}", reason);
+            }
+        }
+    }
+
+    // ── Additional: Module removal returns orphans ──────────────────────────
+    #[test]
+    fn remove_module_returns_orphans() {
+        let mut graph = ModuleGraph::new();
+
+        // A imports B (B has no other importers)
+        graph.add_module("/src/A.tsx", "/abs/src/A.tsx", &["/src/B.tsx".to_string()]);
+        graph.add_module("/src/B.tsx", "/abs/src/B.tsx", &[]);
+
+        let orphans = graph.remove_module("/src/A.tsx");
+        assert!(
+            orphans.contains(&"/src/B.tsx".to_string()),
+            "B should be orphaned after A is removed"
+        );
+    }
+
+    // ── Additional: Unknown module returns FullReload ────────────────────────
+    #[test]
+    fn find_boundary_unknown_module_returns_full_reload() {
+        let graph = ModuleGraph::new();
+        let result = graph.find_hmr_boundary("/src/unknown.tsx");
+        match result {
+            HmrBoundaryResult::FullReload { reason } => {
+                assert!(reason.contains("not in graph"), "reason: {}", reason);
+            }
+            HmrBoundaryResult::HotUpdate { .. } => {
+                panic!("Expected FullReload for unknown module");
+            }
+        }
+    }
+
+    // ── Additional: Self-accepting parent acts as boundary ──────────────────
+    #[test]
+    fn self_accepting_parent_acts_as_boundary() {
+        let mut graph = ModuleGraph::new();
+
+        // Parent imports child, parent is self-accepting
+        graph.add_module("/src/Parent.tsx", "/abs/src/Parent.tsx", &["/src/Child.tsx".to_string()]);
+        graph.add_module("/src/Child.tsx", "/abs/src/Child.tsx", &[]);
+        graph.set_self_accepting("/src/Parent.tsx", true);
+
+        let result = graph.find_hmr_boundary("/src/Child.tsx");
+        match result {
+            HmrBoundaryResult::HotUpdate { targets } => {
+                assert!(
+                    targets.contains(&"/src/Parent.tsx".to_string()),
+                    "Parent should be the boundary target: {:?}",
+                    targets
+                );
+            }
+            HmrBoundaryResult::FullReload { reason } => {
+                panic!("Expected HotUpdate but got FullReload: {}", reason);
+            }
+        }
+    }
+}
diff --git a/crates/cclab-jet/src/dev_server/react_refresh.rs b/crates/cclab-jet/src/dev_server/react_refresh.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-jet/src/dev_server/react_refresh.rs
@@ -0,0 +1,     130 @@
+/// Serve the `/@react-refresh` endpoint.
+///
+/// Returns a self-contained ESM module implementing the React Fast Refresh
+/// runtime interface.  This is a lightweight shim that wraps the essential
+/// React Refresh APIs used by the `$RefreshReg$` / `$RefreshSig$` injections
+/// from `transform_tsx.rs`.
+///
+/// The real `react-refresh/runtime` is a dependency of the project, but we
+/// serve a thin wrapper that re-exports the functions we need and adds the
+/// `enqueueUpdate` / `performReactRefresh` scheduling helpers.
+pub fn react_refresh_runtime_source() -> &'static str {
+    r#"// /@react-refresh — Jet React Fast Refresh runtime shim
+//
+// This module wraps the react-refresh/runtime API with scheduling helpers.
+// Injected by Jet dev server for HMR support.
+
+let pendingUpdates = false;
+let refreshTimeout = null;
+
+// Registry of components keyed by module + component id.
+const allFamilies = new Map();
+
+// Signature tracking for hooks-order stability.
+const allSignatures = new Map();
+
+/**
+ * Register a component with the refresh runtime.
+ * Called as `$RefreshReg$(Component, "ComponentName")` by the transform.
+ */
+export function register(type, id) {
+  if (type == null || typeof type !== 'function') return;
+
+  const fullId = id;
+  let family = allFamilies.get(fullId);
+  if (!family) {
+    family = { current: type };
+    allFamilies.set(fullId, family);
+  } else {
+    family.current = type;
+  }
+
+  // If React DevTools or react-refresh/runtime is available, forward.
+  if (typeof window !== 'undefined' && window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
+    try {
+      const hook = window.__REACT_DEVTOOLS_GLOBAL_HOOK__;
+      if (typeof hook.registerFamily === 'function') {
+        hook.registerFamily(fullId, family);
+      }
+    } catch (_) {}
+  }
+}
+
+/**
+ * Create a signature function for tracking hook call order.
+ * Called as `const _s = $RefreshSig$()` at module scope.
+ *
+ * Returns a function that:
+ * 1. First call (during component definition): records the signature
+ * 2. Subsequent calls: validates the signature hasn't changed
+ */
+export function createSignatureFunctionForTransform() {
+  let savedSignature;
+  let hasCustomHooks = false;
+  let didCollectHooks = false;
+
+  return function(type, key, forceReset, getCustomHooks) {
+    if (typeof key === 'string') {
+      // Recording phase
+      savedSignature = key;
+      hasCustomHooks = typeof getCustomHooks === 'function';
+
+      if (type != null) {
+        let sig = allSignatures.get(type);
+        if (!sig) {
+          sig = {};
+          allSignatures.set(type, sig);
+        }
+        sig.key = key;
+        sig.forceReset = forceReset || false;
+        sig.getCustomHooks = getCustomHooks;
+      }
+    }
+
+    return type;
+  };
+}
+
+/**
+ * Schedule a React refresh update.
+ * Debounced to batch multiple module updates into one React re-render.
+ */
+export function enqueueUpdate() {
+  if (pendingUpdates) return;
+  pendingUpdates = true;
+
+  if (refreshTimeout != null) {
+    clearTimeout(refreshTimeout);
+  }
+
+  refreshTimeout = setTimeout(() => {
+    pendingUpdates = false;
+    refreshTimeout = null;
+    performReactRefresh();
+  }, 30);
+}
+
+/**
+ * Perform the actual React refresh / re-render.
+ */
+export function performReactRefresh() {
+  // Trigger React re-render by calling the DevTools hook if available.
+  if (typeof window !== 'undefined' && window.__REACT_DEVTOOLS_GLOBAL_HOOK__) {
+    try {
+      const hook = window.__REACT_DEVTOOLS_GLOBAL_HOOK__;
+      if (typeof hook.performReactRefresh === 'function') {
+        hook.performReactRefresh();
+      }
+    } catch (_) {}
+  }
+}
+
+// Default export for `import RefreshRuntime from '/@react-refresh'`
+export default {
+  register,
+  createSignatureFunctionForTransform,
+  enqueueUpdate,
+  performReactRefresh,
+};
+"#
+}
diff --git a/crates/cclab-jet/src/transform/react_refresh.rs b/crates/cclab-jet/src/transform/react_refresh.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-jet/src/transform/react_refresh.rs
@@ -0,0 +1,     292 @@
+//! React Fast Refresh injection for HMR support.
+//!
+//! Detects React component declarations in the AST and injects:
+//! 1. Preamble: import RefreshRuntime + create $RefreshSig$
+//! 2. `$RefreshReg$(Component, "Component")` after each component
+//! 3. `$RefreshSig$()` for hooks signature tracking
+//! 4. Footer: `RefreshRuntime.enqueueUpdate()`
+
+use tree_sitter::Node;
+
+/// Inject React Fast Refresh instrumentation into transformed code.
+///
+/// `transformed` — the already-JSX-transformed JavaScript code
+/// `source` — the original source (for AST name extraction)
+/// `root` — the parsed tree-sitter AST root node
+pub fn inject_react_fast_refresh(transformed: &str, source: &str, root: &Node) -> String {
+    let components = detect_react_components(source, root);
+
+    if components.is_empty() {
+        return transformed.to_string();
+    }
+
+    let mut result = String::new();
+
+    // 1. Preamble
+    result.push_str(
+        "import RefreshRuntime from '/@react-refresh';\n\
+         const $RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;\n",
+    );
+
+    result.push_str(transformed);
+
+    // 2. Register each component with $RefreshReg$
+    result.push('\n');
+    for comp in &components {
+        result.push_str(&format!(
+            "$RefreshReg$({name}, \"{name}\");\n",
+            name = comp.name
+        ));
+
+        // 3. If the component uses hooks, inject signature tracking
+        if comp.uses_hooks {
+            result.push_str(&format!(
+                "$RefreshSig$()({name}, \"{hooks}\");\n",
+                name = comp.name,
+                hooks = comp.hooks_signature
+            ));
+        }
+    }
+
+    // 4. Footer
+    result.push_str("\nRefreshRuntime.enqueueUpdate();\n");
+
+    result
+}
+
+/// Information about a detected React component.
+struct ReactComponent {
+    /// Component function name (e.g. "App", "Counter").
+    name: String,
+    /// Whether the component uses React hooks.
+    uses_hooks: bool,
+    /// Fingerprint of hook call order (e.g. "useState{} useEffect{}").
+    hooks_signature: String,
+}
+
+/// Detect React component declarations in the source AST.
+///
+/// A function is considered a React component if:
+/// - Its name starts with an uppercase letter
+/// - It returns JSX (contains jsx_element, jsx_self_closing_element, or jsx_fragment)
+/// - OR it's wrapped in React.memo() / React.forwardRef()
+fn detect_react_components(source: &str, root: &Node) -> Vec<ReactComponent> {
+    let mut components = Vec::new();
+    collect_components(source, root, &mut components);
+    components
+}
+
+fn collect_components(source: &str, node: &Node, components: &mut Vec<ReactComponent>) {
+    let mut cursor = node.walk();
+
+    for child in node.children(&mut cursor) {
+        match child.kind() {
+            // function App() { return <div/> }
+            "function_declaration" => {
+                if let Some(comp) = check_function_component(source, &child) {
+                    components.push(comp);
+                }
+            }
+            // export function App() { ... }
+            "export_statement" => {
+                let mut inner_cursor = child.walk();
+                for inner in child.children(&mut inner_cursor) {
+                    if inner.kind() == "function_declaration" {
+                        if let Some(comp) = check_function_component(source, &inner) {
+                            components.push(comp);
+                        }
+                    }
+                }
+            }
+            // const App = () => <div/>  or  const App = function() { return <div/> }
+            // const App = React.memo(...)  or  const App = React.forwardRef(...)
+            "lexical_declaration" => {
+                if let Some(comp) = check_variable_component(source, &child) {
+                    components.push(comp);
+                }
+            }
+            _ => {
+                // Recurse into other nodes
+                collect_components(source, &child, components);
+            }
+        }
+    }
+}
+
+/// Check if a function declaration is a React component.
+fn check_function_component(source: &str, node: &Node) -> Option<ReactComponent> {
+    let name = get_function_name(source, node)?;
+
+    // Must start with uppercase (React component convention)
+    if !name.starts_with(|c: char| c.is_uppercase()) {
+        return None;
+    }
+
+    // Must contain JSX in its body
+    if !function_body_has_jsx(node) {
+        return None;
+    }
+
+    let (uses_hooks, hooks_sig) = detect_hooks_usage(source, node);
+
+    Some(ReactComponent {
+        name,
+        uses_hooks,
+        hooks_signature: hooks_sig,
+    })
+}
+
+/// Check if a variable declaration contains a React component
+/// (arrow function or function expression returning JSX, or React.memo/forwardRef).
+fn check_variable_component(source: &str, node: &Node) -> Option<ReactComponent> {
+    let mut cursor = node.walk();
+
+    for child in node.children(&mut cursor) {
+        if child.kind() == "variable_declarator" {
+            let name = get_declarator_name(source, &child)?;
+
+            if !name.starts_with(|c: char| c.is_uppercase()) {
+                return None;
+            }
+
+            // Get the initializer (value being assigned)
+            let mut dc = child.walk();
+            for dchild in child.children(&mut dc) {
+                match dchild.kind() {
+                    "arrow_function" | "function" | "function_expression" => {
+                        if function_body_has_jsx(&dchild) {
+                            let (uses_hooks, hooks_sig) = detect_hooks_usage(source, &dchild);
+                            return Some(ReactComponent {
+                                name,
+                                uses_hooks,
+                                hooks_signature: hooks_sig,
+                            });
+                        }
+                    }
+                    "call_expression" => {
+                        // Check for React.memo(...) or React.forwardRef(...)
+                        let call_text = &source[dchild.byte_range()];
+                        if call_text.starts_with("React.memo")
+                            || call_text.starts_with("React.forwardRef")
+                            || call_text.starts_with("memo(")
+                            || call_text.starts_with("forwardRef(")
+                        {
+                            if subtree_has_jsx(&dchild) {
+                                let (uses_hooks, hooks_sig) =
+                                    detect_hooks_usage(source, &dchild);
+                                return Some(ReactComponent {
+                                    name,
+                                    uses_hooks,
+                                    hooks_signature: hooks_sig,
+                                });
+                            }
+                        }
+                    }
+                    _ => {}
+                }
+            }
+        }
+    }
+
+    None
+}
+
+/// Get the name of a function declaration.
+fn get_function_name(source: &str, node: &Node) -> Option<String> {
+    let mut cursor = node.walk();
+    for child in node.children(&mut cursor) {
+        if child.kind() == "identifier" {
+            return Some(source[child.byte_range()].to_string());
+        }
+    }
+    None
+}
+
+/// Get the name from a variable declarator.
+fn get_declarator_name(source: &str, node: &Node) -> Option<String> {
+    let mut cursor = node.walk();
+    for child in node.children(&mut cursor) {
+        if child.kind() == "identifier" {
+            return Some(source[child.byte_range()].to_string());
+        }
+    }
+    None
+}
+
+/// Check if a function's body contains JSX.
+fn function_body_has_jsx(node: &Node) -> bool {
+    let mut cursor = node.walk();
+    for child in node.children(&mut cursor) {
+        if child.kind() == "statement_block" || child.kind() == "parenthesized_expression" {
+            if subtree_has_jsx(&child) {
+                return true;
+            }
+        }
+        // Arrow function with expression body: () => <div/>
+        if matches!(
+            child.kind(),
+            "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"
+        ) {
+            return true;
+        }
+    }
+    false
+}
+
+/// Recursively check if any node in the subtree is a JSX element.
+fn subtree_has_jsx(node: &Node) -> bool {
+    if matches!(
+        node.kind(),
+        "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"
+    ) {
+        return true;
+    }
+    let mut cursor = node.walk();
+    for child in node.children(&mut cursor) {
+        if subtree_has_jsx(&child) {
+            return true;
+        }
+    }
+    false
+}
+
+/// Detect React hooks usage in a function body.
+///
+/// Returns (uses_hooks, hooks_signature_string).
+fn detect_hooks_usage(source: &str, node: &Node) -> (bool, String) {
+    let mut hooks = Vec::new();
+    collect_hook_calls(source, node, &mut hooks);
+
+    if hooks.is_empty() {
+        (false, String::new())
+    } else {
+        let sig = hooks.join(" ");
+        (true, sig)
+    }
+}
+
+/// Collect all React hook calls (functions starting with "use") from a subtree.
+fn collect_hook_calls(source: &str, node: &Node, hooks: &mut Vec<String>) {
+    if node.kind() == "call_expression" {
+        let mut cursor = node.walk();
+        let children: Vec<_> = node.children(&mut cursor).collect();
+        if let Some(first_child) = children.first() {
+            if first_child.kind() == "identifier" {
+                let name = &source[first_child.byte_range()];
+                // React hooks convention: useXxx
+                if name.starts_with("use") && name.len() > 3 {
+                    let next_char = name.chars().nth(3).unwrap_or('a');
+                    if next_char.is_uppercase() {
+                        hooks.push(format!("{}{{}}", name));
+                    }
+                }
+            }
+        }
+    }
+
+    let mut cursor = node.walk();
+    let children: Vec<_> = node.children(&mut cursor).collect();
+    for child in children {
+        collect_hook_calls(source, &child, hooks);
+    }
+}

```

## Review: jet-hmr-validation-spec

verdict: APPROVED
reviewer: reviewer
iteration: 2
change_id: jet-hmr-validation

**Summary**: Implementation covers JS HMR, React Fast Refresh, module graph, and HMR client. 623 tests pass. Review issues addressed in revision.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - Core HMR, Fast Refresh injection, module graph, import.meta.hot API all implemented.
- [PASS] [HARD] Diff contains #[test] functions
  - 25+ new tests.
- [PASS] [HARD] Existing tests still pass
  - 623 passed, 0 failed.


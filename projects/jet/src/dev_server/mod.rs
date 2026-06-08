// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use axum::{
    extract::{ws::WebSocket, FromRequestParts, State, WebSocketUpgrade},
    http::Request,
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod hmr;
pub mod hmr_client;
pub mod importmap;
pub mod incremental_rebuilder;
pub mod module_graph;
pub mod polyfills;
pub mod prebundle;
pub mod proxy;
pub mod react_refresh;
pub mod source_analysis;
pub mod watcher;

use hmr::{HmrManager, HmrMessage, HmrUpdateResult};
use module_graph::ModuleGraph;
use proxy::ProxyHandler;
use source_analysis::{
    build_error_frame, detect_hmr_accept_calls, extract_error_location,
    extract_imports_from_source, file_path_to_url, source_has_react_components,
};
use watcher::FileWatcher;

use crate::css::{CssPipeline, TailwindConfig};
use std::sync::RwLock;

/// GH #3811 — fallback extension string used when a requested file path
/// has no extension at all. Kept as a named constant so call sites and
/// tests pin the same value.
pub(crate) const DEV_SERVER_SERVE_FILE_NO_EXTENSION_FALLBACK: &str = "";

/// GH #3819 — fallback extension string used when an HMR-rebuild path
/// has no extension at all. Mirrors the serve_file fallback so call
/// sites and tests pin the same value, but kept named separately so
/// log-grep distinguishes the two call sites.
pub(crate) const DEV_SERVER_HMR_REBUILD_NO_EXTENSION_FALLBACK: &str = "";

/// GH #3819 — warn shown when the HMR rebuild loop is asked to
/// re-transform a path with no `extension()`. The prior code silently
/// dropped to `""`, falling onto the no-transform branch so a tsx file
/// with a malformed name would be shipped to the browser un-transformed
/// and the browser would report a parse error with no breadcrumb.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_dev_server_hmr_rebuild_no_extension_warn(path: &Path) -> String {
    format!(
        "gh3819: jet dev_server HMR rebuild saw path with no extension path={:?}; \
         falling back to empty extension — the no-transform pass-through arm will \
         ship the source verbatim to the browser, which may surface as a parse error \
         downstream",
        path
    )
}

/// GH #3819 — warn shown when the HMR rebuild loop is asked to
/// re-transform a path whose extension is non-UTF-8. The prior code
/// silently dropped to `""` because `.to_str()` returned `None`,
/// collapsing non-UTF-8 extensions onto the no-extension case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_dev_server_hmr_rebuild_non_utf8_extension_warn(
    path: &Path,
    lossy: &str,
) -> String {
    format!(
        "gh3819: jet dev_server HMR rebuild saw path with non-UTF-8 extension path={:?}; \
         lossy form is {:?}; routing through the lossy form so subsequent transform \
         arms see a visible breadcrumb instead of collapsing onto an empty extension",
        path, lossy
    )
}

/// GH #3819 — coerce the file extension into a string for the
/// HMR-rebuild transform dispatch ladder. Three-way branch:
/// - `Some(utf8)` → silent `Cow::Borrowed(utf8)`
/// - `Some(non-UTF-8)` → gh3819 warn + `Cow::Owned(lossy)`
/// - `None` → gh3819 warn + `Cow::Borrowed("")`
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn coerce_dev_server_hmr_rebuild_extension_or_warn(
    path: &Path,
) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match path.extension() {
        None => {
            tracing::warn!(
                target: "jet::dev::hmr",
                path = %path.display(),
                "{}",
                format_dev_server_hmr_rebuild_no_extension_warn(path)
            );
            Cow::Borrowed(DEV_SERVER_HMR_REBUILD_NO_EXTENSION_FALLBACK)
        }
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::dev::hmr",
                    path = %path.display(),
                    lossy = %lossy,
                    "{}",
                    format_dev_server_hmr_rebuild_non_utf8_extension_warn(path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

/// GH #3811 — warn shown when `serve_file` is asked to dispatch on a
/// path with no `extension()`. The prior code silently dropped to `""`
/// causing every downstream `ext == "..."` arm (js/cjs cache redirect,
/// css module, node_modules js wrap) to miss and the SPA shell to be
/// returned with no breadcrumb.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_dev_server_serve_file_no_extension_warn(file_path: &Path) -> String {
    format!(
        "gh3811: jet dev_server serve_file saw path with no extension file_path={:?}; \
         falling back to empty extension — every ext-driven dispatch arm \
         (js/cjs cache redirect, css module, node_modules js wrap) will silently \
         miss and the SPA shell will be returned",
        file_path
    )
}

/// GH #3811 — warn shown when `serve_file` is asked to dispatch on a
/// path whose extension is non-UTF-8. The prior code silently dropped
/// to `""` because `.to_str()` returned `None`, collapsing non-UTF-8
/// extensions onto the no-extension case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_dev_server_serve_file_non_utf8_extension_warn(
    file_path: &Path,
    lossy: &str,
) -> String {
    format!(
        "gh3811: jet dev_server serve_file saw path with non-UTF-8 extension file_path={:?}; \
         lossy form is {:?}; routing through the lossy form so subsequent \
         dispatch arms see a visible breadcrumb instead of collapsing onto an \
         empty extension",
        file_path, lossy
    )
}

/// GH #3811 — coerce the file extension into a string for the
/// `serve_file` extension-dispatch ladder.
///
/// - `Some(utf8)` → `Cow::Borrowed(utf8)` (silent — recognised UTF-8
///   extensions dispatch through `ext == "..."` arms; unrecognised ones
///   silently fall through to the SPA shell as before).
/// - `Some(non-UTF-8)` → emit a `tracing::warn!` carrying the lossy form
///   and return `Cow::Owned(lossy)` so operators see the encoding.
/// - `None` → emit a `tracing::warn!` naming the path and return
///   `Cow::Borrowed("")` so the historical SPA-shell fallback path is
///   preserved.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn coerce_dev_server_serve_file_extension_or_warn(
    file_path: &Path,
) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match file_path.extension() {
        None => {
            tracing::warn!(
                target: "jet::dev::serve",
                file_path = %file_path.display(),
                "{}",
                format_dev_server_serve_file_no_extension_warn(file_path)
            );
            Cow::Borrowed(DEV_SERVER_SERVE_FILE_NO_EXTENSION_FALLBACK)
        }
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::dev::serve",
                    file_path = %file_path.display(),
                    lossy = %lossy,
                    "{}",
                    format_dev_server_serve_file_non_utf8_extension_warn(file_path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

/// Development server with HMR support
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct DevServer {
    bundler: Arc<crate::bundler::Bundler>,
    watcher: Arc<FileWatcher>,
    hmr_manager: Arc<HmrManager>,
    config: ServerConfig,
    /// Optional CSS entry file processed by `CssPipeline` on every rebuild.
    css_entry: Option<PathBuf>,
    /// Content glob patterns from the Tailwind config (watched for class changes).
    css_content_globs: Vec<String>,
    /// Optional HTTP reverse proxy handler.
    proxy_handler: Option<Arc<ProxyHandler>>,
    /// Cached importmap JSON generated by the pre-bundler.
    /// `None` when pre-bundling hasn't run yet or produced no deps.
    importmap_json: Option<String>,
    /// Server-side module dependency graph for HMR boundary detection.
    module_graph: Arc<RwLock<ModuleGraph>>,
}

/// Server configuration
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub root_dir: PathBuf,
    pub public_dir: Option<PathBuf>,
    pub entry: PathBuf,
    /// Proxy map: path prefix → target URL.
    pub proxy: HashMap<String, String>,
    /// Import alias map: prefix → replacement path.
    pub aliases: HashMap<String, String>,
}

/// Server state shared across handlers
#[derive(Clone)]
struct ServerState {
    bundler: Arc<crate::bundler::Bundler>,
    hmr_manager: Arc<HmrManager>,
    config: ServerConfig,
    /// Optional proxy handler; `None` when no proxy rules are configured.
    proxy_handler: Option<Arc<ProxyHandler>>,
    /// Cached importmap JSON from pre-bundler.
    importmap_json: Option<String>,
    /// Shared module graph for HMR boundary detection.
    #[allow(dead_code)]
    module_graph: Arc<RwLock<ModuleGraph>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl DevServer {
    pub fn new(bundler: crate::bundler::Bundler, config: ServerConfig) -> Result<Self> {
        let bundler = Arc::new(bundler);
        let hmr_manager = Arc::new(HmrManager::new());

        let watcher = Arc::new(FileWatcher::new(config.root_dir.clone())?);

        // Build proxy handler when proxy routes are configured.
        let proxy_handler = if config.proxy.is_empty() {
            None
        } else {
            Some(Arc::new(ProxyHandler::new(config.proxy.clone())))
        };

        Ok(Self {
            bundler,
            watcher,
            hmr_manager,
            config,
            css_entry: None,
            css_content_globs: Vec::new(),
            proxy_handler,
            importmap_json: None,
            module_graph: Arc::new(RwLock::new(ModuleGraph::new())),
        })
    }

    /// Register a CSS entry file for the CSS pipeline.
    ///
    /// When registered, changes to `.css` files or to files matching
    /// `content_globs` will trigger a CSS pipeline rebuild followed by an
    /// HMR CSS-update message.
    pub fn register_css_entry(&mut self, css_entry: PathBuf, content_globs: Vec<String>) {
        self.css_entry = Some(css_entry);
        self.css_content_globs = content_globs;
    }

    pub async fn start(mut self: Arc<Self>) -> Result<()> {
        eprintln!("[jet] Starting...");
        // Pre-bundle CJS dependencies before starting the server
        let prebundler = prebundle::PreBundler::new(self.config.root_dir.clone());
        match prebundler.prebundle_deps().await {
            Ok(result) => {
                if !result.importmap_json.is_empty() {
                    // Safety: we haven't shared the Arc yet, so get_mut succeeds
                    if let Some(this) = Arc::get_mut(&mut self) {
                        this.importmap_json = Some(result.importmap_json);
                    }
                }
                if result.cache_hit {
                    eprintln!("[jet] Pre-bundle cache hit");
                } else {
                    eprintln!("[jet] Pre-bundling complete");
                }
            }
            Err(e) => {
                eprintln!("[jet] Pre-bundling failed (non-fatal): {}", e);
            }
        }

        let addr = format!("{}:{}", self.config.host, self.config.port).parse::<SocketAddr>()?;
        crate::dev_session::clear_shutdown_request(&self.config.root_dir);

        // Pre-bundle CJS dependencies into ESM for browser compatibility
        pre_bundle_cjs_deps(&self.config.root_dir);

        let app = self.create_router();

        self.start_file_watcher().await?;

        let listener = tokio::net::TcpListener::bind(addr).await?;
        // Resolve the actual bound address — essential when port is 0
        // (OS-assigned), but also correct for explicit ports.
        let actual_addr = listener.local_addr()?;
        crate::dev_session::write(
            &self.config.root_dir,
            &crate::dev_session::DevSession {
                mode: crate::dev_session::DevSessionMode::Dom,
                url: format!("http://{actual_addr}/"),
                host: actual_addr.ip().to_string(),
                port: actual_addr.port(),
                pid: std::process::id(),
                started_at: crate::dev_session::now_unix(),
            },
        )?;
        let proxy_info = if self.proxy_handler.is_some() {
            format!(" (proxy: {} rules)", self.config.proxy.len())
        } else {
            String::new()
        };
        eprintln!("\n  Jet dev server v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("  Local:   http://{}{}", actual_addr, proxy_info);
        if !self.config.proxy.is_empty() {
            for (path, target) in &self.config.proxy {
                eprintln!("  Proxy:   {} → {}", path, target);
            }
        }
        eprintln!();

        // Machine-readable ready signal on stdout for test harnesses and
        // programmatic consumers (e.g. Playwright). Always emitted, but
        // especially useful with `--port 0` so the caller can discover the
        // OS-assigned port.
        println!(
            "jet-dev-server:listening {{\"port\":{},\"host\":\"{}\"}}",
            actual_addr.port(),
            actual_addr.ip()
        );

        // GH #3725 — was `tokio::signal::ctrl_c().await.ok();` which
        // silently swallowed handler-registration errors. When ctrl_c
        // returns `Err` immediately (signal limits exhausted, sandboxed
        // runtime that forbids `sigaction`, etc.), `.ok()` discarded the
        // error and the shutdown future resolved on the very next poll
        // — `axum::serve(...).with_graceful_shutdown(...)` then shut the
        // server down right after printing the listening banner, with
        // no Ctrl+C from the user and no breadcrumb. From the user's
        // perspective `jet dev` started and then immediately exited.
        // Match on the result instead: warn explicitly on Err and park
        // the shutdown future forever so the server keeps serving until
        // the OS kills it (better UX than silent immediate exit).
        let dev_root = self.config.root_dir.clone();
        let shutdown = async move {
            let ctrl_c = async {
                match tokio::signal::ctrl_c().await {
                    Ok(()) => "Ctrl-C",
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::dev_server",
                            error = %err,
                            "{}",
                            format_dev_server_ctrl_c_warn(&err)
                        );
                        std::future::pending::<&'static str>().await
                    }
                }
            };
            let shutdown_request = wait_for_dev_shutdown_request(dev_root);
            let reason = tokio::select! {
                reason = ctrl_c => reason,
                _ = shutdown_request => "jet dev shutdown",
            };
            eprintln!("\n  Shutting down ({reason})...");
        };

        let server_result = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await;
        crate::dev_session::clear_shutdown_request(&self.config.root_dir);
        crate::dev_session::clear(&self.config.root_dir);
        server_result?;

        Ok(())
    }

    fn create_router(self: &Arc<Self>) -> Router {
        let state = ServerState {
            bundler: self.bundler.clone(),
            hmr_manager: self.hmr_manager.clone(),
            config: self.config.clone(),
            proxy_handler: self.proxy_handler.clone(),
            importmap_json: self.importmap_json.clone(),
            module_graph: self.module_graph.clone(),
        };

        Router::new()
            // HMR WebSocket — always served locally, never proxied.
            .route("/__jet_hmr", get(hmr_websocket_handler))
            // Root and wildcard use `any()` so proxy can handle all HTTP methods.
            .route("/", any(root_dispatch_handler))
            .route("/{*path}", any(path_dispatch_handler))
            .layer(axum::middleware::from_fn(request_logger))
            .with_state(state)
    }

    async fn start_file_watcher(self: &Arc<Self>) -> Result<()> {
        let watcher = self.watcher.clone();
        let hmr_manager = self.hmr_manager.clone();
        let _bundler = self.bundler.clone();
        let css_entry = self.css_entry.clone();
        let css_root = self.config.root_dir.clone();
        let css_content_globs = self.css_content_globs.clone();
        let module_graph = self.module_graph.clone();
        let root_dir = self.config.root_dir.clone();

        tokio::spawn(async move {
            let mut rx = watcher.subscribe();

            while let Ok(path) = rx.recv().await {
                tracing::info!("File changed: {:?}", path);

                let path_str = path.to_string_lossy().to_string();
                // GH #3680 — was `.unwrap()` which PANICS if the host
                // wall clock is before UNIX_EPOCH (Mac VM reset, container
                // without `--rtc`, freshly-booted devboard before NTP).
                // The panic happens inside tokio::spawn without an awaited
                // JoinHandle, so the dev-server process keeps running but
                // the watcher task dies silently — HMR appears to "stop
                // working" with no breadcrumb pointing at clock skew.
                // `safe_dev_server_now_ms` falls back to ms=0 and emits
                // a tagged warn so the watcher task survives.
                let (timestamp, ts_warn) = safe_dev_server_now_ms(std::time::SystemTime::now());
                if let Some(msg) = ts_warn {
                    tracing::warn!(target: "jet::dev_server", "{}", msg);
                }

                // CSS HMR: rebuild CSS pipeline when a .css file changes.
                let is_css_change = path_str.ends_with(".css");
                let is_js_change = matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("ts" | "tsx" | "js" | "jsx")
                );

                // For CSS files, rebuild CSS pipeline
                if is_css_change {
                    if let Some(ref css_entry_path) = css_entry {
                        if let Some(css_hmr) = rebuild_css(
                            css_entry_path,
                            &css_root,
                            &css_content_globs,
                            &path_str,
                            timestamp,
                        )
                        .await
                        {
                            hmr_manager.broadcast(css_hmr).await;
                            continue;
                        }
                    }
                }

                // For JS/TS files, also trigger CSS rebuild (Tailwind class changes)
                // then run module graph boundary detection.
                if is_js_change {
                    // Rebuild CSS in case Tailwind classes changed
                    if let Some(ref css_entry_path) = css_entry {
                        if let Some(css_hmr) = rebuild_css(
                            css_entry_path,
                            &css_root,
                            &css_content_globs,
                            &path_str,
                            timestamp,
                        )
                        .await
                        {
                            hmr_manager.broadcast(css_hmr).await;
                        }
                    }

                    // Compute URL path from filesystem path
                    let module_url = file_path_to_url(&path, &root_dir);

                    // Re-transform the file and extract imports
                    let source = match std::fs::read_to_string(&path) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!("Failed to read {}: {}", path_str, e);
                            continue;
                        }
                    };

                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let options = crate::transform::TransformOptions::default();
                    let transform_result = match ext {
                        "tsx" => crate::transform::transform_tsx::transform_tsx(&source, &options),
                        "ts" => {
                            crate::transform::typescript::transform_typescript(&source, &options)
                        }
                        "jsx" => crate::transform::jsx::transform_jsx(&source, &options),
                        _ => Ok(crate::transform::TransformResult {
                            code: source.clone(),
                            source_map: None,
                        }),
                    };

                    match transform_result {
                        Ok(result) => {
                            // Extract import paths from the transformed code
                            let imports = extract_imports_from_source(&result.code);
                            let has_react_components = source_has_react_components(&source);

                            // Detect import.meta.hot.accept() calls in original source
                            let (is_self_accepting, accepted_deps) =
                                detect_hmr_accept_calls(&source);

                            // Update module graph
                            {
                                let mut graph = module_graph.write().unwrap();
                                graph.update_module(&module_url, &path_str, &imports);
                                graph.set_has_react_refresh(&module_url, has_react_components);
                                graph.set_self_accepting(&module_url, is_self_accepting);
                                if !accepted_deps.is_empty() {
                                    graph.set_accepted_deps(
                                        &module_url,
                                        accepted_deps.into_iter().collect(),
                                    );
                                }
                                graph.set_timestamp(&module_url, timestamp);
                            }

                            // Run boundary detection
                            let boundary = {
                                let graph = module_graph.read().unwrap();
                                HmrUpdateResult::determine(&module_url, &graph)
                            };

                            match boundary {
                                HmrUpdateResult::HotUpdate { targets } => {
                                    for target in targets {
                                        let accepted_by = if target != module_url {
                                            Some(target.clone())
                                        } else {
                                            None
                                        };
                                        let message = HmrMessage::Update {
                                            path: module_url.clone(),
                                            timestamp,
                                            accepted_by,
                                        };
                                        hmr_manager.broadcast(message).await;
                                    }
                                }
                                HmrUpdateResult::FullReload { reason } => {
                                    let message = HmrMessage::FullReload { reason };
                                    hmr_manager.broadcast(message).await;
                                }
                            }
                        }
                        Err(e) => {
                            // Transform error — send error message with details
                            let err_msg = format!("{}", e);
                            let (err_line, err_column) = extract_error_location(&err_msg);
                            let frame = build_error_frame(&source, err_line);
                            let message = HmrMessage::Error {
                                message: err_msg,
                                file: Some(module_url),
                                line: err_line,
                                column: err_column,
                                frame: Some(frame),
                            };
                            hmr_manager.broadcast(message).await;
                        }
                    }
                    continue;
                }

                // Generic file update (non-JS/TS, non-CSS)
                let message = HmrMessage::Update {
                    path: path_str,
                    timestamp,
                    accepted_by: None,
                };
                hmr_manager.broadcast(message).await;
            }
        });

        Ok(())
    }
}

// ─── Request dispatch ─────────────────────────────────────────────────────────

/// Handle requests to `/` — check proxy first, then serve index.html.
async fn root_dispatch_handler(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Response {
    dispatch_request(state, req, "/").await
}

/// Handle requests to `/{*path}` — check proxy first, then serve static files.
async fn path_dispatch_handler(
    State(state): State<ServerState>,
    req: Request<axum::body::Body>,
) -> Response {
    let path = req.uri().path().to_string();
    dispatch_request(state, req, &path).await
}

/// Core dispatch logic: proxy → static files → SPA fallback.
async fn dispatch_request(
    state: ServerState,
    req: Request<axum::body::Body>,
    path: &str,
) -> Response {
    // 1. Check HTTP reverse proxy
    if let Some(proxy) = &state.proxy_handler {
        if proxy.match_target(path).is_some() {
            // Detect WebSocket upgrades by extracting from request parts.
            let (mut parts, body) = req.into_parts();
            match WebSocketUpgrade::from_request_parts(&mut parts, &state).await {
                Ok(ws_upgrade) => {
                    let proxy = proxy.clone();
                    let path_owned = path.to_string();
                    return ws_upgrade.on_upgrade(move |ws| async move {
                        proxy.forward_websocket(ws, &path_owned).await;
                    });
                }
                Err(_) => {
                    // Not a WebSocket — forward as plain HTTP / SSE.
                    let req = Request::from_parts(parts, body);
                    return proxy.forward_http(req).await;
                }
            }
        }
    }

    // 2. Serve static / bundle files
    let rel_path = path.trim_start_matches('/');

    if rel_path.is_empty() || rel_path == "index.html" {
        return serve_index_html(&state.config, state.importmap_json.as_deref()).await;
    }

    if rel_path == "bundle.js" || rel_path == "main.js" {
        return serve_bundle(state).await;
    }

    // Serve React Fast Refresh runtime shim
    if rel_path == "@react-refresh" {
        return serve_react_refresh();
    }

    if let Some(content) = serve_static_file(&state.config, rel_path).await {
        return content;
    }

    if let Some(content) = serve_root_file(&state.config, rel_path).await {
        return content;
    }

    // 3. SPA fallback — but NOT for node_modules (return 404 for missing deps)
    if rel_path.starts_with("node_modules/") {
        return (
            axum::http::StatusCode::NOT_FOUND,
            format!("Not found: {}", rel_path),
        )
            .into_response();
    }
    serve_index_html(&state.config, state.importmap_json.as_deref()).await
}

// ─── HMR WebSocket handler ────────────────────────────────────────────────────

async fn hmr_websocket_handler(ws: WebSocketUpgrade, State(state): State<ServerState>) -> Response {
    ws.on_upgrade(|socket| hmr_websocket(socket, state))
}

async fn hmr_websocket(socket: WebSocket, state: ServerState) {
    tracing::info!("New HMR client connected");

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.hmr_manager.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender
                .send(axum::extract::ws::Message::Text(json.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        use hmr::{ClientMessage, ConsoleLevel};
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Close(_) => break,
                axum::extract::ws::Message::Text(text) => {
                    let Some(client_msg) = parse_hmr_client_text_or_warn(&text) else {
                        continue;
                    };
                    match client_msg {
                        ClientMessage::ConsoleReport {
                            level,
                            message,
                            stack,
                            url,
                            line,
                            ..
                        } => {
                            let prefix = match level {
                                ConsoleLevel::Error => "\x1b[31m[browser error]\x1b[0m",
                                ConsoleLevel::Warn => "\x1b[33m[browser warn]\x1b[0m",
                            };
                            eprintln!("{} {}", prefix, message);
                            if let Some(u) = &url {
                                if let Some(l) = line {
                                    eprintln!("  at {}:{}", u, l);
                                }
                            }
                            if let Some(s) = &stack {
                                for frame in s.lines().take(10) {
                                    eprintln!("  {}", frame);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    tracing::info!("HMR client disconnected");
}

// ─── Static file serving ──────────────────────────────────────────────────────

async fn serve_bundle(state: ServerState) -> Response {
    let entry = state.config.root_dir.join(&state.config.entry);

    match state.bundler.bundle(entry).await {
        Ok(output) => {
            let code = output.code;

            (
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/javascript; charset=utf-8",
                )],
                code,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Bundle error: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Bundle error: {}", e),
            )
                .into_response()
        }
    }
}

async fn serve_index_html(config: &ServerConfig, importmap_json: Option<&str>) -> Response {
    let index_path = config.root_dir.join("index.html");
    let mut html = load_index_html_or_default(&index_path);

    // Inject importmap into the HTML if available
    if let Some(json) = importmap_json {
        html = importmap::inject_importmap_html(&html, json);
    }

    // Inject process polyfill for libraries that reference process.env (e.g. react-router)
    let process_polyfill = r#"<script>if(typeof process==='undefined'){window.process={env:{NODE_ENV:'development'}}}</script>"#;
    if let Some(pos) = html.find("</head>") {
        html.insert_str(pos, process_polyfill);
    } else if let Some(pos) = html.find("<script") {
        html.insert_str(pos, process_polyfill);
    }

    // Inject HMR client runtime before </body> (or at the end)
    let hmr_runtime = hmr_client::generate_hmr_runtime();
    if let Some(pos) = html.rfind("</body>") {
        html.insert_str(pos, &hmr_runtime);
    } else {
        html.push_str(&hmr_runtime);
    }

    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

/// Read `index_path` as UTF-8, falling back to the built-in template.
///
/// When the file exists but reading fails (permissions, transient I/O,
/// non-UTF-8), log the underlying error to stderr with a `[jet dev]`
/// prefix so the author can tell that their custom `index.html` was
/// shadowed by the default template — instead of silently serving a
/// different page than they wrote (GH #3103, same silent-swallow family
/// as #3061/#3065/#3086/#3094/#3097).
fn load_index_html_or_default(index_path: &Path) -> String {
    if !index_path.exists() {
        return default_index_html();
    }
    match std::fs::read_to_string(index_path) {
        Ok(html) => html,
        Err(e) => {
            eprintln!("[jet dev] Failed to read {}: {e}", index_path.display());
            eprintln!(
                "[jet dev] Falling back to the built-in default index.html; your custom index.html will NOT be served until the read error is fixed."
            );
            default_index_html()
        }
    }
}

fn default_index_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Jet Dev Server</title>
</head>
<body>
    <div id="root"></div>
    <script src="/bundle.js"></script>
</body>
</html>"#
        .to_string()
}

/// GH #3082 — true when the URL path contains a parent-dir (`..`)
/// component. Used to reject path-traversal attempts before any
/// disk-rooted `join`. Mirrors wasm_dev's pattern.
fn has_parent_dir_component(path: &str) -> bool {
    PathBuf::from(path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}

async fn serve_static_file(config: &ServerConfig, path: &str) -> Option<Response> {
    // GH #3082 — reject `..` components so a request like `/../etc/passwd`
    // can't escape `public_dir`. Mirrors wasm_dev::handle_static.
    if has_parent_dir_component(path) {
        return None;
    }
    let public_dir = config.public_dir.as_ref()?;
    let file_path = public_dir.join(path);

    if !file_path.exists() || !file_path.is_file() {
        return None;
    }

    // GH #3140 — once `exists()` and `is_file()` pass, a read failure
    // is unambiguously an error (permissions, transient FS error, file
    // raced out from under us). The pre-fix code used `.ok()?` which
    // returned `None` and fell through to `serve_root_file`, then to
    // the SPA `serve_index_html` fallback — so a request for an
    // existing-but-unreadable asset came back as the SPA shell with
    // HTTP 200. Surface this as 500 with the path + errno so the dev
    // gets a real diagnostic.
    let content = match std::fs::read(&file_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                target: "jet::dev::static",
                "failed to read static asset {:?}: {e} (GH #3140)",
                file_path
            );
            return Some(
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "[jet dev] static asset {} exists but could not be read: {e} (GH #3140)",
                        path
                    ),
                )
                    .into_response(),
            );
        }
    };
    let content_type = guess_content_type(&file_path);

    Some(
        (
            [(axum::http::header::CONTENT_TYPE, content_type.as_str())],
            content,
        )
            .into_response(),
    )
}

async fn serve_root_file(config: &ServerConfig, path: &str) -> Option<Response> {
    // GH #3082 — reject `..` components so a request like
    // `/../etc/passwd` can't escape `root_dir`. The hoisted-package walk
    // below still works because it only ever joins `path` onto an
    // ancestor of `root_dir` (never a `..`-bearing path).
    if has_parent_dir_component(path) {
        return None;
    }
    let mut file_path = config.root_dir.join(path);

    // For node_modules paths, walk up to find hoisted packages (pnpm workspace)
    if path.starts_with("node_modules/") && !file_path.exists() {
        let mut parent = config.root_dir.parent();
        while let Some(p) = parent {
            let candidate = p.join(path);
            if candidate.exists() {
                file_path = candidate;
                break;
            }
            if p.join(".git").exists() || p.join("pnpm-workspace.yaml").exists() {
                break;
            }
            parent = p.parent();
        }
    }

    // Check package.json "browser" field for Node→browser remapping
    // Handles both exact matches and directory-level remaps (e.g. node/ → browser/)
    let mut browser_remapped = false;
    if path.starts_with("node_modules/") {
        let rel = path.strip_prefix("node_modules/").unwrap();
        let (pkg_name, _) = if rel.starts_with('@') {
            let parts: Vec<&str> = rel.splitn(3, '/').collect();
            if parts.len() >= 2 {
                (format!("{}/{}", parts[0], parts[1]), "")
            } else {
                (rel.to_string(), "")
            }
        } else {
            let parts: Vec<&str> = rel.splitn(2, '/').collect();
            (parts[0].to_string(), "")
        };
        let pkg_json_path = config
            .root_dir
            .join("node_modules")
            .join(&pkg_name)
            .join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(browser) = pkg.get("browser").and_then(|b| b.as_object()) {
                    let file_rel = format!(
                        "./{}",
                        rel.strip_prefix(&format!("{}/", pkg_name)).unwrap_or(rel)
                    );

                    // 1. Exact match
                    if let Some(replacement) = browser.get(&file_rel) {
                        if replacement.is_boolean() && !replacement.as_bool().unwrap_or(true) {
                            return Some(
                                (
                                    [(axum::http::header::CONTENT_TYPE, "application/javascript")],
                                    "export default {};".to_string(),
                                )
                                    .into_response(),
                            );
                        }
                        if let Some(alt_path) = replacement.as_str() {
                            let alt_file = config
                                .root_dir
                                .join("node_modules")
                                .join(&pkg_name)
                                .join(alt_path.trim_start_matches("./"));
                            if alt_file.exists() {
                                file_path = alt_file;
                                browser_remapped = true;
                            }
                        }
                    }

                    // 2. Directory-level remap: if browser field maps a/index.js → b/index.js,
                    //    also remap a/foo.js → b/foo.js (for relative imports from remapped files)
                    if !browser_remapped && !file_path.exists() {
                        for (from_path, to_val) in browser {
                            if let Some(to_path) = to_val.as_str() {
                                let from_dir = from_path
                                    .trim_end_matches("/index.js")
                                    .trim_end_matches(".js");
                                let to_dir = to_path
                                    .trim_end_matches("/index.js")
                                    .trim_end_matches(".js");
                                if from_dir != from_path
                                    && file_rel.starts_with(&format!("{}/", from_dir))
                                {
                                    let remapped_rel = file_rel.replacen(from_dir, to_dir, 1);
                                    let alt_file = config
                                        .root_dir
                                        .join("node_modules")
                                        .join(&pkg_name)
                                        .join(remapped_rel.trim_start_matches("./"));
                                    if alt_file.exists() {
                                        file_path = alt_file;
                                        browser_remapped = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Resolve package.json "exports" for subpath imports (e.g. @cclab/ui/layout → src/layout/index.ts)
    if !file_path.exists() && path.starts_with("node_modules/") {
        let rel = path.strip_prefix("node_modules/").unwrap();
        // Split into package name and subpath
        let (pkg_name, subpath) = if rel.starts_with('@') {
            // Scoped: @scope/name/subpath
            let parts: Vec<&str> = rel.splitn(3, '/').collect();
            if parts.len() >= 3 {
                (
                    format!("{}/{}", parts[0], parts[1]),
                    format!("./{}", parts[2]),
                )
            } else {
                (rel.to_string(), ".".to_string())
            }
        } else {
            let parts: Vec<&str> = rel.splitn(2, '/').collect();
            if parts.len() >= 2 {
                (parts[0].to_string(), format!("./{}", parts[1]))
            } else {
                (rel.to_string(), ".".to_string())
            }
        };

        // Find the package dir in node_modules
        let mut pkg_dir = config.root_dir.join("node_modules").join(&pkg_name);
        if !pkg_dir.exists() {
            let mut p = config.root_dir.parent();
            while let Some(pp) = p {
                let candidate = pp.join("node_modules").join(&pkg_name);
                if candidate.exists() {
                    pkg_dir = candidate;
                    break;
                }
                if pp.join(".git").exists() {
                    break;
                }
                p = pp.parent();
            }
        }

        if subpath != "." {
            let pkg_json_path = pkg_dir.join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_json_path) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(exports) = pkg.get("exports").and_then(|e| e.as_object()) {
                        if let Some(target) = exports.get(&subpath) {
                            if let Some(resolved) = prebundle::resolve_exports_entry(target) {
                                let resolved = resolved.trim_start_matches("./");
                                let candidate = pkg_dir.join(resolved);
                                if candidate.exists() {
                                    file_path = candidate;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Resolve extensionless imports: try .tsx, .ts, .jsx, .js, /index.tsx, /index.ts
    if path.contains("platform/node") {}
    let file_path = if file_path.exists() && file_path.is_file() {
        file_path
    } else {
        let extensions = ["tsx", "ts", "jsx", "js"];
        let mut resolved = None;
        for ext in &extensions {
            let candidate = file_path.with_extension(ext);
            if candidate.exists() && candidate.is_file() {
                resolved = Some(candidate);
                break;
            }
        }
        if resolved.is_none() {
            for ext in &extensions {
                let candidate = file_path.join(format!("index.{}", ext));
                if candidate.exists() && candidate.is_file() {
                    resolved = Some(candidate);
                    break;
                }
            }
        }
        match resolved {
            Some(p) => p,
            None => return None,
        }
    };

    if path.starts_with("node_modules/")
        && Path::new(path).extension().is_none()
        && file_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "index.js")
    {
        let location = if path.ends_with('/') {
            format!("/{path}index.js")
        } else {
            format!("/{path}/index.js")
        };
        return Some(
            (
                axum::http::StatusCode::TEMPORARY_REDIRECT,
                [(axum::http::header::LOCATION, location)],
            )
                .into_response(),
        );
    }

    let ext_cow = coerce_dev_server_serve_file_extension_or_warn(&file_path);
    let ext = ext_cow.as_ref();

    // Redirect node_modules CJS files to pre-bundled .jet/ versions
    // Skip if browser field remapped the file (use the remapped file, not the cached original)
    if !browser_remapped
        && path.contains("node_modules/")
        && !path.contains("node_modules/.jet/")
        && matches!(ext, "js" | "cjs")
    {
        let rel = path.trim_start_matches("node_modules/");
        let safe_name = rel
            .trim_end_matches(".js")
            .trim_end_matches(".cjs")
            .replace('/', "__");
        let jet_cached = config
            .root_dir
            .join("node_modules/.jet")
            .join(format!("{}.mjs", safe_name));
        if jet_cached.exists() {
            if let Ok(content) = std::fs::read_to_string(&jet_cached) {
                return Some(
                    (
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "application/javascript; charset=utf-8",
                        )],
                        content,
                    )
                        .into_response(),
                );
            }
        }
    }

    // CSS module imports: process through CssPipeline and return JS that
    // injects a <style> tag with the compiled CSS.  This ensures @tailwind
    // directives are expanded before the browser sees the stylesheet.
    if ext == "css" {
        // GH #3143 — surface IO errors instead of returning None and
        // falling through to the SPA shell, which causes the browser
        // to refuse the response as "wrong MIME type" with no path
        // breadcrumb back to the actual read failure.
        let css_source = match std::fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target: "jet::dev::serve",
                    "failed to read CSS module {:?}: {e} (GH #3143)",
                    file_path
                );
                return Some(
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!(
                            "[jet dev] CSS module {} exists but could not be read: {e} (GH #3143)",
                            path
                        ),
                    )
                        .into_response(),
                );
            }
        };
        let processed_css = if crate::css::directives::has_tailwind_directives(&css_source) {
            // Run through full CSS pipeline (Tailwind JIT + directives + lightningcss)
            // GH #3086 — surface tailwind.config.js / [css.tailwind] parse errors
            // instead of silently falling back to defaults.
            let tw_config = match TailwindConfig::load(&config.root_dir) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("[jet dev] Failed to parse Tailwind config: {e:#}");
                    eprintln!("[jet dev] Continuing with built-in Tailwind defaults; your tailwind.config.js / [css.tailwind] settings will NOT take effect until the parse error is fixed.");
                    TailwindConfig::default()
                }
            };
            let pipeline = CssPipeline::new(config.root_dir.clone(), tw_config, false);
            match pipeline.process(&file_path) {
                Ok(output) => output.css,
                Err(e) => {
                    tracing::warn!("CSS pipeline error for {}: {}", path, e);
                    css_source
                }
            }
        } else {
            css_source
        };

        // Escape the CSS for embedding in a JS template literal
        let escaped = processed_css
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");

        let js = format!(
            "const style = document.createElement('style');\nstyle.setAttribute('data-jet-css', '{}');\nstyle.textContent = `{}`;\ndocument.head.appendChild(style);\n",
            path, escaped
        );
        return Some(
            (
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/javascript; charset=utf-8",
                )],
                js,
            )
                .into_response(),
        );
    }

    // Plain .js files in node_modules: wrap CJS on-the-fly if needed
    if ext == "js" && path.contains("node_modules/") {
        // GH #3143 — surface IO errors instead of returning None and
        // falling through to the SPA shell, which causes the browser
        // to refuse the response as "wrong MIME type" with no path
        // breadcrumb back to the actual read failure.
        let source = match std::fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target: "jet::dev::serve",
                    "failed to read node_modules JS {:?}: {e} (GH #3143)",
                    file_path
                );
                return Some(
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!(
                            "[jet dev] node_modules JS {} exists but could not be read: {e} (GH #3143)",
                            path
                        ),
                    )
                        .into_response(),
                );
            }
        };
        let module_url = format!("/{}", path);
        let hot_preamble = hmr_client::generate_hot_preamble(&module_url);

        // Detect CJS: has `require(`, `module.exports`, or `exports.X =` but no top-level import/export
        let is_cjs = (source.contains("require(")
            || source.contains("module.exports")
            || source.contains("exports.")
            || source.contains("Object.defineProperty(exports"))
            && !source.starts_with("import ")
            && !source.contains("\nexport ");

        // Resolve Node.js subpath imports (#foo → actual path) in node_modules.
        // GH #3234 — both call sites use the same helper now; read/parse errors warn.
        let mut resolved_source = source;
        if resolved_source.contains("'#") || resolved_source.contains("\"#") {
            if let Some(pkg_root) = find_package_root(&file_path) {
                let pkg_json_path = pkg_root.join("package.json");
                if let Ok(pkg_content) = std::fs::read_to_string(&pkg_json_path) {
                    if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_content) {
                        if let Some(imports) = pkg.get("imports").and_then(|v| v.as_object()) {
                            for (key, val) in imports {
                                let resolved = val
                                    .get("default")
                                    .or_else(|| val.get("browser"))
                                    .and_then(|v| v.as_str())
                                    .or_else(|| val.as_str());
                                if let Some(target) = resolved {
                                    let pkg_rel = pkg_root
                                        .strip_prefix(&config.root_dir)
                                        .ok()
                                        .map(|p| format!("/{}", p.display()))
                                        .unwrap_or_default();
                                    let abs_target = if target.starts_with("./") {
                                        format!("{}/{}", pkg_rel, &target[2..])
                                    } else {
                                        format!("{}/{}", pkg_rel, target)
                                    };
                                    resolved_source = resolved_source.replace(
                                        &format!("from '{}'", key),
                                        &format!("from '{}'", abs_target),
                                    );
                                    resolved_source = resolved_source.replace(
                                        &format!("from \"{}\"", key),
                                        &format!("from \"{}\"", abs_target),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let output = if is_cjs {
            let named = extract_named_reexports(&resolved_source);

            // Collect require('...') deps and generate import statements
            let require_re = regex::Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
            let mut deps: Vec<String> = Vec::new();
            for cap in require_re.captures_iter(&resolved_source) {
                let dep = cap[1].to_string();
                if !deps.contains(&dep) {
                    deps.push(dep);
                }
            }

            let mut wrapped = String::with_capacity(
                hot_preamble.len() + resolved_source.len() + named.len() + 500,
            );
            wrapped.push_str(&hot_preamble);

            // Import deps so the require shim can return them
            for (i, dep) in deps.iter().enumerate() {
                // Resolve: relative paths stay relative, bare specifiers go to importmap
                if dep.starts_with('.') || dep.starts_with('/') {
                    wrapped.push_str(&format!("import __cjs_dep_{}__ from '{}';\n", i, dep));
                } else {
                    // Bare specifier — use importmap resolution (just import it)
                    wrapped.push_str(&format!("import __cjs_dep_{}__ from '{}';\n", i, dep));
                }
            }

            wrapped.push_str("var module = { exports: {} };\nvar exports = module.exports;\n");
            wrapped.push_str(&format!(
                "(function(module, exports, require) {{\n{}\n}})(module, exports, function require(id) {{\n",
                resolved_source
            ));
            for (i, dep) in deps.iter().enumerate() {
                wrapped.push_str(&format!(
                    "  if (id === '{}') return __cjs_dep_{}__;\n",
                    dep, i
                ));
            }
            wrapped.push_str("  console.warn('[jet] Dynamic require(\"' + id + '\") — no pre-bundled module');\n  return {};\n});\n");
            wrapped.push_str("export default module.exports;\n");
            if !named.is_empty() {
                wrapped.push_str(&named);
                wrapped.push('\n');
            }
            wrapped
        } else {
            format!("{hot_preamble}{resolved_source}")
        };

        return Some(
            (
                [(
                    axum::http::header::CONTENT_TYPE,
                    "application/javascript; charset=utf-8",
                )],
                output,
            )
                .into_response(),
        );
    }

    // Transform TypeScript/TSX files through the AST-based transformer
    if matches!(ext, "tsx" | "ts" | "jsx" | "js") {
        // GH #3146 — surface IO errors instead of silently falling
        // through to the SPA shell. This is the hottest path in a
        // modern jet project (every `.tsx` import lands here); a
        // silent fallthrough makes the browser refuse the response
        // with "wrong MIME type" and no breadcrumb back to the
        // actual read failure.
        let source = match std::fs::read_to_string(&file_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    target: "jet::dev::serve",
                    "failed to read {} module {:?}: {e} (GH #3146)",
                    ext,
                    file_path
                );
                return Some(
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!(
                            "[jet dev] {} module {} exists but could not be read: {e} (GH #3146)",
                            ext, path
                        ),
                    )
                        .into_response(),
                );
            }
        };
        let options = crate::transform::TransformOptions::default();
        let result = match ext {
            "tsx" => crate::transform::transform_tsx::transform_tsx(&source, &options),
            "ts" => crate::transform::typescript::transform_typescript(&source, &options),
            "jsx" => crate::transform::jsx::transform_jsx(&source, &options),
            "js" => Ok(crate::transform::TransformResult {
                code: source.clone(),
                source_map: None,
            }),
            _ => unreachable!(),
        };
        match result {
            Ok(transformed) => {
                // Inject import.meta.hot preamble for HMR support
                let module_url = format!("/{}", path);
                let hot_preamble = hmr_client::generate_hot_preamble(&module_url);

                // Strip residual TypeScript syntax not handled by transform
                let stripped = transformed
                    .code
                    .lines()
                    .filter(|line| {
                        let t = line.trim();
                        !t.starts_with("export type ")
                            && !t.starts_with("import type ")
                            && !t.starts_with("export interface ")
                            && !t.starts_with("interface ")
                            && !(t.starts_with("type ") && t.contains(" = "))
                            && t != "export"
                    })
                    .map(|line| {
                        // Strip inline type imports: import { type X, Y } → import { Y }
                        if line.contains("import ") && line.contains(" type ") {
                            let mut s = line.to_string();
                            while let Some(idx) = s.find(", type ") {
                                let rest = &s[idx + 7..];
                                let end = rest
                                    .find(|c: char| c == ',' || c == ' ' || c == '}')
                                    .unwrap_or(rest.len());
                                s = format!("{}{}", &s[..idx], &s[idx + 7 + end..]);
                            }
                            while let Some(idx) = s.find("{ type ") {
                                let rest = &s[idx + 7..];
                                let end = rest
                                    .find(|c: char| c == ',' || c == ' ' || c == '}')
                                    .unwrap_or(rest.len());
                                let after = &s[idx + 7 + end..];
                                let after = after
                                    .strip_prefix(", ")
                                    .or(after.strip_prefix(","))
                                    .unwrap_or(after);
                                s = format!("{}{{ {}", &s[..idx], after);
                            }
                            s
                        } else {
                            line.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Apply alias rewriting from config (e.g. @/ → /src/)
                let mut code = stripped;
                for (prefix, replacement) in &config.aliases {
                    let abs = if replacement.starts_with("./") {
                        format!("/{}", &replacement[2..])
                    } else {
                        format!("/{}", replacement)
                    };
                    code = code.replace(&format!("from '{}", prefix), &format!("from '{}", abs));
                    code = code.replace(&format!("from \"{}", prefix), &format!("from \"{}", abs));
                }

                // Resolve Node.js subpath imports (#foo → actual path)
                // Used by packages like vfile: #minpath → ./lib/minpath.browser.js
                if code.contains("'#") || code.contains("\"#") {
                    if let Some(pkg_root) = find_package_root(&file_path) {
                        let pkg_json_path = pkg_root.join("package.json");
                        if let Ok(pkg_content) = std::fs::read_to_string(&pkg_json_path) {
                            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_content)
                            {
                                if let Some(imports) =
                                    pkg.get("imports").and_then(|v| v.as_object())
                                {
                                    for (key, val) in imports {
                                        // Resolve: prefer "default" (browser) over "node"
                                        let resolved = val
                                            .get("default")
                                            .or_else(|| val.get("browser"))
                                            .and_then(|v| v.as_str())
                                            .or_else(|| val.as_str());
                                        if let Some(target) = resolved {
                                            let pkg_rel = pkg_root
                                                .strip_prefix(&config.root_dir)
                                                .ok()
                                                .map(|p| format!("/{}", p.display()))
                                                .unwrap_or_default();
                                            let abs_target = if target.starts_with("./") {
                                                format!("{}/{}", pkg_rel, &target[2..])
                                            } else {
                                                format!("{}/{}", pkg_rel, target)
                                            };
                                            code = code.replace(
                                                &format!("from '{}'", key),
                                                &format!("from '{}'", abs_target),
                                            );
                                            code = code.replace(
                                                &format!("from \"{}\"", key),
                                                &format!("from \"{}\"", abs_target),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Rewrite bare npm specifiers to URL paths the browser can resolve
                let code = rewrite_bare_specifiers(&code, &config.root_dir);

                let mut final_code = String::with_capacity(hot_preamble.len() + code.len());
                final_code.push_str(&hot_preamble);
                final_code.push_str(&code);

                return Some(
                    (
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "application/javascript; charset=utf-8",
                        )],
                        final_code,
                    )
                        .into_response(),
                );
            }
            Err(e) => {
                tracing::error!("Transform error for {}: {}", path, e);
                return Some(
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Transform error: {}", e),
                    )
                        .into_response(),
                );
            }
        }
    }

    // GH #3146 — generic fallthrough branch. Same silent-`.ok()?`
    // pattern as the other branches; surface the IO error so the
    // caller doesn't fall through to the SPA shell.
    let content = match std::fs::read(&file_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                target: "jet::dev::serve",
                "failed to read root file {:?}: {e} (GH #3146)",
                file_path
            );
            return Some(
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "[jet dev] root file {} exists but could not be read: {e} (GH #3146)",
                        path
                    ),
                )
                    .into_response(),
            );
        }
    };
    let content_type = guess_content_type(&file_path);

    Some(
        (
            [(axum::http::header::CONTENT_TYPE, content_type.as_str())],
            content,
        )
            .into_response(),
    )
}

/// Pre-bundle CJS dependencies into ESM format for browser dev mode.
///
/// Scans `package.json` dependencies, checks each for CJS format, and creates
/// ESM-wrapped versions in `node_modules/.jet/`.
fn pre_bundle_cjs_deps(root_dir: &PathBuf) {
    let pkg_json_path = root_dir.join("package.json");
    if !pkg_json_path.exists() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(&pkg_json_path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };

    let jet_dir = root_dir.join("node_modules").join(".jet");
    if let Err(e) = std::fs::create_dir_all(&jet_dir) {
        tracing::warn!(
            target: "jet::dev_server::prebundle",
            "skipping secondary CJS pre-bundle: cannot create {:?}: {e} (GH #3201)",
            jet_dir
        );
        return;
    }

    let mut bundled = 0u32;
    let deps = parsed.get("dependencies").and_then(|d| d.as_object());
    let Some(deps) = deps else { return };

    for (name, _) in deps {
        let cached = jet_dir.join(format!("{}.mjs", name.replace('/', "__")));
        if cached.exists() {
            continue; // already cached
        }

        // Find the package's main entry
        let node_modules = root_dir.join("node_modules");
        let pkg_dir = node_modules.join(name);
        let pkg_json = pkg_dir.join("package.json");
        if !pkg_json.exists() {
            continue;
        }

        let Ok(pkg_content) = std::fs::read_to_string(&pkg_json) else {
            continue;
        };
        let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_content) else {
            continue;
        };

        // Skip if package already has ESM (module field or exports with import condition)
        if pkg.get("module").is_some() {
            continue;
        }
        if let Some(exports) = pkg.get("exports").and_then(|e| e.get(".")) {
            if exports.get("import").is_some() {
                continue; // has ESM export
            }
        }

        // Resolve main entry
        let main_entry = pkg
            .get("main")
            .and_then(|v| v.as_str())
            .unwrap_or("index.js");
        let entry_path = pkg_dir.join(main_entry);
        if !entry_path.exists() {
            continue;
        }

        let Ok(entry_source) = std::fs::read_to_string(&entry_path) else {
            continue;
        };

        // Check if actually CJS
        if !entry_source.contains("module.exports") && !entry_source.contains("exports.") {
            continue;
        }

        // If entry is a conditional env branch (common React pattern), resolve to dev file directly
        let resolved_source = if entry_source.contains("process.env.NODE_ENV") {
            // Extract the development require path
            let dev_req = entry_source
                .lines()
                .find(|l| l.contains("require(") && !l.contains("production"))
                .and_then(|l| extract_require_path(l.trim()));
            if let Some(dev_path) = dev_req {
                let dev_file = resolve_cjs_require(&pkg_dir, &dev_path);
                if let Some(dev_file) = dev_file {
                    match std::fs::read_to_string(&dev_file) {
                        Ok(s) => s,
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            entry_source.clone()
                        }
                        Err(err) => {
                            tracing::warn!(
                                target: "jet::dev_server::prebundle",
                                path = %dev_file.display(),
                                error = %err,
                                "GH #3244 failed to read dev branch source; falling back to entry"
                            );
                            entry_source.clone()
                        }
                    }
                } else {
                    entry_source.clone()
                }
            } else {
                entry_source.clone()
            }
        } else {
            entry_source.clone()
        };

        // Flatten remaining require('./...') calls
        let mut flattened = String::new();
        flatten_cjs(
            &resolved_source,
            &pkg_dir,
            &mut flattened,
            &mut std::collections::HashSet::new(),
        );

        // Detect external require() calls to other packages (not relative)
        let mut dep_imports = String::new();
        let mut dep_cache = String::new();
        let mut seen_deps = std::collections::HashSet::new();
        for line in flattened.lines() {
            if let Some(req) = extract_require_path(line.trim()) {
                if !req.starts_with('.') && !req.starts_with('/') && seen_deps.insert(req.clone()) {
                    let safe = req.replace('/', "__");
                    let jet_path = format!("/node_modules/.jet/{}.mjs", safe);
                    // Import dep to ensure it loads first and populates cache
                    dep_imports.push_str(&format!("import __dep_{safe}__ from '{jet_path}';\n",));
                    dep_cache.push_str(&format!(
                        "if (!window.__jetRequireCache['{req}']) window.__jetRequireCache['{req}'] = __dep_{safe}__;\n",
                    ));
                }
            }
        }

        let esm = format!(
            "// Pre-bundled by Jet (CJS→ESM) — {name}\n\
             {dep_imports}\
             var module = {{exports: {{}}}};\n\
             var exports = module.exports;\n\
             var process = {{env: {{NODE_ENV: 'development'}}}};\n\
             if (!window.__jetRequireCache) window.__jetRequireCache = {{}};\n\
             {dep_cache}\
             var require = function(id) {{\n\
               if (window.__jetRequireCache[id]) return window.__jetRequireCache[id];\n\
               throw new Error('[jet] require(\"' + id + '\") not available');\n\
             }};\n\
             {flattened}\n\
             window.__jetRequireCache['{name}'] = module.exports;\n\
             export default module.exports;\n\
             {named}\n",
            name = name,
            dep_imports = dep_imports,
            dep_cache = dep_cache,
            flattened = flattened,
            named = extract_named_reexports(&flattened),
        );

        match std::fs::write(&cached, &esm) {
            Ok(()) => bundled += 1,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    path = %cached.display(),
                    error = %err,
                    "GH #3244 failed to write prebundled CJS cache for {name}"
                );
            }
        }
    }

    // Pre-bundle transitive CJS deps discovered during require scanning
    let transitive_cjs = discover_transitive_cjs_deps(&jet_dir, &root_dir.join("node_modules"));
    for dep_name in &transitive_cjs {
        let cached = jet_dir.join(format!("{}.mjs", dep_name.replace('/', "__")));
        if cached.exists() {
            continue;
        }
        let pkg_dir = root_dir.join("node_modules").join(dep_name);
        let pkg_json = pkg_dir.join("package.json");
        if !pkg_json.exists() {
            continue;
        }
        let Ok(pkg_content) = std::fs::read_to_string(&pkg_json) else {
            continue;
        };
        let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_content) else {
            continue;
        };

        let main_entry = pkg
            .get("main")
            .and_then(|v| v.as_str())
            .unwrap_or("index.js");
        let entry_path = pkg_dir.join(main_entry);
        if !entry_path.exists() {
            continue;
        }
        let Ok(entry_source) = std::fs::read_to_string(&entry_path) else {
            continue;
        };

        let resolved_source = if entry_source.contains("process.env.NODE_ENV") {
            let dev_req = entry_source
                .lines()
                .find(|l| l.contains("require(") && !l.contains("production"))
                .and_then(|l| extract_require_path(l.trim()));
            if let Some(dev_path) = dev_req {
                match resolve_cjs_require(&pkg_dir, &dev_path) {
                    Some(f) => match std::fs::read_to_string(&f) {
                        Ok(s) => s,
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                            entry_source.clone()
                        }
                        Err(err) => {
                            tracing::warn!(
                                target: "jet::dev_server::prebundle",
                                path = %f.display(),
                                error = %err,
                                "GH #3244 failed to read transitive dev branch source; falling back to entry"
                            );
                            entry_source.clone()
                        }
                    },
                    None => entry_source.clone(),
                }
            } else {
                entry_source.clone()
            }
        } else {
            entry_source.clone()
        };

        let mut flattened = String::new();
        flatten_cjs(
            &resolved_source,
            &pkg_dir,
            &mut flattened,
            &mut std::collections::HashSet::new(),
        );

        let esm = format!(
            "// Pre-bundled by Jet (CJS→ESM) — {dep_name}\n\
             var module = {{exports: {{}}}};\n\
             var exports = module.exports;\n\
             var process = {{env: {{NODE_ENV: 'development'}}}};\n\
             if (!window.__jetRequireCache) window.__jetRequireCache = {{}};\n\
             var require = function(id) {{\n\
               if (window.__jetRequireCache[id]) return window.__jetRequireCache[id];\n\
               throw new Error('[jet] require not available: ' + id);\n\
             }};\n\
             {flattened}\n\
             window.__jetRequireCache['{dep_name}'] = module.exports;\n\
             export default module.exports;\n",
        );

        match std::fs::write(&cached, &esm) {
            Ok(()) => bundled += 1,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    path = %cached.display(),
                    error = %err,
                    "GH #3244 failed to write transitive prebundled CJS cache for {dep_name}"
                );
            }
        }
    }

    // Also pre-bundle subpath entries (react/jsx-runtime, react/jsx-dev-runtime)
    for subpath in &[
        "react/jsx-runtime",
        "react/jsx-dev-runtime",
        "react-dom/client",
    ] {
        let cached = jet_dir.join(format!("{}.mjs", subpath.replace('/', "__")));
        if cached.exists() {
            continue;
        }

        let parts: Vec<&str> = subpath.split('/').collect();
        let pkg_dir = root_dir.join("node_modules").join(parts[0]);
        let subpath_file = pkg_dir.join(format!("{}.js", parts[1]));
        if !subpath_file.exists() {
            continue;
        }

        let Ok(source) = std::fs::read_to_string(&subpath_file) else {
            continue;
        };
        if !source.contains("module.exports") && !source.contains("require(") {
            continue;
        }

        let mut flattened = String::new();
        flatten_cjs(
            &source,
            &pkg_dir,
            &mut flattened,
            &mut std::collections::HashSet::new(),
        );

        // Subpath entries use the global require cache (parent must load first)
        let esm = format!(
            "// Pre-bundled by Jet (CJS→ESM) — {subpath}\n\
             var module = {{exports: {{}}}};\n\
             var exports = module.exports;\n\
             var process = {{env: {{NODE_ENV: 'development'}}}};\n\
             if (!window.__jetRequireCache) window.__jetRequireCache = {{}};\n\
             var require = function(id) {{\n\
               if (window.__jetRequireCache[id]) return window.__jetRequireCache[id];\n\
               throw new Error('[jet] require(\"' + id + '\") not available');\n\
             }};\n\
             {flattened}\n\
             window.__jetRequireCache['{subpath}'] = module.exports;\n\
             export default module.exports;\n\
             {named}\n",
            subpath = subpath,
            flattened = flattened,
            named = extract_named_reexports(&flattened),
        );

        match std::fs::write(&cached, &esm) {
            Ok(()) => bundled += 1,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    path = %cached.display(),
                    error = %err,
                    "GH #3244 failed to write subpath prebundled cache for {subpath}"
                );
            }
        }
    }

    if bundled > 0 {
        eprintln!("  Pre-bundled {} CJS dependencies → ESM", bundled);
    }
}

/// Discover transitive CJS deps by scanning pre-bundled .mjs files for import statements
/// pointing to `.jet/` paths that don't exist yet.
///
/// GH #3215 — the prior implementation silently swallowed three classes
/// of IO failure: \[1\] `read_dir(jet_dir)` returning an error, \[2\]
/// `entries.flatten()` dropping per-dirent IO errors, and \[3\]
/// `read_to_string(entry.path())` failing on a sibling cache file. Each
/// silent failure removed transitive deps from the prebundle queue,
/// causing the next page load to 404 a module with zero breadcrumb back
/// to the failed scan. The function is now explicit: NotFound on
/// `read_dir` is the only legitimate silent path (matches the
/// `!jet_dir.exists()` early-return contract); every other failure is
/// surfaced via `tracing::warn!`.
fn discover_transitive_cjs_deps(
    jet_dir: &std::path::Path,
    node_modules: &std::path::Path,
) -> Vec<String> {
    let mut missing = Vec::new();
    if !jet_dir.exists() {
        return missing;
    }
    if let Ok(entries) = std::fs::read_dir(jet_dir) {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for line in content.lines() {
                    // Match: import __dep_xxx__ from '/node_modules/.jet/xxx.mjs';
                    if line.contains("/node_modules/.jet/") && line.contains("import ") {
                        if let Some(start) = line.find("/node_modules/.jet/") {
                            let rest = &line[start + "/node_modules/.jet/".len()..];
                            if let Some(end) = rest.find(".mjs") {
                                let dep_safe = &rest[..end];
                                let dep_name = dep_safe.replace("__", "/");
                                let cached = jet_dir.join(format!("{}.mjs", dep_safe));
                                if !cached.exists()
                                    && node_modules.join(&dep_name).join("package.json").exists()
                                {
                                    missing.push(dep_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

/// Recursively flatten CJS require('./relative') calls by inlining file contents.
fn flatten_cjs(
    source: &str,
    base_dir: &std::path::Path,
    output: &mut String,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    for line in source.lines() {
        // Match: require('./something') or require("./something")
        let trimmed = line.trim();
        if let Some(req_path) = extract_require_path(trimmed) {
            if req_path.starts_with("./") || req_path.starts_with("../") {
                let resolved = resolve_cjs_require(base_dir, &req_path);
                if let Some(resolved_path) = resolved {
                    if visited.insert(resolved_path.clone()) {
                        // GH #3272 — the prior `if let Ok(content) = ...`
                        // collapsed two distinct failure modes into one
                        // "keep require() as-is" fall-through:
                        //   1. The file doesn't exist — fine, downstream
                        //      diagnostics handle it.
                        //   2. The file EXISTS (resolve already stat'd
                        //      it) but read_to_string failed (perms,
                        //      transient EIO). Emitting the raw
                        //      `require('./foo')` into a browser bundle
                        //      yields `require is not defined` or
                        //      `Cannot find module` at runtime — long
                        //      after the dev server should have called
                        //      this out.
                        // Now: split via explicit match. NotFound stays
                        // silent (resolve_cjs_require already filtered
                        // it; this is defensive). Other I/O errors
                        // warn AND emit an inline `throw new Error(...)`
                        // at the require's position so the bundle fails
                        // loudly with the right context, matching what
                        // esbuild/rollup do for resolution diagnostics.
                        match std::fs::read_to_string(&resolved_path) {
                            Ok(content) => {
                                let parent = resolved_path.parent().unwrap_or(base_dir);
                                flatten_cjs(&content, parent, output, visited);
                                continue;
                            }
                            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                // resolve_cjs_require already filtered
                                // this case; treat any leakage as the
                                // documented escape hatch (raw require).
                            }
                            Err(err) => {
                                tracing::warn!(
                                    target: "jet::dev_server::flatten_cjs",
                                    require = %req_path,
                                    resolved = %resolved_path.display(),
                                    base = %base_dir.display(),
                                    error = %err,
                                    "GH #3272 failed to inline relative require; \
                                     emitting inline throw so the bundle fails loudly"
                                );
                                let safe = req_path.replace('\\', "\\\\").replace('"', "\\\"");
                                let err_msg =
                                    err.to_string().replace('\\', "\\\\").replace('"', "\\\"");
                                output.push_str(&format!(
                                    "throw new Error(\"jet: failed to inline require('{safe}'): {err_msg} (GH #3272)\");\n"
                                ));
                                continue;
                            }
                        }
                    }
                }
            }
            // Non-relative require or resolution failed — keep as-is
            output.push_str(line);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
}

/// GH #3680 — convert `SystemTime` to epoch-ms with an observable error
/// branch. Replaces the prior `.unwrap()` panic in the file-watcher
/// loop. Happy path returns wall-clock millis. Error branch (clock
/// before UNIX_EPOCH) returns `0` plus a tagged warn message the caller
/// emits via `tracing::warn!` against a static-target macro.
///
/// The panic that this replaces was particularly insidious because it
/// happened inside `tokio::spawn` without an awaited `JoinHandle` — the
/// dev-server process survived, but the file-watcher task died silently,
/// HMR stopped working, and there was no breadcrumb pointing at clock
/// skew. Falling back to ms=0 lets the watcher keep running; HMR cache-
/// busting still works because no-cache headers are independent of the
/// `?t=<ts>` query param.
///
/// Sibling of `safe_e2e_now_ms` (#3669), `safe_trace_now_ms` (#3673),
/// and `safe_session_now_unix` (#3677) — same shape, different module.
fn parse_hmr_client_text_or_warn(text: &str) -> Option<hmr::ClientMessage> {
    match serde_json::from_str::<hmr::ClientMessage>(text) {
        Ok(msg) => Some(msg),
        Err(err) => {
            let preview: String = text.chars().take(200).collect();
            tracing::warn!(
                target: "jet::dev::hmr",
                error = %err,
                frame_preview = %preview,
                "GH #3302 dropping HMR ClientMessage that failed to parse; \
                 a malformed frame or a client/server protocol-version skew \
                 will silently disappear without this breadcrumb"
            );
            None
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn safe_dev_server_now_ms(now: std::time::SystemTime) -> (u64, Option<String>) {
    match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => (dur.as_millis() as u64, None),
        Err(err) => {
            let warn = format_safe_dev_server_now_ms_warn(&err);
            (0, Some(warn))
        }
    }
}

/// GH #3725 — build the warn wording for the ctrl_c-handler-registration
/// failure branch. Extracted so the issue tag, observable symptom, and
/// operator stop-the-server guidance are unit-testable without provoking
/// the actual signal-registration-failure platform case (which is
/// effectively unreachable on Unix in a CI environment).
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_dev_server_ctrl_c_warn(err: &std::io::Error) -> String {
    format!(
        "GH #3725 jet::dev_server: failed to install the Ctrl+C handler \
         ({err}). Previously this site called `.ok()` which swallowed \
         the error — the shutdown future then resolved on the very next \
         poll and the server exited immediately after printing the \
         listening banner, with no Ctrl+C from the user and no \
         breadcrumb. The dev server will keep running; stop it by \
         sending SIGTERM (e.g. `kill <pid>`) or SIGKILL from another \
         terminal. Fix the underlying cause by checking signal limits \
         (`ulimit -i`) and that your runtime permits `sigaction`."
    )
}

async fn wait_for_dev_shutdown_request(root_dir: PathBuf) {
    loop {
        if crate::dev_session::shutdown_requested(&root_dir) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

/// GH #3680 — build the warn wording for the clock-before-epoch branch.
/// Extracted so the issue tag, error visibility, and operator guidance
/// are unit-testable without provoking the actual broken-clock platform
/// case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_safe_dev_server_now_ms_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3680 jet::dev_server: SystemTime::now() reports a wall clock \
         before UNIX_EPOCH ({err}); falling back to ms=0. Previously this \
         site called `.unwrap()` which PANICKED the file-watcher tokio \
         task — the dev-server process survived but the watcher died \
         silently, so HMR cache-busting timestamps stopped updating and \
         file changes appeared to stop triggering rebuilds with no \
         breadcrumb. The fallback ms=0 keeps the watcher alive; HMR \
         still works because no-cache headers are independent of the \
         `?t=<ts>` query. Fix the host clock (NTP / container --rtc / \
         RTC battery) to restore deterministic cache-busting."
    )
}

/// Extract require path from a line like: `module.exports = require('./cjs/react.development.js');`
fn extract_require_path(line: &str) -> Option<String> {
    let idx = line.find("require(")?;
    let rest = &line[idx + 8..];
    let quote = rest.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let end = rest[1..].find(quote)?;
    Some(rest[1..1 + end].to_string())
}

/// Resolve a CJS require path to a file
fn resolve_cjs_require(base_dir: &std::path::Path, req: &str) -> Option<std::path::PathBuf> {
    let path = base_dir.join(req);

    // Try exact path
    if path.exists() && path.is_file() {
        return Some(path);
    }
    // Try with .js extension
    let with_js = path.with_extension("js");
    if with_js.exists() {
        return Some(with_js);
    }
    // Try index.js
    let index = path.join("index.js");
    if index.exists() {
        return Some(index);
    }
    None
}

/// Extract named re-exports from CJS code by finding common patterns like
/// `exports.Component = ...` or `Object.defineProperty(exports, 'name', ...)`
fn extract_named_reexports(code: &str) -> String {
    let mut names = std::collections::BTreeSet::new();

    for line in code.lines() {
        let trimmed = line.trim();
        // Pattern: exports.Name = ...
        if let Some(rest) = trimmed.strip_prefix("exports.") {
            if let Some(eq_pos) = rest.find(" =") {
                let name = &rest[..eq_pos];
                if name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                    && !name.is_empty()
                    && name != "__esModule"
                    && name != "default"
                {
                    names.insert(name.to_string());
                }
            }
        }
    }

    if names.is_empty() {
        return String::new();
    }

    // Re-export named members from module.exports
    // Use _$name to avoid re-declaration conflicts with CJS source vars
    let mut out = String::from("var __cjs_ns = module.exports;\n");
    for n in &names {
        out.push_str(&format!(
            "var __{n} = __cjs_ns.{n}; export {{ __{n} as {n} }};\n"
        ));
    }
    out
}

/// Generate an importmap <script> tag by scanning node_modules for installed packages.
/// This tells the browser how to resolve bare specifiers like `import React from 'react'`.
#[allow(dead_code)]
fn generate_importmap(root_dir: &PathBuf) -> String {
    let node_modules = root_dir.join("node_modules");
    if !node_modules.exists() {
        return String::new();
    }

    let mut imports = std::collections::BTreeMap::new();

    let jet_dir = node_modules.join(".jet");

    // Scan top-level packages (including scoped @org/pkg)
    let scan_pkg = |pkg_dir: &std::path::Path,
                    pkg_name: &str,
                    imports: &mut std::collections::BTreeMap<String, String>| {
        // Check .jet/ pre-bundled first
        let jet_cached = jet_dir.join(format!("{}.mjs", pkg_name.replace('/', "__")));
        if jet_cached.exists() {
            imports.insert(
                pkg_name.to_string(),
                format!("/node_modules/.jet/{}.mjs", pkg_name.replace('/', "__")),
            );
            return;
        }

        let pkg_json = pkg_dir.join("package.json");
        if !pkg_json.exists() {
            return;
        }
        let Ok(content) = std::fs::read_to_string(&pkg_json) else {
            return;
        };
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
            return;
        };

        // Resolve main entry: exports["."]["browser"] > exports["."]["import"] > module > main > index.js
        // Prefer browser condition for dev server (browser environment)
        let entry = parsed
            .get("exports")
            .and_then(|e| e.get("."))
            .and_then(|dot| {
                // exports["."] can be string or { browser, import, default, ... }
                dot.as_str()
                    .map(String::from)
                    .or_else(|| {
                        // Try browser condition first (may be nested: { browser: { default: "..." } })
                        dot.get("browser").and_then(|b| {
                            b.as_str()
                                .map(String::from)
                                .or_else(|| {
                                    b.get("default").and_then(|v| v.as_str()).map(String::from)
                                })
                                .or_else(|| {
                                    b.get("require").and_then(|v| v.as_str()).map(String::from)
                                })
                        })
                    })
                    .or_else(|| dot.get("import").and_then(|v| v.as_str()).map(String::from))
                    .or_else(|| {
                        dot.get("default")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
            })
            .or_else(|| {
                parsed
                    .get("module")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .or_else(|| {
                parsed
                    .get("main")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| "index.js".to_string());

        let entry = entry.trim_start_matches("./");
        imports.insert(
            pkg_name.to_string(),
            format!("/node_modules/{}/{}", pkg_name, entry),
        );

        // Also map subpath exports like 'react/jsx-runtime'
        if let Some(exports) = parsed.get("exports").and_then(|e| e.as_object()) {
            for (key, val) in exports {
                if key == "." || key == "./package.json" {
                    continue;
                }
                let subpath = key.trim_start_matches("./");
                let target = val
                    .as_str()
                    .map(String::from)
                    .or_else(|| prebundle::resolve_exports_entry(val));
                if let Some(target) = target {
                    let target = target.trim_start_matches("./");
                    imports.insert(
                        format!("{}/{}", pkg_name, subpath),
                        format!("/node_modules/{}/{}", pkg_name, target),
                    );
                }
            }
        }
    };

    // GH #3294 — the prior implementation silently swallowed three classes
    // of IO failure on the discovery walk: [1] `if let Ok(entries) = read_dir(node_modules)`
    // dropping an unreadable `node_modules/` (browser then sees an EMPTY importmap and
    // 404s every bare specifier); [2] `entries.flatten()` dropping per-dirent IO; [3]
    // `if let Ok(scoped) = read_dir(@org/)` dropping every scoped package under an
    // unreadable scope. Each silent path produced a truncated importmap with zero
    // breadcrumb. NotFound on `node_modules` is already guarded by the `.exists()`
    // check at the top of this function, so every remaining read failure is unexpected
    // and must surface.
    match std::fs::read_dir(&node_modules) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::dev::prebundle",
                            path = %node_modules.display(),
                            error = %err,
                            "GH #3294 unreadable dirent during importmap node_modules scan; \
                             a package may be silently omitted from the importmap"
                        );
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if name.starts_with('@') {
                    // Scoped packages
                    let scope_dir = entry.path();
                    match std::fs::read_dir(&scope_dir) {
                        Ok(scoped) => {
                            for sub in scoped {
                                let sub = match sub {
                                    Ok(s) => s,
                                    Err(err) => {
                                        tracing::warn!(
                                            target: "jet::dev::prebundle",
                                            path = %scope_dir.display(),
                                            scope = name,
                                            error = %err,
                                            "GH #3294 unreadable dirent during importmap scoped-scope scan; \
                                             a scoped package may be silently omitted from the importmap"
                                        );
                                        continue;
                                    }
                                };
                                let sub_name =
                                    format!("{}/{}", name, sub.file_name().to_string_lossy());
                                scan_pkg(&sub.path(), &sub_name, &mut imports);
                            }
                        }
                        Err(err) => {
                            tracing::warn!(
                                target: "jet::dev::prebundle",
                                path = %scope_dir.display(),
                                scope = name,
                                error = %err,
                                "GH #3294 unreadable scope dir during importmap scan; \
                                 every package under `{name}` will be omitted from the importmap"
                            );
                        }
                    }
                } else {
                    scan_pkg(&entry.path(), &name, &mut imports);
                }
            }
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::dev::prebundle",
                path = %node_modules.display(),
                error = %err,
                "GH #3294 unreadable node_modules during importmap scan; \
                 the browser will see an EMPTY bare-specifier importmap"
            );
        }
    }

    // Add pre-bundled subpath entries from .jet/.
    //
    // GH #3294 — the same silent-swallow pair (`if let Ok(read_dir)` + `entries.flatten()`)
    // truncated the prebundle entries on EACCES / EIO. NotFound is already
    // guarded by the `jet_dir.exists()` check, so remaining errors must surface.
    if jet_dir.exists() {
        match std::fs::read_dir(&jet_dir) {
            Ok(jet_entries) => {
                for entry in jet_entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".mjs") {
                        let bare_name = fname.trim_end_matches(".mjs").replace("__", "/");
                        if !imports.contains_key(&bare_name) {
                            imports.insert(bare_name, format!("/node_modules/.jet/{}", fname));
                        }
                    }
                }
            }
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev::prebundle",
                    path = %jet_dir.display(),
                    error = %err,
                    "GH #3294 unreadable .jet/ during importmap scan; \
                     every pre-bundled subpath entry will be omitted from the importmap"
                );
            }
        }
    }

    if imports.is_empty() {
        return String::new();
    }

    let entries: Vec<String> = imports
        .iter()
        .map(|(k, v)| format!("      \"{}\": \"{}\"", k, v))
        .collect();

    format!(
        "<script type=\"importmap\">\n{{\n  \"imports\": {{\n{}\n  }}\n}}\n</script>",
        entries.join(",\n")
    )
}

/// Rewrite bare import/export specifiers in transformed JS code to URL paths.
///
/// Bare specifiers like `"react"` or `"react/jsx-runtime"` cannot be resolved
/// by the browser natively. This function rewrites them to URL paths that the
/// dev server can serve:
///
/// 1. If a pre-bundled file exists at `node_modules/.jet/<safe_name>.mjs`, rewrite
///    to `/node_modules/.jet/<safe_name>.mjs`
/// 2. Otherwise, resolve via `node_modules/<package>/package.json` and rewrite
///    to `/node_modules/<package>/<entry>`
///
/// Specifiers starting with `/`, `./`, `../`, `http://`, or `https://` are
/// skipped (already resolvable by the browser).
fn rewrite_bare_specifiers(code: &str, root_dir: &std::path::Path) -> String {
    // Match import/export from clauses with both single and double quotes.
    // We use two separate patterns to avoid backreferences (unsupported by regex crate).
    //
    // Pattern covers:
    //   import ... from 'specifier'  /  import ... from "specifier"
    //   export ... from 'specifier'  /  export ... from "specifier"
    //   import 'specifier'           /  import "specifier"   (side-effect imports)
    let re_single = regex::Regex::new(
        r#"(?m)((?:import|export)\s+(?:(?:\{[^}]*\}|\*\s+as\s+\w+|[\w$]+)(?:\s*,\s*(?:\{[^}]*\}|\*\s+as\s+\w+|[\w$]+))*\s+from\s+|))'([^']+)'"#
    ).unwrap();
    let re_double = regex::Regex::new(
        r#"(?m)((?:import|export)\s+(?:(?:\{[^}]*\}|\*\s+as\s+\w+|[\w$]+)(?:\s*,\s*(?:\{[^}]*\}|\*\s+as\s+\w+|[\w$]+))*\s+from\s+|))"([^"]+)""#
    ).unwrap();

    let node_modules = root_dir.join("node_modules");
    let jet_dir = node_modules.join(".jet");

    // Rewrite single-quoted specifiers
    let code = re_single
        .replace_all(code, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let specifier = &caps[2];
            rewrite_single_specifier(prefix, specifier, "'", &jet_dir, &node_modules)
        })
        .to_string();

    // Rewrite double-quoted specifiers
    re_double
        .replace_all(&code, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let specifier = &caps[2];
            rewrite_single_specifier(prefix, specifier, "\"", &jet_dir, &node_modules)
        })
        .to_string()
}

/// Rewrite a single import specifier if it is a bare specifier.
fn rewrite_single_specifier(
    prefix: &str,
    specifier: &str,
    quote: &str,
    jet_dir: &std::path::Path,
    node_modules: &std::path::Path,
) -> String {
    // Skip non-bare specifiers (relative, absolute, URLs)
    if specifier.starts_with('/')
        || specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier.starts_with("http://")
        || specifier.starts_with("https://")
        || specifier.starts_with("data:")
    {
        return format!("{}{}{}{}", prefix, quote, specifier, quote);
    }

    if let Some((_, target)) = importmap::mui_emotion_patches()
        .iter()
        .find(|(patched, _)| *patched == specifier)
    {
        return format!("{}{}{}{}", prefix, quote, target, quote);
    }

    // Prefer package exports that resolve to browser-loadable ESM files. Some
    // packages publish ESM subpath exports while stale .jet cache entries from
    // older runs may still exist.
    let resolved = resolve_bare_specifier_to_url(specifier, node_modules);
    if let Some(resolved) = resolved.as_deref() {
        if resolved.ends_with(".mjs") {
            return format!("{}{}/node_modules/{}{}", prefix, quote, resolved, quote);
        }
    }

    // Check pre-bundled .jet/ file
    let jet_filename = bare_specifier_to_jet_filename(specifier);
    let jet_path = jet_dir.join(&jet_filename);
    if jet_path.exists() {
        return format!(
            "{}{}/node_modules/.jet/{}{}",
            prefix, quote, jet_filename, quote
        );
    }

    // Resolve via node_modules package.json
    if let Some(resolved) = resolved {
        return format!("{}{}/node_modules/{}{}", prefix, quote, resolved, quote);
    }

    // Fallback: leave unchanged (importmap or browser might handle it)
    format!("{}{}{}{}", prefix, quote, specifier, quote)
}

/// Convert a bare specifier to the `.jet/` filename using the same convention
/// as `prebundle::dep_filename`.
///
/// - Scoped: `@tanstack/react-query` -> `@tanstack__react-query.mjs`
/// - Scoped subpath: `@tanstack/react-query/client` -> `@tanstack__react-query_client.mjs`
/// - Non-scoped: `react` -> `react.mjs`
/// - Non-scoped subpath: `react/jsx-runtime` -> `react_jsx-runtime.mjs`
fn bare_specifier_to_jet_filename(specifier: &str) -> String {
    let sanitized = if specifier.starts_with('@') {
        let parts: Vec<&str> = specifier.splitn(2, '/').collect();
        if parts.len() == 2 {
            let scope_and_name: Vec<&str> = parts[1].splitn(2, '/').collect();
            if scope_and_name.len() == 2 {
                format!("{}__{}_{}", parts[0], scope_and_name[0], scope_and_name[1])
            } else {
                format!("{}__{}", parts[0], parts[1])
            }
        } else {
            specifier.to_string()
        }
    } else {
        specifier.replace('/', "_")
    };
    format!("{}.mjs", sanitized)
}

/// Resolve a bare specifier to a URL path relative to `node_modules/`.
///
/// Splits the specifier into package name + optional subpath, reads the
/// package's `package.json`, and resolves via exports/module/main fields.
fn resolve_bare_specifier_to_url(
    specifier: &str,
    node_modules: &std::path::Path,
) -> Option<String> {
    // Split into package name and subpath
    let (pkg_name, subpath) = if specifier.starts_with('@') {
        // Scoped: @scope/name or @scope/name/subpath
        let parts: Vec<&str> = specifier.splitn(3, '/').collect();
        if parts.len() >= 3 {
            (format!("{}/{}", parts[0], parts[1]), Some(parts[2]))
        } else if parts.len() == 2 {
            (format!("{}/{}", parts[0], parts[1]), None)
        } else {
            return None;
        }
    } else {
        // Non-scoped: react or react/jsx-runtime
        let parts: Vec<&str> = specifier.splitn(2, '/').collect();
        if parts.len() == 2 {
            (parts[0].to_string(), Some(parts[1]))
        } else {
            (specifier.to_string(), None)
        }
    };

    let pkg_dir = node_modules.join(&pkg_name);
    let pkg_json_path = pkg_dir.join("package.json");
    // GH #3185 — the prior `.ok()?` chain silently swallowed read and
    // parse errors here, so a malformed package.json (trailing comma,
    // broken quote) or unreadable file made every `import 'pkg'`
    // resolve fall through to None with no diagnostic. The browser
    // then errored on "Failed to resolve module specifier <pkg>" with
    // no breadcrumb. NotFound stays silent (package not installed is
    // normal); other IO and parse failures emit tracing::warn!.
    let content = match std::fs::read_to_string(&pkg_json_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            tracing::warn!(
                target: "jet::dev_server::resolve",
                "package {} has unreadable package.json at {:?}: {e}; bare specifier \
                 will NOT be resolved (GH #3185)",
                pkg_name,
                pkg_json_path
            );
            return None;
        }
    };
    let pkg: serde_json::Value = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                target: "jet::dev_server::resolve",
                "package {} has malformed package.json at {:?}: {e}; bare specifier \
                 will NOT be resolved (GH #3185)",
                pkg_name,
                pkg_json_path
            );
            return None;
        }
    };

    if let Some(subpath) = subpath {
        // Resolve subpath export from exports map
        let exports_key = format!("./{}", subpath);
        if let Some(target) = pkg
            .get("exports")
            .and_then(|e| e.as_object())
            .and_then(|exports| exports.get(&exports_key))
        {
            if let Some(resolved) = prebundle::resolve_exports_entry(target) {
                let resolved = resolved.trim_start_matches("./");
                return Some(format!("{}/{}", pkg_name, resolved));
            }
        }
        if let Some((pattern, target)) =
            pkg.get("exports")
                .and_then(|e| e.as_object())
                .and_then(|exports| {
                    exports.iter().find_map(|(key, value)| {
                        key.strip_suffix('*')
                            .and_then(|prefix| exports_key.strip_prefix(prefix))
                            .map(|matched| (matched, value))
                    })
                })
        {
            if let Some(resolved) = prebundle::resolve_exports_entry(target) {
                let resolved = resolved.trim_start_matches("./").replace('*', pattern);
                return Some(format!("{}/{}", pkg_name, resolved));
            }
        }
        // Direct file path: react/jsx-runtime → node_modules/react/jsx-runtime(.js)
        let direct = pkg_dir.join(subpath);
        if direct.is_file() {
            return Some(format!("{}/{}", pkg_name, subpath));
        }
        // Try with extensions
        for ext in &["mjs", "js", "cjs"] {
            let candidate = pkg_dir.join(format!("{}.{}", subpath, ext));
            if candidate.exists() {
                return Some(format!("{}/{}.{}", pkg_name, subpath, ext));
            }
        }
        // Try index files
        for ext in &["js", "mjs"] {
            let candidate = pkg_dir.join(subpath).join(format!("index.{}", ext));
            if candidate.exists() {
                return Some(format!("{}/{}/index.{}", pkg_name, subpath, ext));
            }
        }
        None
    } else {
        // Resolve main entry: exports > module > main > index.js
        let entry = pkg
            .get("exports")
            .and_then(prebundle::resolve_exports_entry)
            .or_else(|| pkg.get("module").and_then(|v| v.as_str()).map(String::from))
            .or_else(|| pkg.get("main").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_else(|| "index.js".to_string());

        let entry = entry.trim_start_matches("./");
        Some(format!("{}/{}", pkg_name, entry))
    }
}

/// GH #3241 — read and parse a `package.json` with surfaced failures.
///
/// Returns `Some(value)` on a clean read+parse. Returns `None` silently when
/// the file is missing (legit race-with-deletion / no-package-here case).
/// Returns `None` and emits `tracing::warn!(target: "jet::dev_server",
/// "GH #3241 ...")` on every other IO or JSON parse error so the developer
/// can correlate a confusing browser-side import failure with a broken
/// `package.json`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn read_package_json_value(path: &std::path::Path) -> Option<serde_json::Value> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server",
                path = %path.display(),
                error = %err,
                "GH #3241 failed to read package.json; downstream resolution will be incomplete"
            );
            return None;
        }
    };
    match serde_json::from_str(&content) {
        Ok(v) => Some(v),
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server",
                path = %path.display(),
                error = %err,
                "GH #3241 failed to parse package.json; downstream resolution will be incomplete"
            );
            None
        }
    }
}

/// GH #3592 — build the warn message for a `pkg_root` that does not
/// live under `root_dir` during #subpath-imports rewriting.
/// Extracted so the wording (tag + pkg_root + root_dir + consequence)
/// is unit-testable without provoking a real hoisted-package
/// scenario.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_subpath_imports_rewrite_warn(
    pkg_root: &std::path::Path,
    root_dir: &std::path::Path,
    err: &std::path::StripPrefixError,
) -> String {
    format!(
        "GH #3592 #subpath imports rewrite skipped: package root {} is not under \
         project root {} ({err}); the prior implementation produced an empty \
         pkg_rel and synthesized mis-resolved import URLs (e.g. `from \"#util\"` → \
         `from \"/util\"`). The original `#subpath` specifiers are left unresolved; \
         fix the caller to pass a pkg_root under {}.",
        pkg_root.display(),
        root_dir.display(),
        root_dir.display()
    )
}

/// Walk up from a file path to find the nearest directory containing package.json.
fn find_package_root(file_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = file_path.parent()?;
    loop {
        if dir.join("package.json").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// GH #3234 — resolve Node.js subpath imports (`#foo` → actual path) for one
/// source file by consulting the nearest package.json's `imports` field.
///
/// Previously inlined in two places in this module, each guarded by nested
/// `if let Ok(...)` chains that silently swallowed read and parse failures
/// of the resolved `package.json`. A malformed file produced unresolved
/// `#foo` specifiers and a browser-side "Failed to resolve module specifier"
/// error with no log breadcrumb. Now we warn on non-NotFound IO and on parse
/// errors so the developer can find the offending file.
///
/// Rewrites `source` in place. No-op if the source contains no `#` quotes,
/// if no package.json walks up, or if the package has no `imports` map.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn resolve_node_subpath_imports(
    source: &mut String,
    file_path: &std::path::Path,
    root_dir: &std::path::Path,
) {
    if !source.contains("'#") && !source.contains("\"#") {
        return;
    }
    let Some(pkg_root) = find_package_root(file_path) else {
        return;
    };
    let pkg_json_path = pkg_root.join("package.json");
    let pkg_content = match std::fs::read_to_string(&pkg_json_path) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server",
                path = %pkg_json_path.display(),
                source_file = %file_path.display(),
                error = %err,
                "GH #3234 failed to read package.json for #subpath imports resolution; specifiers left unresolved"
            );
            return;
        }
    };
    let pkg: serde_json::Value = match serde_json::from_str(&pkg_content) {
        Ok(p) => p,
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server",
                path = %pkg_json_path.display(),
                source_file = %file_path.display(),
                error = %err,
                "GH #3234 failed to parse package.json for #subpath imports resolution; specifiers left unresolved"
            );
            return;
        }
    };
    let Some(imports) = pkg.get("imports").and_then(|v| v.as_object()) else {
        return;
    };
    // GH #3592 — the prior `.ok().unwrap_or_default()` silently produced
    // an empty `pkg_rel` when `pkg_root` was not under `root_dir`, then
    // synthesized rewrites like `from "/util"` instead of leaving the
    // import unresolved. That mis-resolved to the project-root URL
    // namespace instead of the actual package layout. Match the
    // strip_prefix explicitly: when it fails, warn and skip the entire
    // rewrite block so the original `#subpath` imports remain in place.
    let pkg_rel = match pkg_root.strip_prefix(root_dir) {
        Ok(p) => format!("/{}", p.display()),
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server",
                "{}",
                format_subpath_imports_rewrite_warn(&pkg_root, root_dir, &err)
            );
            return;
        }
    };
    for (key, val) in imports {
        let resolved = val
            .get("default")
            .or_else(|| val.get("browser"))
            .and_then(|v| v.as_str())
            .or_else(|| val.as_str());
        if let Some(target) = resolved {
            let abs_target = if target.starts_with("./") {
                format!("{}/{}", pkg_rel, &target[2..])
            } else {
                format!("{}/{}", pkg_rel, target)
            };
            *source = source.replace(
                &format!("from '{}'", key),
                &format!("from '{}'", abs_target),
            );
            *source = source.replace(
                &format!("from \"{}\"", key),
                &format!("from \"{}\"", abs_target),
            );
        }
    }
}

fn guess_content_type(path: &PathBuf) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8".to_string(),
        Some("css") => "text/css; charset=utf-8".to_string(),
        Some("js") | Some("mjs") | Some("ts") | Some("tsx") | Some("jsx") => {
            "application/javascript; charset=utf-8".to_string()
        }
        Some("json") => "application/json; charset=utf-8".to_string(),
        Some("png") => "image/png".to_string(),
        Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("svg") => "image/svg+xml".to_string(),
        Some("wasm") => "application/wasm".to_string(),
        Some("woff") => "font/woff".to_string(),
        Some("woff2") => "font/woff2".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

/// Serve the `/@react-refresh` endpoint with the React Fast Refresh runtime shim.
fn serve_react_refresh() -> Response {
    (
        [(
            axum::http::header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        react_refresh::react_refresh_runtime_source(),
    )
        .into_response()
}

async fn request_logger(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();
    let resp = next.run(req).await;
    let status = resp.status().as_u16();
    let elapsed = start.elapsed().as_millis();
    if !uri.path().contains("__jet_hmr") {
        eprintln!("  {} {} {} {}ms", method, uri.path(), status, elapsed);
    }
    resp
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            root_dir: PathBuf::from("."),
            public_dir: Some(PathBuf::from("public")),
            entry: PathBuf::from("src/index.js"),
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        }
    }
}

// ─── CSS rebuild helper ───────────────────────────────────────────────────────

/// Re-run the CSS pipeline and return a `CssUpdate` HMR message if successful.
///
/// `css_entry`     — path to the CSS entry file
/// `root`          — project root (for content scanning)
/// `_content_globs` — Tailwind content patterns (informational; scanner uses config)
/// `_changed_path`  — the file that triggered the rebuild (for logging)
/// `timestamp`     — Unix millisecond timestamp of the rebuild
async fn rebuild_css(
    css_entry: &PathBuf,
    root: &PathBuf,
    _content_globs: &[String],
    _changed_path: &str,
    timestamp: u64,
) -> Option<HmrMessage> {
    // GH #3086 — surface Tailwind config parse errors during HMR rebuilds.
    let config = match TailwindConfig::load(root) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[jet dev] Failed to parse Tailwind config during CSS rebuild: {e:#}");
            eprintln!("[jet dev] Continuing with built-in Tailwind defaults; your tailwind.config.js / [css.tailwind] settings will NOT take effect until the parse error is fixed.");
            TailwindConfig::default()
        }
    };
    let pipeline = CssPipeline::new(root.clone(), config, false /* dev = no minify */);

    match pipeline.process(css_entry) {
        Ok(output) => {
            tracing::info!(
                "CSS rebuilt: {} ({} bytes, hash={})",
                css_entry.display(),
                output.css.len(),
                output.hash
            );
            let filename = css_entry
                .file_stem()
                .map(|s| format!("{}.{}.css", s.to_string_lossy(), output.hash))
                .unwrap_or_else(|| format!("index.{}.css", output.hash));

            Some(HmrMessage::CssUpdate {
                css: output.css,
                filename,
                timestamp,
            })
        }
        Err(e) => {
            tracing::warn!("CSS rebuild failed: {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjs_named_reexports_do_not_duplicate_default() {
        let output = extract_named_reexports(
            r#"
exports.default = styled;
exports.css = css;
exports.__esModule = true;
"#,
        );
        assert!(
            !output.contains(" as default"),
            "default is emitted separately by the CJS wrapper: {}",
            output
        );
        assert!(
            output.contains(" as css"),
            "non-default named exports should still be surfaced: {}",
            output
        );
    }

    #[test]
    fn resolves_wildcard_subpath_exports_to_import_entry() {
        let dir = tempfile::tempdir().unwrap();
        let pkg_dir = dir.path().join("node_modules/@scope/icons");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{
  "exports": {
    "./*": {
      "require": "./*.js",
      "import": "./*.mjs",
      "default": "./*.mjs"
    }
  }
}"#,
        )
        .unwrap();
        std::fs::write(pkg_dir.join("Add.mjs"), "export default 1;").unwrap();
        std::fs::write(pkg_dir.join("Add.js"), "module.exports = 1;").unwrap();

        let resolved =
            resolve_bare_specifier_to_url("@scope/icons/Add", &dir.path().join("node_modules"));

        assert_eq!(resolved.as_deref(), Some("@scope/icons/Add.mjs"));
    }

    /// T33: Line-Based Post-Filter Removed From serve_root_file
    ///
    /// Verifies that serve_root_file() transforms TypeScript via the AST-based
    /// transform_tsx()/transform_typescript() pipeline, NOT a line-based filter.
    /// The test creates a .ts file with TS-only syntax (export type, interface)
    /// and confirms it is correctly stripped by the AST transformer.
    #[tokio::test]
    async fn t33_line_based_post_filter_removed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Write a TypeScript file with TS-only syntax
        let ts_source = r#"
export type Props = { name: string };

interface Config {
    port: number;
    host: string;
}

export const greet = (name: string): string => {
    return "hello " + name;
};
"#;
        std::fs::write(root.join("test.ts"), ts_source).unwrap();

        let config = ServerConfig {
            root_dir: root.to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "test.ts").await;
        assert!(
            response.is_some(),
            "serve_root_file must return a response for .ts files"
        );

        let response = response.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let output = String::from_utf8(body.to_vec()).unwrap();

        // AST-based transform must have stripped TS-only constructs
        assert!(
            !output.contains("export type"),
            "export type must be removed by AST transform: {}",
            output
        );
        assert!(
            !output.contains("interface Config"),
            "interface must be removed by AST transform: {}",
            output
        );
        // But the actual JS code must remain
        assert!(
            output.contains("export const greet"),
            "JS export const must be preserved: {}",
            output
        );
    }

    /// CSS files with @tailwind directives are processed through CssPipeline.
    ///
    /// Verifies that serve_root_file() runs CSS containing @tailwind directives
    /// through the full CSS pipeline (Tailwind JIT + directives + lightningcss)
    /// instead of returning raw @tailwind directives.  The response is a JS
    /// module that injects a <style data-jet-css> tag with the compiled CSS.
    #[tokio::test]
    async fn css_tailwind_directives_processed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Write a CSS file with @tailwind directives
        let css_source = "@tailwind base;\n@tailwind components;\n@tailwind utilities;\n";
        std::fs::write(root.join("index.css"), css_source).unwrap();

        // Write a minimal tailwind.config.js so TailwindConfig::load finds it
        // (not strictly required — TailwindConfig::load falls back to default)

        let config = ServerConfig {
            root_dir: root.to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "index.css").await;
        assert!(
            response.is_some(),
            "serve_root_file must return a response for .css files"
        );

        let response = response.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let output = String::from_utf8(body.to_vec()).unwrap();

        // Response must be JS that injects a <style> tag (not a <link> tag)
        assert!(
            output.contains("data-jet-css"),
            "Response should inject a <style data-jet-css> tag: {}",
            &output[..output.len().min(200)]
        );

        // The @tailwind directives must have been processed — they should NOT
        // appear in the output CSS.
        assert!(
            !output.contains("@tailwind base"),
            "Output must not contain raw @tailwind base directive: {}",
            &output[..output.len().min(500)]
        );
        assert!(
            !output.contains("@tailwind utilities"),
            "Output must not contain raw @tailwind utilities directive: {}",
            &output[..output.len().min(500)]
        );

        // Preflight CSS (from @tailwind base) should be present — it contains
        // box-sizing as a reliable marker.
        assert!(
            output.contains("box-sizing"),
            "Output should contain Preflight CSS (box-sizing) from @tailwind base: {}",
            &output[..output.len().min(500)]
        );
    }

    /// Plain CSS files (without @tailwind) are served inline without pipeline.
    ///
    /// Verifies that serve_root_file() serves plain CSS as a JS module that
    /// injects a <style> tag, without running through CssPipeline.
    #[tokio::test]
    async fn css_plain_served_inline() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Write a plain CSS file (no @tailwind directives)
        let css_source = "body { margin: 0; background: red; }\n";
        std::fs::write(root.join("plain.css"), css_source).unwrap();

        let config = ServerConfig {
            root_dir: root.to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "plain.css").await;
        assert!(
            response.is_some(),
            "serve_root_file must return a response for .css files"
        );

        let response = response.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let output = String::from_utf8(body.to_vec()).unwrap();

        // Response must be JS that injects a <style> tag
        assert!(
            output.contains("data-jet-css"),
            "Response should inject a <style data-jet-css> tag: {}",
            output
        );

        // The plain CSS content should be present in the output
        assert!(
            output.contains("margin") && output.contains("background"),
            "Output should contain the original CSS rules: {}",
            output
        );
    }

    /// GH #3082 — `..` components must be rejected before any disk-rooted
    /// `join`, so a request like `/../<anything>` returns `None` (becomes
    /// 404 at the dispatch layer) instead of escaping the configured dirs.
    #[tokio::test]
    async fn path_traversal_rejected_in_serve_static_file() {
        let dir = tempfile::tempdir().unwrap();
        let public = dir.path().join("public");
        std::fs::create_dir_all(&public).unwrap();
        // Create a sibling file OUTSIDE public_dir that a traversal could try to reach.
        let secret = dir.path().join("secret.txt");
        std::fs::write(&secret, b"TOP-SECRET").unwrap();

        let config = ServerConfig {
            root_dir: dir.path().to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: Some(public.clone()),
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        // The literal `..` component must be rejected — even though the
        // file actually exists, the handler must NOT serve it.
        assert!(
            serve_static_file(&config, "../secret.txt").await.is_none(),
            "serve_static_file must reject `..` traversal"
        );
        // A nested `..` segment must also be rejected.
        assert!(
            serve_static_file(&config, "ok/../../secret.txt")
                .await
                .is_none(),
            "serve_static_file must reject nested `..` traversal"
        );
    }

    /// GH #3082 — same protection applies to `serve_root_file`.
    #[tokio::test]
    async fn path_traversal_rejected_in_serve_root_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        std::fs::create_dir_all(&root).unwrap();
        let secret = dir.path().join("secret.txt");
        std::fs::write(&secret, b"TOP-SECRET").unwrap();

        let config = ServerConfig {
            root_dir: root.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        assert!(
            serve_root_file(&config, "../secret.txt").await.is_none(),
            "serve_root_file must reject `..` traversal"
        );
    }

    /// Happy path: legitimate static asset under `public_dir` is still served.
    #[tokio::test]
    async fn legit_static_asset_still_served() {
        let dir = tempfile::tempdir().unwrap();
        let public = dir.path().join("public");
        std::fs::create_dir_all(public.join("img")).unwrap();
        std::fs::write(public.join("img/logo.png"), b"PNG").unwrap();

        let config = ServerConfig {
            root_dir: dir.path().to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: Some(public),
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_static_file(&config, "img/logo.png").await;
        assert!(
            response.is_some(),
            "legit static asset must still be served"
        );
    }

    /// GH #3103 — when `index.html` exists but cannot be read, the dev
    /// server must surface the I/O error (instead of silently serving
    /// the built-in default) so the author can tell their custom
    /// `index.html` is being shadowed.
    ///
    /// We exercise the helper directly: the "file missing" path must
    /// return the default template, and the "file present + readable"
    /// path must return the file body unchanged. The third regime
    /// (file present but unreadable) is exercised on Unix-likes by
    /// chmodding the file so reading returns an `EACCES` — we only
    /// assert that the helper does not panic and returns *some*
    /// fallback body. The diagnostic itself is best-effort stderr.
    #[test]
    fn load_index_html_returns_default_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("index.html");
        let html = load_index_html_or_default(&missing);
        assert!(html.contains("<!DOCTYPE html>"));
        assert_eq!(html, default_index_html());
    }

    #[test]
    fn load_index_html_returns_file_when_readable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.html");
        std::fs::write(&path, "<!doctype html><title>custom</title>").unwrap();
        let html = load_index_html_or_default(&path);
        assert_eq!(html, "<!doctype html><title>custom</title>");
    }

    #[cfg(unix)]
    #[test]
    fn load_index_html_falls_back_when_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.html");
        std::fs::write(&path, "should never be served").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // If chmod did not actually block reads (e.g. running as root
        // in a container, where mode bits are ignored), skip — the
        // unreadable regime is unreachable in this environment.
        if std::fs::read_to_string(&path).is_ok() {
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let html = load_index_html_or_default(&path);

        // Restore perms so tempdir cleanup can remove the file.
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));

        // Read failed → fell back to default; the user's bytes must NOT leak.
        assert!(!html.contains("should never be served"));
        assert_eq!(html, default_index_html());
    }

    /// GH #3140 — when a static asset exists under `public_dir` but
    /// cannot be read (e.g. permissions), `serve_static_file` must
    /// surface a 5xx diagnostic instead of returning `None` and
    /// letting the SPA `serve_index_html` fallback override with the
    /// HTML shell. Otherwise the dev sees their static asset replaced
    /// by HTML with status 200 and has no breadcrumb.
    #[cfg(unix)]
    #[tokio::test]
    async fn serve_static_file_surfaces_io_error_instead_of_falling_through() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let public = dir.path().join("public");
        std::fs::create_dir_all(&public).unwrap();
        let asset = public.join("guarded.bin");
        std::fs::write(&asset, b"PAYLOAD").unwrap();
        std::fs::set_permissions(&asset, std::fs::Permissions::from_mode(0o000)).unwrap();

        // If chmod is ignored (root in container), skip cleanly.
        if std::fs::read(&asset).is_ok() {
            let _ = std::fs::set_permissions(&asset, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let config = ServerConfig {
            root_dir: dir.path().to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: Some(public),
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_static_file(&config, "guarded.bin").await;

        // Restore perms so tempdir cleanup works.
        let _ = std::fs::set_permissions(&asset, std::fs::Permissions::from_mode(0o644));

        let response = response.expect(
            "serve_static_file must NOT return None on IO error for an existing file; \
             that would silently fall through to the SPA shell (GH #3140)",
        );
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "unreadable existing asset must surface as 500, not as SPA fallback"
        );

        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(
            body.contains("GH #3140"),
            "error body must include searchable issue tag, got: {body}"
        );
        assert!(
            body.contains("guarded.bin"),
            "error body must name the failing asset, got: {body}"
        );
    }

    /// GH #3143 — when a `.css` module under `root_dir` exists but
    /// cannot be read, `serve_root_file` must return a 5xx with the
    /// path + issue tag, NOT fall through to the SPA shell (which the
    /// browser would then refuse with "wrong MIME type" and no
    /// breadcrumb back to the IO error).
    #[cfg(unix)]
    #[tokio::test]
    async fn serve_root_file_surfaces_io_error_for_unreadable_css() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let css = root.join("locked.css");
        std::fs::write(&css, b".a { color: red; }").unwrap();
        std::fs::set_permissions(&css, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read_to_string(&css).is_ok() {
            let _ = std::fs::set_permissions(&css, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let config = ServerConfig {
            root_dir: root.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "locked.css").await;

        let _ = std::fs::set_permissions(&css, std::fs::Permissions::from_mode(0o644));

        let response = response.expect(
            "serve_root_file must surface IO error rather than fall through to SPA (GH #3143)",
        );
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("GH #3143"), "missing issue tag: {body}");
        assert!(body.contains("locked.css"), "missing path: {body}");
    }

    /// GH #3143 — same protection for the `node_modules/*.js` branch.
    #[cfg(unix)]
    #[tokio::test]
    async fn serve_root_file_surfaces_io_error_for_unreadable_node_modules_js() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let pkg = root.join("node_modules/locked-pkg");
        std::fs::create_dir_all(&pkg).unwrap();
        let js = pkg.join("index.js");
        std::fs::write(&js, b"module.exports = {};").unwrap();
        std::fs::set_permissions(&js, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read_to_string(&js).is_ok() {
            let _ = std::fs::set_permissions(&js, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let config = ServerConfig {
            root_dir: root.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "node_modules/locked-pkg/index.js").await;

        let _ = std::fs::set_permissions(&js, std::fs::Permissions::from_mode(0o644));

        let response = response.expect(
            "serve_root_file must surface IO error rather than fall through to SPA (GH #3143)",
        );
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("GH #3143"), "missing issue tag: {body}");
        assert!(body.contains("locked-pkg/index.js"), "missing path: {body}");
    }

    /// GH #3146 — TS/TSX/JSX/JS branch must surface IO errors. This is
    /// the hottest path in modern jet usage (every `.tsx` import lands
    /// here), so a silent fallthrough makes the whole app silently fail
    /// to load with no breadcrumb.
    #[cfg(unix)]
    #[tokio::test]
    async fn serve_root_file_surfaces_io_error_for_unreadable_tsx() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let tsx = root.join("App.tsx");
        std::fs::write(&tsx, b"export default () => <div/>;").unwrap();
        std::fs::set_permissions(&tsx, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read_to_string(&tsx).is_ok() {
            let _ = std::fs::set_permissions(&tsx, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let config = ServerConfig {
            root_dir: root.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "App.tsx").await;

        let _ = std::fs::set_permissions(&tsx, std::fs::Permissions::from_mode(0o644));

        let response = response.expect(
            "serve_root_file must surface IO error for TS/TSX, not fall through to SPA (GH #3146)",
        );
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("GH #3146"), "missing issue tag: {body}");
        assert!(body.contains("App.tsx"), "missing path: {body}");
    }

    /// GH #3146 — generic fallthrough branch (non-CSS, non-JS, non-TS
    /// content) must also surface IO errors instead of returning None.
    #[cfg(unix)]
    #[tokio::test]
    async fn serve_root_file_surfaces_io_error_for_unreadable_generic_content() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // Pick an extension that isn't handled by any earlier branch —
        // `.wasm` lands in the generic content fallthrough at the end
        // of `serve_root_file`.
        let bin = root.join("module.wasm");
        std::fs::write(&bin, b"\0asm\x01\x00\x00\x00").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read(&bin).is_ok() {
            let _ = std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let config = ServerConfig {
            root_dir: root.clone(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.ts"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = serve_root_file(&config, "module.wasm").await;

        let _ = std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o644));

        let response = response.expect(
            "serve_root_file must surface IO error for generic content, not fall through to SPA (GH #3146)",
        );
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();
        assert!(body.contains("GH #3146"), "missing issue tag: {body}");
        assert!(body.contains("module.wasm"), "missing path: {body}");
    }

    /// GH #3201 — Happy path: when the root package.json is well-formed
    /// with no dependencies, the function returns cleanly and creates
    /// `node_modules/.jet/`. Pins that the new diagnostics did not
    /// regress the happy path.
    #[test]
    fn pre_bundle_cjs_deps_happy_path_creates_jet_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"t","version":"0.0.0","dependencies":{}}"#,
        )
        .unwrap();

        super::pre_bundle_cjs_deps(&root);

        assert!(
            root.join("node_modules").join(".jet").is_dir(),
            "secondary pre-bundler must mkdir node_modules/.jet on happy path"
        );
    }

    /// GH #3201 — Absent root package.json must return silently (no
    /// panic, no warn, no jet dir created). This is the canonical
    /// "non-npm project" path.
    #[test]
    fn pre_bundle_cjs_deps_silent_when_pkg_json_absent() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // No package.json written.

        // Must not panic.
        super::pre_bundle_cjs_deps(&root);

        assert!(
            !root.join("node_modules").join(".jet").exists(),
            "no jet dir should be created when package.json is absent"
        );
    }

    /// GH #3201 — Malformed root package.json (trailing comma) used to
    /// silently early-return via `let Ok(...) else`. The user saw CJS
    /// imports fail in the browser with no diagnostic. Post-fix:
    /// `tracing::warn!` surfaces it, and the secondary pre-bundler does
    /// not create `node_modules/.jet/` (it had nothing to bundle).
    #[test]
    fn pre_bundle_cjs_deps_skips_on_malformed_pkg_json() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        // Trailing comma — invalid JSON.
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"t","dependencies":{"react":"18",},}"#,
        )
        .unwrap();

        // Must not panic.
        super::pre_bundle_cjs_deps(&root);

        assert!(
            !root.join("node_modules").join(".jet").exists(),
            "malformed package.json must skip secondary pre-bundle without \
             creating node_modules/.jet (GH #3201)"
        );
    }

    /// GH #3244 — Happy path through the per-dep loop: a CJS dependency
    /// with a parseable package.json and an entry source that contains
    /// `module.exports` must produce a `.jet/<dep>.mjs` cache file.
    /// Pins that the de-`let Ok` refactor still bundles real CJS deps.
    #[test]
    fn pre_bundle_cjs_deps_bundles_simple_cjs_dependency() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"t","version":"0.0.0","dependencies":{"leftpad":"1"}}"#,
        )
        .unwrap();
        let leftpad = root.join("node_modules").join("leftpad");
        std::fs::create_dir_all(&leftpad).unwrap();
        std::fs::write(
            leftpad.join("package.json"),
            r#"{"name":"leftpad","version":"1.0.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(
            leftpad.join("index.js"),
            "module.exports = function leftpad(s){ return s; };",
        )
        .unwrap();

        super::pre_bundle_cjs_deps(&root);

        assert!(
            root.join("node_modules")
                .join(".jet")
                .join("leftpad.mjs")
                .is_file(),
            "happy CJS dep must be pre-bundled to .jet/leftpad.mjs (GH #3244)"
        );
    }

    /// GH #3244 — A *per-dep* malformed package.json used to silently
    /// drop that dep via `let Ok(pkg) = serde_json::from_str(...) else
    /// { continue };`. Post-fix: the bad dep is skipped without panic
    /// and any sibling well-formed deps still bundle successfully.
    #[test]
    fn pre_bundle_cjs_deps_skips_malformed_dep_without_dropping_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"t","version":"0.0.0","dependencies":{"bad":"1","good":"1"}}"#,
        )
        .unwrap();
        let bad = root.join("node_modules").join("bad");
        std::fs::create_dir_all(&bad).unwrap();
        // Trailing comma — invalid JSON.
        std::fs::write(bad.join("package.json"), r#"{"name":"bad",}"#).unwrap();

        let good = root.join("node_modules").join("good");
        std::fs::create_dir_all(&good).unwrap();
        std::fs::write(
            good.join("package.json"),
            r#"{"name":"good","version":"1.0.0","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(good.join("index.js"), "module.exports = {};").unwrap();

        super::pre_bundle_cjs_deps(&root);

        let jet_dir = root.join("node_modules").join(".jet");
        assert!(
            !jet_dir.join("bad.mjs").exists(),
            "malformed-pkg dep must be skipped (GH #3244)"
        );
        assert!(
            jet_dir.join("good.mjs").is_file(),
            "well-formed sibling dep must still be bundled (GH #3244)"
        );
    }

    /// GH #3244 — A dep whose `main` entry file is missing must be
    /// skipped silently (mirrors the existing `entry_path.exists()`
    /// guard) without disturbing the surrounding loop.
    #[test]
    fn pre_bundle_cjs_deps_skips_dep_with_missing_entry_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        std::fs::write(
            root.join("package.json"),
            r#"{"name":"t","version":"0.0.0","dependencies":{"ghost":"1"}}"#,
        )
        .unwrap();
        let ghost = root.join("node_modules").join("ghost");
        std::fs::create_dir_all(&ghost).unwrap();
        std::fs::write(
            ghost.join("package.json"),
            r#"{"name":"ghost","version":"1.0.0","main":"missing.js"}"#,
        )
        .unwrap();
        // Note: missing.js intentionally not written.

        super::pre_bundle_cjs_deps(&root);

        let jet_dir = root.join("node_modules").join(".jet");
        // The `.jet/` dir is still created (mkdir succeeded), but no
        // `.mjs` file exists for the ghost dep.
        assert!(
            !jet_dir.join("ghost.mjs").exists(),
            "missing entry file must skip dep without panic (GH #3244)"
        );
    }

    /// GH #3185 — happy path: valid package.json with `main` resolves the
    /// bare specifier. Pins the well-formed case so the new diagnostic
    /// does not regress it.
    #[test]
    fn resolve_bare_specifier_resolves_main_for_valid_pkg_json() {
        let dir = tempfile::tempdir().unwrap();
        let nm = dir.path().join("node_modules");
        let pkg = nm.join("react");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"react","main":"index.js"}"#,
        )
        .unwrap();
        // Provide a stub of the main file so the resolver's `exists` check is not relevant.
        std::fs::write(pkg.join("index.js"), "").unwrap();

        let resolved = super::resolve_bare_specifier_to_url("react", &nm);
        assert_eq!(
            resolved.as_deref(),
            Some("react/index.js"),
            "valid package.json with `main` must resolve"
        );
    }

    /// GH #3185 — A bare specifier whose package is not installed (no
    /// `node_modules/<pkg>` dir) returns None silently. Pins legitimate
    /// NotFound branch.
    #[test]
    fn resolve_bare_specifier_none_when_pkg_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let nm = dir.path().join("node_modules");
        std::fs::create_dir_all(&nm).unwrap();

        let resolved = super::resolve_bare_specifier_to_url("ghost-pkg", &nm);
        assert_eq!(resolved, None, "missing package must return None silently");
    }

    #[test]
    fn resolve_bare_specifier_subpath_directory_uses_index_file() {
        let dir = tempfile::tempdir().unwrap();
        let nm = dir.path().join("node_modules");
        let pkg = nm.join("@mui/material");
        let button = pkg.join("Button");
        std::fs::create_dir_all(&button).unwrap();
        std::fs::write(pkg.join("package.json"), r#"{"name":"@mui/material"}"#).unwrap();
        std::fs::write(button.join("index.js"), "export default Button;").unwrap();

        let resolved = super::resolve_bare_specifier_to_url("@mui/material/Button", &nm);
        assert_eq!(
            resolved.as_deref(),
            Some("@mui/material/Button/index.js"),
            "subpath directories must resolve to browser-loadable index files"
        );
    }

    #[test]
    fn rewrite_bare_specifiers_uses_mui_patch_table_for_component_subpaths() {
        let dir = tempfile::tempdir().unwrap();
        let code = r#"import Button from "@mui/material/Button";"#;

        let rewritten = super::rewrite_bare_specifiers(code, dir.path());

        assert!(
            rewritten.contains(
                r#"from "/node_modules/@mui/material/Button/index.js""#
            ),
            "MUI component subpaths must rewrite to index.js so their relative imports stay scoped: {rewritten}"
        );
    }

    /// GH #3185 — Malformed package.json (trailing comma — the canonical
    /// hand-edit mistake) used to silently return None from `.ok()?`; the
    /// browser then errored on "Failed to resolve module specifier <pkg>"
    /// with no breadcrumb. Post-fix: still returns None for liveness, but
    /// `tracing::warn!` surfaces the diagnostic so the developer can find
    /// the typo.
    #[test]
    fn resolve_bare_specifier_none_on_malformed_pkg_json() {
        let dir = tempfile::tempdir().unwrap();
        let nm = dir.path().join("node_modules");
        let pkg = nm.join("broken");
        std::fs::create_dir_all(&pkg).unwrap();
        // Trailing comma — invalid JSON.
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"broken","main":"index.js",}"#,
        )
        .unwrap();

        let resolved = super::resolve_bare_specifier_to_url("broken", &nm);
        assert_eq!(
            resolved, None,
            "malformed package.json must return None without panicking"
        );
    }

    /// GH #3215 — happy path: when `.jet/` contains an `.mjs` file that
    /// imports a transitive `.jet/` path which is NOT yet cached but
    /// IS installed in node_modules, the function returns it as a
    /// missing dep.
    #[test]
    fn discover_transitive_cjs_deps_finds_uncached_dep() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        let jet_dir = node_modules.join(".jet");
        std::fs::create_dir_all(&jet_dir).unwrap();

        // The transitive dep is installed in node_modules…
        let dep_pkg = node_modules.join("scheduler");
        std::fs::create_dir_all(&dep_pkg).unwrap();
        std::fs::write(
            dep_pkg.join("package.json"),
            r#"{"name":"scheduler","main":"index.js"}"#,
        )
        .unwrap();

        // …and react-dom's prebundled .mjs imports it via .jet/.
        std::fs::write(
            jet_dir.join("react-dom.mjs"),
            "import __dep_scheduler__ from '/node_modules/.jet/scheduler.mjs';\n",
        )
        .unwrap();

        let missing = super::discover_transitive_cjs_deps(&jet_dir, &node_modules);
        assert_eq!(missing, vec!["scheduler".to_string()]);
    }

    /// GH #3215 — when `.jet/` doesn't exist at all (early dev session,
    /// no prebundling has happened yet) the function returns an empty
    /// vec silently — matches the `!jet_dir.exists()` early-return.
    #[test]
    fn discover_transitive_cjs_deps_missing_jet_dir_returns_empty_silently() {
        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        let jet_dir = node_modules.join(".jet");
        // Intentionally do not create jet_dir.
        std::fs::create_dir_all(&node_modules).unwrap();

        let missing = super::discover_transitive_cjs_deps(&jet_dir, &node_modules);
        assert!(
            missing.is_empty(),
            "missing .jet/ dir must produce empty result, got {:?}",
            missing
        );
    }

    /// GH #3215 — a corrupt/unreadable `.mjs` file in `.jet/` must NOT
    /// block discovery of transitive deps from OTHER readable `.mjs`
    /// files in the same directory. Pre-fix behaviour: the read failure
    /// silently dropped the whole entry; the loop continued but
    /// without visibility. Post-fix: warn is logged and the other
    /// file's deps are still discovered.
    #[cfg(unix)]
    #[test]
    fn discover_transitive_cjs_deps_unreadable_mjs_does_not_drop_siblings() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let node_modules = dir.path().join("node_modules");
        let jet_dir = node_modules.join(".jet");
        std::fs::create_dir_all(&jet_dir).unwrap();

        // Readable sibling that imports `prop-types` transitively.
        let prop_types_pkg = node_modules.join("prop-types");
        std::fs::create_dir_all(&prop_types_pkg).unwrap();
        std::fs::write(
            prop_types_pkg.join("package.json"),
            r#"{"name":"prop-types","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(
            jet_dir.join("readable.mjs"),
            "import __dep_prop_types__ from '/node_modules/.jet/prop-types.mjs';\n",
        )
        .unwrap();

        // Unreadable sibling — would have hidden ALL discovery via the
        // pre-fix `if let Ok(content) = ...` chain returning silently.
        let unreadable = jet_dir.join("unreadable.mjs");
        std::fs::write(
            &unreadable,
            "import __dep_x__ from '/node_modules/.jet/x.mjs';",
        )
        .unwrap();
        std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();

        let missing = super::discover_transitive_cjs_deps(&jet_dir, &node_modules);
        // The readable sibling's dep must still be discovered; the
        // unreadable file's dep is logged but does not block the
        // discovery walk.
        assert!(
            missing.contains(&"prop-types".to_string()),
            "sibling discovery must continue across unreadable file; got {:?}",
            missing
        );

        // Restore so tempdir cleanup works.
        let _ = std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o644));
    }

    // ----- GH #3234 regression tests for resolve_node_subpath_imports -----

    #[test]
    fn resolve_node_subpath_imports_rewrites_hash_specifier() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("node_modules").join("vfile");
        std::fs::create_dir_all(pkg.join("lib")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r##"{"name":"vfile","imports":{"#minpath":{"default":"./lib/minpath.browser.js"}}}"##,
        )
        .unwrap();
        std::fs::write(pkg.join("lib/minpath.browser.js"), "export const x = 1;").unwrap();

        let file_path = pkg.join("lib/uses-subpath.js");
        let mut source = "import { x } from '#minpath';\n".to_string();
        super::resolve_node_subpath_imports(&mut source, &file_path, root);

        assert!(
            source.contains("from '/node_modules/vfile/lib/minpath.browser.js'"),
            "#minpath must be rewritten to absolute path; got: {source}"
        );
    }

    #[test]
    fn resolve_node_subpath_imports_malformed_pkg_leaves_source_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("node_modules").join("brokenvfile");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(pkg.join("package.json"), b"{ not json").unwrap();

        let file_path = pkg.join("lib.js");
        let original = "import { x } from '#anything';\n".to_string();
        let mut source = original.clone();
        super::resolve_node_subpath_imports(&mut source, &file_path, root);

        assert_eq!(
            source, original,
            "malformed package.json must not panic and must leave source unchanged"
        );
    }

    #[test]
    fn resolve_node_subpath_imports_no_imports_field_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("node_modules").join("plainpkg");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"plainpkg","main":"index.js"}"#,
        )
        .unwrap();

        let file_path = pkg.join("index.js");
        let original = "import x from '#nope';\n".to_string();
        let mut source = original.clone();
        super::resolve_node_subpath_imports(&mut source, &file_path, root);

        assert_eq!(source, original, "no imports field must be a clean no-op");
    }

    // ----- GH #3241 regression tests for read_package_json_value -----

    #[test]
    fn read_package_json_value_returns_some_for_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let pj = dir.path().join("package.json");
        std::fs::write(&pj, br#"{"name":"ok","version":"1.0.0"}"#).unwrap();

        let value = super::read_package_json_value(&pj).expect("must parse");
        assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("ok"));
    }

    #[test]
    fn read_package_json_value_returns_none_for_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let pj = dir.path().join("does-not-exist.json");
        assert!(super::read_package_json_value(&pj).is_none());
    }

    #[test]
    fn read_package_json_value_returns_none_for_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        let pj = dir.path().join("package.json");
        std::fs::write(&pj, b"{ not valid json").unwrap();
        // No panic; just None and a warn.
        assert!(super::read_package_json_value(&pj).is_none());
    }

    // GH #3238 — serve_root_file must return the pre-bundled .jet/<safe>.mjs
    // when it exists. Previously, a non-NotFound read failure here was
    // silently swallowed; now it warns. This test confirms the happy path
    // still serves correctly through the explicit-match branch.
    #[tokio::test]
    async fn serve_root_file_returns_jet_cache_for_node_modules_cjs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // The original CJS source — exists so the path is treated as
        // node_modules/<pkg>/<file>.js.
        let pkg_dir = root.join("node_modules").join("legacy-cjs");
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(pkg_dir.join("index.js"), "module.exports = {};").unwrap();

        // The pre-bundled .jet output (what we want served).
        let jet_dir = root.join("node_modules").join(".jet");
        std::fs::create_dir_all(&jet_dir).unwrap();
        let cached_body = "export const fromCache = true;";
        std::fs::write(jet_dir.join("legacy-cjs__index.mjs"), cached_body).unwrap();

        let config = super::ServerConfig {
            root_dir: root.to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.js"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = super::serve_root_file(&config, "node_modules/legacy-cjs/index.js")
            .await
            .expect("serve_root_file must return a response");
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let output = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(
            output, cached_body,
            "must serve the pre-bundled .jet cache rather than the original CJS"
        );
    }

    #[tokio::test]
    async fn serve_root_file_redirects_node_modules_directory_index_to_canonical_js_url() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let input_dir = root.join("node_modules/@mui/material/Input");
        std::fs::create_dir_all(&input_dir).unwrap();
        std::fs::write(input_dir.join("index.js"), "export default Input;").unwrap();

        let config = super::ServerConfig {
            root_dir: root.to_path_buf(),
            host: "127.0.0.1".to_string(),
            port: 0,
            entry: PathBuf::from("index.js"),
            public_dir: None,
            proxy: HashMap::new(),
            aliases: HashMap::new(),
        };

        let response = super::serve_root_file(&config, "node_modules/@mui/material/Input")
            .await
            .expect("serve_root_file must return a redirect");
        assert_eq!(
            response.status(),
            axum::http::StatusCode::TEMPORARY_REDIRECT
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/node_modules/@mui/material/Input/index.js"),
            "directory modules need a canonical URL so their relative imports stay in that directory"
        );
    }

    #[test]
    fn resolve_node_subpath_imports_no_hash_in_source_short_circuits() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Even if pkg.json is malformed, an early short-circuit means we never read it.
        std::fs::write(root.join("package.json"), b"{ not json either").unwrap();

        let mut source = "import x from 'react';\n".to_string();
        super::resolve_node_subpath_imports(&mut source, &root.join("file.js"), root);

        assert_eq!(source, "import x from 'react';\n");
    }

    // ─── GH #3592: pkg_root.strip_prefix(root_dir).ok() empty-fallback ────

    /// GH #3592 — when `pkg_root` is NOT under `root_dir`, the prior
    /// code synthesized rewrites like `from "/util"`. Post-fix the
    /// rewrite is skipped entirely and the original `#subpath`
    /// imports are left in place for downstream resolution.
    #[test]
    fn gh3592_subpath_imports_skipped_when_pkg_root_not_under_root_dir() {
        // Build a real on-disk package under one tempdir...
        let pkg_home = tempfile::tempdir().unwrap();
        let pkg = pkg_home.path().join("vfile");
        std::fs::create_dir_all(pkg.join("lib")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r##"{"name":"vfile","imports":{"#minpath":{"default":"./lib/minpath.browser.js"}}}"##,
        )
        .unwrap();

        // ...and an unrelated `root_dir` that does NOT contain `pkg`.
        let root_dir = tempfile::tempdir().unwrap();

        let file_path = pkg.join("lib/uses-subpath.js");
        let original = "import { x } from '#minpath';\n".to_string();
        let mut source = original.clone();
        super::resolve_node_subpath_imports(&mut source, &file_path, root_dir.path());

        assert_eq!(
            source, original,
            "rewrite must be skipped entirely when pkg_root is not under root_dir; \
             pre-fix this produced `from \"/lib/minpath.browser.js\"`"
        );
        assert!(
            !source.contains("/lib/minpath.browser.js"),
            "pre-fix mis-resolved URL must not appear in source, got: {source}"
        );
    }

    /// GH #3592 — happy-path regression: when `pkg_root` IS under
    /// `root_dir`, the rewrite still produces the absolute path
    /// (covered by `resolve_node_subpath_imports_rewrites_hash_specifier`
    /// but pinning here against accidental shape-change).
    #[test]
    fn gh3592_subpath_imports_happy_path_still_rewrites() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let pkg = root.join("node_modules").join("vfile");
        std::fs::create_dir_all(pkg.join("lib")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r##"{"name":"vfile","imports":{"#minpath":{"default":"./lib/minpath.browser.js"}}}"##,
        )
        .unwrap();

        let file_path = pkg.join("lib/uses-subpath.js");
        let mut source = "import { x } from '#minpath';\n".to_string();
        super::resolve_node_subpath_imports(&mut source, &file_path, root);

        assert!(
            source.contains("from '/node_modules/vfile/lib/minpath.browser.js'"),
            "pkg_root under root_dir must still rewrite to absolute path; got: {source}"
        );
    }

    /// GH #3592 — warn-message wording: must include the tag, the
    /// offending pkg_root, the root_dir, and name the consequence
    /// ("left unresolved" / "skipped" / similar).
    #[test]
    fn gh3592_format_subpath_imports_rewrite_warn_names_tag_paths_consequence() {
        let pkg_root = std::path::Path::new("/Users/chris/elsewhere/pkg");
        let root_dir = std::path::Path::new("/project");
        let err = pkg_root.strip_prefix(root_dir).unwrap_err();
        let msg = super::format_subpath_imports_rewrite_warn(pkg_root, root_dir, &err);

        assert!(
            msg.contains("GH #3592"),
            "must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("/Users/chris/elsewhere/pkg"),
            "must name the offending pkg_root, got: {msg}"
        );
        assert!(
            msg.contains("/project"),
            "must name the root_dir, got: {msg}"
        );
        assert!(
            msg.contains("unresolved") || msg.contains("skipped"),
            "must name the consequence so log readers know the rewrite was abandoned, got: {msg}"
        );
    }

    // ----------------------------------------------------------
    // GH #3272 — flatten_cjs error-handling regression coverage.
    // ----------------------------------------------------------

    /// Happy path: a readable child file is inlined verbatim and
    /// the require() line is dropped. Pin pre-fix behavior wasn't
    /// regressed by the new explicit match.
    #[test]
    fn flatten_cjs_inlines_readable_child_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("child.js"), "var inlined = 1;\n").unwrap();
        let source = "var top = 0;\nrequire('./child.js');\nvar tail = 2;\n";
        let mut out = String::new();
        let mut visited = std::collections::HashSet::new();
        super::flatten_cjs(source, dir.path(), &mut out, &mut visited);
        assert!(out.contains("var inlined = 1;"), "child inlined: {out}");
        assert!(
            !out.contains("require('./child.js')"),
            "raw require leaked: {out}"
        );
    }

    /// Resolution failure path: the require() target doesn't exist;
    /// behaviour stays as documented — keep the raw `require(...)`
    /// so a downstream tool can diagnose.
    #[test]
    fn flatten_cjs_keeps_raw_require_when_resolve_fails() {
        let dir = tempfile::tempdir().unwrap();
        let source = "require('./missing.js');\n";
        let mut out = String::new();
        let mut visited = std::collections::HashSet::new();
        super::flatten_cjs(source, dir.path(), &mut out, &mut visited);
        assert!(
            out.contains("require('./missing.js')"),
            "raw require should be retained when target is absent: {out}"
        );
        assert!(
            !out.contains("throw new Error"),
            "must not emit inline throw for absent files (GH #3272): {out}"
        );
    }

    /// GH #3272 — Resolution succeeds but the file isn't readable
    /// (chmod 0o000 on Unix). Pre-fix: silent fall-through emits a
    /// raw `require('./foo')` into the browser bundle and the
    /// runtime fails with a confusing `require is not defined`.
    /// Post-fix: an inline `throw new Error("jet: failed to inline
    /// require(...): ... (GH #3272)")` lands at the require's
    /// position so the bundle fails loudly with the right context.
    #[cfg(unix)]
    #[test]
    fn flatten_cjs_emits_inline_throw_when_child_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let child_path = dir.path().join("forbidden.js");
        std::fs::write(&child_path, "var x = 1;\n").unwrap();
        std::fs::set_permissions(&child_path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root may bypass perms — restore + skip cleanly in that case.
        if std::fs::read_to_string(&child_path).is_ok() {
            let _ = std::fs::set_permissions(&child_path, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let source = "require('./forbidden.js');\n";
        let mut out = String::new();
        let mut visited = std::collections::HashSet::new();
        super::flatten_cjs(source, dir.path(), &mut out, &mut visited);

        // Restore perms so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&child_path, std::fs::Permissions::from_mode(0o644));

        assert!(
            out.contains("throw new Error"),
            "must emit inline throw (GH #3272), got: {out}"
        );
        assert!(
            out.contains("GH #3272") && out.contains("forbidden.js"),
            "inline throw must name the require path + issue tag, got: {out}"
        );
        // The raw require() must NOT exist as an executable statement.
        // The require path appears inside the throw's error-message
        // string literal — that's the expected diagnostic, not a
        // bundle leak. Assert by checking no line BEGINS with `require(`
        // (after trimming).
        for line in out.lines() {
            let trimmed = line.trim();
            assert!(
                !trimmed.starts_with("require("),
                "raw require must not survive as a statement (would runtime-fail with \
                 `require is not defined`); offending line: {trimmed:?} in: {out}"
            );
        }
    }

    // ----------------------------------------------------------
    // GH #3294 — generate_importmap silent-swallow regression coverage.
    // ----------------------------------------------------------

    /// Happy path: a populated `node_modules/` produces a non-empty
    /// importmap containing the discovered packages. Pins that the
    /// new explicit-match scaffolding didn't regress discovery.
    #[test]
    fn generate_importmap_discovers_top_level_and_scoped_packages() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let nm = root.join("node_modules");
        std::fs::create_dir_all(nm.join("react")).unwrap();
        std::fs::write(
            nm.join("react/package.json"),
            r#"{"name":"react","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(nm.join("@scope/inner")).unwrap();
        std::fs::write(
            nm.join("@scope/inner/package.json"),
            r#"{"name":"@scope/inner","main":"index.js"}"#,
        )
        .unwrap();

        let json = super::generate_importmap(&root);
        assert!(
            json.contains(r#""react""#) && json.contains("/node_modules/react/index.js"),
            "must map react: {json}"
        );
        assert!(
            json.contains(r#""@scope/inner""#)
                && json.contains("/node_modules/@scope/inner/index.js"),
            "must map @scope/inner: {json}"
        );
    }

    /// GH #3294 — when `node_modules/` is unreadable (chmod 0o000)
    /// the prior implementation silently produced an empty importmap
    /// (browser then 404'd every bare specifier). Post-fix: the
    /// function still returns an empty string (the inputs really
    /// did yield nothing), but the warn under
    /// `target = "jet::dev::prebundle"` provides the breadcrumb.
    /// We pin behaviour: no panic, function returns without
    /// crashing.
    #[cfg(unix)]
    #[test]
    fn generate_importmap_handles_unreadable_node_modules_without_panicking() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let nm = root.join("node_modules");
        std::fs::create_dir_all(&nm).unwrap();
        std::fs::write(nm.join(".keep"), b"").unwrap();
        std::fs::set_permissions(&nm, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root bypasses perms — restore and skip cleanly.
        if std::fs::read_dir(&nm).is_ok() {
            let _ = std::fs::set_permissions(&nm, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let result = std::panic::catch_unwind(|| super::generate_importmap(&root));

        // Restore so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&nm, std::fs::Permissions::from_mode(0o755));

        let json = result.expect("generate_importmap must not panic on unreadable node_modules");
        // Best-effort: the importmap is empty (no entries discoverable) — but
        // the function returns cleanly rather than aborting the dev server.
        assert!(
            json.is_empty() || json.contains("imports"),
            "must return without panicking; got: {json:?}"
        );
    }

    /// GH #3294 — an unreadable scoped (`@org/`) dir used to drop
    /// every scoped package via `if let Ok(scoped) = read_dir(...)`.
    /// Post-fix: a sibling top-level package and a healthy scope
    /// neighbour are still discovered.
    #[cfg(unix)]
    #[test]
    fn generate_importmap_unreadable_scope_keeps_siblings() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let nm = root.join("node_modules");
        // Healthy top-level.
        std::fs::create_dir_all(nm.join("lodash")).unwrap();
        std::fs::write(
            nm.join("lodash/package.json"),
            r#"{"name":"lodash","main":"index.js"}"#,
        )
        .unwrap();
        // Healthy scope.
        std::fs::create_dir_all(nm.join("@open/safe")).unwrap();
        std::fs::write(
            nm.join("@open/safe/package.json"),
            r#"{"name":"@open/safe","main":"index.js"}"#,
        )
        .unwrap();
        // Locked scope.
        let locked = nm.join("@locked");
        std::fs::create_dir_all(locked.join("hidden")).unwrap();
        std::fs::write(
            locked.join("hidden/package.json"),
            r#"{"name":"@locked/hidden","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read_dir(&locked).is_ok() {
            let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let json = super::generate_importmap(&root);

        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));

        assert!(
            json.contains(r#""lodash""#),
            "healthy top-level sibling must survive: {json}"
        );
        assert!(
            json.contains(r#""@open/safe""#),
            "healthy scope neighbour must survive: {json}"
        );
        // The locked entry is necessarily omitted; what matters is that
        // siblings did not vanish.
    }

    /// GH #3294 — a malformed `package.json` for one dep used to be
    /// silently swallowed via `let Ok(parsed) = ... else return`.
    /// Post-fix: the malformed entry is still skipped (it has no
    /// useful exports), but a sibling healthy package is still
    /// mapped. The warn under `target = "jet::dev::prebundle"`
    /// provides the breadcrumb.
    #[test]
    fn generate_importmap_malformed_pkg_json_keeps_siblings() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let nm = root.join("node_modules");
        std::fs::create_dir_all(nm.join("ok-pkg")).unwrap();
        std::fs::write(
            nm.join("ok-pkg/package.json"),
            r#"{"name":"ok-pkg","main":"index.js"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(nm.join("broken")).unwrap();
        std::fs::write(nm.join("broken/package.json"), "{this is not json").unwrap();

        let json = super::generate_importmap(&root);
        assert!(
            json.contains(r#""ok-pkg""#),
            "healthy sibling must remain in importmap: {json}"
        );
        // No assertion on the broken pkg — it is correctly omitted; what
        // matters is that the parse failure didn't truncate the rest.
    }

    // GH #3302: HMR WebSocket recv loop must surface malformed ClientMessage
    // frames via tracing::warn! instead of silently dropping them.

    #[test]
    fn hmr_parse_well_formed_console_report_returns_some() {
        let json = r#"{"type":"console-report","level":"error","message":"oops","timestamp":42}"#;
        let parsed = super::parse_hmr_client_text_or_warn(json);
        match parsed {
            Some(hmr::ClientMessage::ConsoleReport { message, .. }) => {
                assert_eq!(message, "oops");
            }
            None => panic!("well-formed ConsoleReport must parse"),
        }
    }

    #[test]
    fn hmr_parse_malformed_json_returns_none() {
        // truncated JSON — the parser must give up but the caller (recv loop)
        // must keep reading subsequent frames; this is asserted by the lack
        // of a panic plus the None return.
        let parsed = super::parse_hmr_client_text_or_warn("{not json");
        assert!(
            parsed.is_none(),
            "malformed JSON must not parse as ClientMessage"
        );
    }

    #[test]
    fn hmr_parse_unknown_variant_returns_none() {
        // valid JSON, but a "type" tag the server doesn't understand
        // (e.g. a future client variant after a protocol upgrade); without
        // the warn breadcrumb the operator has no way to detect skew.
        let json = r#"{"type":"future-variant","payload":{"x":1}}"#;
        let parsed = super::parse_hmr_client_text_or_warn(json);
        assert!(
            parsed.is_none(),
            "unknown ClientMessage variant must not parse"
        );
    }

    #[test]
    fn hmr_parse_missing_required_field_returns_none() {
        // valid JSON, correct tag, but missing the required `timestamp`
        // field — schema drift between client and server.
        let json = r#"{"type":"console-report","level":"warn","message":"hi"}"#;
        let parsed = super::parse_hmr_client_text_or_warn(json);
        assert!(
            parsed.is_none(),
            "ConsoleReport missing required `timestamp` must not parse"
        );
    }
}

#[cfg(test)]
mod gh3680_safe_dev_server_now_ms_tests {
    //! GH #3680 — `dev_server/mod.rs` watcher loop called
    //! `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()`,
    //! which PANICKED the file-watcher tokio task whenever the host
    //! wall clock was before UNIX_EPOCH. Because the watcher ran
    //! inside `tokio::spawn` without an awaited JoinHandle, the panic
    //! killed the watcher silently — HMR appeared to "stop working"
    //! with no breadcrumb. `safe_dev_server_now_ms` replaces the panic
    //! with a graceful fallback + tagged warn.
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn happy_path_returns_millis_and_no_warn() {
        let t = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
        let (ms, warn) = safe_dev_server_now_ms(t);
        assert_eq!(ms, 1_700_000_000_000);
        assert!(warn.is_none(), "happy path must not warn");
    }

    #[test]
    fn epoch_itself_returns_zero_and_no_warn() {
        let (ms, warn) = safe_dev_server_now_ms(UNIX_EPOCH);
        assert_eq!(ms, 0);
        assert!(warn.is_none());
    }

    #[test]
    fn clock_before_epoch_returns_zero_and_warns_instead_of_panicking() {
        // The headline test: this used to panic via `.unwrap()`. It
        // must not panic now — it returns 0 + warn instead.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (ms, warn) = safe_dev_server_now_ms(before);
        assert_eq!(ms, 0, "broken-clock branch must return 0 not panic");
        let msg = warn.expect("broken-clock branch must emit a warn");
        assert!(
            msg.contains("GH #3680"),
            "warn must carry issue tag, got: {msg}"
        );
    }

    #[test]
    fn warn_message_calls_out_the_silent_watcher_death() {
        // The warn must explain WHY this matters — the prior panic was
        // particularly insidious because it killed the watcher task
        // silently. The warn must point at HMR / watcher / cache-
        // busting so the operator can connect a 1970-01-01 ms=0
        // timestamp to the symptom.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_dev_server_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("watcher") || msg.contains("HMR"),
            "warn must name the watcher/HMR symptom, got: {msg}"
        );
        assert!(
            msg.contains("panic") || msg.contains("PANIC"),
            "warn must explain the prior panic behavior, got: {msg}"
        );
    }

    #[test]
    fn warn_message_points_at_the_host_clock_fix_not_jet_code() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_dev_server_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("clock") || msg.contains("NTP") || msg.contains("RTC"),
            "warn must point at host clock as fix surface, got: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_observed_error_text() {
        let err = (UNIX_EPOCH - Duration::from_secs(13))
            .duration_since(UNIX_EPOCH)
            .unwrap_err();
        let msg = format_safe_dev_server_now_ms_warn(&err);
        assert!(msg.contains("GH #3680"));
        assert!(
            msg.contains("13") || msg.contains("seconds") || msg.contains("UNIX_EPOCH"),
            "warn must forward error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_output_is_deterministic_across_calls() {
        let before = UNIX_EPOCH - Duration::from_millis(2024);
        let (_, w1) = safe_dev_server_now_ms(before);
        let (_, w2) = safe_dev_server_now_ms(before);
        assert_eq!(w1, w2);
    }
}

#[cfg(test)]
mod gh3725_dev_server_ctrl_c_warn_tests {
    //! GH #3725 — `Server::start` did `tokio::signal::ctrl_c().await.ok();`,
    //! silently swallowing handler-registration errors. When ctrl_c
    //! returns `Err` immediately, the shutdown future resolved on the
    //! very next poll and the server exited right after printing the
    //! listening banner — no Ctrl+C from the user, no breadcrumb.
    //! `format_dev_server_ctrl_c_warn` documents the bug, names the
    //! observable symptom, and tells the operator how to stop the
    //! server in the failure branch (since the shutdown future now
    //! parks forever via `std::future::pending::<()>().await`).
    use super::*;

    fn sample_err() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "EPERM sample")
    }

    #[test]
    fn helper_tags_gh_issue() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("GH #3725"),
            "warn must carry issue tag, got: {msg}"
        );
    }

    #[test]
    fn helper_round_trips_io_error_text() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("EPERM sample"),
            "warn must forward io::Error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_names_observable_symptom_immediate_exit() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("immediately") || msg.contains("exited immediately"),
            "warn must explain immediate-exit symptom, got: {msg}"
        );
        assert!(
            msg.contains("banner") || msg.contains("listening"),
            "warn must reference the listening banner observable, got: {msg}"
        );
    }

    #[test]
    fn helper_tells_user_how_to_stop_server_when_handler_failed() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("SIGTERM") || msg.contains("SIGKILL") || msg.contains("kill"),
            "warn must name how the operator stops the server, got: {msg}"
        );
    }

    #[test]
    fn helper_points_at_signal_limits_root_cause() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("signal") || msg.contains("sigaction") || msg.contains("ulimit"),
            "warn must point at signal-subsystem root cause, got: {msg}"
        );
    }

    #[test]
    fn helper_names_silent_fallback_root_cause_ok_drop() {
        let msg = format_dev_server_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains(".ok()") || msg.contains("swallowed"),
            "warn must call out the prior `.ok()` swallow, got: {msg}"
        );
    }

    #[test]
    fn helper_is_deterministic_for_fixed_input() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "fixed-msg");
        let a = format_dev_server_ctrl_c_warn(&err);
        let b = format_dev_server_ctrl_c_warn(&err);
        assert_eq!(a, b);
    }

    #[test]
    fn helper_distinct_from_now_ms_warn_3680() {
        // Sibling check: this warn must be distinguishable from
        // `format_safe_dev_server_now_ms_warn` (GH #3680) — both live
        // in dev_server but are unrelated. If they collide on tag or
        // wording, operators can't grep for the right issue.
        let ctrl_c = format_dev_server_ctrl_c_warn(&sample_err());
        let now_err = (std::time::UNIX_EPOCH - std::time::Duration::from_secs(1))
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_err();
        let now_ms = format_safe_dev_server_now_ms_warn(&now_err);
        assert_ne!(ctrl_c, now_ms);
        assert!(
            !ctrl_c.contains("GH #3680"),
            "warn must not carry sibling tag: {ctrl_c}"
        );
        assert!(
            !now_ms.contains("GH #3725"),
            "sibling warn must not carry our tag: {now_ms}"
        );
    }
}

#[cfg(test)]
mod gh3811_serve_file_extension_warn_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn utf8_extension_borrows_silently_for_recognised_js() {
        let cow = coerce_dev_server_serve_file_extension_or_warn(Path::new("a.js"));
        assert_eq!(cow.as_ref(), "js");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn utf8_extension_borrows_silently_for_all_dispatch_arms() {
        for (path, expected) in [
            ("a.js", "js"),
            ("a.cjs", "cjs"),
            ("a.css", "css"),
            ("a.mjs", "mjs"),
            ("a.tsx", "tsx"),
            ("a.ts", "ts"),
        ] {
            let cow = coerce_dev_server_serve_file_extension_or_warn(Path::new(path));
            assert_eq!(cow.as_ref(), expected, "path {path}");
        }
    }

    #[test]
    fn unrecognised_utf8_extension_still_borrows_silently() {
        // Unrecognised UTF-8 extensions (e.g. `.rs`, `.png`) still take the
        // silent borrowed branch — they fall through to SPA shell as before,
        // which is the historical contract for unknown content types.
        let cow = coerce_dev_server_serve_file_extension_or_warn(Path::new("weird.rs"));
        assert_eq!(cow.as_ref(), "rs");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn no_extension_falls_back_to_named_constant() {
        let cow = coerce_dev_server_serve_file_extension_or_warn(Path::new("noext"));
        assert_eq!(cow.as_ref(), DEV_SERVER_SERVE_FILE_NO_EXTENSION_FALLBACK);
        assert_eq!(cow.as_ref(), "");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extension_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"a.\xffweird";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_dev_server_serve_file_extension_or_warn(&path);
        assert!(
            !cow.as_ref().is_empty(),
            "non-UTF-8 must not collapse to empty"
        );
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_extensions_do_not_collide_onto_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let a = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xffone"));
        let b = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xfetwo"));
        let ca = coerce_dev_server_serve_file_extension_or_warn(&a).into_owned();
        let cb = coerce_dev_server_serve_file_extension_or_warn(&b).into_owned();
        assert!(!ca.is_empty() && !cb.is_empty());
        assert_ne!(ca, cb, "distinct non-UTF-8 inputs must remain distinct");
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let _: fn(&Path) -> String = format_dev_server_serve_file_no_extension_warn;
        let _: fn(&Path, &str) -> String = format_dev_server_serve_file_non_utf8_extension_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> =
            coerce_dev_server_serve_file_extension_or_warn;
        assert_eq!(DEV_SERVER_SERVE_FILE_NO_EXTENSION_FALLBACK, "");
    }

    #[test]
    fn each_warn_string_carries_gh3811_tag() {
        let no_ext = format_dev_server_serve_file_no_extension_warn(Path::new("noext"));
        let non_utf8 = format_dev_server_serve_file_non_utf8_extension_warn(
            Path::new("a.bad"),
            "\u{FFFD}weird",
        );
        assert!(no_ext.contains("gh3811"), "no-ext warn lacks tag: {no_ext}");
        assert!(
            non_utf8.contains("gh3811"),
            "non-utf8 warn lacks tag: {non_utf8}"
        );
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let no_ext = format_dev_server_serve_file_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_dev_server_serve_file_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809",
        ] {
            assert!(
                !no_ext.contains(prior),
                "no-ext warn collides with {prior}: {no_ext}"
            );
            assert!(
                !non_utf8.contains(prior),
                "non-utf8 warn collides with {prior}: {non_utf8}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let no_ext = format_dev_server_serve_file_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_dev_server_serve_file_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert_ne!(no_ext, non_utf8);
        assert!(no_ext.contains("no extension"));
        assert!(non_utf8.contains("non-UTF-8"));
    }

    #[test]
    fn happy_path_borrowed_branch_for_zero_alloc_dispatch() {
        let cow = coerce_dev_server_serve_file_extension_or_warn(Path::new("foo.tsx"));
        assert!(
            matches!(cow, std::borrow::Cow::Borrowed("tsx")),
            "recognised extension must take borrowed branch"
        );
    }
}

#[cfg(test)]
mod gh3819_hmr_rebuild_extension_warn_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn utf8_extension_borrows_silently_for_tsx() {
        let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(Path::new("a.tsx"));
        assert_eq!(cow.as_ref(), "tsx");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn utf8_extension_borrows_silently_for_all_transform_arms() {
        for (path, expected) in [("a.tsx", "tsx"), ("a.ts", "ts"), ("a.jsx", "jsx")] {
            let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(Path::new(path));
            assert_eq!(cow.as_ref(), expected, "path {path}");
        }
    }

    #[test]
    fn unrecognised_utf8_extension_borrows_silently() {
        let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(Path::new("weird.png"));
        assert_eq!(cow.as_ref(), "png");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn no_extension_falls_back_to_named_constant() {
        let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(Path::new("noext"));
        assert_eq!(cow.as_ref(), DEV_SERVER_HMR_REBUILD_NO_EXTENSION_FALLBACK);
        assert_eq!(cow.as_ref(), "");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extension_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"a.\xffweird";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(&path);
        assert!(
            !cow.as_ref().is_empty(),
            "non-UTF-8 must not collapse to empty"
        );
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_extensions_do_not_collide_onto_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let a = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xffone"));
        let b = std::path::PathBuf::from(OsStr::from_bytes(b"a.\xfetwo"));
        let ca = coerce_dev_server_hmr_rebuild_extension_or_warn(&a).into_owned();
        let cb = coerce_dev_server_hmr_rebuild_extension_or_warn(&b).into_owned();
        assert_ne!(ca, cb);
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let _: fn(&Path) -> String = format_dev_server_hmr_rebuild_no_extension_warn;
        let _: fn(&Path, &str) -> String = format_dev_server_hmr_rebuild_non_utf8_extension_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> =
            coerce_dev_server_hmr_rebuild_extension_or_warn;
        assert_eq!(DEV_SERVER_HMR_REBUILD_NO_EXTENSION_FALLBACK, "");
    }

    #[test]
    fn each_warn_string_carries_gh3819_tag() {
        let no_ext = format_dev_server_hmr_rebuild_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_dev_server_hmr_rebuild_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert!(no_ext.contains("gh3819"), "no-ext warn lacks tag: {no_ext}");
        assert!(
            non_utf8.contains("gh3819"),
            "non-utf8 warn lacks tag: {non_utf8}"
        );
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families_including_sibling_gh3811() {
        let no_ext = format_dev_server_hmr_rebuild_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_dev_server_hmr_rebuild_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809", "gh3811", "gh3813", "gh3815", "gh3817",
        ] {
            assert!(
                !no_ext.contains(prior),
                "no-ext warn collides with {prior}: {no_ext}"
            );
            assert!(
                !non_utf8.contains(prior),
                "non-utf8 warn collides with {prior}: {non_utf8}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let no_ext = format_dev_server_hmr_rebuild_no_extension_warn(Path::new("noext"));
        let non_utf8 =
            format_dev_server_hmr_rebuild_non_utf8_extension_warn(Path::new("a.bad"), "\u{FFFD}");
        assert_ne!(no_ext, non_utf8);
        assert!(no_ext.contains("no extension"));
        assert!(non_utf8.contains("non-UTF-8"));
    }

    #[test]
    fn happy_path_borrowed_branch_for_zero_alloc_dispatch() {
        let cow = coerce_dev_server_hmr_rebuild_extension_or_warn(Path::new("foo.tsx"));
        assert!(matches!(cow, std::borrow::Cow::Borrowed("tsx")));
    }

    #[test]
    fn warn_distinct_from_serve_file_sibling_when_targeting_hmr() {
        // gh3819 is the HMR-rebuild sibling of gh3811 (serve_file). Their
        // call-site identities should be unmistakable in the warn text so
        // log readers can tell whether the rebuild loop or the serve loop
        // hit the silent fallback.
        let hmr_no_ext = format_dev_server_hmr_rebuild_no_extension_warn(Path::new("noext"));
        let serve_no_ext = format_dev_server_serve_file_no_extension_warn(Path::new("noext"));
        assert_ne!(hmr_no_ext, serve_no_ext);
        assert!(hmr_no_ext.contains("HMR rebuild"));
        assert!(serve_no_ext.contains("serve_file"));
    }
}
// CODEGEN-END

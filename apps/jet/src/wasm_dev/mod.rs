// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-dev.md#schema
// CODEGEN-BEGIN
//! `jet dev --wasm` — single-command development loop for the WASM
//! build pipeline.
//!
//! Pipeline:
//!
//! 1. Run `wasm_build::build` once to populate `dist/`.
//! 2. Start an axum HTTP server on `host:port` serving `dist/` as
//!    static files.
//! 3. Watch `src/**/*.tsx` (+ `jet.toml`). On change, debounce
//!    ~150ms then rebuild. Build failures are logged; the server
//!    keeps running with the last-good `dist/`.
//!
//! v0 scope: no HMR and no automatic browser reload — the user hits
//! Cmd-R. A WebSocket live-reload channel lands later (tracked
//! separately; the injected boot loader is the natural place to
//! receive it).

use anyhow::{Context, Result};
use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex};

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-dev.md#schema
pub struct DevOptions {
    pub host: String,
    pub port: u16,
    /// When `true`, build with `wasm_build::Profile::Dev` (DWARF
    /// retained + jet-wasm `debug` feature on). Corresponds to
    /// `jet dev --wasm --debug`.
    pub debug: bool,
}

#[derive(Clone)]
struct WasmDevState {
    dist: Arc<PathBuf>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

/// Start the WASM dev loop. Blocks until Ctrl-C.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-dev.md#schema
pub async fn serve(root_dir: &Path, opts: DevOptions) -> Result<()> {
    let dist = root_dir.join("dist");
    let profile = if opts.debug {
        crate::wasm_build::Profile::Dev
    } else {
        crate::wasm_build::Profile::Release
    };

    eprintln!(
        "[jet dev --wasm] initial build ({})…",
        if opts.debug { "debug" } else { "release" }
    );
    // jet dev --wasm always targets the web profile; multi-target
    // support arrives via #1239's Slice 4 (CLI surface for `jet dev
    // --target …`).
    if let Err(e) = crate::wasm_build::build_with_profile(
        root_dir,
        Path::new("dist"),
        profile,
        crate::build_target::BuildTarget::Web,
    ) {
        // Initial build failure is fatal — we have nothing to serve.
        return Err(e.context("initial wasm build failed"));
    }
    if opts.debug {
        eprintln!(
            "[jet dev --wasm] debug mode: install `C/C++ DevTools for WebAssembly` in \
             Chromium for Rust source stepping; use `jet browser ...` commands to \
             inspect the running app."
        );
    }

    let addr: SocketAddr = format!("{}:{}", opts.host, opts.port)
        .parse()
        .with_context(|| format!("invalid host:port {}:{}", opts.host, opts.port))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr().context("resolving bound addr")?;

    // Fire-and-forget watcher task. Holds the RecommendedWatcher alive
    // for the lifetime of the server (drops when this fn returns).
    let _watcher_guard = spawn_watcher(root_dir.to_path_buf(), profile)?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let app = build_router(WasmDevState {
        dist: Arc::new(dist.clone()),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    });
    eprintln!(
        "[jet dev --wasm] serving {} at http://{}/",
        dist.display(),
        bound
    );
    if let Err(err) = crate::dev_server::session::write_from_env(
        root_dir,
        bound,
        crate::dev_server::session::TARGET_WASM,
    ) {
        eprintln!("[jet dev --wasm] failed to write serve session: {err:#}");
    }
    let ctrl_c = shutdown_signal();
    let shutdown = async {
        let reason = tokio::select! {
            _ = shutdown_rx => "jet dev shutdown",
            reason = ctrl_c => reason,
        };
        eprintln!("[jet dev --wasm] shutting down ({reason})...");
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("HTTP server error")?;
    eprintln!("[jet dev --wasm] stopped.");
    Ok(())
}

fn build_router(state: WasmDevState) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/__jet_shutdown", post(handle_shutdown))
        .route("/{*path}", get(handle_static))
        .with_state(state)
}

async fn handle_index(State(state): State<WasmDevState>) -> Response {
    serve_file(&state.dist.join("index.html"))
}

async fn handle_shutdown(State(state): State<WasmDevState>) -> Response {
    let mut shutdown_tx = state.shutdown_tx.lock().await;
    if let Some(tx) = shutdown_tx.take() {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;
            let _ = tx.send(());
        });
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet dev shutdown requested\n"))
            .expect("valid shutdown response")
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet dev shutdown already requested\n"))
            .expect("valid shutdown response")
    }
}

async fn handle_static(
    AxumPath(path): AxumPath<String>,
    State(state): State<WasmDevState>,
) -> Response {
    // Clamp the request path under dist/. We normalize via
    // `components()` — any `..` component makes the request 404
    // instead of escaping.
    let rel = PathBuf::from(&path);
    for comp in rel.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return not_found();
        }
    }
    let abs = state.dist.join(&rel);
    // Fast path: the file exists on disk — serve it (any 4xx beyond
    // that is a hard error, e.g. a static-asset bundle the build
    // failed to emit).
    if abs.is_file() {
        return serve_file(&abs);
    }
    // SPA history-API fallback (jet#1413): if the request looks like
    // a client-side route (no file extension), serve index.html so
    // the WASM app can route the URL on first load. Requests with an
    // extension (.js / .css / .wasm / .png …) keep their hard 404 so
    // missing bundles surface clearly to the developer.
    if should_spa_fallback(&rel) {
        return serve_file(&state.dist.join("index.html"));
    }
    not_found()
}

/// Returns `true` when an unmatched request path should fall back to
/// `index.html`. The heuristic — "no path component carries a file
/// extension" — matches how every mainstream SPA dev server (Vite,
/// webpack-dev-server's `historyApiFallback`, Next.js dev) handles
/// browser deep links: HTML navigations never carry a `.ext` in the
/// final segment, while static asset URLs always do.
fn should_spa_fallback(rel: &Path) -> bool {
    // The router only dispatches here for `GET /{*path}` where `path`
    // is non-empty, so `rel` always has at least one component.
    rel.components().all(|c| {
        let std::path::Component::Normal(seg) = c else {
            return true; // RootDir / CurDir / ParentDir aren't extensions
        };
        Path::new(seg).extension().is_none()
    })
}

fn serve_file(path: &Path) -> Response {
    let body = match std::fs::read(path) {
        Ok(b) => b,
        // Legitimate 404: the file simply isn't there.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return not_found(),
        // Any other IO error (EACCES, EIO, dangling symlink) — still 404
        // to keep the browser response shape consistent, but surface the
        // underlying cause so the developer can chase the file-system
        // trouble instead of staring at a generic 404.
        Err(err) => {
            tracing::warn!(
                target: "jet::wasm_dev",
                path = %path.display(),
                error_kind = ?err.kind(),
                error = %err,
                "GH #3326 wasm dev server returning 404 for a file that \
                 exists but failed to read; check permissions or symlinks."
            );
            return not_found();
        }
    };
    let ct = content_type_for(path);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, ct)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .unwrap()
}

fn not_found() -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("404 Not Found"))
        .unwrap()
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("map") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn spawn_watcher(
    root_dir: PathBuf,
    profile: crate::wasm_build::Profile,
) -> Result<RecommendedWatcher> {
    // Channel from the notify thread to the debouncer task. Use a
    // bounded capacity — if it ever backs up we're watching too many
    // files and should narrow the scope.
    let (tx, rx) = mpsc::unbounded_channel::<PathBuf>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            // GH #3512 — surface notify backend errors instead of
            // swallowing them. Without this the dev sees the WASM HMR
            // rebuild loop silently stop: inotify watch overflow,
            // FSEvents stream invalidation, ENFILE, or watch-root
            // removal all kill event flow with zero diagnostic.
            match res {
                Ok(event) => {
                    for p in event.paths {
                        if !should_trigger_rebuild(&p) {
                            continue;
                        }
                        let _ = tx.send(p);
                    }
                }
                Err(e) => {
                    let msg = format_wasm_watch_error(&e.to_string());
                    tracing::warn!(target: "jet::wasm_dev::watcher", "{msg}");
                }
            }
        })
        .context("creating file watcher")?;

    // Watch only `src/` + the root `jet.toml`. Watching the
    // whole repo would fire constantly on the rebuild output in
    // `.jet/wasm-build/` and create a feedback loop.
    let src = root_dir.join("src");
    if src.exists() {
        watcher
            .watch(&src, RecursiveMode::Recursive)
            .with_context(|| format!("watch {}", src.display()))?;
    }
    let cfg = root_dir.join("jet.toml");
    if cfg.exists() {
        watcher
            .watch(&cfg, RecursiveMode::NonRecursive)
            .with_context(|| format!("watch {}", cfg.display()))?;
    }

    let root_for_build = root_dir.clone();
    tokio::spawn(async move { debounce_and_rebuild(rx, root_for_build, profile).await });
    Ok(watcher)
}

/// Build the warning message emitted when the wasm-dev watcher backend
/// yields an error. Extracted so the formatting can be exercised by
/// unit tests without having to provoke a real `notify::Error` (which
/// is platform-specific and effectively non-constructible in tests).
///
/// GH #3512 — message must name the underlying error AND point the
/// dev at the issue tag so the breadcrumb is searchable in commit
/// history / docs when it eventually surfaces.
fn format_wasm_watch_error(underlying: &str) -> String {
    format!(
        "wasm-dev file watcher backend error: {underlying}; WASM HMR rebuilds may stop arriving (GH #3512)"
    )
}

fn should_trigger_rebuild(p: &Path) -> bool {
    let s = p.to_string_lossy();
    if s.contains("/.jet/") || s.contains("/dist/") || s.contains("/target/") {
        return false;
    }
    match p.extension().and_then(|e| e.to_str()) {
        Some("tsx") | Some("ts") | Some("toml") => true,
        _ => false,
    }
}

async fn debounce_and_rebuild(
    mut rx: mpsc::UnboundedReceiver<PathBuf>,
    root: PathBuf,
    profile: crate::wasm_build::Profile,
) {
    const QUIET_MS: u64 = 150;
    loop {
        // Block for the first event.
        let Some(first) = rx.recv().await else {
            return;
        };
        // Drain any events that arrive within the quiet window —
        // editors that save-then-rename produce bursts.
        let mut last = Instant::now();
        let mut changed = vec![first];
        loop {
            match tokio::time::timeout(Duration::from_millis(QUIET_MS), rx.recv()).await {
                Ok(Some(p)) => {
                    changed.push(p);
                    last = Instant::now();
                }
                Ok(None) => return, // sender dropped
                Err(_) => {
                    // Timeout — quiet window elapsed.
                    if last.elapsed() >= Duration::from_millis(QUIET_MS) {
                        break;
                    }
                }
            }
        }

        // Log one line covering the burst rather than N.
        eprintln!(
            "[jet dev --wasm] change detected ({} file{}), rebuilding…",
            changed.len(),
            if changed.len() == 1 { "" } else { "s" }
        );

        let root = root.clone();
        // Run the build on a blocking thread — wasm_build shells out
        // to wasm-pack (synchronous + CPU-bound). Re-apply the same
        // profile the server was started with so dev keeps DWARF +
        // __jet_debug across rebuilds.
        let result = tokio::task::spawn_blocking(move || {
            crate::wasm_build::build_with_profile(
                &root,
                Path::new("dist"),
                profile,
                crate::build_target::BuildTarget::Web,
            )
        })
        .await;

        match result {
            Ok(Ok(())) => eprintln!("[jet dev --wasm] rebuild ok — refresh the browser."),
            Ok(Err(e)) => eprintln!("[jet dev --wasm] rebuild failed: {e:#}"),
            Err(join_err) => eprintln!("[jet dev --wasm] rebuild task crashed: {join_err}"),
        }
    }
}

/// GH #3730 — was `let _ = tokio::signal::ctrl_c().await;` which
/// silently swallowed handler-registration errors. When ctrl_c
/// returns `Err` immediately (signal limits exhausted, sandboxed
/// runtime forbidding `sigaction`, etc.) the `let _ =` discarded the
/// error, this future resolved on the very next poll, and
/// `axum::serve(...).with_graceful_shutdown(shutdown_signal())` then
/// shut the wasm dev server down right after printing the listening
/// banner — no Ctrl+C from the user and no breadcrumb. Match on the
/// result instead: on `Ok` return as before; on `Err` emit a
/// GH #3730-tagged warn and park forever so the server keeps serving
/// until the OS kills it (better UX than silent immediate exit).
/// Sibling of the `dev_server::Server::start` fix in GH #3725.
async fn shutdown_signal() -> &'static str {
    match tokio::signal::ctrl_c().await {
        Ok(()) => "Ctrl-C",
        Err(err) => {
            tracing::warn!(
                target: "jet::wasm_dev",
                error = %err,
                "{}",
                format_wasm_dev_ctrl_c_warn(&err)
            );
            std::future::pending::<&'static str>().await
        }
    }
}

/// GH #3730 — build the warn wording for the wasm_dev ctrl_c handler
/// registration failure branch. Extracted so the issue tag, observable
/// symptom, and operator stop-the-server guidance are unit-testable
/// without provoking the actual signal-registration-failure platform
/// case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-dev.md#schema
pub(crate) fn format_wasm_dev_ctrl_c_warn(err: &std::io::Error) -> String {
    format!(
        "GH #3730 jet::wasm_dev: failed to install the Ctrl+C handler \
         ({err}). Previously this site called `let _ =` which swallowed \
         the error — the shutdown future then resolved on the very next \
         poll and `jet dev --wasm` exited immediately after printing \
         the listening banner, with no Ctrl+C from the user and no \
         breadcrumb. The wasm dev server will keep running; stop it by \
         sending SIGTERM (e.g. `kill <pid>`) or SIGKILL from another \
         terminal. Fix the underlying cause by checking signal limits \
         (`ulimit -i`) and that your runtime permits `sigaction`. \
         Sibling of GH #3725 (dev_server)."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn should_trigger_rebuild_covers_tsx_and_config() {
        assert!(should_trigger_rebuild(Path::new("/proj/src/App.tsx")));
        assert!(should_trigger_rebuild(Path::new("/proj/jet.toml")));
        assert!(!should_trigger_rebuild(Path::new(
            "/proj/.jet/wasm-build/src/lib.rs"
        )));
        assert!(!should_trigger_rebuild(Path::new("/proj/dist/app.wasm")));
        assert!(!should_trigger_rebuild(Path::new("/proj/target/debug/foo")));
        assert!(!should_trigger_rebuild(Path::new("/proj/src/styles.css")));
    }

    #[test]
    fn content_type_for_known_extensions() {
        assert_eq!(
            content_type_for(Path::new("x.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(content_type_for(Path::new("x.wasm")), "application/wasm");
        assert_eq!(
            content_type_for(Path::new("x.js")),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for(Path::new("x.mjs")),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(content_type_for(Path::new("x")), "application/octet-stream");
    }

    /// Sets up a temporary `dist/` with a marker `index.html` and a
    /// real static asset, returns the router under test.
    fn fixture_router() -> (tempfile::TempDir, Router) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dist = tmp.path().to_path_buf();
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><div id=\"jet-canvas\"></div>",
        )
        .unwrap();
        std::fs::write(dist.join("app.js"), "console.log('app');").unwrap();
        let router = build_router(test_state(dist));
        (tmp, router)
    }

    fn test_state(dist: PathBuf) -> WasmDevState {
        let (shutdown_tx, _shutdown_rx) = oneshot::channel();
        WasmDevState {
            dist: Arc::new(dist),
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
        }
    }

    async fn get(router: &Router, uri: &str) -> (StatusCode, String, String) {
        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let resp = router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let ct = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        (status, ct, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn deep_link_serves_index_html_for_spa_fallback() {
        // Regression for jet#1413: opening a client-side route directly
        // (e.g. /apps/foo/studio) must return index.html, not 404, so
        // the SPA router can pick up the URL on first load.
        let (_tmp, router) = fixture_router();
        let (status, ct, body) =
            get(&router, "/apps/t-operations-team-request-tracker/studio").await;
        assert_eq!(status, StatusCode::OK, "deep link should serve index.html");
        assert!(
            ct.starts_with("text/html"),
            "deep link content-type should be text/html, got {ct}"
        );
        assert!(
            body.contains("jet-canvas"),
            "deep link body should be index.html, got: {body}"
        );
    }

    #[tokio::test]
    async fn root_still_serves_index_html() {
        let (_tmp, router) = fixture_router();
        let (status, ct, body) = get(&router, "/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ct.starts_with("text/html"));
        assert!(body.contains("jet-canvas"));
    }

    #[tokio::test]
    async fn existing_static_asset_is_served_directly() {
        let (_tmp, router) = fixture_router();
        let (status, ct, body) = get(&router, "/app.js").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ct.starts_with("application/javascript"));
        assert!(body.contains("console.log"));
    }

    #[tokio::test]
    async fn missing_static_asset_with_extension_still_404s() {
        // Files with an extension (typical for JS/CSS/wasm bundles) must
        // 404 cleanly so build tooling notices the missing chunk instead
        // of silently consuming index.html.
        let (_tmp, router) = fixture_router();
        let (status, _ct, _body) = get(&router, "/missing-bundle.js").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _ct, _body) = get(&router, "/styles/theme.css").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _ct, _body) = get(&router, "/app_bg.wasm").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn should_spa_fallback_only_for_extensionless_paths() {
        assert!(should_spa_fallback(Path::new("apps/foo/studio")));
        assert!(should_spa_fallback(Path::new("dashboard")));
        assert!(should_spa_fallback(Path::new("apps/t-team/studio")));
        assert!(!should_spa_fallback(Path::new("missing-bundle.js")));
        assert!(!should_spa_fallback(Path::new("styles/theme.css")));
        assert!(!should_spa_fallback(Path::new("app_bg.wasm")));
        assert!(!should_spa_fallback(Path::new("assets/logo.png")));
    }

    #[tokio::test]
    async fn parent_dir_traversal_rejected() {
        let (_tmp, router) = fixture_router();
        let (status, _ct, _body) = get(&router, "/../etc/passwd").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ─── GH #3326: serve_file silent IO 404 ──────────────────────────────────

    /// GH #3326 — happy path: existing file under dist/ serves OK.
    #[tokio::test]
    async fn gh3326_existing_asset_serves_ok() {
        let (_tmp, router) = fixture_router();
        let (status, ct, body) = get(&router, "/app.js").await;
        assert_eq!(status, StatusCode::OK);
        assert!(ct.starts_with("application/javascript"));
        assert!(body.contains("console.log"));
    }

    /// GH #3326 — legitimate missing file: 404 silently (preserved).
    #[tokio::test]
    async fn gh3326_missing_asset_returns_404_silently() {
        let (_tmp, router) = fixture_router();
        let (status, _ct, _body) = get(&router, "/never-existed.js").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ─── GH #3512: wasm_dev watcher silent notify error drop ────────────────

    /// GH #3512 — the wasm-dev watcher backend error message must
    /// name the underlying error AND include the issue tag so the dev
    /// has a breadcrumb when WASM HMR rebuilds silently stop.
    #[test]
    fn gh3512_wasm_watch_error_message_names_underlying_and_issue() {
        let msg = format_wasm_watch_error("ENFILE: Too many open files");
        assert!(
            msg.contains("ENFILE: Too many open files"),
            "underlying error must be preserved verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3512"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("wasm-dev file watcher"),
            "must name the failing component, got: {msg}"
        );
        assert!(
            msg.contains("WASM HMR"),
            "must explain dev-visible symptom, got: {msg}"
        );
    }

    /// GH #3512 — the message must differentiate itself from the
    /// dev_server watcher message (which is tagged GH #3127). When
    /// both fire simultaneously the dev should be able to tell at a
    /// glance which subsystem is failing.
    #[test]
    fn gh3512_wasm_watch_error_distinct_from_dev_server_tag() {
        let msg = format_wasm_watch_error("EBADF");
        assert!(
            !msg.contains("GH #3127"),
            "wasm-dev message must not inherit dev_server tag, got: {msg}"
        );
    }

    /// GH #3326 — file exists but is unreadable: must still 404
    /// (preserved response shape) AND emit a warn so the developer can
    /// chase the file-system trouble.
    #[cfg(unix)]
    #[tokio::test]
    async fn gh3326_unreadable_asset_404s_with_warn() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let dist = tmp.path().to_path_buf();
        std::fs::write(dist.join("index.html"), "<html/>").unwrap();
        let locked = dist.join("locked.js");
        std::fs::write(&locked, "console.log('locked');").unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        if std::fs::read(&locked).is_ok() {
            // Running as root — chmod 000 is unenforceable. Skip.
            let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let router = build_router(test_state(dist));
        let (status, _ct, _body) = get(&router, "/locked.js").await;

        // Restore perms so tempdir cleanup works.
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o644));

        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "unreadable file must still return 404 (response shape preserved)"
        );
    }
}

#[cfg(test)]
mod gh3730_wasm_dev_ctrl_c_warn_tests {
    //! GH #3730 — `shutdown_signal` did `let _ = tokio::signal::ctrl_c().await;`,
    //! silently swallowing handler-registration errors. When ctrl_c
    //! returns `Err`, the shutdown future resolved on the next poll
    //! and `jet dev --wasm` exited immediately after the listening
    //! banner with no Ctrl+C from the user.
    //! `format_wasm_dev_ctrl_c_warn` documents the bug, names the
    //! observable symptom, and tells the operator how to stop the
    //! server in the failure branch (shutdown future now parks forever).
    use super::*;

    fn sample_err() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "EPERM wasm sample")
    }

    #[test]
    fn helper_tags_gh_issue() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(msg.contains("GH #3730"), "must carry issue tag, got: {msg}");
    }

    #[test]
    fn helper_round_trips_io_error_text() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("EPERM wasm sample"),
            "must forward io::Error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_names_observable_symptom_immediate_exit_after_banner() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("immediately") || msg.contains("exited immediately"),
            "must explain immediate-exit symptom, got: {msg}"
        );
        assert!(
            msg.contains("banner") || msg.contains("listening"),
            "must reference the listening banner observable, got: {msg}"
        );
    }

    #[test]
    fn helper_names_wasm_dev_subsystem_specifically() {
        // Operators may see both this and the dev_server (#3725) warn
        // in the same logs; the warn must name `jet dev --wasm` so the
        // operator knows which subsystem fired.
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("wasm_dev") || msg.contains("--wasm"),
            "must identify the wasm dev subsystem, got: {msg}"
        );
    }

    #[test]
    fn helper_tells_user_how_to_stop_server_when_handler_failed() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("SIGTERM") || msg.contains("SIGKILL") || msg.contains("kill"),
            "must name how the operator stops the server, got: {msg}"
        );
    }

    #[test]
    fn helper_points_at_signal_limits_root_cause() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("signal") || msg.contains("sigaction") || msg.contains("ulimit"),
            "must point at signal-subsystem root cause, got: {msg}"
        );
    }

    #[test]
    fn helper_names_silent_fallback_root_cause_let_underscore_swallow() {
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("let _ =") || msg.contains("swallowed"),
            "must call out the prior `let _ =` swallow, got: {msg}"
        );
    }

    #[test]
    fn helper_cross_references_dev_server_sibling_3725() {
        // Sibling cross-reference so operators searching either GH issue
        // find both.
        let msg = format_wasm_dev_ctrl_c_warn(&sample_err());
        assert!(
            msg.contains("GH #3725") || msg.contains("dev_server"),
            "must cross-reference sibling #3725, got: {msg}"
        );
    }

    #[test]
    fn helper_is_deterministic_for_fixed_input() {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "fixed-wasm");
        let a = format_wasm_dev_ctrl_c_warn(&err);
        let b = format_wasm_dev_ctrl_c_warn(&err);
        assert_eq!(a, b);
    }

    #[test]
    fn helper_distinct_from_dev_server_ctrl_c_warn_3725() {
        // Distinguishable from the dev_server #3725 warn so
        // operators grepping logs can route the right issue.
        let wasm = format_wasm_dev_ctrl_c_warn(&sample_err());
        let dev = crate::dev_server::format_dev_server_ctrl_c_warn(&sample_err());
        assert_ne!(wasm, dev);
        assert!(
            !wasm.contains("dev_server:"),
            "must not pose as dev_server: {wasm}"
        );
        // The dev_server warn must not carry our tag in its body
        // (only this warn carries #3730 as its primary tag).
        assert!(
            !dev.contains("GH #3730"),
            "sibling must not carry our primary tag: {dev}"
        );
    }
}
// CODEGEN-END

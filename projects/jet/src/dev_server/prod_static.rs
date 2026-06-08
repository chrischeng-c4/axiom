// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
//! Production static server used by `jet serve --prod`.

use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ProdOptions {
    pub host: String,
    pub port: u16,
    pub target: &'static str,
}

#[derive(Clone)]
struct ProdState {
    dist: Arc<PathBuf>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub async fn serve(root_dir: &Path, opts: ProdOptions) -> Result<()> {
    let dist = root_dir.join("dist");
    let index = dist.join("index.html");
    if !index.is_file() {
        anyhow::bail!(
            "jet serve --prod expected {} to exist. Run `jet build` first.",
            index.display()
        );
    }

    let addr: SocketAddr = format!("{}:{}", opts.host, opts.port)
        .parse()
        .with_context(|| format!("invalid host:port {}:{}", opts.host, opts.port))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr().context("resolving bound addr")?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let app = router(ProdState {
        dist: Arc::new(dist.clone()),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    });

    eprintln!(
        "[jet serve --prod] serving {} at http://{}/",
        dist.display(),
        bound
    );
    println!(
        "jet-prod-server:listening {{\"port\":{},\"host\":\"{}\"}}",
        bound.port(),
        bound.ip()
    );
    if let Err(err) = super::session::write_from_env(root_dir, bound, opts.target) {
        eprintln!("[jet serve --prod] failed to write serve session: {err:#}");
    }

    let shutdown = async {
        let reason = tokio::select! {
            _ = shutdown_rx => "jet serve shutdown",
            _ = shutdown_signal() => "Ctrl-C",
        };
        eprintln!("[jet serve --prod] shutting down ({reason})...");
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("HTTP server error")?;
    eprintln!("[jet serve --prod] stopped.");
    Ok(())
}

fn router(state: ProdState) -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/__jet_shutdown", post(handle_shutdown))
        .route("/{*path}", get(handle_static))
        .with_state(state)
}

async fn handle_index(State(state): State<ProdState>) -> Response {
    serve_file_or_status(&state.dist.join("index.html"), StatusCode::OK).await
}

async fn handle_static(
    State(state): State<ProdState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    let Some(rel) = sanitize_rel_path(&path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let candidate = state.dist.join(&rel);
    if candidate.is_file() {
        return serve_file_or_status(&candidate, StatusCode::OK).await;
    }

    if Path::new(&rel).extension().is_none() {
        return serve_file_or_status(&state.dist.join("index.html"), StatusCode::OK).await;
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn handle_shutdown(State(state): State<ProdState>) -> Response {
    let mut shutdown_tx = state.shutdown_tx.lock().await;
    if let Some(tx) = shutdown_tx.take() {
        let _ = tx.send(());
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet serve --prod shutdown requested\n"))
            .expect("valid shutdown response")
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet serve --prod shutdown already requested\n"))
            .expect("valid shutdown response")
    }
}

async fn serve_file_or_status(path: &Path, missing_status: StatusCode) -> Response {
    match tokio::fs::read(path).await {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type_for(path)),
            );
            response
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => missing_status.into_response(),
        Err(err) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!(
                "failed to read {}: {err}\n",
                path.display()
            )))
            .expect("valid error response"),
    }
}

fn sanitize_rel_path(raw: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in Path::new(raw.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => return None,
        }
    }
    Some(out)
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::warn!(
            target: "jet::prod_static",
            error = %err,
            "failed to install Ctrl-C handler for jet serve --prod; server will keep running until shutdown endpoint or process termination"
        );
        std::future::pending::<()>().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_rejects_path_traversal() {
        assert!(sanitize_rel_path("../secret").is_none());
        assert!(sanitize_rel_path("ok/../../secret").is_none());
        assert_eq!(
            sanitize_rel_path("/assets/app.js").unwrap(),
            PathBuf::from("assets/app.js")
        );
    }

    #[test]
    fn content_type_covers_core_prod_assets() {
        assert_eq!(
            content_type_for(Path::new("index.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            content_type_for(Path::new("boot.js")),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type_for(Path::new("app.wasm")), "application/wasm");
    }
}
// CODEGEN-END

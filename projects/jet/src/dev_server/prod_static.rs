// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
//! Production static server used by `jet serve`.

use anyhow::{Context, Result};
use axum::{
    body::{Body, Bytes},
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use httpdate::{fmt_http_date, parse_http_date};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{oneshot, Mutex};
use walkdir::WalkDir;

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
#[derive(Debug, Clone)]
pub struct ProdOptions {
    pub host: String,
    pub port: u16,
    pub target: &'static str,
}

#[derive(Clone)]
struct ProdState {
    manifest: Arc<StaticManifest>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

#[derive(Debug, Clone)]
struct StaticManifest {
    assets: HashMap<String, StaticAsset>,
    index: StaticAsset,
    total_bytes: u64,
}

#[derive(Debug, Clone)]
struct StaticAsset {
    bytes: Bytes,
    content_type: &'static str,
    cache_control: &'static str,
    etag: String,
    last_modified: Option<SystemTime>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl StaticAsset {
    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub async fn serve(root_dir: &Path, opts: ProdOptions) -> Result<()> {
    let dist = root_dir.join("dist");
    let index = dist.join("index.html");
    if !index.is_file() {
        anyhow::bail!(
            "jet serve expected {} to exist. Run `jet build` first.",
            index.display()
        );
    }

    let manifest = load_static_manifest(&dist)?;
    let addr: SocketAddr = format!("{}:{}", opts.host, opts.port)
        .parse()
        .with_context(|| format!("invalid host:port {}:{}", opts.host, opts.port))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let bound = listener.local_addr().context("resolving bound addr")?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let app = router(ProdState {
        manifest: Arc::new(manifest.clone()),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    });

    eprintln!(
        "[jet serve] serving {} at http://{}/ ({} files, {} bytes preloaded)",
        dist.display(),
        bound,
        manifest.assets.len(),
        manifest.total_bytes
    );
    println!(
        "jet-prod-server:listening {{\"port\":{},\"host\":\"{}\"}}",
        bound.port(),
        bound.ip()
    );
    if let Err(err) = super::session::write_from_env(root_dir, bound, opts.target) {
        eprintln!("[jet serve] failed to write serve session: {err:#}");
    }

    let shutdown = async {
        let reason = tokio::select! {
            _ = shutdown_rx => "jet serve shutdown",
            _ = shutdown_signal() => "Ctrl-C",
        };
        eprintln!("[jet serve] shutting down ({reason})...");
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .context("HTTP server error")?;
    eprintln!("[jet serve] stopped.");
    Ok(())
}

fn router(state: ProdState) -> Router {
    Router::new()
        .route("/", get(handle_index).head(handle_index_head))
        .route("/__jet_health", get(handle_health).head(handle_health_head))
        .route("/__jet_ready", get(handle_ready).head(handle_ready_head))
        .route("/__jet_shutdown", post(handle_shutdown))
        .route("/{*path}", get(handle_static).head(handle_static_head))
        .with_state(state)
}

async fn handle_index(headers: HeaderMap, State(state): State<ProdState>) -> Response {
    asset_response(&state.manifest.index, &headers, true)
}

async fn handle_index_head(headers: HeaderMap, State(state): State<ProdState>) -> Response {
    asset_response(&state.manifest.index, &headers, false)
}

async fn handle_static(
    headers: HeaderMap,
    State(state): State<ProdState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    static_response(&state.manifest, &path, &headers, true)
}

async fn handle_static_head(
    headers: HeaderMap,
    State(state): State<ProdState>,
    AxumPath(path): AxumPath<String>,
) -> Response {
    static_response(&state.manifest, &path, &headers, false)
}

fn static_response(
    manifest: &StaticManifest,
    path: &str,
    headers: &HeaderMap,
    send_body: bool,
) -> Response {
    let Some(rel) = sanitize_rel_path(path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let key = rel_path_to_url_key(&rel);
    if let Some(asset) = manifest.assets.get(&key) {
        return asset_response(asset, headers, send_body);
    }
    if Path::new(&rel).extension().is_none() {
        return asset_response(&manifest.index, headers, send_body);
    }
    text_response(StatusCode::NOT_FOUND, "not found\n", "no-store", send_body)
}

async fn handle_health() -> Response {
    text_response(StatusCode::OK, "ok\n", "no-store", true)
}

async fn handle_health_head() -> Response {
    text_response(StatusCode::OK, "ok\n", "no-store", false)
}

async fn handle_ready() -> Response {
    text_response(StatusCode::OK, "ready\n", "no-store", true)
}

async fn handle_ready_head() -> Response {
    text_response(StatusCode::OK, "ready\n", "no-store", false)
}

async fn handle_shutdown(State(state): State<ProdState>) -> Response {
    let mut shutdown_tx = state.shutdown_tx.lock().await;
    if let Some(tx) = shutdown_tx.take() {
        let _ = tx.send(());
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet serve shutdown requested\n"))
            .expect("valid shutdown response")
    } else {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("jet serve shutdown already requested\n"))
            .expect("valid shutdown response")
    }
}

fn asset_response(asset: &StaticAsset, headers: &HeaderMap, send_body: bool) -> Response {
    if request_etag_matches(headers, &asset.etag)
        || request_not_modified_since(headers, asset.last_modified)
    {
        let mut response = Response::builder().status(StatusCode::NOT_MODIFIED);
        apply_common_asset_headers(response.headers_mut().unwrap(), asset);
        return response.body(Body::empty()).expect("valid 304 response");
    }

    match parse_range(headers, asset.len()) {
        RangeSelection::Full => {
            let mut response = Response::builder().status(StatusCode::OK);
            apply_common_asset_headers(response.headers_mut().unwrap(), asset);
            response.headers_mut().unwrap().insert(
                header::CONTENT_LENGTH,
                header_value(&asset.len().to_string()),
            );
            let body = if send_body {
                Body::from(asset.bytes.clone())
            } else {
                Body::empty()
            };
            response.body(body).expect("valid static response")
        }
        RangeSelection::Partial { start, end } => {
            let range_len = end - start + 1;
            let mut response = Response::builder().status(StatusCode::PARTIAL_CONTENT);
            apply_common_asset_headers(response.headers_mut().unwrap(), asset);
            response.headers_mut().unwrap().insert(
                header::CONTENT_RANGE,
                header_value(&format!("bytes {start}-{end}/{}", asset.len())),
            );
            response
                .headers_mut()
                .unwrap()
                .insert(header::CONTENT_LENGTH, header_value(&range_len.to_string()));
            let body = if send_body {
                Body::from(asset.bytes.slice(start as usize..(end as usize + 1)))
            } else {
                Body::empty()
            };
            response.body(body).expect("valid range response")
        }
        RangeSelection::Unsatisfiable => {
            let mut response = Response::builder().status(StatusCode::RANGE_NOT_SATISFIABLE);
            apply_common_asset_headers(response.headers_mut().unwrap(), asset);
            response.headers_mut().unwrap().insert(
                header::CONTENT_RANGE,
                header_value(&format!("bytes */{}", asset.len())),
            );
            response.body(Body::empty()).expect("valid 416 response")
        }
    }
}

fn apply_common_asset_headers(headers: &mut HeaderMap, asset: &StaticAsset) {
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(asset.content_type),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(asset.cache_control),
    );
    headers.insert(header::ETAG, header_value(&asset.etag));
    headers.insert(
        header::HeaderName::from_static("accept-ranges"),
        HeaderValue::from_static("bytes"),
    );
    headers.insert(
        header::HeaderName::from_static("x-jet-prod-static"),
        HeaderValue::from_static("memory-manifest"),
    );
    if let Some(last_modified) = asset.last_modified {
        headers.insert(
            header::LAST_MODIFIED,
            header_value(&fmt_http_date(last_modified)),
        );
    }
}

fn text_response(
    status: StatusCode,
    body: &'static str,
    cache_control: &'static str,
    send_body: bool,
) -> Response {
    let mut response = Response::builder().status(status);
    response.headers_mut().unwrap().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response.headers_mut().unwrap().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(cache_control),
    );
    response.headers_mut().unwrap().insert(
        header::CONTENT_LENGTH,
        header_value(&body.len().to_string()),
    );
    response
        .body(if send_body {
            Body::from(body)
        } else {
            Body::empty()
        })
        .expect("valid text response")
}

fn request_etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    let Some(value) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    value
        .split(',')
        .map(str::trim)
        .any(|candidate| candidate == "*" || candidate == etag)
}

fn request_not_modified_since(headers: &HeaderMap, last_modified: Option<SystemTime>) -> bool {
    let Some(last_modified) = last_modified else {
        return false;
    };
    let Some(value) = headers
        .get(header::IF_MODIFIED_SINCE)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Ok(since) = parse_http_date(value) else {
        return false;
    };
    last_modified <= since + Duration::from_secs(1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeSelection {
    Full,
    Partial { start: u64, end: u64 },
    Unsatisfiable,
}

fn parse_range(headers: &HeaderMap, len: u64) -> RangeSelection {
    let Some(raw) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) else {
        return RangeSelection::Full;
    };
    let Some(spec) = raw.strip_prefix("bytes=") else {
        return RangeSelection::Full;
    };
    if spec.contains(',') || len == 0 {
        return RangeSelection::Unsatisfiable;
    }
    let Some((start_raw, end_raw)) = spec.split_once('-') else {
        return RangeSelection::Unsatisfiable;
    };
    if start_raw.is_empty() {
        let Ok(suffix_len) = end_raw.parse::<u64>() else {
            return RangeSelection::Unsatisfiable;
        };
        if suffix_len == 0 {
            return RangeSelection::Unsatisfiable;
        }
        let start = len.saturating_sub(suffix_len);
        return RangeSelection::Partial {
            start,
            end: len - 1,
        };
    }
    let Ok(start) = start_raw.parse::<u64>() else {
        return RangeSelection::Unsatisfiable;
    };
    if start >= len {
        return RangeSelection::Unsatisfiable;
    }
    let end = if end_raw.is_empty() {
        len - 1
    } else {
        let Ok(parsed) = end_raw.parse::<u64>() else {
            return RangeSelection::Unsatisfiable;
        };
        parsed.min(len - 1)
    };
    if end < start {
        return RangeSelection::Unsatisfiable;
    }
    RangeSelection::Partial { start, end }
}

fn load_static_manifest(dist: &Path) -> Result<StaticManifest> {
    let mut assets = HashMap::new();
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(dist).into_iter() {
        let entry = entry.with_context(|| format!("walking {}", dist.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(dist)
            .with_context(|| format!("relativizing {} under {}", path.display(), dist.display()))?;
        let key = rel_path_to_url_key(rel);
        let metadata =
            std::fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        total_bytes += bytes.len() as u64;
        let asset = StaticAsset {
            content_type: content_type_for(path),
            cache_control: cache_control_for(path),
            etag: etag_for_bytes(&bytes),
            last_modified: metadata.modified().ok(),
            bytes: Bytes::from(bytes),
        };
        assets.insert(key, asset);
    }

    let Some(index) = assets.get("index.html").cloned() else {
        anyhow::bail!(
            "jet serve expected {} to exist",
            dist.join("index.html").display()
        );
    };

    Ok(StaticManifest {
        assets,
        index,
        total_bytes,
    })
}

fn etag_for_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("\"{:x}\"", hasher.finalize())
}

fn header_value(value: &str) -> HeaderValue {
    HeaderValue::from_str(value).expect("static header value must be valid")
}

fn rel_path_to_url_key(rel: &Path) -> String {
    rel.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn cache_control_for(path: &Path) -> &'static str {
    if path.extension().and_then(|ext| ext.to_str()) == Some("html") {
        return "no-cache";
    }
    if is_hashed_asset(path) {
        return "public, max-age=31536000, immutable";
    }
    "no-cache"
}

fn is_hashed_asset(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let mut run = 0usize;
    for byte in name.bytes() {
        if byte.is_ascii_hexdigit() {
            run += 1;
            if run >= 8 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
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
            "failed to install Ctrl-C handler for jet serve; server will keep running until shutdown endpoint or process termination"
        );
        std::future::pending::<()>().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

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

    #[test]
    fn cache_policy_marks_html_revalidate_and_hashed_assets_immutable() {
        assert_eq!(cache_control_for(Path::new("index.html")), "no-cache");
        assert_eq!(
            cache_control_for(Path::new("assets/app.1234abcd.js")),
            "public, max-age=31536000, immutable"
        );
        assert_eq!(cache_control_for(Path::new("brand.svg")), "no-cache");
    }

    #[test]
    fn byte_range_parser_supports_prefix_open_and_suffix_ranges() {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=1-3"));
        assert_eq!(
            parse_range(&headers, 6),
            RangeSelection::Partial { start: 1, end: 3 }
        );

        headers.insert(header::RANGE, HeaderValue::from_static("bytes=4-"));
        assert_eq!(
            parse_range(&headers, 6),
            RangeSelection::Partial { start: 4, end: 5 }
        );

        headers.insert(header::RANGE, HeaderValue::from_static("bytes=-2"));
        assert_eq!(
            parse_range(&headers, 6),
            RangeSelection::Partial { start: 4, end: 5 }
        );

        headers.insert(header::RANGE, HeaderValue::from_static("bytes=9-12"));
        assert_eq!(parse_range(&headers, 6), RangeSelection::Unsatisfiable);
    }

    #[tokio::test]
    async fn asset_response_honors_range_headers() {
        let asset = test_asset("assets/app.1234abcd.js", b"abcdef");
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=1-3"));

        let response = asset_response(&asset, &headers, true);
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_RANGE)
                .unwrap()
                .to_str()
                .unwrap(),
            "bytes 1-3/6"
        );
        assert_eq!(
            response
                .headers()
                .get(header::HeaderName::from_static("accept-ranges"))
                .unwrap()
                .to_str()
                .unwrap(),
            "bytes"
        );
        let body = to_bytes(response.into_body(), 16).await.unwrap();
        assert_eq!(&body[..], b"bcd");
    }

    #[tokio::test]
    async fn asset_response_honors_if_none_match() {
        let asset = test_asset("assets/app.1234abcd.js", b"abcdef");
        let mut headers = HeaderMap::new();
        headers.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_str(&asset.etag).unwrap(),
        );

        let response = asset_response(&asset, &headers, true);
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        let body = to_bytes(response.into_body(), 16).await.unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn asset_response_head_mode_keeps_length_without_body() {
        let asset = test_asset("assets/app.1234abcd.js", b"abcdef");
        let headers = HeaderMap::new();

        let response = asset_response(&asset, &headers, false);
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .unwrap(),
            "6"
        );
        let body = to_bytes(response.into_body(), 16).await.unwrap();
        assert!(body.is_empty());
    }

    fn test_asset(path: &str, bytes: &'static [u8]) -> StaticAsset {
        StaticAsset {
            bytes: Bytes::from_static(bytes),
            content_type: content_type_for(Path::new(path)),
            cache_control: cache_control_for(Path::new(path)),
            etag: etag_for_bytes(bytes),
            last_modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(10)),
        }
    }
}
// CODEGEN-END

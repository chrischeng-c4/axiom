// <HANDWRITE gap="missing-generator:unit-test:99fe4be9" tracker="standardize-gap-projects-jet-tests-publish-library-publish-e2e-rs" reason="In-process mock npm registry (axum) e2e: jet publish a built library to the mock registry, then resolve+download it back (install round-trip), asserting the tarball contains the built JS/.d.ts and that scoped private-registry routing + Bearer auth are exercised. Plus metadata-validation unit tests (missing main/exports path -> error; auto-fill from build output).">
//! End-to-end `jet publish --build` against an in-process mock npm registry.
//!
//! What this exercises (all hermetic — no real network, no Verdaccio/npm):
//!   1. A temp library project (package.json + a tiny TS lib).
//!   2. A temp `.npmrc` pointing a scoped registry at the mock + an auth token.
//!   3. `Publisher::with_build(true).publish(...)` — which builds the library,
//!      auto-fills `main`/`module`/`types`, validates them, packs, and PUTs.
//!   4. Assertions on what the mock received: the PUT reached it, carried a
//!      `Bearer` auth header, and the published tarball contains the built
//!      `index.js` / `index.cjs` / `index.d.ts`.
//!   5. The install/resolve round-trip: a `RegistryClient` pointed at the mock
//!      `GET`s the metadata back and `download_package` pulls the same tarball
//!      bytes the publisher uploaded.
//!
//! @issue #172
//!
//! ## How the mock registry works
//!
//! `spawn_mock_registry()` binds an axum router on `127.0.0.1:0` (ephemeral
//! port) and returns its base URL plus a shared `Store`. Three routes:
//!
//!   * `PUT /{name}`  — npm publish: the body is the publish envelope
//!     (`name`, `dist-tags`, `versions`, `_attachments[*].data` = base64
//!     tarball). We decode the attachment, store the raw tarball bytes, and
//!     record the dist-tags + the `Authorization` header we saw.
//!   * `GET /{name}`  — npm metadata: we synthesize npm-style
//!     `PackageMetadata` JSON whose `versions[v].dist.tarball` points back at
//!     this same mock (`GET /{name}/-/{file}.tgz`).
//!   * `GET /{name}/-/{file}` — tarball download: returns the stored bytes.
//!
//! Scoped names (`@scope/pkg`) arrive URL-encoded as `@scope%2Fpkg`; axum's
//! `Path` extractor decodes them, so the stored key is the real package name.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    body::Bytes,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use jet::pkg_manager::npmrc::NpmrcConfig;
use jet::pkg_manager::publish::Publisher;
use jet::pkg_manager::registry::RegistryClient;
use tempfile::tempdir;

// ──────────────────────────────────────────────────────────────────────────
// Mock registry
// ──────────────────────────────────────────────────────────────────────────

/// One published package as the mock saw it.
#[derive(Default, Clone)]
struct Published {
    /// Raw (gzipped tar) tarball bytes decoded from the `_attachments` base64.
    tarball: Vec<u8>,
    /// `dist-tags` map from the publish body (e.g. `{"latest": "1.0.0"}`).
    dist_tags: HashMap<String, String>,
    /// The `Authorization` header value the PUT carried, if any.
    auth_header: Option<String>,
    /// The tarball file name the publisher used inside `_attachments`.
    tarball_file: String,
}

/// Shared mock state: package name → what was published.
type Store = Arc<Mutex<HashMap<String, Published>>>;

/// Spawn the mock registry on an ephemeral loopback port. Returns the base URL
/// (e.g. `http://127.0.0.1:54321`) and the shared store.
async fn spawn_mock_registry() -> (String, Store) {
    let store: Store = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        // npm publish (PUT) and metadata (GET) for unscoped names share
        // `/{name}`.
        .route("/{name}", put(handle_put).get(handle_get_meta))
        // jet's publisher does NOT %2F-encode the scope separator, so a scoped
        // `@acme/widget` arrives as two path segments. Match it explicitly and
        // rejoin into the real package name.
        .route(
            "/{scope}/{pkg}",
            put(handle_put_scoped).get(handle_get_meta_scoped),
        )
        // Tarball download — flat, slash-free key segment.
        .route("/-/tarball/{file}", get(handle_get_tarball))
        .with_state(store.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (base, store)
}

/// Minimal base64 decoder (mirror of the encoder jet uses for `_attachments`).
fn base64_decode(s: &str) -> Vec<u8> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::new();
    let bytes: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    for chunk in bytes.chunks(4) {
        let mut buf = [0u32; 4];
        let mut pad = 0;
        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                pad += 1;
                buf[i] = 0;
            } else {
                buf[i] = val(c).unwrap_or(0);
            }
        }
        let triple = (buf[0] << 18) | (buf[1] << 12) | (buf[2] << 6) | buf[3];
        out.push((triple >> 16) as u8);
        if pad < 2 {
            out.push((triple >> 8) as u8);
        }
        if pad < 1 {
            out.push(triple as u8);
        }
    }
    out
}

/// PUT /{scope}/{pkg} — scoped publish; rejoin into the real `@scope/pkg`.
async fn handle_put_scoped(
    AxumPath((scope, pkg)): AxumPath<(String, String)>,
    state: State<Store>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    store_put(format!("{scope}/{pkg}"), state, headers, body)
}

/// GET /{scope}/{pkg} — scoped metadata; rejoin into the real `@scope/pkg`.
async fn handle_get_meta_scoped(
    AxumPath((scope, pkg)): AxumPath<(String, String)>,
    state: State<Store>,
    headers: HeaderMap,
) -> impl IntoResponse {
    serve_meta(format!("{scope}/{pkg}"), state, headers)
}

/// PUT /{name} — store the published tarball + dist-tags + auth header.
async fn handle_put(
    AxumPath(name): AxumPath<String>,
    state: State<Store>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    store_put(name, state, headers, body)
}

/// Shared PUT body: decode `_attachments`, record tags + auth header.
fn store_put(
    name: String,
    State(store): State<Store>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let body: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("bad body: {e}")).into_response(),
    };

    let mut published = Published {
        auth_header: headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(String::from),
        ..Default::default()
    };

    if let Some(tags) = body.get("dist-tags").and_then(|v| v.as_object()) {
        for (k, v) in tags {
            if let Some(s) = v.as_str() {
                published.dist_tags.insert(k.clone(), s.to_string());
            }
        }
    }

    if let Some(attachments) = body.get("_attachments").and_then(|v| v.as_object()) {
        if let Some((file, att)) = attachments.iter().next() {
            published.tarball_file = file.clone();
            if let Some(data) = att.get("data").and_then(|v| v.as_str()) {
                published.tarball = base64_decode(data);
            }
        }
    }

    store.lock().unwrap().insert(name, published);
    (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response()
}

/// GET /{name} — synthesize npm metadata pointing the tarball URL back here.
async fn handle_get_meta(
    AxumPath(name): AxumPath<String>,
    state: State<Store>,
    headers: HeaderMap,
) -> impl IntoResponse {
    serve_meta(name, state, headers)
}

/// Sanitize a package name into a single safe download-path segment, so the
/// tarball URL never re-introduces a `/` (which would split into two segments).
fn tarball_key(name: &str) -> String {
    name.replace('/', "__").replace('@', "_at_")
}

/// Shared metadata response: synthesize npm-style metadata whose
/// `dist.tarball` points back at this mock's tarball download route.
fn serve_meta(
    name: String,
    State(store): State<Store>,
    headers: HeaderMap,
) -> axum::response::Response {
    let guard = store.lock().unwrap();
    let Some(pub_) = guard.get(&name) else {
        return (StatusCode::NOT_FOUND, "not published").into_response();
    };

    let version = pub_
        .dist_tags
        .get("latest")
        .cloned()
        .unwrap_or_else(|| "0.0.0".to_string());

    // Build the tarball download URL from the Host header so it points back at
    // this same mock instance. Use a flat, slash-free key segment.
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1");
    let key = tarball_key(&name);
    let tarball_url = format!("http://{host}/-/tarball/{key}.tgz");

    let metadata = serde_json::json!({
        "name": name,
        "dist-tags": pub_.dist_tags,
        "versions": {
            version.clone(): {
                "version": version,
                "dist": {
                    "tarball": tarball_url,
                    "shasum": "0000000000000000000000000000000000000000",
                    "integrity": null
                }
            }
        }
    });

    (StatusCode::OK, Json(metadata)).into_response()
}

/// GET /-/tarball/{key}.tgz — return the stored tarball bytes for `key`.
async fn handle_get_tarball(
    AxumPath(file): AxumPath<String>,
    State(store): State<Store>,
) -> impl IntoResponse {
    let key = file.trim_end_matches(".tgz");
    let guard = store.lock().unwrap();
    let hit = guard
        .iter()
        .find(|(name, _)| tarball_key(name) == key)
        .map(|(_, p)| p.tarball.clone());
    match hit {
        Some(bytes) => (StatusCode::OK, bytes).into_response(),
        None => (StatusCode::NOT_FOUND, "no tarball").into_response(),
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Tarball inspection helper
// ──────────────────────────────────────────────────────────────────────────

/// Decompress a gzipped tar and return the list of entry paths it contains.
fn tarball_entry_paths(tarball: &[u8]) -> Vec<String> {
    use flate2::read::GzDecoder;
    let gz = GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(gz);
    let mut paths = Vec::new();
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        paths.push(entry.path().unwrap().to_string_lossy().to_string());
    }
    paths
}

// ──────────────────────────────────────────────────────────────────────────
// E2E test
// ──────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn publish_built_library_to_mock_registry_and_resolve_back() {
    // 1. Spawn the mock registry.
    let (base_url, store) = spawn_mock_registry().await;

    // 2. Temp library project: a scoped package with a tiny TS lib.
    let dir = tempdir().unwrap();
    let root = dir.path();
    let pkg_name = "@acme/widget";

    std::fs::write(
        root.join("package.json"),
        format!(
            r#"{{
  "name": "{pkg_name}",
  "version": "1.0.0",
  "module": "./src/index.ts"
}}"#
        ),
    )
    .unwrap();

    // Tiny TS lib with an internal helper (inlined) + a typed export (drives
    // .d.ts emission).
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/util.ts"),
        "export function double(x: number): number { return x * 2; }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/index.ts"),
        r#"import { double } from "./util";

export interface Widget { size: number; }

export function makeWidget(n: number): Widget {
    return { size: double(n) };
}
"#,
    )
    .unwrap();

    // 3. Temp `.npmrc`: scoped registry + auth token pointing at the mock.
    //    The auth-token key matches npm's `//host/path/:_authToken` shape; the
    //    host portion is what `auth_token_for` substring-matches against.
    let registry_host = base_url.trim_start_matches("http://");
    std::fs::write(
        root.join(".npmrc"),
        format!(
            "@acme:registry={base_url}/\n//{registry_host}/:_authToken=secret-token-xyz\n"
        ),
    )
    .unwrap();

    // 4. Publish WITH build. This runs `jet build --lib`, auto-fills
    //    main/module/types, validates them, packs, and PUTs to the mock.
    let publisher = Publisher::new(root.to_path_buf()).with_build(true);
    publisher
        .publish("latest", Some("restricted"))
        .await
        .expect("publish --build to the mock registry must succeed");

    // 5. Assert the mock received the publish with the right shape.
    let published = {
        let guard = store.lock().unwrap();
        guard
            .get(pkg_name)
            .cloned()
            .expect("the mock registry must have received a PUT for @acme/widget")
    };

    // 5a. Bearer auth header was sent (scoped private-registry auth exercised).
    assert_eq!(
        published.auth_header.as_deref(),
        Some("Bearer secret-token-xyz"),
        "publish must send the scoped registry's Bearer auth token"
    );

    // 5b. dist-tags carried our tag → version.
    assert_eq!(
        published.dist_tags.get("latest").map(String::as_str),
        Some("1.0.0"),
        "publish must record dist-tags latest=1.0.0"
    );

    // 5c. The tarball contains the freshly-built dist files (ESM + CJS + .d.ts)
    //     plus the transformed package.json.
    assert!(
        !published.tarball.is_empty(),
        "the mock must have stored a non-empty tarball"
    );
    let paths = tarball_entry_paths(&published.tarball);
    let has = |needle: &str| paths.iter().any(|p| p.ends_with(needle));
    assert!(
        has("dist/index.js"),
        "tarball must contain the built ESM file dist/index.js, got: {paths:?}"
    );
    assert!(
        has("dist/index.cjs"),
        "tarball must contain the built CJS file dist/index.cjs, got: {paths:?}"
    );
    assert!(
        has("dist/index.d.ts"),
        "tarball must contain the built declaration dist/index.d.ts, got: {paths:?}"
    );
    assert!(
        has("package/package.json"),
        "tarball must contain package.json, got: {paths:?}"
    );

    // 6. Install/resolve round-trip: a RegistryClient pointed at the mock must
    //    GET resolvable metadata back and download the very bytes we published.
    let npmrc = NpmrcConfig::load(root);
    let registry_url = npmrc.registry_for(pkg_name);
    assert!(
        registry_url.starts_with(&base_url),
        "scoped registry must route @acme/* to the mock, got {registry_url}"
    );

    // no_cache=true so we don't touch the user's real disk cache.
    let client = RegistryClient::new_with_options(registry_url, &npmrc, true)
        .expect("registry client");

    let metadata = client
        .get_package_metadata(pkg_name)
        .await
        .expect("metadata GET must round-trip through the mock");
    assert_eq!(metadata.name, pkg_name);
    assert_eq!(
        metadata.dist_tags.get("latest").map(String::as_str),
        Some("1.0.0"),
        "resolved metadata must report latest=1.0.0"
    );

    let downloaded = client
        .download_package(pkg_name, "1.0.0")
        .await
        .expect("tarball download must round-trip through the mock");
    assert_eq!(
        downloaded, published.tarball,
        "downloaded tarball bytes must be byte-identical to what was published"
    );
}
// </HANDWRITE>

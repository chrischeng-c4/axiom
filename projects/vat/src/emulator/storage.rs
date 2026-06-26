// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-storage-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Built-in Cloud Storage (GCS) emulator — an axum server for the GCS JSON API
//! v1 subset over an in-memory store. Google ships no standalone GCS emulator,
//! so this is vat's own; the client SDKs reach it through the standard
//! `STORAGE_EMULATOR_HOST` env var, so the runner needs no code change. Supports
//! bucket CRUD (auto-create on upload), object upload (media / multipart /
//! minimal resumable), download (`alt=media`), metadata, list (prefix), and
//! delete. Blob state is in-memory and per-run.
//!
//! @spec projects/vat/tech-design/logic/built-in-cloud-storage-gcs-emulator.md#logic

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use md5::{Digest, Md5};
use serde_json::{json, Value};

#[derive(Clone)]
struct Object {
    data: Vec<u8>,
    content_type: String,
    generation: u64,
    md5: String,
    updated: String,
}

#[derive(Default)]
struct Store {
    buckets: HashMap<String, Value>,
    objects: HashMap<(String, String), Object>,
    resumable: HashMap<String, (String, String)>, // upload_id -> (bucket, name)
    seq: u64,
}

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Store>>,
}

/// Serve the Cloud Storage emulator until the process is killed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-storage-rs.md#source
pub async fn serve(host_port: &str) -> Result<()> {
    let state = AppState {
        inner: Arc::new(Mutex::new(Store::default())),
    };
    let app = Router::new()
        .route("/storage/v1/b", post(create_bucket).get(list_buckets))
        .route(
            "/storage/v1/b/{bucket}",
            get(get_bucket).delete(delete_bucket),
        )
        .route("/storage/v1/b/{bucket}/o", get(list_objects))
        .route(
            "/storage/v1/b/{bucket}/o/{*object}",
            get(get_object).delete(delete_object),
        )
        .route(
            "/download/storage/v1/b/{bucket}/o/{*object}",
            get(get_object),
        )
        .route(
            "/upload/storage/v1/b/{bucket}/o",
            post(upload_object).put(resumable_put),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(host_port)
        .await
        .with_context(|| format!("bind cloud-storage emulator on {host_port}"))?;
    axum::serve(listener, app)
        .await
        .context("serve cloud-storage emulator")?;
    Ok(())
}

fn decode(name: &str) -> String {
    percent_encoding::percent_decode_str(name)
        .decode_utf8_lossy()
        .into_owned()
}

fn md5_base64(data: &[u8]) -> String {
    let digest = Md5::digest(data);
    base64::engine::general_purpose::STANDARD.encode(digest)
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn object_resource(bucket: &str, name: &str, obj: &Object) -> Value {
    json!({
        "kind": "storage#object",
        "bucket": bucket,
        "name": name,
        "size": obj.data.len().to_string(),
        "contentType": obj.content_type,
        "generation": obj.generation.to_string(),
        "md5Hash": obj.md5,
        "updated": obj.updated,
        "timeCreated": obj.updated,
        "selfLink": format!("/storage/v1/b/{bucket}/o/{name}"),
        "mediaLink": format!("/download/storage/v1/b/{bucket}/o/{name}?alt=media"),
    })
}

fn store_object(
    store: &mut Store,
    bucket: &str,
    name: &str,
    data: Vec<u8>,
    content_type: String,
) -> Value {
    store
        .buckets
        .entry(bucket.to_string())
        .or_insert_with(|| json!({ "kind": "storage#bucket", "name": bucket }));
    store.seq += 1;
    let obj = Object {
        md5: md5_base64(&data),
        data,
        content_type,
        generation: store.seq,
        updated: now(),
    };
    let resource = object_resource(bucket, name, &obj);
    store
        .objects
        .insert((bucket.to_string(), name.to_string()), obj);
    resource
}

// ---- buckets ----

async fn create_bucket(State(state): State<AppState>, Json(req): Json<Value>) -> Json<Value> {
    let name = req
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let bucket = json!({ "kind": "storage#bucket", "name": name });
    state
        .inner
        .lock()
        .unwrap()
        .buckets
        .insert(name, bucket.clone());
    Json(bucket)
}

async fn list_buckets(State(state): State<AppState>) -> Json<Value> {
    let store = state.inner.lock().unwrap();
    let items: Vec<Value> = store.buckets.values().cloned().collect();
    Json(json!({ "kind": "storage#buckets", "items": items }))
}

async fn get_bucket(State(state): State<AppState>, Path(bucket): Path<String>) -> Response {
    let store = state.inner.lock().unwrap();
    match store.buckets.get(&bucket) {
        Some(b) => Json(b.clone()).into_response(),
        None => not_found("bucket"),
    }
}

async fn delete_bucket(State(state): State<AppState>, Path(bucket): Path<String>) -> Json<Value> {
    let mut store = state.inner.lock().unwrap();
    store.buckets.remove(&bucket);
    store.objects.retain(|(b, _), _| b != &bucket);
    Json(json!({}))
}

// ---- objects ----

async fn list_objects(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(q): Query<HashMap<String, String>>,
) -> Json<Value> {
    let prefix = q.get("prefix").cloned().unwrap_or_default();
    let store = state.inner.lock().unwrap();
    let items: Vec<Value> = store
        .objects
        .iter()
        .filter(|((b, name), _)| b == &bucket && name.starts_with(&prefix))
        .map(|((b, name), obj)| object_resource(b, name, obj))
        .collect();
    Json(json!({ "kind": "storage#objects", "items": items }))
}

async fn get_object(
    State(state): State<AppState>,
    Path((bucket, object)): Path<(String, String)>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let name = decode(&object);
    let store = state.inner.lock().unwrap();
    let Some(obj) = store.objects.get(&(bucket.clone(), name.clone())) else {
        return not_found("object");
    };
    if q.get("alt").map(String::as_str) == Some("media") {
        let mut headers = HeaderMap::new();
        if let Ok(ct) = obj.content_type.parse() {
            headers.insert(header::CONTENT_TYPE, ct);
        }
        (headers, obj.data.clone()).into_response()
    } else {
        Json(object_resource(&bucket, &name, obj)).into_response()
    }
}

async fn delete_object(
    State(state): State<AppState>,
    Path((bucket, object)): Path<(String, String)>,
) -> Json<Value> {
    let name = decode(&object);
    state.inner.lock().unwrap().objects.remove(&(bucket, name));
    Json(json!({}))
}

// ---- upload ----

async fn upload_object(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let upload_type = q.get("uploadType").map(String::as_str).unwrap_or("media");
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    match upload_type {
        "resumable" => {
            // Start a resumable session; client PUTs the bytes next.
            let name = q.get("name").cloned().unwrap_or_default();
            let upload_id = {
                let mut store = state.inner.lock().unwrap();
                store.seq += 1;
                let id = format!("resumable-{}", store.seq);
                store.resumable.insert(id.clone(), (bucket.clone(), name));
                id
            };
            let mut h = HeaderMap::new();
            let loc = format!("/upload/storage/v1/b/{bucket}/o?upload_id={upload_id}");
            if let Ok(v) = loc.parse() {
                h.insert(header::LOCATION, v);
            }
            (StatusCode::OK, h, Json(json!({}))).into_response()
        }
        "multipart" => {
            let boundary = content_type
                .split("boundary=")
                .nth(1)
                .map(|b| b.trim_matches('"').to_string());
            let Some(boundary) = boundary else {
                return bad_request("multipart upload missing boundary");
            };
            let (meta, media, part_ct) = parse_multipart(&body, &boundary);
            let name = meta
                .get("name")
                .and_then(Value::as_str)
                .or_else(|| q.get("name").map(String::as_str))
                .unwrap_or_default()
                .to_string();
            let ct = meta
                .get("contentType")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or(part_ct);
            let mut store = state.inner.lock().unwrap();
            let resource = store_object(&mut store, &bucket, &name, media, ct);
            Json(resource).into_response()
        }
        _ => {
            // media: raw bytes, name from the query.
            let name = q.get("name").cloned().unwrap_or_default();
            let mut store = state.inner.lock().unwrap();
            let resource = store_object(&mut store, &bucket, &name, body.to_vec(), content_type);
            Json(resource).into_response()
        }
    }
}

async fn resumable_put(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let upload_id = q.get("upload_id").cloned().unwrap_or_default();
    let target = state.inner.lock().unwrap().resumable.remove(&upload_id);
    let Some((bkt, name)) = target else {
        // No session: fall back to treating it as a direct media PUT.
        let name = q.get("name").cloned().unwrap_or_default();
        let ct = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let mut store = state.inner.lock().unwrap();
        let resource = store_object(&mut store, &bucket, &name, body.to_vec(), ct);
        return Json(resource).into_response();
    };
    let ct = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let mut store = state.inner.lock().unwrap();
    let resource = store_object(&mut store, &bkt, &name, body.to_vec(), ct);
    Json(resource).into_response()
}

/// Split a `multipart/related` body into (metadata JSON, media bytes, media
/// content-type). GCS sends the JSON metadata part first, the media part second.
fn parse_multipart(body: &[u8], boundary: &str) -> (Value, Vec<u8>, String) {
    let marker = format!("--{boundary}");
    let parts = split_on(body, marker.as_bytes());
    let mut bodies: Vec<(Vec<u8>, String)> = Vec::new();
    for part in &parts {
        // Skip the preamble and the closing "--" epilogue.
        let trimmed = trim_crlf(part);
        if trimmed.is_empty() || trimmed == b"--" {
            continue;
        }
        // Headers and content are separated by a blank line.
        if let Some(idx) = find(trimmed, b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&trimmed[..idx]).to_lowercase();
            let content = trim_crlf(&trimmed[idx + 4..]).to_vec();
            let ct = head
                .split("content-type:")
                .nth(1)
                .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            bodies.push((content, ct));
        }
    }
    let meta = bodies
        .first()
        .and_then(|(b, _)| serde_json::from_slice(b).ok())
        .unwrap_or(Value::Object(Default::default()));
    let (media, media_ct) = bodies
        .get(1)
        .cloned()
        .unwrap_or((Vec::new(), "application/octet-stream".to_string()));
    (meta, media, media_ct)
}

fn split_on(data: &[u8], sep: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + sep.len() <= data.len() {
        if &data[i..i + sep.len()] == sep {
            out.push(data[start..i].to_vec());
            i += sep.len();
            start = i;
        } else {
            i += 1;
        }
    }
    out.push(data[start..].to_vec());
    out
}

fn find(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).position(|w| w == needle)
}

fn trim_crlf(data: &[u8]) -> &[u8] {
    let mut s = data;
    while s.first() == Some(&b'\r') || s.first() == Some(&b'\n') {
        s = &s[1..];
    }
    while s.last() == Some(&b'\r') || s.last() == Some(&b'\n') {
        s = &s[..s.len() - 1];
    }
    s
}

fn not_found(what: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": { "code": 404, "message": format!("{what} not found") } })),
    )
        .into_response()
}

fn bad_request(msg: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": { "code": 400, "message": msg } })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipart_parses_metadata_and_media() {
        let boundary = "abc";
        let body = format!(
            "--{b}\r\nContent-Type: application/json\r\n\r\n{{\"name\":\"f.txt\"}}\r\n--{b}\r\nContent-Type: text/plain\r\n\r\nhello body\r\n--{b}--",
            b = boundary
        );
        let (meta, media, ct) = parse_multipart(body.as_bytes(), boundary);
        assert_eq!(meta.get("name").and_then(Value::as_str), Some("f.txt"));
        assert_eq!(media, b"hello body");
        assert_eq!(ct, "text/plain");
    }

    #[test]
    fn md5_and_decode() {
        assert_eq!(decode("a%2Fb.txt"), "a/b.txt");
        // md5("hello") base64
        assert_eq!(md5_base64(b"hello"), "XUFAKrxLKna5cZ2REBfFkg==");
    }
}
// CODEGEN-END

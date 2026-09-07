use std::net::SocketAddr;
use anyhow::Result;
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::Value;

use super::handle_message;
use crate::store::PmStore;

pub fn router(store: PmStore) -> Router {
    Router::new()
        .route("/", post(handle_jsonrpc).get(handle_health).options(handle_options))
        .route("/mcp", post(handle_jsonrpc).get(handle_health).options(handle_options))
        .with_state(store)
}

async fn handle_health() -> &'static str {
    "pm MCP server running\n"
}

async fn handle_options() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("POST, GET, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
    (StatusCode::NO_CONTENT, headers)
}

async fn handle_jsonrpc(
    State(store): State<PmStore>,
    Json(msg): Json<Value>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("POST, GET, OPTIONS"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));

    if let Some(arr) = msg.as_array() {
        let mut responses = Vec::new();
        for item in arr {
            if let Some(resp) = handle_message(&store, item).await {
                responses.push(resp);
            }
        }
        if responses.is_empty() {
            (StatusCode::NO_CONTENT, headers, Json(Value::Null)).into_response()
        } else {
            (StatusCode::OK, headers, Json(Value::Array(responses))).into_response()
        }
    } else if let Some(resp) = handle_message(&store, &msg).await {
        (StatusCode::OK, headers, Json(resp)).into_response()
    } else {
        (StatusCode::NO_CONTENT, headers, Json(Value::Null)).into_response()
    }
}

pub async fn serve_http(store: PmStore, addr: SocketAddr) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!(
        "pm: HTTP MCP server listening on http://{} (store: {})",
        listener.local_addr()?,
        store.path().display()
    );
    axum::serve(listener, router(store)).await?;
    Ok(())
}

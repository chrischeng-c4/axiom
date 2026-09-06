use std::net::SocketAddr;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde_json::Value;

use super::handle_message;
use crate::store::PmStore;

pub fn router(store: PmStore) -> Router {
    Router::new()
        .route("/", post(handle_jsonrpc).get(handle_health))
        .route("/mcp", post(handle_jsonrpc).get(handle_health))
        .with_state(store)
}

async fn handle_health() -> &'static str {
    "pm MCP server running\n"
}

async fn handle_jsonrpc(
    State(store): State<PmStore>,
    Json(msg): Json<Value>,
) -> impl IntoResponse {
    if let Some(resp) = handle_message(&store, &msg).await {
        (StatusCode::OK, Json(resp)).into_response()
    } else {
        StatusCode::NO_CONTENT.into_response()
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

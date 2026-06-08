//! Unified LSP router for multi-project support
//!
//! Routes LSP requests over TCP to the appropriate RequestHandler based on rootUri.
//! Each project maintains its own in-process RequestHandler instance.

use crate::lens_pool::LensHandlerPool;
use crate::registry::Registry;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;

/// Unified LSP router that handles multi-project LSP over TCP
pub struct UnifiedLspRouter {
    /// Pool of project handlers
    lens_pool: Arc<LensHandlerPool>,
    /// Project registry
    registry: Arc<tokio::sync::RwLock<Registry>>,
    /// LSP server socket address
    pub addr: SocketAddr,
}


impl UnifiedLspRouter {
    /// Create a new unified LSP router
    pub fn new(
        addr: SocketAddr,
        lens_pool: Arc<LensHandlerPool>,
        registry: Registry,
    ) -> Self {
        Self {
            lens_pool,
            registry: Arc::new(tokio::sync::RwLock::new(registry)),
            addr,
        }
    }

    /// Start the LSP server and listen for connections
    pub async fn start(&self) -> Result<JoinHandle<Result<(), String>>, String> {
        // Use tokio's TcpListener directly to avoid blocking socket issues
        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .map_err(|e| format!("Failed to bind LSP server to {}: {}", self.addr, e))?;

        let lens_pool = self.lens_pool.clone();
        let registry = self.registry.clone();

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        eprintln!("LSP client connected from {}", peer_addr);
                        let pool = lens_pool.clone();
                        let reg = registry.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_client(stream, pool, reg).await {
                                eprintln!("Error handling LSP client {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        return Err(format!("Failed to accept connection: {}", e));
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Handle an individual LSP client connection
    async fn handle_client(
        stream: TcpStream,
        lens_pool: Arc<LensHandlerPool>,
        _registry: Arc<tokio::sync::RwLock<Registry>>,
    ) -> Result<(), String> {
        let (mut reader, mut writer) = stream.into_split();
        let mut project_path: Option<PathBuf> = None;
        let mut buffer = vec![0; 8192];

        loop {
            // Read LSP message - simplified approach
            match reader.read(&mut buffer).await {
                Ok(0) => return Ok(()), // Connection closed
                Ok(n) => {
                    let data = &buffer[..n];

                    // Very basic LSP message parsing - look for JSON in the data
                    if let Ok(text) = std::str::from_utf8(data) {
                        // Skip headers and find the JSON part
                        if let Some(json_start) = text.rfind('{') {
                            if let Some(json_end) = text.rfind('}') {
                                if json_start < json_end {
                                    let json_part = &text[json_start..=json_end];
                                    if let Ok(msg) = serde_json::from_str::<Value>(json_part) {
                                        let method = msg["method"].as_str().unwrap_or("");
                                        let id = msg.get("id").cloned();
                                        let params = msg.get("params").cloned().unwrap_or(Value::Null);

                                        // Handle initialize request
                                        if method == "initialize" {
                                            if let Some(root_uri) = params.get("rootUri").and_then(|u| u.as_str()) {
                                                project_path = Self::uri_to_path(root_uri);
                                                eprintln!("LSP initialized for project: {:?}", project_path);
                                            }

                                            let response = json!({
                                                "jsonrpc": "2.0",
                                                "id": id,
                                                "result": {
                                                    "capabilities": {
                                                        "textDocumentSync": 1,
                                                        "hoverProvider": true,
                                                        "definitionProvider": true
                                                    }
                                                }
                                            });
                                            Self::send_response_direct(&mut writer, &response).await?;
                                        }
                                        // Handle document open
                                        else if method == "textDocument/didOpen" {
                                            if let Some(path) = &project_path {
                                                if let Some(doc) = params.get("textDocument") {
                                                    if let (Some(uri), Some(text)) = (
                                                        doc.get("uri").and_then(|u| u.as_str()),
                                                        doc.get("text").and_then(|t| t.as_str()),
                                                    ) {
                                                        if let Some(doc_path) = Self::uri_to_path(uri) {
                                                            if let Ok(handler) = lens_pool.get_handler(path).await {
                                                                handler.set_document_override(&doc_path, text.to_string()).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Handle document change
                                        else if method == "textDocument/didChange" {
                                            if let Some(path) = &project_path {
                                                if let Some(doc) = params.get("textDocument") {
                                                    if let Some(uri) = doc.get("uri").and_then(|u| u.as_str()) {
                                                        if let Some(doc_path) = Self::uri_to_path(uri) {
                                                            if let Some(changes) = params.get("contentChanges").and_then(|c| c.as_array()) {
                                                                if let Some(last_change) = changes.last() {
                                                                    if let Some(text) = last_change.get("text").and_then(|t| t.as_str()) {
                                                                        if let Ok(handler) = lens_pool.get_handler(path).await {
                                                                            handler.set_document_override(&doc_path, text.to_string()).await;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Handle document close
                                        else if method == "textDocument/didClose" {
                                            if let Some(path) = &project_path {
                                                if let Some(doc) = params.get("textDocument") {
                                                    if let Some(uri) = doc.get("uri").and_then(|u| u.as_str()) {
                                                        if let Some(doc_path) = Self::uri_to_path(uri) {
                                                            if let Ok(handler) = lens_pool.get_handler(path).await {
                                                                handler.remove_document_override(&doc_path).await;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Handle shutdown
                                        else if method == "shutdown" {
                                            let response = json!({"jsonrpc": "2.0", "id": id, "result": null});
                                            Self::send_response_direct(&mut writer, &response).await?;
                                            return Ok(());
                                        }
                                        // Handle exit
                                        else if method == "exit" {
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }
    }

    /// Convert URI to file path
    fn uri_to_path(uri: &str) -> Option<PathBuf> {
        // Handle file:// URIs
        if let Some(path_str) = uri.strip_prefix("file://") {
            let path = if cfg!(windows) {
                // On Windows, remove leading / from /C:/path/to/file
                if path_str.starts_with('/') && path_str.len() > 2 && path_str.chars().nth(2) == Some(':') {
                    &path_str[1..]
                } else {
                    path_str
                }
            } else {
                path_str
            };
            Some(PathBuf::from(path))
        } else {
            Some(PathBuf::from(uri))
        }
    }

    /// Send LSP response with Content-Length header
    async fn send_response_direct(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        response: &Value,
    ) -> Result<(), String> {
        let body = serde_json::to_string(response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;
        let content_length = body.len();

        let message = format!("Content-Length: {}\r\n\r\n{}", content_length, body);
        writer.write_all(message.as_bytes()).await
            .map_err(|e| format!("Failed to write response: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let addr: SocketAddr = "127.0.0.1:5007".parse().expect("Failed to parse address");
        let pool = Arc::new(LensHandlerPool::new());
        let registry = Registry::new(0, 5007);
        let router = UnifiedLspRouter::new(addr, pool, registry);
        assert_eq!(router.addr, addr);
    }
}

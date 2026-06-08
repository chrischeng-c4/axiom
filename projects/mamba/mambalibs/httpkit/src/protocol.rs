//! Native protocol model for httpkit host implementations.
//!
//! Wire protocols such as HTTP/1.1 and HTTP/2 are normalized into these
//! structures before dispatch reaches `App` routes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpProtocol {
    Http1,
    Http2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyMode {
    Empty,
    Buffered,
    Streaming,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeRequestHead {
    pub protocol: HttpProtocol,
    pub method: String,
    pub path: String,
    pub query_string: Option<String>,
    pub headers: HashMap<String, String>,
}

impl NativeRequestHead {
    pub fn new(protocol: HttpProtocol, method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            protocol,
            method: method.into(),
            path: path.into(),
            query_string: None,
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeResponseHead {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body_mode: BodyMode,
}

impl NativeResponseHead {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body_mode: BodyMode::Empty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolLimits {
    pub max_header_bytes: usize,
    pub max_body_bytes: usize,
    pub stream_window_bytes: usize,
}

impl Default for ProtocolLimits {
    fn default() -> Self {
        Self {
            max_header_bytes: 64 * 1024,
            max_body_bytes: 16 * 1024 * 1024,
            stream_window_bytes: 1024 * 1024,
        }
    }
}

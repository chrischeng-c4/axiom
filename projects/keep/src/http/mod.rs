//! HTTP/2 + OpenAPI surface for keep.
//!
//! The transport is axum over hyper (HTTP/1.1 and HTTP/2 on the same port).
//! The engine and persistence layers are transport-agnostic; everything here
//! is a thin, typed adapter onto [`crate::engine::KvEngine`].

pub mod error;
pub mod handlers;
pub mod hash;
pub mod meta;
pub mod models;
pub mod openapi;
pub mod routes;
pub mod sets;
pub mod zsets;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::engine::KvEngine;

/// Default request-body limit (16 MiB) — bounds claim-check blob writes.
pub const DEFAULT_BODY_LIMIT: usize = 16 * 1024 * 1024;

/// Shared state behind every handler.
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<KvEngine>,
    /// Max request body size, applied as a router layer.
    pub body_limit: usize,
    draining: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(engine: Arc<KvEngine>) -> Self {
        Self {
            engine,
            body_limit: DEFAULT_BODY_LIMIT,
            draining: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_body_limit(mut self, body_limit: usize) -> Self {
        self.body_limit = body_limit;
        self
    }

    /// Flip readiness to draining so `/readyz` returns 503. Called on SIGTERM.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

pub use openapi::ApiDoc;
pub use routes::router;

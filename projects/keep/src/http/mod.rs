//! HTTP/2 + OpenAPI surface for keep.
//!
//! The transport is axum over hyper (HTTP/1.1 and HTTP/2 on the same port).
//! The engine and persistence layers are transport-agnostic; everything here
//! is a thin, typed adapter onto [`crate::engine::KvEngine`].

pub mod error;
pub mod handlers;
pub mod hash;
pub mod lists;
pub mod meta;
pub mod metrics;
pub mod models;
pub mod openapi;
pub mod routes;
pub mod sets;
pub mod waiters;
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
    /// Per-route HTTP request metrics (counts + latency histograms).
    pub metrics: Arc<metrics::HttpMetrics>,
    /// Per-key wait registry for blocking list pops (BLPOP/BRPOP).
    pub waiters: Arc<waiters::ListWaiters>,
    /// Cluster topology / sharding (single-node by default).
    pub cluster: crate::cluster::Cluster,
    /// Optional HMAC secret for scoped claim-check tokens (#446). When set, worker
    /// ops (GET input / PUT result) require a valid in-scope token; when `None`,
    /// claim-check is open (backward compatible).
    pub token_secret: Option<Arc<Vec<u8>>>,
    draining: Arc<AtomicBool>,
}

impl AppState {
    pub fn new(engine: Arc<KvEngine>) -> Self {
        Self {
            engine,
            body_limit: DEFAULT_BODY_LIMIT,
            metrics: Arc::new(metrics::HttpMetrics::default()),
            waiters: Arc::new(waiters::ListWaiters::default()),
            cluster: Arc::new(crate::cluster::ClusterConfig::default()),
            token_secret: None,
            draining: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_body_limit(mut self, body_limit: usize) -> Self {
        self.body_limit = body_limit;
        self
    }

    /// Enable scoped claim-check token enforcement with `secret` (#446).
    pub fn with_token_secret(mut self, secret: Vec<u8>) -> Self {
        self.token_secret = Some(Arc::new(secret));
        self
    }

    pub fn with_cluster(mut self, cluster: crate::cluster::ClusterConfig) -> Self {
        self.cluster = Arc::new(cluster);
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

/// Readiness source for the shared probe router (`service_http`): `/readyz`
/// reports 503 once SIGTERM flips `start_drain`.
impl service_http::ReadinessHook for AppState {
    fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

/// Prometheus exposition for the shared `/metrics` route: engine gauges plus the
/// per-route HTTP request metrics. This is the body the local `handlers::metrics`
/// produced before keep adopted the shared probe router.
impl service_http::MetricsProvider for AppState {
    fn render_metrics(&self) -> String {
        let keys = self.engine.len();
        let shards = self.engine.num_shards();
        let mem = self.engine.estimate_memory();
        format!(
            "# HELP keep_keys_total Number of keys across all shards.\n\
             # TYPE keep_keys_total gauge\n\
             keep_keys_total {keys}\n\
             # HELP keep_shards Number of engine shards.\n\
             # TYPE keep_shards gauge\n\
             keep_shards {shards}\n\
             # HELP keep_memory_bytes Estimated resident bytes of stored data.\n\
             # TYPE keep_memory_bytes gauge\n\
             keep_memory_bytes {mem}\n{}",
            self.metrics.render()
        )
    }
}

pub use openapi::ApiDoc;
pub use routes::router;

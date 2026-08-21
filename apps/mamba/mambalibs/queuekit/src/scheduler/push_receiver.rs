//! Push receiver HTTP endpoint
//!
//! Shared HTTP endpoint that receives scheduled trigger callbacks from both
//! Cloud Scheduler (GCP) and K8s CronJob pods. Mounted as axum `Router`
//! routes on the existing cclab server at `/scheduler/push/:task_name`.
//!
//! Authentication is auto-detected per-request:
//! - `Authorization: Bearer <token>` → OIDC JWT validation
//! - `X-Scheduler-Signature: sha256=<hex>` → HMAC-SHA256 validation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
#[cfg(feature = "metrics")]
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::broker::Broker;
use crate::{TaskError, TaskMessage};

use super::push_auth::{HmacValidator, OidcValidator};
#[cfg(feature = "scheduler")]
use super::schedule_monitor::ScheduleMonitor;

/// Header name for HMAC signature from K8s CronJob pods
const HMAC_SIGNATURE_HEADER: &str = "x-scheduler-signature";

/// Default OIDC issuer (Google)
const DEFAULT_OIDC_ISSUER: &str = "https://accounts.google.com";

/// Default Google JWKS endpoint
const DEFAULT_OIDC_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

/// Default JWKS cache TTL (1 hour)
const DEFAULT_JWKS_CACHE_TTL_SECS: u64 = 3600;

/// Default max request body size (1 MiB)
const DEFAULT_MAX_BODY_SIZE: usize = 1_048_576;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the push receiver HTTP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushReceiverConfig {
    /// Expected audience claim in OIDC JWT tokens from Cloud Scheduler
    #[serde(default)]
    pub oidc_audience: Option<String>,
    /// Expected issuer claim in OIDC JWT tokens
    #[serde(default = "default_oidc_issuer")]
    pub oidc_issuer: String,
    /// URL to fetch Google JWKS public keys for JWT verification
    #[serde(default = "default_oidc_jwks_url")]
    pub oidc_jwks_url: String,
    /// TTL in seconds for cached JWKS public keys
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub oidc_jwks_cache_ttl_secs: u64,
    /// Shared HMAC-SHA256 secret for K8s CronJob request validation
    #[serde(default)]
    pub hmac_secret: Option<String>,
    /// Which authentication methods are accepted
    #[serde(default = "default_enabled_auth_methods")]
    pub enabled_auth_methods: Vec<AuthMethod>,
    /// Mapping of task_name to target queue name
    #[serde(default)]
    pub task_queue_map: HashMap<String, String>,
    /// Fallback queue when task_name is not in task_queue_map.
    /// If None, unknown tasks return 404.
    #[serde(default = "default_queue")]
    pub default_queue: Option<String>,
    /// Maximum request body size in bytes (default 1 MiB)
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

fn default_oidc_issuer() -> String {
    DEFAULT_OIDC_ISSUER.to_string()
}

fn default_oidc_jwks_url() -> String {
    DEFAULT_OIDC_JWKS_URL.to_string()
}

fn default_jwks_cache_ttl_secs() -> u64 {
    DEFAULT_JWKS_CACHE_TTL_SECS
}

fn default_enabled_auth_methods() -> Vec<AuthMethod> {
    vec![AuthMethod::Oidc, AuthMethod::Hmac]
}

fn default_queue() -> Option<String> {
    Some("default".to_string())
}

fn default_max_body_size() -> usize {
    DEFAULT_MAX_BODY_SIZE
}

impl Default for PushReceiverConfig {
    fn default() -> Self {
        Self {
            oidc_audience: None,
            oidc_issuer: default_oidc_issuer(),
            oidc_jwks_url: default_oidc_jwks_url(),
            oidc_jwks_cache_ttl_secs: default_jwks_cache_ttl_secs(),
            hmac_secret: None,
            enabled_auth_methods: default_enabled_auth_methods(),
            task_queue_map: HashMap::new(),
            default_queue: default_queue(),
            max_body_size: default_max_body_size(),
        }
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Authentication method detected from request headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// OIDC JWT bearer token (Cloud Scheduler)
    Oidc,
    /// HMAC-SHA256 signature (K8s CronJob)
    Hmac,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Oidc => write!(f, "oidc"),
            AuthMethod::Hmac => write!(f, "hmac"),
        }
    }
}

/// Structured error response from push receiver endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct PushErrorResponse {
    pub error: String,
}

impl PushErrorResponse {
    fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }

    /// Convert into an axum Response with the given status code.
    fn into_resp(self, status: StatusCode) -> Response {
        (status, Json(self)).into_response()
    }
}

/// Authentication handler for push receiver.
/// Holds state for both OIDC and HMAC validation.
pub struct PushAuthenticator {
    /// OIDC JWT validator (None if OIDC auth is disabled)
    oidc_validator: Option<OidcValidator>,
    /// HMAC signature validator (None if HMAC auth is disabled)
    hmac_validator: Option<HmacValidator>,
    /// Which auth methods are enabled
    enabled_methods: Vec<AuthMethod>,
}

// ---------------------------------------------------------------------------
// Push Receiver
// ---------------------------------------------------------------------------

/// HTTP push receiver that handles Cloud Scheduler and K8s CronJob callbacks.
///
/// Produces an axum `Router`. Requires `Send + Sync`.
pub struct PushReceiver {
    config: PushReceiverConfig,
    broker: Arc<dyn Broker>,
    #[cfg(feature = "scheduler")]
    monitor: Option<Arc<ScheduleMonitor>>,
    authenticator: PushAuthenticator,
    #[cfg(feature = "metrics")]
    metrics: PushMetrics,
}

// Ensure Send + Sync for axum handler compatibility
macro_rules! static_assertions_send_sync {
    ($t:ty) => {
        const _: fn() = || {
            fn assert_send<T: Send>() {}
            fn assert_sync<T: Sync>() {}
            assert_send::<$t>();
            assert_sync::<$t>();
        };
    };
}
static_assertions_send_sync!(PushReceiver);

impl PushReceiver {
    /// Construct a new `PushReceiver` with initialized authenticators.
    ///
    /// # Errors
    /// Returns `TaskError::Configuration` if HMAC secret is configured but < 32 bytes.
    pub fn new(
        config: PushReceiverConfig,
        broker: Arc<dyn Broker>,
        #[cfg(feature = "scheduler")] monitor: Option<Arc<ScheduleMonitor>>,
    ) -> Result<Self, TaskError> {
        let authenticator = Self::build_authenticator(&config)?;

        Ok(Self {
            config,
            broker,
            #[cfg(feature = "scheduler")]
            monitor,
            authenticator,
            #[cfg(feature = "metrics")]
            metrics: PushMetrics::shared()?,
        })
    }

    /// Build the authenticator from config.
    fn build_authenticator(config: &PushReceiverConfig) -> Result<PushAuthenticator, TaskError> {
        let oidc_enabled = config.enabled_auth_methods.contains(&AuthMethod::Oidc);
        let hmac_enabled = config.enabled_auth_methods.contains(&AuthMethod::Hmac);

        let oidc_validator = if oidc_enabled {
            config.oidc_audience.as_ref().map(|audience| {
                OidcValidator::new(
                    audience.clone(),
                    config.oidc_issuer.clone(),
                    config.oidc_jwks_url.clone(),
                    Duration::from_secs(config.oidc_jwks_cache_ttl_secs),
                )
            })
        } else {
            None
        };

        let hmac_validator = if hmac_enabled {
            match &config.hmac_secret {
                Some(secret) => Some(HmacValidator::new(secret.as_bytes())?),
                None => None,
            }
        } else {
            None
        };

        Ok(PushAuthenticator {
            oidc_validator,
            hmac_validator,
            enabled_methods: config.enabled_auth_methods.clone(),
        })
    }

    /// Returns an axum `Router` with `POST /scheduler/push/:task_name`.
    ///
    /// The router is mergeable into the existing cclab server via
    /// `app.merge(push_receiver.router())`.
    pub fn router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/scheduler/push/{task_name}", post(Self::handle_push))
            .with_state(self)
    }

    /// Main request handler for push receiver endpoint.
    ///
    /// Flow: extract task_name → authenticate → parse body → record monitor →
    /// publish to broker → return response.
    #[tracing::instrument(
        name = "push_receiver.handle",
        skip(receiver, headers, body),
        fields(task_name, auth_method, status_code)
    )]
    async fn handle_push(
        State(receiver): State<Arc<PushReceiver>>,
        Path(task_name): Path<String>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        let start = std::time::Instant::now();

        // Check body size limit
        if body.len() > receiver.config.max_body_size {
            receiver.record_metrics(&task_name, "unknown", "parse_error", start);
            return PushErrorResponse::new(format!(
                "Request body exceeds maximum size of {} bytes",
                receiver.config.max_body_size
            ))
            .into_resp(StatusCode::PAYLOAD_TOO_LARGE);
        }

        // Authenticate
        let auth_method = match receiver.authenticate(&headers, &body).await {
            Ok(method) => method,
            Err(resp) => {
                receiver.record_metrics(&task_name, "unknown", "auth_failed", start);
                return resp;
            }
        };

        let auth_method_str = auth_method.to_string();
        tracing::Span::current().record("auth_method", auth_method_str.as_str());

        // Resolve queue
        let queue = match receiver.resolve_queue(&task_name) {
            Ok(q) => q,
            Err(resp) => {
                receiver.record_metrics(&task_name, &auth_method_str, "parse_error", start);
                return resp;
            }
        };

        // Parse request body as TaskMessage
        let task_message = match Self::parse_task_message(&body) {
            Ok(msg) => msg,
            Err(resp) => {
                receiver.record_metrics(&task_name, &auth_method_str, "parse_error", start);
                return resp;
            }
        };

        // Record trigger in schedule monitor (best-effort, R8)
        #[cfg(feature = "scheduler")]
        if let Some(monitor) = &receiver.monitor {
            if let Err(e) = monitor.record_trigger(&task_name, chrono::Utc::now()) {
                tracing::warn!(
                    task_name = %task_name,
                    error = %e,
                    "Failed to record trigger in schedule monitor"
                );
            }
        }

        // Publish to broker
        if let Err(e) = receiver.broker.publish(&queue, task_message).await {
            receiver.record_metrics(&task_name, &auth_method_str, "publish_error", start);
            return PushErrorResponse::new(format!("Failed to publish task: {}", e))
                .into_resp(StatusCode::INTERNAL_SERVER_ERROR);
        }

        // Success
        receiver.record_metrics(&task_name, &auth_method_str, "ok", start);
        StatusCode::OK.into_response()
    }

    /// Auto-detect and validate authentication based on request headers.
    ///
    /// - `Authorization: Bearer` → OIDC path
    /// - `X-Scheduler-Signature` → HMAC path
    /// - Both present → OIDC takes precedence
    /// - Neither present → 401
    async fn authenticate(&self, headers: &HeaderMap, body: &[u8]) -> Result<AuthMethod, Response> {
        let has_bearer = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("Bearer "));

        let has_hmac = headers.get(HMAC_SIGNATURE_HEADER).is_some();

        // Determine auth method: Bearer takes precedence (R4)
        if has_bearer
            && self
                .authenticator
                .enabled_methods
                .contains(&AuthMethod::Oidc)
        {
            let token = headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .ok_or_else(|| {
                    PushErrorResponse::new("Invalid Authorization header format")
                        .into_resp(StatusCode::UNAUTHORIZED)
                })?;

            let validator = self.authenticator.oidc_validator.as_ref().ok_or_else(|| {
                PushErrorResponse::new("OIDC authentication is enabled but no audience configured")
                    .into_resp(StatusCode::UNAUTHORIZED)
            })?;

            validator.validate_token(token).await.map_err(|e| {
                PushErrorResponse::new(format!("OIDC token validation failed: {}", e))
                    .into_resp(StatusCode::UNAUTHORIZED)
            })?;

            return Ok(AuthMethod::Oidc);
        }

        if has_hmac
            && self
                .authenticator
                .enabled_methods
                .contains(&AuthMethod::Hmac)
        {
            let signature = headers
                .get(HMAC_SIGNATURE_HEADER)
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    PushErrorResponse::new("Invalid X-Scheduler-Signature header")
                        .into_resp(StatusCode::UNAUTHORIZED)
                })?;

            let validator = self.authenticator.hmac_validator.as_ref().ok_or_else(|| {
                PushErrorResponse::new("HMAC authentication is enabled but no secret configured")
                    .into_resp(StatusCode::UNAUTHORIZED)
            })?;

            validator.validate_signature(body, signature).map_err(|_| {
                PushErrorResponse::new("HMAC signature validation failed")
                    .into_resp(StatusCode::UNAUTHORIZED)
            })?;

            return Ok(AuthMethod::Hmac);
        }

        Err(
            PushErrorResponse::new("No authentication credentials provided")
                .into_resp(StatusCode::UNAUTHORIZED),
        )
    }

    /// Look up target queue from task_name.
    ///
    /// Uses `task_queue_map` for explicit routing, falls back to `default_queue`.
    /// Returns 404 if task_name is unknown and no default_queue is configured.
    fn resolve_queue(&self, task_name: &str) -> Result<String, Response> {
        if let Some(queue) = self.config.task_queue_map.get(task_name) {
            return Ok(queue.clone());
        }

        if let Some(default) = &self.config.default_queue {
            return Ok(default.clone());
        }

        Err(
            PushErrorResponse::new(format!("Unknown task: {}", task_name))
                .into_resp(StatusCode::NOT_FOUND),
        )
    }

    /// Deserialize request body as `TaskMessage`.
    ///
    /// Handles both direct JSON and base64-encoded payloads (from Cloud
    /// Scheduler httpTarget).
    fn parse_task_message(body: &[u8]) -> Result<TaskMessage, Response> {
        // Try direct JSON parse first
        match serde_json::from_slice::<TaskMessage>(body) {
            Ok(msg) => Ok(msg),
            Err(direct_err) => {
                // Try base64 decode then JSON parse (Cloud Scheduler httpTarget)
                if let Ok(decoded) =
                    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, body)
                {
                    if let Ok(msg) = serde_json::from_slice::<TaskMessage>(&decoded) {
                        return Ok(msg);
                    }
                }

                // Both attempts failed — report the direct parse error
                Err(
                    PushErrorResponse::new(format!("Failed to parse TaskMessage: {}", direct_err))
                        .into_resp(StatusCode::BAD_REQUEST),
                )
            }
        }
    }

    /// Record Prometheus metrics for a request.
    fn record_metrics(
        &self,
        _task_name: &str,
        _auth_method: &str,
        _status: &str,
        _start: std::time::Instant,
    ) {
        #[cfg(feature = "metrics")]
        {
            self.metrics
                .received_total
                .with_label_values(&[_task_name, _auth_method, _status])
                .inc();
            self.metrics
                .duration_seconds
                .with_label_values(&[_task_name])
                .observe(_start.elapsed().as_secs_f64());
        }
    }
}

// ---------------------------------------------------------------------------
// Prometheus metrics (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "metrics")]
#[derive(Clone)]
struct PushMetrics {
    /// Total push requests received
    received_total: prometheus::CounterVec,
    /// Request processing latency in seconds
    duration_seconds: prometheus::HistogramVec,
}

#[cfg(feature = "metrics")]
static PUSH_METRICS: Lazy<Result<PushMetrics, String>> =
    Lazy::new(|| PushMetrics::new().map_err(|e| e.to_string()));

#[cfg(feature = "metrics")]
impl PushMetrics {
    fn shared() -> Result<Self, TaskError> {
        match &*PUSH_METRICS {
            Ok(metrics) => Ok(metrics.clone()),
            Err(err) => Err(TaskError::Configuration(err.clone())),
        }
    }

    fn new() -> Result<Self, TaskError> {
        let received_total = prometheus::register_counter_vec!(
            prometheus::Opts::new(
                "scheduler_push_received_total",
                "Total push requests received by the scheduler push receiver"
            ),
            &["task_name", "auth_method", "status"]
        )
        .map_err(|e| TaskError::Configuration(format!("Failed to register push metrics: {}", e)))?;

        let duration_seconds = prometheus::register_histogram_vec!(
            prometheus::HistogramOpts::new(
                "scheduler_push_duration_seconds",
                "Push receiver request processing latency in seconds"
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,]),
            &["task_name"]
        )
        .map_err(|e| TaskError::Configuration(format!("Failed to register push metrics: {}", e)))?;

        Ok(Self {
            received_total,
            duration_seconds,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};

    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use crate::broker::{BrokerCapabilities, DeliveryModel};
    use crate::TaskMessage;

    // -----------------------------------------------------------------------
    // Mock Broker
    // -----------------------------------------------------------------------

    /// Mock broker that records publish calls and can be configured to fail.
    struct MockBroker {
        published: tokio::sync::Mutex<Vec<(String, TaskMessage)>>,
        fail_publish: AtomicBool,
    }

    impl MockBroker {
        fn new() -> Self {
            Self {
                published: tokio::sync::Mutex::new(Vec::new()),
                fail_publish: AtomicBool::new(false),
            }
        }

        fn failing() -> Self {
            Self {
                published: tokio::sync::Mutex::new(Vec::new()),
                fail_publish: AtomicBool::new(true),
            }
        }

        async fn published_messages(&self) -> Vec<(String, TaskMessage)> {
            self.published.lock().await.clone()
        }
    }

    #[async_trait]
    impl Broker for MockBroker {
        async fn connect(&self) -> Result<(), TaskError> {
            Ok(())
        }
        async fn disconnect(&self) -> Result<(), TaskError> {
            Ok(())
        }
        async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
            if self.fail_publish.load(Ordering::SeqCst) {
                return Err(TaskError::Connection(
                    "mock broker connection error".to_string(),
                ));
            }
            self.published
                .lock()
                .await
                .push((queue.to_string(), message));
            Ok(())
        }
        async fn health_check(&self) -> Result<(), TaskError> {
            Ok(())
        }
        fn delivery_model(&self) -> DeliveryModel {
            DeliveryModel::Push
        }
        fn capabilities(&self) -> BrokerCapabilities {
            BrokerCapabilities::default()
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// 32-byte HMAC secret for tests
    const TEST_HMAC_SECRET: &str = "this-is-a-32-byte-hmac-secret!!x";

    fn test_config() -> PushReceiverConfig {
        PushReceiverConfig {
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Hmac],
            default_queue: Some("default".to_string()),
            ..Default::default()
        }
    }

    fn test_config_both_auth() -> PushReceiverConfig {
        PushReceiverConfig {
            oidc_audience: Some("https://app.example.com".to_string()),
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Oidc, AuthMethod::Hmac],
            default_queue: Some("default".to_string()),
            ..Default::default()
        }
    }

    fn make_receiver(config: PushReceiverConfig) -> Arc<PushReceiver> {
        let broker = Arc::new(MockBroker::new()) as Arc<dyn Broker>;
        Arc::new(
            PushReceiver::new(
                config,
                broker,
                #[cfg(feature = "scheduler")]
                None,
            )
            .unwrap(),
        )
    }

    fn make_receiver_with_broker(
        config: PushReceiverConfig,
        broker: Arc<dyn Broker>,
    ) -> Arc<PushReceiver> {
        Arc::new(
            PushReceiver::new(
                config,
                broker,
                #[cfg(feature = "scheduler")]
                None,
            )
            .unwrap(),
        )
    }

    fn make_task_message_body(task_name: &str) -> Vec<u8> {
        let msg = TaskMessage::new(task_name, serde_json::json!(["arg1"]));
        serde_json::to_vec(&msg).unwrap()
    }

    fn compute_hmac_signature(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let result = mac.finalize();
        let bytes = result.into_bytes();
        let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        format!("sha256={hex}")
    }

    /// Build a POST request to /scheduler/push/{task_name} with HMAC auth
    fn hmac_request(task_name: &str, body: &[u8]) -> Request<Body> {
        let signature = compute_hmac_signature(TEST_HMAC_SECRET, body);
        Request::builder()
            .method("POST")
            .uri(format!("/scheduler/push/{task_name}"))
            .header("content-type", "application/json")
            .header("x-scheduler-signature", signature)
            .body(Body::from(body.to_vec()))
            .unwrap()
    }

    /// Read response body as bytes
    async fn body_bytes(resp: Response) -> Vec<u8> {
        axum::body::to_bytes(resp.into_body(), 1_048_576)
            .await
            .unwrap()
            .to_vec()
    }

    // -----------------------------------------------------------------------
    // PushReceiverConfig defaults
    // -----------------------------------------------------------------------

    #[test]
    fn config_defaults() {
        let config = PushReceiverConfig::default();
        assert!(config.oidc_audience.is_none());
        assert_eq!(config.oidc_issuer, "https://accounts.google.com");
        assert_eq!(
            config.oidc_jwks_url,
            "https://www.googleapis.com/oauth2/v3/certs"
        );
        assert_eq!(config.oidc_jwks_cache_ttl_secs, 3600);
        assert!(config.hmac_secret.is_none());
        assert_eq!(
            config.enabled_auth_methods,
            vec![AuthMethod::Oidc, AuthMethod::Hmac]
        );
        assert!(config.task_queue_map.is_empty());
        assert_eq!(config.default_queue, Some("default".to_string()));
        assert_eq!(config.max_body_size, 1_048_576);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = PushReceiverConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: PushReceiverConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.oidc_issuer, config.oidc_issuer);
        assert_eq!(parsed.max_body_size, config.max_body_size);
    }

    // -----------------------------------------------------------------------
    // AuthMethod
    // -----------------------------------------------------------------------

    #[test]
    fn auth_method_display() {
        assert_eq!(AuthMethod::Oidc.to_string(), "oidc");
        assert_eq!(AuthMethod::Hmac.to_string(), "hmac");
    }

    #[test]
    fn auth_method_serde_roundtrip() {
        let oidc_json = serde_json::to_string(&AuthMethod::Oidc).unwrap();
        assert_eq!(oidc_json, "\"oidc\"");
        let hmac_json = serde_json::to_string(&AuthMethod::Hmac).unwrap();
        assert_eq!(hmac_json, "\"hmac\"");

        let parsed: AuthMethod = serde_json::from_str(&oidc_json).unwrap();
        assert_eq!(parsed, AuthMethod::Oidc);
    }

    // -----------------------------------------------------------------------
    // PushErrorResponse
    // -----------------------------------------------------------------------

    #[test]
    fn error_response_serialization() {
        let resp = PushErrorResponse::new("something went wrong");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["error"], "something went wrong");
    }

    // -----------------------------------------------------------------------
    // PushReceiver::new
    // -----------------------------------------------------------------------

    #[test]
    fn new_with_valid_hmac_config() {
        let config = test_config();
        let broker = Arc::new(MockBroker::new()) as Arc<dyn Broker>;
        let result = PushReceiver::new(
            config,
            broker,
            #[cfg(feature = "scheduler")]
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn new_with_short_hmac_secret_fails() {
        let config = PushReceiverConfig {
            hmac_secret: Some("short".to_string()), // < 32 bytes
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let broker = Arc::new(MockBroker::new()) as Arc<dyn Broker>;
        let result = PushReceiver::new(
            config,
            broker,
            #[cfg(feature = "scheduler")]
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn new_with_no_hmac_secret_when_hmac_enabled() {
        // HMAC enabled but no secret configured — authenticator has no
        // hmac_validator, which results in 401 at runtime, not construction error.
        let config = PushReceiverConfig {
            hmac_secret: None,
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let broker = Arc::new(MockBroker::new()) as Arc<dyn Broker>;
        let result = PushReceiver::new(
            config,
            broker,
            #[cfg(feature = "scheduler")]
            None,
        );
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // S7: resolve_queue — task name routing
    // -----------------------------------------------------------------------

    #[test]
    fn s7_resolve_queue_mapped_task() {
        let mut config = test_config();
        config
            .task_queue_map
            .insert("daily-cleanup".to_string(), "maintenance".to_string());
        config
            .task_queue_map
            .insert("hourly-sync".to_string(), "sync".to_string());
        let receiver = make_receiver(config);

        assert_eq!(
            receiver.resolve_queue("daily-cleanup").unwrap(),
            "maintenance"
        );
        assert_eq!(receiver.resolve_queue("hourly-sync").unwrap(), "sync");
    }

    #[test]
    fn s7_resolve_queue_fallback_to_default() {
        let config = PushReceiverConfig {
            default_queue: Some("default".to_string()),
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let receiver = make_receiver(config);

        assert_eq!(receiver.resolve_queue("unknown-task").unwrap(), "default");
    }

    // S8: Unknown task with no default queue returns 404
    #[test]
    fn s8_resolve_queue_no_default_returns_error() {
        let config = PushReceiverConfig {
            default_queue: None,
            task_queue_map: {
                let mut m = HashMap::new();
                m.insert("daily-cleanup".to_string(), "maintenance".to_string());
                m
            },
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let receiver = make_receiver(config);

        // Known task works
        assert!(receiver.resolve_queue("daily-cleanup").is_ok());

        // Unknown task with no default_queue → error (will be 404 in handler)
        assert!(receiver.resolve_queue("unknown-task").is_err());
    }

    // -----------------------------------------------------------------------
    // S9: parse_task_message
    // -----------------------------------------------------------------------

    #[test]
    fn s9_parse_valid_json() {
        let msg = TaskMessage::new("test-task", serde_json::json!(["a"]));
        let body = serde_json::to_vec(&msg).unwrap();

        let parsed = PushReceiver::parse_task_message(&body).unwrap();
        assert_eq!(parsed.task_name, "test-task");
    }

    #[test]
    fn s9_parse_base64_encoded_json() {
        let msg = TaskMessage::new("b64-task", serde_json::json!([]));
        let json_bytes = serde_json::to_vec(&msg).unwrap();

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&json_bytes);

        let parsed = PushReceiver::parse_task_message(encoded.as_bytes()).unwrap();
        assert_eq!(parsed.task_name, "b64-task");
    }

    #[test]
    fn s9_parse_invalid_body_returns_error() {
        let body = b"this is not valid json or base64";
        let result = PushReceiver::parse_task_message(body);
        assert!(result.is_err());
    }

    #[test]
    fn s9_parse_empty_body_returns_error() {
        let result = PushReceiver::parse_task_message(b"");
        assert!(result.is_err());
    }

    #[test]
    fn s9_parse_valid_json_but_wrong_schema() {
        let body = b"{\"not_a_task\": true}";
        let result = PushReceiver::parse_task_message(body);
        // TaskMessage requires `id` and `task_name`, so this should fail
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Router construction (R1)
    // -----------------------------------------------------------------------

    #[test]
    fn r1_router_construction() {
        let receiver = make_receiver(test_config());
        let _router = receiver.router();
        // If this compiles and doesn't panic, the router is constructible
    }

    // -----------------------------------------------------------------------
    // Integration tests via axum router (S2, S4, S5, S7–S10)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn s2_valid_hmac_request_succeeds() {
        let broker = Arc::new(MockBroker::new());
        let broker_clone = broker.clone();
        let receiver = make_receiver_with_broker(test_config(), broker_clone as Arc<dyn Broker>);
        let app = receiver.router();

        let body = make_task_message_body("daily-cleanup");
        let req = hmac_request("daily-cleanup", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify broker.publish was called
        let messages = broker.published_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "default"); // default queue
        assert_eq!(messages[0].1.task_name, "daily-cleanup");
    }

    #[tokio::test]
    async fn s4_invalid_hmac_signature_returns_401() {
        let receiver = make_receiver(test_config());
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let wrong_sig = compute_hmac_signature("wrong-secret-that-is-at-least-32chars!", &body);
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header("x-scheduler-signature", wrong_sig)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .contains("HMAC signature validation failed"),
            "Error body: {:?}",
            error
        );
    }

    // S5: Request with no auth headers is rejected
    #[tokio::test]
    async fn s5_no_auth_headers_returns_401() {
        let receiver = make_receiver(test_config());
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            error["error"].as_str().unwrap(),
            "No authentication credentials provided"
        );
    }

    // S6: Auth method auto-detection — HMAC used when no Bearer header
    #[tokio::test]
    async fn s6_hmac_header_triggers_hmac_auth() {
        let receiver = make_receiver(test_config_both_auth());
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let signature = compute_hmac_signature(TEST_HMAC_SECRET, &body);
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header("x-scheduler-signature", signature)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "HMAC auth should succeed");
    }

    // S6: Both headers present → OIDC takes precedence
    // Since we can't validate a real OIDC token, this will fail OIDC
    // validation (no real JWKS), demonstrating that OIDC is preferred.
    #[tokio::test]
    async fn s6_bearer_takes_precedence_over_hmac() {
        let receiver = make_receiver(test_config_both_auth());
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let hmac_sig = compute_hmac_signature(TEST_HMAC_SECRET, &body);

        // Both auth headers present: Bearer (invalid) + HMAC (valid)
        // Since Bearer takes precedence, OIDC validation is attempted
        // and fails (no real JWKS), so we expect 401 from OIDC path.
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header("authorization", "Bearer fake.jwt.token")
            .header("x-scheduler-signature", hmac_sig)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // The OIDC path is chosen (Bearer takes precedence), but validation
        // fails because we don't have real JWKS → 401
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"].as_str().unwrap().contains("OIDC"),
            "Should fail via OIDC path, not HMAC. Error: {:?}",
            error
        );
    }

    // S7: Task name routing — mapped task goes to configured queue
    #[tokio::test]
    async fn s7_handler_routes_to_configured_queue() {
        let broker = Arc::new(MockBroker::new());
        let mut config = test_config();
        config
            .task_queue_map
            .insert("daily-cleanup".to_string(), "maintenance".to_string());
        let receiver = make_receiver_with_broker(config, broker.clone() as Arc<dyn Broker>);
        let app = receiver.router();

        let body = make_task_message_body("daily-cleanup");
        let req = hmac_request("daily-cleanup", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = broker.published_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].0, "maintenance",
            "Task should be routed to 'maintenance' queue"
        );
    }

    // S7: Unmapped task falls back to default queue
    #[tokio::test]
    async fn s7_handler_falls_back_to_default_queue() {
        let broker = Arc::new(MockBroker::new());
        let receiver = make_receiver_with_broker(test_config(), broker.clone() as Arc<dyn Broker>);
        let app = receiver.router();

        let body = make_task_message_body("unknown-task");
        let req = hmac_request("unknown-task", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = broker.published_messages().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].0, "default",
            "Unknown task should fall back to 'default' queue"
        );
    }

    // S8: Unknown task with no default queue returns 404
    #[tokio::test]
    async fn s8_handler_no_default_queue_returns_404() {
        let config = PushReceiverConfig {
            default_queue: None,
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let receiver = make_receiver(config);
        let app = receiver.router();

        let body = make_task_message_body("unknown-task");
        let req = hmac_request("unknown-task", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            error["error"].as_str().unwrap(),
            "Unknown task: unknown-task"
        );
    }

    // S9: Malformed request body returns 400
    #[tokio::test]
    async fn s9_handler_malformed_body_returns_400() {
        let receiver = make_receiver(test_config());
        let app = receiver.router();

        let body = b"not valid json at all";
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header(
                "x-scheduler-signature",
                compute_hmac_signature(TEST_HMAC_SECRET, body),
            )
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .starts_with("Failed to parse TaskMessage"),
            "Error: {:?}",
            error
        );
    }

    // S10: Broker publish failure returns 500
    #[tokio::test]
    async fn s10_broker_publish_failure_returns_500() {
        let broker = Arc::new(MockBroker::failing());
        let receiver = make_receiver_with_broker(test_config(), broker as Arc<dyn Broker>);
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let req = hmac_request("task1", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .starts_with("Failed to publish task"),
            "Error: {:?}",
            error
        );
    }

    // Body size limit — request exceeding max_body_size returns 413
    #[tokio::test]
    async fn body_exceeding_max_size_returns_413() {
        let config = PushReceiverConfig {
            max_body_size: 64, // Very small limit
            hmac_secret: Some(TEST_HMAC_SECRET.to_string()),
            enabled_auth_methods: vec![AuthMethod::Hmac],
            ..Default::default()
        };
        let receiver = make_receiver(config);
        let app = receiver.router();

        // Create a body larger than 64 bytes
        let big_body = vec![b'x'; 128];
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header(
                "x-scheduler-signature",
                compute_hmac_signature(TEST_HMAC_SECRET, &big_body),
            )
            .body(Body::from(big_body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    // S2: Full HMAC flow with broker publish verification
    #[tokio::test]
    async fn s2_full_hmac_flow_publishes_correct_message() {
        let broker = Arc::new(MockBroker::new());
        let receiver = make_receiver_with_broker(test_config(), broker.clone() as Arc<dyn Broker>);
        let app = receiver.router();

        let msg = TaskMessage::new("hourly-sync", serde_json::json!({"key": "value"}));
        let body = serde_json::to_vec(&msg).unwrap();
        let req = hmac_request("hourly-sync", &body);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = broker.published_messages().await;
        assert_eq!(messages.len(), 1);
        let (queue, published_msg) = &messages[0];
        assert_eq!(queue, "default");
        assert_eq!(published_msg.task_name, "hourly-sync");
        assert_eq!(published_msg.args, serde_json::json!({"key": "value"}));
    }

    // S10: broker.publish is NOT called when auth fails
    #[tokio::test]
    async fn s4_broker_not_called_on_auth_failure() {
        let broker = Arc::new(MockBroker::new());
        let receiver = make_receiver_with_broker(test_config(), broker.clone() as Arc<dyn Broker>);
        let app = receiver.router();

        let body = make_task_message_body("task1");
        // Send with no auth
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let messages = broker.published_messages().await;
        assert!(
            messages.is_empty(),
            "broker.publish should NOT be called on auth failure"
        );
    }

    // Broker not called when body parse fails
    #[tokio::test]
    async fn s9_broker_not_called_on_parse_failure() {
        let broker = Arc::new(MockBroker::new());
        let receiver = make_receiver_with_broker(test_config(), broker.clone() as Arc<dyn Broker>);
        let app = receiver.router();

        let body = b"invalid json";
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header(
                "x-scheduler-signature",
                compute_hmac_signature(TEST_HMAC_SECRET, body),
            )
            .body(Body::from(body.to_vec()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let messages = broker.published_messages().await;
        assert!(
            messages.is_empty(),
            "broker.publish should NOT be called on parse failure"
        );
    }

    // Multiple sequential requests on separate routers
    #[tokio::test]
    async fn multiple_tasks_route_correctly() {
        let broker = Arc::new(MockBroker::new());
        let mut config = test_config();
        config
            .task_queue_map
            .insert("task-a".to_string(), "queue-a".to_string());
        config
            .task_queue_map
            .insert("task-b".to_string(), "queue-b".to_string());

        let receiver = make_receiver_with_broker(config, broker.clone() as Arc<dyn Broker>);

        // First request
        let body_a = make_task_message_body("task-a");
        let req_a = hmac_request("task-a", &body_a);
        let app = receiver.clone().router();
        let resp = app.oneshot(req_a).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Second request
        let body_b = make_task_message_body("task-b");
        let req_b = hmac_request("task-b", &body_b);
        let app = receiver.router();
        let resp = app.oneshot(req_b).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let messages = broker.published_messages().await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].0, "queue-a");
        assert_eq!(messages[1].0, "queue-b");
    }

    // S3: Invalid OIDC token is rejected (OIDC-only config)
    #[tokio::test]
    async fn s3_invalid_oidc_token_returns_401() {
        let config = PushReceiverConfig {
            oidc_audience: Some("https://app.example.com".to_string()),
            enabled_auth_methods: vec![AuthMethod::Oidc],
            ..Default::default()
        };
        let receiver = make_receiver(config);
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header("authorization", "Bearer invalid.jwt.token")
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let bytes = body_bytes(resp).await;
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .contains("OIDC token validation failed"),
            "Error: {:?}",
            error
        );
    }

    // Auth method not enabled — HMAC header ignored when only OIDC is enabled
    #[tokio::test]
    async fn hmac_ignored_when_not_enabled() {
        let config = PushReceiverConfig {
            oidc_audience: Some("https://app.example.com".to_string()),
            enabled_auth_methods: vec![AuthMethod::Oidc], // HMAC not enabled
            ..Default::default()
        };
        let receiver = make_receiver(config);
        let app = receiver.router();

        let body = make_task_message_body("task1");
        let signature = compute_hmac_signature(TEST_HMAC_SECRET, &body);
        let req = Request::builder()
            .method("POST")
            .uri("/scheduler/push/task1")
            .header("content-type", "application/json")
            .header("x-scheduler-signature", signature)
            .body(Body::from(body))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // HMAC is not in enabled_auth_methods, so HMAC header is ignored → 401
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Send+Sync assertion — compile-time check (R1 constraint)
    #[test]
    fn push_receiver_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PushReceiver>();
    }
}

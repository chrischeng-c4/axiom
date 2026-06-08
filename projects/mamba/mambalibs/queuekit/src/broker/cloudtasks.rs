//! Google Cloud Tasks broker implementation
//!
//! Push-based broker that creates Cloud Tasks which call back to worker HTTP endpoint.
//! Communicates with GCP Cloud Tasks REST API v2 for task creation, and parses
//! incoming push requests from Cloud Tasks dispatches.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header as JwtHeader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::{TaskError, TaskMessage};
use super::{Broker, BrokerCapabilities, BrokerMessage, DelayedBroker, DeliveryModel, PushBroker};

/// Cloud Tasks REST API v2 base URL
const CLOUD_TASKS_API_BASE: &str = "https://cloudtasks.googleapis.com/v2";

/// GCP metadata server token endpoint
const METADATA_TOKEN_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

/// Maximum delay for Cloud Tasks (30 days)
const MAX_DELAY_SECS: u64 = 30 * 24 * 60 * 60;

/// Token refresh buffer (5 minutes before expiry)
const TOKEN_REFRESH_BUFFER_SECS: i64 = 300;

/// Google OAuth2 token endpoint for service account key exchange
const GOOGLE_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";

/// OAuth2 scope for Cloud Tasks API
const CLOUD_TASKS_SCOPE: &str = "https://www.googleapis.com/auth/cloud-tasks";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for Cloud Tasks broker
#[derive(Debug, Clone)]
pub struct CloudTasksConfig {
    /// GCP project ID
    pub project_id: String,
    /// GCP location (e.g., "us-central1")
    pub location: String,
    /// Worker endpoint URL for task dispatch
    pub worker_url: String,
    /// Service account email for OIDC authentication
    pub service_account_email: Option<String>,
    /// OIDC audience (defaults to worker_url)
    pub oidc_audience: Option<String>,
    /// Default queue name
    pub default_queue: String,
    /// Request timeout for task execution (15s-1800s)
    pub dispatch_deadline: Duration,
    /// Max retry attempts (None = Cloud Tasks default / unlimited)
    pub max_retry_count: Option<u32>,
    /// Path to service account JSON key file for local dev
    pub credentials_path: Option<String>,
}

impl Default for CloudTasksConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            worker_url: String::new(),
            service_account_email: None,
            oidc_audience: None,
            default_queue: "default".to_string(),
            dispatch_deadline: Duration::from_secs(600), // 10 minutes
            max_retry_count: None,
            credentials_path: None,
        }
    }
}

impl CloudTasksConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self, TaskError> {
        let project_id = std::env::var("GCP_PROJECT_ID")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
            .map_err(|_| {
                TaskError::Configuration(
                    "GCP_PROJECT_ID or GOOGLE_CLOUD_PROJECT not set".into(),
                )
            })?;

        let location = std::env::var("CLOUDTASKS_LOCATION")
            .unwrap_or_else(|_| "us-central1".to_string());

        let worker_url = std::env::var("METEOR_WORKER_URL")
            .map_err(|_| TaskError::Configuration("METEOR_WORKER_URL not set".into()))?;

        let service_account_email = std::env::var("CLOUDTASKS_SERVICE_ACCOUNT").ok();
        let oidc_audience = std::env::var("CLOUDTASKS_OIDC_AUDIENCE").ok();
        let default_queue = std::env::var("CLOUDTASKS_QUEUE")
            .unwrap_or_else(|_| "default".to_string());
        let credentials_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
        let max_retry_count = std::env::var("CLOUDTASKS_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok());

        Ok(Self {
            project_id,
            location,
            worker_url,
            service_account_email,
            oidc_audience,
            default_queue,
            dispatch_deadline: Duration::from_secs(600),
            max_retry_count,
            credentials_path,
        })
    }

    /// Get the queue path for Cloud Tasks API
    fn queue_path(&self, queue: &str) -> String {
        format!(
            "projects/{}/locations/{}/queues/{}",
            self.project_id, self.location, queue
        )
    }

    /// Get the effective OIDC audience
    fn effective_audience(&self) -> &str {
        self.oidc_audience.as_deref().unwrap_or(&self.worker_url)
    }
}

// ---------------------------------------------------------------------------
// OIDC Token Cache
// ---------------------------------------------------------------------------

/// Cached OIDC bearer token with expiry tracking
#[derive(Debug, Clone)]
struct OidcTokenCache {
    /// Current bearer token (None if not yet fetched)
    access_token: Option<String>,
    /// Token expiry timestamp (None if not yet fetched)
    expires_at: Option<DateTime<Utc>>,
}

impl OidcTokenCache {
    fn new() -> Self {
        Self {
            access_token: None,
            expires_at: None,
        }
    }

    /// Check if the cached token is still valid (with 5-minute buffer)
    fn is_valid(&self) -> bool {
        match (&self.access_token, self.expires_at) {
            (Some(_), Some(expires_at)) => {
                Utc::now() < expires_at - chrono::Duration::seconds(TOKEN_REFRESH_BUFFER_SECS)
            }
            _ => false,
        }
    }
}

/// Response from GCP metadata server token endpoint
#[derive(Deserialize)]
struct MetadataTokenResponse {
    access_token: String,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: String,
}

/// Google service account key file format (subset of fields)
#[derive(Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default = "default_token_uri")]
    token_uri: String,
}

fn default_token_uri() -> String {
    GOOGLE_TOKEN_URI.to_string()
}

/// JWT claims for service account token exchange
#[derive(Serialize)]
struct ServiceAccountClaims {
    iss: String,
    sub: String,
    aud: String,
    scope: String,
    iat: i64,
    exp: i64,
}

/// Response from Google OAuth2 token endpoint (SA key exchange)
#[derive(Deserialize)]
struct TokenExchangeResponse {
    access_token: String,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: String,
}

/// Claims from an inbound OIDC JWT (for validation in parse_push_request)
#[derive(Deserialize)]
struct InboundOidcClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    aud: Option<serde_json::Value>,
    #[serde(default)]
    exp: Option<i64>,
}

// ---------------------------------------------------------------------------
// Cloud Tasks API request/response types
// ---------------------------------------------------------------------------

/// Request body for Cloud Tasks v2 tasks.create API
#[derive(Debug, Serialize)]
struct CreateTaskRequest {
    task: CloudTaskBody,
}

/// Retry configuration for Cloud Tasks task
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RetryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_retry_count: Option<u32>,
}

/// Task body within the create request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudTaskBody {
    http_request: CloudTasksHttpRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dispatch_deadline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_config: Option<RetryConfig>,
}

/// HTTP request target configuration for a Cloud Tasks task
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudTasksHttpRequest {
    url: String,
    http_method: String,
    body: String,
    headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oidc_token: Option<OidcTokenField>,
}

/// OIDC token field in the task HTTP request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OidcTokenField {
    service_account_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    audience: Option<String>,
}

/// Response from Cloud Tasks API (subset of fields)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CloudTaskResponse {
    name: Option<String>,
    schedule_time: Option<String>,
    create_time: Option<String>,
    dispatch_count: Option<i32>,
    response_count: Option<i32>,
}

// ---------------------------------------------------------------------------
// CloudTasksBroker
// ---------------------------------------------------------------------------

/// Cloud Tasks broker
///
/// Push-based broker that creates HTTP tasks dispatched by GCP Cloud Tasks.
/// Implements `Broker + PushBroker + DelayedBroker`.
pub struct CloudTasksBroker {
    config: CloudTasksConfig,
    /// HTTP client for Cloud Tasks API calls (initialized on connect)
    client: RwLock<Option<reqwest::Client>>,
    /// Cached OIDC access token for outbound API authentication
    token_cache: Arc<RwLock<OidcTokenCache>>,
}

impl CloudTasksBroker {
    /// Create a new Cloud Tasks broker
    pub fn new(config: CloudTasksConfig) -> Self {
        Self {
            config,
            client: RwLock::new(None),
            token_cache: Arc::new(RwLock::new(OidcTokenCache::new())),
        }
    }

    /// Build the task URL for a queue
    fn task_url(&self, queue: &str) -> String {
        format!("{}/meteor/push/{}", self.config.worker_url, queue)
    }

    /// Get OIDC bearer token, using cache when valid.
    ///
    /// When `credentials_path` is set (local dev), reads the service account JSON key file,
    /// creates a signed JWT assertion, and exchanges it at Google's token endpoint.
    /// Otherwise (production/GCP environment), fetches from the GCP metadata server.
    async fn get_oidc_token(&self) -> Result<String, TaskError> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if cache.is_valid() {
                return Ok(cache.access_token.clone().unwrap());
            }
        }

        let client = self.client.read().await;
        let client = client.as_ref().ok_or(TaskError::NotConnected)?;

        let (access_token, expires_in) = if let Some(ref creds_path) = self.config.credentials_path
        {
            // Service account key file flow (local dev)
            self.fetch_token_from_sa_key(client, creds_path).await?
        } else {
            // GCP metadata server flow (production)
            self.fetch_token_from_metadata(client).await?
        };

        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in);

        // Update cache
        let mut cache = self.token_cache.write().await;
        cache.access_token = Some(access_token.clone());
        cache.expires_at = Some(expires_at);

        Ok(access_token)
    }

    /// Fetch an access token from the GCP metadata server (production path)
    async fn fetch_token_from_metadata(
        &self,
        client: &reqwest::Client,
    ) -> Result<(String, i64), TaskError> {
        let response = client
            .get(METADATA_TOKEN_URL)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|e| {
                TaskError::Authentication(format!("Failed to fetch OIDC token: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(TaskError::Authentication(format!(
                "Metadata server returned status {}",
                response.status()
            )));
        }

        let token_response: MetadataTokenResponse = response
            .json()
            .await
            .map_err(|e| {
                TaskError::Authentication(format!("Failed to parse token response: {}", e))
            })?;

        Ok((token_response.access_token, token_response.expires_in))
    }

    /// Fetch an access token using a service account JSON key file (local dev path).
    ///
    /// 1. Read and parse the SA key file
    /// 2. Create a JWT assertion signed with the SA's private key (RS256)
    /// 3. Exchange the JWT at Google's OAuth2 token endpoint for an access token
    async fn fetch_token_from_sa_key(
        &self,
        client: &reqwest::Client,
        credentials_path: &str,
    ) -> Result<(String, i64), TaskError> {
        // Read and parse service account key file
        let key_data = std::fs::read_to_string(credentials_path).map_err(|e| {
            TaskError::Authentication(format!(
                "Failed to read service account key file '{}': {}",
                credentials_path, e
            ))
        })?;

        let sa_key: ServiceAccountKey = serde_json::from_str(&key_data).map_err(|e| {
            TaskError::Authentication(format!(
                "Failed to parse service account key file: {}",
                e
            ))
        })?;

        // Create JWT assertion
        let now = Utc::now().timestamp();
        let claims = ServiceAccountClaims {
            iss: sa_key.client_email.clone(),
            sub: sa_key.client_email,
            aud: sa_key.token_uri.clone(),
            scope: CLOUD_TASKS_SCOPE.to_string(),
            iat: now,
            exp: now + 3600, // 1 hour
        };

        let encoding_key = EncodingKey::from_rsa_pem(sa_key.private_key.as_bytes())
            .map_err(|e| {
                TaskError::Authentication(format!(
                    "Failed to parse private key from SA key file: {}",
                    e
                ))
            })?;

        let jwt = encode(&JwtHeader::new(Algorithm::RS256), &claims, &encoding_key)
            .map_err(|e| {
                TaskError::Authentication(format!("Failed to create JWT assertion: {}", e))
            })?;

        // Exchange JWT for access token
        let token_uri = sa_key.token_uri;
        let response = client
            .post(&token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant_type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| {
                TaskError::Authentication(format!("Failed to exchange JWT for token: {}", e))
            })?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TaskError::Authentication(format!(
                "Token exchange failed: {}",
                body
            )));
        }

        let token_response: TokenExchangeResponse = response
            .json()
            .await
            .map_err(|e| {
                TaskError::Authentication(format!(
                    "Failed to parse token exchange response: {}",
                    e
                ))
            })?;

        tracing::debug!(
            "Obtained access token from SA key file (expires in {}s)",
            token_response.expires_in
        );

        Ok((token_response.access_token, token_response.expires_in))
    }

    /// Build a CreateTaskRequest for the Cloud Tasks API
    fn build_create_task_request(
        &self,
        queue: &str,
        message: &TaskMessage,
        schedule_time: Option<DateTime<Utc>>,
    ) -> Result<CreateTaskRequest, TaskError> {
        let payload_json = serde_json::to_vec(message)
            .map_err(|e| TaskError::Serialization(e.to_string()))?;
        let body_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &payload_json,
        );

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let oidc_token = self.config.service_account_email.as_ref().map(|email| {
            OidcTokenField {
                service_account_email: email.clone(),
                audience: Some(self.config.effective_audience().to_string()),
            }
        });

        let schedule_time_str = schedule_time.map(|t| t.to_rfc3339());

        let dispatch_deadline_str = Some(format!("{}s", self.config.dispatch_deadline.as_secs()));

        let retry_config = self.config.max_retry_count.map(|count| RetryConfig {
            max_retry_count: Some(count),
        });

        Ok(CreateTaskRequest {
            task: CloudTaskBody {
                http_request: CloudTasksHttpRequest {
                    url: self.task_url(queue),
                    http_method: "POST".to_string(),
                    body: body_b64,
                    headers,
                    oidc_token,
                },
                schedule_time: schedule_time_str,
                dispatch_deadline: dispatch_deadline_str,
                retry_config,
            },
        })
    }

    /// Validate an inbound OIDC JWT's claims (email, audience, expiry).
    ///
    /// Decodes the JWT payload without signature verification (signature verification
    /// against Google's public JWK keys should be handled by API gateway / middleware).
    /// Validates:
    /// - `email` claim matches the configured `service_account_email`
    /// - `aud` claim matches `oidc_audience` or `worker_url`
    /// - `exp` claim has not passed
    fn validate_inbound_jwt(
        &self,
        jwt_token: &str,
        expected_email: &str,
    ) -> Result<(), TaskError> {
        // Split JWT into parts and decode payload (middle segment)
        let parts: Vec<&str> = jwt_token.split('.').collect();
        if parts.len() != 3 {
            return Err(TaskError::Authentication(
                "Invalid JWT format: expected 3 segments".into(),
            ));
        }

        let payload_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            parts[1],
        )
        .map_err(|e| {
            TaskError::Authentication(format!("Failed to decode JWT payload: {}", e))
        })?;

        let claims: InboundOidcClaims = serde_json::from_slice(&payload_bytes).map_err(|e| {
            TaskError::Authentication(format!("Failed to parse JWT claims: {}", e))
        })?;

        // Check email claim
        match &claims.email {
            Some(email) if email == expected_email => {}
            Some(email) => {
                return Err(TaskError::Authentication(format!(
                    "JWT email mismatch: expected '{}', got '{}'",
                    expected_email, email
                )));
            }
            None => {
                return Err(TaskError::Authentication(
                    "JWT missing 'email' claim".into(),
                ));
            }
        }

        // Check audience claim
        let expected_audience = self.config.effective_audience();
        let audience_valid = match &claims.aud {
            Some(serde_json::Value::String(aud)) => aud == expected_audience,
            Some(serde_json::Value::Array(auds)) => auds.iter().any(|a| {
                a.as_str().map_or(false, |s| s == expected_audience)
            }),
            _ => false,
        };
        if !audience_valid {
            return Err(TaskError::Authentication(format!(
                "JWT audience mismatch: expected '{}'",
                expected_audience
            )));
        }

        // Check expiry
        if let Some(exp) = claims.exp {
            if Utc::now().timestamp() > exp {
                return Err(TaskError::Authentication("JWT has expired".into()));
            }
        }

        tracing::debug!("Inbound OIDC JWT claims validated (email, audience, expiry)");
        Ok(())
    }

    /// Map GCP HTTP error status to TaskError
    fn map_gcp_error(status: reqwest::StatusCode, body: &str) -> TaskError {
        match status.as_u16() {
            404 => TaskError::TaskNotFound(
                body.to_string(),
            ),
            401 | 403 => TaskError::Authentication(
                format!("GCP API authentication error ({}): {}", status, body),
            ),
            409 => TaskError::AlreadyExists(
                body.to_string(),
            ),
            429 => TaskError::RateLimited(Duration::from_secs(60)),
            500..=599 => TaskError::Backend(
                format!("GCP API server error ({}): {}", status, body),
            ),
            _ => TaskError::Backend(
                format!("GCP API error ({}): {}", status, body),
            ),
        }
    }

    /// Send a task creation request to Cloud Tasks API
    async fn create_task(
        &self,
        queue: &str,
        request: &CreateTaskRequest,
    ) -> Result<(), TaskError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(TaskError::NotConnected)?;

        let token = self.get_oidc_token().await?;
        let queue_path = self.config.queue_path(queue);
        let url = format!("{}/{}/tasks", CLOUD_TASKS_API_BASE, queue_path);

        let response = client
            .post(&url)
            .bearer_auth(&token)
            .json(request)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Cloud Tasks API request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            tracing::debug!("Cloud Task created successfully in queue {}", queue);
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }
}

#[async_trait]
impl Broker for CloudTasksBroker {
    async fn connect(&self) -> Result<(), TaskError> {
        // Validate configuration
        if self.config.project_id.is_empty() {
            return Err(TaskError::Configuration("project_id is required".into()));
        }
        if self.config.worker_url.is_empty() {
            return Err(TaskError::Configuration("worker_url is required".into()));
        }

        // Initialize HTTP client
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TaskError::Connection(format!("Failed to create HTTP client: {}", e)))?;

        *self.client.write().await = Some(http_client);

        tracing::info!(
            project_id = %self.config.project_id,
            location = %self.config.location,
            "Connected to Cloud Tasks"
        );

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), TaskError> {
        *self.client.write().await = None;
        tracing::info!("Disconnected from Cloud Tasks");
        Ok(())
    }

    async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
        tracing::debug!(
            task_id = %message.id,
            queue = %queue,
            "Creating Cloud Task"
        );

        let request = self.build_create_task_request(queue, &message, None)?;
        self.create_task(queue, &request).await?;

        tracing::info!(
            task_id = %message.id,
            queue = %queue,
            "Cloud Task created"
        );

        Ok(())
    }

    async fn health_check(&self) -> Result<(), TaskError> {
        let client = self.client.read().await;
        let client = client.as_ref().ok_or(TaskError::NotConnected)?;

        let token = self.get_oidc_token().await?;
        let url = format!(
            "{}/projects/{}/locations",
            CLOUD_TASKS_API_BASE, self.config.project_id
        );

        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                TaskError::Connection(format!("Cloud Tasks health check failed: {}", e))
            })?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    fn delivery_model(&self) -> DeliveryModel {
        DeliveryModel::Push
    }

    fn capabilities(&self) -> BrokerCapabilities {
        BrokerCapabilities {
            delayed_tasks: true,
            dead_letter: true,
            priority: false,
            batching: false,
            max_delay: Some(Duration::from_secs(MAX_DELAY_SECS)), // 30 days
        }
    }
}

impl PushBroker for CloudTasksBroker {
    fn parse_push_request(
        &self,
        headers: &HashMap<String, String>,
        body: &[u8],
    ) -> Result<BrokerMessage, TaskError> {
        // Validate OIDC token if configured
        if let Some(ref expected_email) = self.config.service_account_email {
            let auth_header = headers
                .get("authorization")
                .ok_or_else(|| {
                    TaskError::Authentication("Missing Authorization header".into())
                })?;

            if !auth_header.starts_with("Bearer ") {
                return Err(TaskError::Authentication(
                    "Invalid Authorization header format".into(),
                ));
            }

            let jwt_token = &auth_header["Bearer ".len()..];
            self.validate_inbound_jwt(jwt_token, expected_email)?;
        }

        // Parse the task message
        let payload: TaskMessage = serde_json::from_slice(body)
            .map_err(|e| TaskError::Deserialization(e.to_string()))?;

        // Extract delivery info from Cloud Tasks headers
        let delivery_tag = headers
            .get("x-cloudtasks-taskname")
            .cloned()
            .unwrap_or_else(|| payload.id.to_string());

        let retry_count = headers
            .get("x-cloudtasks-taskretrycount")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        Ok(BrokerMessage {
            delivery_tag,
            payload,
            headers: headers.clone(),
            timestamp: Utc::now(),
            redelivered: retry_count > 0,
        })
    }

    fn endpoint_path(&self) -> &str {
        "/meteor/push/{queue}"
    }
}

#[async_trait]
impl DelayedBroker for CloudTasksBroker {
    async fn publish_delayed(
        &self,
        queue: &str,
        message: TaskMessage,
        delay: Duration,
    ) -> Result<(), TaskError> {
        // Clamp delay to Cloud Tasks maximum (30 days)
        let clamped = delay.min(Duration::from_secs(MAX_DELAY_SECS));
        let schedule_time = Utc::now() + chrono::Duration::from_std(clamped)
            .unwrap_or_else(|_| chrono::Duration::zero());

        tracing::debug!(
            task_id = %message.id,
            queue = %queue,
            delay_secs = %clamped.as_secs(),
            schedule_time = %schedule_time.to_rfc3339(),
            "Creating delayed Cloud Task"
        );

        let request = self.build_create_task_request(queue, &message, Some(schedule_time))?;
        self.create_task(queue, &request).await?;

        tracing::info!(
            task_id = %message.id,
            queue = %queue,
            schedule_time = %schedule_time.to_rfc3339(),
            "Delayed Cloud Task created"
        );

        Ok(())
    }

    // publish_at uses the default trait implementation:
    // if eta <= now → calls publish() immediately
    // if eta > now → converts to delay and calls publish_delayed()
}

// ===========================================================================
// Unit tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn test_config() -> CloudTasksConfig {
        CloudTasksConfig {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            worker_url: "https://app.example.com".to_string(),
            service_account_email: Some("sa@my-project.iam.gserviceaccount.com".to_string()),
            oidc_audience: None,
            default_queue: "default".to_string(),
            dispatch_deadline: Duration::from_secs(600),
            max_retry_count: None,
            credentials_path: None,
        }
    }

    fn test_config_no_sa() -> CloudTasksConfig {
        CloudTasksConfig {
            service_account_email: None,
            ..test_config()
        }
    }

    fn sample_message() -> TaskMessage {
        TaskMessage::new("test.add", json!([1, 2]))
    }

    /// Build a fake JWT (header.payload.signature) with given claims for testing.
    /// Signature is not valid (dummy), but the payload is properly base64url-encoded.
    fn make_test_jwt(email: &str, audience: &str, exp: i64) -> String {
        let header = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            r#"{"alg":"RS256","typ":"JWT"}"#,
        );
        let payload_json = serde_json::json!({
            "email": email,
            "aud": audience,
            "exp": exp,
            "iss": "accounts.google.com",
            "sub": "1234567890",
        });
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            payload_json.to_string().as_bytes(),
        );
        let signature = "fake_signature_for_test";
        format!("{}.{}.{}", header, payload, signature)
    }

    // -----------------------------------------------------------------------
    // CloudTasksConfig helpers (queue_path, effective_audience)
    // -----------------------------------------------------------------------

    #[test]
    fn test_queue_path_format() {
        let cfg = test_config();
        assert_eq!(
            cfg.queue_path("default"),
            "projects/my-project/locations/us-central1/queues/default"
        );
    }

    #[test]
    fn test_queue_path_custom_queue() {
        let cfg = test_config();
        assert_eq!(
            cfg.queue_path("high-priority"),
            "projects/my-project/locations/us-central1/queues/high-priority"
        );
    }

    #[test]
    fn test_effective_audience_defaults_to_worker_url() {
        let cfg = test_config(); // oidc_audience is None
        assert_eq!(cfg.effective_audience(), "https://app.example.com");
    }

    #[test]
    fn test_effective_audience_with_explicit_value() {
        let cfg = CloudTasksConfig {
            oidc_audience: Some("https://custom-audience.example.com".to_string()),
            ..test_config()
        };
        assert_eq!(cfg.effective_audience(), "https://custom-audience.example.com");
    }

    // -----------------------------------------------------------------------
    // OidcTokenCache validity (R4)
    // -----------------------------------------------------------------------

    #[test]
    fn test_token_cache_new_is_invalid() {
        let cache = OidcTokenCache::new();
        assert!(cache.access_token.is_none());
        assert!(cache.expires_at.is_none());
        assert!(!cache.is_valid(), "Fresh cache with no token should be invalid");
    }

    #[test]
    fn test_token_cache_valid_with_future_expiry() {
        let cache = OidcTokenCache {
            access_token: Some("token-abc".to_string()),
            // Expires 1 hour from now — well within refresh buffer
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };
        assert!(cache.is_valid(), "Token expiring in 1h should be valid");
    }

    #[test]
    fn test_token_cache_invalid_when_near_expiry() {
        let cache = OidcTokenCache {
            access_token: Some("token-abc".to_string()),
            // Expires in 2 minutes — within the 5-minute refresh buffer
            expires_at: Some(Utc::now() + chrono::Duration::minutes(2)),
        };
        assert!(
            !cache.is_valid(),
            "Token expiring in 2min should be invalid (within 5-min refresh buffer)"
        );
    }

    #[test]
    fn test_token_cache_invalid_when_expired() {
        let cache = OidcTokenCache {
            access_token: Some("token-old".to_string()),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
        };
        assert!(!cache.is_valid(), "Expired token should be invalid");
    }

    #[test]
    fn test_token_cache_invalid_without_token() {
        let cache = OidcTokenCache {
            access_token: None,
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        };
        assert!(!cache.is_valid(), "Cache without access_token should be invalid");
    }

    #[test]
    fn test_token_cache_invalid_without_expiry() {
        let cache = OidcTokenCache {
            access_token: Some("token-abc".to_string()),
            expires_at: None,
        };
        assert!(!cache.is_valid(), "Cache without expires_at should be invalid");
    }

    // -----------------------------------------------------------------------
    // CloudTasksBroker::task_url
    // -----------------------------------------------------------------------

    #[test]
    fn test_task_url_construction() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(
            broker.task_url("default"),
            "https://app.example.com/meteor/push/default"
        );
    }

    #[test]
    fn test_task_url_custom_queue() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(
            broker.task_url("priority-high"),
            "https://app.example.com/meteor/push/priority-high"
        );
    }

    // -----------------------------------------------------------------------
    // S1/R6: build_create_task_request — immediate publish
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_create_task_request_immediate() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        // HTTP request fields
        assert_eq!(req.task.http_request.url, "https://app.example.com/meteor/push/default");
        assert_eq!(req.task.http_request.http_method, "POST");
        assert_eq!(
            req.task.http_request.headers.get("Content-Type").unwrap(),
            "application/json"
        );

        // Body should be base64-encoded TaskMessage JSON
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &req.task.http_request.body,
        )
        .expect("body should be valid base64");
        let decoded_msg: TaskMessage =
            serde_json::from_slice(&decoded).expect("decoded body should be valid TaskMessage");
        assert_eq!(decoded_msg.task_name, "test.add");
        assert_eq!(decoded_msg.args, json!([1, 2]));

        // No schedule time for immediate publish
        assert!(req.task.schedule_time.is_none());

        // Dispatch deadline
        assert_eq!(req.task.dispatch_deadline.as_deref(), Some("600s"));
    }

    #[test]
    fn test_build_create_task_request_has_oidc_token() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let oidc = req.task.http_request.oidc_token.as_ref()
            .expect("OIDC token should be present when service_account_email is set");
        assert_eq!(oidc.service_account_email, "sa@my-project.iam.gserviceaccount.com");
        assert_eq!(
            oidc.audience.as_deref(),
            Some("https://app.example.com"),
            "Audience should default to worker_url"
        );
    }

    #[test]
    fn test_build_create_task_request_no_oidc_without_service_account() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        assert!(
            req.task.http_request.oidc_token.is_none(),
            "No OIDC token when service_account_email is None"
        );
    }

    #[test]
    fn test_build_create_task_request_custom_audience() {
        let cfg = CloudTasksConfig {
            oidc_audience: Some("https://custom-aud.example.com".to_string()),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let oidc = req.task.http_request.oidc_token.as_ref().unwrap();
        assert_eq!(oidc.audience.as_deref(), Some("https://custom-aud.example.com"));
    }

    // -----------------------------------------------------------------------
    // S4/R3: build_create_task_request — delayed publish with scheduleTime
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_create_task_request_with_schedule_time() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let schedule = Utc::now() + chrono::Duration::minutes(5);

        let req = broker
            .build_create_task_request("default", &msg, Some(schedule))
            .unwrap();

        let sched_str = req.task.schedule_time
            .as_ref()
            .expect("schedule_time should be present for delayed publish");

        // Parse back to verify it's valid RFC 3339
        let parsed = DateTime::parse_from_rfc3339(sched_str)
            .expect("schedule_time should be valid RFC 3339");
        // Should be roughly 5 minutes from now (allow 10s tolerance)
        let diff = (parsed.timestamp() - schedule.timestamp()).abs();
        assert!(diff < 2, "schedule_time should match the requested time");
    }

    #[test]
    fn test_build_create_task_request_dispatch_deadline() {
        let cfg = CloudTasksConfig {
            dispatch_deadline: Duration::from_secs(300), // 5 minutes
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        assert_eq!(req.task.dispatch_deadline.as_deref(), Some("300s"));
    }

    // -----------------------------------------------------------------------
    // S9/R7: map_gcp_error — HTTP status code to TaskError mapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_map_gcp_error_404_to_task_not_found() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::NOT_FOUND,
            "Queue not found",
        );
        match err {
            TaskError::TaskNotFound(msg) => assert!(msg.contains("Queue not found")),
            other => panic!("Expected TaskNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_401_to_authentication() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::UNAUTHORIZED,
            "Invalid credentials",
        );
        match err {
            TaskError::Authentication(msg) => {
                assert!(msg.contains("401"));
                assert!(msg.contains("Invalid credentials"));
            }
            other => panic!("Expected Authentication, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_403_to_authentication() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::FORBIDDEN,
            "Permission denied",
        );
        match err {
            TaskError::Authentication(msg) => {
                assert!(msg.contains("403"));
                assert!(msg.contains("Permission denied"));
            }
            other => panic!("Expected Authentication, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_409_to_already_exists() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::CONFLICT,
            "Task already exists",
        );
        match err {
            TaskError::AlreadyExists(msg) => assert!(msg.contains("Task already exists")),
            other => panic!("Expected AlreadyExists, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_429_to_rate_limited() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded",
        );
        match err {
            TaskError::RateLimited(duration) => {
                assert_eq!(duration, Duration::from_secs(60));
            }
            other => panic!("Expected RateLimited, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_500_to_backend() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        );
        match err {
            TaskError::Backend(msg) => {
                assert!(msg.contains("500"));
                assert!(msg.contains("Internal server error"));
            }
            other => panic!("Expected Backend, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_503_to_backend() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            "Service unavailable",
        );
        match err {
            TaskError::Backend(msg) => {
                assert!(msg.contains("503"));
            }
            other => panic!("Expected Backend, got: {:?}", other),
        }
    }

    #[test]
    fn test_map_gcp_error_unknown_status_to_backend() {
        let err = CloudTasksBroker::map_gcp_error(
            reqwest::StatusCode::IM_A_TEAPOT,
            "I'm a teapot",
        );
        match err {
            TaskError::Backend(msg) => {
                assert!(msg.contains("418"));
            }
            other => panic!("Expected Backend for unknown status, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Serde serialization — verify camelCase output matches Cloud Tasks API
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_task_request_serializes_camel_case() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let schedule = Utc::now() + chrono::Duration::minutes(10);

        let req = broker
            .build_create_task_request("default", &msg, Some(schedule))
            .unwrap();

        let json_val = serde_json::to_value(&req).expect("should serialize to JSON");

        // Top-level: task
        let task = json_val.get("task").expect("should have 'task' field");

        // camelCase field names
        assert!(task.get("httpRequest").is_some(), "should use camelCase 'httpRequest'");
        assert!(task.get("scheduleTime").is_some(), "should use camelCase 'scheduleTime'");
        assert!(task.get("dispatchDeadline").is_some(), "should use camelCase 'dispatchDeadline'");

        let http_req = task.get("httpRequest").unwrap();
        assert!(http_req.get("httpMethod").is_some(), "should use camelCase 'httpMethod'");
        assert!(http_req.get("oidcToken").is_some(), "should use camelCase 'oidcToken'");

        let oidc = http_req.get("oidcToken").unwrap();
        assert!(
            oidc.get("serviceAccountEmail").is_some(),
            "should use camelCase 'serviceAccountEmail'"
        );
    }

    #[test]
    fn test_create_task_request_omits_null_schedule_time() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let json_val = serde_json::to_value(&req).unwrap();
        let task = json_val.get("task").unwrap();

        assert!(
            task.get("scheduleTime").is_none(),
            "scheduleTime should be omitted when None (skip_serializing_if)"
        );
    }

    #[test]
    fn test_create_task_request_omits_oidc_when_no_service_account() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let json_val = serde_json::to_value(&req).unwrap();
        let http_req = json_val["task"]["httpRequest"].as_object().unwrap();

        assert!(
            http_req.get("oidcToken").is_none(),
            "oidcToken should be omitted when service_account_email is None"
        );
    }

    // -----------------------------------------------------------------------
    // S2/R2: parse_push_request — comprehensive push request parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_push_request_first_delivery() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = TaskMessage::new("compute.sum", json!({"x": 1, "y": 2}));
        let body = serde_json::to_vec(&msg).unwrap();

        let mut headers = HashMap::new();
        headers.insert("x-cloudtasks-taskname".to_string(), "task-123".to_string());
        headers.insert("x-cloudtasks-taskretrycount".to_string(), "0".to_string());

        let result = broker.parse_push_request(&headers, &body).unwrap();
        assert_eq!(result.delivery_tag, "task-123");
        assert_eq!(result.payload.task_name, "compute.sum");
        assert!(!result.redelivered);
        assert_eq!(result.headers.get("x-cloudtasks-taskname").unwrap(), "task-123");
    }

    #[test]
    fn test_parse_push_request_redelivery_detection() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let mut headers = HashMap::new();
        headers.insert("x-cloudtasks-taskname".to_string(), "task-retry".to_string());
        headers.insert("x-cloudtasks-taskretrycount".to_string(), "3".to_string());

        let result = broker.parse_push_request(&headers, &body).unwrap();
        assert!(result.redelivered, "retry count > 0 indicates redelivery");
    }

    #[test]
    fn test_parse_push_request_retry_count_1_is_redelivered() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let mut headers = HashMap::new();
        headers.insert("x-cloudtasks-taskname".to_string(), "task-r1".to_string());
        headers.insert("x-cloudtasks-taskretrycount".to_string(), "1".to_string());

        let result = broker.parse_push_request(&headers, &body).unwrap();
        assert!(result.redelivered, "retry count 1 should be redelivered");
    }

    #[test]
    fn test_parse_push_request_missing_retry_header_defaults_zero() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let mut headers = HashMap::new();
        headers.insert("x-cloudtasks-taskname".to_string(), "task-no-retry".to_string());
        // No x-cloudtasks-taskretrycount header

        let result = broker.parse_push_request(&headers, &body).unwrap();
        assert!(!result.redelivered, "Missing retry count header should default to 0 (not redelivered)");
    }

    #[test]
    fn test_parse_push_request_fallback_delivery_tag() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let expected_id = msg.id.to_string();
        let body = serde_json::to_vec(&msg).unwrap();

        let headers = HashMap::new(); // No x-cloudtasks-taskname

        let result = broker.parse_push_request(&headers, &body).unwrap();
        assert_eq!(
            result.delivery_tag, expected_id,
            "Missing taskname header should fall back to task ID"
        );
    }

    #[test]
    fn test_parse_push_request_invalid_body() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let result = broker.parse_push_request(&HashMap::new(), b"not json");
        match result {
            Err(TaskError::Deserialization(_)) => {} // expected
            other => panic!("Expected Deserialization error, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // S7/R5: OIDC inbound validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_push_request_rejects_missing_auth_when_sa_configured() {
        let broker = CloudTasksBroker::new(test_config()); // has service_account_email
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let result = broker.parse_push_request(&HashMap::new(), &body);
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("Missing Authorization header"));
            }
            other => panic!("Expected Authentication error, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_push_request_rejects_non_bearer_auth() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), "Basic abc123".to_string());

        let result = broker.parse_push_request(&headers, &body);
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("Invalid Authorization header format"));
            }
            other => panic!("Expected Authentication error for non-Bearer, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_push_request_accepts_bearer_token() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        let jwt_token = make_test_jwt(
            "sa@my-project.iam.gserviceaccount.com",
            "https://app.example.com",
            Utc::now().timestamp() + 3600,
        );

        let mut headers = HashMap::new();
        headers.insert("authorization".to_string(), format!("Bearer {}", jwt_token));
        headers.insert("x-cloudtasks-taskname".to_string(), "valid-task".to_string());

        let result = broker.parse_push_request(&headers, &body);
        assert!(result.is_ok(), "Valid Bearer token should pass");
        assert_eq!(result.unwrap().delivery_tag, "valid-task");
    }

    #[test]
    fn test_parse_push_request_skips_auth_when_no_sa() {
        let broker = CloudTasksBroker::new(test_config_no_sa());
        let msg = sample_message();
        let body = serde_json::to_vec(&msg).unwrap();

        // No Authorization header, but no service_account_email configured either
        let result = broker.parse_push_request(&HashMap::new(), &body);
        assert!(result.is_ok(), "Auth check should be skipped when service_account_email is None");
    }

    // -----------------------------------------------------------------------
    // S10/R1: connect() validation
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_connect_rejects_empty_project_id() {
        let cfg = CloudTasksConfig {
            project_id: String::new(),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        match broker.connect().await {
            Err(TaskError::Configuration(msg)) => {
                assert!(msg.contains("project_id is required"));
            }
            other => panic!("Expected Configuration error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connect_rejects_empty_worker_url() {
        let cfg = CloudTasksConfig {
            worker_url: String::new(),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        match broker.connect().await {
            Err(TaskError::Configuration(msg)) => {
                assert!(msg.contains("worker_url is required"));
            }
            other => panic!("Expected Configuration error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connect_succeeds_with_valid_config() {
        let broker = CloudTasksBroker::new(test_config());
        let result = broker.connect().await;
        assert!(result.is_ok(), "connect() should succeed with valid config");
    }

    #[tokio::test]
    async fn test_disconnect_clears_client() {
        let broker = CloudTasksBroker::new(test_config());
        broker.connect().await.unwrap();
        let result = broker.disconnect().await;
        assert!(result.is_ok());
        // Client should be None after disconnect
        assert!(broker.client.read().await.is_none());
    }

    // -----------------------------------------------------------------------
    // S12/R2: Ack/nack status codes, endpoint path
    // -----------------------------------------------------------------------

    #[test]
    fn test_ack_status_code_is_200() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(broker.ack_status_code(), 200);
    }

    #[test]
    fn test_nack_status_code_is_500() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(broker.nack_status_code(), 500);
    }

    #[test]
    fn test_endpoint_path_pattern() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(broker.endpoint_path(), "/meteor/push/{queue}");
    }

    // -----------------------------------------------------------------------
    // Delivery model and capabilities
    // -----------------------------------------------------------------------

    #[test]
    fn test_delivery_model_is_push() {
        let broker = CloudTasksBroker::new(test_config());
        assert_eq!(broker.delivery_model(), DeliveryModel::Push);
    }

    #[test]
    fn test_capabilities_match_spec() {
        let broker = CloudTasksBroker::new(test_config());
        let caps = broker.capabilities();

        assert!(caps.delayed_tasks, "Cloud Tasks supports delayed tasks");
        assert!(caps.dead_letter, "Cloud Tasks supports dead letter queues");
        assert!(!caps.priority, "Cloud Tasks does not support priority");
        assert!(!caps.batching, "Cloud Tasks does not support batching");
        assert_eq!(
            caps.max_delay,
            Some(Duration::from_secs(30 * 24 * 60 * 60)),
            "Max delay should be 30 days"
        );
    }

    // -----------------------------------------------------------------------
    // CloudTasksConfig::default
    // -----------------------------------------------------------------------

    #[test]
    fn test_config_default_values() {
        let cfg = CloudTasksConfig::default();
        assert!(cfg.project_id.is_empty());
        assert_eq!(cfg.location, "us-central1");
        assert!(cfg.worker_url.is_empty());
        assert!(cfg.service_account_email.is_none());
        assert!(cfg.oidc_audience.is_none());
        assert_eq!(cfg.default_queue, "default");
        assert_eq!(cfg.dispatch_deadline, Duration::from_secs(600));
        assert!(cfg.max_retry_count.is_none());
        assert!(cfg.credentials_path.is_none());
    }

    // -----------------------------------------------------------------------
    // Constants
    // -----------------------------------------------------------------------

    #[test]
    fn test_max_delay_is_30_days() {
        assert_eq!(MAX_DELAY_SECS, 30 * 24 * 60 * 60);
        assert_eq!(MAX_DELAY_SECS, 2_592_000);
    }

    #[test]
    fn test_token_refresh_buffer_is_5_minutes() {
        assert_eq!(TOKEN_REFRESH_BUFFER_SECS, 300);
    }

    #[test]
    fn test_cloud_tasks_api_base_url() {
        assert_eq!(CLOUD_TASKS_API_BASE, "https://cloudtasks.googleapis.com/v2");
    }

    #[test]
    fn test_metadata_token_url() {
        assert_eq!(
            METADATA_TOKEN_URL,
            "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token"
        );
    }

    // -----------------------------------------------------------------------
    // create_task requires connected client (not connected → NotConnected)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_create_task_not_connected_returns_error() {
        let broker = CloudTasksBroker::new(test_config());
        // Don't call connect() — client is None
        let req = broker.build_create_task_request("default", &sample_message(), None).unwrap();
        let result = broker.create_task("default", &req).await;
        match result {
            Err(TaskError::NotConnected) => {} // expected
            other => panic!("Expected NotConnected, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_publish_not_connected_returns_error() {
        let broker = CloudTasksBroker::new(test_config());
        let result = broker.publish("default", sample_message()).await;
        match result {
            Err(TaskError::NotConnected) => {} // expected
            other => panic!("Expected NotConnected from publish, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_health_check_not_connected_returns_error() {
        let broker = CloudTasksBroker::new(test_config());
        let result = broker.health_check().await;
        match result {
            Err(TaskError::NotConnected) => {} // expected
            other => panic!("Expected NotConnected from health_check, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // CloudTasksBroker::new initializes correctly
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_new_broker_starts_disconnected() {
        let broker = CloudTasksBroker::new(test_config());
        assert!(broker.client.read().await.is_none(), "New broker should not have an HTTP client");
        let cache = broker.token_cache.read().await;
        assert!(!cache.is_valid(), "New broker should have empty token cache");
    }

    // -----------------------------------------------------------------------
    // MetadataTokenResponse deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_metadata_token_response_deserialization() {
        let json = r#"{
            "access_token": "ya29.test-token",
            "expires_in": 3599,
            "token_type": "Bearer"
        }"#;
        let resp: MetadataTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "ya29.test-token");
        assert_eq!(resp.expires_in, 3599);
        assert_eq!(resp.token_type, "Bearer");
    }

    // -----------------------------------------------------------------------
    // CloudTaskResponse deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_cloud_task_response_deserialization() {
        let json = r#"{
            "name": "projects/p/locations/l/queues/q/tasks/t123",
            "scheduleTime": "2025-01-01T00:00:00Z",
            "createTime": "2025-01-01T00:00:00Z",
            "dispatchCount": 1,
            "responseCount": 1
        }"#;
        let resp: CloudTaskResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.name.as_deref(),
            Some("projects/p/locations/l/queues/q/tasks/t123")
        );
        assert_eq!(resp.dispatch_count, Some(1));
        assert_eq!(resp.response_count, Some(1));
    }

    #[test]
    fn test_cloud_task_response_partial_fields() {
        // API may return only a subset of fields
        let json = r#"{"name": "projects/p/locations/l/queues/q/tasks/t456"}"#;
        let resp: CloudTaskResponse = serde_json::from_str(json).unwrap();
        assert!(resp.name.is_some());
        assert!(resp.schedule_time.is_none());
        assert!(resp.dispatch_count.is_none());
    }

    // -----------------------------------------------------------------------
    // P0 fix: SA key auth — ServiceAccountKey deserialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_service_account_key_deserialization() {
        let json = r#"{
            "type": "service_account",
            "project_id": "my-project",
            "private_key_id": "key-123",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----\n",
            "client_email": "sa@my-project.iam.gserviceaccount.com",
            "client_id": "123456",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token"
        }"#;
        let key: ServiceAccountKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.client_email, "sa@my-project.iam.gserviceaccount.com");
        assert!(key.private_key.contains("RSA PRIVATE KEY"));
        assert_eq!(key.token_uri, "https://oauth2.googleapis.com/token");
    }

    #[test]
    fn test_service_account_key_default_token_uri() {
        // When token_uri is absent, default to GOOGLE_TOKEN_URI
        let json = r#"{
            "client_email": "sa@test.iam.gserviceaccount.com",
            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----\n"
        }"#;
        let key: ServiceAccountKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.token_uri, GOOGLE_TOKEN_URI);
    }

    #[test]
    fn test_token_exchange_response_deserialization() {
        let json = r#"{
            "access_token": "ya29.sa-token",
            "expires_in": 3600,
            "token_type": "Bearer"
        }"#;
        let resp: TokenExchangeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "ya29.sa-token");
        assert_eq!(resp.expires_in, 3600);
        assert_eq!(resp.token_type, "Bearer");
    }

    #[test]
    fn test_google_token_uri_constant() {
        assert_eq!(GOOGLE_TOKEN_URI, "https://oauth2.googleapis.com/token");
    }

    #[test]
    fn test_cloud_tasks_scope_constant() {
        assert_eq!(
            CLOUD_TASKS_SCOPE,
            "https://www.googleapis.com/auth/cloud-tasks"
        );
    }

    #[tokio::test]
    async fn test_get_oidc_token_not_connected_returns_error() {
        // When credentials_path is set but client is not connected,
        // get_oidc_token should still fail with NotConnected when cache is empty
        let cfg = CloudTasksConfig {
            credentials_path: Some("/nonexistent/path.json".to_string()),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        // Don't call connect() → client is None
        let result = broker.get_oidc_token().await;
        match result {
            Err(TaskError::NotConnected) => {} // expected
            other => panic!("Expected NotConnected, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_oidc_token_invalid_credentials_path() {
        let cfg = CloudTasksConfig {
            credentials_path: Some("/nonexistent/sa-key.json".to_string()),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        broker.connect().await.unwrap();
        let result = broker.get_oidc_token().await;
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("Failed to read service account key file"));
            }
            other => panic!("Expected Authentication error for bad path, got: {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // P1 fix: Inbound OIDC JWT validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_inbound_jwt_email_mismatch() {
        let broker = CloudTasksBroker::new(test_config());
        let jwt = make_test_jwt(
            "wrong@other-project.iam.gserviceaccount.com",
            "https://app.example.com",
            Utc::now().timestamp() + 3600,
        );
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("email mismatch"), "Got: {}", msg);
            }
            other => panic!("Expected Authentication error for email mismatch, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_inbound_jwt_audience_mismatch() {
        let broker = CloudTasksBroker::new(test_config());
        let jwt = make_test_jwt(
            "sa@my-project.iam.gserviceaccount.com",
            "https://wrong-audience.example.com",
            Utc::now().timestamp() + 3600,
        );
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("audience mismatch"), "Got: {}", msg);
            }
            other => panic!("Expected Authentication error for audience mismatch, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_inbound_jwt_expired_token() {
        let broker = CloudTasksBroker::new(test_config());
        let jwt = make_test_jwt(
            "sa@my-project.iam.gserviceaccount.com",
            "https://app.example.com",
            Utc::now().timestamp() - 3600, // expired 1 hour ago
        );
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("expired"), "Got: {}", msg);
            }
            other => panic!("Expected Authentication error for expired token, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_inbound_jwt_malformed_token() {
        let broker = CloudTasksBroker::new(test_config());
        let result = broker.validate_inbound_jwt(
            "not.a.valid-jwt",
            "sa@my-project.iam.gserviceaccount.com",
        );
        assert!(result.is_err(), "Malformed JWT should fail validation");
    }

    #[test]
    fn test_validate_inbound_jwt_wrong_segment_count() {
        let broker = CloudTasksBroker::new(test_config());
        let result = broker.validate_inbound_jwt(
            "only-one-segment",
            "sa@my-project.iam.gserviceaccount.com",
        );
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("expected 3 segments"), "Got: {}", msg);
            }
            other => panic!("Expected Authentication error for wrong segments, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_inbound_jwt_valid_token_passes() {
        let broker = CloudTasksBroker::new(test_config());
        let jwt = make_test_jwt(
            "sa@my-project.iam.gserviceaccount.com",
            "https://app.example.com",
            Utc::now().timestamp() + 3600,
        );
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        assert!(result.is_ok(), "Valid JWT should pass validation");
    }

    #[test]
    fn test_validate_inbound_jwt_missing_email_claim() {
        let broker = CloudTasksBroker::new(test_config());
        // Build JWT without email claim
        let header = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            r#"{"alg":"RS256","typ":"JWT"}"#,
        );
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            r#"{"aud":"https://app.example.com","exp":9999999999}"#,
        );
        let jwt = format!("{}.{}.fake_sig", header, payload);
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        match result {
            Err(TaskError::Authentication(msg)) => {
                assert!(msg.contains("missing 'email' claim"), "Got: {}", msg);
            }
            other => panic!("Expected error for missing email, got: {:?}", other),
        }
    }

    #[test]
    fn test_validate_inbound_jwt_audience_as_array() {
        let broker = CloudTasksBroker::new(test_config());
        // JWT with audience as an array (allowed by JWT spec)
        let header = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            r#"{"alg":"RS256","typ":"JWT"}"#,
        );
        let claims = json!({
            "email": "sa@my-project.iam.gserviceaccount.com",
            "aud": ["https://app.example.com", "https://other.example.com"],
            "exp": Utc::now().timestamp() + 3600,
        });
        let payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            claims.to_string().as_bytes(),
        );
        let jwt = format!("{}.{}.fake_sig", header, payload);
        let result = broker.validate_inbound_jwt(
            &jwt,
            "sa@my-project.iam.gserviceaccount.com",
        );
        assert!(result.is_ok(), "JWT with audience array containing expected aud should pass");
    }

    // -----------------------------------------------------------------------
    // P1 fix: max_retry_count forwarded in task creation payload
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_create_task_request_with_max_retry_count() {
        let cfg = CloudTasksConfig {
            max_retry_count: Some(5),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let retry_cfg = req.task.retry_config
            .as_ref()
            .expect("retry_config should be present when max_retry_count is set");
        assert_eq!(retry_cfg.max_retry_count, Some(5));
    }

    #[test]
    fn test_build_create_task_request_no_retry_config_when_none() {
        // Default test_config has max_retry_count: None
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        assert!(
            req.task.retry_config.is_none(),
            "retry_config should be None when max_retry_count is not set"
        );
    }

    #[test]
    fn test_retry_config_serializes_camel_case() {
        let cfg = CloudTasksConfig {
            max_retry_count: Some(10),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let json_val = serde_json::to_value(&req).unwrap();
        let task = json_val.get("task").unwrap();

        let retry = task.get("retryConfig")
            .expect("should use camelCase 'retryConfig'");
        assert_eq!(
            retry.get("maxRetryCount").and_then(|v| v.as_u64()),
            Some(10),
            "should use camelCase 'maxRetryCount'"
        );
    }

    #[test]
    fn test_retry_config_omitted_in_json_when_none() {
        let broker = CloudTasksBroker::new(test_config());
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let json_val = serde_json::to_value(&req).unwrap();
        let task = json_val.get("task").unwrap();

        assert!(
            task.get("retryConfig").is_none(),
            "retryConfig should be omitted from JSON when None"
        );
    }

    #[test]
    fn test_build_create_task_request_with_zero_retry_count() {
        // max_retry_count: Some(0) means "no retries" — should still include the field
        let cfg = CloudTasksConfig {
            max_retry_count: Some(0),
            ..test_config()
        };
        let broker = CloudTasksBroker::new(cfg);
        let msg = sample_message();
        let req = broker.build_create_task_request("default", &msg, None).unwrap();

        let retry_cfg = req.task.retry_config.as_ref()
            .expect("retry_config should be present even with count=0");
        assert_eq!(retry_cfg.max_retry_count, Some(0));
    }
}

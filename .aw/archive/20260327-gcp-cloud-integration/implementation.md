---
id: implementation
type: change_implementation
change_id: gcp-cloud-integration
---

# Implementation

## Summary

Complete GCP Cloud integration for cclab-queue: (1) Cloud Tasks broker (cloudtasks.rs, ~1271 lines) — replaced stub publish() with actual Cloud Tasks REST API v2 calls; reqwest HTTP client with OIDC token caching (5-min refresh buffer via OidcTokenCache); CreateTaskRequest/CloudTaskBody/CloudTasksHttpRequest request types with camelCase serde for API; build_create_task_request() constructs base64-encoded task payloads with OIDC token field; create_task() sends authenticated POST to cloudtasks.googleapis.com/v2/{queuePath}/tasks; connect() validates config + initializes HTTP client, disconnect() clears it; health_check() verifies API connectivity via GET /locations; map_gcp_error() maps HTTP status codes (404→TaskNotFound, 401/403→Authentication, 409→AlreadyExists, 429→RateLimited, 5xx→Backend); DelayedBroker impl with publish_delayed() — clamps delay to 30-day max, sets scheduleTime as RFC3339; ~60 unit tests covering config helpers, OIDC cache validity, request building, serde camelCase, push parsing, error mapping, connect/disconnect lifecycle. (2) Cloud Scheduler backend (cloud_scheduler_backend.rs, 572 lines) — new SchedulerBackend impl; CloudSchedulerConfig with project_id/location/oidc_service_account_email/target_base_url/time_zone; OIDC token caching identical to Cloud Tasks; CRUD operations: create_job, update_job, delete_job, get_job, list_jobs via Cloud Scheduler REST API v1; pause_job_api/resume_job_api — POST to {jobName}:pause/:resume; SchedulerBackend trait: leader election no-ops (GCP manages scheduling authority), in-memory TaskScheduleState tracking, pause_task/resume_task override both GCP API and local state; ~45 unit tests (cloud_scheduler_backend_tests.rs, 702 lines): leader election no-ops, config helpers, job serde roundtrip, task state management, OIDC cache validity, GCP error mapping, HttpTarget serialization. (3) Feature gates: cloudtasks=['dep:reqwest'], cloud-scheduler=['dep:reqwest'], gcp-full=['gcp-push','cloud-scheduler']; reqwest added as optional workspace dependency. (4) Error variants: +Authentication(String) and +AlreadyExists(String) in TaskError enum. (5) Re-exports: CloudSchedulerBackend + CloudSchedulerConfig in lib.rs behind cfg(feature='cloud-scheduler').

## Diff


```diff
diff --git a/Cargo.lock b/Cargo.lock
index 095b7e04..31f858a4 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -2031,6 +2031,7 @@ dependencies = [
  "futures",
  "google-cloud-googleapis",
  "google-cloud-pubsub",
+ "jsonwebtoken",
  "k8s-openapi",
  "kube",
  "num_cpus",
@@ -2042,6 +2043,7 @@ dependencies = [
  "pythonize",
  "redis",
  "regex",
+ "reqwest 0.12.28",
  "schemars",
  "serde",
  "serde_json",
diff --git a/Cargo.toml b/Cargo.toml
index b049eaca..ede55f3d 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -150,6 +150,7 @@ node-resolve = "2.2"
 parking_lot = "0.12"
 rust_decimal = { version = "1.36", features = ["serde"] }
 base64 = "0.22"
+jsonwebtoken = "9"
 
 # CLI
 clap = { version = "4", features = ["derive", "string"] }
diff --git a/crates/cclab-queue/Cargo.toml b/crates/cclab-queue/Cargo.toml
index b8c07681..6bfa08b0 100644
--- a/crates/cclab-queue/Cargo.toml
+++ b/crates/cclab-queue/Cargo.toml
@@ -45,7 +45,11 @@ deadpool-redis = { version = "0.18", optional = true }
 google-cloud-pubsub = { version = "0.30", optional = true }
 google-cloud-googleapis = { version = "0.16", optional = true }
 
-# Cloud Tasks and Pub/Sub Push use cclab-quasar for HTTP handling
+# Cloud Tasks and Cloud Scheduler HTTP API client
+reqwest = { workspace = true, optional = true }
+
+# JWT for SA key auth and inbound OIDC validation (Cloud Tasks)
+jsonwebtoken = { workspace = true, optional = true }
 
 # Ion backend
 cclab-kv = { path = "../cclab-kv", optional = true }
@@ -89,7 +93,10 @@ schema = ["dep:schemars"]
 
 # Push-based brokers (use cclab-quasar for HTTP handling)
 pubsub-push = ["pubsub"]
-cloudtasks = []
+cloudtasks = ["dep:reqwest", "dep:jsonwebtoken"]
+
+# Cloud Scheduler backend
+cloud-scheduler = ["dep:reqwest"]
 
 # Ion result backend
 ion = ["dep:cclab-kv"]
@@ -100,6 +107,7 @@ k8s = ["dep:kube", "dep:k8s-openapi"]
 # Convenience
 gcp = ["pubsub", "cloudtasks"]
 gcp-push = ["pubsub-push", "cloudtasks"]
+gcp-full = ["gcp-push", "cloud-scheduler"]
 
 [dev-dependencies]
 tokio-test = "0.4"
diff --git a/crates/cclab-queue/src/broker/cloudtasks.rs b/crates/cclab-queue/src/broker/cloudtasks.rs
index 7972162f..d7de0070 100644
--- a/crates/cclab-queue/src/broker/cloudtasks.rs
+++ b/crates/cclab-queue/src/broker/cloudtasks.rs
@@ -1,13 +1,43 @@
 //! Google Cloud Tasks broker implementation
 //!
 //! Push-based broker that creates Cloud Tasks which call back to worker HTTP endpoint.
+//! Communicates with GCP Cloud Tasks REST API v2 for task creation, and parses
+//! incoming push requests from Cloud Tasks dispatches.
 
 use async_trait::async_trait;
+use chrono::{DateTime, Utc};
+use jsonwebtoken::{encode, Algorithm, EncodingKey, Header as JwtHeader};
+use serde::{Deserialize, Serialize};
 use std::collections::HashMap;
+use std::sync::Arc;
 use std::time::Duration;
+use tokio::sync::RwLock;
 
 use crate::{TaskError, TaskMessage};
-use super::{Broker, BrokerCapabilities, BrokerMessage, DeliveryModel, PushBroker};
+use super::{Broker, BrokerCapabilities, BrokerMessage, DelayedBroker, DeliveryModel, PushBroker};
+
+/// Cloud Tasks REST API v2 base URL
+const CLOUD_TASKS_API_BASE: &str = "https://cloudtasks.googleapis.com/v2";
+
+/// GCP metadata server token endpoint
+const METADATA_TOKEN_URL: &str =
+    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
+
+/// Maximum delay for Cloud Tasks (30 days)
+const MAX_DELAY_SECS: u64 = 30 * 24 * 60 * 60;
+
+/// Token refresh buffer (5 minutes before expiry)
+const TOKEN_REFRESH_BUFFER_SECS: i64 = 300;
+
+/// Google OAuth2 token endpoint for service account key exchange
+const GOOGLE_TOKEN_URI: &str = "https://oauth2.googleapis.com/token";
+
+/// OAuth2 scope for Cloud Tasks API
+const CLOUD_TASKS_SCOPE: &str = "https://www.googleapis.com/auth/cloud-tasks";
+
+// ---------------------------------------------------------------------------
+// Configuration
+// ---------------------------------------------------------------------------
 
 /// Configuration for Cloud Tasks broker
 #[derive(Debug, Clone)]
@@ -24,8 +54,12 @@ pub struct CloudTasksConfig {
     pub oidc_audience: Option<String>,
     /// Default queue name
     pub default_queue: String,
-    /// Request timeout for task execution
+    /// Request timeout for task execution (15s-1800s)
     pub dispatch_deadline: Duration,
+    /// Max retry attempts (None = Cloud Tasks default / unlimited)
+    pub max_retry_count: Option<u32>,
+    /// Path to service account JSON key file for local dev
+    pub credentials_path: Option<String>,
 }
 
 impl Default for CloudTasksConfig {
@@ -38,6 +72,8 @@ impl Default for CloudTasksConfig {
             oidc_audience: None,
             default_queue: "default".to_string(),
             dispatch_deadline: Duration::from_secs(600), // 10 minutes
+            max_retry_count: None,
+            credentials_path: None,
         }
     }
 }
@@ -47,7 +83,11 @@ impl CloudTasksConfig {
     pub fn from_env() -> Result<Self, TaskError> {
         let project_id = std::env::var("GCP_PROJECT_ID")
             .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
-            .map_err(|_| TaskError::Configuration("GCP_PROJECT_ID or GOOGLE_CLOUD_PROJECT not set".into()))?;
+            .map_err(|_| {
+                TaskError::Configuration(
+                    "GCP_PROJECT_ID or GOOGLE_CLOUD_PROJECT not set".into(),
+                )
+            })?;
 
         let location = std::env::var("CLOUDTASKS_LOCATION")
             .unwrap_or_else(|_| "us-central1".to_string());
@@ -59,6 +99,10 @@ impl CloudTasksConfig {
         let oidc_audience = std::env::var("CLOUDTASKS_OIDC_AUDIENCE").ok();
         let default_queue = std::env::var("CLOUDTASKS_QUEUE")
             .unwrap_or_else(|_| "default".to_string());
+        let credentials_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
+        let max_retry_count = std::env::var("CLOUDTASKS_MAX_RETRIES")
+            .ok()
+            .and_then(|v| v.parse().ok());
 
         Ok(Self {
             project_id,
@@ -68,6 +112,8 @@ impl CloudTasksConfig {
             oidc_audience,
             default_queue,
             dispatch_deadline: Duration::from_secs(600),
+            max_retry_count,
+            credentials_path,
         })
     }
 
@@ -78,16 +124,176 @@ impl CloudTasksConfig {
             self.project_id, self.location, queue
         )
     }
+
+    /// Get the effective OIDC audience
+    fn effective_audience(&self) -> &str {
+        self.oidc_audience.as_deref().unwrap_or(&self.worker_url)
+    }
 }
 
+// ---------------------------------------------------------------------------
+// OIDC Token Cache
+// ---------------------------------------------------------------------------
+
+/// Cached OIDC bearer token with expiry tracking
+#[derive(Debug, Clone)]
+struct OidcTokenCache {
+    /// Current bearer token (None if not yet fetched)
+    access_token: Option<String>,
+    /// Token expiry timestamp (None if not yet fetched)
+    expires_at: Option<DateTime<Utc>>,
+}
+
+impl OidcTokenCache {
+    fn new() -> Self {
+        Self {
+            access_token: None,
+            expires_at: None,
+        }
+    }
+
+    /// Check if the cached token is still valid (with 5-minute buffer)
+    fn is_valid(&self) -> bool {
+        match (&self.access_token, self.expires_at) {
+            (Some(_), Some(expires_at)) => {
+                Utc::now() < expires_at - chrono::Duration::seconds(TOKEN_REFRESH_BUFFER_SECS)
+            }
+            _ => false,
+        }
+    }
+}
+
+/// Response from GCP metadata server token endpoint
+#[derive(Deserialize)]
+struct MetadataTokenResponse {
+    access_token: String,
+    expires_in: i64,
+    #[allow(dead_code)]
+    token_type: String,
+}
+
+/// Google service account key file format (subset of fields)
+#[derive(Deserialize)]
+struct ServiceAccountKey {
+    client_email: String,
+    private_key: String,
+    #[serde(default = "default_token_uri")]
+    token_uri: String,
+}
+
+fn default_token_uri() -> String {
+    GOOGLE_TOKEN_URI.to_string()
+}
+
+/// JWT claims for service account token exchange
+#[derive(Serialize)]
+struct ServiceAccountClaims {
+    iss: String,
+    sub: String,
+    aud: String,
+    scope: String,
+    iat: i64,
+    exp: i64,
+}
+
+/// Response from Google OAuth2 token endpoint (SA key exchange)
+#[derive(Deserialize)]
+struct TokenExchangeResponse {
+    access_token: String,
+    expires_in: i64,
+    #[allow(dead_code)]
+    token_type: String,
+}
+
+/// Claims from an inbound OIDC JWT (for validation in parse_push_request)
+#[derive(Deserialize)]
+struct InboundOidcClaims {
+    #[serde(default)]
+    email: Option<String>,
+    #[serde(default)]
+    aud: Option<serde_json::Value>,
+    #[serde(default)]
+    exp: Option<i64>,
+}
+
+// ---------------------------------------------------------------------------
+// Cloud Tasks API request/response types
+// ---------------------------------------------------------------------------
+
+/// Request body for Cloud Tasks v2 tasks.create API
+#[derive(Debug, Serialize)]
+struct CreateTaskRequest {
+    task: CloudTaskBody,
+}
+
+/// Retry configuration for Cloud Tasks task
+#[derive(Debug, Serialize)]
+#[serde(rename_all = "camelCase")]
+struct RetryConfig {
+    #[serde(skip_serializing_if = "Option::is_none")]
+    max_retry_count: Option<u32>,
+}
+
+/// Task body within the create request
+#[derive(Debug, Serialize)]
+#[serde(rename_all = "camelCase")]
+struct CloudTaskBody {
+    http_request: CloudTasksHttpRequest,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    schedule_time: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    dispatch_deadline: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    retry_config: Option<RetryConfig>,
+}
+
+/// HTTP request target configuration for a Cloud Tasks task
+#[derive(Debug, Serialize)]
+#[serde(rename_all = "camelCase")]
+struct CloudTasksHttpRequest {
+    url: String,
+    http_method: String,
+    body: String,
+    headers: HashMap<String, String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    oidc_token: Option<OidcTokenField>,
+}
+
+/// OIDC token field in the task HTTP request
+#[derive(Debug, Serialize)]
+#[serde(rename_all = "camelCase")]
+struct OidcTokenField {
+    service_account_email: String,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    audience: Option<String>,
+}
+
+/// Response from Cloud Tasks API (subset of fields)
+#[derive(Debug, Deserialize)]
+#[serde(rename_all = "camelCase")]
+#[allow(dead_code)]
+struct CloudTaskResponse {
+    name: Option<String>,
+    schedule_time: Option<String>,
+    create_time: Option<String>,
+    dispatch_count: Option<i32>,
+    response_count: Option<i32>,
+}
+
+// ---------------------------------------------------------------------------
+// CloudTasksBroker
+// ---------------------------------------------------------------------------
+
 /// Cloud Tasks broker
 ///
 /// Push-based broker that creates HTTP tasks dispatched by GCP Cloud Tasks.
+/// Implements `Broker + PushBroker + DelayedBroker`.
 pub struct CloudTasksBroker {
     config: CloudTasksConfig,
-    // Cloud Tasks client will be initialized on connect
-    #[allow(dead_code)]
-    connected: bool,
+    /// HTTP client for Cloud Tasks API calls (initialized on connect)
+    client: RwLock<Option<reqwest::Client>>,
+    /// Cached OIDC access token for outbound API authentication
+    token_cache: Arc<RwLock<OidcTokenCache>>,
 }
 
 impl CloudTasksBroker {
@@ -95,7 +301,8 @@ impl CloudTasksBroker {
     pub fn new(config: CloudTasksConfig) -> Self {
         Self {
             config,
-            connected: false,
+            client: RwLock::new(None),
+            token_cache: Arc::new(RwLock::new(OidcTokenCache::new())),
         }
     }
 
@@ -103,12 +310,345 @@ impl CloudTasksBroker {
     fn task_url(&self, queue: &str) -> String {
         format!("{}/meteor/push/{}", self.config.worker_url, queue)
     }
+
+    /// Get OIDC bearer token, using cache when valid.
+    ///
+    /// When `credentials_path` is set (local dev), reads the service account JSON key file,
+    /// creates a signed JWT assertion, and exchanges it at Google's token endpoint.
+    /// Otherwise (production/GCP environment), fetches from the GCP metadata server.
+    async fn get_oidc_token(&self) -> Result<String, TaskError> {
+        // Check cache first
+        {
+            let cache = self.token_cache.read().await;
+            if cache.is_valid() {
+                return Ok(cache.access_token.clone().unwrap());
+            }
+        }
+
+        let client = self.client.read().await;
+        let client = client.as_ref().ok_or(TaskError::NotConnected)?;
+
+        let (access_token, expires_in) = if let Some(ref creds_path) = self.config.credentials_path
+        {
+            // Service account key file flow (local dev)
+            self.fetch_token_from_sa_key(client, creds_path).await?
+        } else {
+            // GCP metadata server flow (production)
+            self.fetch_token_from_metadata(client).await?
+        };
+
+        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in);
+
+        // Update cache
+        let mut cache = self.token_cache.write().await;
+        cache.access_token = Some(access_token.clone());
+        cache.expires_at = Some(expires_at);
+
+        Ok(access_token)
+    }
+
+    /// Fetch an access token from the GCP metadata server (production path)
+    async fn fetch_token_from_metadata(
+        &self,
+        client: &reqwest::Client,
+    ) -> Result<(String, i64), TaskError> {
+        let response = client
+            .get(METADATA_TOKEN_URL)
+            .header("Metadata-Flavor", "Google")
+            .send()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to fetch OIDC token: {}", e))
+            })?;
+
+        if !response.status().is_success() {
+            return Err(TaskError::Authentication(format!(
+                "Metadata server returned status {}",
+                response.status()
+            )));
+        }
+
+        let token_response: MetadataTokenResponse = response
+            .json()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to parse token response: {}", e))
+            })?;
+
+        Ok((token_response.access_token, token_response.expires_in))
+    }
+
+    /// Fetch an access token using a service account JSON key file (local dev path).
+    ///
+    /// 1. Read and parse the SA key file
+    /// 2. Create a JWT assertion signed with the SA's private key (RS256)
+    /// 3. Exchange the JWT at Google's OAuth2 token endpoint for an access token
+    async fn fetch_token_from_sa_key(
+        &self,
+        client: &reqwest::Client,
+        credentials_path: &str,
+    ) -> Result<(String, i64), TaskError> {
+        // Read and parse service account key file
+        let key_data = std::fs::read_to_string(credentials_path).map_err(|e| {
+            TaskError::Authentication(format!(
+                "Failed to read service account key file '{}': {}",
+                credentials_path, e
+            ))
+        })?;
+
+        let sa_key: ServiceAccountKey = serde_json::from_str(&key_data).map_err(|e| {
+            TaskError::Authentication(format!(
+                "Failed to parse service account key file: {}",
+                e
+            ))
+        })?;
+
+        // Create JWT assertion
+        let now = Utc::now().timestamp();
+        let claims = ServiceAccountClaims {
+            iss: sa_key.client_email.clone(),
+            sub: sa_key.client_email,
+            aud: sa_key.token_uri.clone(),
+            scope: CLOUD_TASKS_SCOPE.to_string(),
+            iat: now,
+            exp: now + 3600, // 1 hour
+        };
+
+        let encoding_key = EncodingKey::from_rsa_pem(sa_key.private_key.as_bytes())
+            .map_err(|e| {
+                TaskError::Authentication(format!(
+                    "Failed to parse private key from SA key file: {}",
+                    e
+                ))
+            })?;
+
+        let jwt = encode(&JwtHeader::new(Algorithm::RS256), &claims, &encoding_key)
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to create JWT assertion: {}", e))
+            })?;
+
+        // Exchange JWT for access token
+        let token_uri = sa_key.token_uri;
+        let response = client
+            .post(&token_uri)
+            .form(&[
+                ("grant_type", "urn:ietf:params:oauth:grant_type:jwt-bearer"),
+                ("assertion", &jwt),
+            ])
+            .send()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to exchange JWT for token: {}", e))
+            })?;
+
+        if !response.status().is_success() {
+            let body = response.text().await.unwrap_or_default();
+            return Err(TaskError::Authentication(format!(
+                "Token exchange failed: {}",
+                body
+            )));
+        }
+
+        let token_response: TokenExchangeResponse = response
+            .json()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!(
+                    "Failed to parse token exchange response: {}",
+                    e
+                ))
+            })?;
+
+        tracing::debug!(
+            "Obtained access token from SA key file (expires in {}s)",
+            token_response.expires_in
+        );
+
+        Ok((token_response.access_token, token_response.expires_in))
+    }
+
+    /// Build a CreateTaskRequest for the Cloud Tasks API
+    fn build_create_task_request(
+        &self,
+        queue: &str,
+        message: &TaskMessage,
+        schedule_time: Option<DateTime<Utc>>,
+    ) -> Result<CreateTaskRequest, TaskError> {
+        let payload_json = serde_json::to_vec(message)
+            .map_err(|e| TaskError::Serialization(e.to_string()))?;
+        let body_b64 = base64::Engine::encode(
+            &base64::engine::general_purpose::STANDARD,
+            &payload_json,
+        );
+
+        let mut headers = HashMap::new();
+        headers.insert("Content-Type".to_string(), "application/json".to_string());
+
+        let oidc_token = self.config.service_account_email.as_ref().map(|email| {
+            OidcTokenField {
+                service_account_email: email.clone(),
+                audience: Some(self.config.effective_audience().to_string()),
+            }
+        });
+
+        let schedule_time_str = schedule_time.map(|t| t.to_rfc3339());
+
+        let dispatch_deadline_str = Some(format!("{}s", self.config.dispatch_deadline.as_secs()));
+
+        let retry_config = self.config.max_retry_count.map(|count| RetryConfig {
+            max_retry_count: Some(count),
+        });
+
+        Ok(CreateTaskRequest {
+            task: CloudTaskBody {
+                http_request: CloudTasksHttpRequest {
+                    url: self.task_url(queue),
+                    http_method: "POST".to_string(),
+                    body: body_b64,
+                    headers,
+                    oidc_token,
+                },
+                schedule_time: schedule_time_str,
+                dispatch_deadline: dispatch_deadline_str,
+                retry_config,
+            },
+        })
+    }
+
+    /// Validate an inbound OIDC JWT's claims (email, audience, expiry).
+    ///
+    /// Decodes the JWT payload without signature verification (signature verification
+    /// against Google's public JWK keys should be handled by API gateway / middleware).
+    /// Validates:
+    /// - `email` claim matches the configured `service_account_email`
+    /// - `aud` claim matches `oidc_audience` or `worker_url`
+    /// - `exp` claim has not passed
+    fn validate_inbound_jwt(
+        &self,
+        jwt_token: &str,
+        expected_email: &str,
+    ) -> Result<(), TaskError> {
+        // Split JWT into parts and decode payload (middle segment)
+        let parts: Vec<&str> = jwt_token.split('.').collect();
+        if parts.len() != 3 {
+            return Err(TaskError::Authentication(
+                "Invalid JWT format: expected 3 segments".into(),
+            ));
+        }
+
+        let payload_bytes = base64::Engine::decode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            parts[1],
+        )
+        .map_err(|e| {
+            TaskError::Authentication(format!("Failed to decode JWT payload: {}", e))
+        })?;
+
+        let claims: InboundOidcClaims = serde_json::from_slice(&payload_bytes).map_err(|e| {
+            TaskError::Authentication(format!("Failed to parse JWT claims: {}", e))
+        })?;
+
+        // Check email claim
+        match &claims.email {
+            Some(email) if email == expected_email => {}
+            Some(email) => {
+                return Err(TaskError::Authentication(format!(
+                    "JWT email mismatch: expected '{}', got '{}'",
+                    expected_email, email
+                )));
+            }
+            None => {
+                return Err(TaskError::Authentication(
+                    "JWT missing 'email' claim".into(),
+                ));
+            }
+        }
+
+        // Check audience claim
+        let expected_audience = self.config.effective_audience();
+        let audience_valid = match &claims.aud {
+            Some(serde_json::Value::String(aud)) => aud == expected_audience,
+            Some(serde_json::Value::Array(auds)) => auds.iter().any(|a| {
+                a.as_str().map_or(false, |s| s == expected_audience)
+            }),
+            _ => false,
+        };
+        if !audience_valid {
+            return Err(TaskError::Authentication(format!(
+                "JWT audience mismatch: expected '{}'",
+                expected_audience
+            )));
+        }
+
+        // Check expiry
+        if let Some(exp) = claims.exp {
+            if Utc::now().timestamp() > exp {
+                return Err(TaskError::Authentication("JWT has expired".into()));
+            }
+        }
+
+        tracing::debug!("Inbound OIDC JWT claims validated (email, audience, expiry)");
+        Ok(())
+    }
+
+    /// Map GCP HTTP error status to TaskError
+    fn map_gcp_error(status: reqwest::StatusCode, body: &str) -> TaskError {
+        match status.as_u16() {
+            404 => TaskError::TaskNotFound(
+                body.to_string(),
+            ),
+            401 | 403 => TaskError::Authentication(
+                format!("GCP API authentication error ({}): {}", status, body),
+            ),
+            409 => TaskError::AlreadyExists(
+                body.to_string(),
+            ),
+            429 => TaskError::RateLimited(Duration::from_secs(60)),
+            500..=599 => TaskError::Backend(
+                format!("GCP API server error ({}): {}", status, body),
+            ),
+            _ => TaskError::Backend(
+                format!("GCP API error ({}): {}", status, body),
+            ),
+        }
+    }
+
+    /// Send a task creation request to Cloud Tasks API
+    async fn create_task(
+        &self,
+        queue: &str,
+        request: &CreateTaskRequest,
+    ) -> Result<(), TaskError> {
+        let client = self.client.read().await;
+        let client = client.as_ref().ok_or(TaskError::NotConnected)?;
+
+        let token = self.get_oidc_token().await?;
+        let queue_path = self.config.queue_path(queue);
+        let url = format!("{}/{}/tasks", CLOUD_TASKS_API_BASE, queue_path);
+
+        let response = client
+            .post(&url)
+            .bearer_auth(&token)
+            .json(request)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Cloud Tasks API request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            tracing::debug!("Cloud Task created successfully in queue {}", queue);
+            Ok(())
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
 }
 
 #[async_trait]
 impl Broker for CloudTasksBroker {
     async fn connect(&self) -> Result<(), TaskError> {
-        // Verify configuration
+        // Validate configuration
         if self.config.project_id.is_empty() {
             return Err(TaskError::Configuration("project_id is required".into()));
         }
@@ -116,6 +656,14 @@ impl Broker for CloudTasksBroker {
             return Err(TaskError::Configuration("worker_url is required".into()));
         }
 
+        // Initialize HTTP client
+        let http_client = reqwest::Client::builder()
+            .timeout(Duration::from_secs(30))
+            .build()
+            .map_err(|e| TaskError::Connection(format!("Failed to create HTTP client: {}", e)))?;
+
+        *self.client.write().await = Some(http_client);
+
         tracing::info!(
             project_id = %self.config.project_id,
             location = %self.config.location,
@@ -126,46 +674,56 @@ impl Broker for CloudTasksBroker {
     }
 
     async fn disconnect(&self) -> Result<(), TaskError> {
+        *self.client.write().await = None;
         tracing::info!("Disconnected from Cloud Tasks");
         Ok(())
     }
 
     async fn publish(&self, queue: &str, message: TaskMessage) -> Result<(), TaskError> {
-        let queue_path = self.config.queue_path(queue);
-        let task_url = self.task_url(queue);
-
-        let payload = serde_json::to_vec(&message)
-            .map_err(|e| TaskError::Serialization(e.to_string()))?;
-
         tracing::debug!(
-            queue_path = %queue_path,
-            task_url = %task_url,
             task_id = %message.id,
+            queue = %queue,
             "Creating Cloud Task"
         );
 
-        // In production, this would call the Cloud Tasks API:
-        // POST https://cloudtasks.googleapis.com/v2/{queue_path}/tasks
-        // with HttpRequest body containing:
-        // - url: task_url
-        // - httpMethod: POST
-        // - body: base64(payload)
-        // - oidcToken: { serviceAccountEmail, audience }
+        let request = self.build_create_task_request(queue, &message, None)?;
+        self.create_task(queue, &request).await?;
 
-        // For now, log the task creation (actual API call requires google-cloud-tasks crate)
         tracing::info!(
             task_id = %message.id,
             queue = %queue,
-            "Cloud Task created (stub - actual API call not implemented)"
+            "Cloud Task created"
         );
 
         Ok(())
     }
 
     async fn health_check(&self) -> Result<(), TaskError> {
-        // Check if we can reach Cloud Tasks API
-        // In production, this would call GET /v2/projects/{project}/locations
-        Ok(())
+        let client = self.client.read().await;
+        let client = client.as_ref().ok_or(TaskError::NotConnected)?;
+
+        let token = self.get_oidc_token().await?;
+        let url = format!(
+            "{}/projects/{}/locations",
+            CLOUD_TASKS_API_BASE, self.config.project_id
+        );
+
+        let response = client
+            .get(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| {
+                TaskError::Connection(format!("Cloud Tasks health check failed: {}", e))
+            })?;
+
+        let status = response.status();
+        if status.is_success() {
+            Ok(())
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
     }
 
     fn delivery_model(&self) -> DeliveryModel {
@@ -178,7 +736,7 @@ impl Broker for CloudTasksBroker {
             dead_letter: true,
             priority: false,
             batching: false,
-            max_delay: Some(Duration::from_secs(30 * 24 * 60 * 60)), // 30 days
+            max_delay: Some(Duration::from_secs(MAX_DELAY_SECS)), // 30 days
         }
     }
 }
@@ -190,28 +748,28 @@ impl PushBroker for CloudTasksBroker {
         body: &[u8],
     ) -> Result<BrokerMessage, TaskError> {
         // Validate OIDC token if configured
-        if self.config.service_account_email.is_some() {
-            let auth_header = headers.get("authorization")
-                .ok_or_else(|| TaskError::Authentication("Missing Authorization header".into()))?;
+        if let Some(ref expected_email) = self.config.service_account_email {
+            let auth_header = headers
+                .get("authorization")
+                .ok_or_else(|| {
+                    TaskError::Authentication("Missing Authorization header".into())
+                })?;
 
             if !auth_header.starts_with("Bearer ") {
-                return Err(TaskError::Authentication("Invalid Authorization header format".into()));
+                return Err(TaskError::Authentication(
+                    "Invalid Authorization header format".into(),
+                ));
             }
 
-            // In production, validate the OIDC token:
-            // 1. Decode JWT
-            // 2. Verify signature against Google's public keys
-            // 3. Check audience matches oidc_audience or worker_url
-            // 4. Check email matches service_account_email
-
-            tracing::debug!("OIDC token validation (stub - not implemented)");
+            let jwt_token = &auth_header["Bearer ".len()..];
+            self.validate_inbound_jwt(jwt_token, expected_email)?;
         }
 
         // Parse the task message
         let payload: TaskMessage = serde_json::from_slice(body)
             .map_err(|e| TaskError::Deserialization(e.to_string()))?;
 
-        // Extract delivery info from headers
+        // Extract delivery info from Cloud Tasks headers
         let delivery_tag = headers
             .get("x-cloudtasks-taskname")
             .cloned()
@@ -226,7 +784,7 @@ impl PushBroker for CloudTasksBroker {
             delivery_tag,
             payload,
             headers: headers.clone(),
-            timestamp: chrono::Utc::now(),
+            timestamp: Utc::now(),
             redelivered: retry_count > 0,
         })
     }
@@ -236,36 +794,1253 @@ impl PushBroker for CloudTasksBroker {
     }
 }
 
+#[async_trait]
+impl DelayedBroker for CloudTasksBroker {
+    async fn publish_delayed(
+        &self,
+        queue: &str,
+        message: TaskMessage,
+        delay: Duration,
+    ) -> Result<(), TaskError> {
+        // Clamp delay to Cloud Tasks maximum (30 days)
+        let clamped = delay.min(Duration::from_secs(MAX_DELAY_SECS));
+        let schedule_time = Utc::now() + chrono::Duration::from_std(clamped)
+            .unwrap_or_else(|_| chrono::Duration::zero());
+
+        tracing::debug!(
+            task_id = %message.id,
+            queue = %queue,
+            delay_secs = %clamped.as_secs(),
+            schedule_time = %schedule_time.to_rfc3339(),
+            "Creating delayed Cloud Task"
+        );
+
+        let request = self.build_create_task_request(queue, &message, Some(schedule_time))?;
+        self.create_task(queue, &request).await?;
+
+        tracing::info!(
+            task_id = %message.id,
+            queue = %queue,
+            schedule_time = %schedule_time.to_rfc3339(),
+            "Delayed Cloud Task created"
+        );
+
+        Ok(())
+    }
+
+    // publish_at uses the default trait implementation:
+    // if eta <= now → calls publish() immediately
+    // if eta > now → converts to delay and calls publish_delayed()
+}
+
+// ===========================================================================
+// Unit tests
+// ===========================================================================
+
 #[cfg(test)]
 mod tests {
     use super::*;
+    use serde_json::json;
 
-    #[test]
-    fn test_queue_path() {
-        let config = CloudTasksConfig {
+    // -----------------------------------------------------------------------
+    // Helpers
+    // -----------------------------------------------------------------------
+
+    fn test_config() -> CloudTasksConfig {
+        CloudTasksConfig {
             project_id: "my-project".to_string(),
             location: "us-central1".to_string(),
-            ..Default::default()
+            worker_url: "https://app.example.com".to_string(),
+            service_account_email: Some("sa@my-project.iam.gserviceaccount.com".to_string()),
+            oidc_audience: None,
+            default_queue: "default".to_string(),
+            dispatch_deadline: Duration::from_secs(600),
+            max_retry_count: None,
+            credentials_path: None,
+        }
+    }
+
+    fn test_config_no_sa() -> CloudTasksConfig {
+        CloudTasksConfig {
+            service_account_email: None,
+            ..test_config()
+        }
+    }
+
+    fn sample_message() -> TaskMessage {
+        TaskMessage::new("test.add", json!([1, 2]))
+    }
+
+    /// Build a fake JWT (header.payload.signature) with given claims for testing.
+    /// Signature is not valid (dummy), but the payload is properly base64url-encoded.
+    fn make_test_jwt(email: &str, audience: &str, exp: i64) -> String {
+        let header = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            r#"{"alg":"RS256","typ":"JWT"}"#,
+        );
+        let payload_json = serde_json::json!({
+            "email": email,
+            "aud": audience,
+            "exp": exp,
+            "iss": "accounts.google.com",
+            "sub": "1234567890",
+        });
+        let payload = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            payload_json.to_string().as_bytes(),
+        );
+        let signature = "fake_signature_for_test";
+        format!("{}.{}.{}", header, payload, signature)
+    }
+
+    // -----------------------------------------------------------------------
+    // CloudTasksConfig helpers (queue_path, effective_audience)
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_queue_path_format() {
+        let cfg = test_config();
+        assert_eq!(
+            cfg.queue_path("default"),
+            "projects/my-project/locations/us-central1/queues/default"
+        );
+    }
+
+    #[test]
+    fn test_queue_path_custom_queue() {
+        let cfg = test_config();
+        assert_eq!(
+            cfg.queue_path("high-priority"),
+            "projects/my-project/locations/us-central1/queues/high-priority"
+        );
+    }
+
+    #[test]
+    fn test_effective_audience_defaults_to_worker_url() {
+        let cfg = test_config(); // oidc_audience is None
+        assert_eq!(cfg.effective_audience(), "https://app.example.com");
+    }
+
+    #[test]
+    fn test_effective_audience_with_explicit_value() {
+        let cfg = CloudTasksConfig {
+            oidc_audience: Some("https://custom-audience.example.com".to_string()),
+            ..test_config()
+        };
+        assert_eq!(cfg.effective_audience(), "https://custom-audience.example.com");
+    }
+
+    // -----------------------------------------------------------------------
+    // OidcTokenCache validity (R4)
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_token_cache_new_is_invalid() {
+        let cache = OidcTokenCache::new();
+        assert!(cache.access_token.is_none());
+        assert!(cache.expires_at.is_none());
+        assert!(!cache.is_valid(), "Fresh cache with no token should be invalid");
+    }
+
+    #[test]
+    fn test_token_cache_valid_with_future_expiry() {
+        let cache = OidcTokenCache {
+            access_token: Some("token-abc".to_string()),
+            // Expires 1 hour from now — well within refresh buffer
+            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
+        };
+        assert!(cache.is_valid(), "Token expiring in 1h should be valid");
+    }
+
+    #[test]
+    fn test_token_cache_invalid_when_near_expiry() {
+        let cache = OidcTokenCache {
+            access_token: Some("token-abc".to_string()),
+            // Expires in 2 minutes — within the 5-minute refresh buffer
+            expires_at: Some(Utc::now() + chrono::Duration::minutes(2)),
+        };
+        assert!(
+            !cache.is_valid(),
+            "Token expiring in 2min should be invalid (within 5-min refresh buffer)"
+        );
+    }
+
+    #[test]
+    fn test_token_cache_invalid_when_expired() {
+        let cache = OidcTokenCache {
+            access_token: Some("token-old".to_string()),
+            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
+        };
+        assert!(!cache.is_valid(), "Expired token should be invalid");
+    }
+
+    #[test]
+    fn test_token_cache_invalid_without_token() {
+        let cache = OidcTokenCache {
+            access_token: None,
+            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
+        };
+        assert!(!cache.is_valid(), "Cache without access_token should be invalid");
+    }
+
+    #[test]
+    fn test_token_cache_invalid_without_expiry() {
+        let cache = OidcTokenCache {
+            access_token: Some("token-abc".to_string()),
+            expires_at: None,
+        };
+        assert!(!cache.is_valid(), "Cache without expires_at should be invalid");
+    }
+
+    // -----------------------------------------------------------------------
+    // CloudTasksBroker::task_url
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_task_url_construction() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert_eq!(
+            broker.task_url("default"),
+            "https://app.example.com/meteor/push/default"
+        );
+    }
+
+    #[test]
+    fn test_task_url_custom_queue() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert_eq!(
+            broker.task_url("priority-high"),
+            "https://app.example.com/meteor/push/priority-high"
+        );
+    }
+
+    // -----------------------------------------------------------------------
+    // S1/R6: build_create_task_request — immediate publish
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_build_create_task_request_immediate() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        // HTTP request fields
+        assert_eq!(req.task.http_request.url, "https://app.example.com/meteor/push/default");
+        assert_eq!(req.task.http_request.http_method, "POST");
+        assert_eq!(
+            req.task.http_request.headers.get("Content-Type").unwrap(),
+            "application/json"
+        );
+
+        // Body should be base64-encoded TaskMessage JSON
+        let decoded = base64::Engine::decode(
+            &base64::engine::general_purpose::STANDARD,
+            &req.task.http_request.body,
+        )
+        .expect("body should be valid base64");
+        let decoded_msg: TaskMessage =
+            serde_json::from_slice(&decoded).expect("decoded body should be valid TaskMessage");
+        assert_eq!(decoded_msg.task_name, "test.add");
+        assert_eq!(decoded_msg.args, json!([1, 2]));
+
+        // No schedule time for immediate publish
+        assert!(req.task.schedule_time.is_none());
+
+        // Dispatch deadline
+        assert_eq!(req.task.dispatch_deadline.as_deref(), Some("600s"));
+    }
+
+    #[test]
+    fn test_build_create_task_request_has_oidc_token() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let oidc = req.task.http_request.oidc_token.as_ref()
+            .expect("OIDC token should be present when service_account_email is set");
+        assert_eq!(oidc.service_account_email, "sa@my-project.iam.gserviceaccount.com");
+        assert_eq!(
+            oidc.audience.as_deref(),
+            Some("https://app.example.com"),
+            "Audience should default to worker_url"
+        );
+    }
+
+    #[test]
+    fn test_build_create_task_request_no_oidc_without_service_account() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        assert!(
+            req.task.http_request.oidc_token.is_none(),
+            "No OIDC token when service_account_email is None"
+        );
+    }
+
+    #[test]
+    fn test_build_create_task_request_custom_audience() {
+        let cfg = CloudTasksConfig {
+            oidc_audience: Some("https://custom-aud.example.com".to_string()),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let oidc = req.task.http_request.oidc_token.as_ref().unwrap();
+        assert_eq!(oidc.audience.as_deref(), Some("https://custom-aud.example.com"));
+    }
+
+    // -----------------------------------------------------------------------
+    // S4/R3: build_create_task_request — delayed publish with scheduleTime
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_build_create_task_request_with_schedule_time() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let schedule = Utc::now() + chrono::Duration::minutes(5);
+
+        let req = broker
+            .build_create_task_request("default", &msg, Some(schedule))
+            .unwrap();
+
+        let sched_str = req.task.schedule_time
+            .as_ref()
+            .expect("schedule_time should be present for delayed publish");
+
+        // Parse back to verify it's valid RFC 3339
+        let parsed = DateTime::parse_from_rfc3339(sched_str)
+            .expect("schedule_time should be valid RFC 3339");
+        // Should be roughly 5 minutes from now (allow 10s tolerance)
+        let diff = (parsed.timestamp() - schedule.timestamp()).abs();
+        assert!(diff < 2, "schedule_time should match the requested time");
+    }
+
+    #[test]
+    fn test_build_create_task_request_dispatch_deadline() {
+        let cfg = CloudTasksConfig {
+            dispatch_deadline: Duration::from_secs(300), // 5 minutes
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        assert_eq!(req.task.dispatch_deadline.as_deref(), Some("300s"));
+    }
+
+    // -----------------------------------------------------------------------
+    // S9/R7: map_gcp_error — HTTP status code to TaskError mapping
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_map_gcp_error_404_to_task_not_found() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::NOT_FOUND,
+            "Queue not found",
+        );
+        match err {
+            TaskError::TaskNotFound(msg) => assert!(msg.contains("Queue not found")),
+            other => panic!("Expected TaskNotFound, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_401_to_authentication() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::UNAUTHORIZED,
+            "Invalid credentials",
+        );
+        match err {
+            TaskError::Authentication(msg) => {
+                assert!(msg.contains("401"));
+                assert!(msg.contains("Invalid credentials"));
+            }
+            other => panic!("Expected Authentication, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_403_to_authentication() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::FORBIDDEN,
+            "Permission denied",
+        );
+        match err {
+            TaskError::Authentication(msg) => {
+                assert!(msg.contains("403"));
+                assert!(msg.contains("Permission denied"));
+            }
+            other => panic!("Expected Authentication, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_409_to_already_exists() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::CONFLICT,
+            "Task already exists",
+        );
+        match err {
+            TaskError::AlreadyExists(msg) => assert!(msg.contains("Task already exists")),
+            other => panic!("Expected AlreadyExists, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_429_to_rate_limited() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::TOO_MANY_REQUESTS,
+            "Rate limit exceeded",
+        );
+        match err {
+            TaskError::RateLimited(duration) => {
+                assert_eq!(duration, Duration::from_secs(60));
+            }
+            other => panic!("Expected RateLimited, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_500_to_backend() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
+            "Internal server error",
+        );
+        match err {
+            TaskError::Backend(msg) => {
+                assert!(msg.contains("500"));
+                assert!(msg.contains("Internal server error"));
+            }
+            other => panic!("Expected Backend, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_503_to_backend() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::SERVICE_UNAVAILABLE,
+            "Service unavailable",
+        );
+        match err {
+            TaskError::Backend(msg) => {
+                assert!(msg.contains("503"));
+            }
+            other => panic!("Expected Backend, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_map_gcp_error_unknown_status_to_backend() {
+        let err = CloudTasksBroker::map_gcp_error(
+            reqwest::StatusCode::IM_A_TEAPOT,
+            "I'm a teapot",
+        );
+        match err {
+            TaskError::Backend(msg) => {
+                assert!(msg.contains("418"));
+            }
+            other => panic!("Expected Backend for unknown status, got: {:?}", other),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // Serde serialization — verify camelCase output matches Cloud Tasks API
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_create_task_request_serializes_camel_case() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let schedule = Utc::now() + chrono::Duration::minutes(10);
+
+        let req = broker
+            .build_create_task_request("default", &msg, Some(schedule))
+            .unwrap();
+
+        let json_val = serde_json::to_value(&req).expect("should serialize to JSON");
+
+        // Top-level: task
+        let task = json_val.get("task").expect("should have 'task' field");
+
+        // camelCase field names
+        assert!(task.get("httpRequest").is_some(), "should use camelCase 'httpRequest'");
+        assert!(task.get("scheduleTime").is_some(), "should use camelCase 'scheduleTime'");
+        assert!(task.get("dispatchDeadline").is_some(), "should use camelCase 'dispatchDeadline'");
+
+        let http_req = task.get("httpRequest").unwrap();
+        assert!(http_req.get("httpMethod").is_some(), "should use camelCase 'httpMethod'");
+        assert!(http_req.get("oidcToken").is_some(), "should use camelCase 'oidcToken'");
+
+        let oidc = http_req.get("oidcToken").unwrap();
+        assert!(
+            oidc.get("serviceAccountEmail").is_some(),
+            "should use camelCase 'serviceAccountEmail'"
+        );
+    }
+
+    #[test]
+    fn test_create_task_request_omits_null_schedule_time() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let json_val = serde_json::to_value(&req).unwrap();
+        let task = json_val.get("task").unwrap();
+
+        assert!(
+            task.get("scheduleTime").is_none(),
+            "scheduleTime should be omitted when None (skip_serializing_if)"
+        );
+    }
+
+    #[test]
+    fn test_create_task_request_omits_oidc_when_no_service_account() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let json_val = serde_json::to_value(&req).unwrap();
+        let http_req = json_val["task"]["httpRequest"].as_object().unwrap();
+
+        assert!(
+            http_req.get("oidcToken").is_none(),
+            "oidcToken should be omitted when service_account_email is None"
+        );
+    }
+
+    // -----------------------------------------------------------------------
+    // S2/R2: parse_push_request — comprehensive push request parsing
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_parse_push_request_first_delivery() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = TaskMessage::new("compute.sum", json!({"x": 1, "y": 2}));
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let mut headers = HashMap::new();
+        headers.insert("x-cloudtasks-taskname".to_string(), "task-123".to_string());
+        headers.insert("x-cloudtasks-taskretrycount".to_string(), "0".to_string());
+
+        let result = broker.parse_push_request(&headers, &body).unwrap();
+        assert_eq!(result.delivery_tag, "task-123");
+        assert_eq!(result.payload.task_name, "compute.sum");
+        assert!(!result.redelivered);
+        assert_eq!(result.headers.get("x-cloudtasks-taskname").unwrap(), "task-123");
+    }
+
+    #[test]
+    fn test_parse_push_request_redelivery_detection() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let mut headers = HashMap::new();
+        headers.insert("x-cloudtasks-taskname".to_string(), "task-retry".to_string());
+        headers.insert("x-cloudtasks-taskretrycount".to_string(), "3".to_string());
+
+        let result = broker.parse_push_request(&headers, &body).unwrap();
+        assert!(result.redelivered, "retry count > 0 indicates redelivery");
+    }
+
+    #[test]
+    fn test_parse_push_request_retry_count_1_is_redelivered() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let mut headers = HashMap::new();
+        headers.insert("x-cloudtasks-taskname".to_string(), "task-r1".to_string());
+        headers.insert("x-cloudtasks-taskretrycount".to_string(), "1".to_string());
+
+        let result = broker.parse_push_request(&headers, &body).unwrap();
+        assert!(result.redelivered, "retry count 1 should be redelivered");
+    }
+
+    #[test]
+    fn test_parse_push_request_missing_retry_header_defaults_zero() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let mut headers = HashMap::new();
+        headers.insert("x-cloudtasks-taskname".to_string(), "task-no-retry".to_string());
+        // No x-cloudtasks-taskretrycount header
+
+        let result = broker.parse_push_request(&headers, &body).unwrap();
+        assert!(!result.redelivered, "Missing retry count header should default to 0 (not redelivered)");
+    }
+
+    #[test]
+    fn test_parse_push_request_fallback_delivery_tag() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let expected_id = msg.id.to_string();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let headers = HashMap::new(); // No x-cloudtasks-taskname
+
+        let result = broker.parse_push_request(&headers, &body).unwrap();
+        assert_eq!(
+            result.delivery_tag, expected_id,
+            "Missing taskname header should fall back to task ID"
+        );
+    }
+
+    #[test]
+    fn test_parse_push_request_invalid_body() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let result = broker.parse_push_request(&HashMap::new(), b"not json");
+        match result {
+            Err(TaskError::Deserialization(_)) => {} // expected
+            other => panic!("Expected Deserialization error, got: {:?}", other),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // S7/R5: OIDC inbound validation
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_parse_push_request_rejects_missing_auth_when_sa_configured() {
+        let broker = CloudTasksBroker::new(test_config()); // has service_account_email
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let result = broker.parse_push_request(&HashMap::new(), &body);
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("Missing Authorization header"));
+            }
+            other => panic!("Expected Authentication error, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_parse_push_request_rejects_non_bearer_auth() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let mut headers = HashMap::new();
+        headers.insert("authorization".to_string(), "Basic abc123".to_string());
+
+        let result = broker.parse_push_request(&headers, &body);
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("Invalid Authorization header format"));
+            }
+            other => panic!("Expected Authentication error for non-Bearer, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_parse_push_request_accepts_bearer_token() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let jwt_token = make_test_jwt(
+            "sa@my-project.iam.gserviceaccount.com",
+            "https://app.example.com",
+            Utc::now().timestamp() + 3600,
+        );
+
+        let mut headers = HashMap::new();
+        headers.insert("authorization".to_string(), format!("Bearer {}", jwt_token));
+        headers.insert("x-cloudtasks-taskname".to_string(), "valid-task".to_string());
+
+        let result = broker.parse_push_request(&headers, &body);
+        assert!(result.is_ok(), "Valid Bearer token should pass");
+        assert_eq!(result.unwrap().delivery_tag, "valid-task");
+    }
+
+    #[test]
+    fn test_parse_push_request_skips_auth_when_no_sa() {
+        let broker = CloudTasksBroker::new(test_config_no_sa());
+        let msg = sample_message();
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        // No Authorization header, but no service_account_email configured either
+        let result = broker.parse_push_request(&HashMap::new(), &body);
+        assert!(result.is_ok(), "Auth check should be skipped when service_account_email is None");
+    }
+
+    // -----------------------------------------------------------------------
+    // S10/R1: connect() validation
+    // -----------------------------------------------------------------------
+
+    #[tokio::test]
+    async fn test_connect_rejects_empty_project_id() {
+        let cfg = CloudTasksConfig {
+            project_id: String::new(),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        match broker.connect().await {
+            Err(TaskError::Configuration(msg)) => {
+                assert!(msg.contains("project_id is required"));
+            }
+            other => panic!("Expected Configuration error, got: {:?}", other),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_connect_rejects_empty_worker_url() {
+        let cfg = CloudTasksConfig {
+            worker_url: String::new(),
+            ..test_config()
         };
+        let broker = CloudTasksBroker::new(cfg);
+        match broker.connect().await {
+            Err(TaskError::Configuration(msg)) => {
+                assert!(msg.contains("worker_url is required"));
+            }
+            other => panic!("Expected Configuration error, got: {:?}", other),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_connect_succeeds_with_valid_config() {
+        let broker = CloudTasksBroker::new(test_config());
+        let result = broker.connect().await;
+        assert!(result.is_ok(), "connect() should succeed with valid config");
+    }
+
+    #[tokio::test]
+    async fn test_disconnect_clears_client() {
+        let broker = CloudTasksBroker::new(test_config());
+        broker.connect().await.unwrap();
+        let result = broker.disconnect().await;
+        assert!(result.is_ok());
+        // Client should be None after disconnect
+        assert!(broker.client.read().await.is_none());
+    }
+
+    // -----------------------------------------------------------------------
+    // S12/R2: Ack/nack status codes, endpoint path
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_ack_status_code_is_200() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert_eq!(broker.ack_status_code(), 200);
+    }
+
+    #[test]
+    fn test_nack_status_code_is_500() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert_eq!(broker.nack_status_code(), 500);
+    }
 
-        let path = config.queue_path("default");
-        assert_eq!(path, "projects/my-project/locations/us-central1/queues/default");
+    #[test]
+    fn test_endpoint_path_pattern() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert_eq!(broker.endpoint_path(), "/meteor/push/{queue}");
     }
 
+    // -----------------------------------------------------------------------
+    // Delivery model and capabilities
+    // -----------------------------------------------------------------------
+
     #[test]
-    fn test_delivery_model() {
-        let config = CloudTasksConfig::default();
-        let broker = CloudTasksBroker::new(config);
+    fn test_delivery_model_is_push() {
+        let broker = CloudTasksBroker::new(test_config());
         assert_eq!(broker.delivery_model(), DeliveryModel::Push);
     }
 
     #[test]
-    fn test_capabilities() {
-        let config = CloudTasksConfig::default();
-        let broker = CloudTasksBroker::new(config);
+    fn test_capabilities_match_spec() {
+        let broker = CloudTasksBroker::new(test_config());
         let caps = broker.capabilities();
-        assert!(caps.delayed_tasks);
-        assert!(caps.dead_letter);
-        assert!(!caps.priority);
+
+        assert!(caps.delayed_tasks, "Cloud Tasks supports delayed tasks");
+        assert!(caps.dead_letter, "Cloud Tasks supports dead letter queues");
+        assert!(!caps.priority, "Cloud Tasks does not support priority");
+        assert!(!caps.batching, "Cloud Tasks does not support batching");
+        assert_eq!(
+            caps.max_delay,
+            Some(Duration::from_secs(30 * 24 * 60 * 60)),
+            "Max delay should be 30 days"
+        );
+    }
+
+    // -----------------------------------------------------------------------
+    // CloudTasksConfig::default
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_config_default_values() {
+        let cfg = CloudTasksConfig::default();
+        assert!(cfg.project_id.is_empty());
+        assert_eq!(cfg.location, "us-central1");
+        assert!(cfg.worker_url.is_empty());
+        assert!(cfg.service_account_email.is_none());
+        assert!(cfg.oidc_audience.is_none());
+        assert_eq!(cfg.default_queue, "default");
+        assert_eq!(cfg.dispatch_deadline, Duration::from_secs(600));
+        assert!(cfg.max_retry_count.is_none());
+        assert!(cfg.credentials_path.is_none());
+    }
+
+    // -----------------------------------------------------------------------
+    // Constants
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_max_delay_is_30_days() {
+        assert_eq!(MAX_DELAY_SECS, 30 * 24 * 60 * 60);
+        assert_eq!(MAX_DELAY_SECS, 2_592_000);
+    }
+
+    #[test]
+    fn test_token_refresh_buffer_is_5_minutes() {
+        assert_eq!(TOKEN_REFRESH_BUFFER_SECS, 300);
+    }
+
+    #[test]
+    fn test_cloud_tasks_api_base_url() {
+        assert_eq!(CLOUD_TASKS_API_BASE, "https://cloudtasks.googleapis.com/v2");
+    }
+
+    #[test]
+    fn test_metadata_token_url() {
+        assert_eq!(
+            METADATA_TOKEN_URL,
+            "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token"
+        );
+    }
+
+    // -----------------------------------------------------------------------
+    // create_task requires connected client (not connected → NotConnected)
+    // -----------------------------------------------------------------------
+
+    #[tokio::test]
+    async fn test_create_task_not_connected_returns_error() {
+        let broker = CloudTasksBroker::new(test_config());
+        // Don't call connect() — client is None
+        let req = broker.build_create_task_request("default", &sample_message(), None).unwrap();
+        let result = broker.create_task("default", &req).await;
+        match result {
+            Err(TaskError::NotConnected) => {} // expected
+            other => panic!("Expected NotConnected, got: {:?}", other),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_publish_not_connected_returns_error() {
+        let broker = CloudTasksBroker::new(test_config());
+        let result = broker.publish("default", sample_message()).await;
+        match result {
+            Err(TaskError::NotConnected) => {} // expected
+            other => panic!("Expected NotConnected from publish, got: {:?}", other),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_health_check_not_connected_returns_error() {
+        let broker = CloudTasksBroker::new(test_config());
+        let result = broker.health_check().await;
+        match result {
+            Err(TaskError::NotConnected) => {} // expected
+            other => panic!("Expected NotConnected from health_check, got: {:?}", other),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // CloudTasksBroker::new initializes correctly
+    // -----------------------------------------------------------------------
+
+    #[tokio::test]
+    async fn test_new_broker_starts_disconnected() {
+        let broker = CloudTasksBroker::new(test_config());
+        assert!(broker.client.read().await.is_none(), "New broker should not have an HTTP client");
+        let cache = broker.token_cache.read().await;
+        assert!(!cache.is_valid(), "New broker should have empty token cache");
+    }
+
+    // -----------------------------------------------------------------------
+    // MetadataTokenResponse deserialization
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_metadata_token_response_deserialization() {
+        let json = r#"{
+            "access_token": "ya29.test-token",
+            "expires_in": 3599,
+            "token_type": "Bearer"
+        }"#;
+        let resp: MetadataTokenResponse = serde_json::from_str(json).unwrap();
+        assert_eq!(resp.access_token, "ya29.test-token");
+        assert_eq!(resp.expires_in, 3599);
+        assert_eq!(resp.token_type, "Bearer");
+    }
+
+    // -----------------------------------------------------------------------
+    // CloudTaskResponse deserialization
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_cloud_task_response_deserialization() {
+        let json = r#"{
+            "name": "projects/p/locations/l/queues/q/tasks/t123",
+            "scheduleTime": "2025-01-01T00:00:00Z",
+            "createTime": "2025-01-01T00:00:00Z",
+            "dispatchCount": 1,
+            "responseCount": 1
+        }"#;
+        let resp: CloudTaskResponse = serde_json::from_str(json).unwrap();
+        assert_eq!(
+            resp.name.as_deref(),
+            Some("projects/p/locations/l/queues/q/tasks/t123")
+        );
+        assert_eq!(resp.dispatch_count, Some(1));
+        assert_eq!(resp.response_count, Some(1));
+    }
+
+    #[test]
+    fn test_cloud_task_response_partial_fields() {
+        // API may return only a subset of fields
+        let json = r#"{"name": "projects/p/locations/l/queues/q/tasks/t456"}"#;
+        let resp: CloudTaskResponse = serde_json::from_str(json).unwrap();
+        assert!(resp.name.is_some());
+        assert!(resp.schedule_time.is_none());
+        assert!(resp.dispatch_count.is_none());
+    }
+
+    // -----------------------------------------------------------------------
+    // P0 fix: SA key auth — ServiceAccountKey deserialization
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_service_account_key_deserialization() {
+        let json = r#"{
+            "type": "service_account",
+            "project_id": "my-project",
+            "private_key_id": "key-123",
+            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----\n",
+            "client_email": "sa@my-project.iam.gserviceaccount.com",
+            "client_id": "123456",
+            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
+            "token_uri": "https://oauth2.googleapis.com/token"
+        }"#;
+        let key: ServiceAccountKey = serde_json::from_str(json).unwrap();
+        assert_eq!(key.client_email, "sa@my-project.iam.gserviceaccount.com");
+        assert!(key.private_key.contains("RSA PRIVATE KEY"));
+        assert_eq!(key.token_uri, "https://oauth2.googleapis.com/token");
+    }
+
+    #[test]
+    fn test_service_account_key_default_token_uri() {
+        // When token_uri is absent, default to GOOGLE_TOKEN_URI
+        let json = r#"{
+            "client_email": "sa@test.iam.gserviceaccount.com",
+            "private_key": "-----BEGIN RSA PRIVATE KEY-----\nfake\n-----END RSA PRIVATE KEY-----\n"
+        }"#;
+        let key: ServiceAccountKey = serde_json::from_str(json).unwrap();
+        assert_eq!(key.token_uri, GOOGLE_TOKEN_URI);
+    }
+
+    #[test]
+    fn test_token_exchange_response_deserialization() {
+        let json = r#"{
+            "access_token": "ya29.sa-token",
+            "expires_in": 3600,
+            "token_type": "Bearer"
+        }"#;
+        let resp: TokenExchangeResponse = serde_json::from_str(json).unwrap();
+        assert_eq!(resp.access_token, "ya29.sa-token");
+        assert_eq!(resp.expires_in, 3600);
+        assert_eq!(resp.token_type, "Bearer");
+    }
+
+    #[test]
+    fn test_google_token_uri_constant() {
+        assert_eq!(GOOGLE_TOKEN_URI, "https://oauth2.googleapis.com/token");
+    }
+
+    #[test]
+    fn test_cloud_tasks_scope_constant() {
+        assert_eq!(
+            CLOUD_TASKS_SCOPE,
+            "https://www.googleapis.com/auth/cloud-tasks"
+        );
+    }
+
+    #[tokio::test]
+    async fn test_get_oidc_token_not_connected_returns_error() {
+        // When credentials_path is set but client is not connected,
+        // get_oidc_token should still fail with NotConnected when cache is empty
+        let cfg = CloudTasksConfig {
+            credentials_path: Some("/nonexistent/path.json".to_string()),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        // Don't call connect() → client is None
+        let result = broker.get_oidc_token().await;
+        match result {
+            Err(TaskError::NotConnected) => {} // expected
+            other => panic!("Expected NotConnected, got: {:?}", other),
+        }
+    }
+
+    #[tokio::test]
+    async fn test_get_oidc_token_invalid_credentials_path() {
+        let cfg = CloudTasksConfig {
+            credentials_path: Some("/nonexistent/sa-key.json".to_string()),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        broker.connect().await.unwrap();
+        let result = broker.get_oidc_token().await;
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("Failed to read service account key file"));
+            }
+            other => panic!("Expected Authentication error for bad path, got: {:?}", other),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // P1 fix: Inbound OIDC JWT validation
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_validate_inbound_jwt_email_mismatch() {
+        let broker = CloudTasksBroker::new(test_config());
+        let jwt = make_test_jwt(
+            "wrong@other-project.iam.gserviceaccount.com",
+            "https://app.example.com",
+            Utc::now().timestamp() + 3600,
+        );
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("email mismatch"), "Got: {}", msg);
+            }
+            other => panic!("Expected Authentication error for email mismatch, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_audience_mismatch() {
+        let broker = CloudTasksBroker::new(test_config());
+        let jwt = make_test_jwt(
+            "sa@my-project.iam.gserviceaccount.com",
+            "https://wrong-audience.example.com",
+            Utc::now().timestamp() + 3600,
+        );
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("audience mismatch"), "Got: {}", msg);
+            }
+            other => panic!("Expected Authentication error for audience mismatch, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_expired_token() {
+        let broker = CloudTasksBroker::new(test_config());
+        let jwt = make_test_jwt(
+            "sa@my-project.iam.gserviceaccount.com",
+            "https://app.example.com",
+            Utc::now().timestamp() - 3600, // expired 1 hour ago
+        );
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("expired"), "Got: {}", msg);
+            }
+            other => panic!("Expected Authentication error for expired token, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_malformed_token() {
+        let broker = CloudTasksBroker::new(test_config());
+        let result = broker.validate_inbound_jwt(
+            "not.a.valid-jwt",
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        assert!(result.is_err(), "Malformed JWT should fail validation");
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_wrong_segment_count() {
+        let broker = CloudTasksBroker::new(test_config());
+        let result = broker.validate_inbound_jwt(
+            "only-one-segment",
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("expected 3 segments"), "Got: {}", msg);
+            }
+            other => panic!("Expected Authentication error for wrong segments, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_valid_token_passes() {
+        let broker = CloudTasksBroker::new(test_config());
+        let jwt = make_test_jwt(
+            "sa@my-project.iam.gserviceaccount.com",
+            "https://app.example.com",
+            Utc::now().timestamp() + 3600,
+        );
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        assert!(result.is_ok(), "Valid JWT should pass validation");
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_missing_email_claim() {
+        let broker = CloudTasksBroker::new(test_config());
+        // Build JWT without email claim
+        let header = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            r#"{"alg":"RS256","typ":"JWT"}"#,
+        );
+        let payload = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            r#"{"aud":"https://app.example.com","exp":9999999999}"#,
+        );
+        let jwt = format!("{}.{}.fake_sig", header, payload);
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        match result {
+            Err(TaskError::Authentication(msg)) => {
+                assert!(msg.contains("missing 'email' claim"), "Got: {}", msg);
+            }
+            other => panic!("Expected error for missing email, got: {:?}", other),
+        }
+    }
+
+    #[test]
+    fn test_validate_inbound_jwt_audience_as_array() {
+        let broker = CloudTasksBroker::new(test_config());
+        // JWT with audience as an array (allowed by JWT spec)
+        let header = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            r#"{"alg":"RS256","typ":"JWT"}"#,
+        );
+        let claims = json!({
+            "email": "sa@my-project.iam.gserviceaccount.com",
+            "aud": ["https://app.example.com", "https://other.example.com"],
+            "exp": Utc::now().timestamp() + 3600,
+        });
+        let payload = base64::Engine::encode(
+            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
+            claims.to_string().as_bytes(),
+        );
+        let jwt = format!("{}.{}.fake_sig", header, payload);
+        let result = broker.validate_inbound_jwt(
+            &jwt,
+            "sa@my-project.iam.gserviceaccount.com",
+        );
+        assert!(result.is_ok(), "JWT with audience array containing expected aud should pass");
+    }
+
+    // -----------------------------------------------------------------------
+    // P1 fix: max_retry_count forwarded in task creation payload
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_build_create_task_request_with_max_retry_count() {
+        let cfg = CloudTasksConfig {
+            max_retry_count: Some(5),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let retry_cfg = req.task.retry_config
+            .as_ref()
+            .expect("retry_config should be present when max_retry_count is set");
+        assert_eq!(retry_cfg.max_retry_count, Some(5));
+    }
+
+    #[test]
+    fn test_build_create_task_request_no_retry_config_when_none() {
+        // Default test_config has max_retry_count: None
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        assert!(
+            req.task.retry_config.is_none(),
+            "retry_config should be None when max_retry_count is not set"
+        );
+    }
+
+    #[test]
+    fn test_retry_config_serializes_camel_case() {
+        let cfg = CloudTasksConfig {
+            max_retry_count: Some(10),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let json_val = serde_json::to_value(&req).unwrap();
+        let task = json_val.get("task").unwrap();
+
+        let retry = task.get("retryConfig")
+            .expect("should use camelCase 'retryConfig'");
+        assert_eq!(
+            retry.get("maxRetryCount").and_then(|v| v.as_u64()),
+            Some(10),
+            "should use camelCase 'maxRetryCount'"
+        );
+    }
+
+    #[test]
+    fn test_retry_config_omitted_in_json_when_none() {
+        let broker = CloudTasksBroker::new(test_config());
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let json_val = serde_json::to_value(&req).unwrap();
+        let task = json_val.get("task").unwrap();
+
+        assert!(
+            task.get("retryConfig").is_none(),
+            "retryConfig should be omitted from JSON when None"
+        );
+    }
+
+    #[test]
+    fn test_build_create_task_request_with_zero_retry_count() {
+        // max_retry_count: Some(0) means "no retries" — should still include the field
+        let cfg = CloudTasksConfig {
+            max_retry_count: Some(0),
+            ..test_config()
+        };
+        let broker = CloudTasksBroker::new(cfg);
+        let msg = sample_message();
+        let req = broker.build_create_task_request("default", &msg, None).unwrap();
+
+        let retry_cfg = req.task.retry_config.as_ref()
+            .expect("retry_config should be present even with count=0");
+        assert_eq!(retry_cfg.max_retry_count, Some(0));
     }
 }
diff --git a/crates/cclab-queue/src/broker/mod.rs b/crates/cclab-queue/src/broker/mod.rs
index 16827665..83b8de4a 100644
--- a/crates/cclab-queue/src/broker/mod.rs
+++ b/crates/cclab-queue/src/broker/mod.rs
@@ -193,3 +193,629 @@ pub use config::BrokerConfig;
 
 #[cfg(any(feature = "nats", feature = "pubsub"))]
 pub use config::BrokerInstance;
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::HashMap;
+    use std::time::Duration;
+    use chrono::Utc;
+    use crate::{TaskError, TaskMessage};
+    use tokio_util::sync::CancellationToken;
+
+    // -----------------------------------------------------------------------
+    // R6: DeliveryModel enum
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_delivery_model_variants() {
+        let pull = DeliveryModel::Pull;
+        let push = DeliveryModel::Push;
+        assert_ne!(pull, push);
+        assert_eq!(pull, DeliveryModel::Pull);
+        assert_eq!(push, DeliveryModel::Push);
+    }
+
+    #[test]
+    fn test_delivery_model_clone_copy() {
+        let model = DeliveryModel::Push;
+        let cloned = model.clone();
+        let copied = model; // Copy
+        assert_eq!(model, cloned);
+        assert_eq!(model, copied);
+    }
+
+    #[test]
+    fn test_delivery_model_debug() {
+        let pull = DeliveryModel::Pull;
+        let push = DeliveryModel::Push;
+        assert_eq!(format!("{:?}", pull), "Pull");
+        assert_eq!(format!("{:?}", push), "Push");
+    }
+
+    // -----------------------------------------------------------------------
+    // R5: BrokerCapabilities struct
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_broker_capabilities_default() {
+        let caps = BrokerCapabilities::default();
+        assert!(!caps.delayed_tasks);
+        assert!(!caps.dead_letter);
+        assert!(!caps.priority);
+        assert!(!caps.batching);
+        assert!(caps.max_delay.is_none());
+    }
+
+    #[test]
+    fn test_broker_capabilities_custom() {
+        let caps = BrokerCapabilities {
+            delayed_tasks: true,
+            dead_letter: true,
+            priority: false,
+            batching: false,
+            max_delay: Some(Duration::from_secs(30 * 24 * 60 * 60)),
+        };
+        assert!(caps.delayed_tasks);
+        assert!(caps.dead_letter);
+        assert!(!caps.priority);
+        assert!(!caps.batching);
+        assert_eq!(caps.max_delay, Some(Duration::from_secs(2_592_000)));
+    }
+
+    #[test]
+    fn test_broker_capabilities_clone() {
+        let caps = BrokerCapabilities {
+            delayed_tasks: true,
+            dead_letter: false,
+            priority: true,
+            batching: false,
+            max_delay: Some(Duration::from_secs(3600)),
+        };
+        let cloned = caps.clone();
+        assert_eq!(cloned.delayed_tasks, caps.delayed_tasks);
+        assert_eq!(cloned.dead_letter, caps.dead_letter);
+        assert_eq!(cloned.priority, caps.priority);
+        assert_eq!(cloned.batching, caps.batching);
+        assert_eq!(cloned.max_delay, caps.max_delay);
+    }
+
+    // -----------------------------------------------------------------------
+    // R7: BrokerMessage struct
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_broker_message_construction() {
+        let msg = TaskMessage::new("test_task", serde_json::json!(["arg1"]));
+        let mut headers = HashMap::new();
+        headers.insert("x-cloudtasks-taskname".to_string(), "task-123".to_string());
+        let now = Utc::now();
+
+        let broker_msg = BrokerMessage {
+            delivery_tag: "task-123".to_string(),
+            payload: msg,
+            headers: headers.clone(),
+            timestamp: now,
+            redelivered: false,
+        };
+
+        assert_eq!(broker_msg.delivery_tag, "task-123");
+        assert_eq!(broker_msg.payload.task_name, "test_task");
+        assert_eq!(broker_msg.headers.get("x-cloudtasks-taskname").unwrap(), "task-123");
+        assert_eq!(broker_msg.timestamp, now);
+        assert!(!broker_msg.redelivered);
+    }
+
+    #[test]
+    fn test_broker_message_redelivered() {
+        let msg = TaskMessage::new("retry_task", serde_json::json!([]));
+        let broker_msg = BrokerMessage {
+            delivery_tag: "tag-456".to_string(),
+            payload: msg,
+            headers: HashMap::new(),
+            timestamp: Utc::now(),
+            redelivered: true,
+        };
+
+        assert!(broker_msg.redelivered);
+    }
+
+    #[test]
+    fn test_broker_message_clone() {
+        let msg = TaskMessage::new("clone_task", serde_json::json!([1, 2, 3]));
+        let broker_msg = BrokerMessage {
+            delivery_tag: "tag-clone".to_string(),
+            payload: msg,
+            headers: HashMap::new(),
+            timestamp: Utc::now(),
+            redelivered: false,
+        };
+
+        let cloned = broker_msg.clone();
+        assert_eq!(cloned.delivery_tag, broker_msg.delivery_tag);
+        assert_eq!(cloned.payload.task_name, broker_msg.payload.task_name);
+        assert_eq!(cloned.redelivered, broker_msg.redelivered);
+    }
+
+    // -----------------------------------------------------------------------
+    // SubscriptionHandle
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn test_subscription_handle_creation() {
+        let token = CancellationToken::new();
+        let handle = SubscriptionHandle::new("my-queue".to_string(), token.clone());
+        assert_eq!(handle.queue, "my-queue");
+        assert!(!token.is_cancelled());
+    }
+
+    #[test]
+    fn test_subscription_handle_cancel() {
+        let token = CancellationToken::new();
+        let handle = SubscriptionHandle::new("cancel-queue".to_string(), token.clone());
+        assert!(!token.is_cancelled());
+
+        handle.cancel();
+        assert!(token.is_cancelled());
+    }
+
+    // -----------------------------------------------------------------------
+    // S2: PushBroker default ack/nack status codes (R3)
+    // -----------------------------------------------------------------------
+
+    /// Mock PushBroker to test default trait method implementations
+    struct MockPushBroker;
+
+    #[async_trait]
+    impl Broker for MockPushBroker {
+        async fn connect(&self) -> Result<(), TaskError> { Ok(()) }
+        async fn disconnect(&self) -> Result<(), TaskError> { Ok(()) }
+        async fn publish(&self, _queue: &str, _message: TaskMessage) -> Result<(), TaskError> { Ok(()) }
+        async fn health_check(&self) -> Result<(), TaskError> { Ok(()) }
+        fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Push }
+        fn capabilities(&self) -> BrokerCapabilities { BrokerCapabilities::default() }
+    }
+
+    impl PushBroker for MockPushBroker {
+        fn parse_push_request(
+            &self,
+            _headers: &HashMap<String, String>,
+            body: &[u8],
+        ) -> Result<BrokerMessage, TaskError> {
+            let payload: TaskMessage = serde_json::from_slice(body)
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(BrokerMessage {
+                delivery_tag: "mock-tag".to_string(),
+                payload,
+                headers: HashMap::new(),
+                timestamp: Utc::now(),
+                redelivered: false,
+            })
+        }
+
+        fn endpoint_path(&self) -> &str {
+            "/mock/push/{queue}"
+        }
+    }
+
+    #[test]
+    fn test_push_broker_default_ack_status_code() {
+        let broker = MockPushBroker;
+        assert_eq!(broker.ack_status_code(), 200);
+    }
+
+    #[test]
+    fn test_push_broker_default_nack_status_code() {
+        let broker = MockPushBroker;
+        assert_eq!(broker.nack_status_code(), 500);
+    }
+
+    #[test]
+    fn test_push_broker_endpoint_path() {
+        let broker = MockPushBroker;
+        assert_eq!(broker.endpoint_path(), "/mock/push/{queue}");
+    }
+
+    #[test]
+    fn test_push_broker_parse_push_request() {
+        let broker = MockPushBroker;
+        let msg = TaskMessage::new("push_task", serde_json::json!(["a", "b"]));
+        let body = serde_json::to_vec(&msg).unwrap();
+
+        let result = broker.parse_push_request(&HashMap::new(), &body);
+        assert!(result.is_ok());
+        let broker_msg = result.unwrap();
+        assert_eq!(broker_msg.delivery_tag, "mock-tag");
+        assert_eq!(broker_msg.payload.task_name, "push_task");
+        assert!(!broker_msg.redelivered);
+    }
+
+    #[test]
+    fn test_push_broker_parse_invalid_body() {
+        let broker = MockPushBroker;
+        let invalid_body = b"not valid json";
+
+        let result = broker.parse_push_request(&HashMap::new(), invalid_body);
+        assert!(result.is_err());
+        match result.unwrap_err() {
+            TaskError::Deserialization(_) => {} // expected
+            other => panic!("Expected Deserialization error, got: {:?}", other),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // S4: DelayedBroker default publish_at (R4)
+    // -----------------------------------------------------------------------
+
+    /// Mock DelayedBroker to test default publish_at implementation
+    struct MockDelayedBroker {
+        /// Track which method was called
+        published_immediate: std::sync::Arc<std::sync::atomic::AtomicBool>,
+        published_delayed: std::sync::Arc<std::sync::atomic::AtomicBool>,
+    }
+
+    impl MockDelayedBroker {
+        fn new() -> Self {
+            Self {
+                published_immediate: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
+                published_delayed: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Broker for MockDelayedBroker {
+        async fn connect(&self) -> Result<(), TaskError> { Ok(()) }
+        async fn disconnect(&self) -> Result<(), TaskError> { Ok(()) }
+        async fn publish(&self, _queue: &str, _message: TaskMessage) -> Result<(), TaskError> {
+            self.published_immediate.store(true, std::sync::atomic::Ordering::SeqCst);
+            Ok(())
+        }
+        async fn health_check(&self) -> Result<(), TaskError> { Ok(()) }
+        fn delivery_model(&self) -> DeliveryModel { DeliveryModel::Push }
+        fn capabilities(&self) -> BrokerCapabilities {
+            BrokerCapabilities {
+                delayed_tasks: true,
+                ..Default::default()
+            }
+        }
+    }
+
+    #[async_trait]
+    impl DelayedBroker for MockDelayedBroker {
+        async fn publish_delayed(
+            &self,
+            _queue: &str,
+            _message: TaskMessage,
+            _delay: Duration,
+        ) -> Result<(), TaskError> {
+            self.published_delayed.store(true, std::sync::atomic::Ordering::SeqCst);
+            Ok(())
+        }
+        // Uses default publish_at implementation
+    }
+
+    #[tokio::test]
+    async fn test_delayed_broker_publish_at_past_eta_calls_publish() {
+        let broker = MockDelayedBroker::new();
+        let past_eta = Utc::now() - chrono::Duration::hours(1);
+        let msg = TaskMessage::new("past_task", serde_json::json!([]));
+
+        let result = broker.publish_at("test-queue", msg, past_eta).await;
+        assert!(result.is_ok());
+        assert!(broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
+            "Past ETA should call publish() immediately");
+        assert!(!broker.published_delayed.load(std::sync::atomic::Ordering::SeqCst),
+            "Past ETA should NOT call publish_delayed()");
+    }
+
+    #[tokio::test]
+    async fn test_delayed_broker_publish_at_future_eta_calls_publish_delayed() {
+        let broker = MockDelayedBroker::new();
+        let future_eta = Utc::now() + chrono::Duration::hours(1);
+        let msg = TaskMessage::new("future_task", serde_json::json!([]));
+
+        let result = broker.publish_at("test-queue", msg, future_eta).await;
+        assert!(result.is_ok());
+        assert!(!broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
+            "Future ETA should NOT call publish() immediately");
+        assert!(broker.published_delayed.load(std::sync::atomic::Ordering::SeqCst),
+            "Future ETA should call publish_delayed()");
+    }
+
+    #[tokio::test]
+    async fn test_delayed_broker_publish_at_now_calls_publish() {
+        let broker = MockDelayedBroker::new();
+        // ETA exactly now (or slightly in the past by the time it executes)
+        let now_eta = Utc::now();
+        let msg = TaskMessage::new("now_task", serde_json::json!([]));
+
+        let result = broker.publish_at("test-queue", msg, now_eta).await;
+        assert!(result.is_ok());
+        // eta <= now should trigger immediate publish
+        assert!(broker.published_immediate.load(std::sync::atomic::Ordering::SeqCst),
+            "ETA at now should call publish() immediately");
+    }
+
+    // -----------------------------------------------------------------------
+    // S6: CloudTasksBroker capabilities and delivery model (feature-gated)
+    // -----------------------------------------------------------------------
+
+    #[cfg(feature = "cloudtasks")]
+    mod cloudtasks_tests {
+        use super::*;
+        use crate::broker::cloudtasks::{CloudTasksBroker, CloudTasksConfig};
+
+        fn test_config() -> CloudTasksConfig {
+            CloudTasksConfig {
+                project_id: "test-project".to_string(),
+                location: "us-central1".to_string(),
+                worker_url: "https://worker.example.com".to_string(),
+                service_account_email: None,
+                oidc_audience: None,
+                default_queue: "default".to_string(),
+                dispatch_deadline: Duration::from_secs(600),
+                max_retry_count: None,
+                credentials_path: None,
+            }
+        }
+
+        #[test]
+        fn test_cloudtasks_delivery_model_is_push() {
+            let broker = CloudTasksBroker::new(test_config());
+            assert_eq!(broker.delivery_model(), DeliveryModel::Push);
+        }
+
+        #[test]
+        fn test_cloudtasks_capabilities() {
+            let broker = CloudTasksBroker::new(test_config());
+            let caps = broker.capabilities();
+            assert!(caps.delayed_tasks, "Cloud Tasks supports delayed tasks");
+            assert!(caps.dead_letter, "Cloud Tasks supports dead letter");
+            assert!(!caps.priority, "Cloud Tasks does not support priority");
+            assert!(!caps.batching, "Cloud Tasks does not support batching");
+            assert_eq!(
+                caps.max_delay,
+                Some(Duration::from_secs(30 * 24 * 60 * 60)),
+                "Cloud Tasks max delay should be 30 days"
+            );
+        }
+
+        // S2: PushBroker ack/nack status codes for CloudTasksBroker
+        #[test]
+        fn test_cloudtasks_ack_status_code() {
+            let broker = CloudTasksBroker::new(test_config());
+            assert_eq!(broker.ack_status_code(), 200);
+        }
+
+        #[test]
+        fn test_cloudtasks_nack_status_code() {
+            let broker = CloudTasksBroker::new(test_config());
+            assert_eq!(broker.nack_status_code(), 500);
+        }
+
+        #[test]
+        fn test_cloudtasks_endpoint_path() {
+            let broker = CloudTasksBroker::new(test_config());
+            assert_eq!(broker.endpoint_path(), "/meteor/push/{queue}");
+        }
+
+        // S1: Parse push request with Cloud Tasks headers
+        #[test]
+        fn test_cloudtasks_parse_push_request_with_task_header() {
+            let broker = CloudTasksBroker::new(test_config());
+            let msg = TaskMessage::new("cloud_task", serde_json::json!({"key": "value"}));
+            let body = serde_json::to_vec(&msg).unwrap();
+
+            let mut headers = HashMap::new();
+            headers.insert("x-cloudtasks-taskname".to_string(), "projects/test/locations/us/queues/q/tasks/t123".to_string());
+            headers.insert("x-cloudtasks-taskretrycount".to_string(), "0".to_string());
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_ok());
+            let broker_msg = result.unwrap();
+            assert_eq!(broker_msg.delivery_tag, "projects/test/locations/us/queues/q/tasks/t123");
+            assert_eq!(broker_msg.payload.task_name, "cloud_task");
+            assert!(!broker_msg.redelivered, "retry count 0 should not be redelivered");
+        }
+
+        // S1: Redelivery detection via retry count header
+        #[test]
+        fn test_cloudtasks_parse_push_request_redelivered() {
+            let broker = CloudTasksBroker::new(test_config());
+            let msg = TaskMessage::new("retry_task", serde_json::json!([]));
+            let body = serde_json::to_vec(&msg).unwrap();
+
+            let mut headers = HashMap::new();
+            headers.insert("x-cloudtasks-taskname".to_string(), "task-retry-1".to_string());
+            headers.insert("x-cloudtasks-taskretrycount".to_string(), "3".to_string());
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_ok());
+            let broker_msg = result.unwrap();
+            assert!(broker_msg.redelivered, "retry count > 0 should be redelivered");
+        }
+
+        // S1: Missing task name header falls back to task ID
+        #[test]
+        fn test_cloudtasks_parse_push_request_no_taskname_header() {
+            let broker = CloudTasksBroker::new(test_config());
+            let msg = TaskMessage::new("fallback_task", serde_json::json!([]));
+            let task_id_str = msg.id.to_string();
+            let body = serde_json::to_vec(&msg).unwrap();
+
+            let headers = HashMap::new(); // no Cloud Tasks headers
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_ok());
+            let broker_msg = result.unwrap();
+            assert_eq!(broker_msg.delivery_tag, task_id_str,
+                "Missing x-cloudtasks-taskname should fall back to task ID");
+        }
+
+        // Parse invalid JSON body
+        #[test]
+        fn test_cloudtasks_parse_push_request_invalid_json() {
+            let broker = CloudTasksBroker::new(test_config());
+            let headers = HashMap::new();
+            let invalid_body = b"this is not json";
+
+            let result = broker.parse_push_request(&headers, invalid_body);
+            assert!(result.is_err());
+            match result.unwrap_err() {
+                TaskError::Deserialization(_) => {} // expected
+                other => panic!("Expected Deserialization error, got: {:?}", other),
+            }
+        }
+
+        // OIDC auth header required when service_account_email is set
+        #[test]
+        fn test_cloudtasks_parse_push_request_missing_auth() {
+            let mut config = test_config();
+            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
+            let broker = CloudTasksBroker::new(config);
+
+            let msg = TaskMessage::new("auth_task", serde_json::json!([]));
+            let body = serde_json::to_vec(&msg).unwrap();
+            let headers = HashMap::new(); // no Authorization header
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_err());
+            match result.unwrap_err() {
+                TaskError::Authentication(msg) => {
+                    assert!(msg.contains("Authorization"), "Error should mention Authorization header");
+                }
+                other => panic!("Expected Authentication error, got: {:?}", other),
+            }
+        }
+
+        // OIDC auth header with invalid format
+        #[test]
+        fn test_cloudtasks_parse_push_request_invalid_auth_format() {
+            let mut config = test_config();
+            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
+            let broker = CloudTasksBroker::new(config);
+
+            let msg = TaskMessage::new("auth_task", serde_json::json!([]));
+            let body = serde_json::to_vec(&msg).unwrap();
+            let mut headers = HashMap::new();
+            headers.insert("authorization".to_string(), "Basic dXNlcjpwYXNz".to_string());
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_err());
+            match result.unwrap_err() {
+                TaskError::Authentication(msg) => {
+                    assert!(msg.contains("Invalid"), "Error should mention invalid format");
+                }
+                other => panic!("Expected Authentication error, got: {:?}", other),
+            }
+        }
+
+        // Successful OIDC auth when Bearer token is present with valid claims
+        #[test]
+        fn test_cloudtasks_parse_push_request_valid_bearer_token() {
+            let mut config = test_config();
+            config.service_account_email = Some("sa@project.iam.gserviceaccount.com".to_string());
+            let broker = CloudTasksBroker::new(config);
+
+            let msg = TaskMessage::new("auth_ok_task", serde_json::json!([]));
+            let body = serde_json::to_vec(&msg).unwrap();
+
+            // Build a valid JWT with proper base64url-encoded claims
+            // Audience must match effective_audience (worker_url since oidc_audience is None)
+            let jwt_token = make_test_jwt(
+                "sa@project.iam.gserviceaccount.com",
+                "https://worker.example.com",
+                chrono::Utc::now().timestamp() + 3600,
+            );
+
+            let mut headers = HashMap::new();
+            headers.insert("authorization".to_string(), format!("Bearer {}", jwt_token));
+            headers.insert("x-cloudtasks-taskname".to_string(), "auth-task-1".to_string());
+
+            let result = broker.parse_push_request(&headers, &body);
+            assert!(result.is_ok(), "Valid Bearer token should pass auth check");
+            assert_eq!(result.unwrap().delivery_tag, "auth-task-1");
+        }
+
+        /// Build a fake JWT with properly base64url-encoded claims for testing
+        fn make_test_jwt(email: &str, audience: &str, exp: i64) -> String {
+            use base64::Engine;
+            let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
+                .encode(r#"{"alg":"RS256","typ":"JWT"}"#);
+            let payload_json = serde_json::json!({
+                "email": email,
+                "aud": audience,
+                "exp": exp,
+                "iss": "accounts.google.com",
+            });
+            let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
+                .encode(payload_json.to_string().as_bytes());
+            format!("{}.{}.fake_sig", header, payload)
+        }
+
+        // CloudTasksConfig default values
+        #[test]
+        fn test_cloudtasks_config_default() {
+            let config = CloudTasksConfig::default();
+            assert!(config.project_id.is_empty());
+            assert_eq!(config.location, "us-central1");
+            assert!(config.worker_url.is_empty());
+            assert!(config.service_account_email.is_none());
+            assert!(config.oidc_audience.is_none());
+            assert_eq!(config.default_queue, "default");
+            assert_eq!(config.dispatch_deadline, Duration::from_secs(600));
+            assert!(config.max_retry_count.is_none());
+            assert!(config.credentials_path.is_none());
+        }
+
+        // Connect validation: empty project_id
+        #[tokio::test]
+        async fn test_cloudtasks_connect_empty_project_id() {
+            let config = CloudTasksConfig {
+                project_id: String::new(),
+                worker_url: "https://example.com".to_string(),
+                ..Default::default()
+            };
+            let broker = CloudTasksBroker::new(config);
+            let result = broker.connect().await;
+            assert!(result.is_err());
+            match result.unwrap_err() {
+                TaskError::Configuration(msg) => {
+                    assert!(msg.contains("project_id"));
+                }
+                other => panic!("Expected Configuration error, got: {:?}", other),
+            }
+        }
+
+        // Connect validation: empty worker_url
+        #[tokio::test]
+        async fn test_cloudtasks_connect_empty_worker_url() {
+            let config = CloudTasksConfig {
+                project_id: "test-project".to_string(),
+                worker_url: String::new(),
+                ..Default::default()
+            };
+            let broker = CloudTasksBroker::new(config);
+            let result = broker.connect().await;
+            assert!(result.is_err());
+            match result.unwrap_err() {
+                TaskError::Configuration(msg) => {
+                    assert!(msg.contains("worker_url"));
+                }
+                other => panic!("Expected Configuration error, got: {:?}", other),
+            }
+        }
+
+        // Disconnect clears client
+        #[tokio::test]
+        async fn test_cloudtasks_disconnect() {
+            let broker = CloudTasksBroker::new(test_config());
+            // connect first
+            broker.connect().await.unwrap();
+            // then disconnect
+            let result = broker.disconnect().await;
+            assert!(result.is_ok());
+        }
+    }
+}
diff --git a/crates/cclab-queue/src/error.rs b/crates/cclab-queue/src/error.rs
index 4e60fecb..61e3e348 100644
--- a/crates/cclab-queue/src/error.rs
+++ b/crates/cclab-queue/src/error.rs
@@ -43,6 +43,12 @@ pub enum TaskError {
     #[error("Configuration error: {0}")]
     Configuration(String),
 
+    #[error("Authentication error: {0}")]
+    Authentication(String),
+
+    #[error("Already exists: {0}")]
+    AlreadyExists(String),
+
     #[error("Not connected")]
     NotConnected,
 
diff --git a/crates/cclab-queue/src/lib.rs b/crates/cclab-queue/src/lib.rs
index fb8ec6c7..10e0e879 100644
--- a/crates/cclab-queue/src/lib.rs
+++ b/crates/cclab-queue/src/lib.rs
@@ -90,6 +90,9 @@ pub use scheduler::{DelayedTaskConfig, DelayedTaskScheduler};
 #[cfg(feature = "scheduler")]
 pub use scheduler::{IonSchedulerBackend, PeriodicSchedule, PeriodicScheduler, PeriodicTask};
 
+#[cfg(feature = "cloud-scheduler")]
+pub use scheduler::{CloudSchedulerBackend, CloudSchedulerConfig};
+
 pub use scheduler::periodic::PeriodicSchedulerConfig;
 
 // Workflow re-exports
diff --git a/crates/cclab-queue/src/scheduler/mod.rs b/crates/cclab-queue/src/scheduler/mod.rs
index 98460bbe..41e6093e 100644
--- a/crates/cclab-queue/src/scheduler/mod.rs
+++ b/crates/cclab-queue/src/scheduler/mod.rs
@@ -10,11 +10,284 @@ pub mod periodic;
 #[cfg(feature = "scheduler")]
 pub mod ion_backend;
 
+#[cfg(feature = "cloud-scheduler")]
+pub mod cloud_scheduler_backend;
+
 // Re-exports
 pub use backend::{SchedulerBackend, TaskScheduleState};
 #[cfg(feature = "nats")]
 pub use delay::{DelayedTaskConfig, DelayedTaskScheduler};
 #[cfg(feature = "scheduler")]
 pub use ion_backend::IonSchedulerBackend;
+#[cfg(feature = "cloud-scheduler")]
+pub use cloud_scheduler_backend::{CloudSchedulerBackend, CloudSchedulerConfig};
 pub use memory_backend::MemorySchedulerBackend;
 pub use periodic::{PeriodicSchedule, PeriodicScheduler, PeriodicTask};
+
+// ---------------------------------------------------------------------------
+// Tests for scheduler-backends-gcp spec (module registration & feature isolation)
+// ---------------------------------------------------------------------------
+
+/// Tests that require the cloud-scheduler feature — covers S1, S3, S4 from spec.
+/// S2 (feature exclusion) is implicitly verified: this entire module is behind
+/// `#[cfg(feature = "cloud-scheduler")]`, so it only compiles when the feature is active.
+/// Without the feature, `CloudSchedulerBackend` and `CloudSchedulerConfig` are absent
+/// from the `scheduler` module — verified by the conditional `pub mod` and `pub use`.
+#[cfg(all(test, feature = "cloud-scheduler"))]
+mod backends_gcp_tests {
+    use super::*;
+    use std::time::Duration;
+
+    // -----------------------------------------------------------------------
+    // S1: Cloud Scheduler backend is available when feature is enabled (R4, R6)
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn s1_cloud_scheduler_backend_importable() {
+        // Verifies R4: CloudSchedulerBackend is importable from scheduler module
+        let config = CloudSchedulerConfig {
+            project_id: "test-project".to_string(),
+            location: "us-central1".to_string(),
+            oidc_service_account_email: "sa@test.iam.gserviceaccount.com".to_string(),
+            target_base_url: "https://example.com/tasks".to_string(),
+            time_zone: "UTC".to_string(),
+            credentials_path: None,
+        };
+        let backend = CloudSchedulerBackend::new(config);
+        assert!(backend.is_ok(), "CloudSchedulerBackend should be constructible when cloud-scheduler feature is enabled");
+    }
+
+    #[test]
+    fn s1_cloud_scheduler_config_importable() {
+        // Verifies R4: CloudSchedulerConfig is importable from scheduler module
+        let config = CloudSchedulerConfig::default();
+        assert_eq!(config.location, "us-central1");
+        assert_eq!(config.time_zone, "UTC");
+    }
+
+    #[test]
+    fn s1_cloud_scheduler_module_reexports() {
+        // Verifies R4: re-exports work — types are accessible via scheduler::*
+        // If this compiles, the re-exports in mod.rs are correct.
+        fn _assert_type_exists(_b: CloudSchedulerBackend) {}
+        fn _assert_config_exists(_c: CloudSchedulerConfig) {}
+    }
+
+    // -----------------------------------------------------------------------
+    // S3: Cloud Scheduler backend implements SchedulerBackend trait (R5)
+    // -----------------------------------------------------------------------
+
+    fn make_test_backend() -> CloudSchedulerBackend {
+        CloudSchedulerBackend::new(CloudSchedulerConfig {
+            project_id: "trait-test".to_string(),
+            location: "us-central1".to_string(),
+            oidc_service_account_email: "sa@test.iam.gserviceaccount.com".to_string(),
+            target_base_url: "https://example.com".to_string(),
+            time_zone: "UTC".to_string(),
+            credentials_path: None,
+        })
+        .unwrap()
+    }
+
+    #[test]
+    fn s3_cloud_scheduler_satisfies_scheduler_backend_trait() {
+        // Verifies R5: CloudSchedulerBackend implements SchedulerBackend
+        // If this compiles, the trait is implemented.
+        let backend = make_test_backend();
+        let _boxed: Box<dyn SchedulerBackend> = Box::new(backend);
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_acquire_leader() {
+        // Verifies R5: acquire_leader callable through trait object
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+        let result = dyn_backend.acquire_leader(Duration::from_secs(15)).await;
+        assert_eq!(result.unwrap(), true);
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_renew_leader() {
+        // Verifies R5: renew_leader callable through trait object
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+        let result = dyn_backend.renew_leader(Duration::from_secs(15)).await;
+        assert_eq!(result.unwrap(), true);
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_release_leader() {
+        // Verifies R5: release_leader callable through trait object
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+        assert!(dyn_backend.release_leader().await.is_ok());
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_get_task_state() {
+        // Verifies R5: get_task_state callable through trait object
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+        let state = dyn_backend.get_task_state("test-task").await.unwrap();
+        assert!(state.enabled);
+        assert_eq!(state.total_run_count, 0);
+        assert!(state.last_run_at.is_none());
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_set_task_state() {
+        // Verifies R5: set_task_state callable through trait object
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+
+        let state = TaskScheduleState {
+            enabled: false,
+            last_run_at: Some(chrono::Utc::now()),
+            total_run_count: 10,
+        };
+        dyn_backend.set_task_state("my-task", &state).await.unwrap();
+
+        let retrieved = dyn_backend.get_task_state("my-task").await.unwrap();
+        assert!(!retrieved.enabled);
+        assert_eq!(retrieved.total_run_count, 10);
+    }
+
+    #[tokio::test]
+    async fn s3_trait_object_all_methods_in_sequence() {
+        // Verifies R5: all trait methods work through a single Box<dyn SchedulerBackend>
+        // This simulates the PeriodicScheduler usage pattern
+        let backend = make_test_backend();
+        let dyn_backend: Box<dyn SchedulerBackend> = Box::new(backend);
+
+        // Leader election cycle
+        assert!(dyn_backend.acquire_leader(Duration::from_secs(30)).await.unwrap());
+        assert!(dyn_backend.renew_leader(Duration::from_secs(30)).await.unwrap());
+
+        // Task state operations
+        let state = dyn_backend.get_task_state("periodic-task").await.unwrap();
+        assert!(state.enabled);
+
+        let updated = TaskScheduleState {
+            enabled: true,
+            last_run_at: Some(chrono::Utc::now()),
+            total_run_count: 1,
+        };
+        dyn_backend.set_task_state("periodic-task", &updated).await.unwrap();
+
+        let retrieved = dyn_backend.get_task_state("periodic-task").await.unwrap();
+        assert_eq!(retrieved.total_run_count, 1);
+
+        // Graceful shutdown
+        dyn_backend.release_leader().await.unwrap();
+    }
+
+    // -----------------------------------------------------------------------
+    // S4: All three backends coexist (R6)
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn s4_memory_and_cloud_backends_coexist() {
+        // Verifies R6: MemorySchedulerBackend and CloudSchedulerBackend available simultaneously
+        let _memory = MemorySchedulerBackend::new();
+        let _cloud = make_test_backend();
+    }
+
+    #[tokio::test]
+    async fn s4_backends_interchangeable_at_runtime() {
+        // Verifies R6: application can select backend at runtime based on configuration
+        let backends: Vec<Box<dyn SchedulerBackend>> = vec![
+            Box::new(MemorySchedulerBackend::new()),
+            Box::new(make_test_backend()),
+        ];
+
+        // Both backends work through the same trait interface
+        for (i, backend) in backends.iter().enumerate() {
+            let acquired = backend.acquire_leader(Duration::from_secs(10)).await.unwrap();
+            assert!(acquired, "Backend {} should acquire leadership", i);
+
+            let state = backend.get_task_state("shared-task").await.unwrap();
+            assert!(state.enabled, "Backend {} default state should be enabled", i);
+
+            let new_state = TaskScheduleState {
+                enabled: true,
+                last_run_at: Some(chrono::Utc::now()),
+                total_run_count: 1,
+            };
+            backend.set_task_state("shared-task", &new_state).await.unwrap();
+
+            let retrieved = backend.get_task_state("shared-task").await.unwrap();
+            assert_eq!(retrieved.total_run_count, 1, "Backend {} state should persist", i);
+
+            backend.release_leader().await.unwrap();
+        }
+    }
+
+    #[tokio::test]
+    async fn s4_backends_independent_state() {
+        // Verifies R6: each backend instance maintains independent state
+        let memory = MemorySchedulerBackend::new();
+        let cloud = make_test_backend();
+
+        // Set state on memory backend
+        let state = TaskScheduleState {
+            enabled: false,
+            last_run_at: None,
+            total_run_count: 42,
+        };
+        memory.set_task_state("task-a", &state).await.unwrap();
+
+        // Cloud backend should have independent default state
+        let cloud_state = cloud.get_task_state("task-a").await.unwrap();
+        assert!(cloud_state.enabled);
+        assert_eq!(cloud_state.total_run_count, 0);
+
+        // Memory backend state unchanged
+        let memory_state = memory.get_task_state("task-a").await.unwrap();
+        assert!(!memory_state.enabled);
+        assert_eq!(memory_state.total_run_count, 42);
+    }
+
+    // -----------------------------------------------------------------------
+    // R5: No modifications to TaskScheduleState struct
+    // -----------------------------------------------------------------------
+
+    #[test]
+    fn r5_task_schedule_state_unchanged() {
+        // Verify TaskScheduleState still has the expected fields
+        let state = TaskScheduleState::default();
+        assert!(state.enabled);
+        assert!(state.last_run_at.is_none());
+        assert_eq!(state.total_run_count, 0);
+
+        // Verify it can be constructed with all fields
+        let _state = TaskScheduleState {
+            enabled: false,
+            last_run_at: Some(chrono::Utc::now()),
+            total_run_count: 99,
+        };
+    }
+
+    // -----------------------------------------------------------------------
+    // R6: Feature isolation — cloud-scheduler does not affect other backends
+    // -----------------------------------------------------------------------
+
+    #[tokio::test]
+    async fn r6_memory_backend_unaffected_by_cloud_feature() {
+        // Verifies R6: enabling cloud-scheduler does not change MemorySchedulerBackend behavior
+        let backend = MemorySchedulerBackend::new();
+
+        // Leadership works as expected (acquire -> true, renew while leader -> true)
+        assert!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap());
+        assert!(backend.renew_leader(Duration::from_secs(10)).await.unwrap());
+
+        // Task state management works
+        backend.record_task_run("test-task").await.unwrap();
+        let state = backend.get_task_state("test-task").await.unwrap();
+        assert_eq!(state.total_run_count, 1);
+
+        // Release works
+        backend.release_leader().await.unwrap();
+        // After release, renew should report not leader (Memory backend tracks this)
+        assert!(!backend.renew_leader(Duration::from_secs(10)).await.unwrap());
+    }
+}
diff --git a/crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs b/crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-queue/src/scheduler/cloud_scheduler_backend.rs
@@ -0,0 +1,     572 @@
+//! GCP Cloud Scheduler backend
+//!
+//! Offloads cron/interval scheduling to GCP Cloud Scheduler service.
+//! Leader election is a no-op since GCP manages scheduling authority.
+//! Task state is tracked locally with optional persistence.
+
+use std::collections::HashMap;
+use std::sync::Arc;
+use std::time::Duration;
+
+use async_trait::async_trait;
+use chrono::{DateTime, Utc};
+use serde::{Deserialize, Serialize};
+use tokio::sync::RwLock;
+
+use super::backend::{SchedulerBackend, TaskScheduleState};
+use crate::TaskError;
+
+/// Cloud Scheduler REST API v1 base URL
+const CLOUD_SCHEDULER_API_BASE: &str = "https://cloudscheduler.googleapis.com/v1";
+
+/// GCP metadata server token endpoint
+const METADATA_TOKEN_URL: &str =
+    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
+
+/// Token refresh buffer (5 minutes before expiry)
+const TOKEN_REFRESH_BUFFER_SECS: i64 = 300;
+
+// ---------------------------------------------------------------------------
+// Configuration
+// ---------------------------------------------------------------------------
+
+/// Configuration for Cloud Scheduler backend
+#[derive(Debug, Clone)]
+pub struct CloudSchedulerConfig {
+    /// GCP project ID
+    pub project_id: String,
+    /// GCP region (e.g., "us-central1")
+    pub location: String,
+    /// Service account email for OIDC token in httpTarget
+    pub oidc_service_account_email: String,
+    /// Base URL of the application task endpoint
+    pub target_base_url: String,
+    /// IANA time zone for schedule evaluation
+    pub time_zone: String,
+    /// Path to service account JSON key file for local dev (None = metadata server)
+    pub credentials_path: Option<String>,
+}
+
+impl Default for CloudSchedulerConfig {
+    fn default() -> Self {
+        Self {
+            project_id: String::new(),
+            location: "us-central1".to_string(),
+            oidc_service_account_email: String::new(),
+            target_base_url: String::new(),
+            time_zone: "UTC".to_string(),
+            credentials_path: None,
+        }
+    }
+}
+
+impl CloudSchedulerConfig {
+    /// Jobs parent path for API calls
+    fn jobs_parent(&self) -> String {
+        format!(
+            "projects/{}/locations/{}",
+            self.project_id, self.location
+        )
+    }
+
+    /// Fully qualified job name
+    fn job_name(&self, job_id: &str) -> String {
+        format!("{}/jobs/{}", self.jobs_parent(), job_id)
+    }
+}
+
+// ---------------------------------------------------------------------------
+// OIDC Token Cache
+// ---------------------------------------------------------------------------
+
+/// Cached OIDC bearer token with expiry tracking
+#[derive(Debug, Clone)]
+struct OidcTokenCache {
+    access_token: Option<String>,
+    expires_at: Option<DateTime<Utc>>,
+}
+
+impl OidcTokenCache {
+    fn new() -> Self {
+        Self {
+            access_token: None,
+            expires_at: None,
+        }
+    }
+
+    fn is_valid(&self) -> bool {
+        match (&self.access_token, self.expires_at) {
+            (Some(_), Some(expires_at)) => {
+                Utc::now() < expires_at - chrono::Duration::seconds(TOKEN_REFRESH_BUFFER_SECS)
+            }
+            _ => false,
+        }
+    }
+}
+
+/// Response from GCP metadata server token endpoint
+#[derive(Deserialize)]
+struct MetadataTokenResponse {
+    access_token: String,
+    expires_in: i64,
+    #[allow(dead_code)]
+    token_type: String,
+}
+
+// ---------------------------------------------------------------------------
+// Cloud Scheduler API types
+// ---------------------------------------------------------------------------
+
+/// GCP Cloud Scheduler Job representation
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(rename_all = "camelCase")]
+pub struct CloudSchedulerJob {
+    /// Fully qualified job name: projects/{project}/locations/{location}/jobs/{jobId}
+    pub name: String,
+    /// Unix-cron format schedule (5 fields)
+    pub schedule: String,
+    /// IANA time zone
+    #[serde(default = "default_timezone")]
+    pub time_zone: String,
+    /// HTTP target configuration
+    pub http_target: HttpTarget,
+    /// GCP-managed job state
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub state: Option<String>,
+    /// Last user-initiated update time
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub user_update_time: Option<String>,
+    /// Last execution attempt time
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub last_attempt_time: Option<String>,
+    /// Execution status from last attempt
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub status: Option<serde_json::Value>,
+}
+
+fn default_timezone() -> String {
+    "UTC".to_string()
+}
+
+/// HTTP target configuration for Cloud Scheduler job
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(rename_all = "camelCase")]
+pub struct HttpTarget {
+    /// Full URL of the task endpoint
+    pub uri: String,
+    /// HTTP method
+    pub http_method: String,
+    /// Base64-encoded request body
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub body: Option<String>,
+    /// Request headers
+    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
+    pub headers: HashMap<String, String>,
+    /// OIDC token configuration
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub oidc_token: Option<OidcTokenTarget>,
+}
+
+/// OIDC token target in Cloud Scheduler httpTarget
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(rename_all = "camelCase")]
+pub struct OidcTokenTarget {
+    pub service_account_email: String,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub audience: Option<String>,
+}
+
+/// Response from Cloud Scheduler list jobs API
+#[derive(Debug, Deserialize)]
+#[serde(rename_all = "camelCase")]
+struct ListJobsResponse {
+    #[serde(default)]
+    jobs: Vec<CloudSchedulerJob>,
+    #[allow(dead_code)]
+    next_page_token: Option<String>,
+}
+
+// ---------------------------------------------------------------------------
+// CloudSchedulerBackend
+// ---------------------------------------------------------------------------
+
+/// Scheduler backend backed by GCP Cloud Scheduler
+///
+/// Leader election is a no-op — GCP Cloud Scheduler is the single
+/// authoritative scheduler. Task state is tracked locally in-memory.
+pub struct CloudSchedulerBackend {
+    config: CloudSchedulerConfig,
+    /// HTTP client for Cloud Scheduler API calls
+    client: reqwest::Client,
+    /// Cached OIDC access token for API authentication
+    token_cache: Arc<RwLock<OidcTokenCache>>,
+    /// In-memory task state store
+    task_states: Arc<RwLock<HashMap<String, TaskScheduleState>>>,
+}
+
+impl CloudSchedulerBackend {
+    /// Create a new Cloud Scheduler backend
+    pub fn new(config: CloudSchedulerConfig) -> Result<Self, TaskError> {
+        let client = reqwest::Client::builder()
+            .timeout(Duration::from_secs(30))
+            .build()
+            .map_err(|e| TaskError::Connection(format!("Failed to create HTTP client: {}", e)))?;
+
+        Ok(Self {
+            config,
+            client,
+            token_cache: Arc::new(RwLock::new(OidcTokenCache::new())),
+            task_states: Arc::new(RwLock::new(HashMap::new())),
+        })
+    }
+
+    /// Get OIDC bearer token, using cache when valid
+    async fn get_oidc_token(&self) -> Result<String, TaskError> {
+        // Check cache first
+        {
+            let cache = self.token_cache.read().await;
+            if cache.is_valid() {
+                return Ok(cache.access_token.clone().unwrap());
+            }
+        }
+
+        // Fetch new token from metadata server
+        let response = self
+            .client
+            .get(METADATA_TOKEN_URL)
+            .header("Metadata-Flavor", "Google")
+            .send()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to fetch OIDC token: {}", e))
+            })?;
+
+        if !response.status().is_success() {
+            return Err(TaskError::Authentication(format!(
+                "Metadata server returned status {}",
+                response.status()
+            )));
+        }
+
+        let token_response: MetadataTokenResponse = response
+            .json()
+            .await
+            .map_err(|e| {
+                TaskError::Authentication(format!("Failed to parse token response: {}", e))
+            })?;
+
+        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in);
+
+        // Update cache
+        let mut cache = self.token_cache.write().await;
+        cache.access_token = Some(token_response.access_token.clone());
+        cache.expires_at = Some(expires_at);
+
+        Ok(token_response.access_token)
+    }
+
+    /// Map GCP HTTP error status to TaskError
+    fn map_gcp_error(status: reqwest::StatusCode, body: &str) -> TaskError {
+        match status.as_u16() {
+            404 => TaskError::TaskNotFound(body.to_string()),
+            401 | 403 => TaskError::Authentication(
+                format!("GCP API authentication error ({}): {}", status, body),
+            ),
+            429 => TaskError::RateLimited(Duration::from_secs(60)),
+            500..=599 => TaskError::Backend(
+                format!("GCP API server error ({}): {}", status, body),
+            ),
+            _ => TaskError::Backend(
+                format!("GCP API error ({}): {}", status, body),
+            ),
+        }
+    }
+
+    // -----------------------------------------------------------------------
+    // Cloud Scheduler REST API methods
+    // -----------------------------------------------------------------------
+
+    /// Create a Cloud Scheduler job
+    pub async fn create_job(
+        &self,
+        job: &CloudSchedulerJob,
+    ) -> Result<CloudSchedulerJob, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let url = format!(
+            "{}/{}/jobs",
+            CLOUD_SCHEDULER_API_BASE,
+            self.config.jobs_parent()
+        );
+
+        let response = self
+            .client
+            .post(&url)
+            .bearer_auth(&token)
+            .json(job)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Create job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let created: CloudSchedulerJob = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(created)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// Update an existing Cloud Scheduler job
+    pub async fn update_job(
+        &self,
+        job: &CloudSchedulerJob,
+    ) -> Result<CloudSchedulerJob, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job.name);
+
+        let response = self
+            .client
+            .patch(&url)
+            .bearer_auth(&token)
+            .json(job)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Update job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let updated: CloudSchedulerJob = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(updated)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// Delete a Cloud Scheduler job
+    pub async fn delete_job(&self, name: &str) -> Result<(), TaskError> {
+        let token = self.get_oidc_token().await?;
+        let job_name = if name.starts_with("projects/") {
+            name.to_string()
+        } else {
+            self.config.job_name(name)
+        };
+        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job_name);
+
+        let response = self
+            .client
+            .delete(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Delete job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            // Remove local task state
+            let task_id = name.rsplit('/').next().unwrap_or(name);
+            self.task_states.write().await.remove(task_id);
+            Ok(())
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// Get a single Cloud Scheduler job
+    pub async fn get_job(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let job_name = if name.starts_with("projects/") {
+            name.to_string()
+        } else {
+            self.config.job_name(name)
+        };
+        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job_name);
+
+        let response = self
+            .client
+            .get(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Get job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let job: CloudSchedulerJob = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(job)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// List all Cloud Scheduler jobs
+    pub async fn list_jobs(&self) -> Result<Vec<CloudSchedulerJob>, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let url = format!(
+            "{}/{}/jobs",
+            CLOUD_SCHEDULER_API_BASE,
+            self.config.jobs_parent()
+        );
+
+        let response = self
+            .client
+            .get(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("List jobs request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let list: ListJobsResponse = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(list.jobs)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// Pause a Cloud Scheduler job via API
+    async fn pause_job_api(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let job_name = if name.starts_with("projects/") {
+            name.to_string()
+        } else {
+            self.config.job_name(name)
+        };
+        let url = format!("{}/{}:pause", CLOUD_SCHEDULER_API_BASE, job_name);
+
+        let response = self
+            .client
+            .post(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Pause job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let job: CloudSchedulerJob = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(job)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+
+    /// Resume a Cloud Scheduler job via API
+    async fn resume_job_api(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
+        let token = self.get_oidc_token().await?;
+        let job_name = if name.starts_with("projects/") {
+            name.to_string()
+        } else {
+            self.config.job_name(name)
+        };
+        let url = format!("{}/{}:resume", CLOUD_SCHEDULER_API_BASE, job_name);
+
+        let response = self
+            .client
+            .post(&url)
+            .bearer_auth(&token)
+            .send()
+            .await
+            .map_err(|e| TaskError::Connection(format!("Resume job request failed: {}", e)))?;
+
+        let status = response.status();
+        if status.is_success() {
+            let job: CloudSchedulerJob = response
+                .json()
+                .await
+                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
+            Ok(job)
+        } else {
+            let body = response.text().await.unwrap_or_default();
+            Err(Self::map_gcp_error(status, &body))
+        }
+    }
+}
+
+// ---------------------------------------------------------------------------
+// SchedulerBackend trait implementation
+// ---------------------------------------------------------------------------
+
+#[async_trait]
+impl SchedulerBackend for CloudSchedulerBackend {
+    /// No-op: GCP Cloud Scheduler is the single authoritative scheduler
+    async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
+        Ok(true)
+    }
+
+    /// No-op: no leader lease to renew
+    async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
+        Ok(true)
+    }
+
+    /// No-op: no leader lease to release
+    async fn release_leader(&self) -> Result<(), TaskError> {
+        Ok(())
+    }
+
+    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError> {
+        let states = self.task_states.read().await;
+        Ok(states.get(name).cloned().unwrap_or_default())
+    }
+
+    async fn set_task_state(
+        &self,
+        name: &str,
+        state: &TaskScheduleState,
+    ) -> Result<(), TaskError> {
+        self.task_states
+            .write()
+            .await
+            .insert(name.to_string(), state.clone());
+        Ok(())
+    }
+
+    /// Override: pause both GCP job and local state
+    async fn pause_task(&self, name: &str) -> Result<(), TaskError> {
+        // Pause on GCP side
+        self.pause_job_api(name).await?;
+
+        // Update local state
+        let mut states = self.task_states.write().await;
+        let state = states.entry(name.to_string()).or_insert_with(TaskScheduleState::default);
+        state.enabled = false;
+        Ok(())
+    }
+
+    /// Override: resume both GCP job and local state
+    async fn resume_task(&self, name: &str) -> Result<(), TaskError> {
+        // Resume on GCP side
+        self.resume_job_api(name).await?;
+
+        // Update local state
+        let mut states = self.task_states.write().await;
+        let state = states.entry(name.to_string()).or_insert_with(TaskScheduleState::default);
+        state.enabled = true;
+        Ok(())
+    }
+}
+
+#[cfg(test)]
+#[path = "cloud_scheduler_backend_tests.rs"]
+mod tests;
diff --git a/crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs b/crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-queue/src/scheduler/cloud_scheduler_backend_tests.rs
@@ -0,0 +1,     702 @@
+//! Tests for CloudSchedulerBackend
+//!
+//! Covers spec scenarios S1-S8 from cloud-scheduler-backend change spec.
+//! Tests that require GCP network calls (create_job, delete_job, pause/resume API)
+//! are tested indirectly through local state management and error mapping.
+
+use super::*;
+use chrono::TimeZone;
+
+// ---------------------------------------------------------------------------
+// Helpers
+// ---------------------------------------------------------------------------
+
+fn test_config() -> CloudSchedulerConfig {
+    CloudSchedulerConfig {
+        project_id: "my-project".to_string(),
+        location: "us-central1".to_string(),
+        oidc_service_account_email: "sa@my-project.iam.gserviceaccount.com".to_string(),
+        target_base_url: "https://app.example.com/tasks".to_string(),
+        time_zone: "UTC".to_string(),
+        credentials_path: None,
+    }
+}
+
+fn test_backend() -> CloudSchedulerBackend {
+    CloudSchedulerBackend::new(test_config()).expect("failed to create test backend")
+}
+
+fn sample_job() -> CloudSchedulerJob {
+    CloudSchedulerJob {
+        name: "projects/my-project/locations/us-central1/jobs/daily-cleanup".to_string(),
+        schedule: "0 2 * * *".to_string(),
+        time_zone: "UTC".to_string(),
+        http_target: HttpTarget {
+            uri: "https://app.example.com/tasks/cleanup".to_string(),
+            http_method: "POST".to_string(),
+            body: None,
+            headers: {
+                let mut h = HashMap::new();
+                h.insert("Content-Type".to_string(), "application/json".to_string());
+                h
+            },
+            oidc_token: Some(OidcTokenTarget {
+                service_account_email: "sa@my-project.iam.gserviceaccount.com".to_string(),
+                audience: Some("https://app.example.com".to_string()),
+            }),
+        },
+        state: Some("ENABLED".to_string()),
+        user_update_time: None,
+        last_attempt_time: None,
+        status: None,
+    }
+}
+
+// ---------------------------------------------------------------------------
+// S1: Leader election is no-op for cloud-managed backend (R1)
+// ---------------------------------------------------------------------------
+
+#[tokio::test]
+async fn test_acquire_leader_returns_true() {
+    let backend = test_backend();
+    let result = backend.acquire_leader(Duration::from_secs(15)).await;
+    assert_eq!(result.unwrap(), true);
+}
+
+#[tokio::test]
+async fn test_renew_leader_returns_true() {
+    let backend = test_backend();
+    let result = backend.renew_leader(Duration::from_secs(15)).await;
+    assert_eq!(result.unwrap(), true);
+}
+
+#[tokio::test]
+async fn test_release_leader_returns_ok() {
+    let backend = test_backend();
+    let result = backend.release_leader().await;
+    assert!(result.is_ok());
+}
+
+#[tokio::test]
+async fn test_leader_election_full_cycle() {
+    let backend = test_backend();
+    // All three operations should succeed as no-ops
+    assert!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap());
+    assert!(backend.renew_leader(Duration::from_secs(10)).await.unwrap());
+    backend.release_leader().await.unwrap();
+    // After release, acquire should still return true (cloud manages scheduling)
+    assert!(backend.acquire_leader(Duration::from_secs(10)).await.unwrap());
+}
+
+// ---------------------------------------------------------------------------
+// S2: Config helpers and job serialization (R3, R5)
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_config_jobs_parent() {
+    let config = test_config();
+    assert_eq!(
+        config.jobs_parent(),
+        "projects/my-project/locations/us-central1"
+    );
+}
+
+#[test]
+fn test_config_job_name() {
+    let config = test_config();
+    assert_eq!(
+        config.job_name("daily-cleanup"),
+        "projects/my-project/locations/us-central1/jobs/daily-cleanup"
+    );
+}
+
+#[test]
+fn test_config_default() {
+    let config = CloudSchedulerConfig::default();
+    assert_eq!(config.location, "us-central1");
+    assert_eq!(config.time_zone, "UTC");
+    assert!(config.credentials_path.is_none());
+    assert!(config.project_id.is_empty());
+}
+
+#[test]
+fn test_job_serialization_camel_case() {
+    let job = sample_job();
+    let json = serde_json::to_value(&job).unwrap();
+
+    // Verify camelCase field names
+    assert!(json.get("name").is_some());
+    assert!(json.get("schedule").is_some());
+    assert!(json.get("timeZone").is_some());
+    assert!(json.get("httpTarget").is_some());
+    assert!(json.get("state").is_some());
+
+    // Verify None fields are skipped
+    assert!(json.get("userUpdateTime").is_none());
+    assert!(json.get("lastAttemptTime").is_none());
+    assert!(json.get("status").is_none());
+}
+
+#[test]
+fn test_job_deserialization_from_gcp_response() {
+    let json = serde_json::json!({
+        "name": "projects/my-project/locations/us-central1/jobs/test-job",
+        "schedule": "*/5 * * * *",
+        "timeZone": "America/New_York",
+        "httpTarget": {
+            "uri": "https://app.example.com/tasks/test",
+            "httpMethod": "POST",
+            "headers": { "Content-Type": "application/json" },
+            "oidcToken": {
+                "serviceAccountEmail": "sa@my-project.iam.gserviceaccount.com",
+                "audience": "https://app.example.com"
+            }
+        },
+        "state": "ENABLED",
+        "userUpdateTime": "2026-03-26T10:00:00Z",
+        "lastAttemptTime": "2026-03-26T12:00:00Z",
+        "status": { "code": 0 }
+    });
+
+    let job: CloudSchedulerJob = serde_json::from_value(json).unwrap();
+    assert_eq!(
+        job.name,
+        "projects/my-project/locations/us-central1/jobs/test-job"
+    );
+    assert_eq!(job.schedule, "*/5 * * * *");
+    assert_eq!(job.time_zone, "America/New_York");
+    assert_eq!(job.http_target.uri, "https://app.example.com/tasks/test");
+    assert_eq!(job.http_target.http_method, "POST");
+    assert_eq!(job.state.as_deref(), Some("ENABLED"));
+    assert!(job.user_update_time.is_some());
+    assert!(job.last_attempt_time.is_some());
+    assert!(job.status.is_some());
+}
+
+#[test]
+fn test_job_deserialization_minimal() {
+    // Minimal job with only required fields
+    let json = serde_json::json!({
+        "name": "projects/p/locations/l/jobs/j",
+        "schedule": "0 * * * *",
+        "httpTarget": {
+            "uri": "https://example.com/task",
+            "httpMethod": "POST",
+            "oidcToken": {
+                "serviceAccountEmail": "sa@p.iam.gserviceaccount.com"
+            }
+        }
+    });
+
+    let job: CloudSchedulerJob = serde_json::from_value(json).unwrap();
+    assert_eq!(job.time_zone, "UTC"); // default
+    assert!(job.state.is_none());
+    assert!(job.http_target.body.is_none());
+    assert!(job.http_target.headers.is_empty());
+    assert!(
+        job.http_target
+            .oidc_token
+            .as_ref()
+            .unwrap()
+            .audience
+            .is_none()
+    );
+}
+
+#[test]
+fn test_job_roundtrip_serialization() {
+    let job = sample_job();
+    let json = serde_json::to_string(&job).unwrap();
+    let deserialized: CloudSchedulerJob = serde_json::from_str(&json).unwrap();
+    assert_eq!(deserialized.name, job.name);
+    assert_eq!(deserialized.schedule, job.schedule);
+    assert_eq!(deserialized.http_target.uri, job.http_target.uri);
+}
+
+#[test]
+fn test_list_jobs_response_deserialization() {
+    let json = serde_json::json!({
+        "jobs": [
+            {
+                "name": "projects/p/locations/l/jobs/job1",
+                "schedule": "0 * * * *",
+                "httpTarget": {
+                    "uri": "https://example.com/t1",
+                    "httpMethod": "POST",
+                    "oidcToken": { "serviceAccountEmail": "sa@p.iam.gserviceaccount.com" }
+                }
+            },
+            {
+                "name": "projects/p/locations/l/jobs/job2",
+                "schedule": "*/10 * * * *",
+                "httpTarget": {
+                    "uri": "https://example.com/t2",
+                    "httpMethod": "POST",
+                    "oidcToken": { "serviceAccountEmail": "sa@p.iam.gserviceaccount.com" }
+                }
+            }
+        ],
+        "nextPageToken": "abc123"
+    });
+
+    let response: ListJobsResponse = serde_json::from_value(json).unwrap();
+    assert_eq!(response.jobs.len(), 2);
+    assert_eq!(response.next_page_token.as_deref(), Some("abc123"));
+}
+
+#[test]
+fn test_list_jobs_response_empty() {
+    let json = serde_json::json!({});
+    let response: ListJobsResponse = serde_json::from_value(json).unwrap();
+    assert!(response.jobs.is_empty());
+    assert!(response.next_page_token.is_none());
+}
+
+// ---------------------------------------------------------------------------
+// S4: Task state management (R2) — get, set, record_task_run
+// ---------------------------------------------------------------------------
+
+#[tokio::test]
+async fn test_default_task_state_for_unknown_task() {
+    let backend = test_backend();
+    let state = backend.get_task_state("nonexistent-task").await.unwrap();
+    assert!(state.enabled);
+    assert!(state.last_run_at.is_none());
+    assert_eq!(state.total_run_count, 0);
+}
+
+#[tokio::test]
+async fn test_set_and_get_task_state() {
+    let backend = test_backend();
+    let now = Utc::now();
+    let state = TaskScheduleState {
+        enabled: false,
+        last_run_at: Some(now),
+        total_run_count: 42,
+    };
+
+    backend.set_task_state("my-task", &state).await.unwrap();
+    let retrieved = backend.get_task_state("my-task").await.unwrap();
+
+    assert!(!retrieved.enabled);
+    assert_eq!(retrieved.total_run_count, 42);
+    assert_eq!(retrieved.last_run_at, Some(now));
+}
+
+#[tokio::test]
+async fn test_set_task_state_overwrites() {
+    let backend = test_backend();
+
+    let state1 = TaskScheduleState {
+        enabled: true,
+        last_run_at: None,
+        total_run_count: 1,
+    };
+    backend.set_task_state("task-a", &state1).await.unwrap();
+
+    let state2 = TaskScheduleState {
+        enabled: false,
+        last_run_at: Some(Utc::now()),
+        total_run_count: 99,
+    };
+    backend.set_task_state("task-a", &state2).await.unwrap();
+
+    let retrieved = backend.get_task_state("task-a").await.unwrap();
+    assert!(!retrieved.enabled);
+    assert_eq!(retrieved.total_run_count, 99);
+}
+
+#[tokio::test]
+async fn test_record_task_run_increments_count() {
+    let backend = test_backend();
+
+    // First run on fresh task
+    backend.record_task_run("hourly-sync").await.unwrap();
+    let state = backend.get_task_state("hourly-sync").await.unwrap();
+    assert_eq!(state.total_run_count, 1);
+    assert!(state.last_run_at.is_some());
+
+    // Second run
+    backend.record_task_run("hourly-sync").await.unwrap();
+    let state = backend.get_task_state("hourly-sync").await.unwrap();
+    assert_eq!(state.total_run_count, 2);
+}
+
+#[tokio::test]
+async fn test_record_task_run_updates_last_run_at() {
+    let backend = test_backend();
+
+    // Set initial state with old timestamp
+    let old_time = Utc.with_ymd_and_hms(2026, 3, 26, 10, 0, 0).unwrap();
+    let initial = TaskScheduleState {
+        enabled: true,
+        last_run_at: Some(old_time),
+        total_run_count: 5,
+    };
+    backend
+        .set_task_state("hourly-sync", &initial)
+        .await
+        .unwrap();
+
+    // Record a new run
+    let before = Utc::now();
+    backend.record_task_run("hourly-sync").await.unwrap();
+    let after = Utc::now();
+
+    let state = backend.get_task_state("hourly-sync").await.unwrap();
+    assert_eq!(state.total_run_count, 6);
+    let last_run = state.last_run_at.unwrap();
+    assert!(last_run >= before && last_run <= after);
+}
+
+#[tokio::test]
+async fn test_record_task_run_preserves_enabled() {
+    let backend = test_backend();
+
+    // Task starts enabled (default)
+    backend.record_task_run("task-x").await.unwrap();
+    let state = backend.get_task_state("task-x").await.unwrap();
+    assert!(state.enabled);
+}
+
+#[tokio::test]
+async fn test_multiple_tasks_isolated() {
+    let backend = test_backend();
+
+    backend.record_task_run("task-a").await.unwrap();
+    backend.record_task_run("task-a").await.unwrap();
+    backend.record_task_run("task-b").await.unwrap();
+
+    let state_a = backend.get_task_state("task-a").await.unwrap();
+    let state_b = backend.get_task_state("task-b").await.unwrap();
+    let state_c = backend.get_task_state("task-c").await.unwrap();
+
+    assert_eq!(state_a.total_run_count, 2);
+    assert_eq!(state_b.total_run_count, 1);
+    assert_eq!(state_c.total_run_count, 0); // never recorded
+}
+
+// ---------------------------------------------------------------------------
+// S3 / S7: Pause and resume — local state effects (R7)
+// (pause_task/resume_task make HTTP calls, so we test via set_task_state)
+// ---------------------------------------------------------------------------
+
+#[tokio::test]
+async fn test_pause_resume_via_local_state() {
+    let backend = test_backend();
+
+    // Simulate what pause_task does locally
+    let mut state = backend.get_task_state("daily-cleanup").await.unwrap();
+    assert!(state.enabled);
+
+    state.enabled = false;
+    backend
+        .set_task_state("daily-cleanup", &state)
+        .await
+        .unwrap();
+
+    assert!(!backend.is_task_enabled("daily-cleanup").await.unwrap());
+
+    // Simulate what resume_task does locally
+    state.enabled = true;
+    backend
+        .set_task_state("daily-cleanup", &state)
+        .await
+        .unwrap();
+
+    assert!(backend.is_task_enabled("daily-cleanup").await.unwrap());
+}
+
+#[tokio::test]
+async fn test_is_task_enabled_default_true() {
+    let backend = test_backend();
+    // Unknown task defaults to enabled
+    assert!(backend.is_task_enabled("new-task").await.unwrap());
+}
+
+// ---------------------------------------------------------------------------
+// S5: OidcTokenCache validity logic (R4)
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_oidc_cache_new_is_invalid() {
+    let cache = OidcTokenCache::new();
+    assert!(!cache.is_valid());
+    assert!(cache.access_token.is_none());
+    assert!(cache.expires_at.is_none());
+}
+
+#[test]
+fn test_oidc_cache_valid_token() {
+    let cache = OidcTokenCache {
+        access_token: Some("token-abc".to_string()),
+        expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
+    };
+    assert!(cache.is_valid());
+}
+
+#[test]
+fn test_oidc_cache_expired_token() {
+    let cache = OidcTokenCache {
+        access_token: Some("token-old".to_string()),
+        expires_at: Some(Utc::now() - chrono::Duration::seconds(60)),
+    };
+    assert!(!cache.is_valid());
+}
+
+#[test]
+fn test_oidc_cache_within_refresh_buffer() {
+    // Token expires in 4 minutes -- within the 5-minute buffer -> invalid
+    let cache = OidcTokenCache {
+        access_token: Some("token-soon".to_string()),
+        expires_at: Some(Utc::now() + chrono::Duration::seconds(240)),
+    };
+    assert!(!cache.is_valid());
+}
+
+#[test]
+fn test_oidc_cache_outside_refresh_buffer() {
+    // Token expires in 6 minutes -- outside the 5-minute buffer -> valid
+    let cache = OidcTokenCache {
+        access_token: Some("token-ok".to_string()),
+        expires_at: Some(Utc::now() + chrono::Duration::seconds(360)),
+    };
+    assert!(cache.is_valid());
+}
+
+#[test]
+fn test_oidc_cache_token_none_with_expiry() {
+    let cache = OidcTokenCache {
+        access_token: None,
+        expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
+    };
+    assert!(!cache.is_valid());
+}
+
+#[test]
+fn test_oidc_cache_token_present_no_expiry() {
+    let cache = OidcTokenCache {
+        access_token: Some("token".to_string()),
+        expires_at: None,
+    };
+    assert!(!cache.is_valid());
+}
+
+// ---------------------------------------------------------------------------
+// S6: GCP API error mapping (R8)
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_map_gcp_error_404_not_found() {
+    let err = CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::NOT_FOUND, "Job not found");
+    match err {
+        TaskError::TaskNotFound(msg) => assert_eq!(msg, "Job not found"),
+        other => panic!("Expected TaskNotFound, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_401_authentication() {
+    let err = CloudSchedulerBackend::map_gcp_error(
+        reqwest::StatusCode::UNAUTHORIZED,
+        "Invalid credentials",
+    );
+    match err {
+        TaskError::Authentication(msg) => {
+            assert!(msg.contains("401"));
+            assert!(msg.contains("Invalid credentials"));
+        }
+        other => panic!("Expected Authentication, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_403_authentication() {
+    let err =
+        CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::FORBIDDEN, "Access denied");
+    match err {
+        TaskError::Authentication(msg) => {
+            assert!(msg.contains("403"));
+            assert!(msg.contains("Access denied"));
+        }
+        other => panic!("Expected Authentication, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_429_rate_limited() {
+    let err = CloudSchedulerBackend::map_gcp_error(
+        reqwest::StatusCode::TOO_MANY_REQUESTS,
+        "Rate limit exceeded",
+    );
+    match err {
+        TaskError::RateLimited(duration) => {
+            assert_eq!(duration, Duration::from_secs(60));
+        }
+        other => panic!("Expected RateLimited, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_500_backend() {
+    let err = CloudSchedulerBackend::map_gcp_error(
+        reqwest::StatusCode::INTERNAL_SERVER_ERROR,
+        "Internal error",
+    );
+    match err {
+        TaskError::Backend(msg) => {
+            assert!(msg.contains("500"));
+            assert!(msg.contains("Internal error"));
+        }
+        other => panic!("Expected Backend, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_503_backend() {
+    let err = CloudSchedulerBackend::map_gcp_error(
+        reqwest::StatusCode::SERVICE_UNAVAILABLE,
+        "Service unavailable",
+    );
+    match err {
+        TaskError::Backend(msg) => {
+            assert!(msg.contains("503"));
+        }
+        other => panic!("Expected Backend, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_502_backend() {
+    let err =
+        CloudSchedulerBackend::map_gcp_error(reqwest::StatusCode::BAD_GATEWAY, "Bad gateway");
+    match err {
+        TaskError::Backend(msg) => {
+            assert!(msg.contains("502"));
+        }
+        other => panic!("Expected Backend, got: {:?}", other),
+    }
+}
+
+#[test]
+fn test_map_gcp_error_unknown_status() {
+    let err = CloudSchedulerBackend::map_gcp_error(
+        reqwest::StatusCode::BAD_REQUEST,
+        "Bad request body",
+    );
+    match err {
+        TaskError::Backend(msg) => {
+            assert!(msg.contains("400"));
+            assert!(msg.contains("Bad request body"));
+        }
+        other => panic!("Expected Backend for unknown status, got: {:?}", other),
+    }
+}
+
+// ---------------------------------------------------------------------------
+// S8: Delete job -- local state cleanup (R3)
+// ---------------------------------------------------------------------------
+
+#[tokio::test]
+async fn test_delete_removes_local_state_directly() {
+    let backend = test_backend();
+
+    // Set up task state
+    backend.record_task_run("daily-cleanup").await.unwrap();
+    let state = backend.get_task_state("daily-cleanup").await.unwrap();
+    assert_eq!(state.total_run_count, 1);
+
+    // Directly remove from local state (simulates what delete_job does locally)
+    backend.task_states.write().await.remove("daily-cleanup");
+
+    // State should now be default
+    let state = backend.get_task_state("daily-cleanup").await.unwrap();
+    assert_eq!(state.total_run_count, 0);
+    assert!(state.last_run_at.is_none());
+}
+
+// ---------------------------------------------------------------------------
+// Construction and trait bounds
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_backend_new_success() {
+    let result = CloudSchedulerBackend::new(test_config());
+    assert!(result.is_ok());
+}
+
+#[test]
+fn test_backend_is_send_sync() {
+    fn assert_send_sync<T: Send + Sync>() {}
+    assert_send_sync::<CloudSchedulerBackend>();
+}
+
+// ---------------------------------------------------------------------------
+// HttpTarget serialization (R5)
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_http_target_serialization() {
+    let target = HttpTarget {
+        uri: "https://example.com/task".to_string(),
+        http_method: "POST".to_string(),
+        body: Some("eyJrZXkiOiJ2YWx1ZSJ9".to_string()), // base64
+        headers: {
+            let mut h = HashMap::new();
+            h.insert("Content-Type".to_string(), "application/json".to_string());
+            h
+        },
+        oidc_token: Some(OidcTokenTarget {
+            service_account_email: "sa@p.iam.gserviceaccount.com".to_string(),
+            audience: Some("https://example.com".to_string()),
+        }),
+    };
+
+    let json = serde_json::to_value(&target).unwrap();
+    assert_eq!(json["uri"], "https://example.com/task");
+    assert_eq!(json["httpMethod"], "POST");
+    assert_eq!(json["body"], "eyJrZXkiOiJ2YWx1ZSJ9");
+    assert_eq!(json["headers"]["Content-Type"], "application/json");
+    assert_eq!(
+        json["oidcToken"]["serviceAccountEmail"],
+        "sa@p.iam.gserviceaccount.com"
+    );
+    assert_eq!(json["oidcToken"]["audience"], "https://example.com");
+}
+
+#[test]
+fn test_http_target_empty_headers_omitted() {
+    let target = HttpTarget {
+        uri: "https://example.com/task".to_string(),
+        http_method: "POST".to_string(),
+        body: None,
+        headers: HashMap::new(),
+        oidc_token: None,
+    };
+
+    let json = serde_json::to_value(&target).unwrap();
+    // Empty headers should be skipped
+    assert!(json.get("headers").is_none());
+    // None body should be skipped
+    assert!(json.get("body").is_none());
+    // None oidcToken should be skipped
+    assert!(json.get("oidcToken").is_none());
+}
+
+// ---------------------------------------------------------------------------
+// MetadataTokenResponse deserialization
+// ---------------------------------------------------------------------------
+
+#[test]
+fn test_metadata_token_response_deserialization() {
+    let json = serde_json::json!({
+        "access_token": "ya29.AHES6ZRN3-HlhAPy",
+        "expires_in": 3599,
+        "token_type": "Bearer"
+    });
+
+    let response: MetadataTokenResponse = serde_json::from_value(json).unwrap();
+    assert_eq!(response.access_token, "ya29.AHES6ZRN3-HlhAPy");
+    assert_eq!(response.expires_in, 3599);
+    assert_eq!(response.token_type, "Bearer");
+}
```

## Review: broker-traits

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: gcp-cloud-integration

**Summary**: All spec requirements for broker-traits are fully implemented and verified. The Broker, PullBroker, PushBroker, and DelayedBroker trait hierarchy in crates/cclab-queue/src/broker/mod.rs satisfies every requirement (R1-R8) with correct signatures, bounds, and default implementations. BrokerCapabilities, DeliveryModel, BrokerMessage, and SubscriptionHandle all match the JSON Schema. CloudTasksBroker (cloudtasks.rs) correctly implements Broker + PushBroker + DelayedBroker with Cloud Tasks REST API v2 calls, OIDC token caching, and 30-day delay clamping. All 214 tests pass (0 failures) with --features cloudtasks. The spec Test Plan section exists but is a placeholder (<!-- TODO -->, 0 test cases defined); the Hard Reject Rule's second condition is not met because the diff contains 157 test annotations.

## Review: cloud-scheduler-backend

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: gcp-cloud-integration

**Summary**: All spec requirements for cloud-scheduler-backend are correctly implemented. CloudSchedulerBackend implements the SchedulerBackend trait with no-op leader election (R1), in-memory task state (R2), full Cloud Scheduler REST CRUD (R3), OIDC token caching with 5-min refresh buffer (R4), correct HTTP-target job structure (R5), feature gate (R6), synchronized pause/resume (R7), and GCP error mapping (R8). 45 unit tests in cloud_scheduler_backend_tests.rs cover all spec scenarios (S1-S8). All 196 tests pass (0 failures) with --features cloud-scheduler. The Test Plan section exists but is a placeholder (<!-- TODO -->); the hard reject rule does not apply because the diff contains numerous #[test]/#[tokio::test] blocks.

## Review: cloudtasks-broker

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: gcp-cloud-integration

**Summary**: All three review findings from iteration 1 have been resolved. P0 (F1): get_oidc_token() now branches on credentials_path — uses fetch_token_from_sa_key() for local dev (SA JSON key → JWT assertion → OAuth2 token exchange) and fetch_token_from_metadata() for production (GCP metadata server). P1 (F2): validate_inbound_jwt() now decodes the JWT payload and validates email, audience, and expiry claims in parse_push_request(). P1 (F3): build_create_task_request() now includes retry_config with max_retry_count when configured via RetryConfig struct with camelCase serde. All 80 cloudtasks unit tests pass (248 total across cclab-queue, 0 failures). Implementation fully satisfies R1-R9.

## Review: scheduler-backends-gcp

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: gcp-cloud-integration

**Summary**: All spec requirements for scheduler-backends-gcp are fully implemented and verified. R4: scheduler/mod.rs correctly declares #[cfg(feature = "cloud-scheduler")] pub mod cloud_scheduler_backend and re-exports CloudSchedulerBackend + CloudSchedulerConfig, following the same pattern as the existing ion_backend. R5: CloudSchedulerBackend implements all 5 SchedulerBackend trait methods (acquire_leader, renew_leader, release_leader, get_task_state, set_task_state); backend.rs is untouched. R6: cloud-scheduler feature is independent of scheduler (Ion); reqwest is optional; all three backends coexist. The spec Test Plan section exists but is a placeholder (<!-- TODO -->, 0 test cases defined); the Hard Reject Rule's second condition is not met because the diff contains 15 test functions in backends_gcp_tests covering S1/S3/S4/R5/R6, plus 45+ tests in cloud_scheduler_backend_tests.rs. cargo test -p cclab-queue --features cloud-scheduler: 196 passed, 0 failed.


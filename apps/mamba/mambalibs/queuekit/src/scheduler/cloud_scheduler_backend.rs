//! GCP Cloud Scheduler backend
//!
//! Offloads cron/interval scheduling to GCP Cloud Scheduler service.
//! Leader election is a no-op since GCP manages scheduling authority.
//! Task state is tracked locally with optional persistence.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::backend::{SchedulerBackend, SchedulingMode, TaskScheduleState};
use super::periodic::{PeriodicSchedule, PeriodicTask};
use crate::TaskError;

/// Cloud Scheduler REST API v1 base URL
const CLOUD_SCHEDULER_API_BASE: &str = "https://cloudscheduler.googleapis.com/v1";

/// GCP metadata server token endpoint
const METADATA_TOKEN_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

/// Token refresh buffer (5 minutes before expiry)
const TOKEN_REFRESH_BUFFER_SECS: i64 = 300;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for Cloud Scheduler backend
#[derive(Debug, Clone)]
pub struct CloudSchedulerConfig {
    /// GCP project ID
    pub project_id: String,
    /// GCP region (e.g., "us-central1")
    pub location: String,
    /// Service account email for OIDC token in httpTarget
    pub oidc_service_account_email: String,
    /// Base URL of the application task endpoint
    pub target_base_url: String,
    /// IANA time zone for schedule evaluation
    pub time_zone: String,
    /// Path to service account JSON key file for local dev (None = metadata server)
    pub credentials_path: Option<String>,
}

impl Default for CloudSchedulerConfig {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            oidc_service_account_email: String::new(),
            target_base_url: String::new(),
            time_zone: "UTC".to_string(),
            credentials_path: None,
        }
    }
}

impl CloudSchedulerConfig {
    /// Jobs parent path for API calls
    fn jobs_parent(&self) -> String {
        format!("projects/{}/locations/{}", self.project_id, self.location)
    }

    /// Fully qualified job name
    fn job_name(&self, job_id: &str) -> String {
        format!("{}/jobs/{}", self.jobs_parent(), job_id)
    }
}

// ---------------------------------------------------------------------------
// OIDC Token Cache
// ---------------------------------------------------------------------------

/// Cached OIDC bearer token with expiry tracking
#[derive(Debug, Clone)]
struct OidcTokenCache {
    access_token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

impl OidcTokenCache {
    fn new() -> Self {
        Self {
            access_token: None,
            expires_at: None,
        }
    }

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

// ---------------------------------------------------------------------------
// Cloud Scheduler API types
// ---------------------------------------------------------------------------

/// GCP Cloud Scheduler Job representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSchedulerJob {
    /// Fully qualified job name: projects/{project}/locations/{location}/jobs/{jobId}
    pub name: String,
    /// Unix-cron format schedule (5 fields)
    pub schedule: String,
    /// IANA time zone
    #[serde(default = "default_timezone")]
    pub time_zone: String,
    /// HTTP target configuration
    pub http_target: HttpTarget,
    /// GCP-managed job state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Last user-initiated update time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_update_time: Option<String>,
    /// Last execution attempt time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_time: Option<String>,
    /// Execution status from last attempt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<serde_json::Value>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// HTTP target configuration for Cloud Scheduler job
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpTarget {
    /// Full URL of the task endpoint
    pub uri: String,
    /// HTTP method
    pub http_method: String,
    /// Base64-encoded request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Request headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// OIDC token configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc_token: Option<OidcTokenTarget>,
}

/// OIDC token target in Cloud Scheduler httpTarget
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OidcTokenTarget {
    pub service_account_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// Response from Cloud Scheduler list jobs API
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListJobsResponse {
    #[serde(default)]
    jobs: Vec<CloudSchedulerJob>,
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

// ---------------------------------------------------------------------------
// CloudSchedulerBackend
// ---------------------------------------------------------------------------

/// Scheduler backend backed by GCP Cloud Scheduler
///
/// Leader election is a no-op — GCP Cloud Scheduler is the single
/// authoritative scheduler. Task state is tracked locally in-memory.
pub struct CloudSchedulerBackend {
    config: CloudSchedulerConfig,
    /// HTTP client for Cloud Scheduler API calls
    client: reqwest::Client,
    /// Cached OIDC access token for API authentication
    token_cache: Arc<RwLock<OidcTokenCache>>,
    /// In-memory task state store
    task_states: Arc<RwLock<HashMap<String, TaskScheduleState>>>,
}

impl CloudSchedulerBackend {
    /// Create a new Cloud Scheduler backend
    pub fn new(config: CloudSchedulerConfig) -> Result<Self, TaskError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TaskError::Connection(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            config,
            client,
            token_cache: Arc::new(RwLock::new(OidcTokenCache::new())),
            task_states: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get OIDC bearer token, using cache when valid
    async fn get_oidc_token(&self) -> Result<String, TaskError> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if cache.is_valid() {
                return Ok(cache.access_token.clone().unwrap());
            }
        }

        // Fetch new token from metadata server
        let response = self
            .client
            .get(METADATA_TOKEN_URL)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|e| TaskError::Authentication(format!("Failed to fetch OIDC token: {}", e)))?;

        if !response.status().is_success() {
            return Err(TaskError::Authentication(format!(
                "Metadata server returned status {}",
                response.status()
            )));
        }

        let token_response: MetadataTokenResponse = response.json().await.map_err(|e| {
            TaskError::Authentication(format!("Failed to parse token response: {}", e))
        })?;

        let expires_at = Utc::now() + chrono::Duration::seconds(token_response.expires_in);

        // Update cache
        let mut cache = self.token_cache.write().await;
        cache.access_token = Some(token_response.access_token.clone());
        cache.expires_at = Some(expires_at);

        Ok(token_response.access_token)
    }

    /// Map GCP HTTP error status to TaskError
    fn map_gcp_error(status: reqwest::StatusCode, body: &str) -> TaskError {
        match status.as_u16() {
            404 => TaskError::TaskNotFound(body.to_string()),
            401 | 403 => TaskError::Authentication(format!(
                "GCP API authentication error ({}): {}",
                status, body
            )),
            429 => TaskError::RateLimited(Duration::from_secs(60)),
            500..=599 => TaskError::Backend(format!("GCP API server error ({}): {}", status, body)),
            _ => TaskError::Backend(format!("GCP API error ({}): {}", status, body)),
        }
    }

    // -----------------------------------------------------------------------
    // Cloud Scheduler REST API methods
    // -----------------------------------------------------------------------

    /// Create a Cloud Scheduler job
    pub async fn create_job(
        &self,
        job: &CloudSchedulerJob,
    ) -> Result<CloudSchedulerJob, TaskError> {
        let token = self.get_oidc_token().await?;
        let url = format!(
            "{}/{}/jobs",
            CLOUD_SCHEDULER_API_BASE,
            self.config.jobs_parent()
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .json(job)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Create job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let created: CloudSchedulerJob = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(created)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// Update an existing Cloud Scheduler job
    pub async fn update_job(
        &self,
        job: &CloudSchedulerJob,
    ) -> Result<CloudSchedulerJob, TaskError> {
        let token = self.get_oidc_token().await?;
        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job.name);

        let response = self
            .client
            .patch(&url)
            .bearer_auth(&token)
            .json(job)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Update job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let updated: CloudSchedulerJob = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(updated)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// Delete a Cloud Scheduler job
    pub async fn delete_job(&self, name: &str) -> Result<(), TaskError> {
        let token = self.get_oidc_token().await?;
        let job_name = if name.starts_with("projects/") {
            name.to_string()
        } else {
            self.config.job_name(name)
        };
        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job_name);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Delete job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            // Remove local task state
            let task_id = name.rsplit('/').next().unwrap_or(name);
            self.task_states.write().await.remove(task_id);
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// Get a single Cloud Scheduler job
    pub async fn get_job(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
        let token = self.get_oidc_token().await?;
        let job_name = if name.starts_with("projects/") {
            name.to_string()
        } else {
            self.config.job_name(name)
        };
        let url = format!("{}/{}", CLOUD_SCHEDULER_API_BASE, job_name);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Get job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let job: CloudSchedulerJob = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(job)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// List all Cloud Scheduler jobs
    pub async fn list_jobs(&self) -> Result<Vec<CloudSchedulerJob>, TaskError> {
        let token = self.get_oidc_token().await?;
        let url = format!(
            "{}/{}/jobs",
            CLOUD_SCHEDULER_API_BASE,
            self.config.jobs_parent()
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("List jobs request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let list: ListJobsResponse = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(list.jobs)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// Pause a Cloud Scheduler job via API
    async fn pause_job_api(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
        let token = self.get_oidc_token().await?;
        let job_name = if name.starts_with("projects/") {
            name.to_string()
        } else {
            self.config.job_name(name)
        };
        let url = format!("{}/{}:pause", CLOUD_SCHEDULER_API_BASE, job_name);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Pause job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let job: CloudSchedulerJob = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(job)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }

    /// Resume a Cloud Scheduler job via API
    async fn resume_job_api(&self, name: &str) -> Result<CloudSchedulerJob, TaskError> {
        let token = self.get_oidc_token().await?;
        let job_name = if name.starts_with("projects/") {
            name.to_string()
        } else {
            self.config.job_name(name)
        };
        let url = format!("{}/{}:resume", CLOUD_SCHEDULER_API_BASE, job_name);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| TaskError::Connection(format!("Resume job request failed: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let job: CloudSchedulerJob = response
                .json()
                .await
                .map_err(|e| TaskError::Deserialization(e.to_string()))?;
            Ok(job)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(Self::map_gcp_error(status, &body))
        }
    }
}

// ---------------------------------------------------------------------------
// SchedulerBackend trait implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl SchedulerBackend for CloudSchedulerBackend {
    /// Returns `ExternalPush` — scheduling is managed by GCP Cloud Scheduler.
    fn scheduling_mode(&self) -> SchedulingMode {
        SchedulingMode::ExternalPush
    }

    /// Create a Cloud Scheduler job for the given periodic task.
    ///
    /// Converts `PeriodicTask.schedule` to Cloud Scheduler format:
    /// - `Cron(expr)` — 6-field cron (with seconds) is converted to 5-field unix-cron
    ///   by stripping the leading seconds field.
    /// - `Interval(secs)` — converted to a `every Ns` style schedule or cron equivalent.
    async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError> {
        let schedule_str = match &task.schedule {
            #[cfg(feature = "scheduler")]
            PeriodicSchedule::Cron(expr) => {
                // The cron crate uses 6-field format (sec min hour dom month dow).
                // Cloud Scheduler uses standard 5-field unix-cron (min hour dom month dow).
                // Strip leading seconds field if 6+ fields.
                let fields: Vec<&str> = expr.split_whitespace().collect();
                if fields.len() >= 6 {
                    // Drop the seconds field (first field)
                    fields[1..].join(" ")
                } else {
                    expr.clone()
                }
            }
            PeriodicSchedule::Interval(secs) => {
                // Convert interval to cron-compatible expression.
                // For intervals that map cleanly to minutes, use cron; otherwise
                // use Cloud Scheduler's "every Xs" notation.
                if *secs >= 60 && *secs % 60 == 0 {
                    let minutes = *secs / 60;
                    format!("*/{} * * * *", minutes)
                } else {
                    // Cloud Scheduler doesn't support sub-minute cron, but
                    // the interval field is documented as "every Xs" in the
                    // schedule string for custom intervals.
                    format!("every {}s", secs)
                }
            }
        };

        let push_url = format!(
            "{}/scheduler/push/{}",
            self.config.target_base_url, task.name
        );

        let body_json = serde_json::json!({
            "task_name": task.task_name,
            "args": task.args,
        });
        let body_b64 =
            BASE64_STANDARD.encode(serde_json::to_string(&body_json).unwrap_or_default());

        let job = CloudSchedulerJob {
            name: self.config.job_name(&task.name),
            schedule: schedule_str,
            time_zone: self.config.time_zone.clone(),
            http_target: HttpTarget {
                uri: push_url,
                http_method: "POST".to_string(),
                body: Some(body_b64),
                headers: HashMap::new(),
                oidc_token: Some(OidcTokenTarget {
                    service_account_email: self.config.oidc_service_account_email.clone(),
                    audience: None,
                }),
            },
            state: None,
            user_update_time: None,
            last_attempt_time: None,
            status: None,
        };

        self.create_job(&job).await?;
        tracing::info!(
            task_name = %task.name,
            "Registered Cloud Scheduler job"
        );
        Ok(())
    }

    /// No-op: GCP Cloud Scheduler is the single authoritative scheduler
    async fn acquire_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
        Ok(true)
    }

    /// No-op: no leader lease to renew
    async fn renew_leader(&self, _ttl: Duration) -> Result<bool, TaskError> {
        Ok(true)
    }

    /// No-op: no leader lease to release
    async fn release_leader(&self) -> Result<(), TaskError> {
        Ok(())
    }

    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError> {
        let states = self.task_states.read().await;
        Ok(states.get(name).cloned().unwrap_or_default())
    }

    async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError> {
        self.task_states
            .write()
            .await
            .insert(name.to_string(), state.clone());
        Ok(())
    }

    /// Override: pause both GCP job and local state
    async fn pause_task(&self, name: &str) -> Result<(), TaskError> {
        // Pause on GCP side
        self.pause_job_api(name).await?;

        // Update local state
        let mut states = self.task_states.write().await;
        let state = states
            .entry(name.to_string())
            .or_insert_with(TaskScheduleState::default);
        state.enabled = false;
        Ok(())
    }

    /// Override: resume both GCP job and local state
    async fn resume_task(&self, name: &str) -> Result<(), TaskError> {
        // Resume on GCP side
        self.resume_job_api(name).await?;

        // Update local state
        let mut states = self.task_states.write().await;
        let state = states
            .entry(name.to_string())
            .or_insert_with(TaskScheduleState::default);
        state.enabled = true;
        Ok(())
    }
}

#[cfg(test)]
#[path = "cloud_scheduler_backend_tests.rs"]
mod tests;

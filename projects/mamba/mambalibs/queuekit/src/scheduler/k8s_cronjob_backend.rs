//! Kubernetes CronJob Scheduler Backend
//!
//! Implements SchedulerBackend backed by Kubernetes CronJob resources.
//! Leader election is a no-op — K8s CronJob controller is the authoritative scheduler.
//! Task state is tracked locally in-memory with Arc<RwLock<HashMap>>.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use k8s_openapi::api::batch::v1::{CronJob, CronJobSpec, JobSpec, JobTemplateSpec};
use k8s_openapi::api::core::v1::{
    Container, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, ResourceRequirements,
    SecretKeySelector,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, ListParams, Patch, PatchParams, PostParams};
use kube::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::backend::{SchedulerBackend, SchedulingMode, TaskScheduleState};
use super::periodic::{PeriodicSchedule, PeriodicTask};
use crate::message::TaskMessage;
use crate::TaskError;

// ---------------------------------------------------------------------------
// Resource configuration
// ---------------------------------------------------------------------------

/// Resource limits and requests for the CronJob trigger pod container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerPodResources {
    /// CPU limit (e.g., "100m", "0.5")
    pub cpu_limit: String,
    /// Memory limit (e.g., "64Mi", "128Mi")
    pub memory_limit: String,
    /// CPU request
    pub cpu_request: String,
    /// Memory request
    pub memory_request: String,
}

impl Default for TriggerPodResources {
    fn default() -> Self {
        Self {
            cpu_limit: "100m".to_string(),
            memory_limit: "64Mi".to_string(),
            cpu_request: "50m".to_string(),
            memory_request: "32Mi".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for K8sCronJobBackend
///
/// Requires `target_base_url` and `trigger_image`. All other fields have defaults.
#[derive(Debug, Clone)]
pub struct K8sCronJobConfig {
    /// K8s namespace for CronJob resources
    pub namespace: String,
    /// Base URL of the push receiver endpoint (e.g., "https://app.example.com")
    pub target_base_url: String,
    /// Container image for the CronJob trigger pod (minimal HTTP client)
    pub trigger_image: String,
    /// K8s Secret name containing the HMAC signing key
    pub hmac_secret_name: String,
    /// Key within the K8s Secret that holds the HMAC value
    pub hmac_secret_key: String,
    /// CronJob concurrencyPolicy — Forbid prevents overlapping trigger executions
    pub concurrency_policy: String,
    /// Number of successful finished CronJob pods to retain
    pub successful_jobs_history_limit: i32,
    /// Number of failed finished CronJob pods to retain
    pub failed_jobs_history_limit: i32,
    /// Default resource limits/requests for trigger pods
    pub default_resources: TriggerPodResources,
    /// Path to kubeconfig file (None = in-cluster config or default kubeconfig)
    pub kubeconfig_path: Option<String>,
}

impl K8sCronJobConfig {
    /// Create a new config with required fields and defaults for all optional fields
    pub fn new(target_base_url: impl Into<String>, trigger_image: impl Into<String>) -> Self {
        Self {
            target_base_url: target_base_url.into(),
            trigger_image: trigger_image.into(),
            ..Default::default()
        }
    }
}

impl Default for K8sCronJobConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            target_base_url: String::new(),
            trigger_image: String::new(),
            hmac_secret_name: "scheduler-hmac-secret".to_string(),
            hmac_secret_key: "hmac-key".to_string(),
            concurrency_policy: "Forbid".to_string(),
            successful_jobs_history_limit: 1,
            failed_jobs_history_limit: 3,
            default_resources: TriggerPodResources::default(),
            kubeconfig_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Backend struct
// ---------------------------------------------------------------------------

/// Scheduler backend backed by Kubernetes CronJob resources
///
/// Leader election is a no-op — K8s CronJob controller is the single
/// authoritative scheduler. Task state is tracked locally in-memory.
pub struct K8sCronJobBackend {
    config: K8sCronJobConfig,
    #[allow(dead_code)]
    client: Client,
    cronjob_api: Api<CronJob>,
    task_states: Arc<RwLock<HashMap<String, TaskScheduleState>>>,
}

impl K8sCronJobBackend {
    /// Create a new K8s CronJob backend using default kubeconfig or in-cluster config
    pub async fn new(config: K8sCronJobConfig) -> Result<Self, TaskError> {
        let client = Client::try_default()
            .await
            .map_err(|e| TaskError::Connection(format!("Failed to create K8s client: {}", e)))?;

        let cronjob_api: Api<CronJob> = Api::namespaced(client.clone(), &config.namespace);

        Ok(Self {
            config,
            client,
            cronjob_api,
            task_states: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create backend with an existing kube Client (for testing/injection)
    pub fn with_client(config: K8sCronJobConfig, client: Client) -> Self {
        let cronjob_api: Api<CronJob> = Api::namespaced(client.clone(), &config.namespace);
        Self {
            config,
            client,
            cronjob_api,
            task_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // -----------------------------------------------------------------------
    // CronJob CRUD operations
    // -----------------------------------------------------------------------

    /// Create a K8s CronJob for a scheduled task
    pub async fn create_cronjob(
        &self,
        name: &str,
        schedule: &str,
        task_message: &TaskMessage,
    ) -> Result<CronJob, TaskError> {
        let cronjob = self.build_cronjob_spec(name, schedule, task_message);
        self.cronjob_api
            .create(&PostParams::default(), &cronjob)
            .await
            .map_err(Self::map_kube_error)
    }

    /// Update an existing K8s CronJob schedule via merge patch
    pub async fn update_cronjob(&self, name: &str, schedule: &str) -> Result<CronJob, TaskError> {
        let patch = serde_json::json!({
            "spec": {
                "schedule": schedule
            }
        });
        self.cronjob_api
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(Self::map_kube_error)
    }

    /// Delete a K8s CronJob and remove its local task state
    pub async fn delete_cronjob(&self, name: &str) -> Result<(), TaskError> {
        self.cronjob_api
            .delete(name, &Default::default())
            .await
            .map_err(Self::map_kube_error)?;

        // Remove local task state
        self.task_states.write().await.remove(name);

        tracing::info!(name = %name, "Deleted K8s CronJob");
        Ok(())
    }

    /// Get a K8s CronJob by name
    pub async fn get_cronjob(&self, name: &str) -> Result<CronJob, TaskError> {
        self.cronjob_api
            .get(name)
            .await
            .map_err(Self::map_kube_error)
    }

    /// List all K8s CronJobs in the configured namespace
    pub async fn list_cronjobs(&self) -> Result<Vec<CronJob>, TaskError> {
        let list = self
            .cronjob_api
            .list(&ListParams::default())
            .await
            .map_err(Self::map_kube_error)?;
        Ok(list.items)
    }

    // -----------------------------------------------------------------------
    // CronJob resource construction
    // -----------------------------------------------------------------------

    /// Construct a K8s CronJob resource for the given task
    ///
    /// The trigger container receives the push URL, task payload, and HMAC secret
    /// (from a K8s Secret) as environment variables. The container image is
    /// responsible for signing and delivering the HTTP request.
    pub fn build_cronjob_spec(
        &self,
        name: &str,
        schedule: &str,
        task_message: &TaskMessage,
    ) -> CronJob {
        let push_url = format!("{}/scheduler/push/{}", self.config.target_base_url, name);

        let payload = serde_json::to_string(task_message).unwrap_or_default();

        let resources = self.build_resource_requirements();

        // Environment variables for the trigger container.
        // SCHEDULER_HMAC_SECRET is injected from a K8s Secret so the container
        // can sign requests with X-Scheduler-Signature: sha256={hmac}.
        let env_vars = vec![
            EnvVar {
                name: "SCHEDULER_PUSH_URL".to_string(),
                value: Some(push_url),
                ..Default::default()
            },
            EnvVar {
                name: "SCHEDULER_TASK_PAYLOAD".to_string(),
                value: Some(payload),
                ..Default::default()
            },
            EnvVar {
                name: "SCHEDULER_TASK_NAME".to_string(),
                value: Some(name.to_string()),
                ..Default::default()
            },
            // HMAC secret from K8s Secret — never embedded in plain text
            EnvVar {
                name: "SCHEDULER_HMAC_SECRET".to_string(),
                value_from: Some(EnvVarSource {
                    secret_key_ref: Some(SecretKeySelector {
                        name: self.config.hmac_secret_name.clone(),
                        key: self.config.hmac_secret_key.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ];

        let container = Container {
            name: "scheduler-trigger".to_string(),
            image: Some(self.config.trigger_image.clone()),
            env: Some(env_vars),
            resources: Some(resources),
            ..Default::default()
        };

        let pod_spec = PodSpec {
            containers: vec![container],
            restart_policy: Some("Never".to_string()),
            ..Default::default()
        };

        let mut labels: BTreeMap<String, String> = BTreeMap::new();
        labels.insert(
            "app.kubernetes.io/managed-by".to_string(),
            "cclab-scheduler".to_string(),
        );
        labels.insert("cclab.io/scheduler-task".to_string(), name.to_string());

        CronJob {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(self.config.namespace.clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(CronJobSpec {
                schedule: schedule.to_string(),
                concurrency_policy: Some(self.config.concurrency_policy.clone()),
                suspend: Some(false),
                successful_jobs_history_limit: Some(self.config.successful_jobs_history_limit),
                failed_jobs_history_limit: Some(self.config.failed_jobs_history_limit),
                job_template: JobTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(JobSpec {
                        template: PodTemplateSpec {
                            spec: Some(pod_spec),
                            ..Default::default()
                        },
                        // No K8s-level retries — scheduler handles retry policy
                        backoff_limit: Some(0),
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Build K8s ResourceRequirements from TriggerPodResources config
    fn build_resource_requirements(&self) -> ResourceRequirements {
        let res = &self.config.default_resources;

        let mut requests: BTreeMap<String, Quantity> = BTreeMap::new();
        let mut limits: BTreeMap<String, Quantity> = BTreeMap::new();

        requests.insert("cpu".to_string(), Quantity(res.cpu_request.clone()));
        requests.insert("memory".to_string(), Quantity(res.memory_request.clone()));
        limits.insert("cpu".to_string(), Quantity(res.cpu_limit.clone()));
        limits.insert("memory".to_string(), Quantity(res.memory_limit.clone()));

        ResourceRequirements {
            requests: Some(requests),
            limits: Some(limits),
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // Error mapping
    // -----------------------------------------------------------------------

    /// Map kube-rs API errors to TaskError variants
    ///
    /// Mapping:
    /// - 404 → TaskError::TaskNotFound
    /// - 401/403 → TaskError::Authentication
    /// - 409 → TaskError::AlreadyExists
    /// - 5xx → TaskError::Backend
    /// - transport errors → TaskError::Connection
    pub fn map_kube_error(err: kube::Error) -> TaskError {
        match &err {
            kube::Error::Api(api_err) => match api_err.code {
                404 => TaskError::TaskNotFound(api_err.message.clone()),
                401 | 403 => TaskError::Authentication(format!(
                    "K8s API authentication error ({}): {}",
                    api_err.code, api_err.message
                )),
                409 => TaskError::AlreadyExists(api_err.message.clone()),
                500..=599 => TaskError::Backend(format!(
                    "K8s API server error ({}): {}",
                    api_err.code, api_err.message
                )),
                _ => TaskError::Backend(format!(
                    "K8s API error ({}): {}",
                    api_err.code, api_err.message
                )),
            },
            // Transport / client-side errors
            _ => TaskError::Connection(format!("K8s transport error: {}", err)),
        }
    }
}

// ---------------------------------------------------------------------------
// SchedulerBackend trait implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl SchedulerBackend for K8sCronJobBackend {
    /// Returns `ExternalPush` — scheduling is managed by K8s CronJob controller.
    fn scheduling_mode(&self) -> SchedulingMode {
        SchedulingMode::ExternalPush
    }

    /// Create a K8s CronJob resource for the given periodic task.
    ///
    /// Converts `PeriodicTask.schedule` to K8s CronJob cron format:
    /// - `Cron(expr)` — 6-field cron (with seconds) is converted to 5-field unix-cron.
    /// - `Interval(secs)` — converted to the closest cron representation.
    async fn register_external_schedule(&self, task: &PeriodicTask) -> Result<(), TaskError> {
        let schedule_str = match &task.schedule {
            #[cfg(feature = "scheduler")]
            PeriodicSchedule::Cron(expr) => {
                // The cron crate uses 6-field format (sec min hour dom month dow).
                // K8s CronJob uses standard 5-field unix-cron (min hour dom month dow).
                // Strip leading seconds field if 6+ fields.
                let fields: Vec<&str> = expr.split_whitespace().collect();
                if fields.len() >= 6 {
                    fields[1..].join(" ")
                } else {
                    expr.clone()
                }
            }
            PeriodicSchedule::Interval(secs) => {
                // Convert interval to the closest cron representation.
                // For minute-aligned intervals, use cron; otherwise approximate.
                if *secs >= 60 && *secs % 60 == 0 {
                    let minutes = *secs / 60;
                    format!("*/{} * * * *", minutes)
                } else {
                    // K8s CronJob minimum granularity is 1 minute
                    let minutes = std::cmp::max(1, *secs / 60);
                    format!("*/{} * * * *", minutes)
                }
            }
        };

        let task_message = TaskMessage::new(&task.task_name, task.args.clone());
        self.create_cronjob(&task.name, &schedule_str, &task_message)
            .await?;

        tracing::info!(
            task_name = %task.name,
            schedule = %schedule_str,
            "Registered K8s CronJob"
        );
        Ok(())
    }

    /// No-op: K8s CronJob controller is the single authoritative scheduler
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

    /// Get task state from in-memory store. Returns default state for unknown tasks.
    async fn get_task_state(&self, name: &str) -> Result<TaskScheduleState, TaskError> {
        let states = self.task_states.read().await;
        Ok(states.get(name).cloned().unwrap_or_default())
    }

    /// Persist task state to in-memory store
    async fn set_task_state(&self, name: &str, state: &TaskScheduleState) -> Result<(), TaskError> {
        self.task_states
            .write()
            .await
            .insert(name.to_string(), state.clone());
        Ok(())
    }

    /// Suspend the CronJob on K8s (spec.suspend=true) and update local state
    async fn pause_task(&self, name: &str) -> Result<(), TaskError> {
        // Patch K8s CronJob spec.suspend = true
        let patch = serde_json::json!({ "spec": { "suspend": true } });
        self.cronjob_api
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(Self::map_kube_error)?;

        // Update local state — keep in sync with K8s resource
        let mut states = self.task_states.write().await;
        let state = states
            .entry(name.to_string())
            .or_insert_with(TaskScheduleState::default);
        state.enabled = false;

        tracing::info!(name = %name, "Paused K8s CronJob (suspend=true)");
        Ok(())
    }

    /// Resume the CronJob on K8s (spec.suspend=false) and update local state
    async fn resume_task(&self, name: &str) -> Result<(), TaskError> {
        // Patch K8s CronJob spec.suspend = false
        let patch = serde_json::json!({ "spec": { "suspend": false } });
        self.cronjob_api
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(Self::map_kube_error)?;

        // Update local state — keep in sync with K8s resource
        let mut states = self.task_states.write().await;
        let state = states
            .entry(name.to_string())
            .or_insert_with(TaskScheduleState::default);
        state.enabled = true;

        tracing::info!(name = %name, "Resumed K8s CronJob (suspend=false)");
        Ok(())
    }
}

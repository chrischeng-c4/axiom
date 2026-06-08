//! Kubernetes Job Executor
//!
//! Spawns K8s Jobs for heavy/GPU/long-running tasks.

use std::collections::BTreeMap;
use std::sync::Arc;

use k8s_openapi::api::batch::v1::{Job, JobSpec};
use k8s_openapi::api::core::v1::{
    Container, EnvVar, PodSpec, PodTemplateSpec, ResourceRequirements, Toleration,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{Api, PostParams};
use kube::Client;
use serde::{Deserialize, Serialize};

use crate::message::{K8sConfig, TaskMessage};
use crate::{ResultBackend, TaskError, TaskId, TaskState};

/// K8s Job executor configuration
#[derive(Debug, Clone)]
pub struct K8sJobExecutorConfig {
    /// Default namespace for jobs
    pub namespace: String,
    /// Default container image
    pub default_image: String,
    /// Default service account
    pub default_service_account: Option<String>,
    /// Result backend URL (passed to job container)
    pub result_backend_url: String,
    /// Broker URL (passed to job container for chain continuation)
    pub broker_url: String,
    /// Job name prefix
    pub job_prefix: String,
    /// Default active deadline seconds
    pub default_deadline_seconds: Option<i64>,
    /// Default backoff limit
    pub default_backoff_limit: i32,
    /// Labels to add to all jobs
    pub labels: BTreeMap<String, String>,
    /// TTL seconds after job finishes (for automatic cleanup)
    pub ttl_seconds_after_finished: Option<i32>,
}

impl Default for K8sJobExecutorConfig {
    fn default() -> Self {
        Self {
            namespace: "default".to_string(),
            default_image: "ghcr.io/cclab/meteor-runner:latest".to_string(),
            default_service_account: None,
            result_backend_url: "redis://localhost:6379".to_string(),
            broker_url: "nats://localhost:4222".to_string(),
            job_prefix: "meteor-job".to_string(),
            default_deadline_seconds: Some(3600), // 1 hour
            default_backoff_limit: 0,             // No K8s-level retries (meteor handles retries)
            labels: BTreeMap::new(),
            ttl_seconds_after_finished: Some(3600), // Auto-cleanup after 1 hour
        }
    }
}

/// Information about an offloaded job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffloadedJobInfo {
    /// K8s Job name
    pub job_name: String,
    /// K8s namespace
    pub namespace: String,
    /// Task ID
    pub task_id: TaskId,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// K8s Job Executor
///
/// Spawns Kubernetes Jobs for tasks marked with executor="k8s-job".
pub struct K8sJobExecutor {
    config: K8sJobExecutorConfig,
    client: Client,
}

impl K8sJobExecutor {
    /// Create a new K8s Job executor
    pub async fn new(config: K8sJobExecutorConfig) -> Result<Self, TaskError> {
        let client = Client::try_default()
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to create K8s client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Create executor with existing client (for testing)
    pub fn with_client(config: K8sJobExecutorConfig, client: Client) -> Self {
        Self { config, client }
    }

    /// Generate a unique job name for a task
    fn job_name(&self, task_id: &TaskId) -> String {
        // K8s names must be lowercase and can't contain underscores
        let id_str = task_id.to_string().to_lowercase().replace('_', "-");
        // Truncate to fit K8s name limits (63 chars max)
        let max_id_len = 63 - self.config.job_prefix.len() - 1;
        let truncated = if id_str.len() > max_id_len {
            &id_str[..max_id_len]
        } else {
            &id_str
        };
        format!("{}-{}", self.config.job_prefix, truncated)
    }

    /// Spawn a K8s Job for a task (non-blocking)
    ///
    /// Returns immediately after the Job is created.
    /// The Job will write results directly to the backend.
    pub async fn spawn<R: ResultBackend>(
        &self,
        message: &TaskMessage,
        backend: Arc<R>,
    ) -> Result<OffloadedJobInfo, TaskError> {
        let task_id = &message.id;
        let default_config = K8sConfig::default();
        let k8s_config = message.k8s_config.as_ref().unwrap_or(&default_config);

        let namespace = k8s_config
            .namespace
            .clone()
            .unwrap_or_else(|| self.config.namespace.clone());

        let job_name = self.job_name(task_id);

        // Build resource requirements
        let resources = self.build_resources(k8s_config);

        // Build tolerations
        let tolerations = self.build_tolerations(k8s_config);

        // Build environment variables
        let env_vars = self.build_env_vars(message);

        // Build the container
        let container = Container {
            name: "task".to_string(),
            image: Some(
                k8s_config
                    .image
                    .clone()
                    .unwrap_or_else(|| self.config.default_image.clone()),
            ),
            command: Some(vec!["meteor-runner".to_string()]),
            args: Some(vec!["run-once".to_string()]),
            env: Some(env_vars),
            resources: Some(resources),
            ..Default::default()
        };

        // Build node selector
        let node_selector = if k8s_config.node_selector.is_empty() {
            None
        } else {
            Some(k8s_config.node_selector.clone())
        };

        // Build pod spec
        let pod_spec = PodSpec {
            containers: vec![container],
            restart_policy: Some("Never".to_string()),
            node_selector,
            tolerations: if tolerations.is_empty() {
                None
            } else {
                Some(tolerations)
            },
            service_account_name: k8s_config
                .service_account
                .clone()
                .or_else(|| self.config.default_service_account.clone()),
            ..Default::default()
        };

        // Build labels
        let mut labels = self.config.labels.clone();
        labels.insert("meteor.cclab.io/task-id".to_string(), task_id.to_string());
        labels.insert(
            "meteor.cclab.io/task-name".to_string(),
            message.task_name.clone(),
        );
        labels.insert(
            "app.kubernetes.io/managed-by".to_string(),
            "meteor".to_string(),
        );

        // Build the Job
        let job = Job {
            metadata: ObjectMeta {
                name: Some(job_name.clone()),
                namespace: Some(namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(JobSpec {
                template: PodTemplateSpec {
                    spec: Some(pod_spec),
                    ..Default::default()
                },
                backoff_limit: Some(
                    k8s_config
                        .backoff_limit
                        .unwrap_or(self.config.default_backoff_limit),
                ),
                active_deadline_seconds: k8s_config
                    .active_deadline_seconds
                    .or(self.config.default_deadline_seconds),
                ttl_seconds_after_finished: self.config.ttl_seconds_after_finished,
                ..Default::default()
            }),
            ..Default::default()
        };

        // Create the Job in K8s (idempotent - handle AlreadyExists)
        let jobs: Api<Job> = Api::namespaced(self.client.clone(), &namespace);
        match jobs.create(&PostParams::default(), &job).await {
            Ok(_) => {
                tracing::info!(
                    task_id = %task_id,
                    job_name = %job_name,
                    namespace = %namespace,
                    "Created K8s Job for task"
                );
            }
            Err(kube::Error::Api(ref err)) if err.code == 409 => {
                // Job already exists - this is fine for idempotent retries
                tracing::warn!(
                    task_id = %task_id,
                    job_name = %job_name,
                    namespace = %namespace,
                    "K8s Job already exists, proceeding with existing job"
                );
            }
            Err(e) => {
                return Err(TaskError::Backend(format!(
                    "Failed to create K8s Job: {}",
                    e
                )));
            }
        }

        // Update task state to OFFLOADED
        backend.set_state(task_id, TaskState::Offloaded).await?;

        // Store offloaded job info in metadata
        let info = OffloadedJobInfo {
            job_name: job_name.clone(),
            namespace: namespace.clone(),
            task_id: task_id.clone(),
            created_at: chrono::Utc::now(),
        };

        let meta_key = format!("offloaded:{}", task_id);
        let info_value =
            serde_json::to_value(&info).map_err(|e| TaskError::Serialization(e.to_string()))?;
        backend.set_metadata(&meta_key, info_value, None).await?;

        Ok(info)
    }

    /// Build K8s resource requirements from config
    fn build_resources(&self, config: &K8sConfig) -> ResourceRequirements {
        let mut requests: BTreeMap<String, Quantity> = BTreeMap::new();
        let mut limits: BTreeMap<String, Quantity> = BTreeMap::new();

        if let Some(cpu) = &config.resources.cpu {
            requests.insert("cpu".to_string(), Quantity(cpu.clone()));
            limits.insert("cpu".to_string(), Quantity(cpu.clone()));
        }

        if let Some(memory) = &config.resources.memory {
            requests.insert("memory".to_string(), Quantity(memory.clone()));
            limits.insert("memory".to_string(), Quantity(memory.clone()));
        }

        if let Some(gpu) = config.resources.gpu {
            limits.insert("nvidia.com/gpu".to_string(), Quantity(gpu.to_string()));
        }

        if let Some(tpu) = config.resources.tpu {
            limits.insert("google.com/tpu".to_string(), Quantity(tpu.to_string()));
        }

        // Extended resources
        for (key, value) in &config.resources.extended {
            limits.insert(key.clone(), Quantity(value.clone()));
        }

        ResourceRequirements {
            requests: if requests.is_empty() {
                None
            } else {
                Some(requests)
            },
            limits: if limits.is_empty() {
                None
            } else {
                Some(limits)
            },
            ..Default::default()
        }
    }

    /// Build K8s tolerations from config
    fn build_tolerations(&self, config: &K8sConfig) -> Vec<Toleration> {
        config
            .tolerations
            .iter()
            .map(|t| Toleration {
                key: Some(t.key.clone()),
                operator: t.operator.clone(),
                value: t.value.clone(),
                effect: t.effect.clone(),
                ..Default::default()
            })
            .collect()
    }

    /// Build environment variables for the job container
    ///
    /// Note: K8s has size limits for environment variables (~32KB per var, ~1MB total).
    /// For very large task payloads, consider storing in backend and passing a reference.
    fn build_env_vars(&self, message: &TaskMessage) -> Vec<EnvVar> {
        let payload = serde_json::to_string(message).unwrap_or_default();

        // Warn if payload is large (approaching K8s limits)
        if payload.len() > 16384 {
            tracing::warn!(
                task_id = %message.id,
                payload_size = payload.len(),
                "Task payload is large; consider storing in backend for K8s jobs"
            );
        }

        vec![
            EnvVar {
                name: "METEOR_TASK_PAYLOAD".to_string(),
                value: Some(payload),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_TASK_ID".to_string(),
                value: Some(message.id.to_string()),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_TASK_NAME".to_string(),
                value: Some(message.task_name.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_RESULT_BACKEND".to_string(),
                value: Some(self.config.result_backend_url.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_BROKER_URL".to_string(),
                value: Some(self.config.broker_url.clone()),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_ROOT_ID".to_string(),
                value: message.root_id.as_ref().map(|id| id.to_string()),
                ..Default::default()
            },
            EnvVar {
                name: "METEOR_PARENT_ID".to_string(),
                value: message.parent_id.as_ref().map(|id| id.to_string()),
                ..Default::default()
            },
        ]
    }

    /// Delete a K8s Job (cleanup)
    pub async fn delete_job(&self, job_name: &str, namespace: &str) -> Result<(), TaskError> {
        let jobs: Api<Job> = Api::namespaced(self.client.clone(), namespace);

        jobs.delete(job_name, &Default::default())
            .await
            .map_err(|e| TaskError::Backend(format!("Failed to delete K8s Job: {}", e)))?;

        tracing::info!(job_name = %job_name, namespace = %namespace, "Deleted K8s Job");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_name_generation() {
        let config = K8sJobExecutorConfig::default();
        let task_id = TaskId::new();

        let job_name = format!(
            "{}-{}",
            config.job_prefix,
            task_id.to_string().to_lowercase().replace('_', "-")
        );

        assert!(job_name.starts_with("meteor-job-"));
        assert!(job_name.len() <= 63);
    }

    #[test]
    fn test_config_defaults() {
        let config = K8sJobExecutorConfig::default();
        assert_eq!(config.namespace, "default");
        assert_eq!(config.job_prefix, "meteor-job");
        assert_eq!(config.default_backoff_limit, 0);
    }
}

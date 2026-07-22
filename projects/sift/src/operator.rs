// HANDWRITE-BEGIN gap="sift-shared-operator-controller" tracker="1606" reason="Define the Sift custom-resource type and compose the shared leader-elected operator reconcile loop."
//! Sift's small service-specific adapter over the shared operator framework.

use axiom_operator::render::{self, RenderCtx};
use axiom_operator::{ManagedService, ReadinessTarget, ReadyFacts};
use kube::{CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "sift.axiom.dev",
    version = "v1alpha1",
    kind = "Sift",
    plural = "sifts",
    namespaced,
    status = "SiftStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct SiftSpec {
    pub image: String,
    #[serde(default = "one")]
    pub replicas_per_shard: u32,
    #[serde(default = "one")]
    pub voter_count: u32,
    #[serde(default = "default_data_size")]
    pub data_size: String,
    #[serde(default)]
    pub auth: AuthMode,
    #[serde(default)]
    pub tokens_secret: Option<String>,
    #[serde(default)]
    pub backup: Option<BackupSpec>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    #[default]
    Off,
    Required,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupSpec {
    pub schedule: String,
    pub destination: String,
    #[serde(default)]
    pub retention_secs: Option<u64>,
    #[serde(default)]
    pub admin_token_secret: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SiftStatus {
    pub phase: String,
    pub observed_generation: i64,
    pub ready_replicas: i64,
    pub desired_shard_count: u32,
    pub current_shard_count: u32,
    pub desired_replicas_per_shard: u32,
    pub current_ready_replicas_per_shard: u32,
    pub backup_phase: String,
    pub backup_message: String,
    pub message: String,
}

const APP: &str = "sift";
const API_VERSION: &str = "sift.axiom.dev/v1alpha1";
const KIND: &str = "Sift";
const SERVER_COMPONENT: &str = "server";
const BACKUP_COMPONENT: &str = "backup";
const HTTP_PORT: i32 = 7380;

fn one() -> u32 {
    1
}

fn default_data_size() -> String {
    "10Gi".to_string()
}

impl ManagedService for Sift {
    const MANAGER: &'static str = "sift-operator";

    fn render(&self) -> Vec<Value> {
        let name = self.name_any();
        let namespace = self.namespace().unwrap_or_else(|| "default".to_string());
        let owner = self
            .metadata
            .uid
            .as_deref()
            .map(|uid| render::owner_ref(API_VERSION, KIND, &name, uid));
        let cx = RenderCtx {
            app: APP,
            manager: Self::MANAGER,
            api_version: API_VERSION,
            kind: KIND,
            name: &name,
            ns: &namespace,
            owner,
        };
        let labels = cx.labels(SERVER_COMPONENT);
        let selector = cx.selector(SERVER_COMPONENT);
        // Sift does not yet own safe live membership changes. The public CRD
        // therefore admits only this 1x1 topology and the renderer also clamps
        // programmatically-constructed or legacy objects to the same baseline.
        let replicas = 1_u32;
        let auth_mode = match self.spec.auth {
            AuthMode::Off => "off",
            AuthMode::Required => "required",
        };
        let mut env = vec![
            json!({"name":"POD_NAME", "valueFrom":{"fieldRef":{"fieldPath":"metadata.name"}}}),
            json!({"name":"SHARD_COUNT", "value":"1"}),
            json!({"name":"REPLICAS_PER_SHARD", "value":"1"}),
            json!({"name":"VOTER_COUNT", "value":"1"}),
            json!({"name":"SIFT_AUTH", "value":auth_mode}),
        ];
        let mut volume_mounts = vec![json!({"name":"data", "mountPath":"/var/lib/sift"})];
        let mut pod_volumes = Vec::new();
        let pod_security_context = json!({
            "runAsNonRoot": true,
            "runAsUser": 65532,
            "runAsGroup": 65532,
            "fsGroup": 65532,
            "fsGroupChangePolicy": "OnRootMismatch",
            "seccompProfile": {"type": "RuntimeDefault"},
        });
        let container_security_context = json!({
            "allowPrivilegeEscalation": false,
            "readOnlyRootFilesystem": true,
            "capabilities": {"drop": ["ALL"]},
        });
        if matches!(self.spec.auth, AuthMode::Required) {
            if let Some(secret) = &self.spec.tokens_secret {
                env.push(json!({
                    "name":"SIFT_TOKEN_REGISTRY_FILE",
                    "value":"/var/run/secrets/sift/token-registry.json"
                }));
                volume_mounts.push(json!({
                    "name":"tokens", "mountPath":"/var/run/secrets/sift", "readOnly":true
                }));
                pod_volumes.push(json!({
                    "name":"tokens", "secret":{"secretName":secret}
                }));
            }
        }
        let mut objects = vec![
            render::service_account(&cx, SERVER_COMPONENT),
            json!({
                "apiVersion": "v1", "kind": "Service",
                "metadata": cx.meta(&format!("{name}-headless"), SERVER_COMPONENT),
                "spec": {
                    "clusterIP": "None", "publishNotReadyAddresses": true,
                    "selector": selector.clone(),
                    "ports": [{"name":"http", "port":HTTP_PORT, "targetPort":"http"}]
                }
            }),
            json!({
                "apiVersion": "v1", "kind": "Service",
                "metadata": cx.meta(&name, SERVER_COMPONENT),
                "spec": {
                    "type": "ClusterIP", "selector": selector.clone(),
                    "ports": [{"name":"http", "port":HTTP_PORT, "targetPort":"http"}]
                }
            }),
            json!({
                "apiVersion": "apps/v1", "kind": "StatefulSet",
                "metadata": cx.meta(&name, SERVER_COMPONENT),
                "spec": {
                    "serviceName": format!("{name}-headless"), "replicas": replicas,
                    "selector": {"matchLabels": selector},
                    "template": {
                        "metadata": {"labels": labels},
                        "spec": {
                          "serviceAccountName": name,
                          "automountServiceAccountToken": false,
                          "enableServiceLinks": false,
                          "securityContext": pod_security_context.clone(), "volumes": pod_volumes, "containers": [{
                            "name": "sift", "image": self.spec.image,
                            "args": ["serve", "--data-dir", "/var/lib/sift"],
                            "ports": [{"name":"http", "containerPort":HTTP_PORT}],
                            "env": env,
                            "volumeMounts": volume_mounts,
                            "securityContext": container_security_context.clone(),
                            "resources": {"requests":{"cpu":"100m","memory":"256Mi"}},
                            "readinessProbe": {"httpGet":{"path":"/readyz","port":"http"},"periodSeconds":5,"timeoutSeconds":3,"failureThreshold":60},
                            "livenessProbe": {"httpGet":{"path":"/healthz","port":"http"},"periodSeconds":15,"timeoutSeconds":5,"failureThreshold":3},
                            "startupProbe": {"httpGet":{"path":"/healthz","port":"http"},"periodSeconds":5,"timeoutSeconds":3,"failureThreshold":120}
                        }]}
                    },
                    "volumeClaimTemplates": [{
                        "metadata":{"name":"data"},
                        "spec":{"accessModes":["ReadWriteOnce"],"resources":{"requests":{"storage":self.spec.data_size}}}
                    }]
                }
            }),
        ];
        if let Some(backup) = &self.spec.backup {
            let backup_name = format!("{name}-backup");
            objects.push(json!({
                "apiVersion": "v1", "kind": "ServiceAccount",
                "metadata": cx.meta(&backup_name, BACKUP_COMPONENT),
            }));
            let mut args = vec![
                "backup".to_string(),
                "--url".to_string(),
                format!("http://{name}.{namespace}.svc.cluster.local:{HTTP_PORT}"),
                "--dest".to_string(),
                backup.destination.clone(),
            ];
            if let Some(seconds) = backup.retention_secs {
                args.push("--retention-secs".to_string());
                args.push(seconds.to_string());
            }
            let mut backup_env = Vec::new();
            if let Some(secret) = &backup.admin_token_secret {
                backup_env.push(json!({
                    "name": "SIFT_BACKUP_TOKEN",
                    "valueFrom": {"secretKeyRef": {"name": secret, "key": "token"}}
                }));
            }
            objects.push(json!({
                "apiVersion":"batch/v1", "kind":"CronJob",
                "metadata": cx.meta(&backup_name, BACKUP_COMPONENT),
                "spec": {
                    "schedule": backup.schedule,
                    "concurrencyPolicy": "Forbid",
                    "successfulJobsHistoryLimit": 3,
                    "failedJobsHistoryLimit": 3,
                    "jobTemplate": {
                        "spec": {
                            "template": {
                                "metadata": {"labels": cx.labels(BACKUP_COMPONENT)},
                                "spec": {
                                    "securityContext": pod_security_context,
                                    "serviceAccountName": backup_name,
                                    "restartPolicy": "OnFailure",
                                    "containers": [{
                                        "name": "backup", "image": self.spec.image,
                                        "args": args,
                                        "env": backup_env,
                                        "securityContext": container_security_context,
                                        "resources": {"requests":{"cpu":"100m","memory":"128Mi"},"limits":{"cpu":"500m","memory":"512Mi"}}
                                    }]
                                }
                            }
                        }
                    }
                }
            }));
        }
        objects
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> Value {
        let ready_replicas = ready.get(&self.name_any()).max(0);
        let desired_replicas_per_shard = self.spec.replicas_per_shard.max(1);
        let supported_topology =
            desired_replicas_per_shard == 1 && self.spec.voter_count.max(1) == 1;
        let (phase, message) = if !supported_topology {
            (
                "UnsupportedTopology",
                format!(
                    "requested 1 shard x {desired_replicas_per_shard} replicas with {} voters; Sift currently admits only 1x1 until safe membership changes are implemented",
                    self.spec.voter_count.max(1)
                ),
            )
        } else if ready_replicas >= 1 {
            ("Ready", "1/1 Sift pod ready".to_string())
        } else {
            ("Pending", "0/1 Sift pods ready".to_string())
        };
        let (backup_phase, backup_message) = if self.spec.backup.is_some() {
            (
                "Configured",
                "scheduled live backup is configured; execution evidence is reported by its CronJob and destination",
            )
        } else {
            ("NotConfigured", "no scheduled backup requested")
        };
        json!({
            "status": {
                "phase": phase,
                "observedGeneration": self.metadata.generation.unwrap_or(0),
                "readyReplicas": ready_replicas,
                "desiredShardCount": 1,
                "currentShardCount": u32::from(ready_replicas > 0),
                "desiredReplicasPerShard": desired_replicas_per_shard,
                "currentReadyReplicasPerShard": ready_replicas as u32,
                "backupPhase": backup_phase,
                "backupMessage": backup_message,
                "message": message,
            }
        })
    }
}

pub async fn run() -> anyhow::Result<()> {
    axiom_operator::run::<Sift>().await
}
// HANDWRITE-END

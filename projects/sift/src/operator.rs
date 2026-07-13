// HANDWRITE-BEGIN gap="sift-shared-operator-controller" tracker="1606" reason="Define the Sift custom-resource type and compose the shared leader-elected operator reconcile loop."
//! Sift's small service-specific adapter over the shared operator framework.

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
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SiftStatus {
    pub phase: String,
    pub ready_replicas: i64,
}

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
        let labels = json!({"app.kubernetes.io/name": "sift", "app.kubernetes.io/instance": name});
        let replicas = self.spec.replicas_per_shard.max(1);
        let auth_mode = match self.spec.auth {
            AuthMode::Off => "off",
            AuthMode::Required => "required",
        };
        let mut env = vec![
            json!({"name":"POD_NAME", "valueFrom":{"fieldRef":{"fieldPath":"metadata.name"}}}),
            json!({"name":"SHARD_COUNT", "value":"1"}),
            json!({"name":"REPLICAS_PER_SHARD", "value":replicas.to_string()}),
            json!({"name":"VOTER_COUNT", "value":self.spec.voter_count.max(1).to_string()}),
            json!({"name":"SIFT_AUTH", "value":auth_mode}),
        ];
        let mut volume_mounts = vec![json!({"name":"data", "mountPath":"/var/lib/sift"})];
        let mut pod_volumes = Vec::new();
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
            json!({
                "apiVersion": "v1", "kind": "Service",
                "metadata": {"name": format!("{name}-headless"), "labels": labels.clone()},
                "spec": {
                    "clusterIP": "None", "selector": labels.clone(),
                    "ports": [{"name":"http", "port":7380, "targetPort":7380}]
                }
            }),
            json!({
                "apiVersion": "apps/v1", "kind": "StatefulSet",
                "metadata": {"name": name, "labels": labels.clone()},
                "spec": {
                    "serviceName": format!("{name}-headless"), "replicas": replicas,
                    "selector": {"matchLabels": labels.clone()},
                    "template": {
                        "metadata": {"labels": labels},
                        "spec": {"volumes": pod_volumes, "containers": [{
                            "name": "sift", "image": self.spec.image,
                            "args": ["serve", "--data-dir", "/var/lib/sift"],
                            "ports": [{"name":"http", "containerPort":7380}],
                            "env": env,
                            "volumeMounts": volume_mounts,
                            "readinessProbe": {"httpGet":{"path":"/readyz","port":"http"}},
                            "livenessProbe": {"httpGet":{"path":"/healthz","port":"http"}}
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
            let mut args = vec![
                "backup".to_string(),
                "--data-dir".to_string(),
                "/var/lib/sift".to_string(),
                "--dest".to_string(),
                backup.destination.clone(),
            ];
            if let Some(seconds) = backup.retention_secs {
                args.push("--retention-secs".to_string());
                args.push(seconds.to_string());
            }
            objects.push(json!({
                "apiVersion":"batch/v1", "kind":"CronJob",
                "metadata":{"name":format!("{name}-backup"), "labels":labels.clone()},
                "spec": {
                    "schedule": backup.schedule,
                    "jobTemplate": {
                        "spec": {
                            "template": {
                                "spec": {
                                    "restartPolicy": "OnFailure",
                                    "containers": [{
                                        "name": "backup", "image": self.spec.image,
                                        "args": args,
                                        "volumeMounts": [{"name":"data", "mountPath":"/var/lib/sift", "readOnly":true}]
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
        let replicas = ready.get(&self.name_any());
        json!({
            "status": {
                "phase": if replicas >= self.spec.replicas_per_shard.max(1) as i64 { "Ready" } else { "Pending" },
                "readyReplicas": replicas,
            }
        })
    }
}

pub async fn run() -> anyhow::Result<()> {
    axiom_operator::run::<Sift>().await
}
// HANDWRITE-END

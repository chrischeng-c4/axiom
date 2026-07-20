// HANDWRITE-BEGIN gap="missing-generator:logic:723cac43" tracker="#2152" reason="scaffold for apps/beam/src/operator/mod.rs — fill in by hand and update tracker when codegen is ready"
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};
use kube::ResourceExt;

/// `beam.dev/v1alpha1` `Beam`. Namespaced custom resource declarations.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "beam.dev",
    version = "v1alpha1",
    kind = "Beam",
    plural = "beams",
    shortname = "bem",
    namespaced,
    status = "BeamStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in mod.rs is hand-written pending codegen support">
#[serde(rename_all = "camelCase")]
pub struct BeamSpec {
    /// Serving container image, e.g. beam:latest.
    pub image: String,
    /// Bind address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Client API port.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u32>,
    /// Log level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    /// Graceful drain window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_secs: Option<u64>,
}
// </HANDWRITE>

/// Status subresource written back by the reconcile loop.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BeamStatus {
    /// `Pending | Reconciling | Ready`.
    #[serde(default)]
    pub phase: String,
    /// The `.metadata.generation` this status reflects.
    #[serde(default)]
    pub observed_generation: i64,
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in mod.rs is hand-written pending codegen support">
impl ManagedService for Beam {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "beam-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        let name = self.name_any();
        let namespace = self.namespace().unwrap_or_else(|| "default".to_string());
        let image = &self.spec.image;
        let port = self.spec.port.unwrap_or(7373);
        let host = self.spec.host.as_deref().unwrap_or("0.0.0.0");
        let grace_secs = self.spec.grace_secs.unwrap_or(30);

        let owner_references = if let (Some(uid), Some(name)) = (&self.metadata.uid, &self.metadata.name) {
            vec![serde_json::json!({
                "apiVersion": "beam.dev/v1alpha1",
                "kind": "Beam",
                "name": name,
                "uid": uid,
                "controller": true,
                "blockOwnerDeletion": true
            })]
        } else {
            vec![]
        };

        let service = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "app": name,
                },
                "ownerReferences": owner_references
            },
            "spec": {
                "ports": [
                    {
                        "port": port,
                        "targetPort": port,
                        "name": "http"
                    }
                ],
                "selector": {
                    "app": name
                }
            }
        });

        let mut env = vec![
            serde_json::json!({
                "name": "BEAM_PORT",
                "value": port.to_string()
            }),
            serde_json::json!({
                "name": "BEAM_HOST",
                "value": host
            }),
        ];

        if let Some(log_level) = &self.spec.log_level {
            env.push(serde_json::json!({
                "name": "RUST_LOG",
                "value": log_level
            }));
        }

        let deployment = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "app": name,
                },
                "ownerReferences": owner_references
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "app": name
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": name
                        }
                    },
                    "spec": {
                        "terminationGracePeriodSeconds": grace_secs,
                        "containers": [
                            {
                                "name": "beam",
                                "image": image,
                                "ports": [
                                    {
                                        "containerPort": port
                                    }
                                ],
                                "env": env
                            }
                        ]
                    }
                }
            }
        });

        vec![service, deployment]
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "Deployment",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let phase = if ready_replicas >= 1 {
            "Ready"
        } else {
            "Reconciling"
        };
        serde_json::json!({
            "status": {
                "phase": phase,
                "observedGeneration": self.metadata.generation.unwrap_or(0),
            }
        })
    }
}
// </HANDWRITE>

/// Run the reconcile loop for `Beam` resources.
pub async fn run() -> anyhow::Result<()> {
    service_k8s::run::<Beam>().await
}
// HANDWRITE-END

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
// <HANDWRITE gap="missing-generator:logic" tracker="#2154" reason="logic section in mod.rs is hand-written pending codegen support">
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
    /// Expected replicas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
    /// Durable storage request size, e.g. 10Gi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_size: Option<String>,
    /// Request GPU resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_gpu: Option<bool>,
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

// <HANDWRITE gap="missing-generator:logic" tracker="#2154" reason="logic section in mod.rs is hand-written pending codegen support">
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

        let profile = self.metadata.labels.as_ref()
            .and_then(|l| l.get("profile"))
            .map(|s| s.as_str())
            .unwrap_or("dev");

        // Profile-based defaults
        let (default_replicas, default_storage, is_gpu, has_backup, has_pdb) = match profile {
            "prod" => (1, "20Gi".to_string(), true, true, true),
            "staging" => (1, "5Gi".to_string(), true, false, true),
            "template" => (1, "REPLACE_ME__STORAGE_SIZE".to_string(), true, true, true),
            _ => (1, "1Gi".to_string(), false, false, false), // dev or other
        };

        let replicas = self.spec.replicas.unwrap_or(default_replicas);
        let storage_size = self.spec.storage_size.clone().unwrap_or(default_storage);
        let request_gpu = self.spec.request_gpu.unwrap_or(is_gpu);

        let owner_references = if let (Some(uid), Some(n)) = (&self.metadata.uid, &self.metadata.name) {
            vec![serde_json::json!({
                "apiVersion": "beam.dev/v1alpha1",
                "kind": "Beam",
                "name": n,
                "uid": uid,
                "controller": true,
                "blockOwnerDeletion": true
            })]
        } else {
            vec![]
        };

        // Service
        let service = serde_json::json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "app": name,
                    "profile": profile
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

        // Env vars
        let mut env = vec![
            serde_json::json!({
                "name": "BEAM_PORT",
                "value": port.to_string()
            }),
            serde_json::json!({
                "name": "BEAM_HOST",
                "value": host
            }),
            serde_json::json!({
                "name": "BEAM_DATA_DIR",
                "value": "/data"
            }),
            serde_json::json!({
                "name": "BEAM_AUTH",
                "value": "required"
            }),
            serde_json::json!({
                "name": "BEAM_TOKEN_REGISTRY_FILE",
                "value": "/var/run/secrets/beam/auth/registry.json"
            })
        ];

        if let Some(log_level) = &self.spec.log_level {
            env.push(serde_json::json!({
                "name": "RUST_LOG",
                "value": log_level
            }));
        }

        // GPU resources & scheduling placement
        let mut resources = serde_json::json!({});
        let mut node_selector = serde_json::json!({});
        let mut tolerations = serde_json::json!([]);

        if request_gpu {
            resources = serde_json::json!({
                "limits": {
                    "nvidia.com/gpu": "1"
                }
            });
            node_selector = serde_json::json!({
                "accelerator": "nvidia-gpu"
            });
            tolerations = serde_json::json!([
                {
                    "key": "nvidia.com/gpu",
                    "operator": "Exists",
                    "effect": "NoSchedule"
                }
            ]);
        }

        // StatefulSet
        let mut stateful_set = serde_json::json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": {
                "name": name,
                "namespace": namespace,
                "labels": {
                    "app": name,
                    "profile": profile
                },
                "ownerReferences": owner_references
            },
            "spec": {
                "serviceName": name,
                "replicas": replicas,
                "selector": {
                    "matchLabels": {
                        "app": name
                    }
                },
                "template": {
                    "metadata": {
                        "labels": {
                            "app": name,
                            "profile": profile
                        },
                        "annotations": {
                            "prometheus.io/scrape": "true",
                            "prometheus.io/port": port.to_string(),
                            "prometheus.io/path": "/metrics"
                        }
                    },
                    "spec": {
                        "terminationGracePeriodSeconds": grace_secs,
                        "securityContext": {
                            "runAsNonRoot": true,
                            "runAsUser": 10001,
                            "fsGroup": 10001
                        },
                        "containers": [
                            {
                                "name": "beam",
                                "image": image,
                                "ports": [
                                    {
                                        "containerPort": port,
                                        "name": "http"
                                    }
                                ],
                                "env": env,
                                "resources": resources,
                                "securityContext": {
                                    "allowPrivilegeEscalation": false,
                                    "readOnlyRootFilesystem": true,
                                    "capabilities": {
                                        "drop": ["ALL"]
                                    }
                                },
                                "volumeMounts": [
                                    {
                                        "name": "data",
                                        "mountPath": "/data"
                                    },
                                    {
                                        "name": "auth-secret",
                                        "mountPath": "/var/run/secrets/beam/auth",
                                        "readOnly": true
                                    },
                                    {
                                        "name": "tmp",
                                        "mountPath": "/tmp"
                                    }
                                ],
                                "livenessProbe": {
                                    "httpGet": {
                                        "path": "/healthz",
                                        "port": port
                                    },
                                    "initialDelaySeconds": 5,
                                    "periodSeconds": 10
                                },
                                "readinessProbe": {
                                    "httpGet": {
                                        "path": "/readyz",
                                        "port": port
                                    },
                                    "initialDelaySeconds": 5,
                                    "periodSeconds": 10
                                }
                            }
                        ],
                        "volumes": [
                            {
                                "name": "auth-secret",
                                "secret": {
                                    "secretName": format!("{name}-auth")
                                }
                            },
                            {
                                "name": "tmp",
                                "emptyDir": {}
                            }
                        ]
                    }
                },
                "volumeClaimTemplates": [
                    {
                        "metadata": {
                            "name": "data"
                        },
                        "spec": {
                            "accessModes": ["ReadWriteOnce"],
                            "resources": {
                                "requests": {
                                    "storage": storage_size
                                }
                            }
                        }
                    }
                ]
            }
        });

        // Add nodeSelector and tolerations if GPU was requested
        if request_gpu {
            if let Some(spec_obj) = stateful_set.pointer_mut("/spec/template/spec") {
                if let Some(map) = spec_obj.as_object_mut() {
                    map.insert("nodeSelector".to_string(), node_selector);
                    map.insert("tolerations".to_string(), tolerations);
                }
            }
        }

        let mut manifests = vec![service, stateful_set];

        // PDB
        if has_pdb {
            let pdb = serde_json::json!({
                "apiVersion": "policy/v1",
                "kind": "PodDisruptionBudget",
                "metadata": {
                    "name": name,
                    "namespace": namespace,
                    "labels": {
                        "app": name,
                        "profile": profile
                    },
                    "ownerReferences": owner_references
                },
                "spec": {
                    "maxUnavailable": 1,
                    "selector": {
                        "matchLabels": {
                            "app": name
                        }
                    }
                }
            });
            manifests.push(pdb);
        }

        // Backup CronJob
        if has_backup {
            let backup_cron = serde_json::json!({
                "apiVersion": "batch/v1",
                "kind": "CronJob",
                "metadata": {
                    "name": format!("{name}-backup"),
                    "namespace": namespace,
                    "labels": {
                        "app": format!("{name}-backup"),
                        "profile": profile
                    },
                    "ownerReferences": owner_references
                },
                "spec": {
                    "schedule": "0 2 * * *",
                    "concurrencyPolicy": "Forbid",
                    "jobTemplate": {
                        "spec": {
                            "template": {
                                "spec": {
                                    "restartPolicy": "OnFailure",
                                    "containers": [
                                        {
                                            "name": "backup",
                                            "image": image,
                                            "command": [
                                                "beam",
                                                "backup",
                                                "--url",
                                                format!("http://{name}:{port}"),
                                                "--dest",
                                                "s3://beam-backups/snapshot.cbor.lz4"
                                            ],
                                            "env": [
                                                {
                                                    "name": "BEAM_BACKUP_TOKEN",
                                                    "valueFrom": {
                                                        "secretKeyRef": {
                                                            "name": format!("{name}-auth"),
                                                            "key": "admin-token"
                                                        }
                                                    }
                                                },
                                                {
                                                    "name": "AWS_ACCESS_KEY_ID",
                                                    "valueFrom": {
                                                        "secretKeyRef": {
                                                            "name": "beam-backup-credentials",
                                                            "key": "aws-access-key-id"
                                                        }
                                                    }
                                                },
                                                {
                                                    "name": "AWS_SECRET_ACCESS_KEY",
                                                    "valueFrom": {
                                                        "secretKeyRef": {
                                                            "name": "beam-backup-credentials",
                                                            "key": "aws-secret-access-key"
                                                        }
                                                    }
                                                }
                                            ]
                                        }
                                    ]
                                }
                            }
                        }
                    }
                }
            });
            manifests.push(backup_cron);
        }

        manifests
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    async fn reconcile_plan(
        &self,
        client: kube::Client,
    ) -> anyhow::Result<service_k8s::service::ReconcilePlan> {
        let children = self.render();

        let profile = self.metadata.labels.as_ref()
            .and_then(|l| l.get("profile"))
            .map(|s| s.as_str())
            .unwrap_or("dev");
        let has_backup = match profile {
            "prod" | "template" => true,
            _ => false,
        };

        if !has_backup {
            let namespace = self.namespace().unwrap_or_else(|| "default".to_string());
            let name = self.name_any();
            let cron_name = format!("{name}-backup");
            let api: kube::Api<kube::api::DynamicObject> = kube::Api::namespaced_with(
                client.clone(),
                &namespace,
                &kube::api::ApiResource {
                    group: "batch".to_string(),
                    version: "v1".to_string(),
                    api_version: "batch/v1".to_string(),
                    kind: "CronJob".to_string(),
                    plural: "cronjobs".to_string(),
                },
            );

            if api.get_opt(&cron_name).await?.is_some() {
                let _ = api.delete(&cron_name, &kube::api::DeleteParams::default()).await;
                eprintln!("deleted old backup CronJob on profile downgrade: {cron_name}");
            }
        }

        Ok(service_k8s::service::ReconcilePlan {
            children,
            context: serde_json::Value::Null,
        })
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let profile = self.metadata.labels.as_ref()
            .and_then(|l| l.get("profile"))
            .map(|s| s.as_str())
            .unwrap_or("dev");

        let expected_replicas = match profile {
            "prod" => 1,
            _ => 1,
        };
        let expected_replicas = self.spec.replicas.unwrap_or(expected_replicas);

        let phase = if ready_replicas >= expected_replicas {
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

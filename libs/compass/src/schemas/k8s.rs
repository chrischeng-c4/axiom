//! Programmatic K8s JSON-Schema definitions for common resources.
//!
//! Each function returns `(kind_name, schema_value)`.  The schemas cover the
//! most commonly misconfigured required-field constraints so that `K8002`
//! catches real issues without bundling the full 15 MB upstream schemas.

use serde_json::{json, Value};

/// Build schemas for all supported resource kinds.
pub(super) fn build_all_schemas() -> Vec<(String, Value)> {
    vec![
        deployment_schema(),
        service_schema(),
        pod_schema(),
        configmap_schema(),
        secret_schema(),
        ingress_schema(),
        statefulset_schema(),
        daemonset_schema(),
        job_schema(),
        cronjob_schema(),
    ]
}

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

/// Common metadata sub-schema (requires `name`).
fn metadata_schema() -> Value {
    json!({
        "type": "object",
        "required": ["name"],
        "properties": {
            "name": { "type": "string", "minLength": 1 },
            "namespace": { "type": "string" },
            "labels": { "type": "object" },
            "annotations": { "type": "object" }
        }
    })
}

/// Container schema (requires `name` and `image`).
fn container_schema() -> Value {
    json!({
        "type": "object",
        "required": ["name", "image"],
        "properties": {
            "name": { "type": "string", "minLength": 1 },
            "image": { "type": "string", "minLength": 1 },
            "ports": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "containerPort": { "type": "integer" }
                    }
                }
            },
            "env": { "type": "array" },
            "resources": { "type": "object" },
            "volumeMounts": { "type": "array" },
            "command": { "type": "array" },
            "args": { "type": "array" },
            "livenessProbe": { "type": "object" },
            "readinessProbe": { "type": "object" },
            "securityContext": { "type": "object" }
        }
    })
}

/// Pod spec sub-schema (requires `containers`).
fn pod_spec_schema() -> Value {
    json!({
        "type": "object",
        "required": ["containers"],
        "properties": {
            "containers": {
                "type": "array",
                "minItems": 1,
                "items": container_schema()
            },
            "initContainers": {
                "type": "array",
                "items": container_schema()
            },
            "volumes": { "type": "array" },
            "serviceAccountName": { "type": "string" },
            "restartPolicy": { "type": "string" },
            "securityContext": { "type": "object" },
            "nodeSelector": { "type": "object" },
            "tolerations": { "type": "array" },
            "affinity": { "type": "object" }
        }
    })
}

/// Pod template spec (has metadata + spec).
fn pod_template_schema() -> Value {
    json!({
        "type": "object",
        "required": ["spec"],
        "properties": {
            "metadata": { "type": "object" },
            "spec": pod_spec_schema()
        }
    })
}

/// Label selector sub-schema.
fn label_selector_schema() -> Value {
    json!({
        "type": "object",
        "required": ["matchLabels"],
        "properties": {
            "matchLabels": { "type": "object" },
            "matchExpressions": { "type": "array" }
        }
    })
}

// -------------------------------------------------------------------------
// Resource schemas
// -------------------------------------------------------------------------

fn deployment_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Deployment" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["selector", "template"],
                "properties": {
                    "replicas": { "type": "integer", "minimum": 0 },
                    "selector": label_selector_schema(),
                    "template": pod_template_schema(),
                    "strategy": { "type": "object" },
                    "minReadySeconds": { "type": "integer" }
                }
            }
        }
    });
    ("Deployment".into(), schema)
}

fn service_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Service" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["ports"],
                "properties": {
                    "ports": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "required": ["port"],
                            "properties": {
                                "port": { "type": "integer" },
                                "targetPort": {},
                                "protocol": { "type": "string" },
                                "name": { "type": "string" }
                            }
                        }
                    },
                    "selector": { "type": "object" },
                    "type": { "type": "string" },
                    "clusterIP": { "type": "string" }
                }
            }
        }
    });
    ("Service".into(), schema)
}

fn pod_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Pod" },
            "metadata": metadata_schema(),
            "spec": pod_spec_schema()
        }
    });
    ("Pod".into(), schema)
}

fn configmap_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "ConfigMap" },
            "metadata": metadata_schema(),
            "data": { "type": "object" },
            "binaryData": { "type": "object" }
        }
    });
    ("ConfigMap".into(), schema)
}

fn secret_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Secret" },
            "metadata": metadata_schema(),
            "data": { "type": "object" },
            "stringData": { "type": "object" },
            "type": { "type": "string" }
        }
    });
    ("Secret".into(), schema)
}

fn ingress_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Ingress" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "properties": {
                    "rules": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "host": { "type": "string" },
                                "http": { "type": "object" }
                            }
                        }
                    },
                    "tls": { "type": "array" },
                    "ingressClassName": { "type": "string" },
                    "defaultBackend": { "type": "object" }
                }
            }
        }
    });
    ("Ingress".into(), schema)
}

fn statefulset_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "StatefulSet" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["selector", "template", "serviceName"],
                "properties": {
                    "replicas": { "type": "integer", "minimum": 0 },
                    "selector": label_selector_schema(),
                    "template": pod_template_schema(),
                    "serviceName": { "type": "string", "minLength": 1 },
                    "volumeClaimTemplates": { "type": "array" }
                }
            }
        }
    });
    ("StatefulSet".into(), schema)
}

fn daemonset_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "DaemonSet" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["selector", "template"],
                "properties": {
                    "selector": label_selector_schema(),
                    "template": pod_template_schema(),
                    "updateStrategy": { "type": "object" }
                }
            }
        }
    });
    ("DaemonSet".into(), schema)
}

fn job_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "Job" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["template"],
                "properties": {
                    "template": pod_template_schema(),
                    "backoffLimit": { "type": "integer" },
                    "completions": { "type": "integer" },
                    "parallelism": { "type": "integer" },
                    "activeDeadlineSeconds": { "type": "integer" },
                    "ttlSecondsAfterFinished": { "type": "integer" }
                }
            }
        }
    });
    ("Job".into(), schema)
}

fn cronjob_schema() -> (String, Value) {
    let schema = json!({
        "type": "object",
        "required": ["apiVersion", "kind", "metadata", "spec"],
        "properties": {
            "apiVersion": { "type": "string" },
            "kind": { "const": "CronJob" },
            "metadata": metadata_schema(),
            "spec": {
                "type": "object",
                "required": ["schedule", "jobTemplate"],
                "properties": {
                    "schedule": { "type": "string", "minLength": 1 },
                    "jobTemplate": {
                        "type": "object",
                        "required": ["spec"],
                        "properties": {
                            "spec": {
                                "type": "object",
                                "required": ["template"],
                                "properties": {
                                    "template": pod_template_schema()
                                }
                            }
                        }
                    },
                    "concurrencyPolicy": { "type": "string" },
                    "successfulJobsHistoryLimit": { "type": "integer" },
                    "failedJobsHistoryLimit": { "type": "integer" },
                    "startingDeadlineSeconds": { "type": "integer" },
                    "suspend": { "type": "boolean" }
                }
            }
        }
    });
    ("CronJob".into(), schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{SchemaRegistry, Validator};

    #[test]
    fn test_all_k8s_schemas_compile() {
        let schemas = build_all_schemas();
        assert_eq!(schemas.len(), 10);
        for (kind, schema) in &schemas {
            Validator::new(schema)
                .unwrap_or_else(|e| panic!("Schema for {} failed to compile: {}", kind, e));
        }
    }

    #[test]
    fn test_deployment_valid() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": { "name": "web" },
            "spec": {
                "selector": { "matchLabels": { "app": "web" } },
                "template": {
                    "spec": {
                        "containers": [{
                            "name": "web",
                            "image": "nginx:1.25"
                        }]
                    }
                }
            }
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(diags.is_empty(), "expected no errors: {:?}", diags);
    }

    #[test]
    fn test_deployment_missing_selector() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": { "name": "web" },
            "spec": {
                "template": {
                    "spec": {
                        "containers": [{ "name": "c", "image": "img:1" }]
                    }
                }
            }
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(!diags.is_empty(), "should report missing selector");
        assert!(diags.iter().any(|d| d.message.contains("selector")));
    }

    #[test]
    fn test_service_missing_ports() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": { "name": "svc" },
            "spec": {}
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(!diags.is_empty(), "should report missing ports");
    }

    #[test]
    fn test_pod_missing_containers() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name": "p" },
            "spec": {}
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(!diags.is_empty(), "should report missing containers");
    }

    #[test]
    fn test_cronjob_missing_schedule() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "batch/v1",
            "kind": "CronJob",
            "metadata": { "name": "cj" },
            "spec": {
                "jobTemplate": {
                    "spec": {
                        "template": {
                            "spec": {
                                "containers": [{ "name": "c", "image": "i:1" }]
                            }
                        }
                    }
                }
            }
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(!diags.is_empty(), "should report missing schedule");
        assert!(diags.iter().any(|d| d.message.contains("schedule")));
    }

    #[test]
    fn test_unknown_kind_no_errors() {
        let registry = SchemaRegistry::new("1.30");
        let doc = json!({
            "apiVersion": "custom.io/v1",
            "kind": "MyCustomResource",
            "metadata": { "name": "x" },
            "spec": {}
        });
        let diags = registry.validate_k8s(&doc, "1.30");
        assert!(diags.is_empty(), "unknown kind should produce no errors");
    }
}

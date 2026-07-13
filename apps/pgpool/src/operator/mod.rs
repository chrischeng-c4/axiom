// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// <HANDWRITE gap="missing-generator:logic:4a951ea7" tracker="#1575" reason="Export Pgpool CRD, render, reconcile, CRD YAML normalization, and operator deployment-manifest rendering.">
pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{
    Pgpool, PgpoolEndpointBudgetSpec, PgpoolEndpointBudgetStatus, PgpoolEndpointProvider,
    PgpoolEndpointRole, PgpoolPodBudgetStatus, PgpoolResources, PgpoolSpec, PgpoolStatus,
};
pub use reconcile::run;

use kube::CustomResourceExt;
use serde_json::{json, Value};

use crate::k8s::{spec_for_profile, InstanceProfile};

pub fn crd_yaml() -> String {
    let mut crd = serde_json::to_value(Pgpool::crd()).expect("CRD serializes");
    normalize_kubernetes_schema_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes to YAML")
}

pub fn instance_yaml(profile: InstanceProfile) -> String {
    let deployment = spec_for_profile(profile);
    let provider = if deployment.backend_host.contains("alloydb") {
        PgpoolEndpointProvider::AlloyDb
    } else if deployment.backend_host.contains("cloudsql") {
        PgpoolEndpointProvider::CloudSql
    } else {
        PgpoolEndpointProvider::PlainPostgres
    };
    let instance = Pgpool::new(
        &deployment.name,
        PgpoolSpec {
            image: deployment.image,
            replicas: deployment.replicas,
            primary_endpoint: "primary".into(),
            endpoints: vec![PgpoolEndpointBudgetSpec {
                name: "primary".into(),
                provider,
                role: PgpoolEndpointRole::Primary,
                host: deployment.backend_host,
                port: deployment.backend_port,
                reserve: deployment.max_backend_connections,
                safety_headroom: deployment.max_backend_connections / 2,
                configured_ceiling: None,
                per_pod_quota: deployment.max_backend_connections,
            }],
            resources: PgpoolResources {
                cpu: deployment.cpu,
                memory: deployment.memory,
            },
            termination_grace_period_seconds: deployment.termination_grace_period_seconds,
        },
    );
    let mut value = serde_json::to_value(instance).expect("instance serializes");
    value["metadata"]["namespace"] = json!(deployment.namespace);
    serde_yaml::to_string(&value).expect("instance serializes to YAML")
}

pub fn operator_yaml(namespace: &str) -> String {
    operator_manifests(namespace)
        .into_iter()
        .map(|manifest| serde_yaml::to_string(&manifest).expect("operator manifest serializes"))
        .collect::<Vec<_>>()
        .join("---\n")
}

pub fn operator_manifests(namespace: &str) -> Vec<Value> {
    let name = "pgpool-operator";
    vec![
        json!({
            "apiVersion": "v1", "kind": "Namespace",
            "metadata": { "name": namespace },
        }),
        json!({
            "apiVersion": "v1", "kind": "ServiceAccount",
            "metadata": { "name": name, "namespace": namespace },
        }),
        json!({
            "apiVersion": "rbac.authorization.k8s.io/v1", "kind": "ClusterRole",
            "metadata": { "name": name },
            "rules": [
                { "apiGroups": ["pgpool.axiom.dev"], "resources": ["pgpools", "pgpools/status"], "verbs": ["get", "list", "watch", "patch", "update"] },
                { "apiGroups": ["apps"], "resources": ["deployments"], "verbs": ["get", "list", "watch", "create", "patch", "update"] },
                { "apiGroups": [""], "resources": ["services", "serviceaccounts"], "verbs": ["get", "list", "watch", "create", "patch", "update"] },
                { "apiGroups": ["policy"], "resources": ["poddisruptionbudgets"], "verbs": ["get", "list", "watch", "create", "patch", "update"] },
                { "apiGroups": ["coordination.k8s.io"], "resources": ["leases"], "verbs": ["get", "create", "update", "patch"] },
            ],
        }),
        json!({
            "apiVersion": "rbac.authorization.k8s.io/v1", "kind": "ClusterRoleBinding",
            "metadata": { "name": name },
            "roleRef": { "apiGroup": "rbac.authorization.k8s.io", "kind": "ClusterRole", "name": name },
            "subjects": [{ "kind": "ServiceAccount", "name": name, "namespace": namespace }],
        }),
        json!({
            "apiVersion": "apps/v1", "kind": "Deployment",
            "metadata": { "name": name, "namespace": namespace, "labels": { "app.kubernetes.io/name": name } },
            "spec": {
                "replicas": 2,
                "selector": { "matchLabels": { "app.kubernetes.io/name": name } },
                "template": {
                    "metadata": { "labels": { "app.kubernetes.io/name": name } },
                    "spec": {
                        "serviceAccountName": name,
                        "securityContext": { "runAsNonRoot": true, "seccompProfile": { "type": "RuntimeDefault" } },
                        "containers": [{
                            "name": "operator", "image": "ghcr.io/chrischeng-c4/pgpool:latest",
                            "command": ["pgpool", "k8s", "operator", "run"],
                            "env": [
                                { "name": "POD_NAME", "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } },
                                { "name": "POD_NAMESPACE", "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } },
                            ],
                            "resources": { "requests": { "cpu": "100m", "memory": "128Mi" }, "limits": { "cpu": "500m", "memory": "256Mi" } },
                            "securityContext": { "allowPrivilegeEscalation": false, "readOnlyRootFilesystem": true, "capabilities": { "drop": ["ALL"] } },
                        }],
                    },
                },
            },
        }),
    ]
}

fn normalize_kubernetes_schema_formats(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if matches!(
                map.get("format").and_then(Value::as_str),
                Some("uint16" | "uint32" | "uint64")
            ) {
                map.remove("format");
                map.entry("minimum").or_insert_with(|| json!(0));
            }
            for child in map.values_mut() {
                normalize_kubernetes_schema_formats(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                normalize_kubernetes_schema_formats(child);
            }
        }
        _ => {}
    }
}
// </HANDWRITE>

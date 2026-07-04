// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-render-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::json;

use crate::discover::BaseWorkloadContract;
use crate::model::{
    BaseSpec, CleanupAction, CleanupPlan, GkeSpec, Label, PreviewEnvironment, PreviewMetadata,
    PreviewPhase, PreviewSpec, PreviewStatus, RouteSpec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderInput {
    pub mr: u32,
    pub sha: String,
    pub image: String,
    pub app: String,
    pub host: String,
    pub base_namespace: String,
    pub owner: String,
    pub ttl_hours: u32,
    pub control_namespace: String,
    pub workload_identity: String,
    pub base_contract: Option<BaseWorkloadContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFile {
    pub path: String,
    pub contents: String,
}

pub fn render_files(input: &RenderInput) -> Result<Vec<RenderFile>> {
    let env = preview_environment(input);
    let cleanup = cleanup_plan(&env, false);

    Ok(vec![
        RenderFile {
            path: "spec/preview-environment.yaml".to_string(),
            contents: serde_yaml::to_string(&env)?,
        },
        RenderFile {
            path: "plans/workload-clone.json".to_string(),
            contents: serde_json::to_string_pretty(&workload_clone_plan(
                &env,
                input.base_contract.as_ref(),
            ))? + "\n",
        },
        RenderFile {
            path: "k8s/namespace.yaml".to_string(),
            contents: yaml(&namespace(&env))?,
        },
        RenderFile {
            path: "k8s/service-account.yaml".to_string(),
            contents: yaml(&service_account(&env))?,
        },
        RenderFile {
            path: "k8s/resource-quota.yaml".to_string(),
            contents: yaml(&resource_quota(&env))?,
        },
        RenderFile {
            path: "k8s/limit-range.yaml".to_string(),
            contents: yaml(&limit_range(&env))?,
        },
        RenderFile {
            path: "k8s/workload-role.yaml".to_string(),
            contents: yaml(&workload_role(&env))?,
        },
        RenderFile {
            path: "k8s/workload-role-binding.yaml".to_string(),
            contents: yaml(&workload_role_binding(&env))?,
        },
        RenderFile {
            path: "k8s/deployment.yaml".to_string(),
            contents: yaml(&deployment(&env))?,
        },
        RenderFile {
            path: "k8s/service.yaml".to_string(),
            contents: yaml(&service(&env))?,
        },
        RenderFile {
            path: "router/route-binding.yaml".to_string(),
            contents: yaml(&route_binding(&env))?,
        },
        RenderFile {
            path: "mr-comment.md".to_string(),
            contents: mr_comment(&env),
        },
        RenderFile {
            path: "cleanup-plan.json".to_string(),
            contents: serde_json::to_string_pretty(&cleanup)? + "\n",
        },
    ])
}

pub fn preview_environment(input: &RenderInput) -> PreviewEnvironment {
    let namespace = format!("uat-mr-{}", input.mr);
    let target = format!("mr-{}", input.mr);
    let app = input.app.clone();

    PreviewEnvironment {
        api_version: "uat.cclab.dev/v1alpha1".to_string(),
        kind: "PreviewEnvironment".to_string(),
        metadata: PreviewMetadata {
            name: target.clone(),
            labels: labels(input.mr, &input.sha, &input.owner, &app),
        },
        spec: PreviewSpec {
            mr: input.mr,
            sha: input.sha.clone(),
            image: input.image.clone(),
            app: app.clone(),
            namespace,
            base: BaseSpec {
                namespace: input
                    .base_contract
                    .as_ref()
                    .map(|contract| contract.namespace.clone())
                    .unwrap_or_else(|| input.base_namespace.clone()),
                workload: input
                    .base_contract
                    .as_ref()
                    .map(|contract| contract.deployment.clone())
                    .unwrap_or_else(|| app.clone()),
                service: input
                    .base_contract
                    .as_ref()
                    .map(|contract| contract.service.clone())
                    .unwrap_or_else(|| app.clone()),
            },
            owner: input.owner.clone(),
            ttl_hours: input.ttl_hours,
            route: RouteSpec {
                host: input.host.clone(),
                target,
                cookie: "uat_target".to_string(),
                header: "X-UAT-Target".to_string(),
                service: app,
                service_port: 80,
            },
            gke: GkeSpec {
                control_namespace: input.control_namespace.clone(),
                workload_identity: input.workload_identity.clone(),
            },
        },
        status: PreviewStatus {
            phase: PreviewPhase::Pending,
            message: "rendered; not applied".to_string(),
        },
    }
}

pub fn cleanup_plan(env: &PreviewEnvironment, mr_closed: bool) -> CleanupPlan {
    let action = if mr_closed {
        CleanupAction::Delete
    } else if matches!(
        env.status.phase,
        PreviewPhase::Draining | PreviewPhase::Deleted
    ) {
        CleanupAction::Delete
    } else {
        CleanupAction::Keep
    };

    CleanupPlan {
        mr: env.spec.mr,
        namespace: env.spec.namespace.clone(),
        route_target: env.spec.route.target.clone(),
        protected_namespaces: vec![
            env.spec.base.namespace.clone(),
            env.spec.gke.control_namespace.clone(),
        ],
        action,
        reason: if action == CleanupAction::Delete {
            "MR is closed or preview is already draining/deleted".to_string()
        } else {
            "MR remains active; keep preview resources".to_string()
        },
        delete_namespace: action == CleanupAction::Delete,
        delete_route_binding: action == CleanupAction::Delete,
    }
}

pub fn mr_comment(env: &PreviewEnvironment) -> String {
    format!(
        r#"### UAT Preview

Status: `{}`
MR: `{}`
SHA: `{}`
Image: `{}`
Namespace: `{}`
Base namespace: `{}`
Base workload: `{}`
Route target: `{}`

Browser entry:
`https://{}/_preview/{}`

Manual/API routing:
`{}: {}`

Cleanup TTL: `{}` hours
"#,
        phase_name(env.status.phase),
        env.spec.mr,
        env.spec.sha,
        env.spec.image,
        env.spec.namespace,
        env.spec.base.namespace,
        env.spec.base.workload,
        env.spec.route.target,
        env.spec.route.host,
        env.spec.route.target,
        env.spec.route.header,
        env.spec.route.target,
        env.spec.ttl_hours,
    )
}

fn phase_name(phase: PreviewPhase) -> &'static str {
    match phase {
        PreviewPhase::Pending => "pending",
        PreviewPhase::Provisioning => "provisioning",
        PreviewPhase::Ready => "ready",
        PreviewPhase::Failed => "failed",
        PreviewPhase::Draining => "draining",
        PreviewPhase::Deleted => "deleted",
    }
}

fn labels(mr: u32, sha: &str, owner: &str, app: &str) -> Vec<Label> {
    vec![
        Label {
            key: "app.kubernetes.io/name".to_string(),
            value: app.to_string(),
        },
        Label {
            key: "preview.cclab.dev/mr".to_string(),
            value: mr.to_string(),
        },
        Label {
            key: "preview.cclab.dev/sha".to_string(),
            value: sha.to_string(),
        },
        Label {
            key: "preview.cclab.dev/owner".to_string(),
            value: owner.to_string(),
        },
    ]
}

fn label_map(env: &PreviewEnvironment) -> BTreeMap<String, String> {
    env.metadata
        .labels
        .iter()
        .map(|label| (label.key.clone(), label.value.clone()))
        .collect()
}

fn selector(env: &PreviewEnvironment) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app.kubernetes.io/name".to_string(), env.spec.app.clone()),
        (
            "preview.cclab.dev/target".to_string(),
            env.spec.route.target.clone(),
        ),
    ])
}

fn namespace(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": env.spec.namespace,
            "labels": label_map(env),
            "annotations": {
                "preview.cclab.dev/ttl-hours": env.spec.ttl_hours.to_string(),
                "preview.cclab.dev/route-target": env.spec.route.target,
                "preview.cclab.dev/base-namespace": env.spec.base.namespace,
                "preview.cclab.dev/source-workload": env.spec.base.workload,
            }
        }
    })
}

fn workload_clone_plan(
    env: &PreviewEnvironment,
    base_contract: Option<&BaseWorkloadContract>,
) -> serde_json::Value {
    json!({
        "source": {
            "namespace": env.spec.base.namespace,
            "workload": env.spec.base.workload,
            "service": env.spec.base.service
        },
        "target": {
            "namespace": env.spec.namespace,
            "workload": env.spec.app,
            "service": env.spec.route.service,
            "routeTarget": env.spec.route.target
        },
        "overrides": {
            "image": env.spec.image,
            "sha": env.spec.sha,
            "serviceAccount": env.spec.gke.workload_identity,
            "owner": env.spec.owner,
            "ttlHours": env.spec.ttl_hours
        },
        "discoveredBase": base_contract,
        "copyPolicy": {
            "include": [
                "pod template labels",
                "container ports",
                "container env contract",
                "probes",
                "resource requests and limits"
            ],
            "exclude": [
                "status",
                "uid",
                "resourceVersion",
                "ownerReferences",
                "clusterIP",
                "nodePort",
                "loadBalancer",
                "base namespace secrets by default"
            ]
        }
    })
}

fn service_account(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": {
            "name": env.spec.gke.workload_identity,
            "namespace": env.spec.namespace,
            "labels": label_map(env),
            "annotations": {
                "iam.gke.io/gcp-service-account": format!("{}@example.iam.gserviceaccount.com", env.spec.gke.workload_identity)
            }
        }
    })
}

fn resource_quota(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "ResourceQuota",
        "metadata": {
            "name": "preview-budget",
            "namespace": env.spec.namespace,
            "labels": label_map(env),
        },
        "spec": {
            "hard": {
                "pods": "3",
                "services": "2",
                "configmaps": "10",
                "requests.cpu": "500m",
                "requests.memory": "768Mi",
                "limits.cpu": "1",
                "limits.memory": "1536Mi"
            }
        }
    })
}

fn limit_range(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "LimitRange",
        "metadata": {
            "name": "preview-defaults",
            "namespace": env.spec.namespace,
            "labels": label_map(env),
        },
        "spec": {
            "limits": [{
                "type": "Container",
                "defaultRequest": {
                    "cpu": "100m",
                    "memory": "128Mi"
                },
                "default": {
                    "cpu": "250m",
                    "memory": "256Mi"
                },
                "max": {
                    "cpu": "500m",
                    "memory": "512Mi"
                }
            }]
        }
    })
}

fn workload_role(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "Role",
        "metadata": {
            "name": "preview-workload-read",
            "namespace": env.spec.namespace,
            "labels": label_map(env),
        },
        "rules": [{
            "apiGroups": [""],
            "resources": ["configmaps", "endpoints", "pods", "services"],
            "verbs": ["get", "list", "watch"]
        }]
    })
}

fn workload_role_binding(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "rbac.authorization.k8s.io/v1",
        "kind": "RoleBinding",
        "metadata": {
            "name": "preview-workload-read",
            "namespace": env.spec.namespace,
            "labels": label_map(env),
        },
        "subjects": [{
            "kind": "ServiceAccount",
            "name": env.spec.gke.workload_identity,
            "namespace": env.spec.namespace
        }],
        "roleRef": {
            "apiGroup": "rbac.authorization.k8s.io",
            "kind": "Role",
            "name": "preview-workload-read"
        }
    })
}

fn deployment(env: &PreviewEnvironment) -> serde_json::Value {
    let mut labels = selector(env);
    labels.extend(label_map(env));

    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
            "name": env.spec.app,
            "namespace": env.spec.namespace,
            "labels": labels,
            "annotations": {
                "preview.cclab.dev/base-namespace": env.spec.base.namespace,
                "preview.cclab.dev/source-workload": env.spec.base.workload
            }
        },
        "spec": {
            "replicas": 1,
            "selector": { "matchLabels": selector(env) },
            "template": {
                "metadata": { "labels": labels },
                "spec": {
                    "serviceAccountName": env.spec.gke.workload_identity,
                    "containers": [{
                        "name": env.spec.app,
                        "image": env.spec.image,
                        "ports": [{ "containerPort": 8080 }],
                        "env": [
                            { "name": "PREVIEW_MR", "value": env.spec.mr.to_string() },
                            { "name": "PREVIEW_SHA", "value": env.spec.sha },
                            { "name": "UAT_ROUTE_TARGET", "value": env.spec.route.target }
                        ],
                        "resources": {
                            "requests": {
                                "cpu": "100m",
                                "memory": "128Mi"
                            },
                            "limits": {
                                "cpu": "250m",
                                "memory": "256Mi"
                            }
                        },
                        "readinessProbe": {
                            "httpGet": { "path": "/readyz", "port": 8080 },
                            "initialDelaySeconds": 5,
                            "periodSeconds": 5
                        },
                        "livenessProbe": {
                            "httpGet": { "path": "/healthz", "port": 8080 },
                            "initialDelaySeconds": 10,
                            "periodSeconds": 10
                        }
                    }]
                }
            }
        }
    })
}

fn service(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": {
            "name": env.spec.route.service,
            "namespace": env.spec.namespace,
            "labels": label_map(env),
            "annotations": {
                "preview.cclab.dev/base-namespace": env.spec.base.namespace,
                "preview.cclab.dev/source-service": env.spec.base.service
            }
        },
        "spec": {
            "type": "ClusterIP",
            "selector": selector(env),
            "ports": [{
                "name": "http",
                "port": env.spec.route.service_port,
                "targetPort": 8080
            }]
        }
    })
}

fn route_binding(env: &PreviewEnvironment) -> serde_json::Value {
    json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": format!("routebinding-{}", env.spec.route.target),
            "namespace": env.spec.gke.control_namespace,
            "labels": {
                "preview.cclab.dev/kind": "route-binding",
                "preview.cclab.dev/target": env.spec.route.target,
                "preview.cclab.dev/mr": env.spec.mr.to_string()
            }
        },
        "data": {
            "target": env.spec.route.target,
            "host": env.spec.route.host,
            "cookie": env.spec.route.cookie,
            "header": env.spec.route.header,
            "namespace": env.spec.namespace,
            "baseNamespace": env.spec.base.namespace,
            "sourceWorkload": env.spec.base.workload,
            "service": env.spec.route.service,
            "servicePort": env.spec.route.service_port.to_string(),
            "sha": env.spec.sha
        }
    })
}

fn yaml(value: &serde_json::Value) -> Result<String> {
    Ok(serde_yaml::to_string(value)?)
}

// </HANDWRITE>

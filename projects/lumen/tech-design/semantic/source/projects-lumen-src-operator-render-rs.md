---
id: projects-lumen-src-operator-render-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
  - id: "long-running-stability"
    role: primary
    gap: "lumen-crd-reconcile-loop-kube-rs-operator"
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: full
    rationale: "render.rs is the pure operator child-object renderer covered by the operator_render EC."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/render.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/render.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `broker_url` | projects/lumen/src/operator/render.rs | function | pub | 46 | broker_url(lumen: &Lumen) -> String |
| `render` | projects/lumen/src/operator/render.rs | function | pub | 103 | render(lumen: &Lumen) -> Vec<Value> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Pure rendering: a [`Lumen`] spec → the set of child Kubernetes objects that
//! realize it. No cluster, no I/O — every object is a self-contained
//! `serde_json::Value` carrying `apiVersion`, `kind`, full `metadata` (labels +
//! owner reference), and `spec`/`data`. This is the operator's source of truth
//! and its primary test surface: assert the rendered objects, no kind needed.
//!
//! The objects mirror `k8s/base` + the staging/prod overlays exactly: serving
//! Deployment/Service/ConfigMap/HPA/PDB/ServiceAccount and (when the broker is
//! managed) Relay StatefulSet/Services/PDB. The reconcile loop in
//! [`super::reconcile`] server-side-applies whatever this returns.

use serde_json::{json, Value};

use super::crd::Lumen;
use operator::render::RenderCtx;

const APP: &str = "lumen";
const API_VERSION: &str = "lumen.dev/v1alpha1";
const KIND: &str = "Lumen";
const CLIENT_PORT: i32 = 7373;
const BROKER_PORT: i32 = 7000;

/// Resolve the instance name (defaults to `lumen` only when metadata is absent,
/// which never happens for a real CR).
fn instance(lumen: &Lumen) -> String {
    lumen
        .metadata
        .name
        .clone()
        .unwrap_or_else(|| APP.to_string())
}

/// Resolve the namespace (defaults to `default` for unit construction).
fn namespace(lumen: &Lumen) -> String {
    lumen
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

/// The Relay client URL serving pods connect to: the managed broker's ClusterIP
/// service, or the caller-supplied external URL.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md#source
pub fn broker_url(lumen: &Lumen) -> String {
    match &lumen.spec.broker.external_url {
        Some(url) => url.clone(),
        None => format!("http://{}-relay:{BROKER_PORT}", instance(lumen)),
    }
}

/// lumen's render identity for the shared [`operator::render`] helpers.
fn ctx(name: &str) -> RenderCtx<'_> {
    RenderCtx {
        app: APP,
        manager: "lumen-operator",
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns: "",
        owner: None,
    }
}

/// Recommended labels common to every child object (via the shared toolkit).
fn labels(name: &str, component: &str) -> Value {
    ctx(name).labels(component)
}

/// Minimal, immutable selector labels (a subset of [`labels`]). Workload and
/// Service selectors are pinned to these so re-applies never hit a
/// selector-immutability error.
fn selector(name: &str, component: &str) -> Value {
    ctx(name).selector(component)
}

/// The owner reference that ties a child to its `Lumen` CR, enabling
/// cascading garbage collection. Omitted when the CR has no `uid` (only in
/// unit construction); a live reconcile always has one.
fn owner_ref(lumen: &Lumen) -> Option<Value> {
    let uid = lumen.metadata.uid.clone()?;
    let name = lumen.metadata.name.clone()?;
    Some(operator::render::owner_ref(API_VERSION, KIND, &name, &uid))
}

/// Assemble an object's `metadata` block.
fn meta(name: &str, ns: &str, labels: Value, owner: &Option<Value>) -> Value {
    let mut m = json!({ "name": name, "namespace": ns, "labels": labels });
    if let Some(o) = owner {
        m["ownerReferences"] = json!([o]);
    }
    m
}

/// Render every child object for `lumen`, in dependency order (namespace-scoped
/// config first, then workloads).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md#source
pub fn render(lumen: &Lumen) -> Vec<Value> {
    let mut out = vec![service_account(lumen), serving_configmap(lumen)];
    if lumen.spec.broker.is_managed() {
        out.push(broker_statefulset(lumen));
        out.push(broker_service(lumen));
        out.push(broker_headless_service(lumen));
        out.push(broker_pdb(lumen));
    }
    out.push(serving_deployment(lumen));
    out.push(serving_service(lumen));
    out.push(serving_hpa(lumen));
    out.push(serving_pdb(lumen));
    if lumen.spec.observability {
        out.push(service_monitor(lumen));
        out.push(prometheus_rule(lumen));
    }
    out
}

fn service_account(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
    })
}

fn serving_configmap(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    let mut data = json!({
        "SHARD_COUNT": lumen.spec.shard_count.to_string(),
        "LUMEN_RELAY_URL": broker_url(lumen),
        "LUMEN_RELAY_SUBJECT": lumen.spec.broker.subject.clone(),
        "LUMEN_LOG_FORMAT": lumen.spec.log_format.as_env(),
        "LUMEN_PORT": CLIENT_PORT.to_string(),
        "LUMEN_AUTH": lumen.spec.auth.as_env(),
    });
    if let Some(level) = &lumen.spec.log_level {
        data["LUMEN_LOG_LEVEL"] = json!(level);
    }
    json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": meta(&format!("{name}-config"), &ns, labels(&name, "server"), &owner),
        "data": data,
    })
}

/// Container env for the serving pod: downward-API identity + literal runtime
/// knobs + the config-driven values (so a ConfigMap edit can roll pods).
fn serving_env(lumen: &Lumen) -> Vec<Value> {
    let cfg = format!("{}-config", instance(lumen));
    let from_cfg = |key: &str| json!({ "name": key, "valueFrom": { "configMapKeyRef": { "name": cfg, "key": key } } });
    let mut env = vec![
        json!({ "name": "POD_NAME", "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": "POD_NAMESPACE", "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } }),
        json!({ "name": "LUMEN_HOST", "value": "0.0.0.0" }),
        json!({ "name": "LUMEN_WAL", "value": "relay" }),
        json!({ "name": "LUMEN_RELAY_SUBSCRIBER_ID", "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": "LUMEN_GRACE_SECS", "value": lumen.spec.serving.grace_secs.to_string() }),
        from_cfg("LUMEN_PORT"),
        from_cfg("LUMEN_RELAY_URL"),
        from_cfg("LUMEN_RELAY_SUBJECT"),
        from_cfg("LUMEN_LOG_FORMAT"),
        from_cfg("LUMEN_AUTH"),
        from_cfg("SHARD_COUNT"),
    ];
    if lumen.spec.log_level.is_some() {
        env.push(from_cfg("LUMEN_LOG_LEVEL"));
    }
    // Strict auth: pull the bearer tokens from a Secret out-of-band.
    if matches!(lumen.spec.auth, super::crd::AuthMode::Required) {
        if let Some(secret) = &lumen.spec.tokens_secret {
            env.push(json!({
                "name": "LUMEN_TOKENS",
                "valueFrom": { "secretKeyRef": { "name": secret, "key": "LUMEN_TOKENS" } }
            }));
        }
    }
    env
}

fn serving_deployment(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    let s = &lumen.spec.serving;
    let res = json!({
        "requests": { "cpu": s.cpu, "memory": s.memory },
        "limits": { "cpu": s.cpu, "memory": s.memory },
    });
    let spread = |key: &str| {
        json!({
            "maxSkew": 1,
            "topologyKey": key,
            "whenUnsatisfiable": "ScheduleAnyway",
            "labelSelector": { "matchLabels": selector(&name, "server") },
        })
    };
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        "spec": {
            // HPA owns the live count; this is the floor at apply time.
            "replicas": s.autoscaling.min_replicas,
            "revisionHistoryLimit": 5,
            "selector": { "matchLabels": selector(&name, "server") },
            "strategy": {
                "type": "RollingUpdate",
                // Keep read capacity during rollouts; new pod must reach /readyz
                // (log tail caught up) before an old one is torn down.
                "rollingUpdate": { "maxUnavailable": 0, "maxSurge": 1 },
            },
            "template": {
                "metadata": {
                    "labels": labels(&name, "server"),
                    "annotations": {
                        "prometheus.io/scrape": "true",
                        "prometheus.io/port": CLIENT_PORT.to_string(),
                        "prometheus.io/path": "/metrics",
                    },
                },
                "spec": {
                    "serviceAccountName": name,
                    "terminationGracePeriodSeconds": s.grace_secs,
                    "securityContext": {
                        "runAsNonRoot": true,
                        "runAsUser": 65532, "runAsGroup": 65532, "fsGroup": 65532,
                        "seccompProfile": { "type": "RuntimeDefault" },
                    },
                    "topologySpreadConstraints": [
                        spread("topology.kubernetes.io/zone"),
                        spread("kubernetes.io/hostname"),
                    ],
                    "containers": [{
                        "name": "lumen",
                        "image": lumen.spec.image,
                        "imagePullPolicy": lumen.spec.image_pull_policy.clone().unwrap_or_else(|| "IfNotPresent".into()),
                        "command": ["lumen", "serve"],
                        "ports": [{ "name": "http", "containerPort": CLIENT_PORT, "protocol": "TCP" }],
                        "env": serving_env(lumen),
                        "resources": res,
                        // 503 until the log tail catches up; generous threshold
                        // lets a cold pod rebuild from the broker log.
                        "readinessProbe": {
                            "httpGet": { "path": "/readyz", "port": "http" },
                            "initialDelaySeconds": 5, "periodSeconds": 10,
                            "timeoutSeconds": 3, "failureThreshold": 60,
                        },
                        "livenessProbe": {
                            "httpGet": { "path": "/healthz", "port": "http" },
                            "initialDelaySeconds": 15, "periodSeconds": 30,
                            "timeoutSeconds": 5, "failureThreshold": 3,
                        },
                        "startupProbe": {
                            "httpGet": { "path": "/healthz", "port": "http" },
                            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
                        },
                        "securityContext": {
                            "runAsNonRoot": true, "runAsUser": 65532, "runAsGroup": 65532,
                            "allowPrivilegeEscalation": false,
                            "readOnlyRootFilesystem": true,
                            "capabilities": { "drop": ["ALL"] },
                        },
                        "volumeMounts": [{ "name": "tmp", "mountPath": "/tmp" }],
                    }],
                    "volumes": [{ "name": "tmp", "emptyDir": {} }],
                },
            },
        },
    })
}

fn serving_service(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        "spec": {
            "type": "ClusterIP",
            "selector": selector(&name, "server"),
            "ports": [{ "name": "http", "port": CLIENT_PORT, "targetPort": "http", "protocol": "TCP" }],
        },
    })
}

fn serving_hpa(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    let a = &lumen.spec.serving.autoscaling;
    json!({
        "apiVersion": "autoscaling/v2",
        "kind": "HorizontalPodAutoscaler",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        "spec": {
            "scaleTargetRef": { "apiVersion": "apps/v1", "kind": "Deployment", "name": name },
            "minReplicas": a.min_replicas,
            "maxReplicas": a.max_replicas,
            "metrics": [{
                "type": "Resource",
                "resource": { "name": "cpu", "target": { "type": "Utilization", "averageUtilization": a.target_cpu_utilization } },
            }],
            "behavior": {
                // React fast to read spikes; scale down slowly so new pods'
                // index-rebuild warm-up cost isn't thrashed.
                "scaleUp": {
                    "stabilizationWindowSeconds": 30,
                    "policies": [{ "type": "Percent", "value": 100, "periodSeconds": 30 }],
                },
                "scaleDown": {
                    "stabilizationWindowSeconds": 300,
                    "policies": [{ "type": "Pods", "value": 1, "periodSeconds": 60 }],
                },
            },
        },
    })
}

fn serving_pdb(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "policy/v1",
        "kind": "PodDisruptionBudget",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        // Comfort guard on stateless cattle: keep read capacity during drains
        // without ever blocking a single-replica dev deployment.
        "spec": { "maxUnavailable": 1, "selector": { "matchLabels": selector(&name, "server") } },
    })
}

// ---- Relay broker (managed only) ------------------------------------------

fn broker_statefulset(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    let b = &lumen.spec.broker;
    let mut pvc_spec = json!({
        "accessModes": ["ReadWriteOnce"],
        "resources": { "requests": { "storage": b.storage.clone() } },
    });
    if let Some(sc) = &b.storage_class {
        pvc_spec["storageClassName"] = json!(sc);
    }
    json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": meta(&format!("{name}-relay"), &ns, labels(&name, "broker"), &owner),
        "spec": {
            // relay-server is a single durable log. Use externalUrl for HA
            // Relay until relay-raft exposes subscribe/len, otherwise multiple
            // pods would be independent logs behind one Service.
            "replicas": 1,
            "serviceName": format!("{name}-relay-headless"),
            "podManagementPolicy": "Parallel",
            "selector": { "matchLabels": selector(&name, "broker") },
            "template": {
                "metadata": {
                    "labels": labels(&name, "broker"),
                    "annotations": {
                        "prometheus.io/scrape": "true",
                        "prometheus.io/port": BROKER_PORT.to_string(),
                        "prometheus.io/path": "/healthz",
                    },
                },
                "spec": {
                    "serviceAccountName": name,
                    "terminationGracePeriodSeconds": 30,
                    "securityContext": {
                        "runAsNonRoot": true, "runAsUser": 10001, "runAsGroup": 10001, "fsGroup": 10001,
                        "seccompProfile": { "type": "RuntimeDefault" },
                    },
                    "containers": [{
                        "name": "relay",
                        "image": b.image.clone(),
                        "imagePullPolicy": "IfNotPresent",
                        "command": ["relay-server"],
                        "ports": [{ "name": "http", "containerPort": BROKER_PORT, "protocol": "TCP" }],
                        "env": [
                            { "name": "RELAY_BIND", "value": format!("0.0.0.0:{BROKER_PORT}") },
                            { "name": "RELAY_DATA_DIR", "value": "/data" },
                        ],
                        "resources": {
                            "requests": { "cpu": b.cpu.clone(), "memory": b.memory.clone() },
                            "limits": { "cpu": b.cpu.clone(), "memory": b.memory.clone() },
                        },
                        "readinessProbe": {
                            "httpGet": { "path": "/healthz", "port": "http" },
                            "initialDelaySeconds": 5, "periodSeconds": 10, "timeoutSeconds": 3, "failureThreshold": 6,
                        },
                        "livenessProbe": {
                            "httpGet": { "path": "/healthz", "port": "http" },
                            "initialDelaySeconds": 15, "periodSeconds": 30, "timeoutSeconds": 5, "failureThreshold": 3,
                        },
                        "startupProbe": {
                            "httpGet": { "path": "/healthz", "port": "http" },
                            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 30,
                        },
                        "securityContext": {
                            "runAsNonRoot": true, "runAsUser": 10001, "runAsGroup": 10001,
                            "allowPrivilegeEscalation": false, "readOnlyRootFilesystem": true,
                            "capabilities": { "drop": ["ALL"] },
                        },
                        "volumeMounts": [{ "name": "data", "mountPath": "/data" }],
                    }],
                },
            },
            "volumeClaimTemplates": [{
                "metadata": { "name": "data", "labels": labels(&name, "broker") },
                "spec": pvc_spec,
            }],
        },
    })
}

fn broker_service(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": meta(&format!("{name}-relay"), &ns, labels(&name, "broker"), &owner),
        "spec": {
            "type": "ClusterIP",
            "selector": selector(&name, "broker"),
            "ports": [{ "name": "http", "port": BROKER_PORT, "targetPort": "http", "protocol": "TCP" }],
        },
    })
}

fn broker_headless_service(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": meta(&format!("{name}-relay-headless"), &ns, labels(&name, "broker"), &owner),
        "spec": {
            "clusterIP": "None",
            "publishNotReadyAddresses": true,
            "selector": selector(&name, "broker"),
            "ports": [{ "name": "http", "port": BROKER_PORT, "targetPort": "http", "protocol": "TCP" }],
        },
    })
}

fn broker_pdb(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "policy/v1",
        "kind": "PodDisruptionBudget",
        "metadata": meta(&format!("{name}-relay"), &ns, labels(&name, "broker"), &owner),
        "spec": { "maxUnavailable": 1, "selector": { "matchLabels": selector(&name, "broker") } },
    })
}

// ---- Observability (optional) ---------------------------------------------

fn service_monitor(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "ServiceMonitor",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        "spec": {
            "selector": { "matchLabels": selector(&name, "server") },
            "endpoints": [{ "port": "http", "path": "/metrics", "interval": "30s" }],
        },
    })
}

fn prometheus_rule(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "PrometheusRule",
        "metadata": meta(&name, &ns, labels(&name, "server"), &owner),
        "spec": {
            "groups": [{
                "name": "lumen.slo",
                "rules": [{
                    "alert": "LumenNoReadyServingPods",
                    "expr": format!("kube_deployment_status_replicas_available{{deployment=\"{name}\"}} == 0"),
                    "for": "2m",
                    "labels": { "severity": "critical" },
                    "annotations": { "summary": "No ready lumen serving pods for {{ $labels.deployment }}" },
                }],
            }],
        },
    })
}
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/render.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/operator/render.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

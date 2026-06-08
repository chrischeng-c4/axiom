//! Pure rendering: a [`Lumen`] spec → the set of child Kubernetes objects that
//! realize it. No cluster, no I/O — every object is a self-contained
//! `serde_json::Value` carrying `apiVersion`, `kind`, full `metadata` (labels +
//! owner reference), and `spec`/`data`. This is the operator's source of truth
//! and its primary test surface: assert the rendered objects, no kind needed.
//!
//! The objects mirror `k8s/base` + the staging/prod overlays exactly: serving
//! Deployment/Service/ConfigMap/HPA/PDB/ServiceAccount and (when the broker is
//! managed) NATS StatefulSet/Services/ConfigMap. The reconcile loop in
//! [`super::reconcile`] server-side-applies whatever this returns.

use serde_json::{json, Value};

use super::crd::Lumen;

const APP: &str = "lumen";
const API_VERSION: &str = "lumen.dev/v1alpha1";
const KIND: &str = "Lumen";
const CLIENT_PORT: i32 = 7373;

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

/// The NATS client URL serving pods connect to: the managed broker's ClusterIP
/// service, or the caller-supplied external URL.
pub fn nats_url(lumen: &Lumen) -> String {
    match &lumen.spec.nats.external_url {
        Some(url) => url.clone(),
        None => format!("nats://{}-nats:4222", instance(lumen)),
    }
}

/// Recommended labels common to every child object.
fn labels(name: &str, component: &str) -> Value {
    json!({
        "app.kubernetes.io/name": APP,
        "app.kubernetes.io/instance": name,
        "app.kubernetes.io/component": component,
        "app.kubernetes.io/managed-by": "lumen-operator",
        "app.kubernetes.io/part-of": APP,
    })
}

/// Minimal, immutable selector labels (a subset of [`labels`]). Workload and
/// Service selectors are pinned to these so re-applies never hit a
/// selector-immutability error.
fn selector(name: &str, component: &str) -> Value {
    json!({
        "app.kubernetes.io/name": APP,
        "app.kubernetes.io/instance": name,
        "app.kubernetes.io/component": component,
    })
}

/// The owner reference that ties a child to its `Lumen` CR, enabling
/// cascading garbage collection. Omitted when the CR has no `uid` (only in
/// unit construction); a live reconcile always has one.
fn owner_ref(lumen: &Lumen) -> Option<Value> {
    let uid = lumen.metadata.uid.clone()?;
    let name = lumen.metadata.name.clone()?;
    Some(json!({
        "apiVersion": API_VERSION,
        "kind": KIND,
        "name": name,
        "uid": uid,
        "controller": true,
        "blockOwnerDeletion": true,
    }))
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
pub fn render(lumen: &Lumen) -> Vec<Value> {
    let mut out = vec![service_account(lumen), serving_configmap(lumen)];
    if lumen.spec.nats.is_managed() {
        out.push(nats_configmap(lumen));
        out.push(nats_statefulset(lumen));
        out.push(nats_service(lumen));
        out.push(nats_headless_service(lumen));
        out.push(nats_pdb(lumen));
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
        "LUMEN_NATS_URL": nats_url(lumen),
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
        json!({ "name": "LUMEN_WAL", "value": "nats" }),
        json!({ "name": "LUMEN_GRACE_SECS", "value": lumen.spec.serving.grace_secs.to_string() }),
        from_cfg("LUMEN_PORT"),
        from_cfg("LUMEN_NATS_URL"),
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
                        // lets a cold pod rebuild from the NATS log.
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

// ---- NATS broker (managed only) -------------------------------------------

fn nats_configmap(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": meta(&format!("{name}-nats-config"), &ns, labels(&name, "nats"), &owner),
        // A write (incl. a bulk index batch) is one JetStream message; lift the
        // 1MB default. The async-nats client learns the ceiling from INFO.
        "data": { "nats.conf": "max_payload: 8MB\n" },
    })
}

fn nats_args(lumen: &Lumen) -> Vec<Value> {
    let name = instance(lumen);
    let replicas = lumen.spec.nats.replicas.max(1);
    let mut args: Vec<Value> = [
        "-c",
        "/etc/nats/nats.conf",
        "-js",
        "-sd",
        "/data",
        "-m",
        "8222",
    ]
    .iter()
    .map(|s| json!(s))
    .collect();
    // Clustered JetStream: wire routes so the brokers form one RAFT meta-group.
    if replicas > 1 {
        let routes = (0..replicas)
            .map(|i| format!("nats://{name}-nats-{i}.{name}-nats-headless:6222"))
            .collect::<Vec<_>>()
            .join(",");
        for a in [
            "--cluster_name",
            APP,
            "--cluster",
            "nats://0.0.0.0:6222",
            "--routes",
            &routes,
        ] {
            args.push(json!(a));
        }
    }
    args
}

fn nats_statefulset(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    let n = &lumen.spec.nats;
    let mut pvc_spec = json!({
        "accessModes": ["ReadWriteOnce"],
        "resources": { "requests": { "storage": n.storage } },
    });
    if let Some(sc) = &n.storage_class {
        pvc_spec["storageClassName"] = json!(sc);
    }
    json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": meta(&format!("{name}-nats"), &ns, labels(&name, "nats"), &owner),
        "spec": {
            "replicas": n.replicas,
            "serviceName": format!("{name}-nats-headless"),
            "podManagementPolicy": "Parallel",
            "selector": { "matchLabels": selector(&name, "nats") },
            "template": {
                "metadata": {
                    "labels": labels(&name, "nats"),
                    "annotations": {
                        "prometheus.io/scrape": "true",
                        "prometheus.io/port": "8222",
                        "prometheus.io/path": "/metrics",
                    },
                },
                "spec": {
                    "serviceAccountName": name,
                    "terminationGracePeriodSeconds": 60,
                    "securityContext": {
                        "runAsNonRoot": true, "runAsUser": 1000, "runAsGroup": 1000, "fsGroup": 1000,
                        "seccompProfile": { "type": "RuntimeDefault" },
                    },
                    "containers": [{
                        "name": "nats",
                        "image": "nats:2.10-alpine",
                        "imagePullPolicy": "IfNotPresent",
                        "args": nats_args(lumen),
                        "ports": [
                            { "name": "client", "containerPort": 4222, "protocol": "TCP" },
                            { "name": "cluster", "containerPort": 6222, "protocol": "TCP" },
                            { "name": "monitor", "containerPort": 8222, "protocol": "TCP" },
                        ],
                        "resources": {
                            "requests": { "cpu": n.cpu, "memory": n.memory },
                            "limits": { "cpu": n.cpu, "memory": n.memory },
                        },
                        "readinessProbe": {
                            "httpGet": { "path": "/healthz", "port": "monitor" },
                            "initialDelaySeconds": 5, "periodSeconds": 10, "timeoutSeconds": 3, "failureThreshold": 6,
                        },
                        "livenessProbe": {
                            "httpGet": { "path": "/healthz", "port": "monitor" },
                            "initialDelaySeconds": 15, "periodSeconds": 30, "timeoutSeconds": 5, "failureThreshold": 3,
                        },
                        "startupProbe": {
                            "httpGet": { "path": "/healthz", "port": "monitor" },
                            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 30,
                        },
                        "securityContext": {
                            "runAsNonRoot": true, "runAsUser": 1000, "runAsGroup": 1000,
                            "allowPrivilegeEscalation": false, "readOnlyRootFilesystem": true,
                            "capabilities": { "drop": ["ALL"] },
                        },
                        "volumeMounts": [
                            { "name": "data", "mountPath": "/data" },
                            { "name": "nats-config", "mountPath": "/etc/nats", "readOnly": true },
                        ],
                    }],
                    "volumes": [{ "name": "nats-config", "configMap": { "name": format!("{name}-nats-config") } }],
                },
            },
            "volumeClaimTemplates": [{
                "metadata": { "name": "data", "labels": labels(&name, "nats") },
                "spec": pvc_spec,
            }],
        },
    })
}

fn nats_service(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": meta(&format!("{name}-nats"), &ns, labels(&name, "nats"), &owner),
        "spec": {
            "type": "ClusterIP",
            "selector": selector(&name, "nats"),
            "ports": [
                { "name": "client", "port": 4222, "targetPort": "client", "protocol": "TCP" },
                { "name": "monitor", "port": 8222, "targetPort": "monitor", "protocol": "TCP" },
            ],
        },
    })
}

fn nats_headless_service(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": meta(&format!("{name}-nats-headless"), &ns, labels(&name, "nats"), &owner),
        "spec": {
            "clusterIP": "None",
            "publishNotReadyAddresses": true,
            "selector": selector(&name, "nats"),
            "ports": [
                { "name": "client", "port": 4222, "targetPort": "client", "protocol": "TCP" },
                { "name": "cluster", "port": 6222, "targetPort": "cluster", "protocol": "TCP" },
                { "name": "monitor", "port": 8222, "targetPort": "monitor", "protocol": "TCP" },
            ],
        },
    })
}

fn nats_pdb(lumen: &Lumen) -> Value {
    let (name, ns, owner) = (instance(lumen), namespace(lumen), owner_ref(lumen));
    json!({
        "apiVersion": "policy/v1",
        "kind": "PodDisruptionBudget",
        "metadata": meta(&format!("{name}-nats"), &ns, labels(&name, "nats"), &owner),
        // Never let a voluntary disruption drop the JetStream quorum.
        "spec": { "maxUnavailable": 1, "selector": { "matchLabels": selector(&name, "nats") } },
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

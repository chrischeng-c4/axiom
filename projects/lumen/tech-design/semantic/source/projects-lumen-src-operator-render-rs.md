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
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "operator-owned-storage-topology-and-reshard-status"
    claim: "operator-owned-storage-topology-and-reshard-status"
    coverage: full
    rationale: "The pure renderer owns StatefulSet storage topology and reshard status shape for the Kubernetes-native deployment path."
  - id: "dynamic-shard-topology"
    role: primary
    gap: "storage-pressure-operator-split-policy"
    claim: "storage-pressure-operator-split-policy"
    coverage: full
    rationale: "The renderer exposes storage pressure and reshard recommendations without changing HPA-owned serving scale."
  - id: "dynamic-shard-topology"
    role: primary
    gap: "multi-shard-replica-kind-e2e"
    claim: "multi-shard-replica-kind-e2e"
    coverage: partial
    rationale: "The operator render shape is the static proof behind the live kind multi-shard and replicated-shard profiles."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/render.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/render.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `render` | projects/lumen/src/operator/render.rs | function | pub | 112 | render(lumen: &Lumen) -> Vec<Value> |
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
//! The objects mirror `k8s/base` + the staging/prod overlays exactly: a
//! serving StatefulSet (always — its `volumeClaimTemplates`-backed `raft` PVC
//! is the WAL's only durable home, even at `replicasPerShard:1`), its
//! headless Service, a ClusterIP Service, ConfigMap, HPA when applicable,
//! PDB, and ServiceAccount. The reconcile loop in [`super::reconcile`]
//! server-side-applies whatever this returns.

use serde_json::{json, Value};

use super::crd::Lumen;
use operator::render::{
    self, HorizontalPodAutoscaler, RenderCtx, ServiceStatefulSet, WorkloadVolumeClaim,
};

const APP: &str = "lumen";
const MANAGER: &str = "lumen-operator";
const API_VERSION: &str = "lumen.dev/v1alpha1";
const KIND: &str = "Lumen";
const COMPONENT: &str = "server";
const CLIENT_PORT: i32 = 7373;
const BACKUP_COMPONENT: &str = "backup";
const HEADLESS_ENV_KEY: &str = "LUMEN_HEADLESS_SERVICE";
const TOKEN_REGISTRY_VOLUME: &str = "lumen-token-registry";
const TOKEN_REGISTRY_KEY: &str = "token-registry.json";
const TOKEN_REGISTRY_MOUNT_DIR: &str = "/var/run/secrets/lumen";
const TOKEN_REGISTRY_FILE: &str = "/var/run/secrets/lumen/token-registry.json";

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

/// lumen's render identity for the shared [`operator::render`] helpers.
fn ctx<'a>(lumen: &Lumen, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    RenderCtx {
        app: APP,
        manager: MANAGER,
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns,
        owner: owner_ref(lumen),
    }
}

/// The owner reference that ties a child to its `Lumen` CR, enabling
/// cascading garbage collection. Omitted when the CR has no `uid` (only in
/// unit construction); a live reconcile always has one.
fn owner_ref(lumen: &Lumen) -> Option<Value> {
    let uid = lumen.metadata.uid.clone()?;
    let name = lumen.metadata.name.clone()?;
    Some(render::owner_ref(API_VERSION, KIND, &name, &uid))
}

/// Which source (if any) supplies the token registry file. `tokensSecret`
/// wins over `tokensSecretProviderClass` when both are set (backward
/// compatible; documented as precedence, not schema-enforced mutual
/// exclusion). `None` when `auth: off` or neither is set.
enum TokenRegistrySource<'a> {
    Secret(&'a str),
    Csi(&'a str),
}

fn token_registry_source(lumen: &Lumen) -> Option<TokenRegistrySource<'_>> {
    if !matches!(lumen.spec.auth, super::crd::AuthMode::Required) {
        return None;
    }
    if let Some(secret) = lumen.spec.tokens_secret.as_deref() {
        return Some(TokenRegistrySource::Secret(secret));
    }
    lumen
        .spec
        .tokens_secret_provider_class
        .as_deref()
        .map(TokenRegistrySource::Csi)
}

/// Render every child object for `lumen`, in dependency order (namespace-scoped
/// config first, then workloads).
///
/// The serving fleet is always a StatefulSet — with its durable
/// `volumeClaimTemplates`-backed `raft` PVC and headless Service — regardless
/// of `replicasPerShard`. `replicasPerShard <= 1` (single shard) means a
/// single member with no raft consensus: an HPA is still rendered, but
/// (#1317) clamped to exactly 1 replica — CPU-driven scaling above 1 pod
/// here would produce uncoordinated shard-0 copies with no consensus link,
/// confirmed on a kind cluster. `> 1` means raft-HA with a fixed peer set
/// (no HPA).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md#source
pub fn render(lumen: &Lumen) -> Vec<Value> {
    let name = instance(lumen);
    let ns = namespace(lumen);
    let cx = ctx(lumen, &name, &ns);
    let headless = format!("{name}-headless");
    let mut out = vec![
        render::service_account(&cx, COMPONENT),
        serving_configmap(lumen, &cx),
        serving_statefulset(lumen, &cx, &headless),
        render::headless_service(&cx, &headless, COMPONENT, CLIENT_PORT),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
    ];
    if lumen.spec.replicas_per_shard <= 1 && lumen.spec.shard_count <= 1 {
        // Single shard, no raft consensus: keep the legacy dev HPA path.
        // Multi-shard storage ownership is fixed by shardCount and is never
        // changed by HPA.
        out.push(serving_hpa(lumen, &cx));
    }
    // raft-HA (`replicasPerShard > 1`): no HPA — raft needs a fixed membership.
    out.push(render::pdb(&cx, &name, COMPONENT, 1));
    if lumen.spec.observability {
        out.push(service_monitor(&cx));
        out.push(prometheus_rule(&cx));
    }
    // Optional scheduled backup runner: only when a policy is configured (#808).
    if let Some(cj) = backup_cron_job(lumen, &cx) {
        out.push(cj);
    }
    out
}

/// The optional backup CronJob (#808): rendered only when
/// `spec.serving.backup` is set. Lumen already produces a consistent
/// point-in-time snapshot over HTTP (`GET /admin/backup`, see
/// `projects/lumen/src/api.rs`); this CronJob adds nothing new to the
/// WAL/snapshot path, it only *schedules and transports* that existing
/// endpoint's bytes to a destination via `lumen backup`
/// (`libs/service-backup`). The shared [`operator::render::cron_job`] helper
/// stays manifest-only.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-render-rs.md#source
fn backup_cron_job(lumen: &Lumen, cx: &RenderCtx<'_>) -> Option<Value> {
    let policy = lumen.spec.serving.backup.as_ref()?;
    let cron_name = format!("{}-backup", cx.name);
    // Cluster-DNS FQDN of the serving ClusterIP Service (`serving_service`),
    // reachable from any namespace's CronJob pod regardless of the operator's
    // own DNS search suffix.
    let url = format!(
        "http://{}.{}.svc.cluster.local:{CLIENT_PORT}",
        cx.name, cx.ns
    );
    let mut args = vec![
        "backup".to_string(),
        "--url".to_string(),
        url,
        "--dest".to_string(),
        policy.destination.clone(),
    ];
    if let Some(secs) = policy.retention_secs {
        args.push("--retention-secs".to_string());
        args.push(secs.to_string());
    }
    let mut env = Vec::new();
    if let Some(secret) = &policy.admin_token_secret {
        env.push(json!({
            "name": "LUMEN_BACKUP_TOKEN",
            "valueFrom": { "secretKeyRef": { "name": secret, "key": "token" } },
        }));
    }
    let image_pull_policy = lumen
        .spec
        .image_pull_policy
        .clone()
        .unwrap_or_else(|| "IfNotPresent".to_string());
    Some(render::cron_job(render::CronJob {
        cx,
        name: &cron_name,
        component: BACKUP_COMPONENT,
        schedule: &policy.schedule,
        image: lumen.spec.image.as_str(),
        image_pull_policy: &image_pull_policy,
        command: vec!["lumen".into()],
        args,
        env,
        env_from: vec![],
        volumes: vec![],
        volume_mounts: vec![],
        service_account_name: Some(cx.name),
        cpu: "100m",
        memory: "128Mi",
        successful_jobs_history_limit: 3,
        failed_jobs_history_limit: 3,
    }))
}

/// The serving fleet: the shared workload primitive provides the StatefulSet's
/// identity, headless binding, downward-API pod identity, and common pod
/// template shell; Lumen layers its own ConfigMap-driven env, auth-secret
/// mount, PVC, probes, and observability annotations on top. At
/// `replicasPerShard <= 1` (single shard) the single-member path strips the
/// raft-only env vars and resets the apply-time replica count to exactly 1
/// (#1317) — `autoscaling.minReplicas` is ignored here since more than one
/// pod would be an uncoordinated shard-0 copy with no consensus link.
fn serving_statefulset(lumen: &Lumen, cx: &RenderCtx<'_>, headless: &str) -> Value {
    let s = &lumen.spec.serving;
    let res = render::guaranteed_resources(&s.cpu, &s.memory);
    let mut volume_mounts = vec![json!({ "name": "tmp", "mountPath": "/tmp" })];
    let mut volumes = vec![json!({ "name": "tmp", "emptyDir": {} })];
    if let Some(source) = token_registry_source(lumen) {
        volume_mounts.push(json!({
            "name": TOKEN_REGISTRY_VOLUME,
            "mountPath": TOKEN_REGISTRY_MOUNT_DIR,
            "readOnly": true,
        }));
        let mut volume = json!({ "name": TOKEN_REGISTRY_VOLUME });
        match source {
            TokenRegistrySource::Secret(secret) => {
                volume["secret"] = json!({
                    "secretName": secret,
                    "items": [{ "key": TOKEN_REGISTRY_KEY, "path": TOKEN_REGISTRY_KEY }],
                });
            }
            TokenRegistrySource::Csi(provider_class) => {
                volume["csi"] = json!({
                    "driver": "secrets-store.csi.k8s.io",
                    "readOnly": true,
                    "volumeAttributes": { "secretProviderClass": provider_class },
                });
            }
        }
        volumes.push(volume);
    }
    let spread = |key: &str| {
        json!({
            "maxSkew": 1,
            "topologyKey": key,
            "whenUnsatisfiable": "ScheduleAnyway",
            "labelSelector": { "matchLabels": cx.selector(COMPONENT) },
        })
    };
    let image_pull_policy = lumen
        .spec
        .image_pull_policy
        .as_deref()
        .unwrap_or("IfNotPresent");
    let mut pvc_template = json!({
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": { "requests": { "storage": s.raft_storage.clone() } },
        },
    });
    if let Some(sc) = &s.raft_storage_class {
        pvc_template["spec"]["storageClassName"] = json!(sc);
    }
    let mut sts = render::service_statefulset(ServiceStatefulSet {
        cx,
        name: cx.name,
        component: COMPONENT,
        image: lumen.spec.image.as_str(),
        image_pull_policy,
        command: vec!["lumen".into(), "serve".into()],
        args: vec![],
        ports: vec![json!({ "name": "http", "containerPort": CLIENT_PORT, "protocol": "TCP" })],
        headless_service: headless,
        shard_count: lumen.spec.shard_count,
        replicas_per_shard: lumen.spec.replicas_per_shard,
        voter_count: lumen.spec.voter_count,
        headless_env_key: HEADLESS_ENV_KEY,
        service_account_name: Some(cx.name),
        env: serving_env(lumen),
        env_from: vec![],
        resources: res,
        pod_annotations: Some(json!({
            "prometheus.io/scrape": "true",
            "prometheus.io/port": CLIENT_PORT.to_string(),
            "prometheus.io/path": "/metrics",
        })),
        pod_security_context: Some(json!({
            "runAsNonRoot": true,
            "runAsUser": 65532, "runAsGroup": 65532, "fsGroup": 65532,
            "seccompProfile": { "type": "RuntimeDefault" },
        })),
        container_security_context: Some(json!({
            "runAsNonRoot": true, "runAsUser": 65532, "runAsGroup": 65532,
            "allowPrivilegeEscalation": false,
            "readOnlyRootFilesystem": true,
            "capabilities": { "drop": ["ALL"] },
        })),
        termination_grace_period_seconds: Some(s.grace_secs),
        readiness_probe: Some(json!({
            "httpGet": { "path": "/readyz", "port": "http" },
            "initialDelaySeconds": 5, "periodSeconds": 10,
            "timeoutSeconds": 3, "failureThreshold": 60,
        })),
        liveness_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http" },
            "initialDelaySeconds": 15, "periodSeconds": 30,
            "timeoutSeconds": 5, "failureThreshold": 3,
        })),
        startup_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http" },
            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
        })),
        volumes,
        volume_mounts,
        topology_spread_constraints: vec![
            spread("topology.kubernetes.io/zone"),
            spread("kubernetes.io/hostname"),
        ],
        revision_history_limit: Some(5),
        update_strategy: Some(json!({ "type": "RollingUpdate" })),
        volume_claim: Some(WorkloadVolumeClaim {
            name: "raft".into(),
            template: pvc_template,
            mount_path: "/var/lib/lumen",
            read_only: false,
        }),
    });
    if lumen.spec.replicas_per_shard <= 1 {
        if let Some(spec) = sts["spec"].as_object_mut() {
            let replicas = if lumen.spec.shard_count > 1 {
                lumen.spec.shard_count as i32
            } else {
                // Single shard, single member, no raft consensus (#1317):
                // clamp to exactly 1 regardless of `autoscaling.minReplicas`
                // — see `LumenSpec::storage_pod_count` for why more than one
                // pod here means uncoordinated shard-0 copies.
                1
            };
            spec.insert("replicas".into(), json!(replicas));
        }
        if let Some(env) = sts["spec"]["template"]["spec"]["containers"][0]["env"].as_array_mut() {
            env.retain(|value| {
                let Some(name) = value["name"].as_str() else {
                    return true;
                };
                !matches!(
                    name,
                    "REPLICAS_PER_SHARD" | "VOTER_COUNT" | HEADLESS_ENV_KEY
                )
            });
        }
    }
    sts
}

/// Container env layered onto the shared pod identity/downward-API scaffold:
/// Lumen's literal runtime knobs + the config-driven values (so a ConfigMap
/// edit can roll pods).
fn serving_env(lumen: &Lumen) -> Vec<Value> {
    let cfg = format!("{}-config", instance(lumen));
    let from_cfg = |key: &str| json!({ "name": key, "valueFrom": { "configMapKeyRef": { "name": cfg, "key": key } } });
    let mut env = vec![
        json!({ "name": "LUMEN_HOST", "value": "0.0.0.0" }),
        json!({ "name": "LUMEN_WAL", "value": "auto" }),
        json!({ "name": "LUMEN_GRACE_SECS", "value": lumen.spec.serving.grace_secs.to_string() }),
        from_cfg("LUMEN_PORT"),
        from_cfg("LUMEN_LOG_FORMAT"),
        from_cfg("LUMEN_AUTH"),
        // #1384: mirror `serving_configmap`'s shard-map keys onto the
        // serving container so `lumen::config::shard_map_from_env` (used by
        // `serve()`'s `EngineShardSearch::new_with_shard_map` wiring) can
        // actually see the operator/reshard-driver-committed map instead of
        // always falling back to the balanced default.
        from_cfg("SHARD_MAP_VERSION"),
        from_cfg("VIRTUAL_BUCKET_COUNT"),
    ];
    if lumen.spec.log_level.is_some() {
        env.push(from_cfg("LUMEN_LOG_LEVEL"));
    }
    // SHARD_MAP_ASSIGNMENTS is only written into the ConfigMap once
    // assignments are non-empty (see `serving_configmap` below); a
    // `configMapKeyRef` to an absent key would fail the pod at start, so
    // this must mirror that same condition exactly.
    if !lumen.spec.shard_map.assignments.is_empty() {
        env.push(from_cfg("SHARD_MAP_ASSIGNMENTS"));
    }
    // Strict auth: the registry is mounted from a Secret or CSI-provided projection.
    if token_registry_source(lumen).is_some() {
        env.push(json!({
            "name": "LUMEN_TOKEN_REGISTRY_FILE",
            "value": TOKEN_REGISTRY_FILE,
        }));
    }
    if let Some(bootstrap) = &lumen.spec.serving.bootstrap {
        env.push(json!({
            "name": "LUMEN_BOOTSTRAP_SEED_URI",
            "value": bootstrap.seed_uri,
        }));
        if let Some(limit) = bootstrap.max_bytes_per_sec {
            env.push(json!({
                "name": "LUMEN_BOOTSTRAP_MAX_BYTES_PER_SEC",
                "value": limit.to_string(),
            }));
        }
    }
    env
}

fn serving_configmap(lumen: &Lumen, cx: &RenderCtx<'_>) -> Value {
    let name = format!("{}-config", cx.name);
    let mut data = json!({
        "SHARD_COUNT": lumen.spec.shard_count.to_string(),
        "SHARD_MAP_VERSION": lumen.spec.shard_map.version.to_string(),
        "VIRTUAL_BUCKET_COUNT": lumen.spec.shard_map.virtual_bucket_count.to_string(),
        "LUMEN_LOG_FORMAT": lumen.spec.log_format.as_env(),
        "LUMEN_PORT": CLIENT_PORT.to_string(),
        "LUMEN_AUTH": lumen.spec.auth.as_env(),
    });
    if !lumen.spec.shard_map.assignments.is_empty() {
        data["SHARD_MAP_ASSIGNMENTS"] = json!(lumen
            .spec
            .shard_map
            .assignments
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","));
    }
    if let Some(level) = &lumen.spec.log_level {
        data["LUMEN_LOG_LEVEL"] = json!(level);
    }
    json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": cx.meta(&name, COMPONENT),
        "data": data,
    })
}

fn serving_hpa(lumen: &Lumen, cx: &RenderCtx<'_>) -> Value {
    let a = &lumen.spec.serving.autoscaling;
    // Only called for single-shard, single-member deployments (`render`'s
    // `replicas_per_shard <= 1 && shard_count <= 1` guard) — no raft
    // consensus, so more than one live pod would be an uncoordinated
    // shard-0 copy behind this HPA-scaled StatefulSet (#1317, confirmed on
    // a kind cluster). Clamp both bounds to 1 regardless of the CR's
    // `serving.autoscaling` values; CPU-driven scaling requires opting into
    // `replicasPerShard > 1` (raft-HA), which never renders an HPA at all.
    let min_replicas = 1;
    let max_replicas = 1;
    render::horizontal_pod_autoscaler(HorizontalPodAutoscaler {
        cx,
        name: cx.name,
        component: COMPONENT,
        target_api_version: "apps/v1",
        target_kind: "StatefulSet",
        target_name: cx.name,
        min_replicas,
        max_replicas,
        metrics: vec![json!({
            "type": "Resource",
            "resource": { "name": "cpu", "target": { "type": "Utilization", "averageUtilization": a.target_cpu_utilization } },
        })],
        behavior: Some(json!({
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
        })),
    })
}

// ---- Observability (optional) ---------------------------------------------

fn service_monitor(cx: &RenderCtx<'_>) -> Value {
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "ServiceMonitor",
        "metadata": cx.meta(cx.name, COMPONENT),
        "spec": {
            "selector": { "matchLabels": cx.selector(COMPONENT) },
            "endpoints": [{ "port": "http", "path": "/metrics", "interval": "30s" }],
        },
    })
}

fn prometheus_rule(cx: &RenderCtx<'_>) -> Value {
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "PrometheusRule",
        "metadata": cx.meta(cx.name, COMPONENT),
        "spec": {
            "groups": [{
                "name": "lumen.slo",
                "rules": [{
                    "alert": "LumenNoReadyServingPods",
                    "expr": format!("kube_deployment_status_replicas_available{{deployment=\"{}\"}} == 0", cx.name),
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

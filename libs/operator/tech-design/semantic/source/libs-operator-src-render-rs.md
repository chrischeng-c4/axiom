---
id: libs-operator-src-render-rs
summary: Lossless rust-source-unit coverage for `libs/operator/src/render.rs`.
capability_refs:
  - id: shared-kubernetes-operator-scaffold
    role: primary
    claim: shared-kubernetes-operator-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Operator library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/operator/src/render.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/operator/src/render.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ENV_POD_NAME` | libs/operator/src/render.rs | const | pub | 19 | pub const ENV_POD_NAME: &str = "POD_NAME"; |
| `ENV_POD_NAMESPACE` | libs/operator/src/render.rs | const | pub | 20 | pub const ENV_POD_NAMESPACE: &str = "POD_NAMESPACE"; |
| `ENV_SHARD_COUNT` | libs/operator/src/render.rs | const | pub | 21 | pub const ENV_SHARD_COUNT: &str = "SHARD_COUNT"; |
| `ENV_REPLICAS_PER_SHARD` | libs/operator/src/render.rs | const | pub | 22 | pub const ENV_REPLICAS_PER_SHARD: &str = "REPLICAS_PER_SHARD"; |
| `ENV_VOTER_COUNT` | libs/operator/src/render.rs | const | pub | 23 | pub const ENV_VOTER_COUNT: &str = "VOTER_COUNT"; |
| `RenderCtx` | libs/operator/src/render.rs | struct | pub | 26 | pub struct RenderCtx<'a> { |
| `labels` | libs/operator/src/render.rs | function | pub | 38 | pub fn labels(&self, component: &str) -> Value { |
| `selector` | libs/operator/src/render.rs | function | pub | 50 | pub fn selector(&self, component: &str) -> Value { |
| `meta` | libs/operator/src/render.rs | function | pub | 59 | pub fn meta(&self, name: &str, component: &str) -> Value { |
| `owner_ref` | libs/operator/src/render.rs | function | pub | 70 | pub fn owner_ref(api_version: &str, kind: &str, name: &str, uid: &str) -> Value { |
| `guaranteed_resources` | libs/operator/src/render.rs | function | pub | 82 | pub fn guaranteed_resources(cpu: &str, memory: &str) -> Value { |
| `WorkloadVolumeClaim` | libs/operator/src/render.rs | struct | pub | 90 | pub struct WorkloadVolumeClaim<'a> { |
| `service_account` | libs/operator/src/render.rs | function | pub | 98 | pub fn service_account(cx: &RenderCtx, component: &str) -> Value { |
| `headless_service_with_ports` | libs/operator/src/render.rs | function | pub | 137 | pub fn headless_service_with_ports( |
| `headless_service` | libs/operator/src/render.rs | function | pub | 147 | pub fn headless_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value { |
| `client_service_with_ports` | libs/operator/src/render.rs | function | pub | 157 | pub fn client_service_with_ports( |
| `client_service` | libs/operator/src/render.rs | function | pub | 167 | pub fn client_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value { |
| `pdb` | libs/operator/src/render.rs | function | pub | 177 | pub fn pdb(cx: &RenderCtx, name: &str, component: &str, max_unavailable: i32) -> Value { |
| `HorizontalPodAutoscaler` | libs/operator/src/render.rs | struct | pub | 187 | pub struct HorizontalPodAutoscaler<'a> { |
| `horizontal_pod_autoscaler` | libs/operator/src/render.rs | function | pub | 201 | pub fn horizontal_pod_autoscaler(p: HorizontalPodAutoscaler) -> Value { |
| `CronJob` | libs/operator/src/render.rs | struct | pub | 236 | pub struct CronJob<'a> { |
| `cron_job` | libs/operator/src/render.rs | function | pub | 260 | pub fn cron_job(p: CronJob) -> Value { |
| `ServiceStatefulSet` | libs/operator/src/render.rs | struct | pub | 340 | pub struct ServiceStatefulSet<'a> { |
| `service_statefulset` | libs/operator/src/render.rs | function | pub | 381 | pub fn service_statefulset(p: ServiceStatefulSet) -> Value { |
| `ShardedStatefulSet` | libs/operator/src/render.rs | struct | pub | 525 | pub struct ShardedStatefulSet<'a> { |
| `sharded_statefulset` | libs/operator/src/render.rs | function | pub | 554 | pub fn sharded_statefulset(p: ShardedStatefulSet) -> Value { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The sharded-HA render toolkit: a [`RenderCtx`] carrying the per-service
//! identity (app/manager/GVK/name/ns/owner) plus helpers that emit the common
//! k8s objects — labels/selector/meta, ServiceAccount, headless + client
//! Services, PodDisruptionBudget, CronJobs, and [`sharded_statefulset`]: the
//! downward-API StatefulSet whose env feeds
//! `raft_host::cluster::ClusterTopology::from_env`.
//!
//! Lifted + parameterized from lumen's `operator::render` helpers. A service
//! keeps its own service-specific rendering and calls these for the shared
//! shapes.

use serde_json::{json, Value};

// The downward-API env keys a sharded-HA StatefulSet injects. These MUST match
// `raft_host::cluster::ClusterTopology::from_env` (the consumer) — duplicated
// here (rather than depending on raft-host) to keep this kube-only lib free of
// the raftcore/h2c/reqwest dep tree; the `downward_api_env_keys` test asserts
// `sharded_statefulset` emits exactly these.
pub const ENV_POD_NAME: &str = "POD_NAME";
pub const ENV_POD_NAMESPACE: &str = "POD_NAMESPACE";
pub const ENV_SHARD_COUNT: &str = "SHARD_COUNT";
pub const ENV_REPLICAS_PER_SHARD: &str = "REPLICAS_PER_SHARD";
pub const ENV_VOTER_COUNT: &str = "VOTER_COUNT";

/// Per-service render identity, threaded through the helpers.
pub struct RenderCtx<'a> {
    pub app: &'a str,
    pub manager: &'a str,
    pub api_version: &'a str,
    pub kind: &'a str,
    pub name: &'a str,
    pub ns: &'a str,
    pub owner: Option<Value>,
}

impl RenderCtx<'_> {
    /// Recommended labels common to every child object.
    pub fn labels(&self, component: &str) -> Value {
        json!({
            "app.kubernetes.io/name": self.app,
            "app.kubernetes.io/instance": self.name,
            "app.kubernetes.io/component": component,
            "app.kubernetes.io/managed-by": self.manager,
            "app.kubernetes.io/part-of": self.app,
        })
    }

    /// Immutable selector labels (a subset of [`Self::labels`]) — workload and
    /// Service selectors pin to these so re-applies never hit selector-immutability.
    pub fn selector(&self, component: &str) -> Value {
        json!({
            "app.kubernetes.io/name": self.app,
            "app.kubernetes.io/instance": self.name,
            "app.kubernetes.io/component": component,
        })
    }

    /// Assemble an object's `metadata` block (name/ns/labels + owner ref).
    pub fn meta(&self, name: &str, component: &str) -> Value {
        let mut m = json!({ "name": name, "namespace": self.ns, "labels": self.labels(component) });
        if let Some(o) = &self.owner {
            m["ownerReferences"] = json!([o]);
        }
        m
    }
}

/// The owner reference that ties a child to its CR (cascading GC). `uid` comes
/// from the live CR's metadata.
pub fn owner_ref(api_version: &str, kind: &str, name: &str, uid: &str) -> Value {
    json!({
        "apiVersion": api_version,
        "kind": kind,
        "name": name,
        "uid": uid,
        "controller": true,
        "blockOwnerDeletion": true,
    })
}

/// Guaranteed-QoS CPU/memory resources (`requests == limits`).
pub fn guaranteed_resources(cpu: &str, memory: &str) -> Value {
    json!({
        "requests": { "cpu": cpu, "memory": memory },
        "limits": { "cpu": cpu, "memory": memory },
    })
}

/// A rendered PVC template plus the container mount path it should back.
pub struct WorkloadVolumeClaim<'a> {
    pub name: String,
    pub template: Value,
    pub mount_path: &'a str,
    pub read_only: bool,
}

/// A ServiceAccount for the workload pods.
pub fn service_account(cx: &RenderCtx, component: &str) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": cx.meta(cx.name, component),
    })
}

fn service(
    cx: &RenderCtx,
    name: &str,
    component: &str,
    ports: Vec<Value>,
    cluster_ip: Option<&str>,
    publish_not_ready_addresses: bool,
    service_type: Option<&str>,
) -> Value {
    let mut spec = json!({
        "selector": cx.selector(component),
        "ports": ports,
    });
    if let Some(cluster_ip) = cluster_ip {
        spec["clusterIP"] = json!(cluster_ip);
    }
    if publish_not_ready_addresses {
        spec["publishNotReadyAddresses"] = json!(true);
    }
    if let Some(service_type) = service_type {
        spec["type"] = json!(service_type);
    }
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": cx.meta(name, component),
        "spec": spec,
    })
}

/// A headless Service with caller-supplied ports.
pub fn headless_service_with_ports(
    cx: &RenderCtx,
    name: &str,
    component: &str,
    ports: Vec<Value>,
) -> Value {
    service(cx, name, component, ports, Some("None"), true, None)
}

/// A headless Service (stable per-pod DNS for a StatefulSet's peers).
pub fn headless_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value {
    headless_service_with_ports(
        cx,
        name,
        component,
        vec![json!({ "name": "http", "port": port, "targetPort": "http", "protocol": "TCP" })],
    )
}

/// A ClusterIP Service with caller-supplied ports.
pub fn client_service_with_ports(
    cx: &RenderCtx,
    name: &str,
    component: &str,
    ports: Vec<Value>,
) -> Value {
    service(cx, name, component, ports, None, false, Some("ClusterIP"))
}

/// A ClusterIP client Service.
pub fn client_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value {
    client_service_with_ports(
        cx,
        name,
        component,
        vec![json!({ "name": "http", "port": port, "targetPort": "http", "protocol": "TCP" })],
    )
}

/// A PodDisruptionBudget.
pub fn pdb(cx: &RenderCtx, name: &str, component: &str, max_unavailable: i32) -> Value {
    json!({
        "apiVersion": "policy/v1",
        "kind": "PodDisruptionBudget",
        "metadata": cx.meta(name, component),
        "spec": { "maxUnavailable": max_unavailable, "selector": { "matchLabels": cx.selector(component) } },
    })
}

/// Parameters for [`horizontal_pod_autoscaler`].
pub struct HorizontalPodAutoscaler<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub name: &'a str,
    pub component: &'a str,
    pub target_api_version: &'a str,
    pub target_kind: &'a str,
    pub target_name: &'a str,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub metrics: Vec<Value>,
    pub behavior: Option<Value>,
}

/// A HorizontalPodAutoscaler targeting a rendered service workload.
pub fn horizontal_pod_autoscaler(p: HorizontalPodAutoscaler) -> Value {
    let HorizontalPodAutoscaler {
        cx,
        name,
        component,
        target_api_version,
        target_kind,
        target_name,
        min_replicas,
        max_replicas,
        metrics,
        behavior,
    } = p;
    let mut spec = json!({
        "scaleTargetRef": {
            "apiVersion": target_api_version,
            "kind": target_kind,
            "name": target_name,
        },
        "minReplicas": min_replicas,
        "maxReplicas": max_replicas,
        "metrics": metrics,
    });
    if let Some(behavior) = behavior {
        spec["behavior"] = behavior;
    }
    json!({
        "apiVersion": "autoscaling/v2",
        "kind": "HorizontalPodAutoscaler",
        "metadata": cx.meta(name, component),
        "spec": spec,
    })
}

/// Parameters for [`cron_job`].
pub struct CronJob<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub name: &'a str,
    pub component: &'a str,
    pub schedule: &'a str,
    pub image: &'a str,
    pub image_pull_policy: &'a str,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<Value>,
    pub env_from: Vec<Value>,
    pub volumes: Vec<Value>,
    pub volume_mounts: Vec<Value>,
    pub service_account_name: Option<&'a str>,
    pub cpu: &'a str,
    pub memory: &'a str,
    pub successful_jobs_history_limit: i32,
    pub failed_jobs_history_limit: i32,
}

/// A CronJob for service-side maintenance runners such as object-store backups.
///
/// Operators schedule and wire the runner; the service or runner still owns the
/// actual domain bytes. This helper deliberately stays manifest-only.
pub fn cron_job(p: CronJob) -> Value {
    let cx = p.cx;
    let mut container = json!({
        "name": p.component,
        "image": p.image,
        "imagePullPolicy": p.image_pull_policy,
        "command": p.command,
        "args": p.args,
        "env": p.env,
        "resources": {
            "requests": { "cpu": p.cpu, "memory": p.memory },
            "limits": { "cpu": p.cpu, "memory": p.memory },
        },
    });
    if !p.env_from.is_empty() {
        container["envFrom"] = json!(p.env_from);
    }
    if !p.volume_mounts.is_empty() {
        container["volumeMounts"] = json!(p.volume_mounts);
    }

    let mut pod_spec = json!({
        "restartPolicy": "OnFailure",
        "containers": [container],
    });
    if let Some(service_account_name) = p.service_account_name {
        pod_spec["serviceAccountName"] = json!(service_account_name);
    }
    if !p.volumes.is_empty() {
        pod_spec["volumes"] = json!(p.volumes);
    }

    json!({
        "apiVersion": "batch/v1",
        "kind": "CronJob",
        "metadata": cx.meta(p.name, p.component),
        "spec": {
            "schedule": p.schedule,
            "concurrencyPolicy": "Forbid",
            "successfulJobsHistoryLimit": p.successful_jobs_history_limit,
            "failedJobsHistoryLimit": p.failed_jobs_history_limit,
            "jobTemplate": {
                "spec": {
                    "template": {
                        "metadata": { "labels": cx.labels(p.component) },
                        "spec": pod_spec,
                    },
                },
            },
        },
    })
}

fn merge_labels(target: &mut Value, labels: &Value) {
    if !target.is_object() {
        *target = json!({});
    }
    let target = target.as_object_mut().expect("labels object");
    for (key, value) in labels.as_object().expect("labels object") {
        target.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

fn ensure_named_template_metadata(mut template: Value, name: &str, labels: &Value) -> Value {
    if !template.is_object() {
        template = json!({});
    }
    let template_obj = template.as_object_mut().expect("template object");
    let metadata = template_obj.entry("metadata").or_insert_with(|| json!({}));
    if !metadata.is_object() {
        *metadata = json!({});
    }
    let metadata_obj = metadata.as_object_mut().expect("metadata object");
    metadata_obj.insert("name".into(), json!(name));
    let labels_value = metadata_obj.entry("labels").or_insert_with(|| json!({}));
    merge_labels(labels_value, labels);
    template
}

/// Parameters for [`service_statefulset`].
pub struct ServiceStatefulSet<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub name: &'a str,
    pub component: &'a str,
    pub image: &'a str,
    pub image_pull_policy: &'a str,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub ports: Vec<Value>,
    /// The headless Service name (`serviceName`) + the value of `headless_env_key`.
    pub headless_service: &'a str,
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    /// The env key the service reads for its headless-DNS suffix
    /// (e.g. `LUMEN_HEADLESS_SERVICE`).
    pub headless_env_key: &'a str,
    pub service_account_name: Option<&'a str>,
    pub env: Vec<Value>,
    pub env_from: Vec<Value>,
    pub resources: Value,
    pub pod_annotations: Option<Value>,
    pub pod_security_context: Option<Value>,
    pub container_security_context: Option<Value>,
    pub termination_grace_period_seconds: Option<u64>,
    pub readiness_probe: Option<Value>,
    pub liveness_probe: Option<Value>,
    pub startup_probe: Option<Value>,
    pub volumes: Vec<Value>,
    pub volume_mounts: Vec<Value>,
    pub topology_spread_constraints: Vec<Value>,
    pub revision_history_limit: Option<i32>,
    pub update_strategy: Option<Value>,
    /// `Some(pvc)` for a durable workload (adds the claim template + mount).
    pub volume_claim: Option<WorkloadVolumeClaim<'a>>,
}

/// A configurable, downward-API StatefulSet primitive for sharded service
/// workloads. It preserves the exact raft-host env contract while letting a
/// service supply its own probes, security hardening, storage path, extra
/// volumes, and rollout details.
pub fn service_statefulset(p: ServiceStatefulSet) -> Value {
    let ServiceStatefulSet {
        cx,
        name,
        component,
        image,
        image_pull_policy,
        command,
        args,
        ports,
        headless_service,
        shard_count,
        replicas_per_shard,
        voter_count,
        headless_env_key,
        service_account_name,
        env: extra_env,
        env_from,
        resources,
        pod_annotations,
        pod_security_context,
        container_security_context,
        termination_grace_period_seconds,
        readiness_probe,
        liveness_probe,
        startup_probe,
        volumes,
        volume_mounts,
        topology_spread_constraints,
        revision_history_limit,
        update_strategy,
        volume_claim,
    } = p;

    let mut env = vec![
        json!({ "name": ENV_POD_NAME, "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": ENV_POD_NAMESPACE, "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } }),
        json!({ "name": ENV_SHARD_COUNT, "value": shard_count.to_string() }),
        json!({ "name": ENV_REPLICAS_PER_SHARD, "value": replicas_per_shard.to_string() }),
        json!({ "name": ENV_VOTER_COUNT, "value": voter_count.to_string() }),
        json!({ "name": headless_env_key, "value": headless_service }),
    ];
    env.extend(extra_env);

    let mut container = json!({
        "name": component,
        "image": image,
        "imagePullPolicy": image_pull_policy,
        "command": command,
        "ports": ports,
        "env": env,
        "resources": resources,
    });
    if !args.is_empty() {
        container["args"] = json!(args);
    }
    if !env_from.is_empty() {
        container["envFrom"] = json!(env_from);
    }
    if let Some(readiness_probe) = readiness_probe {
        container["readinessProbe"] = readiness_probe;
    }
    if let Some(liveness_probe) = liveness_probe {
        container["livenessProbe"] = liveness_probe;
    }
    if let Some(startup_probe) = startup_probe {
        container["startupProbe"] = startup_probe;
    }
    if let Some(container_security_context) = container_security_context {
        container["securityContext"] = container_security_context;
    }

    let mut mounts = volume_mounts;
    let mut claim_templates = Vec::new();
    if let Some(claim) = volume_claim {
        let claim_name = claim.name;
        mounts.push(json!({
            "name": claim_name.clone(),
            "mountPath": claim.mount_path,
            "readOnly": claim.read_only,
        }));
        claim_templates.push(ensure_named_template_metadata(
            claim.template,
            &claim_name,
            &cx.labels(component),
        ));
    }
    if !mounts.is_empty() {
        container["volumeMounts"] = json!(mounts);
    }

    let mut pod_metadata = json!({ "labels": cx.labels(component) });
    if let Some(pod_annotations) = pod_annotations {
        pod_metadata["annotations"] = pod_annotations;
    }

    let mut pod_spec = json!({
        "containers": [container],
    });
    if let Some(service_account_name) = service_account_name {
        pod_spec["serviceAccountName"] = json!(service_account_name);
    }
    if let Some(termination_grace_period_seconds) = termination_grace_period_seconds {
        pod_spec["terminationGracePeriodSeconds"] = json!(termination_grace_period_seconds);
    }
    if let Some(pod_security_context) = pod_security_context {
        pod_spec["securityContext"] = pod_security_context;
    }
    if !volumes.is_empty() {
        pod_spec["volumes"] = json!(volumes);
    }
    if !topology_spread_constraints.is_empty() {
        pod_spec["topologySpreadConstraints"] = json!(topology_spread_constraints);
    }

    let mut spec = json!({
        "replicas": shard_count * replicas_per_shard,
        "serviceName": headless_service,
        "podManagementPolicy": "Parallel",
        "selector": { "matchLabels": cx.selector(component) },
        "template": {
            "metadata": pod_metadata,
            "spec": pod_spec,
        },
    });
    if let Some(revision_history_limit) = revision_history_limit {
        spec["revisionHistoryLimit"] = json!(revision_history_limit);
    }
    if let Some(update_strategy) = update_strategy {
        spec["updateStrategy"] = update_strategy;
    }
    if !claim_templates.is_empty() {
        spec["volumeClaimTemplates"] = json!(claim_templates);
    }

    json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": cx.meta(name, component),
        "spec": spec,
    })
}

/// Parameters for [`sharded_statefulset`].
pub struct ShardedStatefulSet<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub name: &'a str,
    pub component: &'a str,
    pub image: &'a str,
    pub image_pull_policy: &'a str,
    pub command: Vec<String>,
    pub ports: Vec<(&'a str, i32)>,
    /// The headless Service name (`serviceName`) + the value of `headless_env_key`.
    pub headless_service: &'a str,
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    /// The env key the service reads for its headless-DNS suffix
    /// (e.g. `LUMEN_HEADLESS_SERVICE`).
    pub headless_env_key: &'a str,
    pub cpu: &'a str,
    pub memory: &'a str,
    /// Service-specific env appended after the downward-API quartet.
    pub extra_env: Vec<Value>,
    /// `Some(pvc)` for a durable workload (adds the claim template + a `/data` mount).
    pub volume_claim: Option<Value>,
}

/// The downward-API StatefulSet: `replicas = shard_count * replicas_per_shard`,
/// `podManagementPolicy: Parallel`, and the env quartet
/// (`POD_NAME`/`POD_NAMESPACE`/`SHARD_COUNT`/`REPLICAS_PER_SHARD`/`VOTER_COUNT`)
/// + `<headless_env_key>` that `raft_host::cluster::ClusterTopology::from_env`
/// reads to derive node id / membership / peers.
pub fn sharded_statefulset(p: ShardedStatefulSet) -> Value {
    let volume_claim = p.volume_claim.map(|template| {
        let name = template["metadata"]["name"]
            .as_str()
            .unwrap_or("data")
            .to_owned();
        WorkloadVolumeClaim {
            name,
            template,
            mount_path: "/data",
            read_only: false,
        }
    });

    service_statefulset(ServiceStatefulSet {
        cx: p.cx,
        name: p.name,
        component: p.component,
        image: p.image,
        image_pull_policy: p.image_pull_policy,
        command: p.command,
        args: vec![],
        ports: p
            .ports
            .iter()
            .map(|(n, port)| json!({ "name": n, "containerPort": port, "protocol": "TCP" }))
            .collect(),
        headless_service: p.headless_service,
        shard_count: p.shard_count,
        replicas_per_shard: p.replicas_per_shard,
        voter_count: p.voter_count,
        headless_env_key: p.headless_env_key,
        service_account_name: Some(p.cx.name),
        env: p.extra_env,
        env_from: vec![],
        resources: guaranteed_resources(p.cpu, p.memory),
        pod_annotations: None,
        pod_security_context: None,
        container_security_context: None,
        termination_grace_period_seconds: None,
        readiness_probe: None,
        liveness_probe: None,
        startup_probe: None,
        volumes: vec![],
        volume_mounts: vec![],
        topology_spread_constraints: vec![],
        revision_history_limit: None,
        update_strategy: None,
        volume_claim,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cx() -> RenderCtx<'static> {
        RenderCtx {
            app: "svc",
            manager: "svc-operator",
            api_version: "svc.dev/v1",
            kind: "Svc",
            name: "s",
            ns: "ns",
            owner: None,
        }
    }

    #[test]
    fn helper_shapes() {
        let cx = cx();
        assert_eq!(service_account(&cx, "server")["kind"], "ServiceAccount");
        let h = headless_service_with_ports(
            &cx,
            "s-h",
            "server",
            vec![
                json!({ "name": "http", "port": 7000, "targetPort": "http", "protocol": "TCP" }),
                json!({ "name": "grpc", "port": 7443, "targetPort": "grpc", "protocol": "TCP" }),
            ],
        );
        assert_eq!(h["spec"]["clusterIP"], "None");
        assert_eq!(h["spec"]["publishNotReadyAddresses"], true);
        assert_eq!(h["spec"]["ports"].as_array().unwrap().len(), 2);
        assert_eq!(
            client_service_with_ports(
                &cx,
                "s",
                "server",
                vec![
                    json!({ "name": "http", "port": 7000, "targetPort": "http", "protocol": "TCP" })
                ],
            )["spec"]["type"],
            "ClusterIP"
        );
        assert_eq!(pdb(&cx, "s", "server", 1)["spec"]["maxUnavailable"], 1);
        // labels carry the per-service manager.
        assert_eq!(
            cx.labels("server")["app.kubernetes.io/managed-by"],
            "svc-operator"
        );
    }

    #[test]
    fn cron_job_wires_runner_without_domain_bytes() {
        let cx = cx();
        let cj = cron_job(CronJob {
            cx: &cx,
            name: "s-backup",
            component: "backup",
            schedule: "*/5 * * * *",
            image: "svc:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["svc".into()],
            args: vec!["backup".into(), "run".into()],
            env: vec![json!({ "name": "DESTINATION", "value": "s3://bucket/prefix" })],
            env_from: vec![json!({ "secretRef": { "name": "s-backup" } })],
            volumes: vec![json!({ "name": "token", "projected": {} })],
            volume_mounts: vec![json!({ "name": "token", "mountPath": "/var/run/secrets" })],
            service_account_name: Some("s-backup"),
            cpu: "100m",
            memory: "128Mi",
            successful_jobs_history_limit: 1,
            failed_jobs_history_limit: 3,
        });
        assert_eq!(cj["kind"], "CronJob");
        assert_eq!(cj["spec"]["concurrencyPolicy"], "Forbid");
        assert_eq!(cj["spec"]["schedule"], "*/5 * * * *");
        let pod = &cj["spec"]["jobTemplate"]["spec"]["template"]["spec"];
        assert_eq!(pod["serviceAccountName"], "s-backup");
        assert_eq!(pod["restartPolicy"], "OnFailure");
        assert_eq!(
            pod["containers"][0]["env"][0]["value"],
            "s3://bucket/prefix"
        );
        assert_eq!(
            pod["containers"][0]["envFrom"][0]["secretRef"]["name"],
            "s-backup"
        );
        assert_eq!(pod["volumes"][0]["name"], "token");
    }

    #[test]
    fn horizontal_pod_autoscaler_renders_expected_shape() {
        let cx = cx();
        let hpa = horizontal_pod_autoscaler(HorizontalPodAutoscaler {
            cx: &cx,
            name: "s",
            component: "server",
            target_api_version: "apps/v1",
            target_kind: "StatefulSet",
            target_name: "s",
            min_replicas: 2,
            max_replicas: 8,
            metrics: vec![json!({
                "type": "Resource",
                "resource": {
                    "name": "cpu",
                    "target": { "type": "Utilization", "averageUtilization": 70 }
                }
            })],
            behavior: Some(json!({
                "scaleUp": {
                    "stabilizationWindowSeconds": 30,
                    "policies": [{ "type": "Percent", "value": 100, "periodSeconds": 30 }]
                }
            })),
        });
        assert_eq!(hpa["kind"], "HorizontalPodAutoscaler");
        assert_eq!(hpa["spec"]["scaleTargetRef"]["kind"], "StatefulSet");
        assert_eq!(hpa["spec"]["minReplicas"], 2);
        assert_eq!(
            hpa["spec"]["metrics"][0]["resource"]["target"]["averageUtilization"],
            70
        );
        assert_eq!(
            hpa["spec"]["behavior"]["scaleUp"]["policies"][0]["periodSeconds"],
            30
        );
    }

    #[test]
    fn service_statefulset_keeps_exact_downward_api_env_contract() {
        let cx = cx();
        let ss = service_statefulset(ServiceStatefulSet {
            cx: &cx,
            name: "s",
            component: "server",
            image: "img:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["serve".into()],
            args: vec![],
            ports: vec![json!({ "name": "http", "containerPort": 7000, "protocol": "TCP" })],
            headless_service: "s-headless",
            shard_count: 2,
            replicas_per_shard: 3,
            voter_count: 3,
            headless_env_key: "SVC_HEADLESS_SERVICE",
            service_account_name: Some("s"),
            env: vec![json!({ "name": "EXTRA", "value": "x" })],
            env_from: vec![],
            resources: guaranteed_resources("1", "1Gi"),
            pod_annotations: None,
            pod_security_context: None,
            container_security_context: None,
            termination_grace_period_seconds: None,
            readiness_probe: None,
            liveness_probe: None,
            startup_probe: None,
            volumes: vec![],
            volume_mounts: vec![],
            topology_spread_constraints: vec![],
            revision_history_limit: None,
            update_strategy: None,
            volume_claim: None,
        });
        assert_eq!(ss["spec"]["replicas"], 6); // shard_count * replicas_per_shard
        assert_eq!(ss["spec"]["serviceName"], "s-headless");
        assert_eq!(ss["spec"]["podManagementPolicy"], "Parallel");
        let env = ss["spec"]["template"]["spec"]["containers"][0]["env"]
            .as_array()
            .unwrap();
        let keys: Vec<&str> = env.iter().map(|e| e["name"].as_str().unwrap()).collect();
        assert_eq!(
            &keys[..6],
            &[
                ENV_POD_NAME,
                ENV_POD_NAMESPACE,
                ENV_SHARD_COUNT,
                ENV_REPLICAS_PER_SHARD,
                ENV_VOTER_COUNT,
                "SVC_HEADLESS_SERVICE",
            ]
        );
        for k in [
            ENV_POD_NAME,
            ENV_POD_NAMESPACE,
            ENV_SHARD_COUNT,
            ENV_REPLICAS_PER_SHARD,
            ENV_VOTER_COUNT,
            "SVC_HEADLESS_SERVICE",
            "EXTRA",
        ] {
            assert!(keys.contains(&k), "missing env {k}");
        }
        // the field-ref quartet members use the downward API, not a literal value.
        let pod_name = env.iter().find(|e| e["name"] == ENV_POD_NAME).unwrap();
        assert_eq!(
            pod_name["valueFrom"]["fieldRef"]["fieldPath"],
            "metadata.name"
        );
    }

    #[test]
    fn service_statefulset_can_render_lumen_style_production_template() {
        let cx = cx();
        let spread = |key: &str| {
            json!({
                "maxSkew": 1,
                "topologyKey": key,
                "whenUnsatisfiable": "ScheduleAnyway",
                "labelSelector": { "matchLabels": cx.selector("server") },
            })
        };
        let ss = service_statefulset(ServiceStatefulSet {
            cx: &cx,
            name: "s",
            component: "server",
            image: "img:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["lumen".into(), "serve".into()],
            args: vec![],
            ports: vec![json!({ "name": "http", "containerPort": 7373, "protocol": "TCP" })],
            headless_service: "s-headless",
            shard_count: 1,
            replicas_per_shard: 1,
            voter_count: 1,
            headless_env_key: "LUMEN_HEADLESS_SERVICE",
            service_account_name: Some("s"),
            env: vec![
                json!({ "name": "LUMEN_TOKEN_REGISTRY_FILE", "value": "/var/run/secrets/lumen/token-registry.json" }),
            ],
            env_from: vec![json!({ "configMapRef": { "name": "s-config" } })],
            resources: guaranteed_resources("2", "4Gi"),
            pod_annotations: Some(json!({
                "prometheus.io/scrape": "true",
                "prometheus.io/port": "7373",
                "prometheus.io/path": "/metrics",
            })),
            pod_security_context: Some(json!({
                "runAsNonRoot": true,
                "runAsUser": 65532,
                "runAsGroup": 65532,
                "fsGroup": 65532,
                "seccompProfile": { "type": "RuntimeDefault" },
            })),
            container_security_context: Some(json!({
                "runAsNonRoot": true,
                "runAsUser": 65532,
                "runAsGroup": 65532,
                "allowPrivilegeEscalation": false,
                "readOnlyRootFilesystem": true,
                "capabilities": { "drop": ["ALL"] },
            })),
            termination_grace_period_seconds: Some(30),
            readiness_probe: Some(json!({
                "httpGet": { "path": "/readyz", "port": "http" },
                "initialDelaySeconds": 5, "periodSeconds": 10, "timeoutSeconds": 3, "failureThreshold": 60,
            })),
            liveness_probe: Some(json!({
                "httpGet": { "path": "/healthz", "port": "http" },
                "initialDelaySeconds": 15, "periodSeconds": 30, "timeoutSeconds": 5, "failureThreshold": 3,
            })),
            startup_probe: Some(json!({
                "httpGet": { "path": "/healthz", "port": "http" },
                "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
            })),
            volumes: vec![
                json!({ "name": "tmp", "emptyDir": {} }),
                json!({
                    "name": "token-registry",
                    "secret": {
                        "secretName": "lumen-token-registry",
                        "items": [{ "key": "token-registry.json", "path": "token-registry.json" }],
                    },
                }),
            ],
            volume_mounts: vec![
                json!({ "name": "tmp", "mountPath": "/tmp" }),
                json!({ "name": "token-registry", "mountPath": "/var/run/secrets/lumen", "readOnly": true }),
            ],
            topology_spread_constraints: vec![
                spread("topology.kubernetes.io/zone"),
                spread("kubernetes.io/hostname"),
            ],
            revision_history_limit: Some(5),
            update_strategy: Some(json!({ "type": "RollingUpdate" })),
            volume_claim: Some(WorkloadVolumeClaim {
                name: "raft".into(),
                template: json!({
                    "spec": {
                        "accessModes": ["ReadWriteOnce"],
                        "resources": { "requests": { "storage": "100Gi" } },
                    },
                }),
                mount_path: "/var/lib/lumen",
                read_only: false,
            }),
        });
        let pod = &ss["spec"]["template"]["spec"];
        let container = &pod["containers"][0];
        assert_eq!(
            ss["spec"]["template"]["metadata"]["annotations"]["prometheus.io/path"],
            "/metrics"
        );
        assert_eq!(pod["serviceAccountName"], "s");
        assert_eq!(pod["terminationGracePeriodSeconds"], 30);
        assert_eq!(
            pod["securityContext"]["seccompProfile"]["type"],
            "RuntimeDefault"
        );
        assert_eq!(
            pod["topologySpreadConstraints"].as_array().unwrap().len(),
            2
        );
        assert_eq!(container["resources"]["requests"]["memory"], "4Gi");
        assert_eq!(container["envFrom"][0]["configMapRef"]["name"], "s-config");
        assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
        assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
        assert_eq!(container["volumeMounts"].as_array().unwrap().len(), 3);
        assert_eq!(container["volumeMounts"][2]["mountPath"], "/var/lib/lumen");
        assert_eq!(
            ss["spec"]["volumeClaimTemplates"][0]["metadata"]["name"],
            "raft"
        );
        assert_eq!(
            ss["spec"]["volumeClaimTemplates"][0]["metadata"]["labels"]["app.kubernetes.io/name"],
            "svc"
        );
    }

    #[test]
    fn sharded_statefulset_legacy_wrapper_keeps_default_claim_mount() {
        let cx = cx();
        let ss = sharded_statefulset(ShardedStatefulSet {
            cx: &cx,
            name: "s",
            component: "server",
            image: "img:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["serve".into()],
            ports: vec![("http", 7000)],
            headless_service: "s-headless",
            shard_count: 1,
            replicas_per_shard: 1,
            voter_count: 1,
            headless_env_key: "SVC_HEADLESS_SERVICE",
            cpu: "1",
            memory: "1Gi",
            extra_env: vec![],
            volume_claim: Some(json!({ "metadata": { "name": "data" }, "spec": {} })),
        });
        assert!(ss["spec"]["volumeClaimTemplates"].is_array());
        assert_eq!(
            ss["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]["mountPath"],
            "/data"
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/operator/src/render.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/operator/src/render.rs` captured during libs codegen standardization.
```

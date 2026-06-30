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

/// A ServiceAccount for the workload pods.
pub fn service_account(cx: &RenderCtx, component: &str) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": cx.meta(cx.name, component),
    })
}

/// A headless Service (stable per-pod DNS for a StatefulSet's peers).
pub fn headless_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": cx.meta(name, component),
        "spec": {
            "clusterIP": "None",
            "publishNotReadyAddresses": true,
            "selector": cx.selector(component),
            "ports": [{ "name": "http", "port": port, "targetPort": "http", "protocol": "TCP" }],
        },
    })
}

/// A ClusterIP client Service.
pub fn client_service(cx: &RenderCtx, name: &str, component: &str, port: i32) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "Service",
        "metadata": cx.meta(name, component),
        "spec": {
            "type": "ClusterIP",
            "selector": cx.selector(component),
            "ports": [{ "name": "http", "port": port, "targetPort": "http", "protocol": "TCP" }],
        },
    })
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
    let cx = p.cx;
    let mut env = vec![
        json!({ "name": ENV_POD_NAME, "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": ENV_POD_NAMESPACE, "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } }),
        json!({ "name": ENV_SHARD_COUNT, "value": p.shard_count.to_string() }),
        json!({ "name": ENV_REPLICAS_PER_SHARD, "value": p.replicas_per_shard.to_string() }),
        json!({ "name": ENV_VOTER_COUNT, "value": p.voter_count.to_string() }),
        json!({ "name": p.headless_env_key, "value": p.headless_service }),
    ];
    env.extend(p.extra_env);
    let ports: Vec<Value> = p
        .ports
        .iter()
        .map(|(n, port)| json!({ "name": n, "containerPort": port, "protocol": "TCP" }))
        .collect();

    let mut container = json!({
        "name": p.component,
        "image": p.image,
        "imagePullPolicy": p.image_pull_policy,
        "command": p.command,
        "ports": ports,
        "env": env,
        "resources": {
            "requests": { "cpu": p.cpu, "memory": p.memory },
            "limits": { "cpu": p.cpu, "memory": p.memory },
        },
    });
    if p.volume_claim.is_some() {
        container["volumeMounts"] = json!([{ "name": "data", "mountPath": "/data" }]);
    }

    let mut spec = json!({
        "replicas": p.shard_count * p.replicas_per_shard,
        "serviceName": p.headless_service,
        "podManagementPolicy": "Parallel",
        "selector": { "matchLabels": cx.selector(p.component) },
        "template": {
            "metadata": { "labels": cx.labels(p.component) },
            "spec": {
                "serviceAccountName": cx.name,
                "containers": [container],
            },
        },
    });
    if let Some(vc) = p.volume_claim {
        spec["volumeClaimTemplates"] = json!([vc]);
    }

    json!({
        "apiVersion": "apps/v1",
        "kind": "StatefulSet",
        "metadata": cx.meta(p.name, p.component),
        "spec": spec,
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
        let h = headless_service(&cx, "s-h", "server", 7000);
        assert_eq!(h["spec"]["clusterIP"], "None");
        assert_eq!(h["spec"]["publishNotReadyAddresses"], true);
        assert_eq!(
            client_service(&cx, "s", "server", 7000)["spec"]["type"],
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
    fn downward_api_env_keys() {
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
            shard_count: 2,
            replicas_per_shard: 3,
            voter_count: 3,
            headless_env_key: "SVC_HEADLESS_SERVICE",
            cpu: "1",
            memory: "1Gi",
            extra_env: vec![json!({ "name": "EXTRA", "value": "x" })],
            volume_claim: None,
        });
        assert_eq!(ss["spec"]["replicas"], 6); // shard_count * replicas_per_shard
        assert_eq!(ss["spec"]["serviceName"], "s-headless");
        assert_eq!(ss["spec"]["podManagementPolicy"], "Parallel");
        let env = ss["spec"]["template"]["spec"]["containers"][0]["env"]
            .as_array()
            .unwrap();
        let keys: Vec<&str> = env.iter().map(|e| e["name"].as_str().unwrap()).collect();
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
    fn durable_workload_gets_claim_and_mount() {
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

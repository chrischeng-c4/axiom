//! Pure rendering: a [`Keep`] spec → the child Kubernetes objects that realize
//! it. No cluster, no I/O — each object is a self-contained `serde_json::Value`
//! carrying `apiVersion`, `kind`, full `metadata` (labels + owner reference),
//! and `spec`/`data`. This is the operator's source of truth and its primary
//! test surface.
//!
//! keep is always a durable StatefulSet (per-pod WAL + snapshot PVC), so unlike
//! a stateless service there is no Deployment branch — single-node is just
//! `shardCount = replicasPerShard = 1`. The shared [`operator::render`] toolkit
//! supplies the identity, the downward-API StatefulSet (the env
//! `raft_host::cluster::ClusterTopology::from_env` consumes), and the
//! Service/PDB/ServiceAccount shapes; keep adds its ConfigMap, health probes,
//! security hardening, and disk tier on top.
//!
//! @spec projects/keep/tech-design/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md

use serde_json::{json, Value};

use super::crd::Keep;
use operator::render::{self, RenderCtx, ShardedStatefulSet};

const APP: &str = "keep";
const MANAGER: &str = "keep-operator";
const API_VERSION: &str = "keep.dev/v1alpha1";
const KIND: &str = "Keep";
const CLIENT_PORT: i32 = 7117;
const COMPONENT: &str = "server";
const BACKUP_COMPONENT: &str = "backup";

/// Resolve the instance name (defaults to `keep` only when metadata is absent,
/// which never happens for a real CR).
fn instance(keep: &Keep) -> String {
    keep.metadata
        .name
        .clone()
        .unwrap_or_else(|| APP.to_string())
}

/// Resolve the namespace (defaults to `default` for unit construction).
fn namespace(keep: &Keep) -> String {
    keep.metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

/// The owner reference that ties a child to its `Keep` CR (cascading GC).
/// Omitted when the CR has no `uid` (only in unit construction).
fn owner_ref(keep: &Keep) -> Option<Value> {
    let uid = keep.metadata.uid.clone()?;
    let name = keep.metadata.name.clone()?;
    Some(render::owner_ref(API_VERSION, KIND, &name, &uid))
}

/// keep's render identity for the shared [`operator::render`] helpers.
fn ctx<'a>(keep: &Keep, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    RenderCtx {
        app: APP,
        manager: MANAGER,
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns,
        owner: owner_ref(keep),
    }
}

/// Render every child object for `keep`, in dependency order (namespace-scoped
/// config first, then the workload + its Services + PDB).
pub fn render(keep: &Keep) -> Vec<Value> {
    let name = instance(keep);
    let ns = namespace(keep);
    let cx = ctx(keep, &name, &ns);
    let headless = format!("{name}-headless");

    let mut objects = vec![
        render::service_account(&cx, COMPONENT),
        configmap(keep, &cx),
        statefulset(keep, &cx, &headless),
        render::headless_service(&cx, &headless, COMPONENT, CLIENT_PORT),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
        // keep must not lose its sole store pod to a voluntary drain without an
        // operator override: maxUnavailable 0 (mirrors k8s/base/pdb.yaml).
        render::pdb(&cx, &name, COMPONENT, 0),
    ];
    // Optional scheduled backup runner: only when a policy is configured (#776).
    if let Some(cj) = backup_cron_job(keep, &cx) {
        objects.push(cj);
    }
    objects
}

/// The optional backup CronJob (#776): rendered only when `spec.backup` is set.
/// It invokes `keep backup --dest <uri> --data-dir /data --shards <engineShards>
/// [--retention-secs <n>]` on the policy's schedule, so keep still owns
/// producing the consistent snapshot bytes (the runner just schedules it). The
/// shared [`operator::render::cron_job`] helper stays manifest-only.
fn backup_cron_job(keep: &Keep, cx: &RenderCtx) -> Option<Value> {
    let policy = keep.spec.backup.as_ref()?;
    let s = &keep.spec;
    let name = format!("{}-backup", cx.name);
    let mut args = vec![
        "backup".to_string(),
        "--dest".to_string(),
        policy.destination.clone(),
        "--data-dir".to_string(),
        "/data".to_string(),
        "--shards".to_string(),
        s.engine_shards.to_string(),
    ];
    if let Some(secs) = policy.retention_secs {
        args.push("--retention-secs".to_string());
        args.push(secs.to_string());
    }
    Some(render::cron_job(render::CronJob {
        cx,
        name: &name,
        component: BACKUP_COMPONENT,
        schedule: &policy.schedule,
        image: s.cluster.image.as_str(),
        image_pull_policy: s
            .cluster
            .image_pull_policy
            .as_deref()
            .unwrap_or("IfNotPresent"),
        command: vec!["keep".into()],
        args,
        env: vec![],
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

/// keep's serving ConfigMap: the config-driven runtime knobs a pod reads (so a
/// ConfigMap edit can roll pods). Cluster/topology values ride the downward-API
/// env instead.
fn configmap(keep: &Keep, cx: &RenderCtx) -> Value {
    let name = format!("{}-config", cx.name);
    let data = json!({
        "KEEP_PORT": CLIENT_PORT.to_string(),
        "KEEP_SHARDS": keep.spec.engine_shards.to_string(),
        "KEEP_LOG_LEVEL": keep.spec.log_level.clone().unwrap_or_else(|| "info".to_string()),
    });
    json!({
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": cx.meta(&name, COMPONENT),
        "data": data,
    })
}

/// The sharded, durable serving StatefulSet: the toolkit's downward-API base
/// (`replicas = shardCount * replicasPerShard`, the raft-host env quartet + the
/// headless-service env, the `/data` PVC) hardened with keep's probes, security
/// contexts, and writable `/tmp`.
fn statefulset(keep: &Keep, cx: &RenderCtx, headless: &str) -> Value {
    let s = &keep.spec;
    let cfg = format!("{}-config", cx.name);
    let from_cfg = |key: &str| json!({ "name": key, "valueFrom": { "configMapKeyRef": { "name": cfg, "key": key } } });
    let cpu = if s.cluster.resources.cpu.is_empty() {
        "2"
    } else {
        s.cluster.resources.cpu.as_str()
    };
    let memory = if s.cluster.resources.memory.is_empty() {
        "4Gi"
    } else {
        s.cluster.resources.memory.as_str()
    };

    // Per-pod durable disk tier: WAL + snapshots on a ReadWriteOnce PVC, mounted
    // at /data (the helper mounts a `data` claim there).
    let mut pvc = json!({
        "metadata": { "name": "data", "labels": cx.labels(COMPONENT) },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": { "requests": { "storage": s.storage } },
        },
    });
    if let Some(sc) = &s.storage_class {
        pvc["spec"]["storageClassName"] = json!(sc);
    }

    // keep runtime env layered on top of the downward-API quartet the helper
    // injects: bind-all, the /data disk tier, the drain window, the cluster
    // routing fan-out + per-shard replica count, and the config-driven knobs.
    let extra_env = vec![
        json!({ "name": "KEEP_HOST", "value": "0.0.0.0" }),
        json!({ "name": "KEEP_DATA_DIR", "value": "/data" }),
        json!({ "name": "KEEP_GRACE_SECS", "value": s.grace_secs.to_string() }),
        json!({ "name": "KEEP_SHARD_COUNT", "value": s.cluster.shard_count.to_string() }),
        json!({ "name": "KEEP_NODE_COUNT", "value": s.cluster.shard_count.to_string() }),
        json!({ "name": "KEEP_REPLICAS_PER_SHARD", "value": s.cluster.replicas_per_shard.to_string() }),
        from_cfg("KEEP_PORT"),
        from_cfg("KEEP_SHARDS"),
        from_cfg("KEEP_LOG_LEVEL"),
    ];

    let mut sts = render::sharded_statefulset(ShardedStatefulSet {
        cx,
        name: cx.name,
        component: COMPONENT,
        image: s.cluster.image.as_str(),
        image_pull_policy: s
            .cluster
            .image_pull_policy
            .as_deref()
            .unwrap_or("IfNotPresent"),
        command: vec!["keep".into()],
        ports: vec![("http", CLIENT_PORT)],
        headless_service: headless,
        shard_count: s.cluster.shard_count,
        replicas_per_shard: s.cluster.replicas_per_shard,
        voter_count: s.cluster.voter_count,
        headless_env_key: "KEEP_HEADLESS_SERVICE",
        cpu,
        memory,
        extra_env,
        volume_claim: Some(pvc),
    });
    harden(&mut sts, s.grace_secs);
    sts
}

/// Layer keep's production hardening onto the toolkit's base StatefulSet:
/// rolling-update policy, prometheus scrape annotations, non-root pod/container
/// security contexts, health/liveness/startup probes, and a writable `/tmp`
/// (required by `readOnlyRootFilesystem`).
fn harden(sts: &mut Value, grace_secs: u64) {
    if let Some(spec) = sts["spec"].as_object_mut() {
        spec.insert("revisionHistoryLimit".into(), json!(5));
        spec.insert("updateStrategy".into(), json!({ "type": "RollingUpdate" }));
    }
    sts["spec"]["template"]["metadata"]["annotations"] = json!({
        "prometheus.io/scrape": "true",
        "prometheus.io/port": CLIENT_PORT.to_string(),
        "prometheus.io/path": "/metrics",
    });
    if let Some(pod) = sts["spec"]["template"]["spec"].as_object_mut() {
        pod.insert("terminationGracePeriodSeconds".into(), json!(grace_secs));
        pod.insert(
            "securityContext".into(),
            json!({
                "runAsNonRoot": true,
                "runAsUser": 65532, "runAsGroup": 65532, "fsGroup": 65532,
                "seccompProfile": { "type": "RuntimeDefault" },
            }),
        );
        match pod.get_mut("volumes").and_then(|v| v.as_array_mut()) {
            Some(vols) => vols.push(json!({ "name": "tmp", "emptyDir": {} })),
            None => {
                pod.insert("volumes".into(), json!([{ "name": "tmp", "emptyDir": {} }]));
            }
        }
    }
    let container = &mut sts["spec"]["template"]["spec"]["containers"][0];
    container["readinessProbe"] = json!({
        "httpGet": { "path": "/readyz", "port": "http" },
        "initialDelaySeconds": 5, "periodSeconds": 10, "timeoutSeconds": 3, "failureThreshold": 60,
    });
    container["livenessProbe"] = json!({
        "httpGet": { "path": "/healthz", "port": "http" },
        "initialDelaySeconds": 15, "periodSeconds": 30, "timeoutSeconds": 5, "failureThreshold": 3,
    });
    container["startupProbe"] = json!({
        "httpGet": { "path": "/healthz", "port": "http" },
        "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
    });
    container["securityContext"] = json!({
        "runAsNonRoot": true, "runAsUser": 65532, "runAsGroup": 65532,
        "allowPrivilegeEscalation": false,
        "readOnlyRootFilesystem": true,
        "capabilities": { "drop": ["ALL"] },
    });
    match container["volumeMounts"].as_array_mut() {
        Some(mounts) => mounts.push(json!({ "name": "tmp", "mountPath": "/tmp" })),
        None => container["volumeMounts"] = json!([{ "name": "tmp", "mountPath": "/tmp" }]),
    }
}

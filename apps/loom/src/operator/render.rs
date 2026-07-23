//! Pure render: a [`Loom`] spec → the child objects the operator applies. No I/O.
//!
//! loom's control plane is always a raft group (single-node when
//! `replicasPerShard == 1`), so — unlike lumen's stateless-vs-stateful split —
//! there is one shape: a [`operator::render::sharded_statefulset`] carrying the
//! downward-API env `raft_host::ClusterTopology::from_env` reads, plus its
//! headless + client Services, a ServiceAccount, a PDB, and (when a backup
//! schedule is set) a snapshot-upload CronJob.

use service_k8s::render::{self, CronJob, RenderCtx, ShardedStatefulSet};
use serde_json::{json, Value};

use crate::operator::crd::Loom;

const APP: &str = "loom";
const API_VERSION: &str = "loom.dev/v1alpha1";
const KIND: &str = "Loom";
const COMPONENT: &str = "controller";
const PORT: i32 = 7474;

fn instance(loom: &Loom) -> String {
    loom.metadata.name.clone().unwrap_or_else(|| APP.to_string())
}

fn namespace(loom: &Loom) -> String {
    loom.metadata.namespace.clone().unwrap_or_else(|| "default".to_string())
}

/// The render identity for the shared [`operator::render`] helpers, carrying the
/// owner ref (cascading GC) when the live CR has a uid.
fn ctx<'a>(loom: &'a Loom, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    let owner = match (loom.metadata.uid.as_deref(), loom.metadata.name.as_deref()) {
        (Some(uid), Some(cr)) => Some(render::owner_ref(API_VERSION, KIND, cr, uid)),
        _ => None,
    };
    RenderCtx { app: APP, manager: super::MANAGER, api_version: API_VERSION, kind: KIND, name, ns, owner }
}

fn image_pull_policy(loom: &Loom) -> &str {
    loom.spec.image_pull_policy.as_deref().unwrap_or("IfNotPresent")
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2415" reason="logic section in render.rs is hand-written pending codegen support">
/// The loom-specific env appended after the downward-API quartet.
fn extra_env(loom: &Loom, headless: &str) -> Vec<Value> {
    vec![
        json!({ "name": "LOOM_ADDR", "value": format!("0.0.0.0:{PORT}") }),
        json!({ "name": "LOOM_RAFT_DIR", "value": "/data/raft" }),
        json!({ "name": "LOOM_HEADLESS_SERVICE", "value": headless }),
        json!({ "name": "LOOM_DRAIN_GRACE_SECS", "value": "10" }),
        json!({ "name": "LOOM_LOG_FORMAT", "value": "json" }),
        json!({ "name": "LOOM_RELAY", "value": loom.spec.relay }),
        json!({ "name": "LOOM_KEEP", "value": loom.spec.keep }),
        json!({ "name": "LOOM_COMPLETION_SHARDS", "value": loom.spec.completion_shards.to_string() }),
        json!({ "name": "LOOM_GC_RETENTION_SECS", "value": loom.spec.gc_retention_secs.to_string() }),
    ]
}
// </HANDWRITE>

/// The controller StatefulSet: the shared sharded-StatefulSet helper (env
/// quartet + PVC + Parallel) augmented with the archetype probes + hardening.
fn statefulset(loom: &Loom, name: &str, headless: &str) -> Value {
    let volume_claim = json!({
        "metadata": { "name": "data" },
        "spec": {
            "accessModes": ["ReadWriteOnce"],
            "resources": { "requests": { "storage": loom.spec.storage } },
        },
    });
    let mut sts = render::sharded_statefulset(ShardedStatefulSet {
        cx: &ctx(loom, name, &namespace(loom)),
        name,
        component: COMPONENT,
        image: &loom.spec.image,
        image_pull_policy: image_pull_policy(loom),
        command: vec!["loom".into(), "controller".into()],
        ports: vec![("http", PORT)],
        headless_service: headless,
        shard_count: loom.spec.shard_count,
        replicas_per_shard: loom.spec.replicas_per_shard,
        voter_count: loom.spec.voter_count,
        headless_env_key: "LOOM_HEADLESS_SERVICE",
        cpu: &loom.spec.cpu,
        memory: &loom.spec.memory,
        extra_env: extra_env(loom, headless),
        volume_claim: Some(volume_claim),
    });

    // Scrape annotations on the pod template.
    sts["spec"]["template"]["metadata"]["annotations"] = json!({
        "prometheus.io/scrape": "true",
        "prometheus.io/port": PORT.to_string(),
        "prometheus.io/path": "/metrics",
    });
    // Pod-level hardening.
    sts["spec"]["template"]["spec"]["securityContext"] = json!({
        "runAsNonRoot": true,
        "runAsUser": 65532,
        "runAsGroup": 65532,
        "fsGroup": 65532,
        "seccompProfile": { "type": "RuntimeDefault" },
    });
    sts["spec"]["template"]["spec"]["terminationGracePeriodSeconds"] = json!(30);
    // Container probes + hardening (the helper leaves these to the caller).
    let c = &mut sts["spec"]["template"]["spec"]["containers"][0];
    c["readinessProbe"] = json!({
        "httpGet": { "path": "/readyz", "port": "http" },
        "initialDelaySeconds": 5, "periodSeconds": 10, "timeoutSeconds": 3, "failureThreshold": 6,
    });
    c["livenessProbe"] = json!({
        "httpGet": { "path": "/healthz", "port": "http" },
        "initialDelaySeconds": 15, "periodSeconds": 30, "timeoutSeconds": 5, "failureThreshold": 3,
    });
    c["startupProbe"] = json!({
        "httpGet": { "path": "/healthz", "port": "http" },
        "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 60,
    });
    c["securityContext"] = json!({
        "allowPrivilegeEscalation": false,
        "capabilities": { "drop": ["ALL"] },
    });
    sts
}

/// The snapshot-upload CronJob (rendered only when `backupSchedule` is set):
/// runs `loom backup` against node 0's raft snapshot PVC and uploads via
/// `service-backup`. Note the ReadWriteOnce PVC caveat (see HA.md).
fn backup_cronjob(loom: &Loom, name: &str, ns: &str, schedule: &str, dest: &str) -> Value {
    let cx = ctx(loom, name, ns);
    render::cron_job(CronJob {
        cx: &cx,
        name: &format!("{name}-backup"),
        component: "backup",
        schedule,
        image: &loom.spec.image,
        image_pull_policy: image_pull_policy(loom),
        command: vec!["loom".into(), "backup".into()],
        args: vec![
            "--source".into(),
            "/data/raft/runs.snapshot.json".into(),
            "--destination".into(),
            dest.to_string(),
        ],
        env: vec![],
        env_from: vec![],
        volumes: vec![json!({
            "name": "data",
            "persistentVolumeClaim": { "claimName": format!("data-{name}-0"), "readOnly": true },
        })],
        volume_mounts: vec![json!({ "name": "data", "mountPath": "/data", "readOnly": true })],
        service_account_name: Some(name),
        cpu: "100m",
        memory: "128Mi",
        successful_jobs_history_limit: 3,
        failed_jobs_history_limit: 3,
    })
}

/// Render every child object for `loom`, config/identity first then workloads.
pub fn render(loom: &Loom) -> Vec<Value> {
    let name = instance(loom);
    let ns = namespace(loom);
    let headless = format!("{name}-headless");
    let cx = ctx(loom, &name, &ns);

    let mut out = vec![
        render::service_account(&cx, COMPONENT),
        statefulset(loom, &name, &headless),
        render::headless_service(&cx, &headless, COMPONENT, PORT),
        render::client_service(&cx, &name, COMPONENT, PORT),
        render::pdb(&cx, &name, COMPONENT, 1),
    ];
    if let (Some(schedule), Some(dest)) =
        (loom.spec.backup_schedule.as_deref(), loom.spec.backup_destination.as_deref())
    {
        out.push(backup_cronjob(loom, &name, &ns, schedule, dest));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::crd::{Loom, LoomSpec};

    fn loom(replicas_per_shard: u32, backup: bool) -> Loom {
        let mut cr = Loom::new(
            "loom",
            LoomSpec {
                image: "loom:test".into(),
                image_pull_policy: None,
                shard_count: 1,
                replicas_per_shard,
                voter_count: replicas_per_shard,
                relay: "http://relay:7400".into(),
                keep: "http://keep:7117".into(),
                completion_shards: 8,
                gc_retention_secs: 3600,
                storage: "5Gi".into(),
                cpu: "500m".into(),
                memory: "512Mi".into(),
                backup_schedule: backup.then(|| "0 */6 * * *".to_string()),
                backup_destination: backup.then(|| "s3://b/loom".to_string()),
            },
        );
        cr.metadata.namespace = Some("apps".into());
        cr
    }

    fn kinds(objs: &[Value]) -> Vec<String> {
        objs.iter().map(|o| o["kind"].as_str().unwrap_or("").to_string()).collect()
    }

    #[test]
    fn ha_render_has_the_core_objects_and_raft_env() {
        let objs = render(&loom(3, false));
        assert_eq!(
            kinds(&objs),
            ["ServiceAccount", "StatefulSet", "Service", "Service", "PodDisruptionBudget"]
        );
        let sts = &objs[1];
        assert_eq!(sts["spec"]["replicas"], 3);
        assert_eq!(sts["spec"]["serviceName"], "loom-headless");
        assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");
        // The downward-API quartet raft-host reads is present.
        let env = sts["spec"]["template"]["spec"]["containers"][0]["env"]
            .as_array()
            .unwrap();
        let names: Vec<&str> = env.iter().filter_map(|e| e["name"].as_str()).collect();
        for want in ["POD_NAME", "REPLICAS_PER_SHARD", "VOTER_COUNT", "LOOM_HEADLESS_SERVICE"] {
            assert!(names.contains(&want), "missing env {want}");
        }
        // Archetype probes are wired.
        let c = &sts["spec"]["template"]["spec"]["containers"][0];
        assert_eq!(c["readinessProbe"]["httpGet"]["path"], "/readyz");
        assert_eq!(c["livenessProbe"]["httpGet"]["path"], "/healthz");
    }

    #[test]
    fn backup_schedule_adds_a_cronjob() {
        let with = render(&loom(1, true));
        assert!(kinds(&with).contains(&"CronJob".to_string()));
        let without = render(&loom(1, false));
        assert!(!kinds(&without).contains(&"CronJob".to_string()));
    }
}

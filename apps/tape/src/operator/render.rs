// HANDWRITE-BEGIN gap="missing-generator:logic:c41fb0fe" tracker="#1809" reason="Pure render (no I/O), composing shared service_k8s::render::ServiceStatefulSet with Tape-owned image, ports, journal PVC, TAPE_* environment names, auth Secret policy, and typed workload defaults; ServiceAccount, headless/client Services, PDB, and the opt-in spec.backup CronJob remain shared helper outputs; the always-rendered <name>-backup identity is hand-rolled JSON."
//! Pure rendering: a [`Tape`] spec → the child Kubernetes objects that
//! realize it. No cluster, no I/O — each object is a self-contained
//! `serde_json::Value` carrying `apiVersion`, `kind`, full `metadata` (labels
//! and owner reference), and `spec`. This is the operator's source of truth and
//! its primary test surface.
//!
//! tape is always a durable StatefulSet (per-pod journal and raft-state PVC),
//! so there is no Deployment branch — single-node is just
//! `replicasPerShard: 1` (no raft env consumed: `replica_mode()` flips HA only
//! when `REPLICAS_PER_SHARD > 1`). The shared [`service_k8s::render`] toolkit
//! supplies the identity, the downward-API StatefulSet (the env
//! `raft_runtime::cluster::ClusterTopology::from_env` consumes), and the
//! Service/PDB/ServiceAccount shapes; tape adds its runtime env, health
//! probes, security hardening, disk tier, and the opt-in token-registry
//! Secret wiring on top.

use serde_json::{json, Value};

use super::crd::Tape;
use service_k8s::render::{self, RenderCtx, ServiceStatefulSet, WorkloadVolumeClaim};

const APP: &str = "tape";
const MANAGER: &str = "tape-operator";
const API_VERSION: &str = "tape.dev/v1alpha1";
const KIND: &str = "Tape";
/// Public HTTP/1.1 + h2c data/probe port. Raft peers use `RAFT_PORT` when
/// their shared mTLS transport is configured.
const CLIENT_PORT: i32 = 7137;
const RAFT_PORT: i32 = 7138;
const COMPONENT: &str = "server";
/// Component label for the scheduled-backup CronJob (#2574), kept distinct
/// from `server` so its pods are never selected by the serving Services nor
/// counted against the PDB.
const BACKUP_COMPONENT: &str = "backup";
const TOKEN_REGISTRY_VOLUME: &str = "tape-token-registry";
const TOKEN_REGISTRY_KEY: &str = "token-registry.json";
const TOKEN_REGISTRY_MOUNT_DIR: &str = "/var/run/secrets/tape";
const TOKEN_REGISTRY_FILE: &str = "/var/run/secrets/tape/token-registry.json";

/// Resolve the instance name (defaults to `tape` only when metadata is
/// absent, which never happens for a real CR).
fn instance(tape: &Tape) -> String {
    tape.metadata
        .name
        .clone()
        .unwrap_or_else(|| APP.to_string())
}

/// Resolve the namespace (defaults to `default` for unit construction).
fn namespace(tape: &Tape) -> String {
    tape.metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

/// The owner reference that ties a child to its `Tape` CR (cascading GC).
/// Omitted when the CR has no `uid` (only in unit construction).
fn owner_ref(tape: &Tape) -> Option<Value> {
    let uid = tape.metadata.uid.clone()?;
    let name = tape.metadata.name.clone()?;
    Some(render::owner_ref(API_VERSION, KIND, &name, &uid))
}

/// tape's render identity for the shared [`service_k8s::render`] helpers.
fn ctx<'a>(tape: &Tape, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    RenderCtx {
        app: APP,
        manager: MANAGER,
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns,
        owner: owner_ref(tape),
    }
}

/// Which shared projection source (if any) supplies the token registry file.
/// `tokensSecret` wins over `tokensSecretProviderClass`; both are inactive
/// unless the CR enables required bearer auth.
fn token_registry_source(tape: &Tape) -> Option<render::TokenRegistrySource<'_>> {
    if tape.spec.auth != "required" {
        return None;
    }
    if let Some(secret) = tape.spec.tokens_secret.as_deref() {
        return Some(render::TokenRegistrySource::Secret {
            name: secret,
            key: TOKEN_REGISTRY_KEY,
        });
    }
    tape.spec
        .tokens_secret_provider_class
        .as_deref()
        .map(|provider_class| render::TokenRegistrySource::Csi {
            provider_class,
            driver: tape.spec.tokens_secret_csi_driver.as_deref(),
        })
}

// <HANDWRITE gap="missing-generator:kubernetes-peer-service" tracker="#1805" reason="kubernetes-peer-service section in render.rs is hand-written pending codegen support">
/// Render every child object for `tape`, in dependency order (identity first,
/// then the workload + its Services + PDB, then the optional backup CronJob).
pub fn render(tape: &Tape) -> Vec<Value> {
    let name = instance(tape);
    let ns = namespace(tape);
    let cx = ctx(tape, &name, &ns);
    let headless = format!("{name}-headless");

    let mut objects = vec![
        render::service_account(&cx, COMPONENT),
        statefulset(tape, &cx, &headless),
        render::headless_service_with_ports(
            &cx,
            &headless,
            COMPONENT,
            vec![
                json!({ "name": "http", "port": CLIENT_PORT, "targetPort": "http", "protocol": "TCP" }),
                json!({ "name": "raft", "port": RAFT_PORT, "targetPort": "raft", "protocol": "TCP" }),
            ],
        ),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
        // Keep a raft quorum during voluntary disruptions: at most one tape
        // pod may be unavailable at a time.
        render::pdb(&cx, &name, COMPONENT, 1),
        backup_service_account(&cx),
    ];
    if let Some(cron) = backup_cron_job(tape, &cx) {
        objects.push(cron);
    }
    objects
}

/// A stable, per-instance identity for scheduled backup jobs (lumen's #808
/// pattern, adopted for #2574).
///
/// Rendered even when `spec.backup` is unset. The backup runner writes to a
/// cloud object store, so its ServiceAccount is the binding target for cloud
/// IAM — GKE Workload Identity annotates it, and the GCP acceptance harness
/// already pre-creates `<name>-backup` for exactly that
/// (`benchmarks/gcp-operator-acceptance/scripts/render-manifests.sh`). An
/// identity that blinked in and out with the schedule would drop that binding
/// every time the policy was toggled off, so its lifecycle is deliberately
/// decoupled from the policy's. Like every other child it is owned by the
/// `Tape` CR and garbage collected with it; the cloud annotation is set by a
/// different field manager and survives reconcile.
///
/// It is emitted after the PDB so the workload ServiceAccount stays the first
/// `ServiceAccount` in the render order.
fn backup_service_account(cx: &RenderCtx) -> Value {
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": cx.meta(&format!("{}-backup", cx.name), BACKUP_COMPONENT),
    })
}

/// The optional scheduled-backup CronJob (#2574): `tape backup` run on the
/// CR's schedule against this instance's own client Service.
///
/// Returns `None` when `spec.backup` is unset, which is the default — a CR
/// that declares no backup renders exactly the object set it rendered before
/// this field existed.
///
/// The container reuses the instance's image so the backup runner tracks the
/// CR rather than drifting from it, which is the whole reason to render this
/// instead of hand-authoring a CronJob alongside. It runs under the dedicated
/// [`backup_service_account`], not the serving one: only this pod needs cloud
/// object-store credentials.
///
/// Auth: when `adminTokenSecret` is set the token is projected as
/// `TAPE_BACKUP_TOKEN`, the env var `tape backup --token` already falls back
/// to. `/admin/backup` requires `admin` on `*`, so an instance running
/// `auth: required` without this field will render a CronJob whose runs fail
/// 401 — the CR is accepted either way because `auth: off` instances
/// legitimately need no token.
fn backup_cron_job(tape: &Tape, cx: &RenderCtx) -> Option<Value> {
    let backup = tape.spec.backup.as_ref()?;
    let cron_name = format!("{}-backup", cx.name);

    let mut args = vec![
        "backup".to_string(),
        "--url".to_string(),
        format!(
            "http://{}.{}.svc.cluster.local:{CLIENT_PORT}",
            cx.name, cx.ns
        ),
        "--dest".to_string(),
        backup.destination.clone(),
    ];
    if let Some(seconds) = backup.retention_secs {
        args.extend(["--retention-secs".to_string(), seconds.to_string()]);
    }

    let env = match &backup.admin_token_secret {
        Some(secret) => vec![json!({
            "name": "TAPE_BACKUP_TOKEN",
            "valueFrom": { "secretKeyRef": { "name": secret, "key": "token" } },
        })],
        None => vec![],
    };

    Some(render::cron_job(render::CronJob {
        cx,
        name: &cron_name,
        component: BACKUP_COMPONENT,
        schedule: &backup.schedule,
        image: &tape.spec.cluster.image,
        image_pull_policy: tape
            .spec
            .cluster
            .image_pull_policy
            .as_deref()
            .unwrap_or("IfNotPresent"),
        command: vec!["tape".to_string()],
        args,
        env,
        env_from: vec![],
        volumes: vec![],
        volume_mounts: vec![],
        service_account_name: Some(&cron_name),
        cpu: "100m",
        memory: "128Mi",
        successful_jobs_history_limit: 3,
        failed_jobs_history_limit: 3,
    }))
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:kubernetes-peer-workload" tracker="#1805" reason="kubernetes-peer-workload section in render.rs is hand-written pending codegen support">
/// The durable serving StatefulSet: the toolkit's downward-API base
/// (`replicas = replicasPerShard` — `shard_count` PINNED to 1, tape is a
/// single raft group; the raft-runtime env quartet + `TAPE_PEER_SERVICE`; the
/// `/data` PVC) hardened with tape's probes, security contexts, and writable
/// `/tmp`.
fn statefulset(tape: &Tape, cx: &RenderCtx, headless: &str) -> Value {
    let s = &tape.spec;
    // Empty values are resolved by libs/service-k8s to the shared request-only
    // data-plane baseline (1 CPU / 4Gi); tape owns no resource fallback.
    let cpu = s.cluster.resources.cpu.as_str();
    let memory = s.cluster.resources.memory.as_str();

    // Per-pod durable disk tier: ordered journal plus shared Raft hard state,
    // commit watermark, log, and snapshots on one ReadWriteOnce PVC.
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

    // tape runtime env layered on top of the downward-API quartet +
    // TAPE_PEER_SERVICE the helper injects: bind-all on the serve port, the
    // /data disk tier, and the drain window. TAPE_AUTH wiring is opt-in.
    let mut extra_env = vec![
        json!({ "name": "TAPE_BIND", "value": format!("0.0.0.0:{CLIENT_PORT}") }),
        json!({ "name": "TAPE_RAFT_PORT", "value": RAFT_PORT.to_string() }),
        json!({ "name": "TAPE_DATA_DIR", "value": "/data" }),
        json!({ "name": "TAPE_GRACE_SECS", "value": s.grace_secs.to_string() }),
        json!({ "name": "TAPE_LOG_FORMAT", "value": "json" }),
    ];
    if let Some(level) = &s.log_level {
        extra_env.push(json!({ "name": "RUST_LOG", "value": level }));
    }
    if let Some(limit) = s.body_limit_bytes {
        extra_env.push(json!({ "name": "TAPE_BODY_LIMIT_BYTES", "value": limit.to_string() }));
    }
    if token_registry_source(tape).is_some() {
        extra_env.push(json!({ "name": "TAPE_AUTH", "value": "required" }));
        extra_env.push(json!({ "name": "TAPE_TOKEN_REGISTRY_FILE", "value": TOKEN_REGISTRY_FILE }));
    }
    if let Some(seed_uri) = &s.bootstrap_seed_uri {
        extra_env.push(json!({ "name": "TAPE_BOOTSTRAP_SEED_URI", "value": seed_uri }));
    }
    if let Some(topics) = &s.topics {
        if !topics.is_empty() {
            // Compact JSON representation of topic/subscription declarations for the serve path
            let topics_json = serde_json::to_string(topics).expect("topics serialize as JSON");
            extra_env.push(json!({ "name": "TAPE_PROVISION_TOPICS", "value": topics_json }));
        }
    }

    let mut volumes = vec![json!({ "name": "tmp", "emptyDir": {} })];
    let mut volume_mounts = vec![json!({ "name": "tmp", "mountPath": "/tmp" })];
    if let Some(source) = token_registry_source(tape) {
        let projection = render::TokenRegistryProjection {
            volume_name: TOKEN_REGISTRY_VOLUME,
            mount_path: TOKEN_REGISTRY_MOUNT_DIR,
            source,
        };
        volumes.push(render::token_registry_volume(&projection));
        volume_mounts.push(render::token_registry_mount(&projection));
    }

    render::service_statefulset(ServiceStatefulSet {
        cx,
        name: cx.name,
        component: COMPONENT,
        image: s.cluster.image.as_str(),
        image_pull_policy: s
            .cluster
            .image_pull_policy
            .as_deref()
            .unwrap_or("IfNotPresent"),
        command: vec!["tape".into(), "serve".into()],
        args: vec![],
        ports: vec![
            json!({ "name": "http", "containerPort": CLIENT_PORT, "protocol": "TCP" }),
            json!({ "name": "raft", "containerPort": RAFT_PORT, "protocol": "TCP" }),
        ],
        headless_service: headless,
        // tape is a single raft group: shardCount is part of the shared CRD
        // shape but the render pins it to 1 (replicasPerShard is the scale
        // knob; serve's replica_mode() flips HA when it exceeds 1).
        shard_count: 1,
        replicas_per_shard: s.cluster.replicas_per_shard,
        voter_count: s.cluster.voter_count,
        headless_env_key: "TAPE_PEER_SERVICE",
        service_account_name: Some(cx.name),
        env: extra_env,
        env_from: vec![],
        resources: render::requested_resources(cpu, memory),
        pod_annotations: Some(json!({
            "prometheus.io/scrape": "true",
            "prometheus.io/port": CLIENT_PORT.to_string(),
            "prometheus.io/path": "/metrics",
        })),
        pod_security_context: Some(render::restricted_pod_security_context()),
        container_security_context: Some(render::restricted_container_security_context()),
        termination_grace_period_seconds: Some(s.grace_secs),
        readiness_probe: Some(json!({
            "httpGet": { "path": "/readyz", "port": "http" },
            "initialDelaySeconds": 2, "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 60,
        })),
        liveness_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http" },
            "initialDelaySeconds": 5, "periodSeconds": 15, "timeoutSeconds": 5, "failureThreshold": 3,
        })),
        startup_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http" },
            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
        })),
        volumes,
        volume_mounts,
        affinity: Some(render::dedicated_node_affinity(cx.selector(COMPONENT))),
        topology_spread_constraints: vec![],
        revision_history_limit: Some(5),
        update_strategy: Some(json!({ "type": "RollingUpdate" })),
        volume_claim: Some(WorkloadVolumeClaim {
            name: "data".to_owned(),
            template: pvc,
            mount_path: "/data",
            read_only: false,
        }),
    })
}
// HANDWRITE-END

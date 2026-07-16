// HANDWRITE-BEGIN gap="missing-generator:logic:c41fb0fe" tracker="pending-tracker" reason="Pure render (no I/O), everything via the shared service_k8s::render toolkit: RenderCtx (app tape, manager tape-operator, owner_ref from CR uid) -> ServiceAccount, StatefulSet via sharded_statefulset (command [tape, serve], port http 7137, shard_count pinned 1, headless_env_key TAPE_PEER_SERVICE, /data PVC with storage/storageClass, extra_env TAPE_BIND 0.0.0.0:7137 + TAPE_DATA_DIR /data + TAPE_GRACE_SECS + optional RUST_LOG + opt-in TAPE_AUTH/TAPE_TOKEN_REGISTRY_FILE with the token-registry Secret volume mounted read-only at /var/run/secrets/tape, off unless auth: required AND tokensSecret), then harden(): RollingUpdate + revisionHistoryLimit 5 + prometheus annotations + nonroot 65532 pod/container security contexts + readOnlyRootFilesystem + writable /tmp + terminationGracePeriodSeconds = graceSecs + readiness /readyz + liveness/startup /healthz probes; headless + client Services on 7137; PDB maxUnavailable 1."
//! Pure rendering: a [`Tape`] spec → the child Kubernetes objects that
//! realize it. No cluster, no I/O — each object is a self-contained
//! `serde_json::Value` carrying `apiVersion`, `kind`, full `metadata` (labels
//! + owner reference), and `spec`. This is the operator's source of truth and
//! its primary test surface.
//!
//! tape is always a durable StatefulSet (per-pod journal + raft-state PVC),
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
use service_k8s::render::{self, RenderCtx, ShardedStatefulSet};

const APP: &str = "tape";
const MANAGER: &str = "tape-operator";
const API_VERSION: &str = "tape.dev/v1alpha1";
const KIND: &str = "Tape";
/// The one serve port: HTTP/1.1 + h2c, data plane + probes + raft peer RPCs.
const CLIENT_PORT: i32 = 7137;
const COMPONENT: &str = "server";
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

/// Which Secret (if any) supplies the token registry file: only when the CR
/// sets `auth: required` AND names a `tokensSecret` (relay/lumen's pattern —
/// off unless the CR asks; probes stay tokenless either way).
fn token_registry_secret(tape: &Tape) -> Option<&str> {
    if tape.spec.auth != "required" {
        return None;
    }
    tape.spec.tokens_secret.as_deref()
}

// <HANDWRITE gap="missing-generator:kubernetes-peer-service" tracker="pending-tracker" reason="kubernetes-peer-service section in render.rs is hand-written pending codegen support">
/// Render every child object for `tape`, in dependency order (identity first,
/// then the workload + its Services + PDB).
pub fn render(tape: &Tape) -> Vec<Value> {
    let name = instance(tape);
    let ns = namespace(tape);
    let cx = ctx(tape, &name, &ns);
    let headless = format!("{name}-headless");

    vec![
        render::service_account(&cx, COMPONENT),
        statefulset(tape, &cx, &headless),
        render::headless_service(&cx, &headless, COMPONENT, CLIENT_PORT),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
        // Keep a raft quorum during voluntary disruptions: at most one tape
        // pod may be unavailable at a time.
        render::pdb(&cx, &name, COMPONENT, 1),
    ]
}
// </HANDWRITE>

// <HANDWRITE gap="missing-generator:kubernetes-peer-workload" tracker="pending-tracker" reason="kubernetes-peer-workload section in render.rs is hand-written pending codegen support">
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

    // Per-pod durable disk tier: the ordered journal + raft hard state +
    // applied-index marker on a ReadWriteOnce PVC, mounted at /data (the
    // helper mounts a `data` claim).
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
        json!({ "name": "TAPE_DATA_DIR", "value": "/data" }),
        json!({ "name": "TAPE_GRACE_SECS", "value": s.grace_secs.to_string() }),
    ];
    if let Some(level) = &s.log_level {
        extra_env.push(json!({ "name": "RUST_LOG", "value": level }));
    }
    if token_registry_secret(tape).is_some() {
        extra_env.push(json!({ "name": "TAPE_AUTH", "value": "required" }));
        extra_env.push(json!({ "name": "TAPE_TOKEN_REGISTRY_FILE", "value": TOKEN_REGISTRY_FILE }));
    }
    if let Some(seed_uri) = &s.bootstrap_seed_uri {
        extra_env.push(json!({ "name": "TAPE_BOOTSTRAP_SEED_URI", "value": seed_uri }));
    }

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
        command: vec!["tape".into(), "serve".into()],
        ports: vec![("http", CLIENT_PORT)],
        headless_service: headless,
        // tape is a single raft group: shardCount is part of the shared CRD
        // shape but the render pins it to 1 (replicasPerShard is the scale
        // knob; serve's replica_mode() flips HA when it exceeds 1).
        shard_count: 1,
        replicas_per_shard: s.cluster.replicas_per_shard,
        voter_count: s.cluster.voter_count,
        headless_env_key: "TAPE_PEER_SERVICE",
        cpu,
        memory,
        extra_env,
        volume_claim: Some(pvc),
    });
    harden(tape, &mut sts);
    sts
}
// </HANDWRITE>

/// Layer tape's production hardening onto the toolkit's base StatefulSet:
/// rolling-update policy, prometheus scrape annotations, non-root
/// pod/container security contexts, health/liveness/startup probes, a
/// writable `/tmp` (required by `readOnlyRootFilesystem`), and the opt-in
/// token-registry Secret mount.
fn harden(tape: &Tape, sts: &mut Value) {
    if let Some(spec) = sts["spec"].as_object_mut() {
        spec.insert("revisionHistoryLimit".into(), json!(5));
        spec.insert("updateStrategy".into(), json!({ "type": "RollingUpdate" }));
    }
    sts["spec"]["template"]["metadata"]["annotations"] = json!({
        "prometheus.io/scrape": "true",
        "prometheus.io/port": CLIENT_PORT.to_string(),
        "prometheus.io/path": "/metrics",
    });
    let mut volumes = vec![json!({ "name": "tmp", "emptyDir": {} })];
    let mut mounts = vec![json!({ "name": "tmp", "mountPath": "/tmp" })];
    if let Some(secret) = token_registry_secret(tape) {
        volumes.push(json!({
            "name": TOKEN_REGISTRY_VOLUME,
            "secret": {
                "secretName": secret,
                "items": [{ "key": TOKEN_REGISTRY_KEY, "path": TOKEN_REGISTRY_KEY }],
            },
        }));
        mounts.push(json!({
            "name": TOKEN_REGISTRY_VOLUME,
            "mountPath": TOKEN_REGISTRY_MOUNT_DIR,
            "readOnly": true,
        }));
    }
    if let Some(pod) = sts["spec"]["template"]["spec"].as_object_mut() {
        pod.insert(
            "terminationGracePeriodSeconds".into(),
            json!(tape.spec.grace_secs),
        );
        pod.insert(
            "securityContext".into(),
            json!({
                "runAsNonRoot": true,
                "runAsUser": 65532, "runAsGroup": 65532, "fsGroup": 65532,
                "seccompProfile": { "type": "RuntimeDefault" },
            }),
        );
        match pod.get_mut("volumes").and_then(|v| v.as_array_mut()) {
            Some(vols) => vols.extend(volumes),
            None => {
                pod.insert("volumes".into(), json!(volumes));
            }
        }
    }
    let container = &mut sts["spec"]["template"]["spec"]["containers"][0];
    container["readinessProbe"] = json!({
        "httpGet": { "path": "/readyz", "port": "http" },
        "initialDelaySeconds": 2, "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 60,
    });
    container["livenessProbe"] = json!({
        "httpGet": { "path": "/healthz", "port": "http" },
        "initialDelaySeconds": 5, "periodSeconds": 15, "timeoutSeconds": 5, "failureThreshold": 3,
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
        Some(existing) => existing.extend(mounts),
        None => container["volumeMounts"] = json!(mounts),
    }
}
// HANDWRITE-END

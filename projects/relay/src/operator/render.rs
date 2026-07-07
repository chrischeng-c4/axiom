// HANDWRITE-BEGIN gap="missing-generator:logic:67ddc87b" tracker="pending-tracker" reason="Pure render (no I/O), everything via the shared operator::render toolkit: RenderCtx (app relay, manager relay-operator, owner_ref from CR uid) -> ServiceAccount, StatefulSet via sharded_statefulset (command [relay], port http 7000, shard_count pinned 1, headless_env_key RELAY_PEER_SERVICE = {name}-headless, /data PVC with storage/storageClass, extra_env RELAY_BIND 0.0.0.0:7000 + RELAY_DATA_DIR /data + RELAY_GRACE_SECS + optional RUST_LOG + opt-in RELAY_AUTH/RELAY_TOKEN_REGISTRY_FILE with the token-registry Secret volume mounted read-only at /var/run/secrets/relay — lumen's pattern, off unless auth: required AND tokensSecret), then harden(): RollingUpdate + revisionHistoryLimit 5 + prometheus annotations + nonroot 65532 pod/container security contexts + readOnlyRootFilesystem + writable /tmp + terminationGracePeriodSeconds = graceSecs + readiness /readyz + liveness/startup /healthz probes; headless + client Services on 7000; PDB maxUnavailable 1."
//! Pure rendering: a [`Relay`] spec → the child Kubernetes objects that
//! realize it. No cluster, no I/O — each object is a self-contained
//! `serde_json::Value` carrying `apiVersion`, `kind`, full `metadata` (labels
//! + owner reference), and `spec`. This is the operator's source of truth and
//! its primary test surface.
//!
//! relay is always a durable StatefulSet (per-pod durable-log + raft-state
//! PVC), so there is no Deployment branch — single-node is just
//! `replicasPerShard: 1` (no raft env consumed: `replica_mode()` flips HA only
//! when `REPLICAS_PER_SHARD > 1`). The shared [`operator::render`] toolkit
//! supplies the identity, the downward-API StatefulSet (the env
//! `raft_host::cluster::ClusterTopology::from_env` consumes), and the
//! Service/PDB/ServiceAccount shapes; relay adds its runtime env, health
//! probes, security hardening, disk tier, and the opt-in token-registry
//! Secret wiring on top.

use serde_json::{json, Value};

use super::crd::Relay;
use operator::render::{self, RenderCtx, ShardedStatefulSet};

const APP: &str = "relay";
const MANAGER: &str = "relay-operator";
const API_VERSION: &str = "relay.dev/v1alpha1";
const KIND: &str = "Relay";
/// The one serve port: HTTP/1.1 + h2c, data plane + probes + raft peer RPCs.
const CLIENT_PORT: i32 = 7000;
const COMPONENT: &str = "server";
const TOKEN_REGISTRY_VOLUME: &str = "relay-token-registry";
const TOKEN_REGISTRY_KEY: &str = "token-registry.json";
const TOKEN_REGISTRY_MOUNT_DIR: &str = "/var/run/secrets/relay";
const TOKEN_REGISTRY_FILE: &str = "/var/run/secrets/relay/token-registry.json";

/// Resolve the instance name (defaults to `relay` only when metadata is
/// absent, which never happens for a real CR).
fn instance(relay: &Relay) -> String {
    relay
        .metadata
        .name
        .clone()
        .unwrap_or_else(|| APP.to_string())
}

/// Resolve the namespace (defaults to `default` for unit construction).
fn namespace(relay: &Relay) -> String {
    relay
        .metadata
        .namespace
        .clone()
        .unwrap_or_else(|| "default".to_string())
}

/// The owner reference that ties a child to its `Relay` CR (cascading GC).
/// Omitted when the CR has no `uid` (only in unit construction).
fn owner_ref(relay: &Relay) -> Option<Value> {
    let uid = relay.metadata.uid.clone()?;
    let name = relay.metadata.name.clone()?;
    Some(render::owner_ref(API_VERSION, KIND, &name, &uid))
}

/// relay's render identity for the shared [`operator::render`] helpers.
fn ctx<'a>(relay: &Relay, name: &'a str, ns: &'a str) -> RenderCtx<'a> {
    RenderCtx {
        app: APP,
        manager: MANAGER,
        api_version: API_VERSION,
        kind: KIND,
        name,
        ns,
        owner: owner_ref(relay),
    }
}

/// Which Secret (if any) supplies the token registry file: only when the CR
/// sets `auth: required` AND names a `tokensSecret` (lumen's pattern — off
/// unless the CR asks; probes stay tokenless either way).
fn token_registry_secret(relay: &Relay) -> Option<&str> {
    if relay.spec.auth != "required" {
        return None;
    }
    relay.spec.tokens_secret.as_deref()
}

/// Render every child object for `relay`, in dependency order (identity first,
/// then the workload + its Services + PDB).
pub fn render(relay: &Relay) -> Vec<Value> {
    let name = instance(relay);
    let ns = namespace(relay);
    let cx = ctx(relay, &name, &ns);
    let headless = format!("{name}-headless");

    vec![
        render::service_account(&cx, COMPONENT),
        statefulset(relay, &cx, &headless),
        render::headless_service(&cx, &headless, COMPONENT, CLIENT_PORT),
        render::client_service(&cx, &name, COMPONENT, CLIENT_PORT),
        // Keep a raft quorum during voluntary disruptions: at most one broker
        // pod may be unavailable at a time (mirrors the old k8s/pdb.yaml).
        render::pdb(&cx, &name, COMPONENT, 1),
    ]
}

/// The durable serving StatefulSet: the toolkit's downward-API base
/// (`replicas = replicasPerShard` — `shard_count` PINNED to 1, relay is a
/// single raft group; the raft-host env quartet + `RELAY_PEER_SERVICE`; the
/// `/data` PVC) hardened with relay's probes, security contexts, and writable
/// `/tmp`.
fn statefulset(relay: &Relay, cx: &RenderCtx, headless: &str) -> Value {
    let s = &relay.spec;
    let cpu = if s.cluster.resources.cpu.is_empty() {
        "1"
    } else {
        s.cluster.resources.cpu.as_str()
    };
    let memory = if s.cluster.resources.memory.is_empty() {
        "1Gi"
    } else {
        s.cluster.resources.memory.as_str()
    };

    // Per-pod durable disk tier: the ordered log + raft hard state on a
    // ReadWriteOnce PVC, mounted at /data (the helper mounts a `data` claim).
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

    // relay runtime env layered on top of the downward-API quartet +
    // RELAY_PEER_SERVICE the helper injects: bind-all on the serve port, the
    // /data disk tier, and the drain window. RELAY_AUTH wiring is opt-in.
    let mut extra_env = vec![
        json!({ "name": "RELAY_BIND", "value": format!("0.0.0.0:{CLIENT_PORT}") }),
        json!({ "name": "RELAY_DATA_DIR", "value": "/data" }),
        json!({ "name": "RELAY_GRACE_SECS", "value": s.grace_secs.to_string() }),
    ];
    if let Some(level) = &s.log_level {
        extra_env.push(json!({ "name": "RUST_LOG", "value": level }));
    }
    if token_registry_secret(relay).is_some() {
        extra_env.push(json!({ "name": "RELAY_AUTH", "value": "required" }));
        extra_env.push(json!({ "name": "RELAY_TOKEN_REGISTRY_FILE", "value": TOKEN_REGISTRY_FILE }));
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
        command: vec!["relay".into()],
        ports: vec![("http", CLIENT_PORT)],
        headless_service: headless,
        // relay is a single raft group: shardCount is part of the shared CRD
        // shape but the render pins it to 1 (replicasPerShard is the scale
        // knob; serve's replica_mode() flips HA when it exceeds 1).
        shard_count: 1,
        replicas_per_shard: s.cluster.replicas_per_shard,
        voter_count: s.cluster.voter_count,
        headless_env_key: "RELAY_PEER_SERVICE",
        cpu,
        memory,
        extra_env,
        volume_claim: Some(pvc),
    });
    harden(relay, &mut sts);
    sts
}

/// Layer relay's production hardening onto the toolkit's base StatefulSet:
/// rolling-update policy, prometheus scrape annotations, non-root
/// pod/container security contexts, health/liveness/startup probes, a
/// writable `/tmp` (required by `readOnlyRootFilesystem`), and the opt-in
/// token-registry Secret mount.
fn harden(relay: &Relay, sts: &mut Value) {
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
    if let Some(secret) = token_registry_secret(relay) {
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
            json!(relay.spec.grace_secs),
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

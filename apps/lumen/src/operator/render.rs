// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#rust-source-unit
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
//! headless Service, a ClusterIP Service, ConfigMap, PDB, serving
//! ServiceAccount, and a dedicated backup ServiceAccount. The backup identity
//! is intentionally cloud-neutral: deployment harnesses may annotate it for
//! Workload Identity without giving object-storage credentials to serving
//! pods.
//! Stateful data pods are not a direct HPA target. The reconcile loop in [`super::reconcile`]
//! server-side-applies whatever this returns.

use serde_json::{json, Value};

use super::crd::{AuthMode, Lumen};
use service_k8s::render::{
    self, projected_token::ProjectedServiceAccountToken, rbac, RenderCtx, ServiceStatefulSet,
    WorkloadVolumeClaim,
};
use service_k8s::service::PruneTarget;

const APP: &str = "lumen";
const MANAGER: &str = "lumen-operator";
const API_VERSION: &str = "lumen.dev/v1alpha1";
const KIND: &str = "Lumen";
const COMPONENT: &str = "server";
const CLIENT_PORT: i32 = 7373;
const RAFT_PORT: i32 = 7374;
const BACKUP_COMPONENT: &str = "backup";
/// Component label for the cluster-scoped auth-delegation binding (#2876). Its
/// own value, not `server`: the sweep in [`super::reconcile`] selects on it,
/// and sharing a component with the namespaced serving children would make
/// that selector match objects the sweep has no business deleting.
const AUTH_DELEGATION_COMPONENT: &str = "auth-delegation";
/// The built-in ClusterRole granting `create` on `tokenreviews` and
/// `subjectaccessreviews` — the two APIs delegated request auth is made of.
/// Kubernetes ships and maintains it; lumen binds it rather than copying it.
const AUTH_DELEGATOR_ROLE: &str = "system:auth-delegator";
/// Which namespace a cluster-scoped child belongs to (#2876). The recommended
/// label set has no equivalent, and the object has no `metadata.namespace` of
/// its own to read.
const OWNER_NAMESPACE_LABEL: &str = "lumen.dev/owner-namespace";
const HEADLESS_ENV_KEY: &str = "LUMEN_HEADLESS_SERVICE";
// #1387: embedded-mode persistence subtree, disjoint from the raft backend's
// `/var/lib/lumen/raft` default (`LUMEN_RAFT_DATA_DIR` in `bin/lumen.rs`) so
// both can coexist on the one `raft` PVC mount across a `replicasPerShard`
// change without colliding.
const EMBEDDED_DATA_DIR: &str = "/var/lib/lumen/data";
/// Where `spec.peerTlsSecret` is projected into every Raft member (#2890).
/// Lumen-specific and disjoint from `/var/lib/lumen`: peer identity is
/// read-only credential material, not index state, and must not land on the
/// PVC that survives a pod.
const PEER_TLS_MOUNT_PATH: &str = "/var/run/secrets/lumen-peer";
/// The pod-local volume carrying it.
const PEER_TLS_VOLUME: &str = "lumen-peer-tls";
/// The three keys the Secret must carry — the same contract Relay and Defer
/// project, and exactly what `peer_tls::PeerTlsConfig` loads.
pub const PEER_TLS_KEYS: [&str; 3] = ["tls.crt", "tls.key", "ca.crt"];
/// Where `spec.servingTlsSecret` is projected (#3113 R2). A separate path from
/// [`PEER_TLS_MOUNT_PATH`] because the two identities are separate: one mount
/// per listener means a misconfiguration points a listener at nothing rather
/// than at the other listener's key.
const SERVING_TLS_MOUNT_PATH: &str = "/var/run/secrets/lumen-serving";
/// The pod-local volume carrying it.
const SERVING_TLS_VOLUME: &str = "lumen-serving-tls";
/// The three keys the serving Secret must carry — same shape as the peer
/// Secret, different subject.
pub const SERVING_TLS_KEYS: [&str; 3] = ["tls.crt", "tls.key", "ca.crt"];
/// The Kubernetes Service DNS names the serving leaf answers to (#3113 R2).
///
/// Both forms, because both are real: in-cluster callers resolve the short
/// `<service>.<namespace>.svc` and `lumen connect` addresses the fully
/// qualified one, and a certificate carrying only one of them fails hostname
/// verification for half its callers. Kept identical to what
/// [`super::certificate::serving_profile`] requests — the names the operator
/// asks for and the names the pod is told to expect are one list, read twice.
pub fn serving_dns_names(lumen: &Lumen) -> Vec<String> {
    let (name, ns) = (instance(lumen), namespace(lumen));
    vec![
        format!("{name}.{ns}.svc"),
        format!("{name}.{ns}.svc.cluster.local"),
    ]
}

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

/// lumen's render identity for the shared [`service_k8s::render`] helpers.
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

/// Stateful data pods are never a direct HPA target. A vanilla HPA changes
/// total pods, cannot preserve whole per-shard replica layers, and cannot
/// perform the Raft membership transition required before a replica delta.
/// The retained handoff loop consults this function to prune HPAs emitted by
/// older Lumen versions for every topology.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub(crate) fn wants_hpa(_lumen: &Lumen) -> bool {
    false
}

/// The exact labels older Lumen versions stamped on their rendered HPA object
/// (mirrors [`service_k8s::render::RenderCtx::labels`]'s five recommended
/// labels). Exposed crate-private so `super::reconcile`'s HPA handoff loop
/// (#1385, R2) can confirm a live HPA found at this CR's name was actually
/// rendered by lumen — not a user-created object with a coincidentally
/// matching name — before deleting it.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub(crate) fn hpa_labels(lumen: &Lumen) -> std::collections::BTreeMap<String, String> {
    let mut labels = std::collections::BTreeMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), APP.to_string());
    labels.insert("app.kubernetes.io/instance".to_string(), instance(lumen));
    labels.insert(
        "app.kubernetes.io/component".to_string(),
        COMPONENT.to_string(),
    );
    labels.insert(
        "app.kubernetes.io/managed-by".to_string(),
        MANAGER.to_string(),
    );
    labels.insert("app.kubernetes.io/part-of".to_string(), APP.to_string());
    labels
}

/// Render every child object for `lumen`, in dependency order (namespace-scoped
/// config first, then workloads).
///
/// The serving fleet is always a StatefulSet — with its durable
/// `volumeClaimTemplates`-backed `raft` PVC and headless Service — regardless
/// of `replicasPerShard`. No topology renders a direct HPA: single-member
/// scale-out would create uncoordinated copies, while raft-HA needs a
/// membership-aware whole-layer transition before changing pod count.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub fn render(lumen: &Lumen) -> Vec<Value> {
    let name = instance(lumen);
    let ns = namespace(lumen);
    let cx = ctx(lumen, &name, &ns);
    let headless = format!("{name}-headless");
    let mut out = Vec::new();
    // Skip rendering the workload ServiceAccount entirely when the deployer
    // points at a pre-existing, externally-managed one (#2497): the operator
    // must never create, own, or delete an SA it doesn't render.
    if lumen.spec.service_account_name.is_none() {
        let mut sa = render::service_account(&cx, COMPONENT);
        attach_service_account_annotations(&mut sa, &lumen.spec.service_account_annotations);
        out.push(sa);
    }
    let mut bsa = backup_service_account(&cx);
    attach_service_account_annotations(&mut bsa, &lumen.spec.service_account_annotations);
    out.push(bsa);
    out.push(serving_configmap(lumen, &cx));
    out.extend([
        serving_statefulset(lumen, &cx, &headless),
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
    ]);
    out.push(render::pdb(&cx, &name, COMPONENT, 1));
    if lumen.spec.network_policy {
        out.push(serving_network_policy(&cx, &name));
    }
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

/// Children a previous spec rendered that this one no longer wants (#2603).
///
/// Only the NetworkPolicy qualifies today, and it qualifies because it is the
/// one conditional child whose *presence* changes runtime behavior rather than
/// just adding an object. Leaving a stale ServiceMonitor around scrapes a
/// metric nobody reads; leaving a stale NetworkPolicy around keeps dropping
/// traffic the spec has stopped asking to drop. Flipping `networkPolicy` to
/// `false` therefore has to actively remove it, or the field is opt-in only.
///
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub fn prunes(lumen: &Lumen) -> Vec<PruneTarget> {
    if lumen.spec.network_policy {
        return Vec::new();
    }
    vec![PruneTarget {
        api_version: "networking.k8s.io/v1",
        kind: "NetworkPolicy",
        // Resolved through the same `instance` helper `render` uses, so this is
        // the exact inverse of the branch that creates it rather than an
        // independent guess at the name.
        name: instance(lumen),
    }]
}

/// The ServiceAccount the serving pods actually run as (#2497, #2876).
///
/// Two callers depend on this being one answer: [`serving_statefulset`] puts it
/// in the pod spec, and [`auth_delegator_binding`] grants it delegated review.
/// If they resolved it separately, a spec that names an external SA would run
/// pods as one identity and authorize a different one — and the symptom would
/// be every request failing authentication, not an obviously wrong manifest.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub(crate) fn serving_service_account_name(lumen: &Lumen) -> String {
    lumen
        .spec
        .service_account_name
        .clone()
        .unwrap_or_else(|| instance(lumen))
}

/// The cluster-scoped ClusterRoleBinding's name for this instance (#2876).
///
/// Dots, not dashes, join the two components. A cluster-scoped name has no
/// namespace to disambiguate it, so `lumen-<ns>-<name>-…` would map
/// `(a-b, c)` and `(a, b-c)` to one object — two Lumens in different
/// namespaces silently sharing a binding, each granting the other's
/// ServiceAccount delegated review. A namespace is a DNS-1123 *label* and
/// cannot contain a dot, so splitting at the first dot recovers the namespace
/// exactly and the mapping is injective.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub fn auth_delegator_binding_name(lumen: &Lumen) -> String {
    format!(
        "lumen.{}.{}.auth-delegator",
        namespace(lumen),
        instance(lumen)
    )
}

/// The exact labels [`auth_delegator_binding`] stamps.
///
/// These are load-bearing, not decoration. A cluster-scoped object cannot be
/// owned by a namespaced CR (see [`service_k8s::render::rbac`]), so labels are
/// the only link back to the instance — and the only thing the cleanup sweep
/// in [`super::reconcile`] can use to prove lumen rendered a binding before
/// deleting it. `lumen.dev/owner-namespace` exists because the recommended
/// label set has no way to say which namespace an object belongs *to* when the
/// object itself has none.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub fn auth_delegator_labels(lumen: &Lumen) -> std::collections::BTreeMap<String, String> {
    let mut labels = std::collections::BTreeMap::new();
    labels.insert("app.kubernetes.io/name".to_string(), APP.to_string());
    labels.insert("app.kubernetes.io/instance".to_string(), instance(lumen));
    labels.insert(
        "app.kubernetes.io/component".to_string(),
        AUTH_DELEGATION_COMPONENT.to_string(),
    );
    labels.insert(
        "app.kubernetes.io/managed-by".to_string(),
        MANAGER.to_string(),
    );
    labels.insert("app.kubernetes.io/part-of".to_string(), APP.to_string());
    labels.insert(OWNER_NAMESPACE_LABEL.to_string(), namespace(lumen));
    labels
}

/// The ClusterRoleBinding that lets the serving ServiceAccount ask the API
/// server to validate a caller's token and authorize the request (#2876).
///
/// Lumen delegates both halves of request auth: `TokenReview` decides who the
/// caller is, `SubjectAccessReview` decides what they may do. Both are
/// cluster-scoped review APIs, so no namespaced RoleBinding can grant them —
/// this has to be a ClusterRoleBinding or the serving process cannot
/// authenticate anyone.
///
/// It binds the built-in `system:auth-delegator`. Rendering a replacement
/// ClusterRole would mean maintaining a private copy of a grant Kubernetes
/// already maintains, and every future upstream change to it would be a
/// divergence nobody is watching for.
///
/// Exactly one subject, always: the resolved serving ServiceAccount. Not
/// `system:authenticated`, not the namespace's ServiceAccount group, not the
/// operator's own identity, not the backup runner's, not a client's. Each of
/// those would hand delegated authentication review to a population rather
/// than to a process.
///
/// This is deliberately *not* part of [`render`]. Everything that function
/// returns is applied into the CR's namespace and owned by the CR; this object
/// is neither, and mixing it in would either be applied to a namespaced
/// endpoint that rejects it or stamped with an owner reference that gets it
/// garbage collected. [`super::reconcile`] applies it on its own path and
/// sweeps it on its own path.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub fn auth_delegator_binding(lumen: &Lumen) -> Value {
    let sa = serving_service_account_name(lumen);
    let ns = namespace(lumen);
    let subjects = [rbac::ServiceAccountSubject {
        namespace: &ns,
        name: &sa,
    }];
    rbac::cluster_role_binding(rbac::ClusterRoleBinding {
        name: &auth_delegator_binding_name(lumen),
        labels: serde_json::to_value(auth_delegator_labels(lumen)).unwrap_or_else(|_| json!({})),
        cluster_role: AUTH_DELEGATOR_ROLE,
        subjects: &subjects,
    })
}

/// The optional per-instance NetworkPolicy (#2603), rendered only when
/// `spec.networkPolicy` is set.
///
/// Lumen's two ports have genuinely different audiences: `7373` is the search
/// API any workload may call, `7374` carries Raft — append entries, vote
/// requests, snapshot transfer — and must be reachable only from this
/// instance's own pods. The shared helper expresses exactly that split, so the
/// isolation posture is one contract across every service that adopts it
/// rather than six hand-written policies that drift.
///
/// The backup CronJob is deliberately *not* selected: it runs under its own
/// `<instance>-backup` component label, calls the client Service like any other
/// in-cluster client, and needs egress to object storage — the serving pods'
/// posture would be wrong for it in both directions.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
fn serving_network_policy(cx: &RenderCtx<'_>, name: &str) -> Value {
    render::common::network_policy(render::common::NetworkPolicy {
        cx,
        name,
        component: COMPONENT,
        client_ports: vec![CLIENT_PORT],
        peer_ports: vec![RAFT_PORT],
        // Lumen's operator path never configures the NATS WAL relay; backups
        // reach object storage over TLS, which the shared baseline already
        // allows.
        extra_egress: vec![],
    })
}

fn attach_service_account_annotations(
    sa: &mut Value,
    annotations: &std::collections::BTreeMap<String, String>,
) {
    if annotations.is_empty() {
        return;
    }
    if let Some(meta) = sa.get_mut("metadata").and_then(|m| m.as_object_mut()) {
        if let Some(existing) = meta.get_mut("annotations").and_then(|a| a.as_object_mut()) {
            for (k, v) in annotations {
                existing.insert(k.clone(), Value::String(v.clone()));
            }
        } else {
            meta.insert(
                "annotations".to_string(),
                serde_json::to_value(annotations).unwrap(),
            );
        }
    }
}

/// A stable, per-instance identity for scheduled backup jobs.
///
/// It is rendered even when no backup schedule is currently configured. That
/// keeps its lifecycle declarative across policy toggles and gives platform
/// automation a stable cloud-neutral target for Workload Identity annotations.
/// Like every other child, it is owned by the `Lumen` CR and is garbage
/// collected with the instance.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
fn backup_service_account(cx: &RenderCtx<'_>) -> Value {
    let name = format!("{}-backup", cx.name);
    json!({
        "apiVersion": "v1",
        "kind": "ServiceAccount",
        "metadata": cx.meta(&name, BACKUP_COMPONENT),
    })
}

/// The credential a Lumen control-plane workload presents to a serving
/// instance (#2877): the operator's reshard driver and the backup runner.
///
/// One definition, two consumers, and the mount path is the same constant
/// [`crate::auth::control_plane_token_file`] reads — a renderer that invented
/// its own path would produce a pod with a token mounted somewhere the client
/// never looks, and the symptom would be an authentication failure rather than
/// a missing file.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
pub(crate) fn control_plane_token() -> ProjectedServiceAccountToken<'static> {
    ProjectedServiceAccountToken::new(
        crate::auth::CONTROL_PLANE_TOKEN_VOLUME,
        crate::auth::CONTROL_PLANE_TOKEN_MOUNT,
        crate::auth::AUDIENCE,
    )
}

/// The optional backup CronJob (#808): rendered only when
/// `spec.serving.backup` is set. Lumen already produces a consistent
/// point-in-time snapshot over HTTP (`GET /admin/backup`, see
/// `apps/lumen/src/api.rs`); this CronJob adds nothing new to the
/// WAL/snapshot path, it only *schedules and transports* that existing
/// endpoint's bytes to a destination via `lumen backup`
/// (`libs/service-backup`). The shared [`service_k8s::render::cron_job`] helper
/// stays manifest-only.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-render-rs.md#source
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
    // The runner's own credential (#2877): a token minted for the backup
    // ServiceAccount, bound to Lumen's audience, expiring in ten minutes, and
    // rotated in place by the kubelet. The projection itself is
    // unconditional — one pod shape whatever `spec.auth` says — because a
    // mounted file nobody reads costs nothing, while a manifest that changes
    // shape with an auth toggle is a second thing to get wrong.
    //
    // Presenting it is conditional. A fleet with `auth: disabled` rejects a
    // *presented* bearer (#2871), so the flag that makes the runner read the
    // file only appears when the fleet actually requires an identity.
    //
    // What travels on the CronJob is the path, not the token: the material
    // never appears in the pod spec, in `kubectl describe`, or in whatever
    // pipeline ships that manifest to the cluster.
    let projection = control_plane_token();
    if matches!(lumen.spec.auth, AuthMode::Required) {
        args.push("--token-file".to_string());
        args.push(projection.file_path());
    }
    let env: Vec<serde_json::Value> = Vec::new();
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
        volumes: vec![projection.volume()],
        volume_mounts: vec![projection.mount()],
        service_account_name: Some(&cron_name),
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
    let sa_name = serving_service_account_name(lumen);
    let res = render::requested_resources(&s.cpu, &s.memory);
    let mut volume_mounts = vec![json!({ "name": "tmp", "mountPath": "/tmp" })];
    let mut volumes = vec![json!({ "name": "tmp", "emptyDir": {} })];
    // #2890 R2: the instance's Raft identity, projected read-only. `items`
    // rather than a whole-Secret mount so the three keys the peer transport
    // loads are the three keys that reach the pod — an extra key added to the
    // Secret later cannot silently become part of what the container sees.
    if let Some(secret) = lumen.spec.peer_tls_secret.as_deref() {
        volumes.push(json!({
            "name": PEER_TLS_VOLUME,
            "secret": {
                "secretName": secret,
                "items": PEER_TLS_KEYS
                    .iter()
                    .map(|key| json!({ "key": key, "path": key }))
                    .collect::<Vec<_>>(),
            },
        }));
        volume_mounts.push(json!({
            "name": PEER_TLS_VOLUME,
            "mountPath": PEER_TLS_MOUNT_PATH,
            "readOnly": true,
        }));
    }
    // #3113 R2: the serving leaf, projected the same way and to its own path.
    // Kubernetes refreshes a projected Secret in place, so a renewed leaf
    // reaches the container without a new pod — which is what makes R9's
    // "no Pod rollout" a property of the projection rather than of the
    // controller's restraint.
    if let Some(secret) = lumen.spec.serving_tls_secret.as_deref() {
        volumes.push(json!({
            "name": SERVING_TLS_VOLUME,
            "secret": {
                "secretName": secret,
                "items": SERVING_TLS_KEYS
                    .iter()
                    .map(|key| json!({ "key": key, "path": key }))
                    .collect::<Vec<_>>(),
            },
        }));
        volume_mounts.push(json!({
            "name": SERVING_TLS_VOLUME,
            "mountPath": SERVING_TLS_MOUNT_PATH,
            "readOnly": true,
        }));
    }
    // Probes follow the port. A kubelet that spoke cleartext to a TLS listener
    // would read every failed handshake as an unhealthy pod and restart a
    // container that was serving correctly (#3113 R2/AC2).
    let probe_scheme = if lumen.spec.serving_tls_secret.is_some() {
        "HTTPS"
    } else {
        "HTTP"
    };
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
        ports: vec![
            json!({ "name": "http", "containerPort": CLIENT_PORT, "protocol": "TCP" }),
            json!({ "name": "raft", "containerPort": RAFT_PORT, "protocol": "TCP" }),
        ],
        headless_service: headless,
        shard_count: lumen.spec.shard_count,
        replicas_per_shard: lumen.spec.replicas_per_shard,
        voter_count: lumen.spec.voter_count,
        headless_env_key: HEADLESS_ENV_KEY,
        // External SA name wins when configured (#2497); default to the
        // operator-owned per-instance SA when unset. Resolved through the same
        // helper the auth-delegator binding uses, so the identity the pods run
        // as and the identity that is granted delegated review cannot diverge.
        service_account_name: Some(&sa_name),
        env: serving_env(lumen),
        env_from: vec![],
        resources: res,
        pod_annotations: Some(json!({
            "prometheus.io/scrape": "true",
            "prometheus.io/port": CLIENT_PORT.to_string(),
            "prometheus.io/path": "/metrics",
        })),
        pod_security_context: Some(render::restricted_pod_security_context()),
        container_security_context: Some(render::restricted_container_security_context()),
        termination_grace_period_seconds: Some(s.grace_secs),
        readiness_probe: Some(json!({
            "httpGet": { "path": "/readyz", "port": "http", "scheme": probe_scheme },
            "initialDelaySeconds": 5, "periodSeconds": 10,
            "timeoutSeconds": 3, "failureThreshold": 60,
        })),
        liveness_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http", "scheme": probe_scheme },
            "initialDelaySeconds": 15, "periodSeconds": 30,
            "timeoutSeconds": 5, "failureThreshold": 3,
        })),
        startup_probe: Some(json!({
            "httpGet": { "path": "/healthz", "port": "http", "scheme": probe_scheme },
            "periodSeconds": 5, "timeoutSeconds": 3, "failureThreshold": 120,
        })),
        lifecycle: None,
        volumes,
        volume_mounts,
        affinity: Some(render::dedicated_node_affinity(cx.selector(COMPONENT))),
        // `spec.placement` names the node pool; the anti-affinity above stays
        // operator-owned so asking for a pool can never cost the constraint
        // that keeps two replicas of a shard off one host.
        node_selector: (!lumen.spec.placement.node_selector.is_empty())
            .then(|| json!(lumen.spec.placement.node_selector)),
        tolerations: lumen
            .spec
            .placement
            .tolerations
            .iter()
            .map(|t| json!(t))
            .collect(),
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
            // `shard_count > 1` at `replicasPerShard <= 1` is the routed
            // serving topology (#1398): each pod still needs its own stable
            // headless DNS name to forward cross-shard requests one hop to
            // the owning pod (`lumen::routing::shard_host`), so
            // `HEADLESS_ENV_KEY` is only stripped alongside the raft peer
            // env when there is truly one physical shard and nothing to
            // route to.
            let strip_headless = lumen.spec.shard_count <= 1;
            env.retain(|value| {
                let Some(name) = value["name"].as_str() else {
                    return true;
                };
                if name == HEADLESS_ENV_KEY {
                    return !strip_headless;
                }
                !matches!(name, "REPLICAS_PER_SHARD" | "VOTER_COUNT")
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
    // #2890 R2: the four env vars `lumen::tls::PeerTlsConfig::from_env` reads,
    // pointing at the projected Secret. `LUMEN_PEER_MTLS=on` is what makes the
    // peer listener *require* a client certificate rather than merely offer
    // TLS, so it is set alongside the paths and never on its own.
    if lumen.spec.peer_tls_secret.is_some() {
        env.push(json!({ "name": "LUMEN_PEER_MTLS", "value": "on" }));
        env.push(json!({ "name": "LUMEN_PEER_TLS_CERT", "value": format!("{PEER_TLS_MOUNT_PATH}/tls.crt") }));
        env.push(json!({ "name": "LUMEN_PEER_TLS_KEY", "value": format!("{PEER_TLS_MOUNT_PATH}/tls.key") }));
        env.push(json!({ "name": "LUMEN_PEER_TLS_CA", "value": format!("{PEER_TLS_MOUNT_PATH}/ca.crt") }));
    }
    // #3113 R1: the serving listener's own four. `LUMEN_TLS=on` is what turns
    // the client port from h2c into a TLS listener that refuses rather than
    // downgrades; the paths alone would leave it cleartext, and the flag alone
    // would leave it with nothing to serve, so all four move together.
    if lumen.spec.serving_tls_secret.is_some() {
        env.push(json!({ "name": "LUMEN_TLS", "value": "on" }));
        env.push(json!({ "name": "LUMEN_TLS_CERT", "value": format!("{SERVING_TLS_MOUNT_PATH}/tls.crt") }));
        env.push(json!({ "name": "LUMEN_TLS_KEY", "value": format!("{SERVING_TLS_MOUNT_PATH}/tls.key") }));
        env.push(json!({ "name": "LUMEN_TLS_CA", "value": format!("{SERVING_TLS_MOUNT_PATH}/ca.crt") }));
        // The names the leaf must answer to, from the operator that asked for
        // it — the pod has no other way to learn which Service it fronts, and
        // guessing from its own hostname would accept a certificate issued for
        // a different Service in the same namespace.
        env.push(json!({
            "name": "LUMEN_TLS_SERVER_NAMES",
            "value": serving_dns_names(lumen).join(","),
        }));
    }
    // #1387: `LUMEN_WAL=auto` above resolves to `Embedded` (`MemWal::new()`,
    // RAM-only) whenever `resolve_wal_backend` sees no raft cluster context —
    // exactly the `replicasPerShard <= 1` regime (its raft peer-identity env
    // is stripped in `serving_statefulset` below). Without `LUMEN_DATA_DIR`
    // that mode never touches the already-mounted `raft` PVC, so a pod
    // restart — including the reshard cutover's own rolling restart — wipes
    // all data despite the volume being durable. `replicasPerShard > 1` pods
    // run raft (already PVC-backed via `LUMEN_RAFT_DATA_DIR`) and are
    // unaffected by this block. `--persistence=segment` (not the CBOR
    // default) is deliberate: it activates the local AOF (`src/aof.rs`)
    // alongside the periodic segment checkpoint, giving `everysec`-fsync
    // crash durability (~1s RPO bound) instead of only surviving cleanly
    // between `LUMEN_SNAPSHOT_SECS` (default 300s) CBOR snapshots.
    if lumen.spec.replicas_per_shard <= 1 {
        env.push(json!({ "name": "LUMEN_DATA_DIR", "value": EMBEDDED_DATA_DIR }));
        env.push(json!({ "name": "LUMEN_PERSISTENCE", "value": "segment" }));
    }
    // #2477: pure exposure of the pre-existing `LUMEN_ADMISSION_*` env
    // grammar (`service_http::AdmissionConfig`) — no new semantics, only a
    // declarative path onto the same env vars `serve()` already reads.
    if let Some(admission) = &lumen.spec.admission {
        if let Some(v) = admission.read_capacity {
            env.push(json!({ "name": "LUMEN_ADMISSION_READ_CAPACITY", "value": v.to_string() }));
        }
        if let Some(v) = admission.write_capacity {
            env.push(json!({ "name": "LUMEN_ADMISSION_WRITE_CAPACITY", "value": v.to_string() }));
        }
        if let Some(v) = admission.admin_capacity {
            env.push(json!({ "name": "LUMEN_ADMISSION_ADMIN_CAPACITY", "value": v.to_string() }));
        }
        if let Some(v) = admission.refill_secs {
            env.push(json!({ "name": "LUMEN_ADMISSION_REFILL_SECS", "value": v.to_string() }));
        }
        if let Some(v) = admission.max_keys {
            env.push(json!({ "name": "LUMEN_ADMISSION_MAX_KEYS", "value": v.to_string() }));
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
        "LUMEN_RAFT_PORT": RAFT_PORT.to_string(),
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

// #2475: alerts are added here only when the metric an `expr` reads is
// actually published today. `LumenRaftLeaderAbsent` reads this pod's
// self-scraped `lumen_raft_leader_known` gauge (`src/metrics.rs`, wired in
// `src/bin/lumen.rs`). `LumenPvcNearFull` reads the kubelet's
// `kubelet_volume_stats_*` series against the `raft-<name>-<ordinal>`
// StatefulSet PVC name pattern (`volumeClaimTemplates` name is `raft`, see
// `serving_statefulset`). `LumenStorageDegraded` (#2516) reads
// `lumen_storage_degraded`, the self-scraped gauge a pod sets to `1` the
// moment a durable write path (AOF append, segment/RDB checkpoint save, or
// raft log append) actually hits ENOSPC and the pod enters sticky degraded
// read-only mode (`Metrics::mark_storage_degraded`, `src/coordinator.rs` /
// `src/bin/lumen.rs` / `src/raft_sm.rs`) — it is the "disk is now actually
// full and writes are failing" companion to `LumenPvcNearFull`'s "disk is
// nearly full" early warning. `LumenReshardWorkflowStalled` is a PARTIAL proxy:
// the reshard driver's phase machine (`LumenStatus.reshard`, CR status) is
// not published to Prometheus by any customresourcestate config this
// operator ships, so this alert reads the driver's write-fence instead
// (`lumen_reshard_fence_active`/`_armed_unixtime`, `src/api.rs`'s
// `reshard_fence` handler) — it only catches a fence left armed past the
// fenced final `CatchingUp` pass's expected duration, not an early stall in
// `PrepareSplit`/`Splitting` (which never arms a fence at all). A full fix
// needs either a customresourcestate config or a driver-side liveness gauge
// and is out of this WI's scope. `LumenSlowQueries` (#2519) reads
// `lumen_slow_queries_total` (`src/metrics.rs`'s `Metrics::observe_search`),
// incremented once per search whose latency meets or exceeds the
// `LUMEN_SLOW_QUERY_MS` threshold (default 500ms).
fn prometheus_rule(cx: &RenderCtx<'_>) -> Value {
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "PrometheusRule",
        "metadata": cx.meta(cx.name, COMPONENT),
        "spec": {
            "groups": [{
                "name": "lumen.slo",
                "rules": [
                    {
                        "alert": "LumenNoReadyServingPods",
                        "expr": format!(
                            "kube_statefulset_status_replicas_ready{{statefulset=\"{}\", namespace=\"{}\"}} == 0",
                            cx.name, cx.ns
                        ),
                        "for": "2m",
                        "labels": { "severity": "critical" },
                        "annotations": {
                            "summary": "No ready lumen serving pods for {{ $labels.statefulset }}",
                            "runbook": "kubectl get pods -n {{ $labels.namespace }} -l app.kubernetes.io/instance={{ $labels.statefulset }} -o wide; check pod events/logs for crash or readiness-probe failure.",
                        },
                    },
                    {
                        "alert": "LumenBackupCronJobFailed",
                        "expr": format!(
                            "kube_job_status_failed{{namespace=\"{}\", job_name=~\"^{}-backup-.*\"}} >= 2",
                            cx.ns, cx.name
                        ),
                        "for": "5m",
                        "labels": { "severity": "warning" },
                        "annotations": {
                            "summary": "lumen backup CronJob {{ $labels.job_name }} has failed repeatedly (>=2 retained failed Jobs) in {{ $labels.namespace }}",
                            "runbook": "kubectl logs -n {{ $labels.namespace }} job/{{ $labels.job_name }}; a single failed Job is retained (not alerted) as a flake tolerance, so this means the CronJob is failing on every recent run.",
                        },
                    },
                    {
                        "alert": "LumenPodCrashLooping",
                        "expr": format!(
                            "increase(kube_pod_container_status_restarts_total{{namespace=\"{}\", pod=~\"^{}-[0-9]+$\"}}[15m]) > 3",
                            cx.ns, cx.name
                        ),
                        "for": "5m",
                        "labels": { "severity": "warning" },
                        "annotations": {
                            "summary": "lumen pod {{ $labels.pod }} is crash-looping in {{ $labels.namespace }}",
                            "runbook": "kubectl logs -n {{ $labels.namespace }} {{ $labels.pod }} --previous; check for OOMKilled (kubectl describe pod) or a bad rollout image.",
                        },
                    },
                    {
                        "alert": "LumenRaftLeaderAbsent",
                        "expr": format!(
                            "max(lumen_raft_leader_known{{namespace=\"{}\"}}) by (shard) == 0",
                            cx.ns
                        ),
                        "for": "2m",
                        "labels": { "severity": "critical" },
                        "annotations": {
                            "summary": "No lumen replica of shard {{ $labels.shard }} reports a known raft leader in {{ $labels.namespace }}",
                            "runbook": "kubectl get pods -n {{ $labels.namespace }} -o wide; map shard {{ $labels.shard }} to its StatefulSet ordinals (README §Dynamic Shard Topology) and check those pods for a minority-partition network split or a majority of voters down.",
                        },
                    },
                    {
                        "alert": "LumenReshardWorkflowStalled",
                        "expr": format!(
                            "lumen_reshard_fence_active{{namespace=\"{}\"}} == 1 and (time() - lumen_reshard_fence_armed_unixtime{{namespace=\"{}\"}}) > 900",
                            cx.ns, cx.ns
                        ),
                        "for": "1m",
                        "labels": { "severity": "warning" },
                        "annotations": {
                            "summary": "lumen reshard write fence has stayed armed for over 15m in {{ $labels.namespace }} -- the driver's final catch-up pass may be stuck",
                            "runbook": "kubectl get lumen -n {{ $labels.namespace }} -o jsonpath='{.items[*].status.reshard}'; check the reshard-driver operator pod's logs for a stuck :apply/:prune/:evict admin call, then POST /admin/reshard:fence with empty buckets to clear a wedged fence if the workflow is abandoned. Coverage note: this alert only detects a stall in the fenced final CatchingUp pass, not an early PrepareSplit/Splitting stall (#2475).",
                        },
                    },
                    {
                        "alert": "LumenPvcNearFull",
                        "expr": format!(
                            "kubelet_volume_stats_available_bytes{{namespace=\"{}\", persistentvolumeclaim=~\"^raft-{}-[0-9]+$\"}} / kubelet_volume_stats_capacity_bytes{{namespace=\"{}\", persistentvolumeclaim=~\"^raft-{}-[0-9]+$\"}} < 0.1",
                            cx.ns, cx.name, cx.ns, cx.name
                        ),
                        "for": "10m",
                        "labels": { "severity": "warning" },
                        "annotations": {
                            "summary": "lumen raft PVC {{ $labels.persistentvolumeclaim }} has less than 10% free space in {{ $labels.namespace }}",
                            "runbook": "kubectl get pvc -n {{ $labels.namespace }} {{ $labels.persistentvolumeclaim }} -o wide; the owning pod is {{ $labels.persistentvolumeclaim }} with its `raft-` prefix stripped -- exec in and run `df -h`, then expand volumeClaimTemplates (if the StorageClass supports online resize) or prune old snapshots/segments.",
                        },
                    },
                    {
                        "alert": "LumenStorageDegraded",
                        "expr": format!(
                            "max(lumen_storage_degraded{{namespace=\"{}\"}}) by (pod) == 1",
                            cx.ns
                        ),
                        "for": "1m",
                        "labels": { "severity": "critical" },
                        "annotations": {
                            "summary": "lumen pod {{ $labels.pod }} is in ENOSPC degraded read-only mode in {{ $labels.namespace }} -- mutating writes are being fast-failed with 507 storage_full",
                            "runbook": "kubectl exec -n {{ $labels.namespace }} {{ $labels.pod }} -- df -h; a durable write (AOF append, segment/RDB checkpoint, or raft log append) hit ENOSPC on the raft-<ordinal> PVC -- LumenPvcNearFull should have fired earlier as the early warning, so also check why it didn't. Free space (prune old snapshots/segments, or expand volumeClaimTemplates if the StorageClass supports online resize); the pod's periodic re-probe (LUMEN_STORAGE_FULL_REPROBE_SECS, default 30s) clears this automatically once a probe write succeeds, or restart the pod. If disk pressure traces back to an unfinished reshard leaving stale buckets, see LumenReshardWorkflowStalled too.",
                        },
                    },
                    {
                        "alert": "LumenSlowQueries",
                        "expr": format!(
                            "rate(lumen_slow_queries_total{{namespace=\"{}\"}}[5m]) > 0.1",
                            cx.ns
                        ),
                        "for": "10m",
                        "labels": { "severity": "warning" },
                        "annotations": {
                            "summary": "lumen is serving slow queries (>0.1/s at/above the LUMEN_SLOW_QUERY_MS threshold) for over 10m in {{ $labels.namespace }}",
                            "runbook": "kubectl top pods -n {{ $labels.namespace }}; check lumen_search_latency_seconds_bucket for the shifted percentile, look for a hot shard/collection, undersized HNSW ef, or resource pressure, and consider raising LUMEN_SLOW_QUERY_MS if the new baseline is expected.",
                        },
                    },
                ],
            }],
        },
    })
}
// CODEGEN-END

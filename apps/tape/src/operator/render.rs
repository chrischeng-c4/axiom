// HANDWRITE-BEGIN gap="missing-generator:logic:c41fb0fe" tracker="#1809" reason="Pure render (no I/O), composing shared service_k8s::render::ServiceStatefulSet with Tape-owned image, ports, journal PVC, TAPE_* environment names, auth Secret policy, and typed workload defaults; ServiceAccount, headless/client Services, PDB, and the opt-in spec.backup CronJob remain shared helper outputs; the always-rendered <name>-backup identity and the opt-in spec.observability ServiceMonitor/PrometheusRule pair are hand-rolled JSON."
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

use super::crd::{AuthMode, Tape};
use service_k8s::render::{self, RenderCtx, ServiceStatefulSet, WorkloadVolumeClaim};
use service_k8s::service::PruneTarget;
use service_k8s::stateful::{
    resource_request_or_default, DEFAULT_CPU_REQUEST, DEFAULT_MEMORY_REQUEST,
};

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

/// The shared name of every backup-scoped child: the CronJob, its
/// ServiceAccount, and the CronJob's [`prunes`] target.
///
/// One spelling on purpose. [`prunes`] must name the exact object
/// [`backup_cron_job`] rendered, and a prune that misses by one character is
/// silent — the controller GETs a name that does not exist, finds nothing to
/// delete, and reports success while the real CronJob keeps firing on its old
/// schedule. Deriving both from here makes that class of drift a compile-time
/// impossibility rather than something a test has to notice.
fn backup_child(name: &str) -> String {
    format!("{name}-backup")
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
/// Both are inactive unless the CR enables required bearer auth.
///
/// There is no precedence branch, and its absence is the point (#2765). The
/// two fields are mutually exclusive by CRD schema, so a spec naming both
/// never came from an API server; rendering *neither* is the safe answer,
/// because it leaves `TAPE_AUTH=required` with no registry file and the pod
/// fails startup naming the problem — instead of quietly serving whichever
/// registry a precedence rule happened to pick while the operator reads the
/// other one.
fn token_registry_source(tape: &Tape) -> Option<render::TokenRegistrySource<'_>> {
    if tape.spec.auth != AuthMode::Required {
        return None;
    }
    match (
        tape.spec.tokens_secret.as_deref(),
        tape.spec.tokens_secret_provider_class.as_deref(),
    ) {
        (Some(name), None) => Some(render::TokenRegistrySource::Secret {
            name,
            key: TOKEN_REGISTRY_KEY,
        }),
        (None, Some(provider_class)) => Some(render::TokenRegistrySource::Csi {
            provider_class,
            driver: tape.spec.tokens_secret_csi_driver.as_deref(),
        }),
        _ => None,
    }
}

// <HANDWRITE gap="missing-generator:kubernetes-peer-service" tracker="#1805" reason="kubernetes-peer-service section in render.rs is hand-written pending codegen support">
/// Render every child object for `tape`, in dependency order (identity first,
/// then the workload + its Services + PDB, then the opt-in observability pair
/// and the optional backup CronJob).
pub fn render(tape: &Tape) -> Vec<Value> {
    let name = instance(tape);
    let ns = namespace(tape);
    let cx = ctx(tape, &name, &ns);
    let headless = format!("{name}-headless");

    let mut objects = Vec::new();
    if tape.spec.service_account_name.is_none() {
        objects.push(render::service_account(&cx, COMPONENT));
    }
    objects.push(statefulset(tape, &cx, &headless));
    objects.push(render::headless_service_with_ports(
        &cx,
        &headless,
        COMPONENT,
        vec![
            json!({ "name": "http", "port": CLIENT_PORT, "targetPort": "http", "protocol": "TCP" }),
            json!({ "name": "raft", "port": RAFT_PORT, "targetPort": "raft", "protocol": "TCP" }),
        ],
    ));
    objects.push(render::client_service(&cx, &name, COMPONENT, CLIENT_PORT));
    // Keep a raft quorum during voluntary disruptions: at most one tape
    // pod may be unavailable at a time.
    objects.push(render::pdb(&cx, &name, COMPONENT, 1));
    objects.push(backup_service_account(&cx));
    if tape.spec.observability {
        objects.push(service_monitor(&cx));
        objects.push(prometheus_rule(&cx));
    }
    if let Some(cron) = backup_cron_job(tape, &cx) {
        objects.push(cron);
    }
    objects
}

/// Children a previous spec rendered that this one no longer wants (#3054).
///
/// The inverse of the `spec.backup` branch in [`render`] above, naming its
/// target through the same [`instance`] / [`backup_child`] helpers the render
/// path resolves its own name with rather than re-spelling it. Removing a
/// backup schedule from the CR must actually stop the CronJob from existing —
/// Server-Side Apply only reconciles fields on objects it is still given, it
/// never deletes an object that stopped being rendered, so without this the
/// CronJob keeps firing on its old schedule forever.
///
/// # Why the observability pair is not pruned
///
/// The `spec.observability` ServiceMonitor/PrometheusRule are the other two
/// conditional children, and naming them here is actively wrong for two
/// independent reasons.
///
/// The disqualifying one is that a `PruneTarget` costs a GET on every requeue,
/// and both are `monitoring.coreos.com/v1` kinds. On a cluster without the
/// Prometheus Operator CRDs that API group is not served at all, so the
/// apiserver answers the GET with a plain-text `404 page not found` rather than
/// a structured `NotFound`; `get_opt` keys off `reason == "NotFound"`, does not
/// recognise it, and propagates it — failing the *entire* reconcile, including
/// the apply and the status write, on a 15s retry loop that never converges.
/// `spec.observability` is default-off precisely so a vanilla cluster stays
/// installable (see its doc comment on `TapeSpec`); pruning on the `false`
/// branch would reach for that API group in exactly the vanilla case the
/// default exists to protect, inverting the invariant. Caught by the Kind gate,
/// which is the only place a missing API group is real. Restoring these two
/// targets is blocked on `get_opt` treating a bare 404 as absence (#3079).
///
/// The independent one is that they would not earn it anyway: lumen's
/// `prunes()` draws the line at children whose *presence changes runtime
/// behaviour*, and a stale ServiceMonitor only scrapes a metric nobody reads,
/// where a stale CronJob keeps writing backups to a destination the spec has
/// stopped naming.
///
/// [`backup_service_account`] is likewise never named, for a third reason: it
/// is rendered *unconditionally* (`render`, above) as a stable per-instance
/// identity for Workload Identity annotations, precisely so that identity
/// survives toggling `spec.backup` on and off. Pruning it would break that
/// guarantee in the other direction.
///
/// R4/AC6: this only *names* candidates; it does not itself verify ownership.
/// The controller (`libs/service-k8s/src/controller.rs:198` `prune_object`)
/// GETs the live object and only deletes it when a controller `ownerReference`
/// UID matches this CR's UID, warning and skipping otherwise — so a
/// same-named object this CR does not own is never touched. Tape does not
/// reimplement that check here.
pub fn prunes(tape: &Tape) -> Vec<PruneTarget> {
    let mut targets = Vec::new();
    if tape.spec.backup.is_none() {
        targets.push(PruneTarget {
            api_version: "batch/v1",
            kind: "CronJob",
            name: backup_child(&instance(tape)),
        });
    }
    targets
}

/// Prometheus label-selector fragment scoping a *self-scraped tape* series to
/// this instance, the operator-path replacement for the static component's
/// `{app="tape",role="server"}`.
///
/// Those two labels are not intrinsic to the metric: they exist on the series
/// only because the component's ServiceMonitor grafts the Service's `app` /
/// `role` labels on via `targetLabels`. The operator labels its children with
/// the `app.kubernetes.io/*` recommended set instead, so the same trick needs
/// the same names — [`service_monitor`] grafts
/// `app.kubernetes.io/{instance,component}`, Prometheus sanitizes those to
/// `app_kubernetes_io_{instance,component}`, and the exprs select on the
/// sanitized form. Lifting the component's exprs verbatim would render, apply,
/// and evaluate cleanly while matching nothing — a permanently silent alert,
/// which is the precise failure #2575 exists to prevent.
fn series_selector(cx: &RenderCtx<'_>) -> String {
    format!(
        "namespace=\"{}\",app_kubernetes_io_instance=\"{}\",app_kubernetes_io_component=\"{}\"",
        cx.ns, cx.name, COMPONENT
    )
}

/// Static labels stamped on every alert this rule fires.
///
/// `sum(...)` without a `by` clause discards every series label, so the two
/// latency alerts would otherwise reach Alertmanager with nothing but their
/// name — the static component re-adds `app`/`role` for exactly that reason.
/// Only `[a-zA-Z_][a-zA-Z0-9_]*` is a legal Prometheus label name, so the
/// operator's dotted/slashed label keys appear here in their sanitized form,
/// matching what the un-aggregated series carry.
fn alert_labels(cx: &RenderCtx<'_>, severity: &str) -> Value {
    json!({
        "severity": severity,
        "namespace": cx.ns,
        "app_kubernetes_io_instance": cx.name,
    })
}

/// Selector label kube-prometheus-stack's default `serviceMonitorSelector` /
/// `ruleSelector` matches on (`release: <helm release name>`). Both static
/// observability objects already carry it; without it the stack's default
/// install silently ignores the rendered pair, so it is reproduced verbatim
/// rather than left to the CR author.
const PROMETHEUS_RELEASE_LABEL: &str = "prometheus";

/// Scrape config for this instance's `/metrics` (#2575), the operator-rendered
/// twin of `k8s/components/observability/servicemonitor.yaml`.
///
/// The selector matches this instance's *serving* Services — both the headless
/// and the client Service carry the component labels, so, exactly as with the
/// static component's `{app: tape, role: server}` selector against
/// `k8s/base/service.yaml`'s two Services, each pod is discovered twice. It is
/// deliberately not "fixed" here: the latency alerts divide two equally
/// doubled aggregates and are immune, and changing it would diverge the
/// operator path from the deployed static one for no alerting benefit.
fn service_monitor(cx: &RenderCtx<'_>) -> Value {
    let mut meta = cx.meta(cx.name, COMPONENT);
    meta["labels"]["release"] = json!(PROMETHEUS_RELEASE_LABEL);
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "ServiceMonitor",
        "metadata": meta,
        "spec": {
            "selector": { "matchLabels": cx.selector(COMPONENT) },
            // Graft the instance identity onto every scraped series; the alert
            // exprs below select on the sanitized form (see [`series_selector`]).
            "targetLabels": ["app.kubernetes.io/instance", "app.kubernetes.io/component"],
            "endpoints": [{
                "port": "http",
                "path": "/metrics",
                "interval": "15s",
                "scrapeTimeout": "10s",
                "honorLabels": true,
            }],
        },
    })
}

/// Container resources for tape's workload: requests for both CPU and memory,
/// plus a memory limit matching the request (#3051).
///
/// This is `render::requested_resources` plus the memory limit. Requests alone
/// make the pod Burstable, and the kubelet then relieves node memory pressure
/// by picking victims on QoS and usage — so a tape that grows without bound
/// gets its neighbours evicted while it keeps running. Bounding memory makes
/// the OOMKill land on the container that caused it.
///
/// Memory only, no CPU limit: tape is scheduled one pod per node and should be
/// free to use idle CPU, and throttling the persist path would make the very
/// latency this alert watches worse.
///
/// Do not raise the limit in response to an OOMKill. The growth it bounds is
/// the whole-journal rewrite in `AppState::persist`, and #3052 (WAL + group
/// commit) is what removes it.
fn container_resources(cpu: &str, memory: &str) -> Value {
    // Same defaulting as `render::requested_resources`, reused rather than
    // re-spelled: a whitespace-only `spec.cluster.resources.cpu` must fall back
    // to the shared baseline, not render as an unparseable quantity that makes
    // the API server reject the StatefulSet on every reconcile.
    let cpu = resource_request_or_default(cpu, DEFAULT_CPU_REQUEST);
    let memory = resource_request_or_default(memory, DEFAULT_MEMORY_REQUEST);
    json!({
        "requests": { "cpu": cpu, "memory": memory },
        "limits": { "memory": memory },
    })
}

/// tape's four SLO alerts (#2575), the operator-rendered twin of
/// `k8s/components/observability/prometheusrule.yaml`.
///
/// Every alert reads a series tape actually publishes today.
/// `TapeAppendLatencyHigh` / `TapeReplayLatencyHigh` divide the
/// `tape_{append,replay}_latency_ms_sum` / `_count` pair `src/metrics.rs`
/// records per request, `clamp_min(..., 1)` guarding the idle-window
/// zero-denominator. `TapeSubscriptionLagGrowing` reads the
/// `tape_subscription_lag{topic,subscription}` gauge `src/server.rs` publishes
/// per subscription (#2485). `TapePodRestarting` is the one kube-state-metrics
/// series in the set, so it scopes by pod name and container instead — and the
/// container name is the one substantive difference from the static file,
/// which filters `container="tape"`: the shared StatefulSet helper names the
/// container after the *component*, so on this path it is `server`.
///
/// Thresholds, `for` windows, severities, summaries, and both #2485 runbooks
/// are reproduced from the static component verbatim; `tests/operator.rs`
/// holds the two documents to that.
fn prometheus_rule(cx: &RenderCtx<'_>) -> Value {
    let s = series_selector(cx);
    let mut meta = cx.meta(cx.name, COMPONENT);
    meta["labels"]["release"] = json!(PROMETHEUS_RELEASE_LABEL);
    json!({
        "apiVersion": "monitoring.coreos.com/v1",
        "kind": "PrometheusRule",
        "metadata": meta,
        "spec": {
            "groups": [{
                "name": "tape.slo",
                "interval": "30s",
                "rules": [
                    {
                        "alert": "TapeAppendLatencyHigh",
                        "expr": format!(
                            "sum(rate(tape_append_latency_ms_sum{{{s}}}[5m])) \
                             / clamp_min(sum(rate(tape_append_latency_ms_count{{{s}}}[5m])), 1) > 500"
                        ),
                        "for": "10m",
                        "labels": alert_labels(cx, "warning"),
                        "annotations": {
                            "summary": "tape append average latency above 500ms",
                        },
                    },
                    {
                        "alert": "TapeReplayLatencyHigh",
                        "expr": format!(
                            "sum(rate(tape_replay_latency_ms_sum{{{s}}}[5m])) \
                             / clamp_min(sum(rate(tape_replay_latency_ms_count{{{s}}}[5m])), 1) > 2000"
                        ),
                        "for": "10m",
                        "labels": alert_labels(cx, "warning"),
                        "annotations": {
                            "summary": "tape replay average latency above 2s",
                        },
                    },
                    {
                        "alert": "TapePodRestarting",
                        "expr": format!(
                            "increase(kube_pod_container_status_restarts_total{{namespace=\"{}\",pod=~\"^{}-[0-9]+$\",container=\"{}\"}}[15m]) > 2",
                            cx.ns, cx.name, COMPONENT
                        ),
                        "for": "5m",
                        "labels": alert_labels(cx, "warning"),
                        "annotations": {
                            "summary": "tape pod restarting repeatedly",
                            "runbook": POD_RESTARTING_RUNBOOK,
                        },
                    },
                    {
                        // #2573. `max_over_time` on purpose: the node
                        // re-probes every 30s and clears the gauge itself, so
                        // a volume that keeps filling and draining can read 0
                        // at every scrape while rejecting writes between them.
                        // The window trades a ~5m tail after real recovery for
                        // not missing that. `critical`, not `warning`: unlike
                        // the latency alerts this one means writes are already
                        // being refused.
                        "alert": "TapeStorageDegraded",
                        "expr": format!("max_over_time(tape_storage_degraded{{{s}}}[5m]) > 0"),
                        "for": "2m",
                        "labels": alert_labels(cx, "critical"),
                        "annotations": {
                            "summary": "tape node in ENOSPC degraded read-only mode",
                            "runbook": STORAGE_DEGRADED_RUNBOOK,
                        },
                    },
                    {
                        "alert": "TapeSubscriptionLagGrowing",
                        "expr": format!("increase(tape_subscription_lag{{{s}}}[15m]) > 0"),
                        "for": "15m",
                        "labels": alert_labels(cx, "warning"),
                        "annotations": {
                            "summary": "tape subscription lag growing over 15m",
                            "runbook": SUBSCRIPTION_LAG_RUNBOOK,
                        },
                    },
                    {
                        // #3051: Memory headroom alert fires before the limit is
                        // reached. Both series come from cAdvisor and carry identical
                        // labels, so they divide directly. Guard divide-by-zero by
                        // filtering the denominator: if no limit is set, the container
                        // is unbounded and the gauge is 0, so the division yields +Inf
                        // and the alert fires permanently.
                        "alert": "TapeMemoryHeadroomLow",
                        "expr": format!(
                            "max by (pod) (\n\
                             container_memory_working_set_bytes{{namespace=\"{}\",pod=~\"^{}-[0-9]+$\",container=\"{}\"}}\n\
                             / (container_spec_memory_limit_bytes{{namespace=\"{}\",pod=~\"^{}-[0-9]+$\",container=\"{}\"}} > 0)\n\
                             ) > 0.85",
                            cx.ns, cx.name, COMPONENT,
                            cx.ns, cx.name, COMPONENT
                        ),
                        "for": "10m",
                        "labels": alert_labels(cx, "warning"),
                        "annotations": {
                            "summary": "tape container memory headroom below 15%",
                            "runbook": MEMORY_HEADROOM_RUNBOOK,
                        },
                    },
                ],
            }],
        },
    })
}

/// #2485's seed-failure triage, verbatim from the static component. A restart
/// loop on a CR carrying `spec.bootstrapSeedUri` is ambiguous until the log
/// decision field is read, so the runbook's job is to make the three outcomes
/// distinguishable rather than to describe the symptom.
const POD_RESTARTING_RUNBOOK: &str = "#2485: Differentiate seed-failure restarts by checking if `spec.bootstrapSeedUri` is set on the CR and examining the pod log for structured decision fields: (1) `decision=\"seeded\"` = seed succeeded, look for other causes (probe failures, image pull, resource limits); (2) `decision=\"skipped_existing_state\"` = seed was skipped (PVC already had data), pod is healthy; (3) NEITHER line present before a crash = seed fetch/decode failed (bad URI, IAM, corrupt object) — restore logs show which. #2468: the one-shot seed cleared bit is separate. See docs/deployment-handoff.md Cold restore runbook.";

/// #2485's consumer-liveness triage, verbatim from the static component.
/// Growing lag is only sometimes a fault — the runbook exists to separate a
/// dead consumer from a fast producer.
const SUBSCRIPTION_LAG_RUNBOOK: &str = "#2485: Check if the consumer bound to the subscription is alive and actively pulling. If the consumer has stopped or stalled, verify it has not crashed or exhausted resources. If the consumer is healthy, check the append rate to the topic — a rapid append rate combined with a slow consumer will naturally grow lag. No action needed if this is expected; adjust retention policy if needed to protect the subscription's checkpoint from expiry.";

/// #2573's ENOSPC triage, verbatim from the static component. The alert on its
/// own tells an operator writes are being refused but not that the node will
/// recover itself — the runbook's job is to stop a reflexive pod restart and
/// point at the two things that actually decide the outcome: capacity, and the
/// flap counter that distinguishes "recovered" from "recovering every 30s".
const STORAGE_DEGRADED_RUNBOOK: &str = "#2573: The node hit ENOSPC on its journal persist path and latched degraded read-only mode — mutating requests answer 507 `storage_full`, reads keep serving. Check `tape_storage_full_errors_total`: a rising counter with the gauge back at 0 means the volume is flapping in and out of full, not that it recovered. Remedy is capacity — free objects via retention or expand the PVC on a resizable StorageClass. No restart is needed: the pod re-probes the store directory every `TAPE_STORAGE_FULL_REPROBE_SECS` (default 30s) and clears the flag itself. If the gauge stays 1 after the volume has room, read the pod log for the re-probe warning — the store directory can be unwritable for reasons other than capacity (read-only remount, permissions).";

/// #3051 / #3052: Memory headroom runbook.
///
/// The one thing this text has to stop is the reflex fix. Raising the limit
/// clears the alert and restores exactly the silent growth the limit was added
/// to expose, so the runbook names the two causes that are actionable now
/// (unbounded retention, a stalled consumer) and points the structural one at
/// #3052 rather than at the person holding the pager.
///
/// It states no amplification ratio. The measured figure is from a macOS/APFS
/// host and has not been re-taken on Linux; an unverified multiplier in a
/// runbook is read as a fact about the cluster in front of you.
const MEMORY_HEADROOM_RUNBOOK: &str = "#3051: The container's memory working set is within 15% of its limit, and the limit equals the request, so the next step is an OOMKill of this pod. Do NOT raise the limit as the remedy — it defers the wall and hides the growth again; the limit exists to make this growth visible. Act on the two causes you can fix now: (1) retention — `tape retention list` (or the CR's `spec.retention`); a topic with no entry is NEVER pruned and grows without bound, so give every active topic a byte or count bound; (2) a stalled consumer — check `tape_subscription_lag` and whether the bound consumer is still pulling, since a checkpoint that stops advancing pins events that retention would otherwise drop. If neither applies, the pod is holding more memory than its journal justifies: this is the whole-journal-rewrite persist path (`AppState::persist` serializes the entire journal on every mutation), which #3052 (WAL + group commit) removes. Capacity sizing is #2552 — do not derive a new limit from this alert alone.";

/// A stable, per-instance identity for scheduled backup jobs (lumen's #808
/// pattern, adopted for #2574).
///
/// Rendered even when `spec.backup` is unset. The backup runner writes to a
/// cloud object store, so its ServiceAccount is the binding target for cloud
/// IAM — GKE Workload Identity annotates it, and the GCP acceptance harness
/// already pre-creates `<name>-backup` for exactly that
/// (`acceptance/gcp/scripts/render-manifests.sh`). An
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
        "metadata": cx.meta(&backup_child(cx.name), BACKUP_COMPONENT),
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
/// 401 — the CR is accepted either way because `auth: disabled` instances
/// legitimately need no token. Since #2765 made `required` the default, that
/// combination is now the one a CR reaches by saying nothing, so a `backup`
/// block with no `adminTokenSecret` is worth a second look.
fn backup_cron_job(tape: &Tape, cx: &RenderCtx) -> Option<Value> {
    let backup = tape.spec.backup.as_ref()?;
    let cron_name = backup_child(cx.name);

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
    // /data disk tier, the drain window, and the resolved auth mode.
    //
    // TAPE_AUTH is unconditional and comes from the mode, never from whether a
    // registry source happens to be set (#2765). Deriving it from the source
    // meant `auth: required` with no `tokensSecret` rendered a pod with no
    // TAPE_AUTH at all -- an open data plane produced by a CR that explicitly
    // asked for authentication. Now that same CR starts with
    // TAPE_AUTH=required and no registry file, so it fails startup loudly.
    let mut extra_env = vec![
        json!({ "name": "TAPE_BIND", "value": format!("0.0.0.0:{CLIENT_PORT}") }),
        json!({ "name": "TAPE_RAFT_PORT", "value": RAFT_PORT.to_string() }),
        json!({ "name": "TAPE_DATA_DIR", "value": "/data" }),
        json!({ "name": "TAPE_GRACE_SECS", "value": s.grace_secs.to_string() }),
        json!({ "name": "TAPE_LOG_FORMAT", "value": "json" }),
        json!({ "name": "TAPE_AUTH", "value": s.auth.as_env() }),
    ];
    if let Some(level) = &s.log_level {
        extra_env.push(json!({ "name": "RUST_LOG", "value": level }));
    }
    if let Some(limit) = s.body_limit_bytes {
        extra_env.push(json!({ "name": "TAPE_BODY_LIMIT_BYTES", "value": limit.to_string() }));
    }
    if token_registry_source(tape).is_some() {
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
        service_account_name: Some(s.service_account_name.as_deref().unwrap_or(cx.name)),
        env: extra_env,
        env_from: vec![],
        resources: container_resources(cpu, memory),
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
        lifecycle: None,
        volumes,
        volume_mounts,
        affinity: Some(render::dedicated_node_affinity(cx.selector(COMPONENT))),
        node_selector: None,
        tolerations: vec![],
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

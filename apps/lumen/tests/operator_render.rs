// SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, Lumen-owned raft wiring, and observability toggles.
#![cfg(feature = "operator")]

use std::collections::BTreeMap;

use kube::api::ObjectMeta;
use lumen::operator::crd::{
    AuthMode, Autoscaling, LogFormat, PlacementSpec, ReshardPhase,
    ReshardPolicy, ReshardWorkflowSpec, ServingBootstrapSpec, ServingSpec, ShardMapSpec, Toleration,
};
use lumen::operator::render::{prunes, render};
use lumen::operator::{Lumen, LumenSpec};
use serde::Deserialize;
use serde_json::Value;
use service_k8s::service::PruneTarget;

/// A `Lumen` with metadata set the way a real CR (and owner references) need.
fn lumen(name: &str, spec: LumenSpec) -> Lumen {
    let mut l = Lumen::new(name, spec);
    l.metadata = ObjectMeta {
        name: Some(name.to_string()),
        namespace: Some("acme".to_string()),
        uid: Some("uid-1234".to_string()),
        generation: Some(7),
        ..Default::default()
    };
    l
}

fn dev_spec() -> LumenSpec {
    LumenSpec {
        image: "lumen:latest".into(),
        image_pull_policy: None,
        placement: PlacementSpec::default(),
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        serving: ServingSpec {
            autoscaling: Autoscaling {
                min_replicas: 1,
                max_replicas: 3,
                target_cpu_utilization: 70,
            },
            ..Default::default()
        },
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: BTreeMap::new(),
    }
}

fn prod_spec() -> LumenSpec {
    LumenSpec {
        image: "registry.example.com/lumen:1.2.3".into(),
        image_pull_policy: Some("Always".into()),
        placement: PlacementSpec::default(),
        shard_count: 6,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Json,
        log_level: Some("warn".into()),
        auth: AuthMode::Required,
        serving: ServingSpec {
            autoscaling: Autoscaling {
                min_replicas: 6,
                max_replicas: 12,
                target_cpu_utilization: 65,
            },
            cpu: "4".into(),
            memory: "16Gi".into(),
            grace_secs: 45,
            ..Default::default()
        },
        reshard_policy: ReshardPolicy::default(),
        observability: true,
        network_policy: true,
        admission: None,
        service_account_name: None,
        service_account_annotations: BTreeMap::new(),
    }
}

/// Find the object of (kind, name) in a render set.
fn find<'a>(objs: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objs.iter()
        .find(|o| o["kind"] == kind && o["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name} in render; got: {:?}", kinds(objs)))
}

fn kinds(objs: &[Value]) -> Vec<String> {
    objs.iter()
        .map(|o| {
            format!(
                "{}/{}",
                o["kind"].as_str().unwrap(),
                o["metadata"]["name"].as_str().unwrap()
            )
        })
        .collect()
}

fn has(objs: &[Value], kind: &str, name: &str) -> bool {
    objs.iter()
        .any(|o| o["kind"] == kind && o["metadata"]["name"] == name)
}

/// Every container env var, as (name → rendered value-or-ref) for assertions.
fn env_names(container: &Value) -> Vec<String> {
    container["env"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn dev_renders_full_managed_set() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    // Serving objects, in the CR's namespace, named off the instance. The
    // serving fleet is a StatefulSet even at replicasPerShard:1 (#812). A
    // direct HPA is intentionally absent: it cannot perform Raft membership
    // changes or preserve whole per-shard replica layers.
    for (kind, name) in [
        ("ServiceAccount", "search"),
        ("ConfigMap", "search-config"),
        ("StatefulSet", "search"),
        ("Service", "search-headless"),
        ("Service", "search"),
        ("PodDisruptionBudget", "search"),
    ] {
        assert!(
            has(&objs, kind, name),
            "expected {kind}/{name}; got {:?}",
            kinds(&objs)
        );
    }
    assert!(!has(&objs, "HorizontalPodAutoscaler", "search"));
    // Never a Deployment — the operator no longer switches workload kind by
    // replica count.
    assert!(!has(&objs, "Deployment", "search"));
    // Relay is no longer part of Lumen's deployment surface.
    assert!(!has(&objs, "StatefulSet", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay-headless"));
    assert!(!has(&objs, "PodDisruptionBudget", "search-relay"));
    // No observability when the flag is off.
    assert!(!has(&objs, "ServiceMonitor", "search"));
    assert!(!has(&objs, "PrometheusRule", "search"));
    // Nor isolation: a NetworkPolicy only enforces where the CNI supports it,
    // so #2603 keeps it opt-in and this fixture leaves it off.
    assert!(!has(&objs, "NetworkPolicy", "search"));

    // Everything lands in the CR's namespace and carries the owner reference.
    for o in &objs {
        assert_eq!(
            o["metadata"]["namespace"], "acme",
            "wrong ns for {}",
            o["kind"]
        );
        let owner = &o["metadata"]["ownerReferences"][0];
        assert_eq!(owner["kind"], "Lumen");
        assert_eq!(owner["uid"], "uid-1234");
        assert_eq!(owner["controller"], true);
    }
}

#[test]
fn unset_service_account_name_renders_and_uses_operator_owned_sa() {
    // Regression: with `serviceAccountName` unset, the operator still
    // renders and owns a workload ServiceAccount named after the instance,
    // and the StatefulSet pod spec references it (#2497).
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    assert!(
        has(&objs, "ServiceAccount", "search"),
        "expected operator-owned workload ServiceAccount/search; got {:?}",
        kinds(&objs)
    );
    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(
        sts["spec"]["template"]["spec"]["serviceAccountName"],
        "search"
    );
}

#[test]
fn set_service_account_name_omits_workload_sa_and_uses_external_name() {
    // When `spec.serviceAccountName` points at a pre-existing, externally
    // managed ServiceAccount, the operator must never render (and so never
    // own or delete) a workload ServiceAccount of its own, and the
    // StatefulSet pod spec must reference the external name (#2497).
    let mut spec = dev_spec();
    spec.service_account_name = Some("external-sa".into());
    let l = lumen("search", spec);
    let objs = render(&l);

    // The backup ServiceAccount is a separate, still operator-owned concern
    // and is unaffected by this field — assert specifically on the workload
    // SA named after the instance, not "no ServiceAccount at all".
    assert!(
        !has(&objs, "ServiceAccount", "search"),
        "workload ServiceAccount/search must not be rendered when \
         serviceAccountName is set; got {:?}",
        kinds(&objs)
    );

    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(
        sts["spec"]["template"]["spec"]["serviceAccountName"],
        "external-sa"
    );
}

#[test]
fn set_service_account_name_never_emits_any_serviceaccount_named_the_instance() {
    // Ownership proof: the operator's apply/prune loop only ever manages
    // objects it renders (see reconcile.rs module docs — there is no
    // generic delete-by-kind sweep). Proving that no ServiceAccount object
    // named `<instance>` ever appears in the render set — across every
    // topology, not just the default one — is sufficient to prove the
    // externally-managed SA can never be pruned or deleted by the operator.
    for spec_fn in [dev_spec, prod_spec] {
        let mut spec = spec_fn();
        spec.service_account_name = Some("external-sa".into());
        let l = lumen("search", spec);
        let objs = render(&l);

        assert!(
            !objs
                .iter()
                .any(|o| o["kind"] == "ServiceAccount" && o["metadata"]["name"] == "search"),
            "no ServiceAccount named the instance may ever be rendered when \
             serviceAccountName is externally managed; got {:?}",
            kinds(&objs)
        );
    }
}

#[test]
fn statefulset_wires_serving_contract_single_member() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");

    // replicasPerShard is the apply-time floor; StatefulSet-native rollout knobs.
    assert_eq!(sts["spec"]["replicas"], 1);
    assert_eq!(sts["spec"]["serviceName"], "search-headless");
    assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");
    assert_eq!(sts["spec"]["updateStrategy"]["type"], "RollingUpdate");
    assert!(
        sts["spec"]["strategy"].is_null(),
        "strategy is Deployment-only; StatefulSet uses updateStrategy"
    );

    let c = &sts["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(c["image"], "lumen:latest");
    assert_eq!(c["imagePullPolicy"], "IfNotPresent");
    assert_eq!(c["command"], serde_json::json!(["lumen", "serve"]));
    assert_eq!(c["ports"][0]["name"], "http");
    assert_eq!(c["ports"][0]["containerPort"], 7373);
    assert_eq!(c["ports"][1]["name"], "raft");
    assert_eq!(c["ports"][1]["containerPort"], 7374);

    let headless = find(&objs, "Service", "search-headless");
    assert_eq!(headless["spec"]["ports"][0]["name"], "http");
    assert_eq!(headless["spec"]["ports"][0]["port"], 7373);
    assert_eq!(headless["spec"]["ports"][1]["name"], "raft");
    assert_eq!(headless["spec"]["ports"][1]["port"], 7374);
    assert_eq!(headless["spec"]["ports"][1]["targetPort"], "raft");

    let config = find(&objs, "ConfigMap", "search-config");
    assert_eq!(config["data"]["LUMEN_RAFT_PORT"], "7374");

    // Shared request-only baseline: 1 CPU / 4Gi, no limits.
    assert_eq!(c["resources"]["requests"]["cpu"], "1");
    assert_eq!(c["resources"]["requests"]["memory"], "4Gi");
    assert!(c["resources"].get("limits").is_none());
    assert_eq!(
        sts["spec"]["template"]["spec"]["affinity"]["podAntiAffinity"]
            ["requiredDuringSchedulingIgnoredDuringExecution"][0]["topologyKey"],
        "kubernetes.io/hostname"
    );

    // Probes tuned for log-replay: a generous readiness failureThreshold.
    assert_eq!(c["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(c["readinessProbe"]["failureThreshold"], 60);
    assert_eq!(c["livenessProbe"]["httpGet"]["path"], "/healthz");

    // Hardened: non-root, read-only rootfs, all caps dropped.
    assert_eq!(c["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(c["securityContext"]["runAsNonRoot"], true);
    assert_eq!(
        c["securityContext"]["capabilities"]["drop"],
        serde_json::json!(["ALL"])
    );

    // Env: downward-API identity + Lumen-owned WAL mode + config-driven knobs.
    let names = env_names(c);
    for required in [
        "POD_NAME",
        "POD_NAMESPACE",
        "LUMEN_HOST",
        "LUMEN_WAL",
        "SHARD_COUNT",
        "LUMEN_AUTH",
        // #1384: serving pods must see the shard map the ConfigMap carries
        // so `lumen::config::shard_map_from_env` can route by it instead of
        // always falling back to the balanced default.
        "SHARD_MAP_VERSION",
        "VIRTUAL_BUCKET_COUNT",
        // #1387 AC1: `LUMEN_WAL=auto` resolves to embedded (RAM-only) at
        // replicasPerShard:1 — without these, the mounted `raft` PVC is
        // never touched and a pod restart wipes all data.
        "LUMEN_DATA_DIR",
        "LUMEN_PERSISTENCE",
    ] {
        assert!(
            names.contains(&required.to_string()),
            "missing env {required}; have {names:?}"
        );
    }
    // #1387: the exact values that activate the segment store + local AOF
    // (`everysec`-fsync crash durability) under the durable `raft` PVC mount,
    // disjoint from the raft backend's own `/var/lib/lumen/raft` subtree.
    let env = c["env"].as_array().unwrap();
    let value_of = |name: &str| {
        env.iter()
            .find(|e| e["name"] == name)
            .and_then(|e| e["value"].as_str())
            .unwrap_or_else(|| panic!("env {name} missing a literal value"))
            .to_string()
    };
    assert_eq!(value_of("LUMEN_DATA_DIR"), "/var/lib/lumen/data");
    assert_eq!(value_of("LUMEN_PERSISTENCE"), "segment");
    assert!(
        !value_of("LUMEN_DATA_DIR").starts_with("/var/lib/lumen/raft"),
        "embedded data dir must stay disjoint from the raft backend's subtree"
    );
    // Single member, no raft consensus at replicasPerShard:1 → no raft
    // peer-identity env.
    for absent in [
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "LUMEN_HEADLESS_SERVICE",
    ] {
        assert!(
            !names.contains(&absent.to_string()),
            "unexpected raft env {absent} at replicasPerShard:1; have {names:?}"
        );
    }
    // auth=off and no log level → those env vars are absent. There is no
    // inline-credential env var to check for any more: #2678 deleted the one
    // that existed, because a credential in the environment is a credential in
    // `kubectl describe pod`.
    assert!(!names.contains(&"LUMEN_TOKEN_REGISTRY_FILE".to_string()));
    assert!(!names.contains(&"LUMEN_LOG_LEVEL".to_string()));
    // #1384 AC4: default spec has no shard-map assignments yet, so the
    // ConfigMap key is absent (see `configmap_tracks_serving_spec`) and the
    // container env must not reference it either — a `configMapKeyRef` to a
    // missing key would fail the pod at start.
    assert!(!names.contains(&"SHARD_MAP_ASSIGNMENTS".to_string()));

    // Durable raft PVC (#812): the WAL survives pod reschedule/eviction/node
    // loss even for a single-member deployer — not just an emptyDir.
    let mounts = c["volumeMounts"].as_array().unwrap();
    assert!(
        mounts
            .iter()
            .any(|m| m["name"] == "raft" && m["mountPath"] == "/var/lib/lumen"),
        "missing raft volumeMount; have {mounts:?}"
    );
    // (#809) StatefulSet names the resulting per-pod PVCs
    // `raft-<statefulset-name>-<ordinal>` (e.g. `raft-search-0`); this is the
    // exact `raft-<name>-` prefix `service_k8s::resize::resize_instance` filters
    // on when detecting/patching live PVCs, and the "raft" template name +
    // `resources.requests.storage` field below are what it reads back to
    // compare against `spec.serving.raftStorage`. render() itself is
    // unchanged by #809 — resize tooling only reads what's already rendered
    // here.
    let vcts = sts["spec"]["volumeClaimTemplates"].as_array().unwrap();
    assert_eq!(vcts.len(), 1);
    assert_eq!(vcts[0]["metadata"]["name"], "raft");
    assert_eq!(vcts[0]["spec"]["resources"]["requests"]["storage"], "20Gi");
}

#[test]
fn configmap_tracks_serving_spec() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(cm["data"]["SHARD_COUNT"], "1");
    assert_eq!(cm["data"]["SHARD_MAP_VERSION"], "0");
    assert_eq!(cm["data"]["VIRTUAL_BUCKET_COUNT"], "4096");
    assert!(cm["data"]["SHARD_MAP_ASSIGNMENTS"].is_null());
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "pretty");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "off");
    assert_eq!(cm["data"]["LUMEN_PORT"], "7373");
    // No log level set → key omitted.
    assert!(cm["data"]["LUMEN_LOG_LEVEL"].is_null());
}

#[test]
fn multi_shard_single_replica_is_fixed_storage_topology_not_hpa() {
    let mut spec = dev_spec();
    spec.shard_count = 4;
    spec.replicas_per_shard = 1;
    spec.serving.autoscaling.min_replicas = 1;
    spec.serving.autoscaling.max_replicas = 12;
    let l = lumen("search", spec);
    let objs = render(&l);

    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(sts["spec"]["replicas"], 4);
    assert!(
        !has(&objs, "HorizontalPodAutoscaler", "search"),
        "HPA must not change multi-shard storage ownership"
    );

    // #1398: shardCount > 1 at replicasPerShard <= 1 is the routed serving
    // topology — each pod still needs LUMEN_HEADLESS_SERVICE to build stable
    // per-shard DNS names (`lumen::routing::shard_host`) for one-hop
    // cross-pod forwarding, even though there is no raft consensus to peer.
    let c = &sts["spec"]["template"]["spec"]["containers"][0];
    let names: Vec<String> = c["env"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"LUMEN_HEADLESS_SERVICE".to_string()),
        "routed topology (shardCount>1) must render LUMEN_HEADLESS_SERVICE; have {names:?}"
    );
    for absent in ["REPLICAS_PER_SHARD", "VOTER_COUNT"] {
        assert!(
            !names.contains(&absent.to_string()),
            "single-member shards still have no raft peer env; unexpected {absent}; have {names:?}"
        );
    }
}

#[test]
fn direct_hpa_is_never_rendered_for_single_replica_serving() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    assert!(!has(&objs, "HorizontalPodAutoscaler", "search"));
}

#[test]
fn single_member_storage_pod_count_ignores_legacy_autoscaling_bounds() {
    // #1317: default `Autoscaling` (minReplicas: 3, maxReplicas: 12) at the
    // CRD's own default topology (shardCount: 1, replicasPerShard: 1) must
    // not fan out to 3+ uncoordinated shard-0 copies.
    let mut default_bounds_spec = dev_spec();
    default_bounds_spec.serving.autoscaling = Autoscaling::default();
    assert_eq!(default_bounds_spec.serving.autoscaling.min_replicas, 3);
    assert_eq!(default_bounds_spec.serving.autoscaling.max_replicas, 12);
    assert_eq!(default_bounds_spec.storage_pod_count(), 1);

    let l = lumen("search", default_bounds_spec);
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(sts["spec"]["replicas"], 1);
    assert!(!has(&objs, "HorizontalPodAutoscaler", "search"));

    // Also an explicit, non-default CR bound (minReplicas: 3) — the same
    // clamp applies whether the bounds come from the CRD default or an
    // operator's explicit override.
    let mut explicit_spec = dev_spec();
    explicit_spec.serving.autoscaling.min_replicas = 3;
    explicit_spec.serving.autoscaling.max_replicas = 3;
    assert_eq!(explicit_spec.storage_pod_count(), 1);

    let l = lumen("search", explicit_spec);
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(sts["spec"]["replicas"], 1);
    assert!(!has(&objs, "HorizontalPodAutoscaler", "search"));
}

#[test]
fn raft_ha_storage_pod_count_is_unaffected_by_single_member_clamp() {
    // #1317 regression: `replicasPerShard > 1` (raft-HA) must keep computing
    // the full fixed membership size — the clamp above only applies to the
    // no-raft single-member fallback branch.
    let mut spec = dev_spec();
    spec.shard_count = 2;
    spec.replicas_per_shard = 3;
    spec.serving.autoscaling = Autoscaling::default();
    assert_eq!(spec.storage_pod_count(), 6);
}

#[test]
fn prod_wires_auth_and_observability() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);

    // auth=required without registry projection (retired phase 1).
    let dep = find(&objs, "StatefulSet", "lumen");
    let c = &dep["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(c["image"], "registry.example.com/lumen:1.2.3");
    assert_eq!(c["imagePullPolicy"], "Always");
    // log level set → present.
    assert!(env_names(c).contains(&"LUMEN_LOG_LEVEL".to_string()));

    // ConfigMap reflects 6 shards + json + required auth.
    let cm = find(&objs, "ConfigMap", "lumen-config");
    assert_eq!(cm["data"]["SHARD_COUNT"], "6");
    assert_eq!(cm["data"]["SHARD_MAP_VERSION"], "0");
    assert_eq!(cm["data"]["VIRTUAL_BUCKET_COUNT"], "4096");
    assert!(cm["data"]["SHARD_MAP_ASSIGNMENTS"].is_null());
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "json");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "required");

    // observability=true → monitoring objects present.
    assert!(has(&objs, "ServiceMonitor", "lumen"));
    assert!(has(&objs, "PrometheusRule", "lumen"));

    // networkPolicy=true → isolation present, and the client API stays open to
    // the whole cluster while the Raft port never appears in a rule sourced
    // from `namespaceSelector` (#2603). The namespace/ownerReference sweep
    // below covers this object too, so it is GC'd with the CR.
    let np = find(&objs, "NetworkPolicy", "lumen");
    let cluster_facing = np["spec"]["ingress"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["from"][0]["namespaceSelector"].is_object())
        .expect("a cluster-facing ingress rule");
    let open_ports: Vec<i64> = cluster_facing["ports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["port"].as_i64().unwrap())
        .collect();
    assert_eq!(
        open_ports,
        vec![7373],
        "only the client API is cluster-open"
    );

    for o in &objs {
        assert_eq!(
            o["metadata"]["namespace"], "acme",
            "wrong ns for {}",
            o["kind"]
        );
        assert_eq!(o["metadata"]["ownerReferences"][0]["kind"], "Lumen");
    }
}

/// #2870 AC1: the token and identity registry projections (retired phase 1) are
/// no longer rendered, for auth=off and auth=required alike.
///
/// The third fixture this ran against — one that populated `spec.identities`,
/// the only shape under which the retired code rendered anything — is gone,
/// because #2872 removed the field it set. What replaces it as the gate that
/// cannot pass on an unmodified tree is the schema itself: there is no longer
/// a way to express the input the retired projection consumed.
#[test]
fn no_retired_credential_registry_projections() {
    // Substring residue, not a structural walk: the retired projections reached
    // pod specs, CronJob pod templates, and container env alike, and each of
    // those sits at a different JSON path. Serializing the whole object is the
    // only check that cannot miss a path we forgot to enumerate.
    const RETIRED: [&str; 5] = [
        "lumen-token-registry",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "lumen-identity-registry",
        "LUMEN_IDENTITY_REGISTRY_FILE",
        "search-identities",
    ];

    for spec_fn in [dev_spec, prod_spec] {
        let auth = spec_fn().auth;
        let objs = render(&lumen("search", spec_fn()));

        for obj in &objs {
            let text = serde_json::to_string(obj).expect("rendered object serializes");
            for needle in RETIRED {
                assert!(
                    !text.contains(needle),
                    "{needle} must not be rendered (auth={auth:?}) in {} {}",
                    obj["kind"],
                    obj["metadata"]["name"],
                );
            }
        }

        assert!(
            !has(&objs, "ConfigMap", "search-identities"),
            "search-identities ConfigMap must not be rendered (auth={auth:?}); got {:?}",
            kinds(&objs)
        );
    }
}

/// #2475: the rendered PrometheusRule covers more than just the
/// zero-ready-pods case — backup CronJob failure and pod crash-looping,
/// scoped to this Lumen's namespace/name, are alerted on too.
#[test]
fn prometheus_rule_covers_backup_failure_and_crash_looping() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);
    let rule = find(&objs, "PrometheusRule", "lumen");
    let rules = rule["spec"]["groups"][0]["rules"].as_array().unwrap();
    let alert_names: Vec<&str> = rules.iter().map(|r| r["alert"].as_str().unwrap()).collect();
    assert_eq!(
        alert_names,
        vec![
            "LumenNoReadyServingPods",
            "LumenBackupCronJobFailed",
            "LumenPodCrashLooping",
            "LumenRaftLeaderAbsent",
            "LumenReshardWorkflowStalled",
            "LumenPvcNearFull",
            "LumenStorageDegraded",
            "LumenAuthRegistryReloadFailing",
            "LumenSlowQueries",
        ]
    );

    // #2487: this must be the StatefulSet-ready series — lumen renders
    // serving as a StatefulSet only, so a Deployment series never matches
    // and the alert can never fire.
    let no_ready_pods = rules
        .iter()
        .find(|r| r["alert"] == "LumenNoReadyServingPods")
        .unwrap();
    let no_ready_pods_expr = no_ready_pods["expr"].as_str().unwrap();
    assert!(no_ready_pods_expr.contains("kube_statefulset_status_replicas_ready"));
    assert!(!no_ready_pods_expr.contains("kube_deployment_status_replicas_available"));
    assert!(no_ready_pods_expr.contains("statefulset=\"lumen\""));
    assert!(no_ready_pods_expr.contains("namespace=\"acme\""));

    let backup = rules
        .iter()
        .find(|r| r["alert"] == "LumenBackupCronJobFailed")
        .unwrap();
    let backup_expr = backup["expr"].as_str().unwrap();
    assert!(backup_expr.contains("kube_job_status_failed"));
    assert!(backup_expr.contains("namespace=\"acme\""));
    assert!(backup_expr.contains("job_name=~\"^lumen-backup-.*\""));
    // #2475: >= 2 retained failed Jobs (not > 0) so a single flaky run
    // (successfulJobsHistoryLimit/failedJobsHistoryLimit both retain
    // multiple Job objects) does not page on its own.
    assert!(backup_expr.contains(">= 2"));

    let crash_loop = rules
        .iter()
        .find(|r| r["alert"] == "LumenPodCrashLooping")
        .unwrap();
    let crash_loop_expr = crash_loop["expr"].as_str().unwrap();
    assert!(crash_loop_expr.contains("kube_pod_container_status_restarts_total"));
    assert!(crash_loop_expr.contains("namespace=\"acme\""));
    assert!(crash_loop_expr.contains("pod=~\"^lumen-[0-9]+$\""));

    for r in rules {
        let annotations = &r["annotations"];
        assert!(
            annotations["summary"].is_string(),
            "{} missing summary annotation",
            r["alert"]
        );
        assert!(
            annotations["runbook"].is_string(),
            "{} missing runbook annotation",
            r["alert"]
        );
    }
}

/// #2475: raft-leader-absent, reshard-stalled, PVC-near-full, and
/// auth-registry-reload-failing each bind to a real metric name published
/// by this binary (`src/metrics.rs`) or the kubelet, not a synthesized one.
#[test]
fn prometheus_rule_covers_raft_reshard_pvc_and_auth_failure_modes() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);
    let rule = find(&objs, "PrometheusRule", "lumen");
    let rules = rule["spec"]["groups"][0]["rules"].as_array().unwrap();

    let raft_leader_absent = rules
        .iter()
        .find(|r| r["alert"] == "LumenRaftLeaderAbsent")
        .unwrap();
    let raft_expr = raft_leader_absent["expr"].as_str().unwrap();
    assert!(raft_expr.contains("lumen_raft_leader_known"));
    assert!(raft_expr.contains("namespace=\"acme\""));
    assert!(raft_expr.contains("by (shard)"));

    let reshard_stalled = rules
        .iter()
        .find(|r| r["alert"] == "LumenReshardWorkflowStalled")
        .unwrap();
    let reshard_expr = reshard_stalled["expr"].as_str().unwrap();
    assert!(reshard_expr.contains("lumen_reshard_fence_active"));
    assert!(reshard_expr.contains("lumen_reshard_fence_armed_unixtime"));
    assert!(reshard_expr.contains("namespace=\"acme\""));

    let pvc_near_full = rules
        .iter()
        .find(|r| r["alert"] == "LumenPvcNearFull")
        .unwrap();
    let pvc_expr = pvc_near_full["expr"].as_str().unwrap();
    assert!(pvc_expr.contains("kubelet_volume_stats_available_bytes"));
    assert!(pvc_expr.contains("kubelet_volume_stats_capacity_bytes"));
    assert!(pvc_expr.contains("namespace=\"acme\""));
    assert!(pvc_expr.contains("persistentvolumeclaim=~\"^raft-lumen-[0-9]+$\""));

    let auth_reload_failing = rules
        .iter()
        .find(|r| r["alert"] == "LumenAuthRegistryReloadFailing")
        .unwrap();
    let auth_expr = auth_reload_failing["expr"].as_str().unwrap();
    assert!(auth_expr.contains("lumen_auth_registry_reload_failures_total"));
    assert!(auth_expr.contains("namespace=\"acme\""));
}

/// #2519: the slow-query alert binds to `lumen_slow_queries_total`
/// (`src/metrics.rs`'s `Metrics::observe_search`, gated on
/// `LUMEN_SLOW_QUERY_MS`), carries a `for: 10m` sustained-rate window per
/// the issue's acceptance criteria, and gets the same summary/runbook
/// annotation shape as every other alert in this rule group.
#[test]
fn prometheus_rule_covers_slow_queries() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);
    let rule = find(&objs, "PrometheusRule", "lumen");
    let rules = rule["spec"]["groups"][0]["rules"].as_array().unwrap();

    let slow_queries = rules
        .iter()
        .find(|r| r["alert"] == "LumenSlowQueries")
        .unwrap();
    let expr = slow_queries["expr"].as_str().unwrap();
    assert!(expr.contains("lumen_slow_queries_total"));
    assert!(expr.contains("namespace=\"acme\""));
    assert_eq!(slow_queries["for"], "10m");
    assert_eq!(slow_queries["labels"]["severity"], "warning");
    assert!(slow_queries["annotations"]["summary"].is_string());
    assert!(slow_queries["annotations"]["runbook"].is_string());
}

/// #2516: the storage-degraded alert binds to `lumen_storage_degraded`
/// (`src/metrics.rs`'s `Metrics::mark_storage_degraded`, flipped by
/// `src/coordinator.rs`/`src/bin/lumen.rs`/`src/raft_sm.rs` on a real
/// ENOSPC), pages at `critical` (writes are actively failing, not just
/// nearing capacity like `LumenPvcNearFull`), and its runbook cross-references
/// both `LumenPvcNearFull` (the early warning this alert follows) and
/// `LumenReshardWorkflowStalled` (the disk-pressure story).
#[test]
fn prometheus_rule_covers_storage_degraded() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);
    let rule = find(&objs, "PrometheusRule", "lumen");
    let rules = rule["spec"]["groups"][0]["rules"].as_array().unwrap();

    let storage_degraded = rules
        .iter()
        .find(|r| r["alert"] == "LumenStorageDegraded")
        .unwrap();
    let expr = storage_degraded["expr"].as_str().unwrap();
    assert!(expr.contains("lumen_storage_degraded"));
    assert!(expr.contains("namespace=\"acme\""));
    assert!(expr.contains("by (pod)"));
    assert_eq!(storage_degraded["for"], "1m");
    assert_eq!(storage_degraded["labels"]["severity"], "critical");
    let summary = storage_degraded["annotations"]["summary"].as_str().unwrap();
    assert!(summary.contains("507"));
    let runbook = storage_degraded["annotations"]["runbook"].as_str().unwrap();
    assert!(runbook.contains("LumenPvcNearFull"));
    assert!(runbook.contains("LumenReshardWorkflowStalled"));
    assert!(runbook.contains("LUMEN_STORAGE_FULL_REPROBE_SECS"));
}



#[test]
fn reshard_status_is_recommendation_only_without_capacity_ceiling() {
    let mut spec = dev_spec();
    spec.reshard_policy.workflow = ReshardWorkflowSpec {
        phase: ReshardPhase::PrepareSplit,
        target_shard_count: Some(2),
        ..Default::default()
    };
    let status = spec.reshard_status();

    assert_eq!(status.phase, "PrepareSplit");
    assert!(status.recommendation_only);
    assert_eq!(status.progress_percent, 10);
    assert_eq!(status.target_shard_count, Some(2));
    assert_eq!(status.blocking_conditions, vec!["maxShardBytesUnset"]);
    assert!(status.message.contains("will not auto-split"));
}

#[test]
fn reshard_status_tracks_workflow_phases_with_capacity_policy() {
    for phase in [
        ReshardPhase::PrepareSplit,
        ReshardPhase::Splitting,
        ReshardPhase::CatchingUp,
        ReshardPhase::Complete,
    ] {
        let mut spec = dev_spec();
        spec.shard_count = 2;
        spec.reshard_policy.max_shard_bytes = Some(64 * 1024 * 1024 * 1024 * 1024);
        spec.reshard_policy.max_shards = Some(8);
        spec.reshard_policy.migration_bytes_per_sec = Some(256 * 1024 * 1024);
        spec.reshard_policy.workflow = ReshardWorkflowSpec {
            phase,
            target_shard_count: None,
            ..Default::default()
        };

        let status = spec.reshard_status();
        assert_eq!(status.phase, phase.as_str());
        assert!(!status.recommendation_only);
        assert_eq!(status.target_shard_count, Some(3));
        assert_eq!(status.progress_percent, phase.progress_percent());
        assert_eq!(status.migration_bytes_per_sec, Some(256 * 1024 * 1024));
        assert!(status.blocking_conditions.is_empty());
    }
}

#[test]
fn reshard_status_with_usage_falls_back_without_capacity_ceiling() {
    // #1319 R1: `maxShardBytes` unset (recommendation-only) means there is
    // nothing to compare usage against — falls straight back to
    // `reshard_status()`, `maxObservedPercent` stays `None`.
    let spec = dev_spec();
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 999_999_999u64);
    let status = spec.reshard_status_with_usage(&usage, spec.shard_map.version);
    assert_eq!(status, spec.reshard_status());
    assert_eq!(status.max_observed_percent, None);
}

#[test]
fn reshard_status_with_usage_falls_back_when_usage_not_measured_yet() {
    // Policy configured, but no usage sample yet this tick (empty map).
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    let status = spec.reshard_status_with_usage(&BTreeMap::new(), spec.shard_map.version);
    assert_eq!(status.max_observed_percent, None);
    assert_eq!(status, spec.reshard_status());
}

#[test]
fn reshard_status_with_usage_below_prepare_threshold() {
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    // Defaults: prepare 50%, urgent 85%.
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 100_000u64); // 10%
    let status = spec.reshard_status_with_usage(&usage, spec.shard_map.version);
    assert_eq!(status.max_observed_percent, Some(10));
    assert_eq!(
        status.usage_measured_at_map_version,
        Some(spec.shard_map.version)
    );
    assert!(status.blocking_conditions.is_empty());
    assert!(status.message.contains("below prepare threshold"));
}

#[test]
fn reshard_status_with_usage_reports_prepare_threshold_crossed() {
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 600_000u64); // 60%: past prepare(50), below urgent(85)
    let status = spec.reshard_status_with_usage(&usage, spec.shard_map.version);
    assert_eq!(status.max_observed_percent, Some(60));
    assert_eq!(status.blocking_conditions, vec!["prepareThresholdCrossed"]);
    assert!(status.message.contains("prepare threshold crossed"));
}

#[test]
fn reshard_status_with_usage_reports_urgent_threshold_crossed() {
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 900_000u64); // 90%: past urgent(85)
    let status = spec.reshard_status_with_usage(&usage, spec.shard_map.version);
    assert_eq!(status.max_observed_percent, Some(90));
    assert_eq!(status.blocking_conditions, vec!["urgentThresholdCrossed"]);
    assert!(status.message.contains("urgent threshold crossed"));
}

#[test]
fn reshard_status_with_usage_picks_the_busiest_shard() {
    let mut spec = dev_spec();
    spec.shard_count = 3;
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 100_000u64);
    usage.insert(1u32, 950_000u64); // busiest: 95%, urgent
    usage.insert(2u32, 400_000u64);
    let status = spec.reshard_status_with_usage(&usage, spec.shard_map.version);
    assert_eq!(status.max_observed_percent, Some(95));
    assert!(status.message.contains("shard 1"));
}

#[test]
fn reshard_status_with_usage_holds_on_pre_cutover_measurement() {
    // #1386 R1/R3: a measurement tagged with an older `shardMap.version` than
    // the CR's current one (the shard-usage cache right after a split
    // completes, before the next scrape) must not report a crossed
    // threshold even though the raw percentage is well past urgent — and
    // the status must visibly say so (not silently look idle).
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    spec.shard_map.version = 1; // post-cutover
    let mut usage = BTreeMap::new();
    usage.insert(0u32, 900_000u64); // 90%: past urgent(85), but stale
    let status = spec.reshard_status_with_usage(&usage, 0 /* pre-cutover measurement */);
    assert_eq!(status.max_observed_percent, Some(90));
    assert_eq!(status.usage_measured_at_map_version, Some(0));
    assert_eq!(status.blocking_conditions, vec!["usageStalePostCutover"]);
    assert!(status
        .message
        .contains("holding for a fresh post-cutover measurement"));
}

#[test]
fn reshard_status_with_usage_reports_urgent_after_fresh_post_cutover_measurement() {
    // #1386 R2: once the usage cache carries a measurement tagged with the
    // CR's *current* `shardMap.version`, a genuinely still-hot shard is
    // reported normally and can legitimately trigger the next split.
    let mut spec = dev_spec();
    spec.reshard_policy.max_shard_bytes = Some(1_000_000);
    spec.shard_map.version = 1; // post-cutover
    let mut usage = BTreeMap::new();
    usage.insert(1u32, 900_000u64); // 90%: past urgent(85), fresh
    let status =
        spec.reshard_status_with_usage(&usage, 1 /* fresh: matches shardMap.version */);
    assert_eq!(status.max_observed_percent, Some(90));
    assert_eq!(status.usage_measured_at_map_version, Some(1));
    assert_eq!(status.blocking_conditions, vec!["urgentThresholdCrossed"]);
    assert!(status.message.contains("urgent threshold crossed"));
}

#[test]
fn shard_map_assignments_are_exposed_to_serving_config() {
    let mut spec = dev_spec();
    spec.shard_count = 2;
    spec.shard_map = ShardMapSpec {
        version: 7,
        virtual_bucket_count: 4,
        assignments: vec![0, 1, 1, 0],
    };
    let l = lumen("search", spec);
    let objs = render(&l);
    let cm = find(&objs, "ConfigMap", "search-config");

    assert_eq!(cm["data"]["SHARD_MAP_VERSION"], "7");
    assert_eq!(cm["data"]["VIRTUAL_BUCKET_COUNT"], "4");
    assert_eq!(cm["data"]["SHARD_MAP_ASSIGNMENTS"], "0,1,1,0");

    // #1384: once assignments are non-empty, the serving container env must
    // reference SHARD_MAP_ASSIGNMENTS too (not just the ConfigMap), so a pod
    // started/restarted after this commits actually routes by it via
    // `lumen::config::shard_map_from_env`.
    let sts = find(&objs, "StatefulSet", "search");
    let c = &sts["spec"]["template"]["spec"]["containers"][0];
    let names = env_names(c);
    for required in [
        "SHARD_MAP_VERSION",
        "VIRTUAL_BUCKET_COUNT",
        "SHARD_MAP_ASSIGNMENTS",
    ] {
        assert!(
            names.contains(&required.to_string()),
            "missing env {required}; have {names:?}"
        );
    }
}

#[test]
fn relay_objects_are_not_rendered() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    // No managed Relay objects at all: Lumen owns HA via raft-runtime.
    assert!(!has(&objs, "StatefulSet", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay-headless"));
    assert!(!has(&objs, "PodDisruptionBudget", "search-relay"));
}

#[test]
fn raft_ha_renders_serving_statefulset() {
    // `replicasPerShard > 1` switches the serving fleet from a Deployment+HPA to a
    // raft-HA StatefulSet whose pods carry the downward-API env raft_runtime::cluster
    // reads — the operator↔raft-runtime wiring, end to end.
    let mut spec = dev_spec();
    spec.shard_count = 2;
    spec.replicas_per_shard = 3;
    spec.voter_count = 3;
    let l = lumen("search", spec);
    let objs = render(&l);

    // The serving fleet is now a StatefulSet + headless Service; no Deployment/HPA.
    assert!(
        has(&objs, "StatefulSet", "search"),
        "got {:?}",
        kinds(&objs)
    );
    assert!(has(&objs, "Service", "search-headless"));
    assert!(!has(&objs, "Deployment", "search"));
    assert!(!has(&objs, "HorizontalPodAutoscaler", "search"));

    let sts = find(&objs, "StatefulSet", "search");
    assert_eq!(sts["spec"]["serviceName"], "search-headless");
    assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");
    assert_eq!(sts["spec"]["replicas"], 6); // shard_count(2) × replicasPerShard(3)

    // Exactly the env `raft_runtime::cluster::ClusterTopology::from_env` reads.
    let env = env_names(&sts["spec"]["template"]["spec"]["containers"][0]);
    for k in [
        "POD_NAME",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "LUMEN_HEADLESS_SERVICE",
    ] {
        assert!(env.contains(&k.to_string()), "missing {k} in {env:?}");
    }
    // #1387 regression: raft mode is already PVC-backed via
    // `LUMEN_RAFT_DATA_DIR` (out of scope) — the embedded-mode data-dir env
    // only applies at `replicasPerShard <= 1` and must stay absent here.
    for absent in ["LUMEN_DATA_DIR", "LUMEN_PERSISTENCE"] {
        assert!(
            !env.contains(&absent.to_string()),
            "unexpected embedded-persistence env {absent} in raft mode; have {env:?}"
        );
    }

    // The raft PVC shape is unchanged by #812 — it was already unconditional
    // in the raft-HA regime. Still unchanged by #809: `service_k8s::resize`
    // only reads this rendered `raft-<name>-<ordinal>` PVC shape, it never
    // alters render()'s output.
    let vcts = sts["spec"]["volumeClaimTemplates"].as_array().unwrap();
    assert_eq!(vcts.len(), 1);
    assert_eq!(vcts[0]["metadata"]["name"], "raft");
}

/// `crd_yaml()` ships every custom resource as ONE multi-document file, so a
/// single `kubectl apply -f` installs the whole API. The schema assertions
/// below are about the `Lumen` CRD specifically, so they must say so: parsing
/// the file as a single document stopped compiling the moment `LumenFleet`
/// joined it ("deserializing from YAML containing more than one document is not
/// supported"), and picking document [0] would silently start asserting against
/// whichever CRD happens to be serialized first.
fn crd_document(name: &str) -> serde_yaml::Value {
    let yaml = lumen::operator::crd_yaml();
    let mut found = serde_yaml::Deserializer::from_str(&yaml).filter_map(|doc| {
        let value = serde_yaml::Value::deserialize(doc).expect("each CRD document parses as YAML");
        (value["metadata"]["name"] == name).then_some(value)
    });
    let doc = found
        .next()
        .unwrap_or_else(|| panic!("crd_yaml() has no document named `{name}`"));
    assert!(
        found.next().is_none(),
        "crd_yaml() has more than one document named `{name}`"
    );
    doc
}

#[test]
fn crd_yaml_emits_lumen_definition() {
    let yaml = lumen::operator::crd_yaml();
    assert!(yaml.contains("kind: CustomResourceDefinition"));
    assert!(
        yaml.contains("lumens.lumen.dev"),
        "CRD name should be plural.group: {yaml}"
    );
    assert!(yaml.contains("v1alpha1"));
    assert!(
        !yaml.contains("format: uint32") && !yaml.contains("format: uint64"),
        "Kubernetes OpenAPI does not recognize unsigned integer formats: {yaml}"
    );
    for needle in ["shardMap", "reshardPolicy", "PrepareSplit"] {
        assert!(
            yaml.contains(needle),
            "CRD should publish the reshard surface; missing `{needle}`: {yaml}"
        );
    }
    // #2870: the CRD no longer teaches the retired registry. `tokensSecret` and
    // `identities` survive as no-op fields until #2872 drops them from the
    // schema, but their docs must not describe a mount the operator stopped
    // rendering — that is the ambiguity phase 1 exists to remove.
    for needle in [
        "token-registry.json",
        "/var/run/secrets/lumen/token-registry.json",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "LUMEN_IDENTITY_REGISTRY_FILE",
    ] {
        assert!(
            !yaml.contains(needle),
            "CRD must not publish the retired registry shape; found `{needle}`: {yaml}"
        );
    }
}

/// The checked-in `k8s/operator/crd.yaml` is what a kustomize user applies —
/// the library's `crd_yaml()` is what the tests above assert against. Nothing
/// held the two together, and they had come apart: at the commit before
/// #2678's render the file carried the pre-R4 `default: disabled` and no
/// `x-kubernetes-validations` at all, so R7's mutual-exclusion rule existed in
/// `crd.rs`, passed its own test, and still shipped a CRD that accepted both
/// token sources. Every other manifest under `k8s/operator/` is already
/// `include_str!`-gated; this closes the one that was not.
#[test]
fn checked_in_crd_yaml_matches_the_renderer() {
    let rendered = lumen::operator::crd_yaml();
    let checked_in = include_str!("../k8s/operator/crd.yaml");
    assert_eq!(
        checked_in, rendered,
        "apps/lumen/k8s/operator/crd.yaml is stale — regenerate with \
         `lumen k8s crd render --out apps/lumen/k8s/operator/crd.yaml`"
    );
}

/// A stateful search workload wants a dedicated node pool — local SSD, high
/// memory — and until `spec.placement` existed there was no way to ask for one:
/// the StatefulSet is operator-rendered, so a `kubectl patch` adding a
/// `nodeSelector` is reverted on the next reconcile.
///
/// The second half is the assertion that matters. Naming a pool must not cost
/// the pod anti-affinity, which is what keeps two replicas of one shard off the
/// same host. A spec that exposed the whole `affinity` block instead would let a
/// deployer replace that constraint while asking only for a pool, rendering a
/// StatefulSet that still reads as correct and loses both replicas of a shard to
/// the first node failure.
#[test]
fn placement_names_a_node_pool_without_replacing_the_anti_affinity() {
    let mut spec = prod_spec();
    spec.replicas_per_shard = 3;
    spec.voter_count = 3;
    spec.placement = PlacementSpec {
        node_selector: BTreeMap::from([(
            "cloud.google.com/gke-nodepool".to_string(),
            "lumen-ssd".to_string(),
        )]),
        tolerations: vec![Toleration {
            key: Some("dedicated".into()),
            operator: Some("Equal".into()),
            value: Some("lumen".into()),
            effect: Some("NoSchedule".into()),
            toleration_seconds: None,
        }],
    };
    let objs = render(&lumen("search", spec));
    let pod = &find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"];

    assert_eq!(
        pod["nodeSelector"]["cloud.google.com/gke-nodepool"],
        "lumen-ssd"
    );
    assert_eq!(pod["tolerations"][0]["key"], "dedicated");
    assert_eq!(pod["tolerations"][0]["operator"], "Equal");
    assert_eq!(pod["tolerations"][0]["value"], "lumen");
    assert_eq!(pod["tolerations"][0]["effect"], "NoSchedule");
    // Unset fields are omitted, not rendered as explicit `null`, so the
    // toleration is byte-identical to what a hand-written pod spec carries and
    // repeated applies produce no diff.
    assert!(pod["tolerations"][0].get("tolerationSeconds").is_none());

    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["topologyKey"],
        "kubernetes.io/hostname",
        "naming a node pool must not cost the one-replica-per-host constraint"
    );
}

/// Placement is additive: a CR that does not ask for a pool renders the pod spec
/// it rendered before, so every existing instance adopts the new operator
/// without a rolling restart it did not ask for.
#[test]
fn no_placement_leaves_the_pod_spec_as_it_was() {
    let objs = render(&lumen("search", prod_spec()));
    let pod = &find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"];
    assert!(pod.get("nodeSelector").is_none(), "{pod}");
    assert!(pod.get("tolerations").is_none(), "{pod}");
}

/// A renderer that accepts `spec.placement` is worth nothing if the CRD the API
/// server validates against prunes it — the exact failure #2678 found in
/// `x-kubernetes-validations`. `checked_in_crd_yaml_matches_the_renderer` holds
/// the file to the renderer; this holds the schema to the field.
#[test]
fn the_crd_accepts_placement() {
    let doc = crd_document("lumens.lumen.dev");
    let placement = &doc["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]["spec"]
        ["properties"]["placement"];

    // `default: {}` is what keeps every CR written before this field valid.
    assert!(
        placement["default"].is_mapping(),
        "placement must default to empty so existing CRs still validate: {placement:?}"
    );
    let props = &placement["properties"];
    assert_eq!(props["nodeSelector"]["type"], "object");
    assert_eq!(props["nodeSelector"]["additionalProperties"]["type"], "string");
    assert_eq!(props["tolerations"]["type"], "array");
    for field in ["key", "operator", "value", "effect", "tolerationSeconds"] {
        assert!(
            props["tolerations"]["items"]["properties"]
                .get(field)
                .is_some(),
            "toleration schema must carry `{field}`"
        );
    }

    // The whole point of the narrow surface: `affinity` stays operator-owned,
    // so there is no CR field that can replace the anti-affinity.
    assert!(
        doc["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]["spec"]["properties"]
            .get("affinity")
            .is_none(),
        "exposing spec.affinity would let a deployer drop the anti-affinity"
    );
}

#[test]
fn crd_backup_schema_flattens_shared_policy() {
    let doc = crd_document("lumens.lumen.dev");
    let backup_props = &doc["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]
        ["spec"]["properties"]["serving"]["properties"]["backup"]["properties"];
    for field in [
        "schedule",
        "destination",
        "retentionSecs",
        "adminTokenSecret",
    ] {
        assert!(
            backup_props.get(field).is_some(),
            "shared backup schema must keep flat field `{field}`"
        );
    }
    assert_eq!(backup_props["destination"]["type"], "string");
}

#[test]
fn crd_schema_reshard_recommendation_only_default_matches_runtime_default() {
    // #1319 R3: the declared schema default for `status.reshard.recommendationOnly`
    // must agree with the runtime default (`ReshardPolicy::default().max_shard_bytes
    // .is_none() == true`, i.e. `LumenSpec::default().reshard_status()
    // .recommendation_only == true` — see `dev_spec()`/`prod_spec()`, neither of
    // which set `maxShardBytes`), not `bool::default()` (`false`).
    let spec = dev_spec();
    assert!(
        spec.reshard_status().recommendation_only,
        "runtime default: recommendationOnly should be true when maxShardBytes is unset"
    );

    let doc = crd_document("lumens.lumen.dev");
    let versions = doc["spec"]["versions"].as_sequence().expect("versions");
    let v1alpha1 = versions
        .iter()
        .find(|v| v["name"] == "v1alpha1")
        .expect("v1alpha1 version");
    let recommendation_only = &v1alpha1["schema"]["openAPIV3Schema"]["properties"]["status"]
        ["properties"]["reshard"]["properties"]["recommendationOnly"];
    assert_eq!(
        recommendation_only["default"],
        serde_yaml::Value::Bool(true),
        "declared schema default must match the runtime default: {recommendation_only:?}"
    );
}

#[test]
fn no_backup_cronjob_when_unset() {
    // #808 R2: `spec.serving.backup` absent (the `dev_spec`/`prod_spec`
    // default, via `ServingSpec::default()`) renders no CronJob at all.
    for spec in [dev_spec(), prod_spec()] {
        let l = lumen("search", spec);
        let objs = render(&l);
        assert!(
            !has(&objs, "CronJob", "search-backup"),
            "unexpected backup CronJob with no serving.backup policy: {:?}",
            kinds(&objs)
        );
    }
}

#[test]
fn backup_cronjob_wires_schedule_and_destination() {
    // #808 R3: `spec.serving.backup` set renders exactly one `batch/v1`
    // CronJob named `<name>-backup` with the configured schedule and a
    // `lumen backup --url <cluster-dns-fqdn> --dest <destination>` args list.
    let mut spec = dev_spec();
    spec.serving.backup = Some(lumen::operator::crd::ServingBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "0 * * * *".into(),
            destination: "s3://my-bucket/lumen-backups".into(),
            retention_secs: None,
        },
        admin_token_secret: None,
    });
    let l = lumen("search", spec);
    let objs = render(&l);

    assert_eq!(
        objs.iter().filter(|o| o["kind"] == "CronJob").count(),
        1,
        "expected exactly one CronJob; got {:?}",
        kinds(&objs)
    );
    let cj = find(&objs, "CronJob", "search-backup");
    assert_eq!(cj["apiVersion"], "batch/v1");
    assert_eq!(cj["spec"]["schedule"], "0 * * * *");

    let c = &cj["spec"]["jobTemplate"]["spec"]["template"]["spec"]["containers"][0];
    let args: Vec<String> = c["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        args,
        vec![
            "backup",
            "--url",
            "http://search.acme.svc.cluster.local:7373",
            "--dest",
            "s3://my-bucket/lumen-backups",
        ]
    );

    // Owner reference + namespace still flow through the shared render toolkit.
    assert_eq!(cj["metadata"]["namespace"], "acme");
    let owner = &cj["metadata"]["ownerReferences"][0];
    assert_eq!(owner["kind"], "Lumen");
    assert_eq!(owner["uid"], "uid-1234");
}

/// #2872 renamed this from `..._and_audiences_env`: the audience env var it
/// checked for came from `spec.identityAudiences`, and both are gone. The
/// CronJob now carries no credential env at all — asserted, because "no token
/// is injected" is the whole security claim on this path.
#[test]
fn backup_cronjob_wires_retention_and_carries_no_credential_env() {
    let mut spec = dev_spec();
    spec.serving.backup = Some(lumen::operator::crd::ServingBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "@daily".into(),
            destination: "file:///backups/lumen".into(),
            retention_secs: Some(604800),
        },
        admin_token_secret: None,
    });
    let l = lumen("search", spec);
    let objs = render(&l);
    let cj = find(&objs, "CronJob", "search-backup");
    let c = &cj["spec"]["jobTemplate"]["spec"]["template"]["spec"]["containers"][0];

    let args: Vec<String> = c["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        args.windows(2)
            .any(|w| w[0] == "--retention-secs" && w[1] == "604800"),
        "missing --retention-secs 604800 in {args:?}"
    );

    let env = env_names(c);
    for retired in [
        "BACKUP_TOKEN",
        "LUMEN_AUTH_GOOGLE_AUDIENCES",
        "LUMEN_TOKEN_REGISTRY_FILE",
    ] {
        assert!(
            !env.iter().any(|e| e.contains(retired)),
            "CronJob should carry no `{retired}` env var: {env:?}"
        );
    }
}

#[test]
fn service_account_annotations_pass_through_to_both_service_accounts() {
    let mut spec = dev_spec();
    spec.service_account_annotations.insert(
        "iam.gke.io/gcp-service-account".to_string(),
        "lumen-sa@project.iam.gserviceaccount.com".to_string(),
    );
    let l = lumen("search", spec);
    let objs = render(&l);

    let sa = find(&objs, "ServiceAccount", "search");
    assert_eq!(
        sa["metadata"]["annotations"]["iam.gke.io/gcp-service-account"],
        "lumen-sa@project.iam.gserviceaccount.com"
    );

    let bsa = find(&objs, "ServiceAccount", "search-backup");
    assert_eq!(
        bsa["metadata"]["annotations"]["iam.gke.io/gcp-service-account"],
        "lumen-sa@project.iam.gserviceaccount.com"
    );
}

#[test]
fn bootstrap_seed_policy_wires_serving_env() {
    let mut spec = dev_spec();
    spec.serving.bootstrap = Some(ServingBootstrapSpec {
        seed_uri: "file:///seed/snapshot.json".into(),
        max_bytes_per_sec: Some(1_048_576),
    });
    let l = lumen("search", spec);
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");
    let env = sts["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .unwrap();

    let seed = env
        .iter()
        .find(|e| e["name"] == "LUMEN_BOOTSTRAP_SEED_URI")
        .expect("bootstrap seed env");
    assert_eq!(seed["value"], "file:///seed/snapshot.json");
    let limit = env
        .iter()
        .find(|e| e["name"] == "LUMEN_BOOTSTRAP_MAX_BYTES_PER_SEC")
        .expect("bootstrap throttle env");
    assert_eq!(limit["value"], "1048576");
}

/// #2477: `spec.admission` set renders the matching `LUMEN_ADMISSION_*` envs,
/// one per configured field, with no envs for the fields left unset.
#[test]
fn admission_spec_wires_serving_env() {
    let mut spec = dev_spec();
    spec.admission = Some(lumen::operator::crd::AdmissionSpec {
        read_capacity: Some(500),
        write_capacity: Some(100),
        admin_capacity: None,
        refill_secs: Some(30),
        max_keys: None,
    });
    let l = lumen("search", spec);
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");
    let env = sts["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .unwrap();
    let value_of = |name: &str| {
        env.iter()
            .find(|e| e["name"] == name)
            .and_then(|e| e["value"].as_str())
            .unwrap_or_else(|| panic!("env {name} missing a literal value"))
            .to_string()
    };

    assert_eq!(value_of("LUMEN_ADMISSION_READ_CAPACITY"), "500");
    assert_eq!(value_of("LUMEN_ADMISSION_WRITE_CAPACITY"), "100");
    assert_eq!(value_of("LUMEN_ADMISSION_REFILL_SECS"), "30");
    let names = env_names(&sts["spec"]["template"]["spec"]["containers"][0]);
    for absent in ["LUMEN_ADMISSION_ADMIN_CAPACITY", "LUMEN_ADMISSION_MAX_KEYS"] {
        assert!(
            !names.contains(&absent.to_string()),
            "unset admission field must not render {absent}: {names:?}"
        );
    }
}

/// #2477: `spec.admission` absent (the `dev_spec`/`prod_spec` default) renders
/// none of the `LUMEN_ADMISSION_*` envs — pure exposure, no default-on
/// behavior change.
#[test]
fn admission_spec_absent_renders_no_admission_env() {
    for spec in [dev_spec(), prod_spec()] {
        let l = lumen("search", spec);
        let objs = render(&l);
        let sts = find(&objs, "StatefulSet", "search");
        let names = env_names(&sts["spec"]["template"]["spec"]["containers"][0]);
        for absent in [
            "LUMEN_ADMISSION_READ_CAPACITY",
            "LUMEN_ADMISSION_WRITE_CAPACITY",
            "LUMEN_ADMISSION_ADMIN_CAPACITY",
            "LUMEN_ADMISSION_REFILL_SECS",
            "LUMEN_ADMISSION_MAX_KEYS",
        ] {
            assert!(
                !names.contains(&absent.to_string()),
                "no spec.admission must render no {absent}: {names:?}"
            );
        }
    }
}

/// #2603: turning `networkPolicy` off must *remove* the policy, not merely stop
/// rendering it.
///
/// Server-side apply reconciles fields, never object lifetime, so a child that
/// drops out of `render` keeps running until the CR is deleted. For a
/// NetworkPolicy that made the field opt-in only: enforcement started and could
/// never be stopped. `prunes` is the inverse of the render branch, and this
/// pins both directions.
#[test]
fn network_policy_off_prunes_the_policy_render_no_longer_emits() {
    let mut spec = prod_spec();
    spec.network_policy = true;
    let on = lumen("search", spec.clone());
    assert!(
        has(&render(&on), "NetworkPolicy", "search"),
        "networkPolicy=true must render the policy"
    );
    assert!(
        prunes(&on).is_empty(),
        "a CR that still wants the policy must never nominate it for deletion"
    );

    spec.network_policy = false;
    let off = lumen("search", spec);
    assert!(
        !has(&render(&off), "NetworkPolicy", "search"),
        "networkPolicy=false must stop rendering the policy"
    );
    assert_eq!(
        prunes(&off),
        vec![PruneTarget {
            api_version: "networking.k8s.io/v1",
            kind: "NetworkPolicy",
            name: "search".to_string(),
        }],
        "networkPolicy=false must nominate exactly the policy for deletion"
    );
}

/// #2603 anti-drift: the pruned name is the rendered name.
///
/// The prune target is built from the CR name independently of
/// `serving_network_policy`, so a future change to how the policy is named
/// would silently orphan every policy already in a cluster — render would emit
/// the new name while prune kept deleting the old one, and neither side would
/// fail to compile. Deriving both from a real render and comparing is what
/// makes that a test failure instead of a production surprise.
#[test]
fn pruned_network_policy_name_matches_the_rendered_one() {
    for name in ["search", "lumen", "a-much-longer-instance-name"] {
        let mut spec = prod_spec();
        spec.network_policy = true;
        let rendered = render(&lumen(name, spec.clone()));
        let policy = rendered
            .iter()
            .find(|o| o["kind"] == "NetworkPolicy")
            .expect("networkPolicy=true renders one");

        spec.network_policy = false;
        let targets = prunes(&lumen(name, spec));
        assert_eq!(targets.len(), 1);
        assert_eq!(
            Value::from(targets[0].name.clone()),
            policy["metadata"]["name"],
            "prune must target the name render actually uses"
        );
        assert_eq!(
            Value::from(targets[0].api_version),
            policy["apiVersion"],
            "prune must target the apiVersion render actually uses"
        );
        assert_eq!(Value::from(targets[0].kind), policy["kind"]);
    }
}
// CODEGEN-END

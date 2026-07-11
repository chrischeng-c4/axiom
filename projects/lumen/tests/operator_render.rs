// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, Lumen-owned raft wiring, and observability toggles.
#![cfg(feature = "operator")]

use std::collections::BTreeMap;

use kube::api::ObjectMeta;
use lumen::operator::crd::{
    AuthMode, Autoscaling, LogFormat, ReshardPhase, ReshardPolicy, ReshardWorkflowSpec,
    ServingBootstrapSpec, ServingSpec, ShardMapSpec,
};
use lumen::operator::render::render;
use lumen::operator::{Lumen, LumenSpec};
use serde_json::Value;

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
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        tokens_secret: None,
        tokens_secret_provider_class: None,
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
    }
}

fn prod_spec() -> LumenSpec {
    LumenSpec {
        image: "registry.example.com/lumen:1.2.3".into(),
        image_pull_policy: Some("Always".into()),
        shard_count: 6,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Json,
        log_level: Some("warn".into()),
        auth: AuthMode::Required,
        tokens_secret: Some("lumen-tokens".into()),
        tokens_secret_provider_class: None,
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
    // serving fleet is a StatefulSet even at replicasPerShard:1 (#812) — its
    // headless Service and the ClusterIP Service and HPA are all still
    // rendered for the single-member regime.
    for (kind, name) in [
        ("ServiceAccount", "search"),
        ("ConfigMap", "search-config"),
        ("StatefulSet", "search"),
        ("Service", "search-headless"),
        ("Service", "search"),
        ("HorizontalPodAutoscaler", "search"),
        ("PodDisruptionBudget", "search"),
    ] {
        assert!(
            has(&objs, kind, name),
            "expected {kind}/{name}; got {:?}",
            kinds(&objs)
        );
    }
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
fn statefulset_wires_serving_contract_single_member() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let sts = find(&objs, "StatefulSet", "search");

    // HPA floor == apply-time replicas; StatefulSet-native rollout knobs.
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
    assert_eq!(c["ports"][0]["containerPort"], 7373);

    // Guaranteed QoS: requests == limits, from the spec.
    assert_eq!(c["resources"]["requests"]["cpu"], "2");
    assert_eq!(c["resources"]["limits"]["cpu"], "2");
    assert_eq!(
        c["resources"]["requests"]["memory"],
        c["resources"]["limits"]["memory"]
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
    // auth=off and no log level → those env vars are absent.
    assert!(!names.contains(&"LUMEN_TOKENS".to_string()));
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
    // exact `raft-<name>-` prefix `operator::resize::resize_instance` filters
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
fn hpa_is_rendered_for_single_replica_serving() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    // #1317: at replicasPerShard<=1 with shardCount<=1 there is no raft
    // consensus, so the HPA's bounds are clamped to exactly 1/1 regardless
    // of the CR's `serving.autoscaling` values (dev_spec sets min=1/max=3) —
    // more than one live pod here would be an uncoordinated shard-0 copy.
    // Confirmed empirically on a kind cluster: with minReplicas raised above
    // 1, the resulting StatefulSet pods each hold independent local state
    // and the fronting Service returns divergent results for identical
    // reads.
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 1);
    assert_eq!(hpa["spec"]["scaleTargetRef"]["name"], "search");
    // The serving fleet is a StatefulSet (#812) — the HPA must target it, not
    // the retired Deployment kind.
    assert_eq!(hpa["spec"]["scaleTargetRef"]["kind"], "StatefulSet");
}

#[test]
fn single_member_hpa_and_storage_pod_count_clamp_to_one_regardless_of_autoscaling_bounds() {
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
    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 1);

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
    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 1);
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

    // auth=required + tokensSecret → registry file env + Secret volume mount.
    let dep = find(&objs, "StatefulSet", "lumen");
    let c = &dep["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(c["image"], "registry.example.com/lumen:1.2.3");
    assert_eq!(c["imagePullPolicy"], "Always");
    let registry_env = c["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "LUMEN_TOKEN_REGISTRY_FILE")
        .expect("LUMEN_TOKEN_REGISTRY_FILE env");
    assert_eq!(
        registry_env["value"],
        "/var/run/secrets/lumen/token-registry.json"
    );
    let registry_mount = c["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "lumen-token-registry")
        .expect("token registry mount");
    assert_eq!(registry_mount["mountPath"], "/var/run/secrets/lumen");
    assert_eq!(registry_mount["readOnly"], true);
    let registry_volume = dep["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "lumen-token-registry")
        .expect("token registry volume");
    assert_eq!(registry_volume["secret"]["secretName"], "lumen-tokens");
    assert_eq!(
        registry_volume["secret"]["items"][0]["key"],
        "token-registry.json"
    );
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
}

#[test]
fn prod_wires_auth_via_csi_secret_provider_class() {
    let mut spec = prod_spec();
    spec.tokens_secret = None;
    spec.tokens_secret_provider_class = Some("lumen-tokens-spc".into());
    let l = lumen("lumen", spec);
    let objs = render(&l);

    // auth=required + tokensSecretProviderClass (no tokensSecret) → registry
    // file env + CSI volume mount, same mount path/readOnly as the Secret path.
    let dep = find(&objs, "StatefulSet", "lumen");
    let c = &dep["spec"]["template"]["spec"]["containers"][0];
    let registry_env = c["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "LUMEN_TOKEN_REGISTRY_FILE")
        .expect("LUMEN_TOKEN_REGISTRY_FILE env");
    assert_eq!(
        registry_env["value"],
        "/var/run/secrets/lumen/token-registry.json"
    );
    let registry_mount = c["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "lumen-token-registry")
        .expect("token registry mount");
    assert_eq!(registry_mount["mountPath"], "/var/run/secrets/lumen");
    assert_eq!(registry_mount["readOnly"], true);
    let registry_volume = dep["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "lumen-token-registry")
        .expect("token registry volume");
    assert!(
        registry_volume["secret"].is_null(),
        "CSI-sourced volume must not carry a secret key: {registry_volume}"
    );
    assert_eq!(registry_volume["csi"]["driver"], "secrets-store.csi.k8s.io");
    assert_eq!(registry_volume["csi"]["readOnly"], true);
    assert_eq!(
        registry_volume["csi"]["volumeAttributes"]["secretProviderClass"],
        "lumen-tokens-spc"
    );
}

#[test]
fn tokens_secret_wins_over_provider_class_when_both_set() {
    let mut spec = prod_spec();
    spec.tokens_secret_provider_class = Some("lumen-tokens-spc".into());
    let l = lumen("lumen", spec);
    let objs = render(&l);

    // Both set → tokensSecret wins (backward compatible); no csi key at all.
    let dep = find(&objs, "StatefulSet", "lumen");
    let registry_volume = dep["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "lumen-token-registry")
        .expect("token registry volume");
    assert_eq!(registry_volume["secret"]["secretName"], "lumen-tokens");
    assert!(
        registry_volume["csi"].is_null(),
        "tokensSecret must win when both fields are set: {registry_volume}"
    );
}

#[test]
fn reshard_status_is_recommendation_only_without_capacity_ceiling() {
    let mut spec = dev_spec();
    spec.reshard_policy.workflow = ReshardWorkflowSpec {
        phase: ReshardPhase::PrepareSplit,
        target_shard_count: Some(2),
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

    // No managed Relay objects at all: Lumen owns HA via raft-host.
    assert!(!has(&objs, "StatefulSet", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay-headless"));
    assert!(!has(&objs, "PodDisruptionBudget", "search-relay"));
}

#[test]
fn raft_ha_renders_serving_statefulset() {
    // `replicasPerShard > 1` switches the serving fleet from a Deployment+HPA to a
    // raft-HA StatefulSet whose pods carry the downward-API env raft_host::cluster
    // reads — the operator↔raft-host wiring, end to end.
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

    // Exactly the env `raft_host::cluster::ClusterTopology::from_env` reads.
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
    // in the raft-HA regime. Still unchanged by #809: `operator::resize`
    // only reads this rendered `raft-<name>-<ordinal>` PVC shape, it never
    // alters render()'s output.
    let vcts = sts["spec"]["volumeClaimTemplates"].as_array().unwrap();
    assert_eq!(vcts.len(), 1);
    assert_eq!(vcts[0]["metadata"]["name"], "raft");
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
    for needle in [
        "token-registry.json",
        "/var/run/secrets/lumen/token-registry.json",
        "LUMEN_TOKEN_REGISTRY_FILE",
        "read|write|admin",
        "shardMap",
        "reshardPolicy",
        "PrepareSplit",
        "tokensSecretProviderClass",
        "SecretProviderClass",
        "secrets-store.csi.k8s.io",
    ] {
        assert!(
            yaml.contains(needle),
            "CRD should publish token registry shape in tokensSecret docs; missing `{needle}`: {yaml}"
        );
    }
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

    let yaml = lumen::operator::crd_yaml();
    let doc: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("valid CRD yaml");
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
        "declared schema default must match the runtime default: {yaml}"
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
        schedule: "0 * * * *".into(),
        destination: "s3://my-bucket/lumen-backups".into(),
        retention_secs: None,
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

#[test]
fn backup_cronjob_wires_retention_and_admin_token() {
    // #808 R4: `retentionSecs` becomes `--retention-secs`, and
    // `adminTokenSecret` becomes a `LUMEN_BACKUP_TOKEN` env var sourced from
    // that Secret's `token` key.
    let mut spec = dev_spec();
    spec.serving.backup = Some(lumen::operator::crd::ServingBackupSpec {
        schedule: "@daily".into(),
        destination: "file:///backups/lumen".into(),
        retention_secs: Some(604800),
        admin_token_secret: Some("lumen-backup-token".into()),
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
    assert!(
        env.contains(&"LUMEN_BACKUP_TOKEN".to_string()),
        "missing LUMEN_BACKUP_TOKEN in {env:?}"
    );
    let token_env = c["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "LUMEN_BACKUP_TOKEN")
        .unwrap();
    assert_eq!(
        token_env["valueFrom"]["secretKeyRef"]["name"],
        "lumen-backup-token"
    );
    assert_eq!(token_env["valueFrom"]["secretKeyRef"]["key"], "token");
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
// CODEGEN-END

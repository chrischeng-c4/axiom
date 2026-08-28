// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, Lumen-owned raft wiring, and observability toggles.
//!
//! ## Contracts inherited from the retired EC shells
//!
//! These 9 sentences were the whole of the `// Contract:` comment in 9 AW-EC shells
//! under `apps/lumen/e2e/`, each of which ran `cargo test -p lumen --features operator
//! --test operator_render` in a subprocess and asserted the child's exit status.
//!
//! Until 2026-08-20 these shells could not be deleted. The project's only declared gate
//! was `cargo test -p lumen`, and with `default = []` that command compiled every
//! `#![cfg(feature = "operator")]` target into an empty binary that printed `0 passed`
//! and exited 0 — so the shells were the sole surviving record that these checks should
//! run at all. `apps/lumen/CONTRIBUTING.md` declared `cargo test -p lumen --features
//! "operator delegated-auth"` as a required second gate row that day, and that run
//! executes this target directly. That made each shell a second, nested run of a target
//! the gate already covers, so they were deleted the same day. The sentence is the only
//! thing they held that nothing else did. Each line below is prefixed with the EC id
//! its shell was filed under.
//!
//! - `lumen-claim-cli-deployment-operator-command-surface` — The operator-facing
//!   command surface renders CRD and serving objects used by the deployment path.
//! - `lumen-claim-dynamic-post-cutover-usage-freshness` — A pre-cutover usage sample
//!   cannot trigger another split; a fresh generation can.
//! - `lumen-claim-dynamic-serve-shard-map` — The operator-delivered shard map is
//!   projected into the serving process configuration.
//! - `lumen-claim-dynamic-single-member-persistence` — A single-member topology renders
//!   as a durable StatefulSet with its serving storage contract.
//! - `lumen-claim-dynamic-storage-pressure-split-policy` — The operator render gate
//!   proves rendering topology conformance: storage-pressure reshard recommendations
//!   compute correctly without changing HPA-owned serving scale (rendering only —
//!   reshard driver execution, admin verbs, and migration durability are covered by the
//!   dedicated reshard-durability gate).
//! - `lumen-claim-k8s-operator-reconcile` — The kube-rs operator render path proves
//!   rendering topology conformance: Lumen CRD inputs map to serving resources,
//!   including storage-pressure reshard policy, status phases, and fixed storage
//!   topology (rendering only — the live reconcile loop, reshard driver, and admin
//!   verbs are covered by the dedicated reshard-durability gate).
//! - `lumen-claim-k8s-operator-storage-topology-reshard` — The operator render gate
//!   proves rendering topology conformance: fixed StatefulSet storage topology and
//!   reshard status exposure (rendering only — reshard driver execution, admin verbs,
//!   and migration durability are covered by the dedicated reshard-durability gate).
//! - `lumen-claim-k8s-single-member-persistence` — The Kubernetes instance renderer
//!   gives a single-member service durable StatefulSet storage.
//! - `lumen-long-running-stability-operator-render` — render(Lumen) emits the managed
//!   serving Deployment/Service/HPA/PDB plus the Relay StatefulSet/Service/PDB when the
//!   broker is managed.
#![cfg(feature = "operator")]

use std::collections::BTreeMap;

use kube::api::ObjectMeta;
use lumen::operator::crd::{
    AuthMode, LogFormat, LumenStatus, PlacementSpec, ReshardPhase, ReshardPolicy,
    ReshardWorkflowSpec, ServingBootstrapSpec, ServingSpec, ShardMapSpec, Toleration,
};
use lumen::operator::render::{
    auth_delegator_binding, auth_delegator_binding_name, auth_delegator_labels, prunes, render,
};
use lumen::operator::{Lumen, LumenSpec};
use serde::Deserialize;
use serde_json::Value;
use service_k8s::service::PruneTarget;
use service_k8s::{ConditionFact, ConditionStatus, ManagedService, ReadyFacts};

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
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: BTreeMap::new(),
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
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
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
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

// ---- #2876: the serving KSA's delegated-review grant -----------------------

#[test]
fn auth_delegator_binding_grants_exactly_the_serving_service_account() {
    // AC1: one ClusterRoleBinding, bound to the built-in delegator role, with
    // the instance's own serving ServiceAccount as its only subject.
    let l = lumen("search", dev_spec());
    let binding = auth_delegator_binding(&l);

    assert_eq!(binding["kind"], "ClusterRoleBinding");
    assert_eq!(binding["roleRef"]["kind"], "ClusterRole");
    assert_eq!(binding["roleRef"]["name"], "system:auth-delegator");
    assert_eq!(
        binding["subjects"],
        serde_json::json!([{
            "kind": "ServiceAccount",
            "name": "search",
            "namespace": "acme",
        }]),
        "got {binding}"
    );
}

#[test]
fn auth_delegator_binding_follows_an_externally_managed_service_account() {
    // The pod runs as `spec.serviceAccountName` when it is set, so the grant
    // has to follow it. Binding the instance-named SA instead would authorize
    // an identity nothing runs as, and every request would fail authentication
    // against a manifest that looks correct.
    let mut spec = dev_spec();
    spec.service_account_name = Some("external-sa".into());
    let l = lumen("search", spec);
    let objs = render(&l);
    let binding = auth_delegator_binding(&l);

    assert_eq!(binding["subjects"][0]["name"], "external-sa");
    assert_eq!(
        binding["subjects"][0]["name"],
        find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"]["serviceAccountName"],
        "the bound subject and the identity the pods run as are one answer"
    );
}

#[test]
fn auth_delegator_binding_names_no_wildcard_and_no_group_subject() {
    // AC5. `system:authenticated` or a ServiceAccount *group* would turn a
    // grant for one process into a grant for a population; `*` in the roleRef
    // would hand out every verb on every resource.
    for spec_fn in [dev_spec, prod_spec] {
        let l = lumen("search", spec_fn());
        let binding = auth_delegator_binding(&l);

        assert_ne!(binding["roleRef"]["name"], "*");
        assert_ne!(binding["roleRef"]["name"], "cluster-admin");
        assert!(
            binding.get("rules").is_none(),
            "the binding must reference the built-in role, never define one: {binding}"
        );
        let subjects = binding["subjects"].as_array().expect("subjects");
        assert_eq!(subjects.len(), 1, "got {binding}");
        for s in subjects {
            assert_eq!(s["kind"], "ServiceAccount", "no Group/User subject");
            assert_ne!(s["name"], "*");
            assert_ne!(s["name"], "system:authenticated");
            assert_ne!(s["namespace"], "*");
        }
    }
}

#[test]
fn auth_delegator_binding_is_cluster_scoped_and_unowned() {
    // A cluster-scoped object may not name a namespaced owner: the garbage
    // collector does not ignore such a reference, it reads the owner as already
    // gone and deletes the dependent. The `lumen.dev/owner-namespace` label is
    // what replaces the reference for cleanup purposes.
    let l = lumen("search", dev_spec());
    let binding = auth_delegator_binding(&l);
    let meta = binding["metadata"].as_object().expect("metadata");

    assert!(meta.get("ownerReferences").is_none(), "got {binding}");
    assert!(meta.get("namespace").is_none(), "got {binding}");
    assert_eq!(meta["labels"]["lumen.dev/owner-namespace"], "acme");
    assert_eq!(
        meta["labels"]["app.kubernetes.io/managed-by"],
        "lumen-operator"
    );
    assert_eq!(
        meta["labels"]["app.kubernetes.io/component"],
        "auth-delegation"
    );
    assert_eq!(
        serde_json::to_value(auth_delegator_labels(&l)).unwrap(),
        binding["metadata"]["labels"],
        "the sweep proves authorship by full label-set equality, so the labels \
         it recomputes must be the ones the render actually stamps"
    );
}

#[test]
fn auth_delegator_binding_names_cannot_collide_across_namespaces() {
    // The hazard a dash separator would create: `lumen-a-b-c-auth-delegator`
    // is both (ns `a-b`, name `c`) and (ns `a`, name `b-c`). Two unrelated
    // Lumens would share one binding, each granting the other's ServiceAccount
    // delegated review.
    let mut names = std::collections::BTreeSet::new();
    for (ns, name) in [
        ("a-b", "c"),
        ("a", "b-c"),
        ("team", "search"),
        ("team-search", ""),
    ] {
        let mut l = lumen(if name.is_empty() { "x" } else { name }, dev_spec());
        l.metadata.namespace = Some(ns.to_string());
        assert!(
            names.insert(auth_delegator_binding_name(&l)),
            "two distinct instances rendered the same binding name"
        );
    }
}

#[test]
fn the_auth_delegator_binding_is_never_one_of_the_namespaced_children() {
    // `render`'s objects are all applied with a namespaced API and stamped with
    // the CR's owner reference (see the assertions above). A cluster-scoped
    // binding in that list would be applied to an endpoint that rejects it, or
    // owner-stamped into deletion.
    for spec_fn in [dev_spec, prod_spec] {
        let l = lumen("search", spec_fn());
        assert!(
            !render(&l).iter().any(|o| o["kind"] == "ClusterRoleBinding"),
            "the delegated-review grant is applied on its own path, not as a child"
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
    assert_eq!(
        sts["spec"]["template"]["spec"]["enableServiceLinks"],
        false
    );
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
    assert_eq!(vcts[0]["spec"]["resources"]["requests"]["storage"], "10Gi");
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
fn single_member_storage_pod_count_clamps_to_one() {
    // #1317: single-member topology (shardCount: 1, replicasPerShard: 1)
    // must clamp to exactly 1 replica.
    let default_spec = dev_spec();
    assert_eq!(default_spec.storage_pod_count(), 1);

    let l = lumen("search", default_spec);
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

    // #2871 retired the bearer/identity registry, so the counter
    // `LumenAuthRegistryReloadFailing` read no longer exists. No alert may
    // name it — an alert whose `expr` reads a series nothing publishes is a
    // permanently-silent rule that reads like coverage.
    assert!(
        !serde_json::to_string(&rules)
            .unwrap()
            .contains("auth_registry_reload"),
        "no rule may read a retired auth-registry series (#2871)"
    );
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
        initial_machine_type: "e2-standard-2".into(),
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
    let tols = pod["tolerations"].as_array().unwrap();
    assert!(tols.iter().any(|t| t["key"] == "dedicated"
        && t["operator"] == "Equal"
        && t["value"] == "lumen"
        && t["effect"] == "NoSchedule"));

    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["topologyKey"],
        "kubernetes.io/hostname",
        "naming a node pool must not cost the one-replica-per-host constraint"
    );
    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["namespaceSelector"],
        serde_json::json!({}),
        "anti-affinity must match cross-namespace"
    );
}

/// A default CR renders cross-namespace anti-affinity.
#[test]
fn default_placement_renders_cross_namespace_anti_affinity() {
    let objs = render(&lumen("search", prod_spec()));
    let pod = &find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"];
    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["namespaceSelector"],
        serde_json::json!({}),
        "anti-affinity must match cross-namespace"
    );
    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["topologyKey"],
        "kubernetes.io/hostname"
    );
}

#[test]
fn render_with_profile_sets_capacity_profile_node_selector_and_toleration() {
    let profile = lumen::operator::capacity::ResolvedProfile {
        machine_type: "e2-standard-2".to_string(),
        selector: "lumen.axiom.dev/capacity-profile=e2-standard-2".to_string(),
        selector_key: "lumen.axiom.dev/capacity-profile".to_string(),
        selector_value: "e2-standard-2".to_string(),
        max_nodes: 10,
        min_nodes: 0,
        lifecycle_state: "ready".to_string(),
    };
    let objs = lumen::operator::render::render_with_profile(
        &lumen("search", prod_spec()),
        &profile,
    );
    let pod = &find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"];
    assert_eq!(
        pod["nodeSelector"]["lumen.axiom.dev/capacity-profile"],
        "e2-standard-2"
    );
    assert_eq!(
        pod["tolerations"][0]["key"],
        "lumen.axiom.dev/capacity-profile"
    );
    assert_eq!(pod["tolerations"][0]["value"], "e2-standard-2");
    assert_eq!(pod["tolerations"][0]["effect"], "NoSchedule");
}

#[test]
fn three_namespaces_share_same_pool_selector_and_cross_namespace_anti_affinity() {
    for ns in ["alpha", "beta", "gamma"] {
        let mut l = lumen("search", prod_spec());
        l.metadata.namespace = Some(ns.to_string());
        let objs = render(&l);
        let pod = &find(&objs, "StatefulSet", "search")["spec"]["template"]["spec"];
        assert_eq!(
            pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
                ["labelSelector"]["matchLabels"],
            serde_json::json!({
                "app.kubernetes.io/name": "lumen",
                "app.kubernetes.io/component": "server",
            })
        );
        assert_eq!(
            pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
                ["namespaceSelector"],
            serde_json::json!({})
        );
    }
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
    assert_eq!(
        props["nodeSelector"]["additionalProperties"]["type"],
        "string"
    );
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

// ---- #2890: instance-scoped Raft peer identity -------------------------
//
// Raft on `:7374` carries committed index mutations between pods, and nothing
// else on that port says who is dialing. These assert the two halves of what
// the operator owes that port: the material reaches every member, and a
// replicated instance that has none says so out loud instead of coming up
// plaintext.

/// A replicated instance — the only shape that owes a peer identity.
fn replicated_spec(secret: Option<&str>) -> LumenSpec {
    LumenSpec {
        replicas_per_shard: 3,
        voter_count: 3,
        peer_tls_secret: secret.map(str::to_string),
        ..prod_spec()
    }
}

/// The pod template of the rendered serving StatefulSet.
fn pod_spec(objs: &[Value]) -> Value {
    find(objs, "StatefulSet", "search")["spec"]["template"]["spec"].clone()
}

fn env_value(container: &Value, name: &str) -> Option<String> {
    container["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == name)
        .map(|e| e["value"].as_str().unwrap().to_string())
}

/// AC1, the volume half: the Secret is projected read-only, with exactly the
/// three keys the peer transport loads. `items` rather than a whole-Secret
/// mount is the point — a fourth key added to the Secret later must not
/// silently become part of what the container can read.
#[test]
fn peer_tls_secret_projects_exactly_its_three_keys_read_only() {
    let objs = render(&lumen("search", replicated_spec(Some("search-peer-tls"))));
    let pod = pod_spec(&objs);

    let volume = pod["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| !v["secret"].is_null())
        .unwrap_or_else(|| panic!("no Secret-backed volume in {pod:#}"));
    assert_eq!(volume["secret"]["secretName"], "search-peer-tls");
    let projected: Vec<&str> = volume["secret"]["items"]
        .as_array()
        .unwrap_or_else(|| panic!("peer TLS volume must project named items, got {volume:#}"))
        .iter()
        .map(|item| item["key"].as_str().unwrap())
        .collect();
    assert_eq!(
        projected,
        lumen::operator::render::PEER_TLS_KEYS.to_vec(),
        "the projected keys must be exactly the peer transport's contract"
    );

    let mount = pod["containers"][0]["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == volume["name"])
        .unwrap_or_else(|| panic!("the peer TLS volume is never mounted: {pod:#}"));
    assert_eq!(
        mount["readOnly"], true,
        "peer identity is credential material, not state"
    );
    let mount_path = mount["mountPath"].as_str().unwrap();
    assert!(
        !mount_path.starts_with("/var/lib/lumen"),
        "peer material must not land on the PVC that outlives the pod, got {mount_path}"
    );
}

/// AC1, the env half: the four variables `PeerTlsConfig::from_env` reads, each
/// pointing into the mount the same render produced. `LUMEN_PEER_MTLS=on` is
/// what makes the listener *require* a client certificate rather than merely
/// offer TLS, so it travels with the paths and never alone.
#[test]
fn peer_tls_secret_sets_the_four_peer_mtls_env_vars() {
    let objs = render(&lumen("search", replicated_spec(Some("search-peer-tls"))));
    let pod = pod_spec(&objs);
    let container = &pod["containers"][0];
    let mount_path = container["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "lumen-peer-tls")
        .unwrap()["mountPath"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        env_value(container, "LUMEN_PEER_MTLS").as_deref(),
        Some("on")
    );
    for (var, key) in [
        ("LUMEN_PEER_TLS_CERT", "tls.crt"),
        ("LUMEN_PEER_TLS_KEY", "tls.key"),
        ("LUMEN_PEER_TLS_CA", "ca.crt"),
    ] {
        assert_eq!(
            env_value(container, var).as_deref(),
            Some(format!("{mount_path}/{key}").as_str()),
            "{var} must point at the projected {key}, got env {:?}",
            env_names(container)
        );
    }
}

/// The converse: a single-replica instance runs no consensus link, so it gets
/// neither the volume nor the env. Without this, "the field is optional" and
/// "the field is ignored" would look the same.
#[test]
fn an_instance_without_peer_tls_secret_renders_no_peer_volume_or_env() {
    let objs = render(&lumen("search", dev_spec()));
    let pod = pod_spec(&objs);
    let container = &pod["containers"][0];

    assert!(
        pod["volumes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["name"] != "lumen-peer-tls"),
        "unexpected peer TLS volume: {pod:#}"
    );
    for var in [
        "LUMEN_PEER_MTLS",
        "LUMEN_PEER_TLS_CERT",
        "LUMEN_PEER_TLS_KEY",
        "LUMEN_PEER_TLS_CA",
    ] {
        assert!(
            env_value(container, var).is_none(),
            "unexpected {var} on a single-replica instance"
        );
    }
}

/// `ReadyFacts` reporting `count` ready pods for `name`'s StatefulSet.
fn ready_facts(name: &str, count: i64) -> ReadyFacts {
    let mut ready = std::collections::HashMap::new();
    ready.insert(name.to_string(), count);
    ReadyFacts { ready }
}

fn condition<'a>(facts: &'a [ConditionFact], type_: &str) -> &'a ConditionFact {
    facts
        .iter()
        .find(|c| c.type_ == type_)
        .unwrap_or_else(|| panic!("expected a `{type_}` condition, got: {facts:?}"))
}

/// AC2: every replica is up, so without the peer-identity verdict this CR would
/// report `Ready=True` — a replicated group advertising itself as healthy while
/// having no authenticated way to replicate. An unset `spec.peerTlsSecret` on a
/// replicated CR reports `Ready=False/PeerIdentityNotConfigured` and
/// `PeerIdentityReady=False/PeerTlsSecretNotNamed`, naming the field and the
/// required keys.
#[test]
fn a_replicated_cr_with_no_peer_secret_named_is_not_ready_and_says_which_keys_it_needs() {
    let l = lumen("search", replicated_spec(None));
    let expected_msg = format!(
        "replicasPerShard={} requires spec.peerTlsSecret naming a Secret with {}; \
         replicated Raft traffic has no plaintext fallback",
        l.spec.replicas_per_shard,
        lumen::operator::render::PEER_TLS_KEYS.join(", ")
    );
    let context = serde_json::json!({
        lumen::operator::reconcile::PEER_IDENTITY_CONTEXT_KEY: expected_msg,
    });

    // 6 shards x 3 replicas: every serving pod is up.
    let facts = l.conditions(&ready_facts("search", 18), &context);

    let ready = condition(&facts, "Ready");
    assert_eq!(ready.status, ConditionStatus::False, "got: {facts:?}");
    assert_eq!(ready.reason, "PeerIdentityNotConfigured");
    assert!(
        ready.message.contains("spec.peerTlsSecret"),
        "the Ready message must name the spec field, got: {facts:?}"
    );
    for key in lumen::operator::render::PEER_TLS_KEYS {
        assert!(
            ready.message.contains(key),
            "the Ready message must name required key {key}, got: {facts:?}"
        );
    }

    let peer = condition(&facts, "PeerIdentityReady");
    assert_eq!(peer.status, ConditionStatus::False);
    assert_eq!(peer.reason, "PeerTlsSecretNotNamed");
    assert!(
        peer.message.contains("spec.peerTlsSecret"),
        "the PeerIdentityReady message must name the spec field, got: {facts:?}"
    );
    for key in lumen::operator::render::PEER_TLS_KEYS {
        assert!(
            peer.message.contains(key),
            "the PeerIdentityReady message must name required key {key}, got: {facts:?}"
        );
    }
}

/// The satisfied case, so the condition above is a verdict rather than a
/// constant.
#[test]
fn a_replicated_cr_with_peer_material_leaves_readiness_to_the_workload() {
    let l = lumen("search", replicated_spec(Some("search-peer-tls")));

    let facts = l.conditions(&ready_facts("search", 18), &serde_json::json!({}));

    let peer = condition(&facts, "PeerIdentityReady");
    assert_eq!(peer.status, ConditionStatus::True);
    assert_eq!(peer.reason, "PeerTlsSecretProjected");
    assert_eq!(
        condition(&facts, "Ready").status,
        ConditionStatus::True,
        "got: {facts:?}"
    );
}

/// A single-replica instance reports `True` with a reason that says why —
/// there is no peer to authenticate — rather than one implying material was
/// found.
#[test]
fn a_single_replica_cr_owes_no_peer_identity() {
    let l = lumen("search", dev_spec());

    let facts = l.conditions(&ready_facts("search", 1), &serde_json::json!({}));

    let peer = condition(&facts, "PeerIdentityReady");
    assert_eq!(peer.status, ConditionStatus::True);
    assert_eq!(peer.reason, "NoReplicatedPeers");
}

/// AC6 / R7: the shipped profiles. A replicated profile that stayed silent
/// about `peerTlsSecret` would render a CR whose pods refuse to start, which is
/// a rendering bug, not an operator mistake. `dev` is single-replica and
/// deliberately excluded.
#[test]
fn replicated_instance_profiles_state_their_peer_tls_secret() {
    for profile in ["staging", "prod", "template"] {
        let rendered = run_lumen(&["k8s", "instance", "render", "--profile", profile]);
        assert!(
            rendered
                .lines()
                .any(|line| line.trim_start().starts_with("peerTlsSecret:")),
            "profile `{profile}` renders no `peerTlsSecret:` line:\n{rendered}"
        );
    }
    let dev = run_lumen(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(
        !dev.contains("peerTlsSecret"),
        "the single-replica dev profile runs no consensus link:\n{dev}"
    );
}

/// #3113 AC6: the same discipline one port over, for the opposite failure. An
/// unstated `peerTlsSecret` fails closed — the pods refuse to start and name
/// what is missing. An unstated `servingTlsSecret` fails *open*: the client
/// port stays h2c and the profile serves KSA-bearing requests in cleartext
/// while every readiness probe passes. `dev` is local-only and stays h2c on
/// purpose.
#[test]
fn production_profiles_state_their_serving_tls_secret() {
    for profile in ["staging", "prod", "template"] {
        let rendered = run_lumen(&["k8s", "instance", "render", "--profile", profile]);
        assert!(
            rendered
                .lines()
                .any(|line| line.trim_start().starts_with("servingTlsSecret:")),
            "profile `{profile}` renders no `servingTlsSecret:` line, so it serves \
             cleartext without saying so:\n{rendered}"
        );
    }
    let dev = run_lumen(&["k8s", "instance", "render", "--profile", "dev"]);
    assert!(
        !dev.contains("servingTlsSecret"),
        "the dev profile is local-only and runs h2c:\n{dev}"
    );

    // A fleet's `defaults` is a whole LumenSpec, and it is what every tenant
    // namespace inherits — silence there is cleartext multiplied by the number
    // of app teams.
    for profile in ["prod", "template"] {
        let fleet = run_lumen(&["k8s", "fleet", "render", "--profile", profile]);
        assert!(
            fleet
                .lines()
                .any(|line| line.trim_start().starts_with("servingTlsSecret:")),
            "fleet profile `{profile}` hands every tenant a cleartext default:\n{fleet}"
        );
    }
}

// ---- #3113: serving TLS on the private client Service ---------------------
//
// The client port carries the same request bodies the peer port carries index
// mutations for, and until now it carried them in the clear. These assert the
// four things the operator owes that port: the leaf reaches the pods, the
// process is told to use it, the kubelet speaks the protocol the port now
// speaks, and none of it touches the peer identity next door.

/// An instance serving TLS. Built from `prod_spec` because AC1 is a statement
/// about the production shape, not about a fixture.
fn serving_tls_spec(secret: Option<&str>) -> LumenSpec {
    LumenSpec {
        serving_tls_secret: secret.map(str::to_string),
        ..prod_spec()
    }
}

/// R2/AC2, the volume half. Same `items` discipline as the peer Secret, and a
/// separate mount path: two listeners reading one directory is how a
/// misconfiguration ends up serving the peer identity to clients.
#[test]
fn serving_tls_secret_projects_exactly_its_three_keys_read_only() {
    let objs = render(&lumen(
        "search",
        serving_tls_spec(Some("search-serving-tls")),
    ));
    let pod = pod_spec(&objs);

    let volume = pod["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["secret"]["secretName"] == "search-serving-tls")
        .unwrap_or_else(|| panic!("no serving TLS volume in {pod:#}"));
    let projected: Vec<&str> = volume["secret"]["items"]
        .as_array()
        .unwrap_or_else(|| panic!("serving TLS volume must project named items, got {volume:#}"))
        .iter()
        .map(|item| item["key"].as_str().unwrap())
        .collect();
    assert_eq!(
        projected,
        lumen::operator::render::SERVING_TLS_KEYS.to_vec(),
        "the projected keys must be exactly what the serving listener loads"
    );

    let mount = pod["containers"][0]["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == volume["name"])
        .unwrap_or_else(|| panic!("the serving TLS volume is never mounted: {pod:#}"));
    assert_eq!(mount["readOnly"], true);
    let mount_path = mount["mountPath"].as_str().unwrap();
    assert!(
        !mount_path.starts_with("/var/lib/lumen"),
        "serving material must not land on the PVC that outlives the pod, got {mount_path}"
    );
    assert_ne!(
        mount_path, "/var/run/secrets/lumen-peer",
        "the serving leaf and the peer leaf must not share a directory"
    );
}

/// R1/AC2, the env half. `LUMEN_TLS=on` is what turns the port from h2c into a
/// listener that refuses without material; the paths alone would leave it
/// cleartext, so all four travel together.
#[test]
fn serving_tls_secret_sets_the_four_serving_tls_env_vars() {
    let objs = render(&lumen(
        "search",
        serving_tls_spec(Some("search-serving-tls")),
    ));
    let pod = pod_spec(&objs);
    let container = &pod["containers"][0];
    let mount_path = container["volumeMounts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "lumen-serving-tls")
        .unwrap()["mountPath"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(env_value(container, "LUMEN_TLS").as_deref(), Some("on"));
    for (var, key) in [
        ("LUMEN_TLS_CERT", "tls.crt"),
        ("LUMEN_TLS_KEY", "tls.key"),
        ("LUMEN_TLS_CA", "ca.crt"),
    ] {
        assert_eq!(
            env_value(container, var).as_deref(),
            Some(format!("{mount_path}/{key}").as_str()),
            "{var} must point at the projected {key}, got env {:?}",
            env_names(container)
        );
    }
}

/// R2/AC2: probes follow the port. A kubelet still speaking cleartext to a TLS
/// listener reads every failed handshake as an unhealthy pod and restarts a
/// container that was serving correctly — a readiness gate that fires *because*
/// the certificate arrived.
#[test]
fn every_probe_speaks_the_scheme_the_client_port_speaks() {
    for (spec, scheme) in [
        (serving_tls_spec(Some("search-serving-tls")), "HTTPS"),
        (serving_tls_spec(None), "HTTP"),
    ] {
        let objs = render(&lumen("search", spec));
        let container = &pod_spec(&objs)["containers"][0];
        for probe in ["readinessProbe", "livenessProbe", "startupProbe"] {
            assert_eq!(
                container[probe]["httpGet"]["scheme"], scheme,
                "{probe} must speak {scheme}: {:#}",
                container[probe]
            );
            assert_eq!(
                container[probe]["httpGet"]["port"], "http",
                "{probe} must stay on the client port"
            );
        }
    }
}

/// The converse, and R3's separation read from the render: an instance with a
/// peer identity and no serving certificate gets the peer volume and nothing
/// else. Without this, "the field is optional" and "the field is ignored" look
/// identical.
#[test]
fn a_peer_identity_alone_renders_no_serving_tls_volume_or_env() {
    let objs = render(&lumen("search", replicated_spec(Some("search-peer-tls"))));
    let pod = pod_spec(&objs);
    let container = &pod["containers"][0];

    assert!(
        pod["volumes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "lumen-peer-tls"),
        "the peer volume must still be there: {pod:#}"
    );
    assert!(
        pod["volumes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["name"] != "lumen-serving-tls"),
        "unexpected serving TLS volume: {pod:#}"
    );
    for var in [
        "LUMEN_TLS",
        "LUMEN_TLS_CERT",
        "LUMEN_TLS_KEY",
        "LUMEN_TLS_CA",
    ] {
        assert!(
            env_value(container, var).is_none(),
            "unexpected {var} without a serving certificate"
        );
    }
}

/// R3: both listeners armed, and armed from different Secrets. The two
/// identities answer different questions — "I am the Service you dialed" and
/// "I am a member of this Raft group" — and one Secret answering both would let
/// either listener's material authenticate on the other's port.
#[test]
fn serving_and_peer_identities_never_share_material() {
    let spec = LumenSpec {
        serving_tls_secret: Some("search-serving-tls".into()),
        ..replicated_spec(Some("search-peer-tls"))
    };
    let objs = render(&lumen("search", spec));
    let pod = pod_spec(&objs);
    let container = &pod["containers"][0];

    let serving = env_value(container, "LUMEN_TLS_CERT").expect("serving cert path");
    let peer = env_value(container, "LUMEN_PEER_TLS_CERT").expect("peer cert path");
    assert_ne!(
        serving, peer,
        "the two listeners must not load the same certificate"
    );
    assert_eq!(
        env_value(container, "LUMEN_PEER_MTLS").as_deref(),
        Some("on")
    );
    let secrets: Vec<&str> = pod["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v["secret"]["secretName"].as_str())
        .collect();
    assert!(
        secrets.contains(&"search-serving-tls") && secrets.contains(&"search-peer-tls"),
        "both Secrets must be projected, got {secrets:?}"
    );
}

/// AC1: the client Service stays a private ClusterIP. Serving TLS is what makes
/// a public address *look* defensible, so this is the moment the render most
/// needs to say it does not want one.
#[test]
fn serving_tls_never_renders_a_public_address() {
    let objs = render(&lumen(
        "search",
        serving_tls_spec(Some("search-serving-tls")),
    ));

    let service = find(&objs, "Service", "search");
    let service_type = service["spec"]["type"].as_str().unwrap_or("ClusterIP");
    assert_eq!(
        service_type, "ClusterIP",
        "the client Service must stay private: {service:#}"
    );
    for kind in ["Ingress", "Gateway", "HTTPRoute"] {
        assert!(
            !objs.iter().any(|o| o["kind"] == kind),
            "serving TLS terminates in the pod, not in a {kind}"
        );
    }
}

#[test]
fn historical_status_unknown_trust_bundle_field_is_ignored() {
    let historical = serde_json::json!({
        "phase": "Ready",
        "observedGeneration": 7,
        "clientTrustBundle": {"configMap": "search-client-ca", "key": "ca.crt"},
    });
    let status: LumenStatus = serde_json::from_value(historical)
        .expect("historical status with removed field remains readable");
    let serialized = serde_json::to_value(status).expect("status serializes");
    assert_eq!(serialized["phase"], "Ready");
    assert!(serialized.get("clientTrustBundle").is_none());
}

#[test]
fn render_has_no_trust_anchor_publisher_or_writer_resources() {
    let objs = render(&lumen(
        "search",
        serving_tls_spec(Some("search-serving-tls")),
    ));
    assert!(!objs.iter().any(|object| {
        let kind = object["kind"].as_str().unwrap_or_default();
        let name = object["metadata"]["name"].as_str().unwrap_or_default();
        (kind == "ConfigMap" && name == "search-client-ca")
            || ((kind == "Role" || kind == "RoleBinding") && name.contains("client-ca"))
    }));
    let serialized = serde_json::to_string(&objs).expect("rendered objects serialize");
    assert!(!serialized.contains("clientTrustBundle"));
    assert!(!serialized.contains("LUMEN_CLIENT_TRUST"));
}

/// The instance renderer lives in the binary, not the library — the same reason
/// `cli_convention.rs` shells out for it.
fn run_lumen(args: &[&str]) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn lumen {args:?}: {e}"));
    assert!(
        output.status.success(),
        "lumen {args:?} failed ({:?}):\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("lumen renders UTF-8")
}

fn run_lumen_failure(args: &[&str]) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("spawn lumen {args:?}: {e}"));
    assert!(
        !output.status.success(),
        "lumen {args:?} unexpectedly succeeded"
    );
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn run_lumen_with_env(args: &[&str], envs: &[(&str, &str)]) -> String {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lumen"));
    command.args(args);
    for (name, value) in envs {
        command.env(name, value);
    }
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("spawn lumen {args:?}: {e}"));
    assert!(
        output.status.success(),
        "lumen {args:?} failed ({:?}):\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("lumen renders UTF-8")
}

#[test]
fn checked_in_and_rendered_operator_manifests_have_no_retired_issuer_surface() {
    let manifest = include_str!("../k8s/operator/deployment.yaml");
    let val: serde_yaml::Value =
        serde_yaml::from_str(manifest).expect("parse checked-in deployment.yaml");
    let spec = &val["spec"]["template"]["spec"];
    assert_eq!(spec["serviceAccountName"].as_str(), Some("lumen-operator"));
    assert!(
        spec.get("nodeSelector").is_none()
            || spec["nodeSelector"]
                .get("iam.gke.io/gke-metadata-server-enabled")
                .is_none(),
        "checked-in deployment manifest must omit GKE metadata server node selector"
    );

    let envs = spec["containers"][0]["env"]
        .as_sequence()
        .expect("containers[0].env sequence");
    let env_names: Vec<&str> = envs.iter().filter_map(|e| e["name"].as_str()).collect();
    for retired in [
        "LUMEN_ISSUER",
        "LUMEN_TRUST_DOMAIN",
        "LUMEN_CA_POOL",
        "LUMEN_WORKLOAD_IDENTITY_AUDIENCE",
        "LUMEN_PROJECTED_TOKEN_PATH",
    ] {
        assert!(
            !env_names.contains(&retired),
            "checked-in deployment still carries retired env {retired}"
        );
    }

    let rendered = run_lumen(&["k8s", "operator", "render", "--namespace", "custom-system"]);
    for retired in [
        "LUMEN_ISSUER",
        "LUMEN_TRUST_DOMAIN",
        "LUMEN_CA_POOL",
        "LUMEN_WORKLOAD_IDENTITY_AUDIENCE",
        "LUMEN_PROJECTED_TOKEN_PATH",
        "iam.gke.io/gke-metadata-server-enabled",
        "iam.gke.io/gcp-service-account",
        "gcp-ksa-token",
        "issuer:",
        "caPool:",
    ] {
        assert!(
            !rendered.contains(retired),
            "rendered operator manifest still carries retired surface {retired}"
        );
    }
}

#[test]
fn retired_issuer_flags_are_rejected_for_operator_run_and_render() {
    let retired_flags: [(&[&str], &str); 3] = [
        (&["--issuer", "ephemeral"], "--issuer"),
        (
            &["--trust-domain", "lumen-dev.svc.id.goog"],
            "--trust-domain",
        ),
        (
            &["--ca-pool", "projects/p/locations/l/caPools/n"],
            "--ca-pool",
        ),
    ];

    for verb in ["run", "render"] {
        for (flags, flag) in retired_flags {
            let mut args = vec!["k8s", "operator", verb];
            args.extend_from_slice(flags);
            let stderr = run_lumen_failure(&args);
            assert!(
                stderr.contains(flag),
                "{verb} must identify retired flag {flag}; stderr:\n{stderr}"
            );
        }
    }
}

#[test]
fn retired_issuer_environment_does_not_reactivate_operator_render() {
    let baseline = run_lumen(&["k8s", "operator", "render", "--namespace", "custom-system"]);
    let with_retired = run_lumen_with_env(
        &["k8s", "operator", "render", "--namespace", "custom-system"],
        &[
            ("LUMEN_ISSUER", "cas"),
            ("LUMEN_TRUST_DOMAIN", "lumen-prod.svc.id.goog"),
            (
                "LUMEN_CA_POOL",
                "projects/p/locations/us-central1/caPools/my-pool",
            ),
        ],
    );

    assert_eq!(
        with_retired, baseline,
        "retired issuer environment must not alter ordinary operator output"
    );
    for retired in [
        "LUMEN_ISSUER",
        "LUMEN_TRUST_DOMAIN",
        "LUMEN_CA_POOL",
        "issuer:",
        "caPool:",
    ] {
        assert!(
            !with_retired.contains(retired),
            "retired environment leaked into operator output: {retired}"
        );
    }
}

#[test]
fn instance_profiles_document_external_serving_and_peer_secrets_without_issuer_fields() {
    for profile in ["dev", "staging", "prod", "template"] {
        let yaml = run_lumen(&["k8s", "instance", "render", "--profile", profile]);
        assert!(
            yaml.contains("TLS Secrets are provisioned by the deployment administrator or an external platform"),
            "{profile} profile must identify external TLS Secret provisioning"
        );
        if profile == "dev" {
            assert!(
                !yaml.contains("servingTlsSecret") && !yaml.contains("peerTlsSecret"),
                "dev profile may omit TLS Secret names for local development"
            );
        } else {
            assert!(
                yaml.contains("servingTlsSecret"),
                "{profile} profile must name its external serving Secret"
            );
            assert!(
                yaml.contains("peerTlsSecret"),
                "{profile} profile must name its external peer Secret"
            );
        }
        assert!(!yaml.contains("issuer:"), "{profile} carries issuer field");
        assert!(!yaml.contains("caPool:"), "{profile} carries CA pool field");
        assert!(
            !yaml.contains("LUMEN_ISSUER"),
            "{profile} carries issuer env"
        );
        assert!(
            !yaml.contains("LUMEN_TRUST_DOMAIN"),
            "{profile} carries trust-domain env"
        );
        assert!(
            !yaml.contains("LUMEN_CA_POOL"),
            "{profile} carries CA pool env"
        );
    }
}
// CODEGEN-END

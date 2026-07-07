// HANDWRITE-BEGIN gap="missing-generator:unit-test:38a65c0d" tracker="pending-tracker" reason="Feature-gated (#![cfg(feature = 'operator')]) render-shape tests: CRD flattens ClusterSpec + relay knobs (no nested cluster wrapper); render() emits the downward-API StatefulSet with the exact env/probe contract serve reads (POD_NAME/SHARD_COUNT=1/REPLICAS_PER_SHARD/VOTER_COUNT/RELAY_PEER_SERVICE + RELAY_BIND/RELAY_DATA_DIR/RELAY_GRACE_SECS, /readyz + /healthz probes, PVC storage) plus ServiceAccount/Services/PDB; auth Secret wiring is opt-in (env + volume only when auth: required + tokensSecret); status_patch phases Pending/Reconciling/Ready; rustls provider install is idempotent."
//! Operator-adoption render-shape tests (WI #1208). Compiled only with
//! `--features operator`.
//!
//! - R2: `RelaySpec` flattens `operator::ClusterSpec` into the CRD schema.
//! - R2: `render` emits the downward-API StatefulSet with the exact env/probe
//!   contract relay's serve reads, plus ServiceAccount/Services/PDB.
//! - R2: the token-registry Secret wiring is opt-in (lumen's pattern).
//! - R2: `status_patch` reports Pending/Reconciling/Ready.
//! - R3: the generated CRD is Kubernetes-OpenAPI compatible (no
//!   `uint32`/`uint64`; normalized counts keep a `minimum` floor).
//! - R1: the process-level rustls crypto provider install is idempotent.
#![cfg(feature = "operator")]

use std::collections::HashMap;

use operator::{ClusterSpec, ManagedService, ReadyFacts};
use relay::operator::render::render;
use relay::operator::{crd_yaml, Relay, RelaySpec};
use relay::tls::install_default_crypto_provider;
use serde_json::Value;

fn spec(replicas: u32) -> RelaySpec {
    RelaySpec {
        cluster: ClusterSpec {
            image: "relay:test".into(),
            image_pull_policy: None,
            shard_count: 1,
            replicas_per_shard: replicas,
            voter_count: replicas,
            resources: Default::default(),
        },
        storage: "10Gi".into(),
        storage_class: None,
        grace_secs: 10,
        log_level: None,
        auth: "off".into(),
        tokens_secret: None,
    }
}

fn of_kind<'a>(objs: &'a [Value], kind: &str) -> &'a Value {
    objs.iter()
        .find(|o| o["kind"] == kind)
        .unwrap_or_else(|| panic!("render output has no {kind}"))
}

fn env_of(sts: &Value) -> Vec<(&str, &Value)> {
    sts["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .expect("container env array")
        .iter()
        .map(|e| (e["name"].as_str().unwrap(), e))
        .collect()
}

/// R2 — the flattened `ClusterSpec` fields sit directly under the CRD's `spec`
/// schema (no `cluster` wrapper), so a `Relay` CR declares them inline.
#[test]
fn crd_flattens_cluster_spec() {
    let yaml = crd_yaml();
    let doc: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("CRD parses as YAML");
    let props = &doc["spec"]["versions"][0]["schema"]["openAPIV3Schema"]["properties"]["spec"]
        ["properties"];
    for field in [
        "image",
        "imagePullPolicy",
        "shardCount",
        "replicasPerShard",
        "voterCount",
        "resources",
    ] {
        assert!(
            props.get(field).is_some(),
            "CRD spec schema must carry flattened field `{field}`"
        );
    }
    // Flatten merges properties → there is no nested `cluster` wrapper.
    assert!(
        props.get("cluster").is_none(),
        "ClusterSpec must be flattened, not nested under `cluster`"
    );
    // relay's own knobs are present too.
    for field in [
        "storage",
        "storageClass",
        "graceSecs",
        "logLevel",
        "auth",
        "tokensSecret",
    ] {
        assert!(props.get(field).is_some(), "missing relay knob `{field}`");
    }
    // R3 — Kubernetes structural-schema compatible.
    assert!(
        !yaml.contains("uint32"),
        "CRD must not carry format: uint32"
    );
    assert!(
        !yaml.contains("uint64"),
        "CRD must not carry format: uint64"
    );
    assert!(
        yaml.contains("minimum"),
        "normalized uints keep a minimum floor"
    );
}

/// R2 / AC3 — the rendered StatefulSet carries exactly the downward-API env
/// relay's serve reads, the right replica count (single group), relay's
/// runtime env + disk tier, the probe contract, and the sibling child objects.
#[test]
fn render_emits_downward_api_statefulset() {
    let relay = Relay::new("relay", spec(3));
    let objs = render(&relay);

    let sts = of_kind(&objs, "StatefulSet");
    // Single raft group: shardCount pinned to 1 → replicas == replicasPerShard.
    assert_eq!(sts["spec"]["replicas"], 3, "replicas = replicasPerShard");
    assert_eq!(sts["spec"]["serviceName"], "relay-headless");
    assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");

    let env = env_of(sts);
    let keys: Vec<&str> = env.iter().map(|(k, _)| *k).collect();
    // The exact contract serve reads: raft_host::cluster (quartet) +
    // --peer-service (RELAY_PEER_SERVICE) + bind/data-dir/grace.
    for k in [
        "POD_NAME",
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "RELAY_PEER_SERVICE",
        "RELAY_BIND",
        "RELAY_DATA_DIR",
        "RELAY_GRACE_SECS",
    ] {
        assert!(keys.contains(&k), "missing env {k}");
    }
    let get = |k: &str| env.iter().find(|(n, _)| *n == k).unwrap().1;
    assert_eq!(
        get("POD_NAME")["valueFrom"]["fieldRef"]["fieldPath"],
        "metadata.name"
    );
    assert_eq!(get("SHARD_COUNT")["value"], "1", "relay is a single group");
    assert_eq!(get("REPLICAS_PER_SHARD")["value"], "3");
    assert_eq!(get("VOTER_COUNT")["value"], "3");
    assert_eq!(get("RELAY_PEER_SERVICE")["value"], "relay-headless");
    assert_eq!(get("RELAY_BIND")["value"], "0.0.0.0:7000");
    assert_eq!(get("RELAY_DATA_DIR")["value"], "/data");

    // Probe contract on the serve port: /readyz readiness, /healthz liveness
    // + startup — what service-http's standard probe routes answer.
    let container = &sts["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["startupProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["ports"][0]["containerPort"], 7000);
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);

    // Durable disk tier: the /data PVC carries the CR's storage size.
    assert_eq!(
        sts["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "10Gi"
    );
    assert_eq!(container["volumeMounts"][0]["mountPath"], "/data");

    // The rest of the child set is present.
    assert_eq!(
        of_kind(&objs, "ServiceAccount")["metadata"]["name"],
        "relay"
    );
    let headless = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["clusterIP"] == "None")
        .expect("headless service");
    assert_eq!(headless["metadata"]["name"], "relay-headless");
    let client = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["type"] == "ClusterIP")
        .expect("client service");
    assert_eq!(client["spec"]["ports"][0]["port"], 7000);
    assert_eq!(
        of_kind(&objs, "PodDisruptionBudget")["spec"]["maxUnavailable"],
        1
    );
}

/// R2 — RELAY_AUTH / RELAY_TOKEN_REGISTRY_FILE + the Secret volume render only
/// when the CR sets `auth: required` AND names a `tokensSecret` (off by
/// default — lumen's pattern).
#[test]
fn auth_secret_wiring_is_opt_in() {
    // Default CR: no auth env, no token-registry volume.
    let plain = Relay::new("relay", spec(1));
    let objs = render(&plain);
    let sts = of_kind(&objs, "StatefulSet");
    let keys: Vec<&str> = env_of(sts).iter().map(|(k, _)| *k).collect();
    assert!(!keys.contains(&"RELAY_AUTH"), "auth env must be opt-in");
    assert!(!keys.contains(&"RELAY_TOKEN_REGISTRY_FILE"));
    let vols = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap();
    assert!(
        vols.iter().all(|v| v["name"] != "relay-token-registry"),
        "no token-registry volume without tokensSecret"
    );

    // auth: required alone (no Secret named) still renders nothing — a
    // half-configured CR must not produce a pod that crash-loops on a missing
    // registry file.
    let mut half = spec(1);
    half.auth = "required".into();
    let objs = render(&Relay::new("relay", half));
    let keys: Vec<String> = env_of(of_kind(&objs, "StatefulSet"))
        .iter()
        .map(|(k, _)| k.to_string())
        .collect();
    assert!(!keys.contains(&"RELAY_AUTH".to_string()));

    // auth: required + tokensSecret: env + read-only Secret mount.
    let mut secured = spec(3);
    secured.auth = "required".into();
    secured.tokens_secret = Some("relay-token-registry".into());
    let objs = render(&Relay::new("relay", secured));
    let sts = of_kind(&objs, "StatefulSet");
    let env = env_of(sts);
    let get = |k: &str| env.iter().find(|(n, _)| *n == k).unwrap().1;
    assert_eq!(get("RELAY_AUTH")["value"], "required");
    assert_eq!(
        get("RELAY_TOKEN_REGISTRY_FILE")["value"],
        "/var/run/secrets/relay/token-registry.json"
    );
    let vols = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap();
    let vol = vols
        .iter()
        .find(|v| v["name"] == "relay-token-registry")
        .expect("token-registry volume");
    assert_eq!(vol["secret"]["secretName"], "relay-token-registry");
    let mounts = sts["spec"]["template"]["spec"]["containers"][0]["volumeMounts"]
        .as_array()
        .unwrap();
    let mount = mounts
        .iter()
        .find(|m| m["name"] == "relay-token-registry")
        .expect("token-registry mount");
    assert_eq!(mount["mountPath"], "/var/run/secrets/relay");
    assert_eq!(mount["readOnly"], true);
}

/// R2 — readiness target + status phases (Pending / Reconciling / Ready).
#[test]
fn status_patch_reports_phases() {
    let relay = Relay::new("relay", spec(3));

    let targets = relay.readiness_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].kind, "StatefulSet");
    assert_eq!(targets[0].name, "relay");

    let mut all_ready = HashMap::new();
    all_ready.insert("relay".to_string(), 3i64);
    let status = relay.status_patch(&ReadyFacts { ready: all_ready });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["desiredReplicas"], 3);
    assert_eq!(status["status"]["readyReplicas"], 3);

    let mut partial = HashMap::new();
    partial.insert("relay".to_string(), 1i64);
    let status = relay.status_patch(&ReadyFacts { ready: partial });
    assert_eq!(status["status"]["phase"], "Reconciling");

    let status = relay.status_patch(&ReadyFacts {
        ready: HashMap::new(),
    });
    assert_eq!(status["status"]["phase"], "Pending");
}

/// R1 — installing the process-level rustls crypto provider is idempotent and
/// safe to call repeatedly (as `main` does before command parsing).
#[test]
fn rustls_provider_install_is_idempotent() {
    install_default_crypto_provider();
    install_default_crypto_provider();
}
// HANDWRITE-END

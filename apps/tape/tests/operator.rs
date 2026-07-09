// HANDWRITE-BEGIN gap="missing-generator:unit-test:c19f401a" tracker="pending-tracker" reason="Feature-gated (operator) render-shape tests: CRD flattens ClusterSpec + tape knobs; render() emits the downward-API StatefulSet with the exact env/probe contract serve reads, plus ServiceAccount/Services/PDB; auth Secret wiring is opt-in; status_patch phases Pending/Reconciling/Ready."
//! Operator-adoption render-shape tests (#1328). Compiled only with
//! `--features operator`.
//!
//! - R4: `TapeSpec` flattens `operator::ClusterSpec` into the CRD schema.
//! - R5: `render` emits the downward-API StatefulSet with the exact env/probe
//!   contract tape's serve reads, plus ServiceAccount/Services/PDB.
//! - R6: the token-registry Secret wiring is opt-in (relay/lumen's pattern).
//! - R7: `status_patch` reports Pending/Reconciling/Ready.
#![cfg(feature = "operator")]

use std::collections::HashMap;

use operator::{ClusterSpec, ManagedService, ReadyFacts};
use serde_json::Value;
use tape::operator::render::render;
use tape::operator::{crd_yaml, Tape, TapeSpec};

fn spec(replicas: u32) -> TapeSpec {
    TapeSpec {
        cluster: ClusterSpec {
            image: "tape:test".into(),
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

/// R4 — the flattened `ClusterSpec` fields sit directly under the CRD's
/// `spec` schema (no `cluster` wrapper), so a `Tape` CR declares them inline.
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
    // tape's own knobs are present too.
    for field in [
        "storage",
        "storageClass",
        "graceSecs",
        "logLevel",
        "auth",
        "tokensSecret",
    ] {
        assert!(props.get(field).is_some(), "missing tape knob `{field}`");
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

/// R5 — the rendered StatefulSet carries exactly the downward-API env tape's
/// serve reads, the right replica count (single group), tape's runtime env +
/// disk tier, the probe contract, and the sibling child objects.
#[test]
fn render_emits_expected_child_objects() {
    let tape = Tape::new("tape", spec(3));
    let objs = render(&tape);

    let sts = of_kind(&objs, "StatefulSet");
    // Single raft group: shardCount pinned to 1 → replicas == replicasPerShard.
    assert_eq!(sts["spec"]["replicas"], 3, "replicas = replicasPerShard");
    assert_eq!(sts["spec"]["serviceName"], "tape-headless");
    assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");

    let env = env_of(sts);
    let keys: Vec<&str> = env.iter().map(|(k, _)| *k).collect();
    // The exact contract serve reads: raft_host::cluster (quartet) +
    // --peer-service (TAPE_PEER_SERVICE) + bind/data-dir/grace.
    for k in [
        "POD_NAME",
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "TAPE_PEER_SERVICE",
        "TAPE_BIND",
        "TAPE_DATA_DIR",
        "TAPE_GRACE_SECS",
    ] {
        assert!(keys.contains(&k), "missing env {k}");
    }
    let get = |k: &str| env.iter().find(|(n, _)| *n == k).unwrap().1;
    assert_eq!(
        get("POD_NAME")["valueFrom"]["fieldRef"]["fieldPath"],
        "metadata.name"
    );
    assert_eq!(get("SHARD_COUNT")["value"], "1", "tape is a single group");
    assert_eq!(get("REPLICAS_PER_SHARD")["value"], "3");
    assert_eq!(get("VOTER_COUNT")["value"], "3");
    assert_eq!(get("TAPE_PEER_SERVICE")["value"], "tape-headless");
    assert_eq!(get("TAPE_BIND")["value"], "0.0.0.0:7137");
    assert_eq!(get("TAPE_DATA_DIR")["value"], "/data");

    // Probe contract on the serve port: /readyz readiness, /healthz liveness
    // + startup — what service-http's standard probe routes answer.
    let container = &sts["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["startupProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["ports"][0]["containerPort"], 7137);
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);

    // Durable disk tier: the /data PVC carries the CR's storage size.
    assert_eq!(
        sts["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "10Gi"
    );
    assert_eq!(container["volumeMounts"][0]["mountPath"], "/data");

    // The rest of the child set is present.
    assert_eq!(of_kind(&objs, "ServiceAccount")["metadata"]["name"], "tape");
    let headless = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["clusterIP"] == "None")
        .expect("headless service");
    assert_eq!(headless["metadata"]["name"], "tape-headless");
    let client = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["type"] == "ClusterIP")
        .expect("client service");
    assert_eq!(client["spec"]["ports"][0]["port"], 7137);
    assert_eq!(
        of_kind(&objs, "PodDisruptionBudget")["spec"]["maxUnavailable"],
        1
    );
}

/// R6 — TAPE_AUTH / TAPE_TOKEN_REGISTRY_FILE + the Secret volume render only
/// when the CR sets `auth: required` AND names a `tokensSecret` (off by
/// default — relay/lumen's pattern).
#[test]
fn token_registry_secret_wiring_is_opt_in() {
    // Default CR: no auth env, no token-registry volume.
    let plain = Tape::new("tape", spec(1));
    let objs = render(&plain);
    let sts = of_kind(&objs, "StatefulSet");
    let keys: Vec<&str> = env_of(sts).iter().map(|(k, _)| *k).collect();
    assert!(!keys.contains(&"TAPE_AUTH"), "auth env must be opt-in");
    assert!(!keys.contains(&"TAPE_TOKEN_REGISTRY_FILE"));
    let vols = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap();
    assert!(
        vols.iter().all(|v| v["name"] != "tape-token-registry"),
        "no token-registry volume without tokensSecret"
    );

    // auth: required alone (no Secret named) still renders nothing — a
    // half-configured CR must not produce a pod that crash-loops on a missing
    // registry file.
    let mut half = spec(1);
    half.auth = "required".into();
    let objs = render(&Tape::new("tape", half));
    let keys: Vec<String> = env_of(of_kind(&objs, "StatefulSet"))
        .iter()
        .map(|(k, _)| k.to_string())
        .collect();
    assert!(!keys.contains(&"TAPE_AUTH".to_string()));

    // auth: required + tokensSecret: env + read-only Secret mount.
    let mut secured = spec(3);
    secured.auth = "required".into();
    secured.tokens_secret = Some("tape-token-registry".into());
    let objs = render(&Tape::new("tape", secured));
    let sts = of_kind(&objs, "StatefulSet");
    let env = env_of(sts);
    let get = |k: &str| env.iter().find(|(n, _)| *n == k).unwrap().1;
    assert_eq!(get("TAPE_AUTH")["value"], "required");
    assert_eq!(
        get("TAPE_TOKEN_REGISTRY_FILE")["value"],
        "/var/run/secrets/tape/token-registry.json"
    );
    let vols = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap();
    let vol = vols
        .iter()
        .find(|v| v["name"] == "tape-token-registry")
        .expect("token-registry volume");
    assert_eq!(vol["secret"]["secretName"], "tape-token-registry");
    let mounts = sts["spec"]["template"]["spec"]["containers"][0]["volumeMounts"]
        .as_array()
        .unwrap();
    let mount = mounts
        .iter()
        .find(|m| m["name"] == "tape-token-registry")
        .expect("token-registry mount");
    assert_eq!(mount["mountPath"], "/var/run/secrets/tape");
    assert_eq!(mount["readOnly"], true);
}

/// R7 — readiness target + status phases (Pending / Reconciling / Ready).
#[test]
fn status_patch_reports_pending_reconciling_ready() {
    let tape = Tape::new("tape", spec(3));

    let targets = tape.readiness_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].kind, "StatefulSet");
    assert_eq!(targets[0].name, "tape");

    let mut all_ready = HashMap::new();
    all_ready.insert("tape".to_string(), 3i64);
    let status = tape.status_patch(&ReadyFacts { ready: all_ready });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["desiredReplicas"], 3);
    assert_eq!(status["status"]["readyReplicas"], 3);

    let mut partial = HashMap::new();
    partial.insert("tape".to_string(), 1i64);
    let status = tape.status_patch(&ReadyFacts { ready: partial });
    assert_eq!(status["status"]["phase"], "Reconciling");

    let status = tape.status_patch(&ReadyFacts {
        ready: HashMap::new(),
    });
    assert_eq!(status["status"]["phase"], "Pending");
}
// HANDWRITE-END

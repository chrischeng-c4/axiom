//! Operator-adoption integration tests (#775). Compiled only with
//! `--features operator`.
//!
//! - R1: `KeepSpec` flattens `operator::ClusterSpec` into the CRD schema.
//! - R2: `render` emits the sharded StatefulSet with the downward-API env
//!   raft-host consumes, correct replica math, and status phases.
//! - R4: the generated CRD is Kubernetes-OpenAPI compatible (no `uint32`/
//!   `uint64`; unsigned counts keep `minimum`).
//! - R5: the process-level rustls crypto provider install is idempotent.
//!
//! @spec projects/keep/tech-design/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md
#![cfg(feature = "operator")]

use std::collections::HashMap;

use keep::operator::render::render;
use keep::operator::{crd_yaml, Keep, KeepSpec};
use keep::tls::install_default_crypto_provider;
use operator::{ClusterSpec, ManagedService, ReadyFacts};
use serde_json::Value;

fn spec(shard_count: u32, replicas: u32) -> KeepSpec {
    KeepSpec {
        cluster: ClusterSpec {
            image: "keep:test".into(),
            image_pull_policy: None,
            shard_count,
            replicas_per_shard: replicas,
            voter_count: replicas,
            resources: Default::default(),
        },
        engine_shards: 256,
        log_level: None,
        grace_secs: 30,
        storage: "10Gi".into(),
        storage_class: None,
        backup: None,
    }
}

fn of_kind<'a>(objs: &'a [Value], kind: &str) -> &'a Value {
    objs.iter()
        .find(|o| o["kind"] == kind)
        .unwrap_or_else(|| panic!("render output has no {kind}"))
}

/// R1 — the flattened `ClusterSpec` fields sit directly under the CRD's `spec`
/// schema (no `cluster` wrapper), so a `Keep` CR declares them inline.
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
            "CRD spec schema must carry flattened field `{field}`; props = {props:?}"
        );
    }
    // Flatten merges properties → there is no nested `cluster` wrapper property.
    assert!(
        props.get("cluster").is_none(),
        "ClusterSpec must be flattened, not nested under `cluster`"
    );
    // keep's own knobs are present too.
    assert!(props.get("engineShards").is_some());
    assert!(props.get("storage").is_some());
}

/// R2 — the rendered StatefulSet carries the downward-API env raft-host reads,
/// the right replica count, keep's runtime knobs + disk tier, and drives a
/// readiness/status contract.
#[test]
fn render_emits_downward_api_statefulset() {
    let keep = Keep::new("store", spec(2, 3));
    let objs = render(&keep);

    let sts = of_kind(&objs, "StatefulSet");
    assert_eq!(
        sts["spec"]["replicas"], 6,
        "replicas = shardCount * replicasPerShard"
    );
    assert_eq!(sts["spec"]["serviceName"], "store-headless");
    assert_eq!(sts["spec"]["podManagementPolicy"], "Parallel");

    let env = sts["spec"]["template"]["spec"]["containers"][0]["env"]
        .as_array()
        .expect("container env array");
    let keys: Vec<&str> = env.iter().map(|e| e["name"].as_str().unwrap()).collect();
    for k in [
        "POD_NAME",
        "POD_NAMESPACE",
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "KEEP_HEADLESS_SERVICE",
    ] {
        assert!(keys.contains(&k), "missing downward-API env {k}");
    }
    for k in ["KEEP_HOST", "KEEP_DATA_DIR", "KEEP_SHARD_COUNT"] {
        assert!(keys.contains(&k), "missing keep runtime env {k}");
    }
    // Durable disk tier + production hardening survive the enrichment.
    assert!(sts["spec"]["volumeClaimTemplates"].is_array());
    assert_eq!(
        sts["spec"]["template"]["spec"]["containers"][0]["readinessProbe"]["httpGet"]["path"],
        "/readyz"
    );
    assert_eq!(
        sts["spec"]["template"]["spec"]["containers"][0]["securityContext"]
            ["readOnlyRootFilesystem"],
        true
    );

    // The rest of the child set is present.
    assert_eq!(
        of_kind(&objs, "ServiceAccount")["metadata"]["name"],
        "store"
    );
    assert_eq!(
        of_kind(&objs, "ConfigMap")["metadata"]["name"],
        "store-config"
    );
    assert_eq!(
        of_kind(&objs, "PodDisruptionBudget")["spec"]["maxUnavailable"],
        0
    );

    // Readiness target + status phases.
    let targets = keep.readiness_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].kind, "StatefulSet");

    let mut all_ready = HashMap::new();
    all_ready.insert("store".to_string(), 6i64);
    let status = keep.status_patch(&ReadyFacts { ready: all_ready });
    assert_eq!(status["status"]["phase"], "Ready");
    assert_eq!(status["status"]["desiredReplicas"], 6);
    assert_eq!(status["status"]["shardCount"], 2);

    let mut partial = HashMap::new();
    partial.insert("store".to_string(), 2i64);
    let status = keep.status_patch(&ReadyFacts { ready: partial });
    assert_eq!(status["status"]["phase"], "Reconciling");

    let status = keep.status_patch(&ReadyFacts {
        ready: HashMap::new(),
    });
    assert_eq!(status["status"]["phase"], "Pending");
}

/// R3 (#776) — the operator renders a backup CronJob invoking `keep backup`
/// when `spec.backup` is configured, and omits it otherwise.
#[test]
fn render_emits_backup_cronjob_when_configured() {
    use keep::operator::KeepBackupSpec;

    // No policy -> no CronJob.
    let plain = Keep::new("store", spec(1, 1));
    assert!(
        render(&plain).iter().all(|o| o["kind"] != "CronJob"),
        "no backup policy must render no CronJob"
    );

    // Policy set -> a CronJob invoking `keep backup` with the schedule + dest +
    // retention flags.
    let mut s = spec(1, 1);
    s.backup = Some(KeepBackupSpec {
        schedule: "17 3 * * *".into(),
        destination: "s3://bucket/keep".into(),
        retention_secs: Some(604_800),
    });
    let keep = Keep::new("store", s);
    let objs = render(&keep);

    let cj = of_kind(&objs, "CronJob");
    assert_eq!(cj["metadata"]["name"], "store-backup");
    assert_eq!(cj["spec"]["schedule"], "17 3 * * *");
    let container = &cj["spec"]["jobTemplate"]["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["command"][0], "keep");
    let args: Vec<&str> = container["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert_eq!(args[0], "backup");
    assert!(args.contains(&"--dest"));
    assert!(args.contains(&"s3://bucket/keep"));
    assert!(args.contains(&"--retention-secs"));
    assert!(args.contains(&"604800"));
    // The runner runs under keep's ServiceAccount.
    assert_eq!(
        cj["spec"]["jobTemplate"]["spec"]["template"]["spec"]["serviceAccountName"],
        "store"
    );
}

/// R4 — the generated CRD is Kubernetes-OpenAPI compatible.
#[test]
fn crd_is_openapi_compatible() {
    let yaml = crd_yaml();
    assert!(
        !yaml.contains("uint32"),
        "CRD must not carry format: uint32"
    );
    assert!(
        !yaml.contains("uint64"),
        "CRD must not carry format: uint64"
    );
    // The unsigned counts were normalized to integer with a minimum floor.
    assert!(
        yaml.contains("minimum"),
        "normalized uints keep a minimum floor"
    );
}

/// R5 — installing the process-level rustls crypto provider is idempotent and
/// safe to call repeatedly (as `main` does before command parsing).
#[test]
fn rustls_provider_install_is_idempotent() {
    install_default_crypto_provider();
    install_default_crypto_provider();
}

// HANDWRITE-BEGIN gap="missing-generator:unit-test:c19f401a" tracker="pending-tracker" reason="Feature-gated (operator) render-shape tests: CRD flattens ClusterSpec + tape knobs; render() emits the downward-API StatefulSet with the exact env/probe contract serve reads, plus ServiceAccount/Services/PDB; auth Secret wiring is opt-in; status_patch phases Pending/Reconciling/Ready."
//! Operator-adoption render-shape tests (#1328). Compiled only with
//! `--features operator`.
//!
//! - R4: `TapeSpec` flattens `service_k8s::ClusterSpec` into the CRD schema.
//! - R5: `render` emits the downward-API StatefulSet with the exact env/probe
//!   contract tape's serve reads, plus ServiceAccount/Services/PDB.
//! - R6: the token-registry Secret wiring is opt-in (relay/lumen's pattern).
//! - R7: `status_patch` reports Pending/Reconciling/Ready.
#![cfg(feature = "operator")]

use std::collections::HashMap;

use serde_json::Value;
use service_k8s::{ClusterSpec, ManagedService, ReadyFacts};
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
        tokens_secret_provider_class: None,
        tokens_secret_csi_driver: None,
        bootstrap_seed_uri: None,
        body_limit_bytes: None,
        topics: None,
        backup: None,
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
        "tokensSecretProviderClass",
        "tokensSecretCsiDriver",
        "bootstrapSeedUri",
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
    assert!(
        yaml.contains("default: \"off\""),
        "the auth default must remain a string when Kubernetes parses YAML 1.1"
    );
    assert_eq!(props["auth"]["default"], "off");
    assert_eq!(
        include_str!("../k8s/operator/crd.yaml"),
        yaml,
        "checked-in CRD must be regenerated from the renderer"
    );
}

// <HANDWRITE gap="missing-generator:kubernetes-peer-port-test" tracker="#1805" reason="kubernetes-peer-port-test section in operator.rs is hand-written pending codegen support">
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
    // The exact contract serve reads: raft_runtime::cluster (quartet) +
    // --peer-service (TAPE_PEER_SERVICE) + public/raft binds + data-dir/grace.
    for k in [
        "POD_NAME",
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "VOTER_COUNT",
        "TAPE_PEER_SERVICE",
        "TAPE_BIND",
        "TAPE_RAFT_PORT",
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
    assert_eq!(get("TAPE_RAFT_PORT")["value"], "7138");
    assert_eq!(get("TAPE_DATA_DIR")["value"], "/data");

    // Probe contract on the serve port: /readyz readiness, /healthz liveness
    // + startup — what service-http's standard probe routes answer.
    let container = &sts["spec"]["template"]["spec"]["containers"][0];
    let pod = &sts["spec"]["template"]["spec"];
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["startupProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["ports"][0]["containerPort"], 7137);
    assert_eq!(container["ports"][1]["name"], "raft");
    assert_eq!(container["ports"][1]["containerPort"], 7138);
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);
    assert_eq!(container["resources"]["requests"]["cpu"], "1");
    assert_eq!(container["resources"]["requests"]["memory"], "4Gi");
    assert!(container["resources"].get("limits").is_none());
    assert_eq!(sts["spec"]["revisionHistoryLimit"], 5);
    assert_eq!(sts["spec"]["updateStrategy"]["type"], "RollingUpdate");
    assert_eq!(
        sts["spec"]["template"]["metadata"]["annotations"]["prometheus.io/path"],
        "/metrics"
    );
    assert_eq!(pod["securityContext"]["runAsNonRoot"], true);
    assert_eq!(
        pod["securityContext"]["seccompProfile"]["type"],
        "RuntimeDefault"
    );
    assert_eq!(
        pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0]
            ["topologyKey"],
        "kubernetes.io/hostname"
    );

    // Durable disk tier: the /data PVC carries the CR's storage size.
    assert_eq!(
        sts["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "10Gi"
    );
    let mounts = container["volumeMounts"].as_array().unwrap();
    let data_mount = mounts
        .iter()
        .find(|mount| mount["name"] == "data")
        .expect("data PVC mount");
    assert_eq!(data_mount["mountPath"], "/data");
    assert!(pod["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|volume| volume["name"] == "tmp" && volume["emptyDir"] == serde_json::json!({})));

    // The rest of the child set is present.
    assert_eq!(of_kind(&objs, "ServiceAccount")["metadata"]["name"], "tape");
    let headless = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["clusterIP"] == "None")
        .expect("headless service");
    assert_eq!(headless["metadata"]["name"], "tape-headless");
    assert_eq!(headless["spec"]["ports"][0]["port"], 7137);
    assert_eq!(headless["spec"]["ports"][1]["name"], "raft");
    assert_eq!(headless["spec"]["ports"][1]["port"], 7138);
    let client = objs
        .iter()
        .find(|o| o["kind"] == "Service" && o["spec"]["type"] == "ClusterIP")
        .expect("client service");
    assert_eq!(client["spec"]["ports"][0]["port"], 7137);
    assert_eq!(client["spec"]["ports"].as_array().unwrap().len(), 1);
    assert_eq!(
        of_kind(&objs, "PodDisruptionBudget")["spec"]["maxUnavailable"],
        1
    );
}
// </HANDWRITE>

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

    // auth: required + CSI provider class: same process contract, but no
    // Kubernetes Secret object; the shared projection helper emits the CSI
    // volume exactly once.
    let mut csi = spec(3);
    csi.auth = "required".into();
    csi.tokens_secret_provider_class = Some("tape-registry-csi".into());
    let objs = render(&Tape::new("tape", csi));
    let sts = of_kind(&objs, "StatefulSet");
    let env = env_of(sts);
    let get = |k: &str| env.iter().find(|(n, _)| *n == k).unwrap().1;
    assert_eq!(get("TAPE_AUTH")["value"], "required");
    assert_eq!(
        get("TAPE_TOKEN_REGISTRY_FILE")["value"],
        "/var/run/secrets/tape/token-registry.json"
    );
    let vol = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "tape-token-registry")
        .expect("CSI token-registry volume");
    assert_eq!(vol["csi"]["driver"], "secrets-store.csi.k8s.io");
    assert_eq!(
        vol["csi"]["volumeAttributes"]["secretProviderClass"],
        "tape-registry-csi"
    );

    // Explicit Secret retains precedence over CSI, matching Lumen's
    // backwards-compatible deployment contract.
    let mut both = spec(3);
    both.auth = "required".into();
    both.tokens_secret = Some("tape-registry-secret".into());
    both.tokens_secret_provider_class = Some("tape-registry-csi".into());
    let objs = render(&Tape::new("tape", both));
    let sts = of_kind(&objs, "StatefulSet");
    let vol = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "tape-token-registry")
        .expect("token-registry volume");
    assert_eq!(vol["secret"]["secretName"], "tape-registry-secret");
    assert!(vol.get("csi").is_none());
}

/// GKE's managed Secrets Store add-on registers a different CSI driver name
/// than the community default (#2456/#2457); an explicit
/// `tokensSecretCsiDriver` must render that name on the pod volume.
#[test]
fn tokens_secret_csi_driver_overrides_the_default_csi_driver_name() {
    let mut csi = spec(3);
    csi.auth = "required".into();
    csi.tokens_secret_provider_class = Some("tape-registry-csi".into());
    csi.tokens_secret_csi_driver = Some("secrets-store-gke.csi.k8s.io".into());
    let objs = render(&Tape::new("tape", csi));
    let sts = of_kind(&objs, "StatefulSet");
    let vol = sts["spec"]["template"]["spec"]["volumes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "tape-token-registry")
        .expect("CSI token-registry volume");
    assert_eq!(vol["csi"]["driver"], "secrets-store-gke.csi.k8s.io");
    assert_eq!(
        vol["csi"]["volumeAttributes"]["secretProviderClass"],
        "tape-registry-csi"
    );
}

/// The optional cold-recovery seed stays absent by default and projects to the
/// exact serve environment only when the CR intentionally requests it.
#[test]
fn bootstrap_seed_uri_wiring_is_opt_in() {
    let plain = Tape::new("tape", spec(3));
    let plain_objects = render(&plain);
    let plain_env = env_of(of_kind(&plain_objects, "StatefulSet"));
    assert!(
        plain_env
            .iter()
            .all(|(name, _)| *name != "TAPE_BOOTSTRAP_SEED_URI"),
        "ordinary PVC restarts must not reapply a seed"
    );

    let mut seeded = spec(3);
    seeded.bootstrap_seed_uri = Some("s3://tape-backups/orders/snapshot-42.json".into());
    let seeded = Tape::new("tape", seeded);
    let seeded_objects = render(&seeded);
    let seeded_env = env_of(of_kind(&seeded_objects, "StatefulSet"));
    let seed = seeded_env
        .iter()
        .find(|(name, _)| *name == "TAPE_BOOTSTRAP_SEED_URI")
        .expect("bootstrap seed env when CR requests one")
        .1;
    assert_eq!(seed["value"], "s3://tape-backups/orders/snapshot-42.json");
}

/// #2574 — `spec.backup` is the declarative way to schedule `tape backup`.
///
/// Unset renders no CronJob, so adding the field changes nothing about the
/// workload for CRs that do not ask for a backup. Set, it renders a CronJob
/// that invokes the same CLI verb an operator would run by hand — with the
/// instance's own image and client Service URL, which is the whole reason to
/// render it rather than hand-author one alongside the CR.
#[test]
fn backup_cron_job_is_opt_in_and_invokes_the_backup_verb() {
    use tape::operator::TapeBackupSpec;

    let plain = render(&Tape::new("tape", spec(3)));
    assert!(
        plain.iter().all(|o| o["kind"] != "CronJob"),
        "no backup policy must render no CronJob"
    );

    let mut configured = spec(3);
    configured.backup = Some(TapeBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "17 3 * * *".into(),
            destination: "gs://tape-backups/orders".into(),
            retention_secs: Some(604_800),
        },
        admin_token_secret: Some("tape-backup-token".into()),
    });
    let objects = render(&Tape::new("tape", configured));

    let cj = of_kind(&objects, "CronJob");
    assert_eq!(cj["metadata"]["name"], "tape-backup");
    assert_eq!(cj["spec"]["schedule"], "17 3 * * *");

    let pod = &cj["spec"]["jobTemplate"]["spec"]["template"]["spec"];
    let container = &pod["containers"][0];
    assert_eq!(container["command"][0], "tape");
    assert_eq!(
        container["image"], "tape:test",
        "the runner uses the instance's image, so it cannot drift from it"
    );

    let args: Vec<&str> = container["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert_eq!(args[0], "backup");
    assert!(args.contains(&"--dest"));
    assert!(args.contains(&"gs://tape-backups/orders"));
    assert!(args.contains(&"--retention-secs"));
    assert!(args.contains(&"604800"));
    // The URL must address this instance's client Service on the client port,
    // not the headless Service and not the raft port.
    assert!(
        args.contains(&"http://tape.default.svc.cluster.local:7137"),
        "backup pulls from the instance's own client Service; args were {args:?}"
    );

    // Runs under the dedicated backup identity, not the serving one: only this
    // pod needs cloud object-store credentials.
    assert_eq!(pod["serviceAccountName"], "tape-backup");

    // The admin token is projected as the env var `tape backup --token`
    // already falls back to; `/admin/backup` needs `admin` on `*`.
    let token = container["env"]
        .as_array()
        .expect("env array")
        .iter()
        .find(|e| e["name"] == "TAPE_BACKUP_TOKEN")
        .expect("admin token env when the CR names a secret");
    assert_eq!(token["valueFrom"]["secretKeyRef"]["name"], "tape-backup-token");
    assert_eq!(token["valueFrom"]["secretKeyRef"]["key"], "token");
}

/// #2574 — the `<name>-backup` ServiceAccount is rendered whether or not a
/// backup is currently scheduled (lumen's #808 pattern).
///
/// It is the binding target for cloud IAM — GKE Workload Identity annotates
/// it — so an identity whose lifecycle followed the schedule would drop that
/// binding every time the policy was toggled off. The workload ServiceAccount
/// must still come first in the render order: `of_kind` takes the first match,
/// and the pre-existing identity assertion resolves through it.
#[test]
fn backup_service_account_exists_independently_of_the_schedule() {
    use tape::operator::TapeBackupSpec;

    let names = |objs: &[Value]| -> Vec<String> {
        objs.iter()
            .filter(|o| o["kind"] == "ServiceAccount")
            .map(|o| o["metadata"]["name"].as_str().unwrap().to_string())
            .collect()
    };

    assert_eq!(
        names(&render(&Tape::new("tape", spec(3)))),
        vec!["tape".to_string(), "tape-backup".to_string()],
        "the backup identity is rendered even with no schedule configured"
    );

    let mut configured = spec(3);
    configured.backup = Some(TapeBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "0 4 * * *".into(),
            destination: "gs://tape-backups/orders".into(),
            retention_secs: None,
        },
        admin_token_secret: None,
    });
    let objects = render(&Tape::new("tape", configured));
    assert_eq!(
        names(&objects),
        vec!["tape".to_string(), "tape-backup".to_string()],
        "configuring a schedule must not render a second copy of the identity"
    );

    let sa = objects
        .iter()
        .find(|o| o["kind"] == "ServiceAccount" && o["metadata"]["name"] == "tape-backup")
        .expect("backup ServiceAccount");
    assert_eq!(
        sa["metadata"]["labels"]["app.kubernetes.io/component"], "backup",
        "the backup identity must not be labelled as a serving pod"
    );
}

/// #2574 — `retentionSecs` and `adminTokenSecret` are both optional, and
/// omitting them must not emit an empty flag or an unresolvable env var.
#[test]
fn backup_cron_job_omits_unset_retention_and_token() {
    use tape::operator::TapeBackupSpec;

    let mut minimal = spec(1);
    minimal.backup = Some(TapeBackupSpec {
        policy: service_backup::ScheduledBackupPolicy {
            schedule: "0 * * * *".into(),
            destination: "file:///var/backups/tape".into(),
            retention_secs: None,
        },
        admin_token_secret: None,
    });
    let objects = render(&Tape::new("tape", minimal));
    let container =
        &of_kind(&objects, "CronJob")["spec"]["jobTemplate"]["spec"]["template"]["spec"]
            ["containers"][0];

    let args: Vec<&str> = container["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    assert!(
        !args.contains(&"--retention-secs"),
        "omitted retention must keep every object, not pass an empty flag"
    );
    assert!(
        container["env"].as_array().expect("env array").is_empty(),
        "an auth:off instance needs no admin token"
    );
}

/// #2574 — the committed CRD carries the `backup` properties, and has not
/// drifted from the generator at all.
///
/// Asserting on the generator alone would pass while the checked-in
/// `k8s/operator/crd.yaml` still pruned `spec.backup` in-cluster — a
/// structural schema drops properties it does not declare. That is not
/// hypothetical: this test's byte-equality half caught the committed file
/// still missing `spec.topics` from #2557, which had never been regenerated.
/// The acceptance harness renders the CRD from the binary
/// (`benchmarks/gcp-operator-acceptance/scripts/render-manifests.sh`), so the
/// stale file went unnoticed there.
#[test]
fn generated_crd_carries_the_backup_properties() {
    let yaml = crd_yaml();
    let committed = include_str!("../k8s/operator/crd.yaml");

    for field in [
        "backup",
        "schedule",
        "destination",
        "retentionSecs",
        "adminTokenSecret",
    ] {
        assert!(
            yaml.contains(field),
            "generated CRD schema is missing `{field}`"
        );
        assert!(
            committed.contains(field),
            "committed CRD is missing `{field}` — the API server would prune it"
        );
    }

    // Deliberately `assert!`, not `assert_eq!`: the two documents are ~8 KB
    // each and dumping both escaped into the failure output buries the one
    // line that matters.
    assert!(
        committed.trim_end() == yaml.trim_end(),
        "apps/tape/k8s/operator/crd.yaml has drifted from the generator — \
         regenerate it with `cargo run -p tape --bin tape --features operator \
         -- k8s crd render --out apps/tape/k8s/operator/crd.yaml`"
    );
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

/// The binary installs this process-level provider before CLI dispatch. The
/// shared helper is intentionally safe when an earlier TLS path installed it.
#[test]
fn rustls_provider_install_is_idempotent() {
    peer_tls::install_default_crypto_provider();
    peer_tls::install_default_crypto_provider();
}
// HANDWRITE-END

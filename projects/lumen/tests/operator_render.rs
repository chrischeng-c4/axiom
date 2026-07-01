// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, Lumen-owned raft wiring, and observability toggles.
#![cfg(feature = "operator")]

use kube::api::ObjectMeta;
use lumen::operator::crd::{AuthMode, Autoscaling, LogFormat, ServingSpec};
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
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        tokens_secret: None,
        serving: ServingSpec {
            autoscaling: Autoscaling {
                min_replicas: 1,
                max_replicas: 3,
                target_cpu_utilization: 70,
            },
            ..Default::default()
        },
        observability: false,
    }
}

fn prod_spec() -> LumenSpec {
    LumenSpec {
        image: "registry.example.com/lumen:1.2.3".into(),
        image_pull_policy: Some("Always".into()),
        shard_count: 6,
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Json,
        log_level: Some("warn".into()),
        auth: AuthMode::Required,
        tokens_secret: Some("lumen-tokens".into()),
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
    ] {
        assert!(
            names.contains(&required.to_string()),
            "missing env {required}; have {names:?}"
        );
    }
    // Single member, no raft consensus at replicasPerShard:1 → no raft
    // peer-identity env.
    for absent in ["REPLICAS_PER_SHARD", "VOTER_COUNT", "LUMEN_HEADLESS_SERVICE"] {
        assert!(
            !names.contains(&absent.to_string()),
            "unexpected raft env {absent} at replicasPerShard:1; have {names:?}"
        );
    }
    // auth=off and no log level → those env vars are absent.
    assert!(!names.contains(&"LUMEN_TOKENS".to_string()));
    assert!(!names.contains(&"LUMEN_TOKEN_REGISTRY_FILE".to_string()));
    assert!(!names.contains(&"LUMEN_LOG_LEVEL".to_string()));

    // Durable raft PVC (#812): the WAL survives pod reschedule/eviction/node
    // loss even for a single-member deployer — not just an emptyDir.
    let mounts = c["volumeMounts"].as_array().unwrap();
    assert!(
        mounts
            .iter()
            .any(|m| m["name"] == "raft" && m["mountPath"] == "/var/lib/lumen"),
        "missing raft volumeMount; have {mounts:?}"
    );
    let vcts = sts["spec"]["volumeClaimTemplates"].as_array().unwrap();
    assert_eq!(vcts.len(), 1);
    assert_eq!(vcts[0]["metadata"]["name"], "raft");
    assert_eq!(
        vcts[0]["spec"]["resources"]["requests"]["storage"],
        "20Gi"
    );
}

#[test]
fn configmap_tracks_serving_spec() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(cm["data"]["SHARD_COUNT"], "1");
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "pretty");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "off");
    assert_eq!(cm["data"]["LUMEN_PORT"], "7373");
    // No log level set → key omitted.
    assert!(cm["data"]["LUMEN_LOG_LEVEL"].is_null());
}

#[test]
fn hpa_is_rendered_for_single_replica_serving() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 3);
    assert_eq!(hpa["spec"]["scaleTargetRef"]["name"], "search");
    // The serving fleet is a StatefulSet (#812) — the HPA must target it, not
    // the retired Deployment kind.
    assert_eq!(hpa["spec"]["scaleTargetRef"]["kind"], "StatefulSet");
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
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "json");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "required");

    // observability=true → monitoring objects present.
    assert!(has(&objs, "ServiceMonitor", "lumen"));
    assert!(has(&objs, "PrometheusRule", "lumen"));
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

    // The raft PVC shape is unchanged by #812 — it was already unconditional
    // in the raft-HA regime.
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
    ] {
        assert!(
            yaml.contains(needle),
            "CRD should publish token registry shape in tokensSecret docs; missing `{needle}`: {yaml}"
        );
    }
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
// CODEGEN-END

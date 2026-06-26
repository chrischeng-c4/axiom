// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, Relay broker wiring, and the BYO-broker / observability
//! toggles.
#![cfg(feature = "operator")]

use kube::api::ObjectMeta;
use lumen::operator::crd::{AuthMode, Autoscaling, BrokerSpec, LogFormat, ServingSpec};
use lumen::operator::render::{broker_url, render};
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
        broker: BrokerSpec {
            replicas: 1,
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
        },
        broker: BrokerSpec {
            external_url: None,
            image: "registry.example.com/relay:1.2.3".into(),
            subject: "lumen-wal".into(),
            replicas: 3,
            storage: "100Gi".into(),
            storage_class: Some("ssd".into()),
            cpu: "2".into(),
            memory: "4Gi".into(),
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

    // Serving objects, in the CR's namespace, named off the instance.
    for (kind, name) in [
        ("ServiceAccount", "search"),
        ("ConfigMap", "search-config"),
        ("Deployment", "search"),
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
    // Managed Relay broker.
    for (kind, name) in [
        ("StatefulSet", "search-relay"),
        ("Service", "search-relay"),
        ("Service", "search-relay-headless"),
        ("PodDisruptionBudget", "search-relay"),
    ] {
        assert!(
            has(&objs, kind, name),
            "expected {kind}/{name}; got {:?}",
            kinds(&objs)
        );
    }
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
fn deployment_wires_serving_contract() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let dep = find(&objs, "Deployment", "search");

    // HPA floor == apply-time replicas; zero-downtime rollout.
    assert_eq!(dep["spec"]["replicas"], 1);
    assert_eq!(
        dep["spec"]["strategy"]["rollingUpdate"]["maxUnavailable"],
        0
    );
    assert_eq!(dep["spec"]["strategy"]["rollingUpdate"]["maxSurge"], 1);

    let c = &dep["spec"]["template"]["spec"]["containers"][0];
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

    // Env: downward-API identity + the Relay write-log + config-driven knobs.
    let names = env_names(c);
    for required in [
        "POD_NAME",
        "POD_NAMESPACE",
        "LUMEN_HOST",
        "LUMEN_WAL",
        "LUMEN_RELAY_URL",
        "LUMEN_RELAY_SUBJECT",
        "LUMEN_RELAY_SUBSCRIBER_ID",
        "SHARD_COUNT",
        "LUMEN_AUTH",
    ] {
        assert!(
            names.contains(&required.to_string()),
            "missing env {required}; have {names:?}"
        );
    }
    // auth=off and no log level → those env vars are absent.
    assert!(!names.contains(&"LUMEN_TOKENS".to_string()));
    assert!(!names.contains(&"LUMEN_LOG_LEVEL".to_string()));
}

#[test]
fn configmap_and_broker_url_track_spec() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(cm["data"]["SHARD_COUNT"], "1");
    assert_eq!(cm["data"]["LUMEN_RELAY_URL"], "http://search-relay:7000");
    assert_eq!(cm["data"]["LUMEN_RELAY_SUBJECT"], "lumen-wal");
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "pretty");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "off");
    assert_eq!(cm["data"]["LUMEN_PORT"], "7373");
    // No log level set → key omitted.
    assert!(cm["data"]["LUMEN_LOG_LEVEL"].is_null());

    assert_eq!(broker_url(&l), "http://search-relay:7000");
}

#[test]
fn hpa_and_single_replica_relay_are_rendered() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 3);
    assert_eq!(hpa["spec"]["scaleTargetRef"]["name"], "search");

    // Managed relay-server is one durable broker. HA uses an external Relay URL
    // until relay-raft exposes subscribe/len for Lumen.
    let sts = find(&objs, "StatefulSet", "search-relay");
    assert_eq!(sts["spec"]["replicas"], 1);
    let command = sts["spec"]["template"]["spec"]["containers"][0]["command"]
        .as_array()
        .unwrap();
    let joined: Vec<&str> = command.iter().map(|a| a.as_str().unwrap()).collect();
    assert_eq!(joined, vec!["relay-server"]);
    // Base PVC: no storageClassName (portable / cluster default).
    assert!(sts["spec"]["volumeClaimTemplates"][0]["spec"]["storageClassName"].is_null());
}

#[test]
fn prod_wires_managed_relay_and_auth() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);

    // Managed relay-server is intentionally clamped to one broker; externalUrl
    // is the HA path until relay-raft exposes Lumen's subscribe/len surface.
    let sts = find(&objs, "StatefulSet", "lumen-relay");
    assert_eq!(sts["spec"]["replicas"], 1);
    let c0 = &sts["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(c0["image"], "registry.example.com/relay:1.2.3");
    assert_eq!(c0["ports"][0]["containerPort"], 7000);
    let data_dir = c0["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "RELAY_DATA_DIR")
        .expect("RELAY_DATA_DIR env");
    assert_eq!(data_dir["value"], "/data");
    // Cloud SSD storage class + size from spec.
    assert_eq!(
        sts["spec"]["volumeClaimTemplates"][0]["spec"]["storageClassName"],
        "ssd"
    );
    assert_eq!(
        sts["spec"]["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"],
        "100Gi"
    );

    // auth=required + tokensSecret → LUMEN_TOKENS env from the Secret.
    let dep = find(&objs, "Deployment", "lumen");
    let c = &dep["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(c["image"], "registry.example.com/lumen:1.2.3");
    assert_eq!(c["imagePullPolicy"], "Always");
    let tokens = c["env"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["name"] == "LUMEN_TOKENS")
        .expect("LUMEN_TOKENS env");
    assert_eq!(tokens["valueFrom"]["secretKeyRef"]["name"], "lumen-tokens");
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
fn external_broker_skips_managed_relay_objects() {
    let mut spec = dev_spec();
    spec.broker = BrokerSpec {
        external_url: Some("http://shared-relay.infra:7000".into()),
        ..Default::default()
    };
    let l = lumen("search", spec);
    let objs = render(&l);

    // BYO broker: no managed Relay objects at all.
    assert!(!has(&objs, "StatefulSet", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay"));
    assert!(!has(&objs, "Service", "search-relay-headless"));
    assert!(!has(&objs, "PodDisruptionBudget", "search-relay"));

    // Serving still wired to the external URL.
    assert_eq!(broker_url(&l), "http://shared-relay.infra:7000");
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(
        cm["data"]["LUMEN_RELAY_URL"],
        "http://shared-relay.infra:7000"
    );
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
    assert!(has(&objs, "StatefulSet", "search"), "got {:?}", kinds(&objs));
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
}
// CODEGEN-END

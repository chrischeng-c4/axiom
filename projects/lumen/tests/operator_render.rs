// SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-tests.md#unit-test
// CODEGEN-BEGIN
//! Operator render tests: a `Lumen` spec → the exact child objects, with no
//! cluster. This encodes the operational knowledge that lives in `k8s/base` +
//! the overlays as executable assertions — replicas, env wiring, resources,
//! probes, owner refs, NATS clustering, and the BYO-broker / observability
//! toggles.
#![cfg(feature = "operator")]

use kube::api::ObjectMeta;
use lumen::operator::crd::{AuthMode, Autoscaling, LogFormat, NatsSpec, ServingSpec};
use lumen::operator::render::{nats_url, render};
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
        nats: NatsSpec {
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
        nats: NatsSpec {
            external_url: None,
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
    // Managed NATS broker.
    for (kind, name) in [
        ("ConfigMap", "search-nats-config"),
        ("StatefulSet", "search-nats"),
        ("Service", "search-nats"),
        ("Service", "search-nats-headless"),
        ("PodDisruptionBudget", "search-nats"),
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

    // Env: downward-API identity + the NATS write-log + config-driven knobs.
    let names = env_names(c);
    for required in [
        "POD_NAME",
        "POD_NAMESPACE",
        "LUMEN_HOST",
        "LUMEN_WAL",
        "LUMEN_NATS_URL",
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
fn configmap_and_nats_url_track_spec() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(cm["data"]["SHARD_COUNT"], "1");
    assert_eq!(cm["data"]["LUMEN_NATS_URL"], "nats://search-nats:4222");
    assert_eq!(cm["data"]["LUMEN_LOG_FORMAT"], "pretty");
    assert_eq!(cm["data"]["LUMEN_AUTH"], "off");
    assert_eq!(cm["data"]["LUMEN_PORT"], "7373");
    // No log level set → key omitted.
    assert!(cm["data"]["LUMEN_LOG_LEVEL"].is_null());

    assert_eq!(nats_url(&l), "nats://search-nats:4222");
}

#[test]
fn hpa_and_single_replica_nats_have_no_routes() {
    let l = lumen("search", dev_spec());
    let objs = render(&l);

    let hpa = find(&objs, "HorizontalPodAutoscaler", "search");
    assert_eq!(hpa["spec"]["minReplicas"], 1);
    assert_eq!(hpa["spec"]["maxReplicas"], 3);
    assert_eq!(hpa["spec"]["scaleTargetRef"]["name"], "search");

    // 1 replica → plain JetStream args, no cluster routes.
    let sts = find(&objs, "StatefulSet", "search-nats");
    assert_eq!(sts["spec"]["replicas"], 1);
    let args = sts["spec"]["template"]["spec"]["containers"][0]["args"]
        .as_array()
        .unwrap();
    let joined: Vec<&str> = args.iter().map(|a| a.as_str().unwrap()).collect();
    assert!(
        !joined.iter().any(|a| *a == "--routes"),
        "single-replica must not wire routes: {joined:?}"
    );
    // Base PVC: no storageClassName (portable / cluster default).
    assert!(sts["spec"]["volumeClaimTemplates"][0]["spec"]["storageClassName"].is_null());
}

#[test]
fn prod_clusters_nats_and_wires_auth() {
    let l = lumen("lumen", prod_spec());
    let objs = render(&l);

    // 3 NATS replicas → clustered JetStream routes to per-pod headless DNS.
    let sts = find(&objs, "StatefulSet", "lumen-nats");
    assert_eq!(sts["spec"]["replicas"], 3);
    let args: Vec<String> = sts["spec"]["template"]["spec"]["containers"][0]["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    let routes_idx = args
        .iter()
        .position(|a| a == "--routes")
        .expect("routes wired");
    let routes = &args[routes_idx + 1];
    for i in 0..3 {
        assert!(
            routes.contains(&format!("nats://lumen-nats-{i}.lumen-nats-headless:6222")),
            "route {i} missing: {routes}"
        );
    }
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
fn external_nats_skips_broker_objects() {
    let mut spec = dev_spec();
    spec.nats = NatsSpec {
        external_url: Some("nats://shared-broker.infra:4222".into()),
        ..Default::default()
    };
    let l = lumen("search", spec);
    let objs = render(&l);

    // BYO broker: no NATS objects at all.
    assert!(!has(&objs, "StatefulSet", "search-nats"));
    assert!(!has(&objs, "Service", "search-nats"));
    assert!(!has(&objs, "Service", "search-nats-headless"));
    assert!(!has(&objs, "ConfigMap", "search-nats-config"));
    assert!(!has(&objs, "PodDisruptionBudget", "search-nats"));

    // Serving still wired to the external URL.
    assert_eq!(nats_url(&l), "nats://shared-broker.infra:4222");
    let cm = find(&objs, "ConfigMap", "search-config");
    assert_eq!(
        cm["data"]["LUMEN_NATS_URL"],
        "nats://shared-broker.infra:4222"
    );
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

//! Deployment contract for Sift's common `/var/lib/sift` durable root.

use std::{fs, path::PathBuf, process::Command};

use serde_json::{json, Value};
use service_k8s::ManagedService;
use sift::operator::Sift;

fn sift_resource() -> Sift {
    serde_json::from_value(json!({
        "apiVersion": "sift.axiom.dev/v1alpha1",
        "kind": "Sift",
        "metadata": {
            "name": "events",
            "namespace": "observability",
            "uid": "sift-owner-uid"
        },
        "spec": {
            "image": "example.invalid/sift:test",
            "peerTlsSecret": "sift-peer-tls",
            "dataSize": "10Gi",
            "auth": "off"
        }
    }))
    .expect("decode Sift resource")
}

fn object<'a>(objects: &'a [Value], kind: &str, name: &str) -> &'a Value {
    objects
        .iter()
        .find(|object| object["kind"] == kind && object["metadata"]["name"] == name)
        .unwrap_or_else(|| panic!("missing {kind}/{name}"))
}

#[test]
fn every_runtime_image_prepares_a_private_nonroot_volume() {
    for (name, dockerfile, stage) in [
        ("source", include_str!("../Dockerfile"), "build"),
        ("release", include_str!("../Dockerfile.release"), "fetch"),
        ("test", include_str!("../Dockerfile.test"), "data-root"),
    ] {
        assert!(
            dockerfile.contains("SIFT_DATA_DIR=/var/lib/sift"),
            "{name} image must use the shared data root"
        );
        assert!(
            dockerfile.contains("VOLUME [\"/var/lib/sift\"]"),
            "{name} image must declare a durable volume"
        );
        assert!(
            dockerfile.contains(&format!(
                "COPY --from={stage} --chown=65532:65532 --chmod=0700 /image-root/var/lib/sift /var/lib/sift"
            )),
            "{name} image must make the mounted root writable by its nonroot runtime"
        );
    }
}

#[test]
fn the_official_compose_example_uses_a_named_volume() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("compose.yaml");
    let body = fs::read_to_string(path).expect("official compose example must exist");
    let compose: serde_yaml::Value = serde_yaml::from_str(&body).expect("parse compose yaml");
    assert_eq!(
        compose["services"]["sift"]["volumes"][0],
        "sift-data:/var/lib/sift"
    );
    assert!(compose["volumes"]["sift-data"].is_mapping());
}

#[test]
fn every_operator_role_uses_the_common_private_persistent_root() {
    let objects = sift_resource().render();
    for (kind, name, role) in [
        ("StatefulSet", "events-store", "store"),
        ("StatefulSet", "events-control", "control"),
        ("Deployment", "events-gateway", "gateway"),
        ("Deployment", "events-query", "query"),
    ] {
        let workload = object(&objects, kind, name);
        let pod = &workload["spec"]["template"]["spec"];
        assert_eq!(pod["securityContext"]["fsGroup"], 65532, "{name}");
        assert_eq!(
            pod["securityContext"]["fsGroupChangePolicy"], "OnRootMismatch",
            "{name}"
        );
        assert!(!pod.to_string().contains("emptyDir"), "{name}");
        let init = &pod["initContainers"][0];
        assert_eq!(init["name"], "prepare-data-root", "{name}");
        assert_eq!(init["securityContext"]["runAsUser"], 0, "{name}");
        assert_eq!(init["volumeMounts"][0]["mountPath"], "/var/lib/sift");
        let script = init["args"][0].as_str().expect("init script");
        assert!(script.contains("chown 65532:65532 /var/lib/sift"));
        assert!(script.contains("chmod 0700 /var/lib/sift"));
        let container = &pod["containers"][0];
        assert_eq!(container["volumeMounts"][0]["mountPath"], "/var/lib/sift");
        assert_eq!(
            container["args"],
            json!(["serve", "--role", role, "--data-dir", "/var/lib/sift"])
        );
        if kind == "StatefulSet" {
            assert_eq!(
                workload["spec"]["volumeClaimTemplates"][0]["metadata"]["name"],
                "data"
            );
        } else {
            let pvc_name = format!("{name}-data");
            object(&objects, "PersistentVolumeClaim", &pvc_name);
            assert_eq!(
                pod["volumes"][0]["persistentVolumeClaim"]["claimName"],
                pvc_name
            );
        }
    }

    assert_eq!(
        object(&objects, "StatefulSet", "events-store")["spec"]["replicas"],
        3
    );
    assert_eq!(
        object(&objects, "StatefulSet", "events-control")["spec"]["replicas"],
        3
    );
    let agent = object(&objects, "DaemonSet", "events-agent");
    let agent_pod = &agent["spec"]["template"]["spec"];
    assert_eq!(
        agent_pod["securityContext"]["fsGroupChangePolicy"],
        "OnRootMismatch"
    );
    assert!(agent.to_string().contains("/var/lib/sift/agent"));
    assert!(!agent.to_string().contains("emptyDir"));
    assert!(
        agent_pod["initContainers"][0]["securityContext"]["capabilities"]["add"]
            .as_array()
            .is_some_and(|capabilities| capabilities.contains(&json!("DAC_OVERRIDE")))
    );
    assert_eq!(
        agent_pod["containers"][0]["securityContext"]["capabilities"]["add"],
        json!(["DAC_OVERRIDE"])
    );
}

#[test]
fn the_agent_keeps_checkpoints_below_the_common_root() {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "k8s",
            "collector",
            "render",
            "--namespace",
            "observability",
            "--image",
            "example.invalid/sift:test",
        ])
        .output()
        .expect("render collector");
    assert!(output.status.success());
    let yaml = String::from_utf8(output.stdout).expect("collector yaml is utf-8");
    assert!(yaml.contains("path: /var/lib/sift"));
    assert!(yaml.contains("mountPath: /var/lib/sift"));
    assert!(yaml.contains("/var/lib/sift/agent/checkpoint.json"));
    assert!(yaml.contains("/var/lib/sift/agent/rejected.jsonl"));
    assert!(!yaml.contains("/var/lib/sift-collector"));
}

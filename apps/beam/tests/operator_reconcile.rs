// HANDWRITE-BEGIN gap="missing-generator:unit-test:0e5de3f4" tracker="#2152" reason="scaffold for apps/beam/tests/operator_reconcile.rs — fill in by hand and update tracker when codegen is ready"
use std::collections::HashMap;
use beam::operator::{Beam, BeamSpec};
use service_k8s::{ManagedService, ReadyFacts};

fn spec() -> BeamSpec {
    BeamSpec {
        image: "beam:test".into(),
        host: Some("0.0.0.0".into()),
        port: Some(7373),
        log_level: Some("debug".into()),
        grace_secs: Some(45),
    }
}

#[test]
fn test_beam_crd_spec_and_status() {
    let beam = Beam::new("my-beam", spec());
    assert_eq!(beam.spec.image, "beam:test");
    assert_eq!(beam.spec.port, Some(7373));
    assert_eq!(beam.spec.grace_secs, Some(45));
}

#[test]
fn test_beam_operator_render() {
    let beam = Beam::new("my-beam", spec());
    let objs = beam.render();

    assert_eq!(objs.len(), 2);

    let svc = objs.iter().find(|o| o["kind"] == "Service").expect("Service missing");
    assert_eq!(svc["metadata"]["name"], "my-beam");
    assert_eq!(svc["spec"]["ports"][0]["port"], 7373);
    assert_eq!(svc["spec"]["selector"]["app"], "my-beam");

    let dep = objs.iter().find(|o| o["kind"] == "Deployment").expect("Deployment missing");
    assert_eq!(dep["metadata"]["name"], "my-beam");
    assert_eq!(dep["spec"]["replicas"], 1);
    assert_eq!(dep["spec"]["template"]["spec"]["terminationGracePeriodSeconds"], 45);
    
    let container = &dep["spec"]["template"]["spec"]["containers"][0];
    assert_eq!(container["image"], "beam:test");
    
    let envs = container["env"].as_array().expect("env array");
    let beam_port_env = envs.iter().find(|e| e["name"] == "BEAM_PORT").expect("BEAM_PORT missing");
    assert_eq!(beam_port_env["value"], "7373");
}

#[test]
fn test_beam_operator_readiness_and_status() {
    let beam = Beam::new("my-beam", spec());

    let targets = beam.readiness_targets();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].kind, "Deployment");
    assert_eq!(targets[0].name, "my-beam");

    let mut ready_map = HashMap::new();
    ready_map.insert("my-beam".to_string(), 0);
    let patch = beam.status_patch(&ReadyFacts { ready: ready_map });
    assert_eq!(patch["status"]["phase"], "Reconciling");

    let mut ready_map = HashMap::new();
    ready_map.insert("my-beam".to_string(), 1);
    let patch = beam.status_patch(&ReadyFacts { ready: ready_map });
    assert_eq!(patch["status"]["phase"], "Ready");
}
// HANDWRITE-END

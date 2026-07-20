// HANDWRITE-BEGIN gap="missing-generator:unit-test:f56da2b2" tracker="#2154" reason="scaffold for apps/beam/tests/k8s_render.rs — fill in by hand and update tracker when codegen is ready"
use beam::operator::{Beam, BeamSpec};
use service_k8s::ManagedService;

#[test]
fn test_dev_profile_rendering() {
    let mut beam = Beam::new("my-beam", BeamSpec {
        image: "beam:test".to_string(),
        host: None,
        port: Some(7373),
        log_level: None,
        grace_secs: None,
        replicas: None,
        storage_size: None,
        request_gpu: None,
    });
    beam.metadata.namespace = Some("default".to_string());
    beam.metadata.labels.get_or_insert_with(std::collections::BTreeMap::new)
        .insert("profile".to_string(), "dev".to_string());

    let manifests = beam.render();
    
    // Dev profile should render: 1 Service, 1 StatefulSet. No PDB, No CronJob.
    assert_eq!(manifests.len(), 2);
    assert_eq!(manifests[0]["kind"], "Service");
    assert_eq!(manifests[1]["kind"], "StatefulSet");

    let spec = &manifests[1]["spec"];
    assert_eq!(spec["replicas"], 1);
    
    // Check PVC storage size request
    let storage = &spec["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"];
    assert_eq!(storage, "1Gi");

    // Check no GPU limits
    let container = &spec["template"]["spec"]["containers"][0];
    assert!(container["resources"]["limits"]["nvidia.com/gpu"].is_null());

    // Check environment vars mapping auth directory/secrets
    let envs = container["env"].as_array().unwrap();
    let data_dir_env = envs.iter().find(|e| e["name"] == "BEAM_DATA_DIR").unwrap();
    assert_eq!(data_dir_env["value"], "/data");
    let auth_env = envs.iter().find(|e| e["name"] == "BEAM_AUTH").unwrap();
    assert_eq!(auth_env["value"], "required");
    
    // Check liveness/readiness probes
    assert_eq!(container["livenessProbe"]["httpGet"]["path"], "/healthz");
    assert_eq!(container["readinessProbe"]["httpGet"]["path"], "/readyz");
}

#[test]
fn test_prod_profile_rendering() {
    let mut beam = Beam::new("my-beam", BeamSpec {
        image: "beam:test".to_string(),
        host: None,
        port: Some(7373),
        log_level: None,
        grace_secs: None,
        replicas: None,
        storage_size: None,
        request_gpu: None,
    });
    beam.metadata.namespace = Some("production".to_string());
    beam.metadata.labels.get_or_insert_with(std::collections::BTreeMap::new)
        .insert("profile".to_string(), "prod".to_string());

    let manifests = beam.render();
    
    // Prod profile renders: Service, StatefulSet, PDB, CronJob (backup)
    assert_eq!(manifests.len(), 4);
    assert_eq!(manifests[0]["kind"], "Service");
    assert_eq!(manifests[1]["kind"], "StatefulSet");
    assert_eq!(manifests[2]["kind"], "PodDisruptionBudget");
    assert_eq!(manifests[3]["kind"], "CronJob");

    let spec = &manifests[1]["spec"];
    assert_eq!(spec["replicas"], 2);
    
    // Check PVC storage size request
    let storage = &spec["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"];
    assert_eq!(storage, "20Gi");

    // Check GPU limits, node placement selectors, and tolerations
    let pod_spec = &spec["template"]["spec"];
    let container = &pod_spec["containers"][0];
    assert_eq!(container["resources"]["limits"]["nvidia.com/gpu"], "1");
    assert_eq!(pod_spec["nodeSelector"]["accelerator"], "nvidia-gpu");
    assert_eq!(pod_spec["tolerations"][0]["key"], "nvidia.com/gpu");

    // Check CronJob details
    let cron_spec = &manifests[3]["spec"];
    assert_eq!(cron_spec["schedule"], "0 2 * * *");
    let container_command = &cron_spec["jobTemplate"]["spec"]["template"]["spec"]["containers"][0]["command"];
    assert_eq!(container_command[0], "beam");
    assert_eq!(container_command[1], "dump");
}

#[test]
fn test_spec_overrides() {
    let mut beam = Beam::new("my-beam", BeamSpec {
        image: "beam:test".to_string(),
        host: None,
        port: Some(7373),
        log_level: None,
        grace_secs: None,
        replicas: Some(5),
        storage_size: Some("100Gi".to_string()),
        request_gpu: Some(false),
    });
    beam.metadata.labels.get_or_insert_with(std::collections::BTreeMap::new)
        .insert("profile".to_string(), "prod".to_string());

    let manifests = beam.render();
    let spec = &manifests[1]["spec"];
    
    // Replicas override
    assert_eq!(spec["replicas"], 5);
    
    // Storage override
    let storage = &spec["volumeClaimTemplates"][0]["spec"]["resources"]["requests"]["storage"];
    assert_eq!(storage, "100Gi");

    // GPU override
    let container = &spec["template"]["spec"]["containers"][0];
    assert!(container["resources"]["limits"]["nvidia.com/gpu"].is_null());
}
// HANDWRITE-END

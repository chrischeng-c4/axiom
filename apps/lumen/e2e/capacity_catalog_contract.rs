#![cfg(feature = "operator")]

//! Black-box capacity catalog contract tests for #2946.
//!
//! Validates capacity catalog JSON consumption matching the Terraform schema,
//! direct GCE machine type resolution, fail-closed preflight, request derivation,
//! cross-namespace placement anti-affinity, and reapplication state ownership.

use kube::api::ObjectMeta;
use lumen::operator::capacity::{
    apply_capacity_reapplication, decide_capacity_spec, decide_storage, decide_transition,
    derive_requests, preflight_capacity, preflight_capacity_with_nodes, resolve_machine_type,
    resolve_shared_placement, CapacityCatalog, CapacityPolicy, CapacityRequest, CapacitySpec,
    CapacityState, CapacityStorage, CapacityVector, CatalogEntry, Placement, RejectionReason,
    StableSelector,
};
use lumen::operator::crd::{
    AuthMode, LogFormat, Lumen, LumenSpec, PlacementSpec, ReshardPolicy, ServingSpec, ShardMapSpec,
};
use serde_json::json;

fn sample_catalog_json() -> &'static str {
    r#"{
      "version": "1.0.0",
      "entries": [
        {
          "machine_type": "e2-standard-2",
          "selector": "lumen.axiom.dev/capacity-profile=e2-standard-2",
          "stable_selector": {
            "key": "lumen.axiom.dev/capacity-profile",
            "value": "e2-standard-2"
          },
          "max_nodes": 10,
          "min_nodes": 0,
          "lifecycle_state": "ready",
          "pool_group": "lumen-data"
        },
        {
          "machine_type": "n2-standard-4",
          "selector": "lumen.axiom.dev/capacity-profile=n2-standard-4",
          "stable_selector": {
            "key": "lumen.axiom.dev/capacity-profile",
            "value": "n2-standard-4"
          },
          "max_nodes": 8,
          "min_nodes": 0,
          "lifecycle_state": "ready",
          "pool_group": "lumen-data"
        },
        {
          "machine_type": "n2-standard-8",
          "selector": "lumen.axiom.dev/capacity-profile=n2-standard-8",
          "stable_selector": {
            "key": "lumen.axiom.dev/capacity-profile",
            "value": "n2-standard-8"
          },
          "max_nodes": 6,
          "min_nodes": 0,
          "lifecycle_state": "ready",
          "pool_group": "lumen-data"
        },
        {
          "machine_type": "c2-standard-8",
          "selector": "lumen.axiom.dev/capacity-profile=c2-standard-8",
          "stable_selector": {
            "key": "lumen.axiom.dev/capacity-profile",
            "value": "c2-standard-8"
          },
          "max_nodes": 4,
          "min_nodes": 0,
          "lifecycle_state": "ready",
          "pool_group": "lumen-data"
        }
      ]
    }"#
}

fn test_lumen(name: &str, ns: &str, machine_type: &str) -> Lumen {
    let spec = LumenSpec {
        image: "lumen:latest".into(),
        image_pull_policy: None,
        placement: PlacementSpec {
            initial_machine_type: machine_type.to_string(),
            ..Default::default()
        },
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: std::collections::BTreeMap::new(),
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
    };
    let mut l = Lumen::new(name, spec);
    l.metadata = ObjectMeta {
        name: Some(name.to_string()),
        namespace: Some(ns.to_string()),
        uid: Some(format!("uid-{name}")),
        generation: Some(1),
        ..Default::default()
    };
    l
}

#[test]
fn catalog_json_parses_matching_terraform_catalog_tf_schema() {
    let catalog: CapacityCatalog =
        CapacityCatalog::from_json(sample_catalog_json()).expect("parse catalog JSON");
    assert_eq!(catalog.version, "1.0.0");
    assert_eq!(catalog.entries.len(), 4);

    let e2 = catalog
        .entries
        .iter()
        .find(|e| e.machine_type == "e2-standard-2")
        .expect("e2-standard-2 entry");
    assert_eq!(e2.stable_selector.key, "lumen.axiom.dev/capacity-profile");
    assert_eq!(e2.stable_selector.value, "e2-standard-2");
    assert_eq!(e2.min_nodes, 0);
    assert_eq!(e2.max_nodes, 10);
    assert_eq!(e2.lifecycle_state, "ready");
    assert_eq!(e2.pool_group.as_deref(), Some("lumen-data"));
}

#[test]
fn default_cr_resolves_e2_standard_2_through_catalog_with_10gi_pvc_and_cross_ns_anti_affinity() {
    let catalog: CapacityCatalog = CapacityCatalog::from_json(sample_catalog_json()).unwrap();
    let profile = resolve_machine_type("e2-standard-2", &catalog).expect("resolve e2-standard-2");
    let l = test_lumen("search", "default", "e2-standard-2");
    let objs = lumen::operator::render::render_with_profile(&l, &profile);
    assert!(!objs.is_empty(), "rendered objects should not be empty");

    let sts = objs
        .iter()
        .find(|o| o["kind"] == "StatefulSet" && o["metadata"]["name"] == "search")
        .expect("StatefulSet search");
    let pod = &sts["spec"]["template"]["spec"];

    assert_eq!(
        pod["nodeSelector"]["lumen.axiom.dev/capacity-profile"],
        "e2-standard-2"
    );
    assert_eq!(
        pod["tolerations"][0]["key"],
        "lumen.axiom.dev/capacity-profile"
    );
    assert_eq!(pod["tolerations"][0]["value"], "e2-standard-2");
    assert_eq!(pod["tolerations"][0]["effect"], "NoSchedule");

    // 10 GiB default PVC
    let vcts = sts["spec"]["volumeClaimTemplates"].as_array().unwrap();
    assert_eq!(vcts[0]["spec"]["resources"]["requests"]["storage"], "10Gi");

    // Cross-namespace anti-affinity
    let anti_affinity =
        &pod["affinity"]["podAntiAffinity"]["requiredDuringSchedulingIgnoredDuringExecution"][0];
    assert_eq!(anti_affinity["topologyKey"], "kubernetes.io/hostname");
    assert_eq!(
        anti_affinity["labelSelector"]["matchLabels"],
        json!({
            "app.kubernetes.io/name": "lumen",
            "app.kubernetes.io/component": "server",
        })
    );
    assert_eq!(anti_affinity["namespaceSelector"], json!({}));
}

#[test]
fn allowed_direct_machine_type_override_resolves_without_exposing_tier_or_pool_names() {
    let catalog: CapacityCatalog = CapacityCatalog::from_json(sample_catalog_json()).unwrap();
    let resolved = resolve_machine_type("n2-standard-4", &catalog).expect("resolve n2-standard-4");
    assert_eq!(resolved.selector_key, "lumen.axiom.dev/capacity-profile");
    assert_eq!(resolved.selector_value, "n2-standard-4");
    assert_eq!(
        resolved.selector,
        "lumen.axiom.dev/capacity-profile=n2-standard-4"
    );

    let l = test_lumen("search", "prod", "n2-standard-4");
    let objs = lumen::operator::render::render_with_profile(&l, &resolved);
    let sts = objs
        .iter()
        .find(|o| o["kind"] == "StatefulSet")
        .expect("StatefulSet");
    let pod = &sts["spec"]["template"]["spec"];
    assert_eq!(
        pod["nodeSelector"]["lumen.axiom.dev/capacity-profile"],
        "n2-standard-4"
    );
}

#[test]
fn three_namespaces_selecting_e2_standard_2_share_pool_selector_and_render_cross_namespace_anti_affinity() {
    let catalog: CapacityCatalog = CapacityCatalog::from_json(sample_catalog_json()).unwrap();
    let placements = vec![
        Placement {
            instance: "lumen-a".to_string(),
            namespace: "alpha".to_string(),
            node_name: "node-1".to_string(),
        },
        Placement {
            instance: "lumen-b".to_string(),
            namespace: "beta".to_string(),
            node_name: "node-2".to_string(),
        },
        Placement {
            instance: "lumen-c".to_string(),
            namespace: "gamma".to_string(),
            node_name: "node-3".to_string(),
        },
    ];

    let shared = resolve_shared_placement("e2-standard-2", &catalog, &placements)
        .expect("shared placement");
    assert_eq!(shared.selectors["lumen-a"], shared.selectors["lumen-b"]);
    assert_eq!(shared.selectors["lumen-b"], shared.selectors["lumen-c"]);

    let profile = resolve_machine_type("e2-standard-2", &catalog).expect("resolve e2-standard-2");
    for ns in ["alpha", "beta", "gamma"] {
        let l = test_lumen("search", ns, "e2-standard-2");
        let objs = lumen::operator::render::render_with_profile(&l, &profile);
        let sts = objs.iter().find(|o| o["kind"] == "StatefulSet").unwrap();
        let pod = &sts["spec"]["template"]["spec"];
        assert_eq!(
            pod["nodeSelector"]["lumen.axiom.dev/capacity-profile"],
            "e2-standard-2"
        );
        let anti = &pod["affinity"]["podAntiAffinity"]
            ["requiredDuringSchedulingIgnoredDuringExecution"][0];
        assert_eq!(anti["namespaceSelector"], json!({}));
    }
}

#[test]
fn unsupported_or_absent_machine_type_in_published_catalog_fails_to_resolve() {
    let catalog: CapacityCatalog = CapacityCatalog::from_json(sample_catalog_json()).unwrap();

    for absent_machine in ["m1-megamem-96", "c3-standard-4", "n1-standard-1", "a2-highgpu-1g"] {
        let err = resolve_machine_type(absent_machine, &catalog)
            .expect_err("absent machine type must not resolve");
        assert_eq!(err.reason, RejectionReason::UnsupportedMachineType);
        assert_eq!(err.field_path, "machine_type");

        let req = CapacityRequest {
            spec: CapacitySpec {
                initial_machine_type: absent_machine.to_string(),
            },
            old_member_disrupted: false,
        };
        let preflight_err = preflight_capacity(&req, Some(&catalog))
            .expect_err("absent machine type preflight must fail");
        assert_eq!(preflight_err.reason, RejectionReason::UnsupportedMachineType);
    }
}

#[test]
fn cross_namespace_duplicate_node_placement_fails_closed() {
    let catalog: CapacityCatalog = CapacityCatalog::from_json(sample_catalog_json()).unwrap();
    let conflicting_placements = vec![
        Placement {
            instance: "lumen-a".to_string(),
            namespace: "alpha".to_string(),
            node_name: "node-shared-1".to_string(),
        },
        Placement {
            instance: "lumen-b".to_string(),
            namespace: "beta".to_string(),
            node_name: "node-shared-1".to_string(),
        },
    ];

    let err = resolve_shared_placement("e2-standard-2", &catalog, &conflicting_placements)
        .expect_err("should reject duplicate node placement");
    assert_eq!(err.reason, RejectionReason::DataMemberNodeConflict);
    assert_eq!(err.field_path, "placements");
}

#[test]
fn zero_node_capacity_pool_resolves_successfully() {
    let catalog = CapacityCatalog::new(vec![CatalogEntry {
        machine_type: "e2-standard-2".to_string(),
        selector: "lumen.axiom.dev/capacity-profile=e2-standard-2".to_string(),
        stable_selector: StableSelector {
            key: "lumen.axiom.dev/capacity-profile".to_string(),
            value: "e2-standard-2".to_string(),
        },
        max_nodes: 10,
        min_nodes: 0,
        lifecycle_state: "ready".to_string(),
        pool_group: Some("lumen-data".to_string()),
    }]);

    let req = CapacityRequest {
        spec: CapacitySpec {
            initial_machine_type: "e2-standard-2".to_string(),
        },
        old_member_disrupted: false,
    };

    let resolved = preflight_capacity(&req, Some(&catalog)).expect("zero-node pool must resolve");
    assert_eq!(resolved.machine_type, "e2-standard-2");
    assert_eq!(resolved.min_nodes, 0);
    assert_eq!(resolved.max_nodes, 10);
}

#[test]
fn missing_ambiguous_draining_full_and_incompatible_catalogs_fail_closed() {
    let req = CapacityRequest {
        spec: CapacitySpec {
            initial_machine_type: "e2-standard-2".to_string(),
        },
        old_member_disrupted: false,
    };

    // Missing catalog
    let err_missing = preflight_capacity(&req, None).expect_err("missing catalog");
    assert_eq!(err_missing.reason, RejectionReason::CatalogMissing);
    assert_eq!(err_missing.field_path, "catalog");

    // Ambiguous catalog
    let ambiguous_catalog = CapacityCatalog::new(vec![
        CatalogEntry::new(
            "e2-standard-2",
            "lumen.axiom.dev/capacity-profile",
            "ready",
            10,
        ),
        CatalogEntry::new(
            "e2-standard-2",
            "lumen.axiom.dev/capacity-profile",
            "ready",
            5,
        ),
    ]);
    let err_ambiguous =
        preflight_capacity(&req, Some(&ambiguous_catalog)).expect_err("ambiguous catalog");
    assert_eq!(err_ambiguous.reason, RejectionReason::CatalogAmbiguous);
    assert_eq!(err_ambiguous.field_path, "catalog");

    // Draining catalog
    let draining_catalog = CapacityCatalog::new(vec![CatalogEntry::new(
        "e2-standard-2",
        "lumen.axiom.dev/capacity-profile",
        "draining",
        10,
    )]);
    let err_draining =
        preflight_capacity(&req, Some(&draining_catalog)).expect_err("draining catalog");
    assert_eq!(err_draining.reason, RejectionReason::CatalogDraining);
    assert_eq!(err_draining.field_path, "catalog");

    // Full catalog (max_nodes = 0 or current_nodes >= max_nodes)
    let full_catalog = CapacityCatalog::new(vec![CatalogEntry::new(
        "e2-standard-2",
        "lumen.axiom.dev/capacity-profile",
        "ready",
        0,
    )]);
    let err_full = preflight_capacity(&req, Some(&full_catalog)).expect_err("full catalog");
    assert_eq!(err_full.reason, RejectionReason::CapacityFull);
    assert_eq!(err_full.field_path, "catalog");

    // Full catalog verified with node count
    let ready_catalog = CapacityCatalog::new(vec![CatalogEntry::new(
        "e2-standard-2",
        "lumen.axiom.dev/capacity-profile",
        "ready",
        5,
    )]);
    let err_full_nodes = preflight_capacity_with_nodes(&req, Some(&ready_catalog), 5)
        .expect_err("full catalog at current max");
    assert_eq!(err_full_nodes.reason, RejectionReason::CapacityFull);

    // Incompatible catalog (empty selector key)
    let incompatible_catalog = CapacityCatalog::new(vec![CatalogEntry {
        machine_type: "e2-standard-2".to_string(),
        selector: "invalid".to_string(),
        stable_selector: StableSelector {
            key: "".to_string(),
            value: "".to_string(),
        },
        max_nodes: 10,
        min_nodes: 0,
        lifecycle_state: "ready".to_string(),
        pool_group: None,
    }]);
    let err_incompatible =
        preflight_capacity(&req, Some(&incompatible_catalog)).expect_err("incompatible catalog");
    assert_eq!(
        err_incompatible.reason,
        RejectionReason::CatalogIncompatible
    );
    assert_eq!(err_incompatible.field_path, "catalog");
}

#[test]
fn unchanged_reapplication_preserves_operator_owned_transition_state() {
    let previous = CapacityState {
        current_machine_type: "n2-standard-4".to_string(),
        target_machine_type: "n2-standard-8".to_string(),
        transition_generation: 17,
        phase: "Stable".to_string(),
        old_member_authoritative: true,
    };

    let spec = CapacitySpec {
        initial_machine_type: "e2-standard-2".to_string(),
    };

    let reapplied = apply_capacity_reapplication(&previous, &spec);
    assert_eq!(reapplied.current_machine_type, "n2-standard-4");
    assert_eq!(reapplied.target_machine_type, "n2-standard-8");
    assert_eq!(reapplied.transition_generation, 17);
    assert_eq!(reapplied.phase, "Stable");
    assert!(reapplied.old_member_authoritative);
}

#[test]
fn request_derivation_subtracts_reserves_and_headroom_below_node_allocatable() {
    let allocatable = CapacityVector {
        cpu_millicores: 4000,
        memory_mib: 16384,
    };
    let reserves = CapacityVector {
        cpu_millicores: 1000,
        memory_mib: 2048,
    };
    let headroom = CapacityVector {
        cpu_millicores: 1000,
        memory_mib: 2048,
    };

    let requests = derive_requests(allocatable, reserves, headroom).expect("derive requests");
    assert_eq!(requests.cpu_millicores, 2000);
    assert_eq!(requests.memory_mib, 12288);
}

#[test]
fn insufficient_allocatable_fails_closed() {
    let allocatable = CapacityVector {
        cpu_millicores: 1000,
        memory_mib: 1024,
    };
    let reserves = CapacityVector {
        cpu_millicores: 800,
        memory_mib: 800,
    };
    let headroom = CapacityVector {
        cpu_millicores: 300,
        memory_mib: 300,
    };

    let err =
        derive_requests(allocatable, reserves, headroom).expect_err("insufficient allocatable");
    assert_eq!(err.reason, RejectionReason::InsufficientAllocatable);
    assert_eq!(err.field_path, "allocatable");
}

#[test]
fn service_tier_names_are_rejected_at_admission() {
    for tier in ["lumen-premium", "bronze", "small", "tier-1", "large"] {
        let spec = CapacitySpec {
            initial_machine_type: tier.to_string(),
        };
        let err = decide_capacity_spec(&spec).expect_err("service tier should be rejected");
        assert_eq!(err.reason, RejectionReason::UnsupportedMachineType);
        assert_eq!(err.field_path, "initial_machine_type");
    }

    let direct = CapacitySpec {
        initial_machine_type: "e2-standard-2".to_string(),
    };
    assert!(decide_capacity_spec(&direct).is_ok());
}

#[test]
fn transition_policy_enforces_node_cap_and_cooldown() {
    let policy = CapacityPolicy {
        allowed_transitions: vec!["scale_out".to_string()],
        node_cap: 3,
        read_replica_cap: 2,
        shard_cap: 4,
        cooldown_seconds: 300,
    };

    let decision =
        decide_transition("e2-standard-2", "e2-standard-2", &policy, 3).expect("decide transition");
    assert_eq!(decision.cooldown_seconds, 300);
    assert_eq!(decision.node_cap, 3);
    assert_eq!(decision.read_replica_cap, 2);
    assert_eq!(decision.shard_cap, 4);

    let bounded =
        decide_transition("e2-standard-2", "e2-standard-2", &policy, 2).expect("decide transition");
    assert_eq!(
        bounded.node_cap, 2,
        "node cap must be bounded by catalog maximum"
    );
}

#[test]
fn storage_validation_accepts_valid_storage_and_rejects_empty() {
    let valid = CapacityStorage::default();
    assert_eq!(valid.size, "10Gi");
    assert_eq!(valid.storage_class, "standard-rwo");
    assert_eq!(valid.disk_type, "pd-balanced");
    assert!(decide_storage(&valid).is_ok());

    let mut invalid = valid;
    invalid.size = "".to_string();
    let err = decide_storage(&invalid).expect_err("empty storage size rejected");
    assert_eq!(err.field_path, "size");
}

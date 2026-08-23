#![cfg(feature = "operator")]

use lumen::operator::fleet::{plan, FleetInstance, LumenFleet, LumenFleetSpec, PlanOutcome};
use serde_json::json;

#[test]
fn retired_serving_autoscaling_override_is_rejected_at_the_fleet() {
    let fleet = LumenFleet::new(
        "search",
        LumenFleetSpec {
            defaults: serde_json::from_value(json!({
                "image": "img",
            }))
            .unwrap(),
            instances: vec![FleetInstance {
                namespace: "team-a".to_string(),
                name: Some("search".to_string()),
                spec: Some(json!({
                    "serving": {
                        "autoscaling": {
                            "minReplicas": 5,
                            "maxReplicas": 9,
                            "targetCpuUtilization": 55,
                        }
                    }
                })),
            }],
            prune_policy: Default::default(),
        },
    );
    let planned = plan(&fleet);
    assert_eq!(planned.len(), 1);
    match &planned[0].outcome {
        PlanOutcome::Rejected(reason) => {
            assert!(
                reason.contains("serving.autoscaling"),
                "reason must mention serving.autoscaling: {reason}"
            );
            assert!(
                reason.contains("a Lumen does not have"),
                "reason must explain that a Lumen does not have this field: {reason}"
            );
        }
        PlanOutcome::Ready(_) => {
            panic!("override with retired serving.autoscaling must be rejected, got Ready");
        }
    }
}

#[test]
fn retired_serving_autoscaling_individual_knobs_are_rejected_at_the_fleet() {
    for knob in [
        json!({ "serving": { "autoscaling": { "minReplicas": 3 } } }),
        json!({ "serving": { "autoscaling": { "maxReplicas": 10 } } }),
        json!({ "serving": { "autoscaling": { "targetCpuUtilization": 80 } } }),
    ] {
        let fleet = LumenFleet::new(
            "search",
            LumenFleetSpec {
                defaults: serde_json::from_value(json!({
                    "image": "img",
                }))
                .unwrap(),
                instances: vec![FleetInstance {
                    namespace: "team-a".to_string(),
                    name: Some("search".to_string()),
                    spec: Some(knob),
                }],
                prune_policy: Default::default(),
            },
        );
        let planned = plan(&fleet);
        assert_eq!(planned.len(), 1);
        match &planned[0].outcome {
            PlanOutcome::Rejected(reason) => {
                assert!(
                    reason.contains("serving.autoscaling"),
                    "reason must mention serving.autoscaling: {reason}"
                );
                assert!(
                    reason.contains("a Lumen does not have"),
                    "reason must explain that a Lumen does not have this field: {reason}"
                );
            }
            PlanOutcome::Ready(_) => {
                panic!("override with retired serving.autoscaling must be rejected, got Ready");
            }
        }
    }
}

#[test]
fn supported_serving_cpu_override_is_admitted_at_the_fleet() {
    let fleet = LumenFleet::new(
        "search",
        LumenFleetSpec {
            defaults: serde_json::from_value(json!({
                "image": "img",
            }))
            .unwrap(),
            instances: vec![FleetInstance {
                namespace: "team-a".to_string(),
                name: Some("search".to_string()),
                spec: Some(json!({
                    "serving": {
                        "cpu": "4",
                    }
                })),
            }],
            prune_policy: Default::default(),
        },
    );
    let planned = plan(&fleet);
    assert_eq!(planned.len(), 1);
    match &planned[0].outcome {
        PlanOutcome::Ready(spec) => {
            assert_eq!(spec["serving"]["cpu"], json!("4"));
        }
        PlanOutcome::Rejected(reason) => {
            panic!("override with supported serving.cpu must be admitted, got Rejected: {reason}");
        }
    }
}

#[test]
fn supported_serving_resources_override_is_admitted_at_the_fleet() {
    let fleet = LumenFleet::new(
        "search",
        LumenFleetSpec {
            defaults: serde_json::from_value(json!({
                "image": "img",
            }))
            .unwrap(),
            instances: vec![FleetInstance {
                namespace: "team-a".to_string(),
                name: Some("search".to_string()),
                spec: Some(json!({
                    "serving": {
                        "cpu": "2",
                        "memory": "8Gi",
                        "raftStorage": "50Gi",
                    }
                })),
            }],
            prune_policy: Default::default(),
        },
    );
    let planned = plan(&fleet);
    assert_eq!(planned.len(), 1);
    match &planned[0].outcome {
        PlanOutcome::Ready(spec) => {
            assert_eq!(spec["serving"]["cpu"], json!("2"));
            assert_eq!(spec["serving"]["memory"], json!("8Gi"));
            assert_eq!(spec["serving"]["raftStorage"], json!("50Gi"));
        }
        PlanOutcome::Rejected(reason) => {
            panic!("override with supported serving resources must be admitted, got Rejected: {reason}");
        }
    }
}

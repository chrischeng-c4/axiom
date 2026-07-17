// HANDWRITE-BEGIN gap="missing-generator:unit-test:9d8175b0" tracker="#1849" reason="Verify that Pgpool projects opaque plan capacity context after shared readiness while the existing operator and reconcile unit tests continue covering Deployment-only output and safe capacity holds."
use std::collections::HashMap;

use pgpool::k8s::{ControlPlaneStatus, EndpointControlStatus};
use pgpool::operator::{Pgpool, PgpoolEndpointBudgetSpec, PgpoolResources, PgpoolSpec};
use service_k8s::{ManagedService, ReadyFacts};

fn instance() -> Pgpool {
    let endpoint = PgpoolEndpointBudgetSpec {
        name: "primary".into(),
        configured_ceiling: Some(100),
        per_pod_quota: 40,
        ..PgpoolEndpointBudgetSpec::default()
    };
    let mut instance = Pgpool::new(
        "pool",
        PgpoolSpec {
            image: "pgpool:test".into(),
            replicas: 3,
            primary_endpoint: "primary".into(),
            endpoints: vec![endpoint],
            resources: PgpoolResources::default(),
            termination_grace_period_seconds: 60,
        },
    );
    instance.metadata.namespace = Some("database".into());
    instance.metadata.generation = Some(9);
    instance
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Verify CR status omits unsupported reserve counters and exposes unavailable reserve accounting.">
#[test]
fn context_aware_status_omits_unreconciled_reserve_counters() {
    let service = instance();
    let blocked = "endpoint primary scale blocked: requested=120, usable=80, held=80";
    let context = serde_json::to_value(ControlPlaneStatus {
        endpoints: vec![EndpointControlStatus {
            endpoint: "primary".into(),
            effective_limit: 100,
            reserve: 10,
            non_pgpool_usage: 0,
            safety_headroom: 10,
            usable: 80,
            allocated: 80,
            available: 0,
            reserve_granted: 0,
            reserve_available: 0,
            reserve_accounting_available: false,
            reserve_denials: 0,
            allocator_available: true,
            blocked_scale_reason: Some(blocked.into()),
        }],
        pods: vec![],
        blocked_scale_reason: Some(blocked.into()),
    })
    .expect("control-plane context serializes");
    let ready = ReadyFacts {
        ready: HashMap::from([("pool".into(), 2)]),
    };

    let patch = service.status_patch_with_context(&ready, &context);
    assert_eq!(patch["status"]["phase"], "Blocked");
    assert_eq!(patch["status"]["observedGeneration"], 9);
    assert_eq!(patch["status"]["readyReplicas"], 2);
    assert_eq!(patch["status"]["desiredReplicas"], 3);
    assert_eq!(patch["status"]["endpoints"][0]["usable"], 80);
    assert_eq!(patch["status"]["endpoints"][0]["allocated"], 80);
    assert_eq!(
        patch["status"]["endpoints"][0]["reserveAccountingAvailable"],
        false
    );
    assert!(patch["status"]["endpoints"][0]["reserveGranted"].is_null());
    assert!(patch["status"]["endpoints"][0]["reserveAvailable"].is_null());
    assert_eq!(patch["status"]["blockedScaleReason"], blocked);
}
// </HANDWRITE>
// HANDWRITE-END

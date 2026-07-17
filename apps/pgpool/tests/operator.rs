// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#unit-test
// <HANDWRITE gap="missing-generator:unit-test:4dbe8f81" tracker="#1575" reason="Verify CRD schema, stateless owned rendering, ManagedService readiness, and rich control-plane status projection.">
use std::collections::HashMap;

use kube::CustomResourceExt;
use pgpool::k8s::{
    BackendPoolObservation, EndpointAllocator, EndpointCapacity, GlobalConnectionBudget,
    PgpoolControlPlane, ReserveLeaseRequest,
};
use pgpool::operator::{
    self as pgpool_operator, Pgpool, PgpoolEndpointBudgetSpec, PgpoolEndpointProvider,
    PgpoolEndpointRole, PgpoolResources, PgpoolSpec, PgpoolStatus,
};
use service_k8s::{ManagedService, ReadyFacts};

fn spec() -> PgpoolSpec {
    PgpoolSpec {
        image: "pgpool:test".into(),
        replicas: 2,
        primary_endpoint: "alloy-primary".into(),
        endpoints: vec![
            PgpoolEndpointBudgetSpec {
                name: "alloy-primary".into(),
                provider: PgpoolEndpointProvider::AlloyDb,
                role: PgpoolEndpointRole::Primary,
                host: "primary.alloydb.example".into(),
                port: 5432,
                database: None,
                user: None,
                password_secret_ref: None,
                reserve: 20,
                safety_headroom: 10,
                configured_ceiling: Some(200),
                per_pod_quota: 40,
                ..PgpoolEndpointBudgetSpec::default()
            },
            PgpoolEndpointBudgetSpec {
                name: "alloy-read-pool".into(),
                provider: PgpoolEndpointProvider::AlloyDb,
                role: PgpoolEndpointRole::ReadPool,
                host: "read.alloydb.example".into(),
                port: 5432,
                database: None,
                user: None,
                password_secret_ref: None,
                reserve: 10,
                safety_headroom: 10,
                configured_ceiling: Some(300),
                per_pod_quota: 40,
                ..PgpoolEndpointBudgetSpec::default()
            },
        ],
        resources: PgpoolResources {
            cpu: "500m".into(),
            memory: "512Mi".into(),
        },
        termination_grace_period_seconds: 60,
    }
}

fn instance() -> Pgpool {
    let mut instance = Pgpool::new("pool", spec());
    instance.metadata.namespace = Some("database".into());
    instance.metadata.uid = Some("uid-123".into());
    instance.metadata.generation = Some(7);
    instance
}

#[test]
fn crd_is_namespaced_and_carries_endpoint_budget_status() {
    let value = serde_json::to_value(Pgpool::crd()).unwrap();
    assert_eq!(value["spec"]["scope"], "Namespaced");
    assert_eq!(value["spec"]["names"]["kind"], "Pgpool");
    let rendered = pgpool_operator::crd_yaml();
    for field in [
        "endpoints:",
        "provider:",
        "configuredCeiling:",
        "tlsCaSecretRef:",
        "effectiveLimit:",
        "blockedScaleReason:",
        "backendActive:",
        "drainRequested:",
    ] {
        assert!(rendered.contains(field), "missing CRD field {field}");
    }
    assert!(!rendered.contains("format: uint"));
}

#[test]
fn cr_renders_owned_stateless_shared_deployment() {
    let manifests = pgpool_operator::render::render(&instance());
    assert_eq!(manifests.len(), 4);
    assert_eq!(manifests[1]["kind"], "Deployment");
    assert_eq!(manifests[2]["kind"], "Service");
    assert_eq!(manifests[2]["spec"]["type"], "ClusterIP");
    assert_eq!(
        manifests[1]["spec"]["strategy"]["rollingUpdate"]["maxSurge"],
        0
    );
    assert_eq!(
        manifests[1]["spec"]["template"]["spec"]["containers"][0]["env"][2]["value"],
        "40"
    );
    for manifest in &manifests {
        assert_eq!(manifest["metadata"]["ownerReferences"][0]["uid"], "uid-123");
    }
    let rendered = serde_json::to_string(&manifests).unwrap();
    for forbidden in [
        "StatefulSet",
        "volumeClaimTemplates",
        "podManagementPolicy",
        "SHARD_COUNT",
        "REPLICAS_PER_SHARD",
        "sessionAffinity",
    ] {
        assert!(!rendered.contains(forbidden), "found {forbidden}");
    }
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="Migrate operator control-plane status coverage to the explicit endpoint-scoped Pod lifecycle API.">
#[test]
fn status_projects_global_budget_and_managed_readiness() {
    let mut budgets = GlobalConnectionBudget::default();
    budgets.insert(EndpointAllocator::new(
        "alloy-primary",
        EndpointCapacity {
            effective_limit: 180,
            reserve: 20,
            non_pgpool_usage: 30,
            safety_headroom: 10,
        },
    ));
    let mut control = PgpoolControlPlane::new(budgets);
    control
        .admit_scale(
            "alloy-primary",
            ["pool-a", "pool-b"],
            Vec::<&str>::new(),
            40,
        )
        .unwrap();
    control.mark_ready("alloy-primary", "pool-a").unwrap();
    control
        .observe_pool(
            "alloy-primary",
            "pool-a",
            BackendPoolObservation {
                active: 11,
                idle: 29,
            },
        )
        .unwrap();
    control
        .begin_drain("alloy-primary", "pool-a", 100, 60)
        .unwrap();

    let status = PgpoolStatus::from_control_plane(&spec(), 7, 1, &control.status());
    assert_eq!(status.phase, "Reconciling");
    assert_eq!(
        status.endpoints[0].provider,
        PgpoolEndpointProvider::AlloyDb
    );
    assert_eq!(status.endpoints[0].allocated, 80);
    assert_eq!(status.endpoints[0].available, 40);
    assert_eq!(status.pods[0].backend_active, 11);
    assert_eq!(status.pods[0].phase, "draining");

    let mut instance = instance();
    instance.status = Some(status);
    let patch = instance.status_patch(&ReadyFacts {
        ready: HashMap::from([("pool".into(), 2)]),
    });
    assert_eq!(patch["status"]["phase"], "Ready");
    assert_eq!(patch["status"]["readyReplicas"], 2);
    assert_eq!(patch["status"]["endpoints"][0]["allocated"], 80);
    assert_eq!(patch["status"]["pods"][0]["drainRequested"], true);
}
// </HANDWRITE>

#[test]
fn operator_assets_are_leader_elected_and_layered() {
    let manifests = pgpool_operator::operator_manifests("database-system");
    assert_eq!(manifests[0]["kind"], "Namespace");
    assert_eq!(manifests[4]["kind"], "Deployment");
    assert_eq!(manifests[4]["spec"]["replicas"], 2);
    assert_eq!(
        manifests[4]["spec"]["template"]["spec"]["containers"][0]["command"],
        serde_json::json!(["pgpool", "k8s", "operator", "run"])
    );
    let yaml = pgpool_operator::operator_yaml("database-system");
    assert!(yaml.contains("coordination.k8s.io"));
    assert!(yaml.contains("namespace: database-system"));
}

/// verify: operator::concurrent_pods_cannot_overgrant_reserve_capacity (R4)
#[test]
fn concurrent_pods_cannot_overgrant_reserve_capacity() {
    let mut budgets = GlobalConnectionBudget::default();
    budgets.insert(EndpointAllocator::new(
        "alloy-primary",
        EndpointCapacity {
            effective_limit: 100,
            reserve: 0,
            non_pgpool_usage: 0,
            safety_headroom: 0,
        },
    ));
    let mut control = PgpoolControlPlane::new(budgets);
    control
        .admit_scale(
            "alloy-primary",
            ["pool-a", "pool-b"],
            Vec::<&str>::new(),
            30,
        )
        .unwrap();
    control
        .grant_reserve(
            "alloy-primary",
            10,
            [ReserveLeaseRequest {
                pod: "pool-a".into(),
                token: "first".into(),
                units: 20,
                expires_at_epoch_seconds: 20,
            }],
        )
        .unwrap();
    let denied = control.grant_reserve(
        "alloy-primary",
        10,
        [
            ReserveLeaseRequest {
                pod: "pool-a".into(),
                token: "too-many-a".into(),
                units: 15,
                expires_at_epoch_seconds: 20,
            },
            ReserveLeaseRequest {
                pod: "pool-b".into(),
                token: "too-many-b".into(),
                units: 15,
                expires_at_epoch_seconds: 20,
            },
        ],
    );
    assert!(denied.is_err(), "the concurrent chunk must be atomic");
    let status = control.status();
    assert_eq!(status.endpoints[0].reserve_granted, 20);
    assert_eq!(status.endpoints[0].reserve_denials, 1);
    assert_eq!(
        control
            .reserve_ledger("alloy-primary")
            .unwrap()
            .held_total(),
        80,
        "base allocation plus every granted reserve unit remains capped"
    );
}
// </HANDWRITE>

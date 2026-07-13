// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#unit-test
// <HANDWRITE gap="missing-generator:unit-test:4dbe8f81" tracker="#1575" reason="Verify CRD schema, stateless owned rendering, ManagedService readiness, and rich control-plane status projection.">
use std::collections::HashMap;

use kube::CustomResourceExt;
use service_k8s::{ManagedService, ReadyFacts};
use pgpool::k8s::{
    BackendPoolObservation, EndpointAllocator, EndpointCapacity, GlobalConnectionBudget,
    PgpoolControlPlane,
};
use pgpool::operator::{
    self as pgpool_operator, Pgpool, PgpoolEndpointBudgetSpec, PgpoolEndpointProvider,
    PgpoolEndpointRole, PgpoolResources, PgpoolSpec, PgpoolStatus,
};

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
                reserve: 20,
                safety_headroom: 10,
                configured_ceiling: Some(200),
                per_pod_quota: 40,
            },
            PgpoolEndpointBudgetSpec {
                name: "alloy-read-pool".into(),
                provider: PgpoolEndpointProvider::AlloyDb,
                role: PgpoolEndpointRole::ReadPool,
                host: "read.alloydb.example".into(),
                port: 5432,
                reserve: 10,
                safety_headroom: 10,
                configured_ceiling: Some(300),
                per_pod_quota: 40,
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
    control.mark_ready("pool-a").unwrap();
    control
        .observe_pool(
            "pool-a",
            BackendPoolObservation {
                active: 11,
                idle: 29,
            },
        )
        .unwrap();
    control.begin_drain("pool-a", 100, 60).unwrap();

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
// </HANDWRITE>

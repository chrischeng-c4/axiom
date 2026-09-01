use std::collections::BTreeMap;

use serde_json::json;
use service_k8s::render::{
    ClusterRoleBindingPlan, ContainerPlan, DeploymentPlan, FqdnMatchPlan, FqdnNetworkPolicyPlan,
    NetworkPeerPlan, NetworkPolicyPlan, NetworkPortPlan, NetworkRulePlan, PodPlan,
    PodRuntimePolicy, RenderCtx, ServiceAccountSubjectPlan, ServicePlan, ServicePortPlan,
    WorkloadPlan,
};

fn context() -> RenderCtx<'static> {
    RenderCtx {
        app: "demo",
        manager: "demo-operator",
        api_version: "demo.axiom.dev/v1",
        kind: "Demo",
        name: "sample",
        ns: "observability",
        owner: None,
    }
}

#[test]
fn one_typed_plan_renders_service_deployment_and_default_deny_network_policy() {
    let cx = context();
    let selector = BTreeMap::from([("demo.axiom.dev/role".to_string(), "query".to_string())]);
    let runtime = PodRuntimePolicy::restricted("sample", json!({}));
    let pod = PodPlan::new(
        "query",
        ContainerPlan::new("demo", "demo:1", vec!["serve".into()]),
        runtime,
    )
    .with_selector_labels(selector.clone());

    let mut plan = WorkloadPlan::new(&cx);
    plan.add_service(ServicePlan::cluster_ip(
        "sample-query",
        "query",
        selector.clone(),
        vec![ServicePortPlan::tcp("http", 7380, "http")],
    ));
    plan.add_deployment(DeploymentPlan::new("sample-query", 1, pod));
    plan.add_network_policy(
        NetworkPolicyPlan::new("sample-query", "query", selector).with_ingress(
            NetworkRulePlan::new(
                vec![NetworkPeerPlan::same_namespace()],
                vec![NetworkPortPlan::tcp(7380)],
            ),
        ),
    );

    let objects = plan.render().expect("render typed workload plan");
    assert_eq!(objects[0]["kind"], "Service");
    assert_eq!(objects[1]["kind"], "Deployment");
    assert_eq!(
        objects[1]["spec"]["template"]["spec"]["securityContext"]["fsGroupChangePolicy"],
        "OnRootMismatch"
    );
    assert_eq!(objects[2]["kind"], "NetworkPolicy");
    assert_eq!(
        objects[2]["spec"]["policyTypes"],
        json!(["Ingress", "Egress"])
    );
    assert_eq!(objects[2]["spec"]["egress"], json!([]));
}

#[test]
fn typed_plan_renders_a_cluster_scoped_binding_without_a_namespaced_owner() {
    let cx = context();
    let mut plan = WorkloadPlan::new(&cx);
    plan.add_cluster_role_binding(
        ClusterRoleBindingPlan::new(
            "demo.observability.sample.auth-delegator",
            "auth-delegation",
            "system:auth-delegator",
        )
        .with_service_account(ServiceAccountSubjectPlan::new("observability", "sample"))
        .with_label("demo.axiom.dev/owner-namespace", "observability"),
    );

    let objects = plan.render().expect("render typed workload plan");
    assert_eq!(objects.len(), 1);
    let binding = &objects[0];
    assert_eq!(binding["kind"], "ClusterRoleBinding");
    assert!(binding["metadata"].get("namespace").is_none());
    assert!(binding["metadata"].get("ownerReferences").is_none());
    assert_eq!(
        binding["metadata"]["labels"]["demo.axiom.dev/owner-namespace"],
        "observability"
    );
    assert_eq!(binding["roleRef"]["name"], "system:auth-delegator");
    assert_eq!(
        binding["subjects"],
        json!([{
            "kind": "ServiceAccount",
            "name": "sample",
            "namespace": "observability"
        }])
    );
}

#[test]
fn typed_plan_limits_external_egress_by_ip_and_gke_fqdn() {
    let cx = context();
    let selector = BTreeMap::from([("demo.axiom.dev/role".to_string(), "store".to_string())]);
    let mut plan = WorkloadPlan::new(&cx);
    plan.add_network_policy(
        NetworkPolicyPlan::new("sample-store", "store", selector.clone()).with_egress(
            NetworkRulePlan::new(
                vec![NetworkPeerPlan::ip_block("169.254.169.254/32")],
                vec![NetworkPortPlan::tcp(80)],
            ),
        ),
    );
    plan.add_fqdn_network_policy(
        FqdnNetworkPolicyPlan::new("sample-store-google-apis", "store", selector)
            .with_match(FqdnMatchPlan::name("storage.googleapis.com"))
            .with_port(NetworkPortPlan::tcp(443)),
    );

    let objects = plan.render().expect("render typed workload plan");
    assert_eq!(
        objects[0]["spec"]["egress"][0]["to"][0]["ipBlock"]["cidr"],
        "169.254.169.254/32"
    );
    assert_eq!(objects[1]["apiVersion"], "networking.gke.io/v1alpha1");
    assert_eq!(objects[1]["kind"], "FQDNNetworkPolicy");
    assert_eq!(
        objects[1]["spec"]["egress"][0]["matches"],
        json!([{"name":"storage.googleapis.com"}])
    );
    assert_eq!(
        objects[1]["spec"]["egress"][0]["ports"],
        json!([{"protocol":"TCP", "port":443}])
    );
}

#[test]
fn any_network_peer_omits_the_direction_selector() {
    let cx = context();
    let selector = BTreeMap::from([("demo.axiom.dev/role".to_string(), "gateway".to_string())]);
    let mut plan = WorkloadPlan::new(&cx);
    plan.add_network_policy(
        NetworkPolicyPlan::new("sample-gateway", "gateway", selector).with_ingress(
            NetworkRulePlan::new(
                vec![NetworkPeerPlan::any()],
                vec![NetworkPortPlan::tcp(7380)],
            ),
        ),
    );

    let objects = plan.render().expect("render typed workload plan");
    let ingress = &objects[0]["spec"]["ingress"][0];
    assert!(
        ingress.get("from").is_none(),
        "an unrestricted peer is expressed by omitting `from`, not by an invalid empty peer"
    );
    assert_eq!(ingress["ports"], json!([{"protocol":"TCP", "port":7380}]));
}

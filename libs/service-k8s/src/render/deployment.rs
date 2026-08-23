// HANDWRITE-BEGIN gap="missing-generator:logic:54e8bce5" tracker="#1849" reason="Own ServiceDeployment and service_deployment with replicas and caller-supplied rollout fields while emitting no stateful or sticky-session contract."
//! Stateless `apps/v1` Deployment workload rendering.
//!
//! Half of this module's contract is what it does not emit (#1849).
//! [`service_deployment`] renders an ordinary `apps/v1` Deployment and carries no
//! stateful or sticky-session surface: no `serviceName`, no
//! `volumeClaimTemplates`, no `podManagementPolicy`, no `SHARD_COUNT` /
//! `REPLICAS_PER_SHARD` / `VOTER_COUNT` environment, and no `sessionAffinity`.
//! A caller that needs any of those wants the StatefulSet helpers in [`super`];
//! adding one field here would hand identity-bearing behaviour to every stateless
//! adopter at once, which is the opposite of why this shape was split out.
//!
//! That exclusion is enforced, not merely intended:
//! `deployment_has_no_stateful_or_sticky_session_contract` in this file serializes
//! the rendered object and asserts each of those names is absent, and
//! `ordinary_children_are_cluster_ip_and_non_sticky` in [`super::common`] does the
//! same for the companion Service. Extending the field set means extending that
//! list, so a new stateful field cannot arrive quietly.

use serde_json::{json, Value};

use super::common::ServicePodTemplate;

/// Deployment-owned rollout fields composed with a workload-neutral Pod
/// template.
pub struct ServiceDeployment<'a> {
    pub name: &'a str,
    pub replicas: u32,
    pub min_ready_seconds: Option<u32>,
    pub revision_history_limit: Option<i32>,
    pub strategy: Option<Value>,
    pub pod: ServicePodTemplate<'a>,
}

/// Render an ordinary apps/v1 Deployment without stateful-only fields.
pub fn service_deployment(p: ServiceDeployment<'_>) -> Value {
    let cx = p.pod.cx;
    let component = p.pod.component;
    let mut spec = json!({
        "replicas": p.replicas,
        "selector": { "matchLabels": cx.selector(component) },
        "template": p.pod.render(),
    });
    if let Some(seconds) = p.min_ready_seconds {
        spec["minReadySeconds"] = json!(seconds);
    }
    if let Some(limit) = p.revision_history_limit {
        spec["revisionHistoryLimit"] = json!(limit);
    }
    if let Some(strategy) = p.strategy {
        spec["strategy"] = strategy;
    }
    json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": cx.meta(p.name, component),
        "spec": spec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::common::{client_service, guaranteed_resources, RenderCtx};

    fn cx() -> RenderCtx<'static> {
        RenderCtx {
            app: "pgpool",
            manager: "pgpool-operator",
            api_version: "pgpool.axiom.dev/v1alpha1",
            kind: "Pgpool",
            name: "pool",
            ns: "database",
            owner: None,
        }
    }

    fn pod<'a>(cx: &'a RenderCtx<'a>) -> ServicePodTemplate<'a> {
        ServicePodTemplate {
            cx,
            component: "pool",
            image: "pgpool:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["pgpool".into()],
            args: vec!["serve".into()],
            ports: vec![json!({ "name": "postgres", "containerPort": 6432 })],
            env: vec![json!({ "name": "DB_HOST", "value": "remote-db" })],
            env_from: vec![],
            resources: guaranteed_resources("500m", "512Mi"),
            readiness_probe: Some(json!({ "tcpSocket": { "port": "postgres" } })),
            liveness_probe: None,
            startup_probe: None,
            lifecycle: Some(
                json!({ "preStop": { "httpGet": { "path": "/drain", "port": 9080 } } }),
            ),
            container_security_context: None,
            pod_security_context: None,
            service_account_name: Some("pool"),
            termination_grace_period_seconds: Some(60),
            volumes: vec![],
            volume_mounts: vec![],
            pod_annotations: None,
            topology_spread_constraints: vec![],
        }
    }

    #[test]
    fn deployment_composes_common_pod_and_caller_rollout_fields() {
        let cx = cx();
        let deployment = service_deployment(ServiceDeployment {
            name: "pool",
            replicas: 3,
            min_ready_seconds: Some(10),
            revision_history_limit: Some(5),
            strategy: Some(json!({
                "type": "RollingUpdate",
                "rollingUpdate": { "maxUnavailable": 1, "maxSurge": 0 },
            })),
            pod: pod(&cx),
        });
        assert_eq!(deployment["kind"], "Deployment");
        assert_eq!(deployment["spec"]["replicas"], 3);
        assert_eq!(deployment["spec"]["minReadySeconds"], 10);
        assert_eq!(deployment["spec"]["revisionHistoryLimit"], 5);
        assert_eq!(
            deployment["spec"]["strategy"]["rollingUpdate"]["maxSurge"],
            0
        );
        assert_eq!(
            deployment["spec"]["template"]["spec"]["containers"][0]["lifecycle"]["preStop"]
                ["httpGet"]["path"],
            "/drain"
        );

        let service = client_service(&cx, "pool", "pool", 6432);
        assert_eq!(service["kind"], "Service");
        assert!(service["spec"]["sessionAffinity"].is_null());
    }

    #[test]
    fn deployment_has_no_stateful_or_sticky_session_contract() {
        let cx = cx();
        let deployment = service_deployment(ServiceDeployment {
            name: "pool",
            replicas: 1,
            min_ready_seconds: None,
            revision_history_limit: None,
            strategy: None,
            pod: pod(&cx),
        });
        let rendered = serde_json::to_string(&deployment).unwrap();
        for forbidden in [
            "StatefulSet",
            "serviceName",
            "volumeClaimTemplates",
            "podManagementPolicy",
            "SHARD_COUNT",
            "REPLICAS_PER_SHARD",
            "VOTER_COUNT",
            "sessionAffinity",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "found forbidden field {forbidden}"
            );
        }
    }
}
// HANDWRITE-END

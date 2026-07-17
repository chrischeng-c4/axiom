// HANDWRITE-BEGIN gap="missing-generator:logic:bb5e0fdb" tracker="#1849" reason="Own RenderCtx, ServicePodTemplate, labels, owner references, resources, ServiceAccount, ordinary ClusterIP Service, PDB, HPA, and CronJob composition independent of workload kind."
//! Workload-neutral Pod templates and ordinary Kubernetes child helpers.
//!
//! The helpers are re-exported from the existing render root during the first
//! service-k8s landing so StatefulSet consumers keep their source-compatible
//! imports. New workload profiles compose through this semantic module.

use serde_json::{json, Value};

pub use super::{
    client_service, client_service_with_ports, cron_job, guaranteed_resources,
    horizontal_pod_autoscaler, owner_ref, pdb, requested_resources, service_account, CronJob,
    HorizontalPodAutoscaler, RenderCtx,
};

/// Workload-neutral Pod contract used by the Deployment profile and available
/// to other workload renderers. It deliberately contains no stable identity,
/// PVC, shard, ordinal, peer, or session-affinity fields.
pub struct ServicePodTemplate<'a> {
    pub cx: &'a RenderCtx<'a>,
    pub component: &'a str,
    pub image: &'a str,
    pub image_pull_policy: &'a str,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub ports: Vec<Value>,
    pub env: Vec<Value>,
    pub env_from: Vec<Value>,
    pub resources: Value,
    pub readiness_probe: Option<Value>,
    pub liveness_probe: Option<Value>,
    pub startup_probe: Option<Value>,
    pub lifecycle: Option<Value>,
    pub container_security_context: Option<Value>,
    pub pod_security_context: Option<Value>,
    pub service_account_name: Option<&'a str>,
    pub termination_grace_period_seconds: Option<u64>,
    pub volumes: Vec<Value>,
    pub volume_mounts: Vec<Value>,
    pub pod_annotations: Option<Value>,
    pub topology_spread_constraints: Vec<Value>,
}

impl ServicePodTemplate<'_> {
    /// Render the `spec.template` value shared by workload controllers.
    pub fn render(self) -> Value {
        let mut container = json!({
            "name": self.component,
            "image": self.image,
            "imagePullPolicy": self.image_pull_policy,
            "command": self.command,
            "ports": self.ports,
            "env": self.env,
            "resources": self.resources,
        });
        if !self.args.is_empty() {
            container["args"] = json!(self.args);
        }
        if !self.env_from.is_empty() {
            container["envFrom"] = json!(self.env_from);
        }
        if let Some(probe) = self.readiness_probe {
            container["readinessProbe"] = probe;
        }
        if let Some(probe) = self.liveness_probe {
            container["livenessProbe"] = probe;
        }
        if let Some(probe) = self.startup_probe {
            container["startupProbe"] = probe;
        }
        if let Some(lifecycle) = self.lifecycle {
            container["lifecycle"] = lifecycle;
        }
        if let Some(context) = self.container_security_context {
            container["securityContext"] = context;
        }
        if !self.volume_mounts.is_empty() {
            container["volumeMounts"] = json!(self.volume_mounts);
        }

        let mut pod_spec = json!({ "containers": [container] });
        if let Some(name) = self.service_account_name {
            pod_spec["serviceAccountName"] = json!(name);
        }
        if let Some(seconds) = self.termination_grace_period_seconds {
            pod_spec["terminationGracePeriodSeconds"] = json!(seconds);
        }
        if let Some(context) = self.pod_security_context {
            pod_spec["securityContext"] = context;
        }
        if !self.volumes.is_empty() {
            pod_spec["volumes"] = json!(self.volumes);
        }
        if !self.topology_spread_constraints.is_empty() {
            pod_spec["topologySpreadConstraints"] = json!(self.topology_spread_constraints);
        }

        let mut metadata = json!({ "labels": self.cx.labels(self.component) });
        if let Some(annotations) = self.pod_annotations {
            metadata["annotations"] = annotations;
        }
        json!({ "metadata": metadata, "spec": pod_spec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn pod_template_preserves_runtime_and_drain_fields() {
        let cx = cx();
        let template = ServicePodTemplate {
            cx: &cx,
            component: "pool",
            image: "pgpool:1",
            image_pull_policy: "IfNotPresent",
            command: vec!["pgpool".into()],
            args: vec!["serve".into()],
            ports: vec![json!({ "name": "postgres", "containerPort": 6432 })],
            env: vec![json!({ "name": "DB_HOST", "value": "remote-db" })],
            env_from: vec![json!({ "secretRef": { "name": "database" } })],
            resources: guaranteed_resources("500m", "512Mi"),
            readiness_probe: Some(json!({ "tcpSocket": { "port": "postgres" } })),
            liveness_probe: Some(json!({ "httpGet": { "path": "/healthz", "port": 9080 } })),
            startup_probe: Some(json!({ "httpGet": { "path": "/healthz", "port": 9080 } })),
            lifecycle: Some(
                json!({ "preStop": { "httpGet": { "path": "/drain", "port": 9080 } } }),
            ),
            container_security_context: Some(json!({ "runAsNonRoot": true })),
            pod_security_context: Some(json!({ "seccompProfile": { "type": "RuntimeDefault" } })),
            service_account_name: Some("pool"),
            termination_grace_period_seconds: Some(60),
            volumes: vec![json!({ "name": "tmp", "emptyDir": {} })],
            volume_mounts: vec![json!({ "name": "tmp", "mountPath": "/tmp" })],
            pod_annotations: Some(json!({ "prometheus.io/scrape": "true" })),
            topology_spread_constraints: vec![json!({
                "maxSkew": 1,
                "topologyKey": "kubernetes.io/hostname",
                "whenUnsatisfiable": "ScheduleAnyway",
                "labelSelector": { "matchLabels": cx.selector("pool") },
            })],
        }
        .render();

        let pod = &template["spec"];
        let container = &pod["containers"][0];
        for key in [
            "command",
            "args",
            "ports",
            "env",
            "envFrom",
            "resources",
            "readinessProbe",
            "livenessProbe",
            "startupProbe",
            "lifecycle",
            "securityContext",
            "volumeMounts",
        ] {
            assert!(!container[key].is_null(), "missing container field {key}");
        }
        for key in [
            "serviceAccountName",
            "terminationGracePeriodSeconds",
            "securityContext",
            "volumes",
            "topologySpreadConstraints",
        ] {
            assert!(!pod[key].is_null(), "missing pod field {key}");
        }
        assert_eq!(
            template["metadata"]["annotations"]["prometheus.io/scrape"],
            "true"
        );
    }

    #[test]
    fn ordinary_children_are_cluster_ip_and_non_sticky() {
        let cx = cx();
        let service = client_service(&cx, "pool", "pool", 6432);
        assert_eq!(service["spec"]["type"], "ClusterIP");
        assert!(service["spec"]["sessionAffinity"].is_null());
        assert_eq!(service_account(&cx, "pool")["kind"], "ServiceAccount");
        assert_eq!(pdb(&cx, "pool", "pool", 1)["spec"]["maxUnavailable"], 1);
    }
}
// HANDWRITE-END

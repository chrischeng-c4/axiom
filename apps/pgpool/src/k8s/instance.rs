// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-stateless-deployment-instance.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-k8s-instance" tracker="#1561" reason="Shared Deployment composition needs a typed Rust generator primitive.">
use service_k8s::render::common::{
    client_service_with_ports, guaranteed_resources, pdb, service_account, RenderCtx,
    ServicePodTemplate,
};
use service_k8s::render::deployment::{service_deployment, ServiceDeployment};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstanceProfile {
    Dev,
    Staging,
    #[default]
    Prod,
    Template,
}

/// Stateless Pgpool instance inputs. Remote PostgreSQL owns all durable data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgpoolInstanceSpec {
    pub name: String,
    pub namespace: String,
    pub image: String,
    pub replicas: u32,
    pub backend_host: String,
    pub backend_port: u16,
    pub max_backend_connections: u32,
    pub cpu: String,
    pub memory: String,
    pub termination_grace_period_seconds: u64,
}

pub fn spec_for_profile(profile: InstanceProfile) -> PgpoolInstanceSpec {
    match profile {
        InstanceProfile::Dev => PgpoolInstanceSpec {
            name: "pgpool".into(),
            namespace: "default".into(),
            image: "pgpool:dev".into(),
            replicas: 1,
            backend_host: "postgres.default.svc".into(),
            backend_port: 5432,
            max_backend_connections: 32,
            cpu: "100m".into(),
            memory: "128Mi".into(),
            termination_grace_period_seconds: 30,
        },
        InstanceProfile::Staging => PgpoolInstanceSpec {
            name: "pgpool".into(),
            namespace: "database".into(),
            image: "ghcr.io/chrischeng-c4/pgpool:latest".into(),
            replicas: 2,
            backend_host: "postgres.database.example".into(),
            backend_port: 5432,
            max_backend_connections: 64,
            cpu: "250m".into(),
            memory: "256Mi".into(),
            termination_grace_period_seconds: 60,
        },
        InstanceProfile::Prod => PgpoolInstanceSpec {
            name: "pgpool".into(),
            namespace: "database".into(),
            image: "ghcr.io/chrischeng-c4/pgpool:latest".into(),
            replicas: 3,
            backend_host: "postgres.database.example".into(),
            backend_port: 5432,
            max_backend_connections: 128,
            cpu: "500m".into(),
            memory: "512Mi".into(),
            termination_grace_period_seconds: 90,
        },
        InstanceProfile::Template => PgpoolInstanceSpec {
            name: "pgpool".into(),
            namespace: "<namespace>".into(),
            image: "<registry>/pgpool:<version>".into(),
            replicas: 1,
            backend_host: "<remote-postgresql-host>".into(),
            backend_port: 5432,
            max_backend_connections: 32,
            cpu: "250m".into(),
            memory: "256Mi".into(),
            termination_grace_period_seconds: 60,
        },
    }
}

/// Render ServiceAccount, Deployment, client Service, and PDB in apply order.
pub fn render_manifests(spec: &PgpoolInstanceSpec) -> Vec<Value> {
    let cx = RenderCtx {
        app: "pgpool",
        manager: "pgpool-operator",
        api_version: "pgpool.axiom.dev/v1alpha1",
        kind: "Pgpool",
        name: &spec.name,
        ns: &spec.namespace,
        owner: None,
    };

    let component = "pool";
    let deployment = service_deployment(ServiceDeployment {
        name: &spec.name,
        replicas: spec.replicas,
        min_ready_seconds: Some(10),
        revision_history_limit: Some(5),
        strategy: Some(json!({
            "type": "RollingUpdate",
            "rollingUpdate": { "maxUnavailable": 1, "maxSurge": 0 },
        })),
        pod: ServicePodTemplate {
            cx: &cx,
            component,
            image: &spec.image,
            image_pull_policy: "IfNotPresent",
            command: vec!["pgpool".into()],
            args: vec!["serve".into()],
            ports: vec![
                json!({ "name": "postgres", "containerPort": 6432, "protocol": "TCP" }),
                json!({ "name": "admin", "containerPort": 9080, "protocol": "TCP" }),
            ],
            env: vec![
                json!({ "name": "PGPOOL_BACKEND_HOST", "value": spec.backend_host }),
                json!({ "name": "PGPOOL_BACKEND_PORT", "value": spec.backend_port.to_string() }),
                json!({ "name": "PGPOOL_MAX_BACKEND_CONNECTIONS", "value": spec.max_backend_connections.to_string() }),
                json!({ "name": "PGPOOL_DRAIN_TIMEOUT_MS", "value": (spec.termination_grace_period_seconds * 1000).to_string() }),
            ],
            env_from: vec![],
            resources: guaranteed_resources(&spec.cpu, &spec.memory),
            readiness_probe: Some(json!({
                "httpGet": { "path": "/readyz", "port": "admin" },
                "periodSeconds": 5,
                "timeoutSeconds": 3,
                "failureThreshold": 2,
            })),
            liveness_probe: Some(json!({
                "httpGet": { "path": "/healthz", "port": "admin" },
                "periodSeconds": 10,
                "timeoutSeconds": 3,
                "failureThreshold": 3,
            })),
            startup_probe: Some(json!({
                "httpGet": { "path": "/healthz", "port": "admin" },
                "periodSeconds": 2,
                "failureThreshold": 30,
            })),
            lifecycle: Some(json!({
                "preStop": { "httpGet": { "path": "/drain", "port": "admin", "scheme": "HTTP" } },
            })),
            container_security_context: Some(json!({
                "runAsNonRoot": true,
                "runAsUser": 65532,
                "runAsGroup": 65532,
                "allowPrivilegeEscalation": false,
                "readOnlyRootFilesystem": true,
                "capabilities": { "drop": ["ALL"] },
            })),
            pod_security_context: Some(json!({
                "runAsNonRoot": true,
                "runAsUser": 65532,
                "runAsGroup": 65532,
                "seccompProfile": { "type": "RuntimeDefault" },
            })),
            service_account_name: Some(&spec.name),
            termination_grace_period_seconds: Some(spec.termination_grace_period_seconds),
            volumes: vec![json!({ "name": "tmp", "emptyDir": {} })],
            volume_mounts: vec![json!({ "name": "tmp", "mountPath": "/tmp" })],
            pod_annotations: Some(json!({
                "prometheus.io/scrape": "true",
                "prometheus.io/port": "9080",
                "prometheus.io/path": "/metrics",
            })),
            topology_spread_constraints: vec![json!({
                "maxSkew": 1,
                "topologyKey": "kubernetes.io/hostname",
                "whenUnsatisfiable": "ScheduleAnyway",
                "labelSelector": { "matchLabels": cx.selector(component) },
            })],
        },
    });

    let service = client_service_with_ports(
        &cx,
        &spec.name,
        component,
        vec![json!({
            "name": "postgres",
            "port": 6432,
            "targetPort": "postgres",
            "protocol": "TCP",
        })],
    );

    vec![
        service_account(&cx, component),
        deployment,
        service,
        pdb(&cx, &spec.name, component, 1),
    ]
}

pub fn render_instance_yaml(spec: &PgpoolInstanceSpec) -> String {
    render_manifests(spec)
        .into_iter()
        .map(|manifest| serde_yaml::to_string(&manifest).expect("manifest serializes"))
        .collect::<Vec<_>>()
        .join("---\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prod_instance_is_stateless_shared_deployment() {
        let manifests = render_manifests(&spec_for_profile(InstanceProfile::Prod));
        assert_eq!(manifests[1]["kind"], "Deployment");
        assert_eq!(manifests[2]["kind"], "Service");
        assert_eq!(manifests[2]["spec"]["type"], "ClusterIP");
        assert!(manifests[2]["spec"]["sessionAffinity"].is_null());
        assert_eq!(
            manifests[1]["spec"]["strategy"]["rollingUpdate"]["maxSurge"],
            0
        );
        assert_eq!(
            manifests[1]["spec"]["template"]["spec"]["containers"][0]["lifecycle"]["preStop"]
                ["httpGet"]["path"],
            "/drain"
        );
    }

    #[test]
    fn instance_output_has_no_stateful_contract() {
        let yaml = render_instance_yaml(&spec_for_profile(InstanceProfile::Prod));
        for forbidden in [
            "StatefulSet",
            "serviceName:",
            "volumeClaimTemplates",
            "podManagementPolicy",
            "SHARD_COUNT",
            "REPLICAS_PER_SHARD",
            "VOTER_COUNT",
            "sessionAffinity: ClientIP",
        ] {
            assert!(
                !yaml.contains(forbidden),
                "found forbidden token {forbidden}"
            );
        }
    }

    #[test]
    fn profile_controls_replica_and_pool_quota() {
        let dev = spec_for_profile(InstanceProfile::Dev);
        let prod = spec_for_profile(InstanceProfile::Prod);
        assert_eq!(dev.replicas, 1);
        assert_eq!(prod.replicas, 3);
        assert!(prod.max_backend_connections > dev.max_backend_connections);
    }
}
// </HANDWRITE>

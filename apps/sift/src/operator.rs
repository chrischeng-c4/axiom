// HANDWRITE-BEGIN gap="sift-shared-operator-controller" tracker="1606" reason="Define the Sift custom-resource type and compose the shared leader-elected operator reconcile loop."
//! Sift's small service-specific adapter over the shared operator framework.

use k8s_openapi::api::core::v1::Endpoints;
use kube::{Api, CustomResource, ResourceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use service_k8s::render::{self, RenderCtx};
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "sift.axiom.dev",
    version = "v1alpha1",
    kind = "Sift",
    plural = "sifts",
    namespaced,
    status = "SiftStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct SiftSpec {
    pub image: String,
    /// Secret with tls.crt, tls.key, and ca.crt for the dedicated Raft port.
    pub peer_tls_secret: String,
    #[serde(default = "three")]
    pub replicas_per_shard: u32,
    #[serde(default = "three")]
    pub voter_count: u32,
    /// Deprecated single-size compatibility field. New resources use
    /// `storage`, which gives each stateful role its own bounded PVC.
    #[serde(default)]
    pub data_size: Option<String>,
    #[serde(default)]
    pub archive: Option<ArchiveSpec>,
    #[serde(default)]
    pub bootstrap: BootstrapSpec,
    #[serde(default)]
    pub storage: StorageSpec,
    #[serde(default)]
    pub ingest: IngestSpec,
    #[serde(default)]
    pub placement: PlacementSpec,
    #[serde(default)]
    pub auth: AuthMode,
    #[serde(default)]
    pub tokens_secret: Option<String>,
    #[serde(default)]
    pub backup: Option<BackupSpec>,
    #[serde(default)]
    pub gcp_project_id: String,
    #[serde(default)]
    pub gke_cluster_name: String,
    #[serde(default)]
    pub gke_location: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    #[default]
    Off,
    Required,
    Kubernetes,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackupSpec {
    pub schedule: String,
    pub destination: String,
    #[serde(default)]
    pub retention_secs: Option<u64>,
    #[serde(default)]
    pub admin_token_secret: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSpec {
    pub destination: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct BootstrapSpec {
    pub archive_manifest_uri: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct StorageSpec {
    pub store_size: String,
    pub control_size: String,
    pub gateway_size: String,
    pub query_size: String,
}

impl Default for StorageSpec {
    fn default() -> Self {
        Self {
            store_size: "50Gi".to_string(),
            control_size: "5Gi".to_string(),
            gateway_size: "2Gi".to_string(),
            query_size: "2Gi".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct IngestSpec {
    pub max_items_per_minute: u64,
    pub max_concurrent_requests: u32,
}

impl Default for IngestSpec {
    fn default() -> Self {
        Self {
            max_items_per_minute: 720_000,
            max_concurrent_requests: 32,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct PlacementSpec {
    pub node_selector: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SiftStatus {
    pub phase: String,
    pub observed_generation: i64,
    pub ready_replicas: i64,
    pub desired_shard_count: u32,
    pub current_shard_count: u32,
    pub desired_replicas_per_shard: u32,
    pub current_ready_replicas_per_shard: u32,
    pub backup_phase: String,
    pub backup_message: String,
    pub archive_phase: String,
    pub archive_watermark: u64,
    pub last_archive_manifest: String,
    pub restore_phase: String,
    pub restore_source_manifest: String,
    pub backpressure: String,
    pub last_data_error: String,
    pub message: String,
}

const APP: &str = "sift";
const API_VERSION: &str = "sift.axiom.dev/v1alpha1";
const KIND: &str = "Sift";
const GATEWAY_COMPONENT: &str = "gateway";
const QUERY_COMPONENT: &str = "query";
const STORE_COMPONENT: &str = "store";
const CONTROL_COMPONENT: &str = "control";
const AGENT_COMPONENT: &str = "agent";
const BACKUP_COMPONENT: &str = "backup";
const AUTH_DELEGATION_COMPONENT: &str = "auth-delegation";
const AUTH_DELEGATOR_ROLE: &str = "system:auth-delegator";
const HTTP_PORT: i32 = 7380;
const OTLP_GRPC_PORT: i32 = 4317;
const PEER_MTLS_PORT: i32 = 7381;

fn three() -> u32 {
    3
}

fn render_children(
    sift: &Sift,
    kubernetes_api_cidrs: &[String],
    kubernetes_api_ports: &[u16],
) -> Vec<Value> {
    let name = sift.name_any();
    let namespace = sift.namespace().unwrap_or_else(|| "default".to_string());
    let owner = sift
        .metadata
        .uid
        .as_deref()
        .map(|uid| render::owner_ref(API_VERSION, KIND, &name, uid));
    let cx = RenderCtx {
        app: APP,
        manager: "sift-operator",
        api_version: API_VERSION,
        kind: KIND,
        name: &name,
        ns: &namespace,
        owner,
    };
    let mut plan = render::WorkloadPlan::new(&cx);
    plan.add_service_account(render::ServiceAccountPlan::new(&name, "runtime"));
    plan.add_service_account(render::ServiceAccountPlan::new(
        format!("{name}-store"),
        STORE_COMPONENT,
    ));
    plan.add_service_account(render::ServiceAccountPlan::new(
        format!("{name}-backup"),
        BACKUP_COMPONENT,
    ));

    if matches!(sift.spec.auth, AuthMode::Kubernetes) {
        plan.add_role(agent_project_role_plan(&cx));
        plan.add_role_binding(agent_project_role_binding_plan(&cx));
        let mut binding = render::ClusterRoleBindingPlan::new(
            auth_delegator_binding_name(&cx),
            AUTH_DELEGATION_COMPONENT,
            AUTH_DELEGATOR_ROLE,
        )
        .with_service_account(render::ServiceAccountSubjectPlan::new(&namespace, &name))
        .with_service_account(render::ServiceAccountSubjectPlan::new(
            &namespace,
            format!("{name}-store"),
        ))
        .with_label("sift.axiom.dev/owner-namespace", &namespace);
        if let Some(uid) = sift.metadata.uid.as_deref() {
            binding = binding.with_label("service-k8s.axiom.dev/owner-uid", uid);
        }
        plan.add_cluster_role_binding(binding);
    }

    plan.add_service(client_service_plan(&cx));
    for role in [STORE_COMPONENT, CONTROL_COMPONENT] {
        plan.add_service(role_service_plan(&cx, role));
        plan.add_service(headless_service_plan(&cx, role));
        plan.add_stateful_set(stateful_role_plan(&cx, &sift.spec, role));
        plan.add_pod_disruption_budget(disruption_budget_plan(&cx, role, 2));
    }
    for role in [GATEWAY_COMPONENT, QUERY_COMPONENT] {
        plan.add_service(role_service_plan(&cx, role));
        plan.add_deployment(deployment_role_plan(&cx, &sift.spec, role));
    }
    plan.add_daemon_set(agent_daemon_set_plan(&cx, &sift.spec));
    for policy in network_policy_plans(&cx, &sift.spec, kubernetes_api_cidrs, kubernetes_api_ports)
    {
        plan.add_network_policy(policy);
    }
    for policy in fqdn_network_policy_plans(&cx, &sift.spec) {
        plan.add_fqdn_network_policy(policy);
    }

    if let Some(backup) = &sift.spec.backup {
        plan.add_cron_job(backup_cron_job_plan(&cx, &sift.spec, backup));
    }
    plan.render()
        .expect("Sift typed Kubernetes workload plan must be valid")
}

async fn discover_kubernetes_api_endpoint(
    client: kube::Client,
) -> anyhow::Result<(Vec<String>, Vec<u16>)> {
    let endpoints = Api::<Endpoints>::namespaced(client, "default")
        .get("kubernetes")
        .await
        .map_err(|error| anyhow::anyhow!("discover Kubernetes API endpoints: {error}"))?;
    let mut cidrs = BTreeSet::new();
    let mut ports = BTreeSet::new();
    for subset in endpoints.subsets.into_iter().flatten() {
        for address in subset.addresses.into_iter().flatten() {
            let address = address.ip.parse::<IpAddr>().map_err(|error| {
                anyhow::anyhow!(
                    "Kubernetes API endpoint {} is not an IP address: {error}",
                    address.ip
                )
            })?;
            cidrs.insert(match address {
                IpAddr::V4(address) => format!("{address}/32"),
                IpAddr::V6(address) => format!("{address}/128"),
            });
        }
        for port in subset.ports.into_iter().flatten() {
            if port.protocol.as_deref().unwrap_or("TCP") == "TCP" {
                ports.insert(u16::try_from(port.port).map_err(|_| {
                    anyhow::anyhow!("Kubernetes API endpoint port {} is invalid", port.port)
                })?);
            }
        }
    }
    if cidrs.is_empty() {
        anyhow::bail!("Kubernetes API Endpoints/default/kubernetes has no ready addresses");
    }
    if ports.is_empty() {
        anyhow::bail!("Kubernetes API Endpoints/default/kubernetes has no TCP ports");
    }
    Ok((cidrs.into_iter().collect(), ports.into_iter().collect()))
}

impl ManagedService for Sift {
    const MANAGER: &'static str = "sift-operator";

    fn render(&self) -> Vec<Value> {
        render_children(self, &[], &[])
    }

    fn reconcile_plan(
        &self,
        client: kube::Client,
    ) -> impl std::future::Future<Output = anyhow::Result<service_k8s::service::ReconcilePlan>> + Send
    {
        let sift = self.clone();
        async move {
            let (api_cidrs, api_ports) = if matches!(sift.spec.auth, AuthMode::Kubernetes) {
                discover_kubernetes_api_endpoint(client).await?
            } else {
                (Vec::new(), Vec::new())
            };
            Ok(service_k8s::service::ReconcilePlan {
                children: render_children(&sift, &api_cidrs, &api_ports),
                context: Value::Null,
            })
        }
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        let name = self.name_any();
        vec![
            ReadinessTarget {
                kind: "StatefulSet",
                name: format!("{name}-store"),
            },
            ReadinessTarget {
                kind: "StatefulSet",
                name: format!("{name}-control"),
            },
            ReadinessTarget {
                kind: "Deployment",
                name: format!("{name}-gateway"),
            },
            ReadinessTarget {
                kind: "Deployment",
                name: format!("{name}-query"),
            },
            ReadinessTarget {
                kind: "DaemonSet",
                name: format!("{name}-agent"),
            },
        ]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> Value {
        let name = self.name_any();
        let store_ready = ready.get(&format!("{name}-store")).max(0);
        let control_ready = ready.get(&format!("{name}-control")).max(0);
        let gateway_ready = ready.get(&format!("{name}-gateway")).max(0);
        let query_ready = ready.get(&format!("{name}-query")).max(0);
        let agent_ready = ready.get(&format!("{name}-agent")).max(0);
        let ready_replicas =
            store_ready + control_ready + gateway_ready + query_ready + agent_ready;
        let desired_replicas_per_shard = self.spec.replicas_per_shard;
        let supported_topology = desired_replicas_per_shard == 3 && self.spec.voter_count == 3;
        let (phase, message) = if !supported_topology {
            (
                "UnsupportedTopology",
                format!(
                    "requested 1 shard x {desired_replicas_per_shard} replicas with {} voters; Sift requires exactly three durable voters",
                    self.spec.voter_count
                ),
            )
        } else if store_ready >= 3
            && control_ready >= 3
            && gateway_ready >= 1
            && query_ready >= 1
            && agent_ready >= 1
        {
            (
                "Ready",
                format!(
                    "store {store_ready}/3, control {control_ready}/3, gateway {gateway_ready}/1, query {query_ready}/1, agent {agent_ready} ready"
                ),
            )
        } else {
            (
                "Pending",
                format!(
                    "store {store_ready}/3, control {control_ready}/3, gateway {gateway_ready}/1, query {query_ready}/1, agent {agent_ready} ready"
                ),
            )
        };
        let (backup_phase, backup_message) = if self.spec.backup.is_some() {
            (
                "Configured",
                "scheduled live backup is configured; execution evidence is reported by its CronJob and destination",
            )
        } else {
            ("NotConfigured", "no scheduled backup requested")
        };
        let archive_phase = if self.spec.archive.is_some() {
            "Configured"
        } else {
            "NotConfigured"
        };
        let (restore_phase, restore_source_manifest) = self
            .spec
            .bootstrap
            .archive_manifest_uri
            .as_deref()
            .map(|uri| {
                let restore_phase = if store_ready >= 3 {
                    "Restored"
                } else if store_ready > 0 {
                    "Restoring"
                } else {
                    "Requested"
                };
                (restore_phase, uri)
            })
            .unwrap_or(("NotRequested", ""));
        json!({
            "status": {
                "phase": phase,
                "observedGeneration": self.metadata.generation.unwrap_or(0),
                "readyReplicas": ready_replicas,
                "desiredShardCount": 1,
                "currentShardCount": u32::from(store_ready >= 2),
                "desiredReplicasPerShard": desired_replicas_per_shard,
                "currentReadyReplicasPerShard": store_ready.min(3) as u32,
                "backupPhase": backup_phase,
                "backupMessage": backup_message,
                "archivePhase": archive_phase,
                "archiveWatermark": 0,
                "lastArchiveManifest": "",
                "restorePhase": restore_phase,
                "restoreSourceManifest": restore_source_manifest,
                "backpressure": "Healthy",
                "lastDataError": "",
                "message": message,
            }
        })
    }

    fn prunes(&self) -> Vec<service_k8s::service::PruneTarget> {
        let name = self.name_any();
        let store_uses_gcs = self
            .spec
            .archive
            .as_ref()
            .is_some_and(|archive| archive.destination.starts_with("gs://"))
            || self
                .spec
                .bootstrap
                .archive_manifest_uri
                .as_deref()
                .is_some_and(|uri| uri.starts_with("gs://"));
        let backup_uses_gcs = self
            .spec
            .backup
            .as_ref()
            .is_some_and(|backup| backup.destination.starts_with("gs://"));
        [
            (STORE_COMPONENT, store_uses_gcs),
            (BACKUP_COMPONENT, backup_uses_gcs),
        ]
        .into_iter()
        .filter(|(_, desired)| !desired)
        .map(|(role, _)| service_k8s::service::PruneTarget {
            api_version: "networking.gke.io/v1alpha1",
            kind: "FQDNNetworkPolicy",
            name: format!("{name}-{role}-google-apis"),
        })
        .collect()
    }

    fn cluster_scoped_children(&self) -> Vec<service_k8s::service::ClusterScopedChild> {
        let Some(uid) = self.metadata.uid.as_deref() else {
            return Vec::new();
        };
        let name = self.name_any();
        let namespace = self.namespace().unwrap_or_else(|| "default".to_string());
        vec![service_k8s::service::ClusterScopedChild {
            api_version: "rbac.authorization.k8s.io/v1",
            kind: "ClusterRoleBinding",
            name: format!("sift.{namespace}.{name}.auth-delegator"),
            expected_labels: BTreeMap::from([
                ("app.kubernetes.io/name".to_string(), APP.to_string()),
                ("app.kubernetes.io/instance".to_string(), name),
                (
                    "app.kubernetes.io/component".to_string(),
                    AUTH_DELEGATION_COMPONENT.to_string(),
                ),
                ("sift.axiom.dev/owner-namespace".to_string(), namespace),
                (
                    "service-k8s.axiom.dev/owner-uid".to_string(),
                    uid.to_string(),
                ),
            ]),
            desired: matches!(self.spec.auth, AuthMode::Kubernetes),
        }]
    }
}

fn role_name(cx: &RenderCtx<'_>, role: &str) -> String {
    format!("{}-{role}", cx.name)
}

fn auth_delegator_binding_name(cx: &RenderCtx<'_>) -> String {
    // Namespace names cannot contain dots, so this mapping cannot collide
    // between (namespace, instance) pairs that contain dashes.
    format!("sift.{}.{}.auth-delegator", cx.ns, cx.name)
}

fn storage_size<'a>(spec: &'a SiftSpec, role: &str) -> &'a str {
    if let Some(legacy) = spec.data_size.as_deref() {
        return legacy;
    }
    match role {
        STORE_COMPONENT => &spec.storage.store_size,
        CONTROL_COMPONENT => &spec.storage.control_size,
        GATEWAY_COMPONENT => &spec.storage.gateway_size,
        QUERY_COMPONENT => &spec.storage.query_size,
        _ => unreachable!("role {role} does not own a Sift PVC"),
    }
}

fn role_selector_labels(role: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("sift.axiom.dev/role".to_string(), role.to_string())])
}

fn role_extra_labels(role: &str) -> BTreeMap<String, String> {
    let mut labels = role_selector_labels(role);
    if matches!(role, GATEWAY_COMPONENT | QUERY_COMPONENT) {
        labels.insert("sift.axiom.dev/frontend".to_string(), "true".to_string());
    }
    labels
}

fn full_role_selector(cx: &RenderCtx<'_>, role: &str) -> BTreeMap<String, String> {
    let mut selector = BTreeMap::from([
        ("app.kubernetes.io/name".to_string(), APP.to_string()),
        (
            "app.kubernetes.io/instance".to_string(),
            cx.name.to_string(),
        ),
        ("app.kubernetes.io/component".to_string(), role.to_string()),
    ]);
    selector.extend(role_selector_labels(role));
    selector
}

fn container_security_context() -> Value {
    json!({
        "runAsNonRoot": true,
        "runAsUser": 65532,
        "runAsGroup": 65532,
        "allowPrivilegeEscalation": false,
        "readOnlyRootFilesystem": true,
        "capabilities": {"drop": ["ALL"]},
    })
}

fn data_init_container() -> Value {
    json!({
        "name": "prepare-data-root",
        "image": "busybox:1.36.1",
        "command": ["sh", "-ec"],
        "args": ["chown 65532:65532 /var/lib/sift && chmod 0700 /var/lib/sift && test \"$(stat -c '%u:%g:%a' /var/lib/sift)\" = '65532:65532:700'"],
        "volumeMounts": [{"name":"data", "mountPath":"/var/lib/sift"}],
        "securityContext": {
            "runAsNonRoot": false,
            "runAsUser": 0,
            "runAsGroup": 0,
            "allowPrivilegeEscalation": false,
            "readOnlyRootFilesystem": true,
            "capabilities": {"drop":["ALL"], "add":["CHOWN", "FOWNER"]}
        },
        "resources": {
            "requests": {"cpu":"5m", "memory":"8Mi"},
            "limits": {"cpu":"50m", "memory":"32Mi"}
        }
    })
}

fn role_env(cx: &RenderCtx<'_>, spec: &SiftSpec, role: &str) -> Vec<Value> {
    let auth = match spec.auth {
        AuthMode::Off => "off",
        AuthMode::Required => "required",
        AuthMode::Kubernetes => "kubernetes",
    };
    let mut env = vec![
        json!({"name":"SIFT_DATA_DIR", "value":"/var/lib/sift"}),
        json!({"name":"SIFT_AUTH", "value":auth}),
        json!({"name":"SIFT_STORE_ENDPOINT", "value":format!("http://{}-store.{}.svc.cluster.local:{HTTP_PORT}", cx.name, cx.ns)}),
        json!({"name":"SIFT_STORE_GRPC_ENDPOINT", "value":format!("http://{}-store.{}.svc.cluster.local:{OTLP_GRPC_PORT}", cx.name, cx.ns)}),
        json!({"name":"SIFT_QUERY_ENDPOINT", "value":format!("http://{}-query.{}.svc.cluster.local:{HTTP_PORT}", cx.name, cx.ns)}),
        json!({"name":"SIFT_CONTROL_ENDPOINT", "value":format!("http://{}-control.{}.svc.cluster.local:{HTTP_PORT}", cx.name, cx.ns)}),
        json!({"name":"SIFT_MCP_ALLOWED_HOSTS", "value":format!("localhost,127.0.0.1,{},{}.{}.svc,{}.{}.svc.cluster.local", cx.name, cx.name, cx.ns, cx.name, cx.ns)}),
        json!({"name":"SIFT_MCP_ALLOWED_ORIGINS", "value":format!("http://{}.{}.svc.cluster.local:{HTTP_PORT}", cx.name, cx.ns)}),
        json!({"name":"SIFT_MAX_INGEST_ITEMS_PER_PROJECT_WINDOW", "value":spec.ingest.max_items_per_minute.to_string()}),
        json!({"name":"SIFT_MAX_CONCURRENT_INGEST_PER_PROJECT", "value":spec.ingest.max_concurrent_requests.to_string()}),
        json!({"name":"SIFT_MAX_EVENTS_PER_BATCH", "value":"1000"}),
        json!({"name":"SIFT_INGEST_QUOTA_WINDOW_SECS", "value":"60"}),
    ];
    if role == STORE_COMPONENT {
        if let Some(archive) = &spec.archive {
            env.push(json!({
                "name":"SIFT_ARCHIVE_DESTINATION",
                "value":archive.destination
            }));
        }
        if let Some(manifest_uri) = &spec.bootstrap.archive_manifest_uri {
            env.push(json!({
                "name":"SIFT_BOOTSTRAP_ARCHIVE_MANIFEST_URI",
                "value":manifest_uri
            }));
        }
    }
    if matches!(role, STORE_COMPONENT | CONTROL_COMPONENT) {
        env.extend([
            json!({"name":"POD_NAME", "valueFrom":{"fieldRef":{"fieldPath":"metadata.name"}}}),
            json!({"name":"SHARD_COUNT", "value":"1"}),
            json!({"name":"REPLICAS_PER_SHARD", "value":"3"}),
            json!({"name":"VOTER_COUNT", "value":"3"}),
            json!({
                "name":"SIFT_RAFT_HEADLESS",
                "value":format!("{}-{role}-headless.{}.svc.cluster.local", cx.name, cx.ns)
            }),
            json!({"name":"SIFT_PEER_PORT", "value":PEER_MTLS_PORT.to_string()}),
            json!({"name":"SIFT_PEER_MTLS", "value":"on"}),
            json!({"name":"SIFT_PEER_TLS_CERT", "value":"/var/run/secrets/sift/peer-tls/tls.crt"}),
            json!({"name":"SIFT_PEER_TLS_KEY", "value":"/var/run/secrets/sift/peer-tls/tls.key"}),
            json!({"name":"SIFT_PEER_TLS_CA", "value":"/var/run/secrets/sift/peer-tls/ca.crt"}),
        ]);
    }
    if matches!(spec.auth, AuthMode::Kubernetes) {
        env.extend([
            json!({"name":"SIFT_K8S_AUDIENCE", "value":"sift.axiom.dev"}),
            json!({"name":"POD_NAMESPACE", "valueFrom":{"fieldRef":{"fieldPath":"metadata.namespace"}}}),
        ]);
    }
    if matches!(spec.auth, AuthMode::Required) {
        env.push(json!({
            "name":"SIFT_TOKEN_REGISTRY_FILE",
            "value":"/var/run/secrets/sift/token-registry.json"
        }));
    }
    env
}

fn role_support_volumes(spec: &SiftSpec, role: &str) -> (Vec<Value>, Vec<Value>) {
    let mut volumes = Vec::new();
    let mut mounts = Vec::new();
    if matches!(role, STORE_COMPONENT | CONTROL_COMPONENT) {
        volumes.push(json!({
            "name":"peer-tls", "secret":{"secretName":spec.peer_tls_secret}
        }));
        mounts.push(json!({
            "name":"peer-tls", "mountPath":"/var/run/secrets/sift/peer-tls", "readOnly":true
        }));
    }
    if matches!(spec.auth, AuthMode::Required) {
        if let Some(secret) = &spec.tokens_secret {
            volumes.push(json!({"name":"tokens", "secret":{"secretName":secret}}));
            mounts.push(json!({
                "name":"tokens", "mountPath":"/var/run/secrets/sift", "readOnly":true
            }));
        }
    }
    (volumes, mounts)
}

fn role_container_plan(
    cx: &RenderCtx<'_>,
    spec: &SiftSpec,
    role: &str,
    mounts: Vec<Value>,
) -> render::ContainerPlan {
    let mut ports = vec![json!({"name":"http", "containerPort":HTTP_PORT})];
    if matches!(role, GATEWAY_COMPONENT | STORE_COMPONENT) {
        ports.push(json!({"name":"otlp-grpc", "containerPort":OTLP_GRPC_PORT}));
    }
    if matches!(role, STORE_COMPONENT | CONTROL_COMPONENT) {
        ports.push(json!({"name":"raft-mtls", "containerPort":PEER_MTLS_PORT}));
    }
    let mut container = render::ContainerPlan::new(
        "sift",
        &spec.image,
        vec![
            "serve".into(),
            "--role".into(),
            role.into(),
            "--data-dir".into(),
            "/var/lib/sift".into(),
        ],
    );
    container.ports = ports;
    container.env = role_env(cx, spec, role);
    container.volume_mounts = mounts;
    container.security_context = Some(container_security_context());
    container.resources = Some(json!({"requests":{"cpu":"100m","memory":"256Mi"}}));
    container.readiness_probe = Some(
        json!({"httpGet":{"path":"/readyz","port":"http"},"periodSeconds":5,"timeoutSeconds":3,"failureThreshold":60}),
    );
    container.liveness_probe = Some(
        json!({"httpGet":{"path":"/healthz","port":"http"},"periodSeconds":15,"timeoutSeconds":5,"failureThreshold":3}),
    );
    container.startup_probe = Some(
        json!({"httpGet":{"path":"/healthz","port":"http"},"periodSeconds":5,"timeoutSeconds":3,"failureThreshold":120}),
    );
    container
}

fn role_pod_plan(cx: &RenderCtx<'_>, spec: &SiftSpec, role: &str) -> render::PodPlan {
    let (volumes, mounts) = role_support_volumes(spec, role);
    let service_account = if role == STORE_COMPONENT {
        format!("{}-store", cx.name)
    } else {
        cx.name.to_string()
    };
    let mut runtime = render::PodRuntimePolicy::restricted(
        service_account,
        serde_json::to_value(&spec.placement.node_selector).expect("node selector is JSON"),
    )
    .with_automount_service_account_token(matches!(spec.auth, AuthMode::Kubernetes))
    .with_init_containers(vec![data_init_container()])
    .with_volumes(volumes);
    if matches!(role, STORE_COMPONENT | CONTROL_COMPONENT) {
        runtime = runtime.with_affinity(render::dedicated_node_affinity(
            serde_json::to_value(full_role_selector(cx, role)).expect("role selector is JSON"),
        ));
    }
    render::PodPlan::new(role, role_container_plan(cx, spec, role, mounts), runtime)
        .with_selector_labels(role_selector_labels(role))
        .with_labels(role_extra_labels(role))
}

fn client_service_plan(cx: &RenderCtx<'_>) -> render::ServicePlan {
    render::ServicePlan::cluster_ip(
        cx.name,
        "frontend",
        role_selector_labels(GATEWAY_COMPONENT),
        vec![
            render::ServicePortPlan::tcp("http", HTTP_PORT, "http"),
            render::ServicePortPlan::tcp("otlp-grpc", OTLP_GRPC_PORT, "otlp-grpc"),
        ],
    )
    .with_selector_component(GATEWAY_COMPONENT)
}

fn role_service_plan(cx: &RenderCtx<'_>, role: &str) -> render::ServicePlan {
    let name = role_name(cx, role);
    let mut ports = vec![render::ServicePortPlan::tcp("http", HTTP_PORT, "http")];
    if matches!(role, GATEWAY_COMPONENT | STORE_COMPONENT) {
        ports.push(render::ServicePortPlan::tcp(
            "otlp-grpc",
            OTLP_GRPC_PORT,
            "otlp-grpc",
        ));
    }
    render::ServicePlan::cluster_ip(name, role, role_selector_labels(role), ports)
}

fn headless_service_plan(cx: &RenderCtx<'_>, role: &str) -> render::ServicePlan {
    let name = format!("{}-headless", role_name(cx, role));
    render::ServicePlan::headless(
        name,
        role,
        role_selector_labels(role),
        vec![render::ServicePortPlan::tcp(
            "raft-mtls",
            PEER_MTLS_PORT,
            "raft-mtls",
        )],
    )
}

fn stateful_role_plan(cx: &RenderCtx<'_>, spec: &SiftSpec, role: &str) -> render::StatefulSetPlan {
    let name = role_name(cx, role);
    let headless = format!("{name}-headless");
    render::StatefulSetPlan::new(
        name,
        headless,
        3,
        role_pod_plan(cx, spec, role),
        render::PersistentVolumeClaimPlan::new(
            "data",
            role,
            storage_size(spec, role),
            "/var/lib/sift",
        ),
    )
}

fn deployment_role_plan(cx: &RenderCtx<'_>, spec: &SiftSpec, role: &str) -> render::DeploymentPlan {
    let name = role_name(cx, role);
    let claim = format!("{name}-data");
    let mut deployment = render::DeploymentPlan::new(name, 1, role_pod_plan(cx, spec, role))
        .with_persistent_claim(render::PersistentVolumeClaimPlan::new(
            claim,
            role,
            storage_size(spec, role),
            "/var/lib/sift",
        ));
    deployment.strategy = Some(json!({"type":"Recreate"}));
    deployment
}

fn disruption_budget_plan(
    cx: &RenderCtx<'_>,
    role: &str,
    min_available: u32,
) -> render::PodDisruptionBudgetPlan {
    let name = role_name(cx, role);
    render::PodDisruptionBudgetPlan::min_available(
        name,
        role,
        role_selector_labels(role),
        min_available,
    )
}

fn agent_daemon_set_plan(cx: &RenderCtx<'_>, spec: &SiftSpec) -> render::DaemonSetPlan {
    let name = role_name(cx, AGENT_COMPONENT);
    let mut env = vec![
        json!({"name":"SIFT_DATA_DIR", "value":"/var/lib/sift"}),
        json!({"name":"SIFT_URL", "value":format!("http://{}:{HTTP_PORT}", cx.name)}),
        json!({"name":"NODE_NAME", "valueFrom":{"fieldRef":{"fieldPath":"spec.nodeName"}}}),
    ];
    if matches!(spec.auth, AuthMode::Required) {
        if let Some(secret) = &spec.tokens_secret {
            env.push(json!({
                "name":"SIFT_TOKEN",
                "valueFrom":{"secretKeyRef":{"name":secret,"key":"agent-token"}}
            }));
        }
    }
    if matches!(spec.auth, AuthMode::Kubernetes) {
        env.extend([
            json!({"name":"SIFT_TOKEN_FILE", "value":"/var/run/secrets/sift/client/token"}),
            json!({"name":"SIFT_TOKEN_AUDIENCE", "value":"sift.axiom.dev"}),
        ]);
    }
    let mut agent_mounts = vec![
        json!({"name":"pod-logs","mountPath":"/var/log/pods","readOnly":true}),
        json!({"name":"data","mountPath":"/var/lib/sift"}),
    ];
    let mut agent_volumes = vec![
        json!({"name":"pod-logs","hostPath":{"path":"/var/log/pods","type":"Directory"}}),
        json!({"name":"data","hostPath":{"path":"/var/lib/sift","type":"DirectoryOrCreate"}}),
    ];
    if matches!(spec.auth, AuthMode::Kubernetes) {
        agent_mounts.push(json!({
            "name":"sift-client-token",
            "mountPath":"/var/run/secrets/sift/client",
            "readOnly":true
        }));
        agent_volumes.push(json!({
            "name":"sift-client-token",
            "projected":{
                "defaultMode": 420,
                "sources":[{"serviceAccountToken":{
                    "audience":"sift.axiom.dev",
                    "expirationSeconds":600,
                    "path":"token"
                }}]
            }
        }));
    }
    let runtime = render::PodRuntimePolicy::restricted(
        cx.name,
        serde_json::to_value(&spec.placement.node_selector).expect("node selector is JSON"),
    )
    .with_init_containers(vec![json!({
        "name":"prepare-data-root",
        "image":"busybox:1.36.1",
        "command":["sh","-ec"],
        "args":["mkdir -p /var/lib/sift/agent && chown -R 65532:65532 /var/lib/sift && chmod 0700 /var/lib/sift /var/lib/sift/agent"],
        "volumeMounts":[{"name":"data","mountPath":"/var/lib/sift"}],
        "securityContext":{
            "runAsNonRoot":false,"runAsUser":0,"runAsGroup":0,
            "allowPrivilegeEscalation":false,"readOnlyRootFilesystem":true,
            "capabilities":{"drop":["ALL"],"add":["CHOWN","FOWNER","DAC_OVERRIDE"]}
        }
    })])
    .with_volumes(agent_volumes);
    let mut container = render::ContainerPlan::new(
        "sift",
        &spec.image,
        vec![
            "collect".into(),
            "--cri-root".into(),
            "/var/log/pods".into(),
            "--data-dir".into(),
            "/var/lib/sift".into(),
            "--checkpoint".into(),
            "/var/lib/sift/agent/checkpoint.json".into(),
            "--quarantine".into(),
            "/var/lib/sift/agent/rejected.jsonl".into(),
            "--project".into(),
            cx.name.into(),
            "--environment".into(),
            cx.ns.into(),
            "--gcp-project".into(),
            spec.gcp_project_id.clone(),
            "--cluster".into(),
            spec.gke_cluster_name.clone(),
            "--location".into(),
            spec.gke_location.clone(),
            "--node".into(),
            "$(NODE_NAME)".into(),
            "--follow".into(),
        ],
    );
    container.env = env;
    container.volume_mounts = agent_mounts;
    container.security_context = Some(json!({
        "runAsNonRoot":false,"runAsUser":0,
        "allowPrivilegeEscalation":false,"readOnlyRootFilesystem":true,
        "capabilities":{"drop":["ALL"],"add":["DAC_OVERRIDE"]}
    }));
    container.resources = Some(
        json!({"requests":{"cpu":"25m","memory":"64Mi"},"limits":{"cpu":"500m","memory":"256Mi"}}),
    );
    render::DaemonSetPlan::new(
        name,
        render::PodPlan::new(AGENT_COMPONENT, container, runtime)
            .with_selector_labels(role_selector_labels(AGENT_COMPONENT))
            .with_labels(role_extra_labels(AGENT_COMPONENT)),
    )
}

fn agent_project_role_plan(cx: &RenderCtx<'_>) -> render::RolePlan {
    let name = format!("{}-agent-project", cx.name);
    render::RolePlan {
        name,
        component: "auth".into(),
        rules: vec![render::RbacRulePlan {
            api_groups: vec!["sift.axiom.dev".into()],
            resources: vec!["projects".into()],
            resource_names: vec![cx.name.into()],
            verbs: vec!["get".into(), "create".into(), "update".into()],
        }],
    }
}

fn projected_client_token_volume() -> Value {
    json!({
        "name":"sift-client-token",
        "projected":{
            "defaultMode":420,
            "sources":[{"serviceAccountToken":{
                "audience":"sift.axiom.dev",
                "expirationSeconds":600,
                "path":"token"
            }}]
        }
    })
}

fn projected_client_token_mount() -> Value {
    json!({
        "name":"sift-client-token",
        "mountPath":"/var/run/secrets/sift/client",
        "readOnly":true
    })
}

fn agent_project_role_binding_plan(cx: &RenderCtx<'_>) -> render::RoleBindingPlan {
    let name = format!("{}-agent-project", cx.name);
    render::RoleBindingPlan {
        name: name.clone(),
        component: "auth".into(),
        role_name: name,
        subjects: vec![
            render::ServiceAccountSubjectPlan {
                name: cx.name.into(),
                namespace: cx.ns.into(),
            },
            render::ServiceAccountSubjectPlan {
                name: format!("{}-backup", cx.name),
                namespace: cx.ns.into(),
            },
        ],
    }
}

fn backup_cron_job_plan(
    cx: &RenderCtx<'_>,
    spec: &SiftSpec,
    backup: &BackupSpec,
) -> render::CronJobPlan {
    let backup_name = format!("{}-backup", cx.name);
    let mut args = vec![
        "backup".to_string(),
        "--url".to_string(),
        format!("http://{}.{}.svc.cluster.local:{HTTP_PORT}", cx.name, cx.ns),
        "--dest".to_string(),
        backup.destination.clone(),
    ];
    if let Some(seconds) = backup.retention_secs {
        args.push("--retention-secs".to_string());
        args.push(seconds.to_string());
    }
    let mut env = Vec::new();
    let mut volumes = Vec::new();
    let mut mounts = Vec::new();
    if matches!(spec.auth, AuthMode::Kubernetes) {
        args.extend([
            "--token-file".to_string(),
            "/var/run/secrets/sift/client/token".to_string(),
            "--token-audience".to_string(),
            "sift.axiom.dev".to_string(),
            "--project".to_string(),
            cx.name.to_string(),
        ]);
        volumes.push(projected_client_token_volume());
        mounts.push(projected_client_token_mount());
    } else if let Some(secret) = &backup.admin_token_secret {
        env.push(json!({
            "name": "SIFT_BACKUP_TOKEN",
            "valueFrom": {"secretKeyRef": {"name": secret, "key": "token"}}
        }));
    }
    let runtime = render::PodRuntimePolicy::restricted(
        &backup_name,
        serde_json::to_value(&spec.placement.node_selector).expect("node selector is JSON"),
    )
    .with_volumes(volumes)
    .with_restart_policy("OnFailure");
    let mut container = render::ContainerPlan::new("backup", &spec.image, args);
    container.env = env;
    container.volume_mounts = mounts;
    container.security_context = Some(container_security_context());
    container.resources = Some(json!({
        "requests":{"cpu":"100m","memory":"128Mi"},
        "limits":{"cpu":"500m","memory":"512Mi"}
    }));
    render::CronJobPlan {
        name: backup_name,
        schedule: backup.schedule.clone(),
        successful_jobs_history_limit: 3,
        failed_jobs_history_limit: 3,
        pod: render::PodPlan::new(BACKUP_COMPONENT, container, runtime)
            .with_selector_labels(role_selector_labels(BACKUP_COMPONENT)),
    }
}

fn dns_egress() -> render::NetworkRulePlan {
    render::NetworkRulePlan::new(
        vec![render::NetworkPeerPlan::pods_in_namespace(
            "kube-system",
            BTreeMap::new(),
        )],
        vec![
            render::NetworkPortPlan::udp(53),
            render::NetworkPortPlan::tcp(53),
        ],
    )
}

fn metadata_server_egress() -> render::NetworkRulePlan {
    render::NetworkRulePlan::new(
        vec![render::NetworkPeerPlan::ip_block("169.254.169.254/32")],
        vec![
            render::NetworkPortPlan::tcp(80),
            render::NetworkPortPlan::tcp(8080),
        ],
    )
}

fn kubernetes_api_egress(cidrs: &[String], ports: &[u16]) -> render::NetworkRulePlan {
    render::NetworkRulePlan::new(
        cidrs
            .iter()
            .map(|cidr| render::NetworkPeerPlan::ip_block(cidr))
            .collect(),
        ports
            .iter()
            .copied()
            .map(|port| render::NetworkPortPlan::tcp(i32::from(port)))
            .collect(),
    )
}

fn role_peer(cx: &RenderCtx<'_>, role: &str) -> render::NetworkPeerPlan {
    render::NetworkPeerPlan::same_namespace_pods(full_role_selector(cx, role))
}

fn instance_peer(cx: &RenderCtx<'_>) -> render::NetworkPeerPlan {
    render::NetworkPeerPlan::same_namespace_pods(BTreeMap::from([
        ("app.kubernetes.io/name".to_string(), APP.to_string()),
        (
            "app.kubernetes.io/instance".to_string(),
            cx.name.to_string(),
        ),
    ]))
}

fn role_network_policy(cx: &RenderCtx<'_>, role: &str) -> render::NetworkPolicyPlan {
    render::NetworkPolicyPlan::new(
        format!("{}-{role}-network", cx.name),
        role,
        role_selector_labels(role),
    )
    .with_egress(dns_egress())
}

fn network_policy_plans(
    cx: &RenderCtx<'_>,
    spec: &SiftSpec,
    kubernetes_api_cidrs: &[String],
    kubernetes_api_ports: &[u16],
) -> Vec<render::NetworkPolicyPlan> {
    let default_deny =
        render::NetworkPolicyPlan::new(format!("{}-network", cx.name), "network", BTreeMap::new())
            .instance_wide();

    let mut gateway = role_network_policy(cx, GATEWAY_COMPONENT)
        .with_ingress(render::NetworkRulePlan::new(
            vec![render::NetworkPeerPlan::any()],
            vec![
                render::NetworkPortPlan::tcp(HTTP_PORT),
                render::NetworkPortPlan::tcp(OTLP_GRPC_PORT),
            ],
        ))
        .with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, QUERY_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ))
        .with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, STORE_COMPONENT)],
            vec![
                render::NetworkPortPlan::tcp(HTTP_PORT),
                render::NetworkPortPlan::tcp(OTLP_GRPC_PORT),
            ],
        ));
    let mut query = role_network_policy(cx, QUERY_COMPONENT)
        .with_ingress(render::NetworkRulePlan::new(
            vec![role_peer(cx, GATEWAY_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ))
        .with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, STORE_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ));
    let mut store = role_network_policy(cx, STORE_COMPONENT)
        .with_ingress(render::NetworkRulePlan::new(
            vec![role_peer(cx, GATEWAY_COMPONENT)],
            vec![
                render::NetworkPortPlan::tcp(HTTP_PORT),
                render::NetworkPortPlan::tcp(OTLP_GRPC_PORT),
            ],
        ))
        .with_ingress(render::NetworkRulePlan::new(
            vec![role_peer(cx, QUERY_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ))
        .with_ingress(render::NetworkRulePlan::new(
            vec![role_peer(cx, STORE_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(PEER_MTLS_PORT)],
        ))
        .with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, STORE_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(PEER_MTLS_PORT)],
        ));
    let mut control = role_network_policy(cx, CONTROL_COMPONENT)
        .with_ingress(render::NetworkRulePlan::new(
            vec![instance_peer(cx)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ))
        .with_ingress(render::NetworkRulePlan::new(
            vec![role_peer(cx, CONTROL_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(PEER_MTLS_PORT)],
        ))
        .with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, CONTROL_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(PEER_MTLS_PORT)],
        ));
    let agent = role_network_policy(cx, AGENT_COMPONENT).with_egress(render::NetworkRulePlan::new(
        vec![role_peer(cx, GATEWAY_COMPONENT)],
        vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
    ));
    let mut backup =
        role_network_policy(cx, BACKUP_COMPONENT).with_egress(render::NetworkRulePlan::new(
            vec![role_peer(cx, GATEWAY_COMPONENT)],
            vec![render::NetworkPortPlan::tcp(HTTP_PORT)],
        ));
    if matches!(spec.auth, AuthMode::Kubernetes) && !kubernetes_api_cidrs.is_empty() {
        gateway = gateway.with_egress(kubernetes_api_egress(
            kubernetes_api_cidrs,
            kubernetes_api_ports,
        ));
        query = query.with_egress(kubernetes_api_egress(
            kubernetes_api_cidrs,
            kubernetes_api_ports,
        ));
        store = store.with_egress(kubernetes_api_egress(
            kubernetes_api_cidrs,
            kubernetes_api_ports,
        ));
        control = control.with_egress(kubernetes_api_egress(
            kubernetes_api_cidrs,
            kubernetes_api_ports,
        ));
    }

    let store_uses_gcs = spec
        .archive
        .as_ref()
        .is_some_and(|archive| archive.destination.starts_with("gs://"))
        || spec
            .bootstrap
            .archive_manifest_uri
            .as_deref()
            .is_some_and(|uri| uri.starts_with("gs://"));
    if store_uses_gcs {
        store = store.with_egress(metadata_server_egress());
    }
    if spec
        .backup
        .as_ref()
        .is_some_and(|backup| backup.destination.starts_with("gs://"))
    {
        backup = backup.with_egress(metadata_server_egress());
    }

    vec![default_deny, gateway, query, store, control, agent, backup]
}

fn fqdn_network_policy_plans(
    cx: &RenderCtx<'_>,
    spec: &SiftSpec,
) -> Vec<render::FqdnNetworkPolicyPlan> {
    let mut policies = Vec::new();
    let store_uses_gcs = spec
        .archive
        .as_ref()
        .is_some_and(|archive| archive.destination.starts_with("gs://"))
        || spec
            .bootstrap
            .archive_manifest_uri
            .as_deref()
            .is_some_and(|uri| uri.starts_with("gs://"));
    if store_uses_gcs {
        policies.push(google_apis_policy(cx, STORE_COMPONENT));
    }
    if spec
        .backup
        .as_ref()
        .is_some_and(|backup| backup.destination.starts_with("gs://"))
    {
        policies.push(google_apis_policy(cx, BACKUP_COMPONENT));
    }
    policies
}

fn google_apis_policy(cx: &RenderCtx<'_>, role: &str) -> render::FqdnNetworkPolicyPlan {
    render::FqdnNetworkPolicyPlan::new(
        format!("{}-{role}-google-apis", cx.name),
        role,
        full_role_selector(cx, role),
    )
    .with_match(render::FqdnMatchPlan::name("storage.googleapis.com"))
    .with_port(render::NetworkPortPlan::tcp(443))
}

pub async fn run() -> anyhow::Result<()> {
    service_k8s::run::<Sift>().await
}
// HANDWRITE-END

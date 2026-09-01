//! Typed composition for a complete Kubernetes workload set.
//!
//! A product supplies names, selectors, ports, storage sizes, and security
//! policy. This module owns the repeated Kubernetes object shapes.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use thiserror::Error;

use super::{rbac, RenderCtx};

pub type LabelSet = BTreeMap<String, String>;

#[derive(Clone, Debug)]
pub struct PodRuntimePolicy {
    pub service_account_name: String,
    pub automount_service_account_token: bool,
    pub enable_service_links: bool,
    pub termination_grace_period_seconds: u64,
    pub security_context: Value,
    pub node_selector: Value,
    pub affinity: Option<Value>,
    pub init_containers: Vec<Value>,
    pub volumes: Vec<Value>,
    pub restart_policy: Option<String>,
}

impl PodRuntimePolicy {
    /// Restricted non-root baseline for stateful and stateless services.
    pub fn restricted(service_account_name: impl Into<String>, node_selector: Value) -> Self {
        Self {
            service_account_name: service_account_name.into(),
            automount_service_account_token: false,
            enable_service_links: false,
            termination_grace_period_seconds: 30,
            security_context: json!({
                "runAsNonRoot": true,
                "runAsUser": 65532,
                "runAsGroup": 65532,
                "fsGroup": 65532,
                "fsGroupChangePolicy": "OnRootMismatch",
                "seccompProfile": {"type": "RuntimeDefault"},
            }),
            node_selector,
            affinity: None,
            init_containers: Vec::new(),
            volumes: Vec::new(),
            restart_policy: None,
        }
    }

    pub fn with_automount_service_account_token(mut self, value: bool) -> Self {
        self.automount_service_account_token = value;
        self
    }

    pub fn with_affinity(mut self, value: Value) -> Self {
        self.affinity = Some(value);
        self
    }

    pub fn with_init_containers(mut self, values: Vec<Value>) -> Self {
        self.init_containers = values;
        self
    }

    pub fn with_volumes(mut self, values: Vec<Value>) -> Self {
        self.volumes = values;
        self
    }

    pub fn with_restart_policy(mut self, value: impl Into<String>) -> Self {
        self.restart_policy = Some(value.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct ContainerPlan {
    pub name: String,
    pub image: String,
    pub image_pull_policy: Option<String>,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub ports: Vec<Value>,
    pub env: Vec<Value>,
    pub env_from: Vec<Value>,
    pub volume_mounts: Vec<Value>,
    pub security_context: Option<Value>,
    pub resources: Option<Value>,
    pub readiness_probe: Option<Value>,
    pub liveness_probe: Option<Value>,
    pub startup_probe: Option<Value>,
    pub lifecycle: Option<Value>,
}

impl ContainerPlan {
    pub fn new(name: impl Into<String>, image: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            image_pull_policy: None,
            command: Vec::new(),
            args,
            ports: Vec::new(),
            env: Vec::new(),
            env_from: Vec::new(),
            volume_mounts: Vec::new(),
            security_context: None,
            resources: None,
            readiness_probe: None,
            liveness_probe: None,
            startup_probe: None,
            lifecycle: None,
        }
    }

    fn render(mut self) -> Value {
        let mut value = json!({
            "name": self.name,
            "image": self.image,
            "args": self.args,
            "ports": self.ports,
            "env": self.env,
            "volumeMounts": self.volume_mounts,
        });
        if let Some(policy) = self.image_pull_policy.take() {
            value["imagePullPolicy"] = json!(policy);
        }
        if !self.command.is_empty() {
            value["command"] = json!(self.command);
        }
        if !self.env_from.is_empty() {
            value["envFrom"] = json!(self.env_from);
        }
        for (key, field) in [
            ("securityContext", self.security_context),
            ("resources", self.resources),
            ("readinessProbe", self.readiness_probe),
            ("livenessProbe", self.liveness_probe),
            ("startupProbe", self.startup_probe),
            ("lifecycle", self.lifecycle),
        ] {
            if let Some(field) = field {
                value[key] = field;
            }
        }
        value
    }
}

#[derive(Clone, Debug)]
pub struct PodPlan {
    pub component: String,
    pub selector_labels: LabelSet,
    pub labels: LabelSet,
    pub container: ContainerPlan,
    pub runtime: PodRuntimePolicy,
}

impl PodPlan {
    pub fn new(
        component: impl Into<String>,
        container: ContainerPlan,
        runtime: PodRuntimePolicy,
    ) -> Self {
        Self {
            component: component.into(),
            selector_labels: LabelSet::new(),
            labels: LabelSet::new(),
            container,
            runtime,
        }
    }

    pub fn with_selector_labels(mut self, labels: LabelSet) -> Self {
        self.selector_labels.extend(labels.clone());
        self.labels.extend(labels);
        self
    }

    pub fn with_labels(mut self, labels: LabelSet) -> Self {
        self.labels.extend(labels);
        self
    }

    fn selector(&self, cx: &RenderCtx<'_>) -> Value {
        merge_string_labels(cx.selector(&self.component), &self.selector_labels)
    }

    fn labels(&self, cx: &RenderCtx<'_>) -> Value {
        merge_string_labels(cx.labels(&self.component), &self.labels)
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let labels = self.labels(cx);
        let mut spec = json!({
            "serviceAccountName": self.runtime.service_account_name,
            "automountServiceAccountToken": self.runtime.automount_service_account_token,
            "enableServiceLinks": self.runtime.enable_service_links,
            "terminationGracePeriodSeconds": self.runtime.termination_grace_period_seconds,
            "securityContext": self.runtime.security_context,
            "containers": [self.container.render()],
        });
        if self.runtime.node_selector != json!({}) {
            spec["nodeSelector"] = self.runtime.node_selector;
        }
        if let Some(affinity) = self.runtime.affinity {
            spec["affinity"] = affinity;
        }
        if !self.runtime.init_containers.is_empty() {
            spec["initContainers"] = json!(self.runtime.init_containers);
        }
        if !self.runtime.volumes.is_empty() {
            spec["volumes"] = json!(self.runtime.volumes);
        }
        if let Some(restart_policy) = self.runtime.restart_policy {
            spec["restartPolicy"] = json!(restart_policy);
        }
        json!({"metadata": {"labels": labels}, "spec": spec})
    }
}

fn merge_string_labels(mut base: Value, extra: &LabelSet) -> Value {
    let object = base.as_object_mut().expect("render labels are an object");
    for (key, value) in extra {
        object.insert(key.clone(), json!(value));
    }
    base
}

#[derive(Clone, Debug)]
pub struct ServiceAccountPlan {
    pub name: String,
    pub component: String,
    pub automount_service_account_token: bool,
}

impl ServiceAccountPlan {
    pub fn new(name: impl Into<String>, component: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            automount_service_account_token: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServicePortPlan {
    pub name: String,
    pub port: i32,
    pub target_port: String,
    pub protocol: String,
}

impl ServicePortPlan {
    pub fn tcp(name: impl Into<String>, port: i32, target_port: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            port,
            target_port: target_port.into(),
            protocol: "TCP".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServicePlan {
    pub name: String,
    pub component: String,
    pub selector_component: String,
    pub selector: LabelSet,
    pub ports: Vec<ServicePortPlan>,
    pub cluster_ip: Option<String>,
    pub publish_not_ready_addresses: bool,
    pub service_type: Option<String>,
}

impl ServicePlan {
    pub fn cluster_ip(
        name: impl Into<String>,
        component: impl Into<String>,
        selector: LabelSet,
        ports: Vec<ServicePortPlan>,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            selector_component: String::new(),
            selector,
            ports,
            cluster_ip: None,
            publish_not_ready_addresses: false,
            service_type: Some("ClusterIP".into()),
        }
        .with_default_selector_component()
    }

    pub fn headless(
        name: impl Into<String>,
        component: impl Into<String>,
        selector: LabelSet,
        ports: Vec<ServicePortPlan>,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            selector_component: String::new(),
            selector,
            ports,
            cluster_ip: Some("None".into()),
            publish_not_ready_addresses: true,
            service_type: None,
        }
        .with_default_selector_component()
    }

    fn with_default_selector_component(mut self) -> Self {
        self.selector_component = self.component.clone();
        self
    }

    pub fn with_selector_component(mut self, component: impl Into<String>) -> Self {
        self.selector_component = component.into();
        self
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let ports = self
            .ports
            .into_iter()
            .map(|port| {
                json!({
                    "name": port.name,
                    "port": port.port,
                    "targetPort": port.target_port,
                    "protocol": port.protocol,
                })
            })
            .collect::<Vec<_>>();
        let selector = merge_string_labels(cx.selector(&self.selector_component), &self.selector);
        let mut spec = json!({"selector": selector, "ports": ports});
        if let Some(cluster_ip) = self.cluster_ip {
            spec["clusterIP"] = json!(cluster_ip);
        }
        if self.publish_not_ready_addresses {
            spec["publishNotReadyAddresses"] = json!(true);
        }
        if let Some(service_type) = self.service_type {
            spec["type"] = json!(service_type);
        }
        json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": cx.meta(&self.name, &self.component),
            "spec": spec,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PersistentVolumeClaimPlan {
    pub name: String,
    pub component: String,
    pub storage: String,
    pub mount_path: String,
    pub access_modes: Vec<String>,
}

impl PersistentVolumeClaimPlan {
    pub fn new(
        name: impl Into<String>,
        component: impl Into<String>,
        storage: impl Into<String>,
        mount_path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            storage: storage.into(),
            mount_path: mount_path.into(),
            access_modes: vec!["ReadWriteOnce".into()],
        }
    }

    fn standalone(&self, cx: &RenderCtx<'_>) -> Value {
        json!({
            "apiVersion": "v1",
            "kind": "PersistentVolumeClaim",
            "metadata": cx.meta(&self.name, &self.component),
            "spec": {
                "accessModes": self.access_modes,
                "resources": {"requests": {"storage": self.storage}},
            },
        })
    }

    fn template(&self) -> Value {
        json!({
            "metadata": {"name": self.name},
            "spec": {
                "accessModes": self.access_modes,
                "resources": {"requests": {"storage": self.storage}},
            },
        })
    }
}

fn attach_claim(template: &mut Value, volume_name: &str, claim_name: Option<&str>, mount: &str) {
    let pod = template["spec"]
        .as_object_mut()
        .expect("typed pod spec is an object");
    let containers = pod["containers"]
        .as_array_mut()
        .expect("typed pod containers are an array");
    let mounts = containers[0]["volumeMounts"]
        .as_array_mut()
        .expect("typed volume mounts are an array");
    mounts.insert(0, json!({"name": volume_name, "mountPath": mount}));
    if let Some(claim_name) = claim_name {
        let volumes = pod
            .entry("volumes")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .expect("typed pod volumes are an array");
        volumes.insert(
            0,
            json!({
                "name": volume_name,
                "persistentVolumeClaim": {"claimName": claim_name},
            }),
        );
    }
}

#[derive(Clone, Debug)]
pub struct StatefulSetPlan {
    pub name: String,
    pub service_name: String,
    pub replicas: u32,
    pub pod_management_policy: String,
    pub pod: PodPlan,
    pub claim: PersistentVolumeClaimPlan,
}

impl StatefulSetPlan {
    pub fn new(
        name: impl Into<String>,
        service_name: impl Into<String>,
        replicas: u32,
        pod: PodPlan,
        claim: PersistentVolumeClaimPlan,
    ) -> Self {
        Self {
            name: name.into(),
            service_name: service_name.into(),
            replicas,
            pod_management_policy: "Parallel".into(),
            pod,
            claim,
        }
    }

    fn render(self, cx: &RenderCtx<'_>) -> Result<Value, WorkloadPlanError> {
        if self.replicas == 0 {
            return Err(WorkloadPlanError::ZeroReplicas(self.name));
        }
        let selector = self.pod.selector(cx);
        let component = self.pod.component.clone();
        let mut template = self.pod.render(cx);
        attach_claim(
            &mut template,
            &self.claim.name,
            None,
            &self.claim.mount_path,
        );
        Ok(json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": cx.meta(&self.name, &component),
            "spec": {
                "serviceName": self.service_name,
                "replicas": self.replicas,
                "podManagementPolicy": self.pod_management_policy,
                "selector": {"matchLabels": selector},
                "template": template,
                "volumeClaimTemplates": [self.claim.template()],
            },
        }))
    }
}

#[derive(Clone, Debug)]
pub struct DeploymentPlan {
    pub name: String,
    pub replicas: u32,
    pub strategy: Option<Value>,
    pub pod: PodPlan,
    pub claim: Option<PersistentVolumeClaimPlan>,
}

impl DeploymentPlan {
    pub fn new(name: impl Into<String>, replicas: u32, pod: PodPlan) -> Self {
        Self {
            name: name.into(),
            replicas,
            strategy: None,
            pod,
            claim: None,
        }
    }

    pub fn with_persistent_claim(mut self, claim: PersistentVolumeClaimPlan) -> Self {
        self.claim = Some(claim);
        self
    }

    fn render(self, cx: &RenderCtx<'_>) -> Result<Vec<Value>, WorkloadPlanError> {
        if self.replicas == 0 {
            return Err(WorkloadPlanError::ZeroReplicas(self.name));
        }
        let selector = self.pod.selector(cx);
        let component = self.pod.component.clone();
        let mut template = self.pod.render(cx);
        let mut objects = Vec::new();
        if let Some(claim) = self.claim {
            objects.push(claim.standalone(cx));
            attach_claim(&mut template, "data", Some(&claim.name), &claim.mount_path);
        }
        let mut spec = json!({
            "replicas": self.replicas,
            "selector": {"matchLabels": selector},
            "template": template,
        });
        if let Some(strategy) = self.strategy {
            spec["strategy"] = strategy;
        }
        objects.push(json!({
            "apiVersion": "apps/v1",
            "kind": "Deployment",
            "metadata": cx.meta(&self.name, &component),
            "spec": spec,
        }));
        Ok(objects)
    }
}

#[derive(Clone, Debug)]
pub struct DaemonSetPlan {
    pub name: String,
    pub pod: PodPlan,
}

impl DaemonSetPlan {
    pub fn new(name: impl Into<String>, pod: PodPlan) -> Self {
        Self {
            name: name.into(),
            pod,
        }
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let selector = self.pod.selector(cx);
        let component = self.pod.component.clone();
        json!({
            "apiVersion": "apps/v1",
            "kind": "DaemonSet",
            "metadata": cx.meta(&self.name, &component),
            "spec": {
                "selector": {"matchLabels": selector},
                "template": self.pod.render(cx),
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct PodDisruptionBudgetPlan {
    pub name: String,
    pub component: String,
    pub selector: LabelSet,
    pub min_available: u32,
}

impl PodDisruptionBudgetPlan {
    pub fn min_available(
        name: impl Into<String>,
        component: impl Into<String>,
        selector: LabelSet,
        min_available: u32,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            selector,
            min_available,
        }
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let selector = merge_string_labels(cx.selector(&self.component), &self.selector);
        json!({
            "apiVersion": "policy/v1",
            "kind": "PodDisruptionBudget",
            "metadata": cx.meta(&self.name, &self.component),
            "spec": {"minAvailable": self.min_available, "selector": {"matchLabels": selector}},
        })
    }
}

#[derive(Clone, Debug)]
pub struct RbacRulePlan {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub resource_names: Vec<String>,
    pub verbs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct RolePlan {
    pub name: String,
    pub component: String,
    pub rules: Vec<RbacRulePlan>,
}

impl RolePlan {
    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let rules = self
            .rules
            .into_iter()
            .map(|rule| {
                let mut value = json!({
                    "apiGroups": rule.api_groups,
                    "resources": rule.resources,
                    "verbs": rule.verbs,
                });
                if !rule.resource_names.is_empty() {
                    value["resourceNames"] = json!(rule.resource_names);
                }
                value
            })
            .collect::<Vec<_>>();
        json!({
            "apiVersion": "rbac.authorization.k8s.io/v1",
            "kind": "Role",
            "metadata": cx.meta(&self.name, &self.component),
            "rules": rules,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ServiceAccountSubjectPlan {
    pub name: String,
    pub namespace: String,
}

impl ServiceAccountSubjectPlan {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            namespace: namespace.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoleBindingPlan {
    pub name: String,
    pub component: String,
    pub role_name: String,
    pub subjects: Vec<ServiceAccountSubjectPlan>,
}

impl RoleBindingPlan {
    fn render(self, cx: &RenderCtx<'_>) -> Value {
        json!({
            "apiVersion": "rbac.authorization.k8s.io/v1",
            "kind": "RoleBinding",
            "metadata": cx.meta(&self.name, &self.component),
            "roleRef": {
                "apiGroup": "rbac.authorization.k8s.io",
                "kind": "Role",
                "name": self.role_name,
            },
            "subjects": self.subjects.into_iter().map(|subject| json!({
                "kind": "ServiceAccount",
                "name": subject.name,
                "namespace": subject.namespace,
            })).collect::<Vec<_>>(),
        })
    }
}

/// A cluster-scoped binding to an existing ClusterRole.
///
/// Kubernetes does not allow a namespaced custom resource to own a
/// cluster-scoped child. This plan therefore renders labels, but never a
/// namespace or owner reference. Callers must include strong owner labels and
/// return the same object from [`crate::service::ManagedService::cluster_scoped_children`].
/// The shared controller then installs a finalizer and controls cleanup.
#[derive(Clone, Debug)]
pub struct ClusterRoleBindingPlan {
    pub name: String,
    pub component: String,
    pub cluster_role: String,
    pub subjects: Vec<ServiceAccountSubjectPlan>,
    pub labels: LabelSet,
}

impl ClusterRoleBindingPlan {
    pub fn new(
        name: impl Into<String>,
        component: impl Into<String>,
        cluster_role: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            cluster_role: cluster_role.into(),
            subjects: Vec::new(),
            labels: LabelSet::new(),
        }
    }

    pub fn with_service_account(mut self, subject: ServiceAccountSubjectPlan) -> Self {
        self.subjects.push(subject);
        self
    }

    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let mut labels = cx.labels(&self.component);
        let label_object = labels
            .as_object_mut()
            .expect("RenderCtx::labels always returns a JSON object");
        for (key, value) in self.labels {
            label_object.insert(key, Value::String(value));
        }
        let subjects = self
            .subjects
            .iter()
            .map(|subject| rbac::ServiceAccountSubject {
                namespace: &subject.namespace,
                name: &subject.name,
            })
            .collect::<Vec<_>>();
        rbac::cluster_role_binding(rbac::ClusterRoleBinding {
            name: &self.name,
            labels,
            cluster_role: &self.cluster_role,
            subjects: &subjects,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CronJobPlan {
    pub name: String,
    pub schedule: String,
    pub successful_jobs_history_limit: i32,
    pub failed_jobs_history_limit: i32,
    pub pod: PodPlan,
}

impl CronJobPlan {
    fn render(mut self, cx: &RenderCtx<'_>) -> Value {
        self.pod.runtime.restart_policy = Some("OnFailure".into());
        let component = self.pod.component.clone();
        json!({
            "apiVersion": "batch/v1",
            "kind": "CronJob",
            "metadata": cx.meta(&self.name, &component),
            "spec": {
                "schedule": self.schedule,
                "concurrencyPolicy": "Forbid",
                "successfulJobsHistoryLimit": self.successful_jobs_history_limit,
                "failedJobsHistoryLimit": self.failed_jobs_history_limit,
                "jobTemplate": {"spec": {"template": self.pod.render(cx)}},
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct NetworkPortPlan {
    pub protocol: String,
    pub port: i32,
}

impl NetworkPortPlan {
    pub fn tcp(port: i32) -> Self {
        Self {
            protocol: "TCP".into(),
            port,
        }
    }

    pub fn udp(port: i32) -> Self {
        Self {
            protocol: "UDP".into(),
            port,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NetworkPeerPlan {
    Any,
    SameNamespace,
    Pods {
        namespace: Option<String>,
        selector: LabelSet,
    },
    IpBlock {
        cidr: String,
        except: Vec<String>,
    },
}

impl NetworkPeerPlan {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn same_namespace() -> Self {
        Self::SameNamespace
    }

    pub fn same_namespace_pods(selector: LabelSet) -> Self {
        Self::Pods {
            namespace: None,
            selector,
        }
    }

    pub fn pods_in_namespace(namespace: impl Into<String>, selector: LabelSet) -> Self {
        Self::Pods {
            namespace: Some(namespace.into()),
            selector,
        }
    }

    pub fn ip_block(cidr: impl Into<String>) -> Self {
        Self::IpBlock {
            cidr: cidr.into(),
            except: Vec::new(),
        }
    }

    pub fn with_except(mut self, cidr: impl Into<String>) -> Self {
        if let Self::IpBlock { except, .. } = &mut self {
            except.push(cidr.into());
        }
        self
    }

    fn render(&self, namespace: &str) -> Value {
        match self {
            Self::Any => json!({}),
            Self::SameNamespace => json!({
                "namespaceSelector": {"matchLabels": {"kubernetes.io/metadata.name": namespace}},
            }),
            Self::Pods {
                namespace: selected_namespace,
                selector,
            } => json!({
                "namespaceSelector": {"matchLabels": {
                    "kubernetes.io/metadata.name": selected_namespace.as_deref().unwrap_or(namespace)
                }},
                "podSelector": {"matchLabels": selector},
            }),
            Self::IpBlock { cidr, except } => {
                let mut block = json!({"cidr": cidr});
                if !except.is_empty() {
                    block["except"] = json!(except);
                }
                json!({"ipBlock": block})
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkRulePlan {
    pub peers: Vec<NetworkPeerPlan>,
    pub ports: Vec<NetworkPortPlan>,
}

impl NetworkRulePlan {
    pub fn new(peers: Vec<NetworkPeerPlan>, ports: Vec<NetworkPortPlan>) -> Self {
        Self { peers, ports }
    }

    fn render(&self, direction: &str, namespace: &str) -> Value {
        let unrestricted = self
            .peers
            .iter()
            .any(|peer| matches!(peer, NetworkPeerPlan::Any));
        let ports = self
            .ports
            .iter()
            .map(|port| json!({"protocol": port.protocol, "port": port.port}))
            .collect::<Vec<_>>();
        let mut value = json!({"ports": ports});
        if !unrestricted {
            value[direction] = json!(self
                .peers
                .iter()
                .map(|peer| peer.render(namespace))
                .collect::<Vec<_>>());
        }
        value
    }
}

#[derive(Clone, Debug)]
pub enum FqdnMatchPlan {
    Name(String),
    Pattern(String),
}

impl FqdnMatchPlan {
    pub fn name(value: impl Into<String>) -> Self {
        Self::Name(value.into())
    }

    pub fn pattern(value: impl Into<String>) -> Self {
        Self::Pattern(value.into())
    }

    fn render(self) -> Value {
        match self {
            Self::Name(name) => json!({"name": name}),
            Self::Pattern(pattern) => json!({"pattern": pattern}),
        }
    }
}

/// GKE Dataplane V2 external-egress allowlist by DNS name.
#[derive(Clone, Debug)]
pub struct FqdnNetworkPolicyPlan {
    pub name: String,
    pub component: String,
    pub selector: LabelSet,
    pub matches: Vec<FqdnMatchPlan>,
    pub ports: Vec<NetworkPortPlan>,
}

impl FqdnNetworkPolicyPlan {
    pub fn new(name: impl Into<String>, component: impl Into<String>, selector: LabelSet) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            selector,
            matches: Vec::new(),
            ports: Vec::new(),
        }
    }

    pub fn with_match(mut self, matcher: FqdnMatchPlan) -> Self {
        self.matches.push(matcher);
        self
    }

    pub fn with_port(mut self, port: NetworkPortPlan) -> Self {
        self.ports.push(port);
        self
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        json!({
            "apiVersion": "networking.gke.io/v1alpha1",
            "kind": "FQDNNetworkPolicy",
            "metadata": cx.meta(&self.name, &self.component),
            "spec": {
                "podSelector": {"matchLabels": self.selector},
                "egress": [{
                    "matches": self.matches.into_iter().map(FqdnMatchPlan::render).collect::<Vec<_>>(),
                    "ports": self.ports.into_iter().map(|port| json!({
                        "protocol": port.protocol,
                        "port": port.port,
                    })).collect::<Vec<_>>(),
                }],
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct NetworkPolicyPlan {
    pub name: String,
    pub component: String,
    pub selector: LabelSet,
    pub ingress: Vec<NetworkRulePlan>,
    pub egress: Vec<NetworkRulePlan>,
    pub instance_wide: bool,
}

impl NetworkPolicyPlan {
    pub fn new(name: impl Into<String>, component: impl Into<String>, selector: LabelSet) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            selector,
            ingress: Vec::new(),
            egress: Vec::new(),
            instance_wide: false,
        }
    }

    pub fn with_ingress(mut self, rule: NetworkRulePlan) -> Self {
        self.ingress.push(rule);
        self
    }

    pub fn with_egress(mut self, rule: NetworkRulePlan) -> Self {
        self.egress.push(rule);
        self
    }

    /// Select every pod in this managed instance. This is used only for the
    /// base default-deny policy. Role policies should keep their narrow
    /// component selector.
    pub fn instance_wide(mut self) -> Self {
        self.instance_wide = true;
        self
    }

    fn render(self, cx: &RenderCtx<'_>) -> Value {
        let selector = if self.instance_wide {
            merge_string_labels(
                json!({
                    "app.kubernetes.io/name": cx.app,
                    "app.kubernetes.io/instance": cx.name,
                }),
                &self.selector,
            )
        } else {
            merge_string_labels(cx.selector(&self.component), &self.selector)
        };
        let ingress = self
            .ingress
            .iter()
            .map(|rule| rule.render("from", cx.ns))
            .collect::<Vec<_>>();
        let egress = self
            .egress
            .iter()
            .map(|rule| rule.render("to", cx.ns))
            .collect::<Vec<_>>();
        json!({
            "apiVersion": "networking.k8s.io/v1",
            "kind": "NetworkPolicy",
            "metadata": cx.meta(&self.name, &self.component),
            "spec": {
                "podSelector": {"matchLabels": selector},
                "policyTypes": ["Ingress", "Egress"],
                "ingress": ingress,
                "egress": egress,
            },
        })
    }
}

enum PlannedObject {
    ServiceAccount(ServiceAccountPlan),
    Service(ServicePlan),
    StatefulSet(StatefulSetPlan),
    Deployment(DeploymentPlan),
    DaemonSet(DaemonSetPlan),
    PodDisruptionBudget(PodDisruptionBudgetPlan),
    Role(RolePlan),
    RoleBinding(RoleBindingPlan),
    ClusterRoleBinding(ClusterRoleBindingPlan),
    CronJob(CronJobPlan),
    NetworkPolicy(NetworkPolicyPlan),
    FqdnNetworkPolicy(FqdnNetworkPolicyPlan),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkloadPlanError {
    #[error("workload {0} must have at least one replica")]
    ZeroReplicas(String),
}

/// One ordered set of typed Kubernetes children for a managed service.
pub struct WorkloadPlan<'a> {
    cx: &'a RenderCtx<'a>,
    objects: Vec<PlannedObject>,
}

impl<'a> WorkloadPlan<'a> {
    pub fn new(cx: &'a RenderCtx<'a>) -> Self {
        Self {
            cx,
            objects: Vec::new(),
        }
    }

    pub fn add_service_account(&mut self, plan: ServiceAccountPlan) {
        self.objects.push(PlannedObject::ServiceAccount(plan));
    }

    pub fn add_service(&mut self, plan: ServicePlan) {
        self.objects.push(PlannedObject::Service(plan));
    }

    pub fn add_stateful_set(&mut self, plan: StatefulSetPlan) {
        self.objects.push(PlannedObject::StatefulSet(plan));
    }

    pub fn add_deployment(&mut self, plan: DeploymentPlan) {
        self.objects.push(PlannedObject::Deployment(plan));
    }

    pub fn add_daemon_set(&mut self, plan: DaemonSetPlan) {
        self.objects.push(PlannedObject::DaemonSet(plan));
    }

    pub fn add_pod_disruption_budget(&mut self, plan: PodDisruptionBudgetPlan) {
        self.objects.push(PlannedObject::PodDisruptionBudget(plan));
    }

    pub fn add_role(&mut self, plan: RolePlan) {
        self.objects.push(PlannedObject::Role(plan));
    }

    pub fn add_role_binding(&mut self, plan: RoleBindingPlan) {
        self.objects.push(PlannedObject::RoleBinding(plan));
    }

    pub fn add_cluster_role_binding(&mut self, plan: ClusterRoleBindingPlan) {
        self.objects.push(PlannedObject::ClusterRoleBinding(plan));
    }

    pub fn add_cron_job(&mut self, plan: CronJobPlan) {
        self.objects.push(PlannedObject::CronJob(plan));
    }

    pub fn add_network_policy(&mut self, plan: NetworkPolicyPlan) {
        self.objects.push(PlannedObject::NetworkPolicy(plan));
    }

    pub fn add_fqdn_network_policy(&mut self, plan: FqdnNetworkPolicyPlan) {
        self.objects.push(PlannedObject::FqdnNetworkPolicy(plan));
    }

    pub fn render(self) -> Result<Vec<Value>, WorkloadPlanError> {
        let mut rendered = Vec::new();
        for object in self.objects {
            match object {
                PlannedObject::ServiceAccount(plan) => rendered.push(json!({
                    "apiVersion": "v1",
                    "kind": "ServiceAccount",
                    "metadata": self.cx.meta(&plan.name, &plan.component),
                    "automountServiceAccountToken": plan.automount_service_account_token,
                })),
                PlannedObject::Service(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::StatefulSet(plan) => rendered.push(plan.render(self.cx)?),
                PlannedObject::Deployment(plan) => rendered.extend(plan.render(self.cx)?),
                PlannedObject::DaemonSet(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::PodDisruptionBudget(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::Role(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::RoleBinding(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::ClusterRoleBinding(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::CronJob(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::NetworkPolicy(plan) => rendered.push(plan.render(self.cx)),
                PlannedObject::FqdnNetworkPolicy(plan) => rendered.push(plan.render(self.cx)),
            }
        }
        Ok(rendered)
    }
}

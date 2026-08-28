//! Service-neutral rendering for one durable StatefulSet instance.
//!
//! This module contains no service policy.  Callers provide the identity and
//! an already composed pod template; the renderer only attaches storage and
//! emits the Kubernetes StatefulSet shape.  Keeping this seam in the shared
//! kit lets Standalone and Managed/Fleet use the same stateful contract.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use thiserror::Error;

use super::{
    common::ServicePodTemplate, RenderCtx, ServiceStatefulSet, ENV_POD_NAME, ENV_POD_NAMESPACE,
    ENV_REPLICAS_PER_SHARD, ENV_SHARD_COUNT, ENV_VOTER_COUNT,
};

/// Compatibility adapter for the historical render root.  The root API stays
/// source-compatible while the stateful shape is assembled by this module.
pub fn render_compat_service_statefulset(p: ServiceStatefulSet) -> Value {
    let ServiceStatefulSet {
        cx,
        name,
        component,
        image,
        image_pull_policy,
        command,
        args,
        ports,
        headless_service,
        shard_count,
        replicas_per_shard,
        voter_count,
        headless_env_key,
        service_account_name,
        env: extra_env,
        env_from,
        resources,
        pod_annotations,
        pod_security_context,
        container_security_context,
        termination_grace_period_seconds,
        readiness_probe,
        liveness_probe,
        startup_probe,
        lifecycle,
        volumes,
        volume_mounts,
        affinity,
        node_selector,
        tolerations,
        topology_spread_constraints,
        revision_history_limit,
        update_strategy,
        volume_claim,
    } = p;

    let mut env = vec![
        json!({ "name": ENV_POD_NAME, "valueFrom": { "fieldRef": { "fieldPath": "metadata.name" } } }),
        json!({ "name": ENV_POD_NAMESPACE, "valueFrom": { "fieldRef": { "fieldPath": "metadata.namespace" } } }),
        json!({ "name": ENV_SHARD_COUNT, "value": shard_count.to_string() }),
        json!({ "name": ENV_REPLICAS_PER_SHARD, "value": replicas_per_shard.to_string() }),
        json!({ "name": ENV_VOTER_COUNT, "value": voter_count.to_string() }),
        json!({ "name": headless_env_key, "value": headless_service }),
    ];
    env.extend(extra_env);

    let pod = ServicePodTemplate {
        cx,
        component,
        image,
        image_pull_policy,
        command,
        args,
        ports,
        env,
        env_from,
        resources,
        readiness_probe,
        liveness_probe,
        startup_probe,
        lifecycle,
        container_security_context,
        pod_security_context,
        service_account_name,
        termination_grace_period_seconds,
        volumes,
        volume_mounts,
        pod_annotations,
        topology_spread_constraints: vec![],
    };
    let storage = volume_claim.map(|claim| {
        StatefulStorageAttachment::VolumeClaimTemplate(VolumeClaimTemplate {
            name: claim.name,
            template: claim.template,
            mount_path: claim.mount_path.to_owned(),
            read_only: claim.read_only,
        })
    });
    let mut plan = StatefulInstancePlan::without_storage(
        cx,
        headless_service,
        shard_count * replicas_per_shard,
        pod,
    );
    plan.name = name.into();
    plan.storage = storage;
    plan.affinity = affinity;
    plan.node_selector = node_selector;
    plan.tolerations = tolerations;
    plan.topology_spread_constraints = topology_spread_constraints;
    plan.revision_history_limit = revision_history_limit;
    plan.update_strategy = update_strategy;
    let rendered = stateful_instance(plan).expect("validated compatibility stateful plan");
    rendered.workload
}

/// A StatefulSet-managed PVC template and the mount that consumes it.
#[derive(Clone, Debug, PartialEq)]
pub struct VolumeClaimTemplate {
    pub name: String,
    pub template: Value,
    pub mount_path: String,
    pub read_only: bool,
}

impl VolumeClaimTemplate {
    pub fn new(name: impl Into<String>, template: Value, mount_path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            template,
            mount_path: mount_path.into(),
            read_only: false,
        }
    }
}

/// An independently rendered PVC and the pod volume that consumes it.
#[derive(Clone, Debug, PartialEq)]
pub struct ExistingClaim {
    pub volume_name: String,
    pub claim_name: String,
    pub template: Value,
    pub mount_path: String,
    pub read_only: bool,
}

impl ExistingClaim {
    pub fn new(
        volume_name: impl Into<String>,
        claim_name: impl Into<String>,
        template: Value,
        mount_path: impl Into<String>,
    ) -> Self {
        Self {
            volume_name: volume_name.into(),
            claim_name: claim_name.into(),
            template,
            mount_path: mount_path.into(),
            read_only: false,
        }
    }
}

/// How the instance obtains its durable volume.
#[derive(Clone, Debug, PartialEq)]
pub enum StatefulStorageAttachment {
    /// Kubernetes creates one claim per StatefulSet member.
    VolumeClaimTemplate(VolumeClaimTemplate),
    /// Kubernetes binds the pod to an independently managed claim.
    ExistingClaim(ExistingClaim),
}

/// Input to [`stateful_instance`].  `pod_template` is the complete
/// `spec.template` object, so service-specific image, ports, probes, security
/// and environment remain owned by the service.
pub struct StatefulInstancePlan<'a> {
    pub cx: &'a RenderCtx<'a>,
    /// The StatefulSet metadata name. The context name remains the instance
    /// identity used in labels and owner metadata.
    pub name: String,
    pub service_name: String,
    pub replicas: u32,
    pub selector: BTreeMap<String, String>,
    pub labels: BTreeMap<String, String>,
    pub pod: ServicePodTemplate<'a>,
    /// No attachment leaves the workload without a PVC, mount, or volume.
    pub storage: Option<StatefulStorageAttachment>,
    pub pod_management_policy: Option<String>,
    pub affinity: Option<Value>,
    pub node_selector: Option<Value>,
    pub tolerations: Vec<Value>,
    pub topology_spread_constraints: Vec<Value>,
    pub revision_history_limit: Option<i32>,
    pub update_strategy: Option<Value>,
}

impl<'a> StatefulInstancePlan<'a> {
    pub fn new(
        cx: &'a RenderCtx<'a>,
        service_name: impl Into<String>,
        replicas: u32,
        pod: ServicePodTemplate<'a>,
        storage: StatefulStorageAttachment,
    ) -> Self {
        Self::with_optional_storage(cx, service_name, replicas, pod, Some(storage))
    }

    /// Build an instance with no durable attachment.
    pub fn without_storage(
        cx: &'a RenderCtx<'a>,
        service_name: impl Into<String>,
        replicas: u32,
        pod: ServicePodTemplate<'a>,
    ) -> Self {
        Self::with_optional_storage(cx, service_name, replicas, pod, None)
    }

    fn with_optional_storage(
        cx: &'a RenderCtx<'a>,
        service_name: impl Into<String>,
        replicas: u32,
        pod: ServicePodTemplate<'a>,
        storage: Option<StatefulStorageAttachment>,
    ) -> Self {
        Self {
            cx,
            name: cx.name.into(),
            service_name: service_name.into(),
            replicas,
            selector: BTreeMap::new(),
            labels: BTreeMap::new(),
            pod,
            storage,
            pod_management_policy: Some("Parallel".into()),
            affinity: None,
            node_selector: None,
            tolerations: Vec::new(),
            topology_spread_constraints: Vec::new(),
            revision_history_limit: None,
            update_strategy: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatefulInstanceRender {
    /// Present only for [`StatefulStorageAttachment::ExistingClaim`]. Apply it
    /// before [`Self::workload`] so the pod volume has a claim to bind.
    pub storage: Option<Value>,
    /// The StatefulSet workload, rendered after optional independent storage.
    pub workload: Value,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StatefulInstanceError {
    #[error("replicas must be greater than zero")]
    ZeroReplicas,
    #[error("identity must not be empty")]
    EmptyIdentity,
    #[error("{0} must not be empty")]
    EmptyField(&'static str),
    #[error("volume and mount names must be unique")]
    VolumeMountCollision,
    #[error("selector and core identity labels are immutable")]
    SelectorCoreIdentityOverride,
    #[error("pod template must be an object")]
    InvalidPodTemplate,
}

fn nonempty(value: &str, field: &'static str) -> Result<(), StatefulInstanceError> {
    if value.trim().is_empty() {
        Err(StatefulInstanceError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn merge_extra_labels(
    target: &mut Value,
    extra: &BTreeMap<String, String>,
    core: &Value,
) -> Result<(), StatefulInstanceError> {
    let target = target.as_object_mut().expect("labels are an object");
    let core = core.as_object().expect("RenderCtx labels are an object");
    for (key, value) in extra {
        if let Some(expected) = core.get(key) {
            if expected != value {
                return Err(StatefulInstanceError::SelectorCoreIdentityOverride);
            }
            continue;
        }
        target.insert(key.clone(), json!(value));
    }
    Ok(())
}

fn merge_extra_selector(
    target: &mut Value,
    extra: &BTreeMap<String, String>,
    labels: &Value,
) -> Result<(), StatefulInstanceError> {
    let target = target.as_object_mut().expect("selector is an object");
    let labels = labels.as_object().expect("RenderCtx labels are an object");
    for (key, value) in extra {
        if let Some(expected) = labels.get(key) {
            if expected != value {
                return Err(StatefulInstanceError::SelectorCoreIdentityOverride);
            }
            continue;
        }
        target.insert(key.clone(), json!(value));
    }
    Ok(())
}

fn claim_template(
    mut template: Value,
    claim: &VolumeClaimTemplate,
    labels: &Value,
) -> Result<Value, StatefulInstanceError> {
    let object = template
        .as_object_mut()
        .ok_or(StatefulInstanceError::EmptyField("claim template"))?;
    let metadata = object.entry("metadata").or_insert_with(|| json!({}));
    let metadata = metadata
        .as_object_mut()
        .ok_or(StatefulInstanceError::EmptyField("claim template metadata"))?;
    metadata.insert("name".into(), json!(claim.name));
    let template_labels = metadata.entry("labels").or_insert_with(|| json!({}));
    if !template_labels.is_object() {
        return Err(StatefulInstanceError::EmptyField("claim template labels"));
    }
    merge_extra_labels(template_labels, &BTreeMap::new(), labels)?;
    for (key, value) in labels.as_object().expect("RenderCtx labels are an object") {
        match template_labels.get(key) {
            Some(existing) if existing != value => {
                return Err(StatefulInstanceError::SelectorCoreIdentityOverride)
            }
            Some(_) => {}
            None => {
                template_labels
                    .as_object_mut()
                    .expect("labels are an object")
                    .insert(key.clone(), value.clone());
            }
        }
    }
    Ok(template)
}

/// Render one deterministic StatefulSet and, for an independent claim, its
/// PVC.  The returned independent PVC is ready before the StatefulSet so a
/// caller can apply storage before the workload.
pub fn stateful_instance(
    plan: StatefulInstancePlan<'_>,
) -> Result<StatefulInstanceRender, StatefulInstanceError> {
    if plan.replicas == 0 {
        return Err(StatefulInstanceError::ZeroReplicas);
    }
    if plan.cx.app.trim().is_empty() {
        return Err(StatefulInstanceError::EmptyIdentity);
    }
    nonempty(plan.cx.name, "name")?;
    nonempty(plan.cx.ns, "namespace")?;
    nonempty(plan.cx.app, "identity")?;
    nonempty(&plan.service_name, "service_name")?;

    nonempty(&plan.name, "statefulset name")?;
    if let Some(storage) = &plan.storage {
        match storage {
            StatefulStorageAttachment::VolumeClaimTemplate(claim) => {
                nonempty(&claim.name, "claim template name")?;
                nonempty(&claim.mount_path, "mount path")?;
                if !claim.template.is_object() {
                    return Err(StatefulInstanceError::EmptyField("claim template"));
                }
            }
            StatefulStorageAttachment::ExistingClaim(claim) => {
                nonempty(&claim.claim_name, "claim name")?;
                nonempty(&claim.volume_name, "volume name")?;
                nonempty(&claim.mount_path, "mount path")?;
                if !claim.template.is_object() {
                    return Err(StatefulInstanceError::EmptyField("claim template"));
                }
            }
        }
    }

    let mut labels = plan.cx.labels(plan.pod.component);
    merge_extra_labels(
        &mut labels,
        &plan.labels,
        &plan.cx.labels(plan.pod.component),
    )?;
    let mut selector = plan.cx.selector(plan.pod.component);
    merge_extra_selector(
        &mut selector,
        &plan.selector,
        &plan.cx.labels(plan.pod.component),
    )?;
    let core_labels = plan.cx.labels(plan.pod.component);
    for (key, value) in &plan.selector {
        if core_labels.get(key).is_none() {
            labels[key] = json!(value);
        }
    }
    let component = plan.pod.component;
    let topology_spread_constraints = if plan.topology_spread_constraints.is_empty() {
        plan.pod.topology_spread_constraints.clone()
    } else {
        plan.topology_spread_constraints.clone()
    };
    let mut template = plan.pod.render();
    let template_obj = template.as_object_mut().unwrap();
    let metadata = template_obj.entry("metadata").or_insert_with(|| json!({}));
    let metadata = metadata
        .as_object_mut()
        .ok_or(StatefulInstanceError::InvalidPodTemplate)?;
    metadata.insert("labels".into(), labels.clone());
    let spec = template_obj.entry("spec").or_insert_with(|| json!({}));
    let spec = spec
        .as_object_mut()
        .ok_or(StatefulInstanceError::InvalidPodTemplate)?;
    // `ServicePodTemplate` can carry a spread constraint for Deployment. Move
    // it to the StatefulSet ordering point after placement fields so the
    // source-compatible adapter keeps its historical serialized order.
    spec.remove("topologySpreadConstraints");
    let containers = spec.entry("containers").or_insert_with(|| json!([]));
    let containers = containers
        .as_array_mut()
        .ok_or(StatefulInstanceError::InvalidPodTemplate)?;
    if containers.is_empty() {
        return Err(StatefulInstanceError::EmptyField("containers"));
    }
    let container = containers[0]
        .as_object_mut()
        .ok_or(StatefulInstanceError::InvalidPodTemplate)?;
    if let Some(storage) = &plan.storage {
        let (volume_name, mount_path, read_only) = match storage {
            StatefulStorageAttachment::VolumeClaimTemplate(claim) => {
                (&claim.name, &claim.mount_path, claim.read_only)
            }
            StatefulStorageAttachment::ExistingClaim(claim) => {
                (&claim.volume_name, &claim.mount_path, claim.read_only)
            }
        };
        let mounts = container.entry("volumeMounts").or_insert_with(|| json!([]));
        let mounts = mounts
            .as_array_mut()
            .ok_or(StatefulInstanceError::InvalidPodTemplate)?;
        if mounts
            .iter()
            .any(|mount| mount.get("name").and_then(Value::as_str) == Some(volume_name.as_str()))
            || mounts.iter().any(|mount| {
                mount.get("mountPath").and_then(Value::as_str) == Some(mount_path.as_str())
            })
        {
            return Err(StatefulInstanceError::VolumeMountCollision);
        }
        mounts.push(json!({ "name": volume_name, "mountPath": mount_path, "readOnly": read_only }));
    }

    let mut sts_spec = json!({
        "replicas": plan.replicas,
        "serviceName": plan.service_name,
        "podManagementPolicy": "Parallel",
        "selector": { "matchLabels": selector },
        "template": template,
    });
    if let Some(policy) = plan.pod_management_policy {
        sts_spec["podManagementPolicy"] = json!(policy);
    }
    if let Some(value) = plan.affinity {
        sts_spec["template"]["spec"]["affinity"] = value;
    }
    if let Some(value) = plan.node_selector {
        sts_spec["template"]["spec"]["nodeSelector"] = value;
    }
    if !plan.tolerations.is_empty() {
        sts_spec["template"]["spec"]["tolerations"] = json!(plan.tolerations);
    }
    if !topology_spread_constraints.is_empty() {
        sts_spec["template"]["spec"]["topologySpreadConstraints"] =
            json!(topology_spread_constraints);
    }
    if let Some(value) = plan.revision_history_limit {
        sts_spec["revisionHistoryLimit"] = json!(value);
    }
    if let Some(value) = plan.update_strategy {
        sts_spec["updateStrategy"] = value;
    }
    let mut pvc = None;
    match &plan.storage {
        Some(StatefulStorageAttachment::VolumeClaimTemplate(claim)) => {
            sts_spec["volumeClaimTemplates"] =
                json!([claim_template(claim.template.clone(), claim, &labels,)?]);
        }
        Some(StatefulStorageAttachment::ExistingClaim(claim)) => {
            let volumes = sts_spec["template"]["spec"]
                .get_mut("volumes")
                .and_then(Value::as_array_mut);
            let mut volumes = volumes.cloned().unwrap_or_default();
            if volumes.iter().any(|volume| {
                volume.get("name").and_then(Value::as_str) == Some(claim.volume_name.as_str())
            }) {
                return Err(StatefulInstanceError::VolumeMountCollision);
            }
            volumes.push(json!({ "name": claim.volume_name, "persistentVolumeClaim": { "claimName": claim.claim_name } }));
            sts_spec["template"]["spec"]["volumes"] = json!(volumes);
            let mut p = claim.template.clone();
            p["metadata"] = plan.cx.meta(&claim.claim_name, component);
            p["metadata"]["labels"] = labels.clone();
            pvc = Some(
                json!({"apiVersion":"v1","kind":"PersistentVolumeClaim","metadata":p["metadata"].clone(),"spec":p["spec"].clone()}),
            );
        }
        None => {}
    }
    let mut metadata = plan.cx.meta(&plan.name, component);
    metadata["labels"] = labels;
    Ok(StatefulInstanceRender {
        storage: pvc,
        workload: json!({"apiVersion": "apps/v1", "kind": "StatefulSet", "metadata": metadata, "spec": sts_spec}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(storage: StatefulStorageAttachment) -> StatefulInstancePlan<'static> {
        let cx = Box::leak(Box::new(RenderCtx {
            app: "lumen",
            manager: "test",
            api_version: "v1",
            kind: "Lumen",
            name: "lumen",
            ns: "lumen",
            owner: None,
        }));
        let pod = ServicePodTemplate {
            cx,
            component: "serving",
            image: "lumen",
            image_pull_policy: "IfNotPresent",
            command: vec!["lumen".into()],
            args: vec![],
            ports: vec![],
            env: vec![],
            env_from: vec![],
            resources: json!({}),
            readiness_probe: None,
            liveness_probe: None,
            startup_probe: None,
            lifecycle: None,
            container_security_context: None,
            pod_security_context: None,
            service_account_name: None,
            termination_grace_period_seconds: None,
            volumes: vec![],
            volume_mounts: vec![],
            pod_annotations: None,
            topology_spread_constraints: vec![],
        };
        StatefulInstancePlan::new(cx, "lumen", 1, pod, storage)
    }

    fn template_claim() -> VolumeClaimTemplate {
        VolumeClaimTemplate::new(
            "data",
            json!({"spec": {"resources": {"requests": {"storage": "20Gi"}}}}),
            "/data",
        )
    }

    fn existing_claim() -> ExistingClaim {
        ExistingClaim::new(
            "data",
            "data",
            json!({"spec": {"resources": {"requests": {"storage": "20Gi"}}}}),
            "/data",
        )
    }

    #[test]
    fn template_attachment_is_rendered_by_the_statefulset() {
        let rendered = stateful_instance(plan(StatefulStorageAttachment::VolumeClaimTemplate(
            template_claim(),
        )))
        .unwrap();
        assert!(rendered.storage.is_none());
        assert_eq!(
            rendered.workload["spec"]["volumeClaimTemplates"][0]["metadata"]["name"],
            "data"
        );
        assert_eq!(
            rendered.workload["spec"]["template"]["spec"]["containers"][0]["volumeMounts"][0]
                ["mountPath"],
            "/data"
        );
    }

    #[test]
    fn existing_claim_is_an_independent_pvc_and_pod_volume() {
        let rendered = stateful_instance(plan(StatefulStorageAttachment::ExistingClaim(
            existing_claim(),
        )))
        .unwrap();
        assert!(rendered.workload["spec"]["volumeClaimTemplates"].is_null());
        assert_eq!(
            rendered.workload["spec"]["template"]["spec"]["volumes"][0]["persistentVolumeClaim"]
                ["claimName"],
            "data"
        );
        assert_eq!(rendered.storage.unwrap()["metadata"]["name"], "data");
    }

    #[test]
    fn invalid_replica_and_selector_are_rejected() {
        let mut zero = plan(StatefulStorageAttachment::VolumeClaimTemplate(
            template_claim(),
        ));
        zero.replicas = 0;
        assert_eq!(
            stateful_instance(zero),
            Err(StatefulInstanceError::ZeroReplicas)
        );
        let mut override_selector = plan(StatefulStorageAttachment::VolumeClaimTemplate(
            template_claim(),
        ));
        override_selector
            .selector
            .insert("app.kubernetes.io/name".into(), "other".into());
        assert_eq!(
            stateful_instance(override_selector),
            Err(StatefulInstanceError::SelectorCoreIdentityOverride)
        );
    }
}

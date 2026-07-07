// <HANDWRITE gap="issue-1108:base-workload-discovery" tracker="projects-preview-src-discover-rs" reason="Base workload discovery and normalization are hand-authored until Preview has generator primitives for Kubernetes object adapters.">
use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseWorkloadContract {
    pub namespace: String,
    pub app: String,
    pub deployment: String,
    pub service: String,
    pub selector: BTreeMap<String, String>,
    pub pod_labels: BTreeMap<String, String>,
    pub container: BaseContainerContract,
    pub service_ports: Vec<BaseServicePort>,
    pub excluded_runtime_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseContainerContract {
    pub name: String,
    pub image: String,
    pub ports: Vec<BaseContainerPort>,
    pub env: Vec<BaseEnvVar>,
    pub resources: Value,
    pub readiness_path: Option<String>,
    pub liveness_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseContainerPort {
    pub name: Option<String>,
    pub container_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseEnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseServicePort {
    pub name: Option<String>,
    pub port: u16,
    pub target_port: String,
}

pub fn discover_base_with_kubectl(
    namespace: &str,
    app: &str,
    context: Option<&str>,
) -> Result<BaseWorkloadContract> {
    let deployment = kubectl_get_json("deployment", namespace, app, context)?;
    let service = kubectl_get_json("service", namespace, app, context)?;
    normalize_base_workload(&deployment, &service, namespace, app)
}

pub fn normalize_base_workload(
    deployment: &Value,
    service: &Value,
    namespace: &str,
    app: &str,
) -> Result<BaseWorkloadContract> {
    assert_kind(deployment, "Deployment")?;
    assert_kind(service, "Service")?;
    assert_metadata(deployment, namespace, app, "Deployment")?;
    assert_metadata(service, namespace, app, "Service")?;

    let selector = string_map(
        deployment
            .pointer("/spec/selector/matchLabels")
            .ok_or_else(|| {
                anyhow!("Deployment {namespace}/{app} missing spec.selector.matchLabels")
            })?,
    )
    .context("normalize deployment selector")?;
    let pod_labels = string_map(
        deployment
            .pointer("/spec/template/metadata/labels")
            .ok_or_else(|| anyhow!("Deployment {namespace}/{app} missing pod template labels"))?,
    )
    .context("normalize pod labels")?;
    for (key, value) in &selector {
        if pod_labels.get(key) != Some(value) {
            bail!(
                "Deployment {namespace}/{app} selector {key}={value} is not present in pod labels"
            );
        }
    }

    let container = normalize_container(
        deployment
            .pointer("/spec/template/spec/containers")
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("Deployment {namespace}/{app} missing containers"))?,
        app,
    )?;
    let service_ports = normalize_service_ports(service)?;

    Ok(BaseWorkloadContract {
        namespace: namespace.to_string(),
        app: app.to_string(),
        deployment: app.to_string(),
        service: app.to_string(),
        selector,
        pod_labels,
        container,
        service_ports,
        excluded_runtime_fields: vec![
            "metadata.uid".to_string(),
            "metadata.resourceVersion".to_string(),
            "metadata.generation".to_string(),
            "metadata.managedFields".to_string(),
            "metadata.ownerReferences".to_string(),
            "status".to_string(),
            "spec.clusterIP".to_string(),
            "spec.clusterIPs".to_string(),
            "spec.ports[].nodePort".to_string(),
            "status.loadBalancer".to_string(),
            "secrets by default".to_string(),
        ],
    })
}

fn kubectl_get_json(
    kind: &str,
    namespace: &str,
    name: &str,
    context: Option<&str>,
) -> Result<Value> {
    let mut command = Command::new("kubectl");
    if let Some(context) = context {
        command.args(["--context", context]);
    }
    let output = command
        .args(["get", kind, name, "-n", namespace, "-o", "json"])
        .output()
        .with_context(|| format!("start kubectl get {kind} {namespace}/{name}"))?;
    if !output.status.success() {
        bail!(
            "kubectl get {kind} {namespace}/{name} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse kubectl {kind} {namespace}/{name} JSON"))
}

fn assert_kind(value: &Value, expected: &str) -> Result<()> {
    let kind = value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("object missing kind"))?;
    if kind != expected {
        bail!("expected {expected}, got {kind}");
    }
    Ok(())
}

fn assert_metadata(value: &Value, namespace: &str, name: &str, kind: &str) -> Result<()> {
    let actual_namespace = value
        .pointer("/metadata/namespace")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{kind} missing metadata.namespace"))?;
    let actual_name = value
        .pointer("/metadata/name")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("{kind} missing metadata.name"))?;
    if actual_namespace != namespace || actual_name != name {
        bail!(
            "{kind} identity mismatch: expected {namespace}/{name}, got {actual_namespace}/{actual_name}"
        );
    }
    Ok(())
}

fn normalize_container(containers: &[Value], app: &str) -> Result<BaseContainerContract> {
    let selected = containers
        .iter()
        .find(|container| container.get("name").and_then(Value::as_str) == Some(app))
        .or_else(|| (containers.len() == 1).then_some(&containers[0]))
        .ok_or_else(|| {
            anyhow!("ambiguous base Deployment containers; include a container named {app}")
        })?;

    Ok(BaseContainerContract {
        name: selected
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("selected container missing name"))?
            .to_string(),
        image: selected
            .get("image")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("selected container missing image"))?
            .to_string(),
        ports: normalize_container_ports(selected.get("ports").and_then(Value::as_array)),
        env: normalize_env(selected.get("env").and_then(Value::as_array)),
        resources: selected
            .get("resources")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default())),
        readiness_path: probe_path(selected.get("readinessProbe")),
        liveness_path: probe_path(selected.get("livenessProbe")),
    })
}

fn normalize_container_ports(ports: Option<&Vec<Value>>) -> Vec<BaseContainerPort> {
    ports
        .into_iter()
        .flatten()
        .filter_map(|port| {
            Some(BaseContainerPort {
                name: port.get("name").and_then(Value::as_str).map(str::to_string),
                container_port: u16::try_from(port.get("containerPort")?.as_u64()?).ok()?,
            })
        })
        .collect()
}

fn normalize_env(env: Option<&Vec<Value>>) -> Vec<BaseEnvVar> {
    env.into_iter()
        .flatten()
        .filter_map(|var| {
            let value_from_kind = var
                .get("valueFrom")
                .and_then(Value::as_object)
                .and_then(|obj| {
                    obj.keys()
                        .find(|key| key.as_str() != "optional")
                        .map(ToString::to_string)
                });
            Some(BaseEnvVar {
                name: var.get("name")?.as_str()?.to_string(),
                value: var.get("value").and_then(Value::as_str).map(str::to_string),
                value_from_kind,
            })
        })
        .collect()
}

fn normalize_service_ports(service: &Value) -> Result<Vec<BaseServicePort>> {
    let ports = service
        .pointer("/spec/ports")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Service missing spec.ports"))?;
    let normalized = ports
        .iter()
        .filter_map(|port| {
            let target = port.get("targetPort").or_else(|| port.get("port"))?;
            Some(BaseServicePort {
                name: port.get("name").and_then(Value::as_str).map(str::to_string),
                port: u16::try_from(port.get("port")?.as_u64()?).ok()?,
                target_port: match target {
                    Value::String(value) => value.clone(),
                    Value::Number(value) => value.to_string(),
                    _ => return None,
                },
            })
        })
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        bail!("Service has no cloneable ports");
    }
    Ok(normalized)
}

fn probe_path(probe: Option<&Value>) -> Option<String> {
    probe?
        .pointer("/httpGet/path")
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn string_map(value: &Value) -> Result<BTreeMap<String, String>> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow!("expected object map"))?;
    object
        .iter()
        .map(|(key, value)| {
            let value = value
                .as_str()
                .ok_or_else(|| anyhow!("map value for {key} is not a string"))?;
            Ok((key.clone(), value.to_string()))
        })
        .collect()
}

// </HANDWRITE>

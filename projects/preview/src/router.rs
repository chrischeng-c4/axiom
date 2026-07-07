// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-router-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteBinding {
    pub target: String,
    pub host: String,
    pub cookie: String,
    pub header: String,
    pub namespace: String,
    pub service: String,
    pub service_port: u16,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseRoute {
    pub host: String,
    pub namespace: String,
    pub service: String,
    pub service_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteRequest {
    pub host: String,
    pub cookies: BTreeMap<String, String>,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRoute {
    pub target: String,
    pub namespace: String,
    pub service: String,
    pub service_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecision {
    pub outcome: RouteOutcome,
    pub target: Option<String>,
    pub namespace: Option<String>,
    pub service: Option<String>,
    pub service_port: Option<u16>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteOutcome {
    Base,
    Preview,
    NotFound,
}

pub fn resolve_route(
    bindings: &BTreeMap<String, RouteBinding>,
    request: &RouteRequest,
) -> Option<ResolvedRoute> {
    let target = request
        .headers
        .get("X-UAT-Target")
        .or_else(|| request.cookies.get("uat_target"))?;

    let binding = bindings.get(target)?;
    if binding.host != request.host {
        return None;
    }

    Some(ResolvedRoute {
        target: binding.target.clone(),
        namespace: binding.namespace.clone(),
        service: binding.service.clone(),
        service_port: binding.service_port,
    })
}

pub fn resolve_route_with_base(
    bindings: &BTreeMap<String, RouteBinding>,
    base: &BaseRoute,
    request: &RouteRequest,
) -> RouteDecision {
    let Some(selected) = selected_target(request) else {
        if request.host == base.host {
            return RouteDecision {
                outcome: RouteOutcome::Base,
                target: None,
                namespace: Some(base.namespace.clone()),
                service: Some(base.service.clone()),
                service_port: Some(base.service_port),
                reason: "no target header or cookie; using base route".to_string(),
            };
        }
        return not_found(None, "host does not match base route");
    };

    let Some(binding) = bindings.get(selected) else {
        return not_found(Some(selected.to_string()), "unknown route target");
    };
    if binding.host != request.host {
        return not_found(
            Some(binding.target.clone()),
            "route target exists but host does not match",
        );
    }

    RouteDecision {
        outcome: RouteOutcome::Preview,
        target: Some(binding.target.clone()),
        namespace: Some(binding.namespace.clone()),
        service: Some(binding.service.clone()),
        service_port: Some(binding.service_port),
        reason: selected_target_reason(request),
    }
}

pub fn load_route_table_from_rendered_dir(dir: &Path) -> Result<BTreeMap<String, RouteBinding>> {
    let path = dir.join("router/route-binding.yaml");
    let contents = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let object: Value =
        serde_yaml::from_str(&contents).with_context(|| format!("parse {}", path.display()))?;
    let binding = route_binding_from_config_map(&object)?;
    Ok(BTreeMap::from([(binding.target.clone(), binding)]))
}

pub fn load_route_table_from_kubectl(
    namespace: &str,
    context: Option<&str>,
) -> Result<BTreeMap<String, RouteBinding>> {
    let mut command = Command::new("kubectl");
    if let Some(context) = context {
        command.args(["--context", context]);
    }
    let output = command
        .args([
            "get",
            "configmaps",
            "-n",
            namespace,
            "-l",
            "preview.cclab.dev/kind=route-binding",
            "-o",
            "json",
        ])
        .output()
        .with_context(|| format!("start kubectl get route-binding ConfigMaps in {namespace}"))?;
    if !output.status.success() {
        bail!(
            "kubectl get route-binding ConfigMaps in {namespace} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let list: Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("parse route-binding ConfigMap list in {namespace}"))?;
    let mut bindings = BTreeMap::new();
    for item in list
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("kubectl ConfigMap list missing items"))?
    {
        let binding = route_binding_from_config_map(item)?;
        bindings.insert(binding.target.clone(), binding);
    }
    Ok(bindings)
}

fn selected_target(request: &RouteRequest) -> Option<&str> {
    request
        .headers
        .get("X-UAT-Target")
        .or_else(|| request.cookies.get("uat_target"))
        .map(String::as_str)
}

fn selected_target_reason(request: &RouteRequest) -> String {
    if request.headers.contains_key("X-UAT-Target") {
        "matched X-UAT-Target header".to_string()
    } else {
        "matched uat_target cookie".to_string()
    }
}

fn not_found(target: Option<String>, reason: &str) -> RouteDecision {
    RouteDecision {
        outcome: RouteOutcome::NotFound,
        target,
        namespace: None,
        service: None,
        service_port: None,
        reason: reason.to_string(),
    }
}

fn route_binding_from_config_map(config_map: &Value) -> Result<RouteBinding> {
    let data = config_map
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("route-binding ConfigMap missing data"))?;
    let string_field = |name: &str| -> Result<String> {
        data.get(name)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("route-binding ConfigMap missing data.{name}"))
    };
    let service_port = string_field("servicePort")?
        .parse::<u16>()
        .context("parse route-binding data.servicePort")?;
    Ok(RouteBinding {
        target: string_field("target")?,
        host: string_field("host")?,
        cookie: string_field("cookie")?,
        header: string_field("header")?,
        namespace: string_field("namespace")?,
        service: string_field("service")?,
        service_port,
        sha: string_field("sha")?,
    })
}

// </HANDWRITE>

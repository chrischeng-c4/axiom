// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-router-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

// </HANDWRITE>

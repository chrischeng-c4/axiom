// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:133c6ad7" tracker="#1575" reason="Purely render a Pgpool CR through the shared stateless Deployment/common Service modules and attach owner references."
use kube::ResourceExt;
use service_k8s::render::common::owner_ref;
use serde_json::Value;

use crate::k8s::{render_manifests, PgpoolInstanceSpec};

use super::crd::Pgpool;

const API_VERSION: &str = "pgpool.axiom.dev/v1alpha1";
const KIND: &str = "Pgpool";

pub fn render(pgpool: &Pgpool) -> Vec<Value> {
    let primary = pgpool.spec.primary();
    let spec = PgpoolInstanceSpec {
        name: pgpool.name_any(),
        namespace: pgpool.namespace().unwrap_or_else(|| "default".into()),
        image: pgpool.spec.image.clone(),
        replicas: pgpool.spec.replicas,
        backend_host: primary.host.clone(),
        backend_port: primary.port,
        max_backend_connections: primary.per_pod_quota,
        cpu: nonempty(&pgpool.spec.resources.cpu, "250m"),
        memory: nonempty(&pgpool.spec.resources.memory, "256Mi"),
        termination_grace_period_seconds: pgpool.spec.termination_grace_period_seconds,
    };
    let owner = pgpool
        .metadata
        .uid
        .as_deref()
        .map(|uid| owner_ref(API_VERSION, KIND, &pgpool.name_any(), uid));
    let mut manifests = render_manifests(&spec);
    if let Some(owner) = owner {
        for manifest in &mut manifests {
            manifest["metadata"]["ownerReferences"] = serde_json::json!([owner.clone()]);
        }
    }
    manifests
}

fn nonempty(value: &str, default: &str) -> String {
    if value.is_empty() {
        default.into()
    } else {
        value.into()
    }
}
// HANDWRITE-END

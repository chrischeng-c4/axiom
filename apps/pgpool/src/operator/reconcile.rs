// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// <HANDWRITE gap="missing-generator:logic:8e369a2f" tracker="#1575" reason="Implement ManagedService readiness and status projection for Deployment replicas and expose the shared operator run loop.">
use kube::ResourceExt;
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use super::crd::Pgpool;
use super::render;

impl ManagedService for Pgpool {
    const MANAGER: &'static str = "pgpool-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "Deployment",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let desired_replicas = self.spec.replicas as i32;
        let mut status = self.status.clone().unwrap_or_default();
        status.observed_generation = self.metadata.generation.unwrap_or(0);
        status.ready_replicas = ready_replicas;
        status.desired_replicas = desired_replicas;
        status.phase = if status.blocked_scale_reason.is_some() {
            "Blocked".into()
        } else if desired_replicas > 0 && ready_replicas >= desired_replicas {
            "Ready".into()
        } else if ready_replicas > 0 {
            "Reconciling".into()
        } else {
            "Pending".into()
        };
        status.message = format!("{ready_replicas}/{desired_replicas} pgpool pods ready");
        json!({ "status": status })
    }
}

pub async fn run() -> anyhow::Result<()> {
    ::service_k8s::run::<Pgpool>().await
}
// </HANDWRITE>

// HANDWRITE-BEGIN gap="missing-generator:logic:defer-reconcile" tracker="#766" reason="Defer ManagedService implementation for shared operator host."
use kube::ResourceExt;
use serde_json::json;
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};

use super::crd::Defer;

impl ManagedService for Defer {
    const MANAGER: &'static str = "defer-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        super::render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let desired = self.spec.cluster.replicas_per_shard as i32;
        let phase = if desired > 0 && ready_replicas >= desired {
            "Ready"
        } else if ready_replicas > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        json!({"status": {
            "phase": phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "readyReplicas": ready_replicas,
            "desiredReplicas": desired,
            "message": format!("{ready_replicas}/{desired} Defer pods ready")
        }})
    }
}

pub async fn run() -> anyhow::Result<()> {
    service_k8s::run::<Defer>().await
}
// HANDWRITE-END

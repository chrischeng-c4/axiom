//! loom's operator wiring onto the shared `libs/operator` controller.
//!
//! The reconcile loop + leader-election lease live in `libs/operator`
//! (`operator::run` drives the watch + leader-gated apply; `operator::lease` is
//! the elector). loom supplies only its `ManagedService` impl — what to render,
//! which workload to poll for readiness, and the `Loom` status subresource.

use kube::ResourceExt;
use operator::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use crate::operator::crd::Loom;
use crate::operator::render;

impl ManagedService for Loom {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = super::MANAGER;

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        // loom's control plane is always a StatefulSet (single-node or raft HA).
        vec![ReadinessTarget { kind: "StatefulSet", name: self.name_any() }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let desired = (self.spec.shard_count * self.spec.replicas_per_shard) as i32;
        let phase = if ready_replicas >= desired {
            "Ready"
        } else if ready_replicas > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        json!({ "status": {
            "phase": phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "readyReplicas": ready_replicas,
            "desiredReplicas": desired,
            "shardCount": self.spec.shard_count,
            "message": format!("{ready_replicas}/{desired} controller pods ready"),
        }})
    }
}

/// `loom k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; HA-safe at `replicas > 1`).
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Loom>().await
}

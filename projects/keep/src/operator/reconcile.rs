//! keep's operator wiring onto the shared `libs/operator` controller.
//!
//! The reconcile loop + leader-election lease live in `libs/operator`
//! (`operator::run` drives the watch + leader-gated server-side apply over
//! kube-rs). keep supplies only its [`ManagedService`] impl — what to render,
//! which workload to poll for readiness, and the `Keep` status subresource to
//! write.
//!
//! @spec projects/keep/tech-design/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md

use kube::ResourceExt;
use operator::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use super::crd::Keep;
use super::render;

impl ManagedService for Keep {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "keep-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        // keep is always a StatefulSet (durable disk tier); poll it for
        // `.status.readyReplicas`.
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        let desired = (self.spec.cluster.shard_count * self.spec.cluster.replicas_per_shard) as i32;
        let phase = if desired > 0 && ready_replicas >= desired {
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
            "shardCount": self.spec.cluster.shard_count,
            "message": format!("{ready_replicas}/{desired} store pods ready"),
        }})
    }
}

/// `keep k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`).
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Keep>().await
}

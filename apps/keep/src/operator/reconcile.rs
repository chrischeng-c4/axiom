//! keep's operator wiring onto the shared `libs/service-k8s` controller.
//!
//! The reconcile loop + leader-election lease live in `libs/service-k8s`
//! (`service_k8s::run` drives the watch + leader-gated server-side apply over
//! kube-rs). keep supplies only its [`ManagedService`] impl — what to render,
//! which workload to poll for readiness, and the `Keep` status subresource to
//! write.
//!
//! @spec .aw/tech-design/projects/keep/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md

use kube::ResourceExt;
use serde_json::json;
use service_k8s::{ManagedService, ReadinessTarget, ReadyFacts};

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
/// `libs/service-k8s` host (leader-gated; safe at `replicas > 1`).
pub async fn run() -> anyhow::Result<()> {
    service_k8s::run::<Keep>().await
}

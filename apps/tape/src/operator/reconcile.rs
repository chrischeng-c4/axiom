// HANDWRITE-BEGIN gap="missing-generator:logic:e9e1ff60" tracker="pending-tracker" reason="impl ManagedService for Tape: MANAGER tape-operator (SSA field manager + leader-election Lease name); render() -> render::render; readiness_targets = [StatefulSet {name}]; status_patch = Pending|Reconciling|Ready from readyReplicas vs desiredReplicas (replicasPerShard, shard pinned 1) + observedGeneration + message; pub async fn run() = operator::run::<Tape>()."
//! tape's operator wiring onto the shared `libs/operator` controller.
//!
//! The reconcile loop + leader-election lease live in `libs/operator`
//! (`operator::run` drives the watch + leader-gated server-side apply over
//! kube-rs). tape supplies only its [`ManagedService`] impl — what to render,
//! which workload to poll for readiness, and the `Tape` status subresource to
//! write.

use kube::ResourceExt;
use operator::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use super::crd::Tape;
use super::render;

impl ManagedService for Tape {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "tape-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        // tape is always a StatefulSet (durable journal + raft state on a
        // PVC); poll it for `.status.readyReplicas`.
        vec![ReadinessTarget {
            kind: "StatefulSet",
            name: self.name_any(),
        }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let ready_replicas = ready.get(&name) as i32;
        // tape is a single raft group: shardCount is pinned to 1 by the
        // render, so replicasPerShard is the desired replica count.
        let desired = self.spec.cluster.replicas_per_shard as i32;
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
            "message": format!("{ready_replicas}/{desired} tape pods ready"),
        }})
    }
}

/// `tape k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`).
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Tape>().await
}
// HANDWRITE-END

// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! lumen's operator wiring onto the shared `libs/operator` controller.
//!
//! The reconcile loop + leader-election lease now live in `libs/operator`
//! (`operator::run` drives the watch + leader-gated apply over h2c-free kube;
//! `operator::lease` is the elector). lumen supplies only its `ManagedService`
//! impl — what to render, which workloads to poll for readiness, and the
//! `Lumen` status subresource to write.

use kube::ResourceExt;
use operator::{ManagedService, ReadinessTarget, ReadyFacts};
use serde_json::json;

use crate::operator::crd::Lumen;
use crate::operator::render;

/// lumen's contribution to the shared operator.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
impl ManagedService for Lumen {
    /// Server-side-apply field manager + leader-election Lease name.
    const MANAGER: &'static str = "lumen-operator";

    fn render(&self) -> Vec<serde_json::Value> {
        render::render(self)
    }

    fn readiness_targets(&self) -> Vec<ReadinessTarget> {
        let name = self.name_any();
        let kind = if self.spec.replicas_per_shard > 1 {
            "StatefulSet"
        } else {
            "Deployment"
        };
        vec![ReadinessTarget { kind, name }]
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let serving_ready = ready.ready.get(&name).copied().unwrap_or(0) as i32;
        let desired = if self.spec.replicas_per_shard > 1 {
            (self.spec.shard_count * self.spec.replicas_per_shard) as i32
        } else {
            self.spec.serving.autoscaling.min_replicas
        };
        let phase = if serving_ready >= desired {
            "Ready"
        } else if serving_ready > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        json!({ "status": {
            "phase": phase,
            "observedGeneration": self.metadata.generation.unwrap_or(0),
            "servingReadyReplicas": serving_ready,
            "desiredReplicas": desired,
            "shardCount": self.spec.shard_count,
            "message": format!("{serving_ready}/{desired} serving pods ready"),
        }})
    }
}

/// `lumen k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Lumen>().await
}
// CODEGEN-END

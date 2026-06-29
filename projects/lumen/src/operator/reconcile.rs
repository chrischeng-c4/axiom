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
        let mut targets = vec![ReadinessTarget {
            kind: "Deployment",
            name: name.clone(),
        }];
        // The managed broker is a StatefulSet `<name>-relay`; an external broker
        // has no workload to poll (its serving pods' connect-retry covers blips).
        if self.spec.broker.is_managed() {
            targets.push(ReadinessTarget {
                kind: "StatefulSet",
                name: format!("{name}-relay"),
            });
        }
        targets
    }

    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value {
        let name = self.name_any();
        let serving_ready = ready.ready.get(&name).copied().unwrap_or(0) as i32;
        let broker_ready = if self.spec.broker.is_managed() {
            ready
                .ready
                .get(&format!("{name}-relay"))
                .copied()
                .unwrap_or(0)
                >= 1
        } else {
            true // external broker: assumed up.
        };
        let desired = self.spec.serving.autoscaling.min_replicas;
        let phase = if serving_ready >= desired && broker_ready {
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
            "brokerReady": broker_ready,
            "message": format!("{serving_ready}/{desired} serving pods ready; brokerReady={broker_ready}"),
        }})
    }
}

/// `lumen k8s operator` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Lumen>().await
}
// CODEGEN-END

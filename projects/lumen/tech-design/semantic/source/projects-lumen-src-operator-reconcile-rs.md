---
id: projects-lumen-src-operator-reconcile-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/reconcile.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/reconcile.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Error` | projects/lumen/src/operator/reconcile.rs | enum | pub | 36 |  |
| `run` | projects/lumen/src/operator/reconcile.rs | function | pub | 67 | run() -> anyhow::Result<()> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

/// `lumen k8s operator run` — run the reconcile controller on the shared
/// `libs/operator` host (leader-gated; safe at `replicas > 1`).
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-reconcile-rs.md#source
pub async fn run() -> anyhow::Result<()> {
    operator::run::<Lumen>().await
}
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/reconcile.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/operator/reconcile.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```

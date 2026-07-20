// SPEC-MANAGED: libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-llm-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! LLM topic provider for the shared Kubernetes operator scaffold.

/// Agent-facing topic describing the shared operator primitive.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "operator",
    summary:
        "Shared Kubernetes controller, lease, render, and maintenance toolkit for service CRDs.",
    body: r#"# Kubernetes operator shared topic

## Ownership boundary
The `service-k8s` crate owns the reusable Kubernetes controller scaffold:
cluster-wide watch, leader-election `Lease`, server-side apply, status patching,
and render helpers for common service objects. Each service supplies a
`ManagedService`: its CRD type, service-specific render policy, readiness facts,
status shape, and public CLI commands.

## Render layers
Keep service CLIs layered:

```text
<cli> k8s crd render
<cli> k8s operator render
<cli> k8s operator run
<cli> k8s instance render
```

The shared crate provides building blocks for `ClusterSpec`, `ResourceSpec`,
StatefulSet-style serving fleets, headless/client Services, PodDisruptionBudget,
CronJob backup runners, and PVC resize helpers. It does not decide service CRD
field names or lifecycle policy on its own.

## Stateful capacity axes
Storage and compute scale independently. `plan_shard_split` plans one new
physical shard when the busiest shard is strictly above 1 GiB by default.
`plan_replica_layer` uses CPU and memory utilization to plan a complete
replica-per-shard layer, never a partial StatefulSet pod total.

Both functions are decisions, not actuators. A service must finish its
domain-safe routing-map cutover and data movement before applying a shard-count
change, and must finish the Raft membership transition before applying a
replica-layer change.

## Reconcile contract
The controller applies the currently rendered children and reports readiness.
It is not a garbage collector for every prior child shape a service ever
rendered; service migrations that change kind or durable storage boundaries
must document an explicit handoff.
"#,
};

/// Return the shared operator topic for CLI composition.
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-llm-rs.md#source
pub fn topic() -> &'static cli_std::llm::Topic {
    &TOPIC
}

#[cfg(test)]
mod tests {
    #[test]
    fn llm_topic_is_nonempty() {
        let topic = super::topic();
        assert_eq!(topic.id, "operator");
        assert!(topic.body.contains("ManagedService"));
        assert!(topic.body.contains("server-side apply"));
        assert!(topic.body.contains("strictly above 1 GiB"));
        assert!(topic.body.contains("CPU and memory"));
        assert!(topic.body.contains("Raft membership transition"));
    }
}
// CODEGEN-END

//! LLM topic provider for the shared Kubernetes operator scaffold.

/// Agent-facing topic describing the shared operator primitive.
pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic {
    id: "operator",
    summary:
        "Shared Kubernetes controller, lease, render, and maintenance toolkit for service CRDs.",
    body: r#"# operator shared topic

## Ownership boundary
The `operator` crate owns the reusable Kubernetes controller scaffold:
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

## Reconcile contract
The controller applies the currently rendered children and reports readiness.
It is not a garbage collector for every prior child shape a service ever
rendered; service migrations that change kind or durable storage boundaries
must document an explicit handoff.
"#,
};

/// Return the shared operator topic for CLI composition.
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
    }
}

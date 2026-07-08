---
id: libs-operator-src-llm-rs
summary: Lossless rust-source-unit coverage for `libs/operator/src/llm.rs`.
capability_refs:
  - id: shared-kubernetes-operator-scaffold
    role: primary
    claim: shared-kubernetes-operator-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Operator library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/operator/src/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/operator/src/llm.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TOPIC` | libs/operator/src/llm.rs | const | pub | 4 | pub const TOPIC: cli_std::llm::Topic = cli_std::llm::Topic { |
| `topic` | libs/operator/src/llm.rs | function | pub | 41 | pub fn topic() -> &'static cli_std::llm::Topic { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/operator/src/llm.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/operator/src/llm.rs` captured during libs codegen standardization.
```

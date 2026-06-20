---
id: semantic-lumen-operator
summary: Semantic coverage for "projects/lumen/src/operator"
capability_refs:
  - id: "k8s-deployment"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/src/operator`."
  - id: "k8s-deployment"
    role: primary
    gap: "lumen-crd-reconcile-loop-kube-rs-operator"
    claim: "lumen-crd-reconcile-loop-kube-rs-operator"
    coverage: full
    rationale: "The operator source group owns the Lumen CRD reconcile/render implementation."
fill_sections: [schema, unit-test, changes]
---

# Semantic TD: lumen/operator

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "lumen/operator"
  source_group: "projects/lumen/src/operator"
  coverage_kind: semantic
  evidence:
    source_units:
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: unit-test
coverage_kind: semantic
strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
evidence:
  source_tests: []
---
requirementDiagram

element UT_SOURCE_TESTS {
  type: "TestEvidence"
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."
```

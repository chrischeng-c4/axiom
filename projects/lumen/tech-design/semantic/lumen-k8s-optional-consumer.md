---
id: semantic-lumen-k8s-optional-consumer
summary: Semantic coverage for "projects/lumen/k8s/optional/consumer"
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/optional/consumer

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/optional/consumer"
    role: "unknown"
  semantic_domain:
    key: "lumen/k8s/optional/consumer"
    source_group: "projects/lumen/k8s/optional/consumer"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/optional/consumer/kustomization.yaml"
        language: "kustomize"
        ownership_state: "handwrite"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/optional/consumer"
  artifacts:
    - path: "projects/lumen/k8s/optional/consumer/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        # Optional sample bundle. NOT included from base/ or any overlay; opt in
        # explicitly by listing this directory in your own overlay.
        namespace: lumen
        
        labels:
          - pairs:
              app.kubernetes.io/name: lumen
              app.kubernetes.io/component: consumer
            includeSelectors: false
        
        resources:
          - deployment.yaml
          - hpa.yaml
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/optional/consumer/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-lumen-k8s-optional-consumer-kustomization-yaml>"
```

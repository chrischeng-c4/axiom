---
id: semantic-lumen-k8s-operator
summary: Semantic coverage for "projects/lumen/k8s/operator"
capability_refs:
  - id: "k8s-deployment"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/k8s/operator`."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/operator

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/operator"
    role: "unknown"
  semantic_domain:
    key: "lumen/k8s/operator"
    source_group: "projects/lumen/k8s/operator"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/operator/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/operator"
  artifacts:
    - path: "projects/lumen/k8s/operator/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        # Installs the lumen Operator: the Lumen CRD, its RBAC, and the controller
        # Deployment (in the lumen-system namespace). After this is applied, create
        # `Lumen` objects (see ../../examples/lumen-cr.yaml) instead of hand-applying
        # k8s/base + overlays.
        #
        #   kubectl apply -k k8s/operator
        #   kubectl apply -f examples/lumen-cr.yaml
        #
        # crd.yaml is generated — regenerate with:
        #   cargo run -p lumen --features operator --bin lumen-operator -- gen-crd > k8s/operator/crd.yaml
        
        resources:
          - crd.yaml
          - rbac.yaml
          - deployment.yaml
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/operator/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

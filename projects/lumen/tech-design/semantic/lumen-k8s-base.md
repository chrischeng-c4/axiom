---
id: semantic-lumen-k8s-base
summary: Semantic coverage for "projects/lumen/k8s/base"
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/k8s/base`."
  - id: "kubernetes-native-deployment"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "The k8s base is the canonical Kubernetes-native deployment foundation."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/base

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/base"
    role: "base"
  semantic_domain:
    key: "lumen/k8s/base"
    source_group: "projects/lumen/k8s/base"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/base/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/base"
  artifacts:
    - path: "projects/lumen/k8s/base/kustomization.yaml"
      kind: "kustomization"
      content: |
        # SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-k8s-base.md#deployment
        # CODEGEN-BEGIN
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        # Base is a Lumen-only direct install. includeSelectors:false keeps these common
        # labels out of immutable workload/Service selectors (which are pinned to
        # app+role), so re-applies never hit a selector-immutability error.
        labels:
          - pairs:
              app.kubernetes.io/name: lumen
              app.kubernetes.io/part-of: lumen
            includeSelectors: false
        
        resources:
          - namespace.yaml
          - serviceaccount.yaml
          - configmap.yaml
          # Single-node direct serving. HA goes through the Lumen operator
          # (`replicasPerShard > 1`) so the data plane gets raft StatefulSet identity.
          - deployment.yaml
          - service.yaml
          - hpa.yaml
          - pdb.yaml
          # Observability (ServiceMonitor + PrometheusRule) is NOT in base: it needs
          # the prometheus-operator CRDs (monitoring.coreos.com/v1). The staging/prod
          # overlays pull it in via components/observability; dev (kind/laptop, no
          # operator) omits it so `kubectl apply -k overlays/dev` works on a vanilla
          # cluster.
        # CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/base/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

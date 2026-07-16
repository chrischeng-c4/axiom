---
id: semantic-lumen-k8s-overlays-dev
summary: Semantic coverage for "apps/lumen/k8s/overlays/dev"
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/lumen/k8s/overlays/dev`."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/overlays/dev

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/overlays/dev"
    role: "overlay"
  semantic_domain:
    key: "lumen/k8s/overlays/dev"
    source_group: "apps/lumen/k8s/overlays/dev"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/k8s/overlays/dev/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "apps/lumen/k8s/overlays/dev"
  artifacts:
    - path: "apps/lumen/k8s/overlays/dev/kustomization.yaml"
      kind: "kustomization"
      content: |
        # SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-k8s-overlays-dev.md#deployment
        # CODEGEN-BEGIN
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        resources:
          - ../../base
        
        # dev: a single serving node, embedded WAL, smallest viable footprint, auth off,
        # human-readable logs. The ConfigMap is a static base resource (stable name, no
        # hash suffix), so overlays patch its data in place rather than using a
        # configMapGenerator merge.
        
        replicas:
          - name: lumen
            count: 1
        
        patches:
          # Dev ConfigMap values: 1 shard, pretty logs, auth off.
          - target:
              kind: ConfigMap
              name: lumen-config
            patch: |-
              - op: replace
                path: /data/SHARD_COUNT
                value: "1"
              - op: replace
                path: /data/LUMEN_LOG_FORMAT
                value: "pretty"
              - op: add
                path: /data/LUMEN_AUTH
                value: "off"
          # Direct-install runtime wiring. Resource requests inherit the shared
          # 1 CPU / 4Gi request-only baseline; users tune this overlay for their node.
          - target:
              kind: Deployment
              name: lumen
            patch: |-
              # AUTH is off in dev; wire LUMEN_AUTH through from the merged ConfigMap.
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_AUTH
                  valueFrom:
                    configMapKeyRef:
                      name: lumen-config
                      key: LUMEN_AUTH
        # CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/lumen/k8s/overlays/dev/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

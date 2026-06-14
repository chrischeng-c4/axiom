---
id: semantic-lumen-k8s-overlays-dev
summary: Semantic coverage for "projects/lumen/k8s/overlays/dev"
capability_refs:
  - id: "k8s-deployment"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/k8s/overlays/dev`."
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
    source_group: "projects/lumen/k8s/overlays/dev"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/overlays/dev/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/overlays/dev"
  artifacts:
    - path: "projects/lumen/k8s/overlays/dev/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        resources:
          - ../../base
        
        # dev: a single serving node, a single NATS broker, smallest viable
        # footprint, auth off, human-readable logs. The ConfigMap is a static
        # base resource (stable name, no hash suffix), so overlays patch its data
        # in place rather than using a configMapGenerator merge.
        
        replicas:
          - name: lumen
            count: 1
          - name: lumen-nats
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
          # Tiny serving-node resources for a laptop / kind cluster.
          - target:
              kind: Deployment
              name: lumen
            patch: |-
              # apply_raft_entry is a synchronous CPU-bound op (bulk index + BM25);
              # too tight a CPU limit lets it starve the tokio runtime / NATS client
              # I/O under load. 1 core keeps a 10k-doc index e2e responsive.
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/cpu
                value: "500m"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/memory
                value: "512Mi"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/cpu
                value: "1"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/memory
                value: "512Mi"
              # Single-node dev clusters rarely have zone labels; relax host spread.
              - op: replace
                path: /spec/template/spec/topologySpreadConstraints/1/whenUnsatisfiable
                value: "ScheduleAnyway"
              # AUTH is off in dev; wire LUMEN_AUTH through from the merged ConfigMap.
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_AUTH
                  valueFrom:
                    configMapKeyRef:
                      name: lumen-config
                      key: LUMEN_AUTH
          # Floor the HPA at 1 so it never forces a second pod in dev.
          - target:
              kind: HorizontalPodAutoscaler
              name: lumen
            patch: |-
              - op: replace
                path: /spec/minReplicas
                value: 1
              - op: replace
                path: /spec/maxReplicas
                value: 2
          # Tiny NATS footprint + small JetStream volume.
          - target:
              kind: StatefulSet
              name: lumen-nats
            patch: |-
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/cpu
                value: "100m"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/memory
                value: "256Mi"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/cpu
                value: "500m"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/memory
                value: "256Mi"
              - op: replace
                path: /spec/volumeClaimTemplates/0/spec/resources/requests/storage
                value: "2Gi"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/overlays/dev/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

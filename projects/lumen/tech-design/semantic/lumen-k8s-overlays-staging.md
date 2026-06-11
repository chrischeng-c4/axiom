---
id: semantic-lumen-k8s-overlays-staging
summary: Semantic coverage for "projects/lumen/k8s/overlays/staging"
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/overlays/staging

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/overlays/staging"
    role: "overlay"
  semantic_domain:
    key: "lumen/k8s/overlays/staging"
    source_group: "projects/lumen/k8s/overlays/staging"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/overlays/staging/kustomization.yaml"
        language: "kustomize"
        ownership_state: "handwrite"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/overlays/staging"
  artifacts:
    - path: "projects/lumen/k8s/overlays/staging/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        resources:
          - ../../base
        
        # staging runs the prometheus-operator, so pull in the ServiceMonitor +
        # PrometheusRule SLO alerts.
        components:
          - ../../components/observability
        
        # staging: 3 serving nodes, a single NATS broker, modest resources,
        # json logs, auth off (a pre-prod soak environment).
        
        replicas:
          - name: lumen
            count: 3
          - name: lumen-nats
            count: 1
        
        patches:
          # Staging ConfigMap values: 3 shards, json logs, auth off.
          - target:
              kind: ConfigMap
              name: lumen-config
            patch: |-
              - op: replace
                path: /data/SHARD_COUNT
                value: "3"
              - op: replace
                path: /data/LUMEN_LOG_FORMAT
                value: "json"
              - op: add
                path: /data/LUMEN_AUTH
                value: "off"
          - target:
              kind: Deployment
              name: lumen
            patch: |-
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/cpu
                value: "1"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/memory
                value: "2Gi"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/cpu
                value: "1"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/memory
                value: "2Gi"
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_AUTH
                  valueFrom:
                    configMapKeyRef:
                      name: lumen-config
                      key: LUMEN_AUTH
          # Match the HPA floor to the Deployment replica count.
          - target:
              kind: HorizontalPodAutoscaler
              name: lumen
            patch: |-
              - op: replace
                path: /spec/minReplicas
                value: 3
              - op: replace
                path: /spec/maxReplicas
                value: 8
          - target:
              kind: StatefulSet
              name: lumen-nats
            patch: |-
              # Cloud SSD storage class (base omits storageClassName for portability).
              - op: add
                path: /spec/volumeClaimTemplates/0/spec/storageClassName
                value: "ssd"
              - op: replace
                path: /spec/volumeClaimTemplates/0/spec/resources/requests/storage
                value: "10Gi"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/overlays/staging/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-lumen-k8s-overlays-staging-kustomization-yaml>"
```

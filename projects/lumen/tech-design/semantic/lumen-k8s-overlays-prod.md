---
id: semantic-lumen-k8s-overlays-prod
summary: Semantic coverage for "projects/lumen/k8s/overlays/prod"
capability_refs:
  - id: "k8s-deployment"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/k8s/overlays/prod`."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/overlays/prod

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/overlays/prod"
    role: "overlay"
  semantic_domain:
    key: "lumen/k8s/overlays/prod"
    source_group: "projects/lumen/k8s/overlays/prod"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/overlays/prod/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/overlays/prod"
  artifacts:
    - path: "projects/lumen/k8s/overlays/prod/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        resources:
          - ../../base
        
        # prod runs the prometheus-operator, so pull in the ServiceMonitor +
        # PrometheusRule SLO alerts.
        components:
          - ../../components/observability
        
        # prod: 6+ serving nodes (HPA floor 6, ceiling 12), a 3-node clustered
        # JetStream broker for HA, strict auth, json logs, full resources.
        
        replicas:
          - name: lumen
            count: 6
          - name: lumen-nats
            count: 3
        
        patches:
          # Prod ConfigMap values: 6 shards, json logs, strict auth.
          - target:
              kind: ConfigMap
              name: lumen-config
            patch: |-
              - op: replace
                path: /data/SHARD_COUNT
                value: "6"
              - op: replace
                path: /data/LUMEN_LOG_FORMAT
                value: "json"
              - op: add
                path: /data/LUMEN_AUTH
                value: "required"
          - target:
              kind: Deployment
              name: lumen
            patch: |-
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/cpu
                value: "4"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/memory
                value: "16Gi"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/cpu
                value: "4"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/memory
                value: "16Gi"
              # Tighten host spread in prod: refuse to co-locate serving pods.
              - op: replace
                path: /spec/template/spec/topologySpreadConstraints/1/whenUnsatisfiable
                value: "DoNotSchedule"
              # Strict auth: LUMEN_TOKENS must be supplied out-of-band (e.g. a Secret
              # patched in by the platform layer). Wire LUMEN_AUTH from the ConfigMap.
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_AUTH
                  valueFrom:
                    configMapKeyRef:
                      name: lumen-config
                      key: LUMEN_AUTH
          # HPA floor 6 (matches the Deployment replica count), ceiling 12.
          - target:
              kind: HorizontalPodAutoscaler
              name: lumen
            patch: |-
              - op: replace
                path: /spec/minReplicas
                value: 6
              - op: replace
                path: /spec/maxReplicas
                value: 12
          # Clustered JetStream: enable routes so the 3 brokers form one RAFT
          # meta-group. Routes resolve via the headless service per-pod DNS.
          - target:
              kind: StatefulSet
              name: lumen-nats
            patch: |-
              - op: replace
                path: /spec/template/spec/containers/0/args
                value:
                  - "-c"
                  - "/etc/nats/nats.conf"
                  - "-js"
                  - "-sd"
                  - "/data"
                  - "-m"
                  - "8222"
                  - "--cluster_name"
                  - "lumen"
                  - "--cluster"
                  - "nats://0.0.0.0:6222"
                  - "--routes"
                  - "nats://lumen-nats-0.lumen-nats-headless:6222,nats://lumen-nats-1.lumen-nats-headless:6222,nats://lumen-nats-2.lumen-nats-headless:6222"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/cpu
                value: "2"
              - op: replace
                path: /spec/template/spec/containers/0/resources/requests/memory
                value: "4Gi"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/cpu
                value: "2"
              - op: replace
                path: /spec/template/spec/containers/0/resources/limits/memory
                value: "4Gi"
              # Cloud SSD storage class (base omits storageClassName for portability).
              - op: add
                path: /spec/volumeClaimTemplates/0/spec/storageClassName
                value: "ssd"
              - op: replace
                path: /spec/volumeClaimTemplates/0/spec/resources/requests/storage
                value: "100Gi"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/overlays/prod/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```

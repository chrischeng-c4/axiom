---
id: semantic-lumen-k8s-overlays-prod
summary: Semantic coverage for "projects/lumen/k8s/overlays/prod"
capability_refs:
  - id: "long-running-stability"
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
        # SPEC-MANAGED: projects/lumen/tech-design/semantic/lumen-k8s-overlays-prod.md#deployment
        # CODEGEN-BEGIN
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        namespace: lumen
        
        resources:
          - ../../base
        
        # prod runs the prometheus-operator, so pull in the ServiceMonitor +
        # PrometheusRule SLO alerts.
        components:
          - ../../components/observability
        
        # prod: direct single-node install with strict auth, json logs, full resources.
        # Production HA uses the operator CR path (`replicasPerShard > 1`) so Lumen owns
        # raft replication without an external broker.
        
        replicas:
          - name: lumen
            count: 1
        
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
              # Strict auth: a Secret named `lumen-tokens` with key
              # `token-registry.json` must be supplied out-of-band, commonly from GCP
              # Secret Manager via External Secrets Operator / Secret Store CSI. Lumen
              # reads only the mounted file path at startup, so rotate by rolling pods
              # or running a Secret reloader controller.
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_AUTH
                  valueFrom:
                    configMapKeyRef:
                      name: lumen-config
                      key: LUMEN_AUTH
              - op: add
                path: /spec/template/spec/containers/0/env/-
                value:
                  name: LUMEN_TOKEN_REGISTRY_FILE
                  value: /var/run/secrets/lumen/token-registry.json
              - op: add
                path: /spec/template/spec/containers/0/volumeMounts/-
                value:
                  name: lumen-token-registry
                  mountPath: /var/run/secrets/lumen
                  readOnly: true
              - op: add
                path: /spec/template/spec/volumes/-
                value:
                  name: lumen-token-registry
                  secret:
                    secretName: lumen-tokens
                    items:
                      - key: token-registry.json
                        path: token-registry.json
          # Direct kustomize stays single-node. Use the operator for raft HA/scaling.
          - target:
              kind: HorizontalPodAutoscaler
              name: lumen
            patch: |-
              - op: replace
                path: /spec/minReplicas
                value: 1
              - op: replace
                path: /spec/maxReplicas
                value: 1
        # CODEGEN-END

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
    impl_mode: codegen
```

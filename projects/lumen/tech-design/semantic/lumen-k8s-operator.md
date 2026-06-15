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
        #   cargo run -p lumen --features operator --bin lumen -- k8s gen-crd > k8s/operator/crd.yaml
        
        resources:
          - crd.yaml
          - rbac.yaml
          - deployment.yaml
    - path: "projects/lumen/k8s/operator/deployment.yaml"
      kind: "kubernetes-deployment"
      content: |
        # The operator: a controller that watches Lumen objects cluster-wide. Ships in
        # the same `lumen` image (run as `lumen k8s operator`, built with
        # --features operator). HA-safe: a coordination.k8s.io Lease elects one active
        # reconciler, so this can be scaled to replicas > 1 (the others stand by).
        apiVersion: apps/v1
        kind: Deployment
        metadata:
          name: lumen-operator
          namespace: lumen-system
          labels:
            app.kubernetes.io/name: lumen-operator
            app.kubernetes.io/part-of: lumen
        spec:
          # Scale to 2+ for HA — leader-election guarantees a single active reconciler.
          replicas: 1
          strategy:
            type: RollingUpdate
          selector:
            matchLabels:
              app.kubernetes.io/name: lumen-operator
          template:
            metadata:
              labels:
                app.kubernetes.io/name: lumen-operator
            spec:
              serviceAccountName: lumen-operator
              terminationGracePeriodSeconds: 15
              securityContext:
                runAsNonRoot: true
                runAsUser: 1000
                runAsGroup: 1000
                seccompProfile:
                  type: RuntimeDefault
              containers:
                - name: operator
                  image: lumen:latest
                  imagePullPolicy: IfNotPresent
                  command: ["/usr/local/bin/lumen", "k8s", "operator"]
                  env:
                    - name: RUST_LOG
                      value: "info"
                    # Leader-election identity + the namespace the Lease lives in.
                    - name: POD_NAME
                      valueFrom:
                        fieldRef:
                          fieldPath: metadata.name
                    - name: POD_NAMESPACE
                      valueFrom:
                        fieldRef:
                          fieldPath: metadata.namespace
                  resources:
                    requests:
                      cpu: 100m
                      memory: 128Mi
                    limits:
                      cpu: 500m
                      memory: 256Mi
                  securityContext:
                    runAsNonRoot: true
                    runAsUser: 1000
                    runAsGroup: 1000
                    allowPrivilegeEscalation: false
                    readOnlyRootFilesystem: true
                    capabilities:
                      drop: ["ALL"]
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
    impl_mode: codegen
  - path: "projects/lumen/k8s/operator/deployment.yaml"
    action: modify
    section: deployment
    description: |
      Operator Deployment manifest is a full-file operations artifact replayed from TD.
    impl_mode: codegen
```

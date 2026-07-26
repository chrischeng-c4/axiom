---
id: semantic-lumen-k8s-operator
summary: Semantic coverage for "apps/lumen/k8s/operator"
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/lumen/k8s/operator`."
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
    source_group: "apps/lumen/k8s/operator"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/k8s/operator/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "apps/lumen/k8s/operator"
  artifacts:
    - path: "apps/lumen/k8s/operator/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        # Installs the lumen Operator: the Lumen CRD, its RBAC, and the controller
        # Deployment + PodDisruptionBudget (in the lumen-system namespace). After this
        # is applied, create `Lumen` objects (see ../../examples/lumen-cr.yaml) instead
        # of hand-applying k8s/base + overlays.
        #
        #   kubectl apply -k k8s/operator
        #   kubectl apply -f examples/lumen-cr.yaml
        #
        # crd.yaml is generated — regenerate with:
        #   cargo run -p lumen --features operator --bin lumen -- k8s crd render > k8s/operator/crd.yaml
        
        resources:
          - crd.yaml
          - rbac.yaml
          - deployment.yaml
          - pdb.yaml
    - path: "apps/lumen/k8s/operator/deployment.yaml"
      kind: "kubernetes-deployment"
      content: |
        # The operator: a controller that watches Lumen objects cluster-wide. Ships in
        # the same `lumen` image (run as `lumen k8s operator run`, built with
        # --features operator). HA: a coordination.k8s.io Lease elects one active
        # reconciler, so the standby replica watches without applying.
        apiVersion: apps/v1
        kind: Deployment
        metadata:
          name: lumen-operator
          namespace: lumen-system
          labels:
            app.kubernetes.io/name: lumen-operator
            app.kubernetes.io/part-of: lumen
        spec:
          # HA floor (#2602): leader election guarantees a single active reconciler, so
          # the second replica is a warm standby that takes over on drain, eviction, or
          # rollout instead of leaving every Lumen CR in the cluster unreconciled.
          replicas: 2
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
              affinity:
                podAntiAffinity:
                  # Preferred, not required: co-located replicas still survive a drain
                  # (the PDB forces it one pod at a time), and a single-node cluster —
                  # kind, minikube — must be able to run both rather than park one
                  # replica Pending forever.
                  preferredDuringSchedulingIgnoredDuringExecution:
                    - weight: 100
                      podAffinityTerm:
                        labelSelector:
                          matchLabels:
                            app.kubernetes.io/name: lumen-operator
                        topologyKey: kubernetes.io/hostname
              securityContext:
                runAsNonRoot: true
                runAsUser: 1000
                runAsGroup: 1000
                seccompProfile:
                  type: RuntimeDefault
              containers:
                - name: operator
                  # Published GHCR release for this workspace's Lumen version. Bumped by
                  # the release procedure alongside Cargo.toml/openapi.json; grep target
                  # for the bump commit: `ghcr.io/chrischeng-c4/lumen:`. To pin an
                  # immutable digest or point at a mirrored registry, render this
                  # manifest with `lumen k8s operator render --image <ref>`.
                  image: ghcr.io/chrischeng-c4/lumen:0.4.25
                  imagePullPolicy: IfNotPresent
                  command: ["/usr/local/bin/lumen", "k8s", "operator", "run"]
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
    - path: "apps/lumen/k8s/operator/pdb.yaml"
      kind: "kubernetes-poddisruptionbudget"
      content: |
        # Keeps one reconciler alive through voluntary disruption (#2602). The operator
        # runs two replicas behind leader election, so a node drain must take them one
        # at a time — evicting both at once leaves every Lumen CR in the cluster
        # unreconciled until a replacement pod becomes ready and acquires the Lease.
        apiVersion: policy/v1
        kind: PodDisruptionBudget
        metadata:
          name: lumen-operator
          namespace: lumen-system
          labels:
            app.kubernetes.io/name: lumen-operator
            app.kubernetes.io/part-of: lumen
        spec:
          maxUnavailable: 1
          selector:
            matchLabels:
              app.kubernetes.io/name: lumen-operator
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/lumen/k8s/operator/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
  - path: "apps/lumen/k8s/operator/deployment.yaml"
    action: modify
    section: deployment
    description: |
      Operator Deployment manifest is a full-file operations artifact replayed from TD.
    impl_mode: codegen
  - path: "apps/lumen/k8s/operator/pdb.yaml"
    action: create
    section: deployment
    description: |
      Operator PodDisruptionBudget: serializes voluntary eviction across the two
      leader-elected control-plane replicas (#2602), so a node drain never leaves
      the cluster without a reconciler.
    impl_mode: codegen
```

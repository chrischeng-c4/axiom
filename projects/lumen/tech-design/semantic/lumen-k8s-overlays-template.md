---
id: semantic-lumen-k8s-overlays-template
summary: Semantic coverage for "projects/lumen/k8s/overlays/template"
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `projects/lumen/k8s/overlays/template`."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/overlays/template

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/overlays/template"
    role: "overlay"
  semantic_domain:
    key: "lumen/k8s/overlays/template"
    source_group: "projects/lumen/k8s/overlays/template"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/lumen/k8s/overlays/template/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "projects/lumen/k8s/overlays/template"
  artifacts:
    - path: "projects/lumen/k8s/overlays/template/kustomization.yaml"
      kind: "kustomization"
      content: |
        apiVersion: kustomize.config.k8s.io/v1beta1
        kind: Kustomization
        
        # ──────────────────────────────────────────────────────────────────────────
        # COPY-TO-CUSTOMIZE starting point. lumen ships only the Dockerfile + this
        # kustomize tree; you build + push the image to your OWN registry and apply.
        #
        #   cp -r k8s/overlays/template k8s/overlays/<your-env>
        #   # edit this file: fill each blank value below (image name, tag, shards)
        #   # then verify no blanks remain (self-check command is in the README
        #   # "How to deploy" section) and apply:
        #   kubectl apply -k k8s/overlays/<your-env>      # kustomize is built into kubectl
        #
        # dev/staging/prod alongside this are worked REFERENCES (real patch examples);
        # this is the blank you fill in.
        # ──────────────────────────────────────────────────────────────────────────
        
        # Namespace everything lands in. To deploy into an existing/shared cluster,
        # change this to your namespace (kustomize renames the Namespace object too).
        namespace: lumen
        
        resources:
          - ../../base
          # AUTH (optional, off by default). To require bearer auth: copy
          # secret.example.yaml -> secret.yaml, fill real tokens, uncomment this line
          # AND the OPTIONAL auth patch block at the bottom.
          # - secret.yaml
        
        # REQUIRED #1 — point the image at YOUR registry. base ships `lumen:latest`;
        # GKE/most clusters cannot pull an unprefixed name. You build + push it
        # (lumen publishes no image). `name: lumen` matches the base image's name.
        images:
          - name: lumen
            newName: REPLACE_ME__REGISTRY/lumen   # e.g. asia-east1-docker.pkg.dev/PROJECT/REPO/lumen
            newTag: REPLACE_ME__IMAGE_TAG         # e.g. v1   (avoid :latest in prod)
        
        # Replica floors at apply time. The HPA owns lumen's live count from here up.
        replicas:
          - name: lumen
            count: 2
          - name: lumen-nats
            count: 1
        
        patches:
          # REQUIRED #2 — SHARD_COUNT is the install-time crc32 client fan-out. Decide
          # it ONCE: it is NOT online-changeable; changing it after data exists
          # re-routes every client and needs a rebuild. json logs for prod capture.
          - target: { kind: ConfigMap, name: lumen-config }
            patch: |-
              - op: replace
                path: /data/SHARD_COUNT
                value: "REPLACE_ME__SHARD_COUNT"   # e.g. "3"  (fixed for the cluster's life)
              - op: replace
                path: /data/LUMEN_LOG_FORMAT
                value: "json"
        
          # GKE storage class for the NATS PVC (the one stateful component). base omits
          # it (cluster default). GKE SSD = premium-rwo; balanced PD = standard-rwo.
          # Non-GKE: set your class, or delete this patch to use the cluster default.
          - target: { kind: StatefulSet, name: lumen-nats }
            patch: |-
              - op: add
                path: /spec/volumeClaimTemplates/0/spec/storageClassName
                value: "premium-rwo"
        
          # ── OPTIONAL: bearer auth ────────────────────────────────────────────────
          # Uncomment this whole block AND the `- secret.yaml` resource line above to
          # require a token on every route. Leave commented for an open (auth=off) start.
          # - target: { kind: ConfigMap, name: lumen-config }
          #   patch: |-
          #     - op: add
          #       path: /data/LUMEN_AUTH
          #       value: "required"
          # - target: { kind: Deployment, name: lumen }
          #   patch: |-
          #     - op: add
          #       path: /spec/template/spec/containers/0/env/-
          #       value:
          #         name: LUMEN_AUTH
          #         valueFrom: { configMapKeyRef: { name: lumen-config, key: LUMEN_AUTH } }
          #     - op: add
          #       path: /spec/template/spec/containers/0/env/-
          #       value:
          #         name: LUMEN_TOKENS
          #         valueFrom: { secretKeyRef: { name: lumen-tokens, key: LUMEN_TOKENS } }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/lumen/k8s/overlays/template/kustomization.yaml"
    action: modify
    section: deployment
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: codegen
```

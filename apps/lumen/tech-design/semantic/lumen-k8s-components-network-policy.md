---
id: semantic-lumen-k8s-components-network-policy
summary: Semantic coverage for "apps/lumen/k8s/components/network-policy"
capability_refs:
  - id: "long-running-stability"
    role: primary
    gap: "kustomize-base-overlays-hpa"
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/lumen/k8s/components/network-policy`."
fill_sections: [deployment, changes]
---

# Semantic TD: lumen/k8s/components/network-policy

## Deployment
<!-- type: deployment lang: yaml -->

```yaml
deployment:
  format: kustomize
  layout:
    group: "lumen/k8s/components/network-policy"
    role: "component"
  semantic_domain:
    key: "lumen/k8s/components/network-policy"
    source_group: "apps/lumen/k8s/components/network-policy"
    coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/lumen/k8s/components/network-policy/kustomization.yaml"
        language: "kustomize"
        ownership_state: "codegen"
        generator_primitives: ["kustomize_manifest"]
        source_evidence_node:
          layer: "operations"
          ecosystem: "kustomize"
          role: "kustomization"
          section_type: "deployment"
          domain: "apps/lumen/k8s/components/network-policy"
  artifacts:
    - path: "apps/lumen/k8s/components/network-policy/kustomization.yaml"
      kind: "kustomization"
      content: |
        # SPEC-MANAGED: apps/lumen/tech-design/semantic/lumen-k8s-components-network-policy.md#deployment
        # CODEGEN-BEGIN
        apiVersion: kustomize.config.k8s.io/v1alpha1
        kind: Component
        
        # Per-instance network isolation for the direct kustomize install. Kept in a
        # component rather than in base because a NetworkPolicy is only meaningful
        # where the CNI enforces it: GKE needs Dataplane V2 or the Calico add-on, and a
        # default kind cluster (kindnet) accepts the object and enforces nothing. The
        # dev overlay therefore omits it; staging and prod pull it in.
        
        resources:
          - networkpolicy.yaml
        # CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/lumen/k8s/components/network-policy/kustomization.yaml"
    action: create
    section: deployment
    description: |
      Declare the opt-in NetworkPolicy component so the staging and prod overlays
      can compose it without changing the dev overlay's vanilla-cluster posture.
    impl_mode: codegen
  - path: "apps/lumen/k8s/components/network-policy/networkpolicy.yaml"
    action: create
    section: deployment
    description: |
      Default-deny ingress/egress for the direct-install serving pod: the search
      API (7373) stays reachable cluster-wide, egress narrows to DNS over both
      transports plus outbound TLS. The direct install is single-node embedded
      mode, so no Raft peer port is admitted here — the operator CR path renders
      its own policy from `spec.networkPolicy` instead.
    impl_mode: codegen
```

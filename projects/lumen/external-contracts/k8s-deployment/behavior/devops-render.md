---
id: lumen-devops-render-ec
summary: Devops-pillar — the operator renders the correct managed child object set (including NATS).
fill_sections: [e2e-test]
---

# EC: Devops Operator Render

The devops pillar (the shipped deployment defaults) needs an EC. The operator's
pure render(Lumen) -> child objects is the cheapest, offline contract — it proves
the manifests we ship are correct (incl. the bundled NATS JetStream), so it gates
production. Live deploy bring-up (kind + NATS) stays infra-gated under stability.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-devops-operator-render
    capability_id: k8s-deployment
    claim_id: operator-renders-managed-child-set-incl-nats
    contract_id: devops-operator-render-golden
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    required_for_production: true
    assertions:
      - "render(Lumen) emits the managed serving Deployment/Service/HPA/PDB plus the NATS JetStream ConfigMap/StatefulSet/Service when NATS is managed."
      - "BYO-NATS (nats.externalUrl) omits the managed NATS objects and wires the external URL into the serving env."
```

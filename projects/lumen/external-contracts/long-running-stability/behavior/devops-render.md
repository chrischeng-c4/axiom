---
id: lumen-long-running-stability-operator-render-ec
summary: Long-running stability — the operator renders the correct managed child object set (including NATS).
fill_sections: [e2e-test]
---

# EC: Long-Running Stability Operator Render

The shipped deployment defaults need an EC. The operator's pure render(Lumen) ->
child objects is the cheapest offline contract: it proves the manifests we ship
are correct, including the bundled NATS JetStream. Live deploy bring-up (kind +
NATS) stays infra-gated under stability.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: lumen-long-running-stability-operator-render
    capability_id: long-running-stability
    claim_id: lumen-crd-reconcile-loop-kube-rs-operator
    contract_id: devops-operator-render-golden
    category: behavior
    command: "cargo test -p lumen --features operator --test operator_render -- --nocapture"
    assertions:
      - "render(Lumen) emits the managed serving Deployment/Service/HPA/PDB plus the NATS JetStream ConfigMap/StatefulSet/Service when NATS is managed."
      - "BYO-NATS (nats.externalUrl) omits the managed NATS objects and wires the external URL into the serving env."
```

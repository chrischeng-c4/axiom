<!-- HANDWRITE-BEGIN gap="missing-generator:logic:89e678c6" tracker="pending-tracker" reason="Tape shared StatefulSet deployment render contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)." -->
---
id: tape-long-running-stability-operator-render-ec
summary: Long-running stability contract for Tape's shared StatefulSet operator render.
fill_sections: [e2e-test]
---

# EC: Long-Running Stability Operator Render

The operator's pure Tape render is the offline deployment contract. It adopts
the Lumen EC shape while proving Tape's own StatefulSet and replay-journal
deployment defaults.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-long-running-stability-operator-render
    capability_id: kubernetes-native-deployment
    claim_id: tape-crd-reconcile-loop-kube-rs-operator
    contract_id: tape-devops-operator-render-golden
    category: behavior
    command: "cargo test -p tape --features operator --test operator -- --nocapture"
    assertions:
      - "render(Tape) emits the managed StatefulSet, client and headless Services, PDB, and Tape CRD-owned labels/selectors."
      - "The rendered workload uses the shared StatefulSet topology projection and carries Tape's journal storage, raft topology, and standard probe configuration."
      - "Optional token-registry and bootstrap-seed inputs alter only their explicit Tape wiring and do not create a second service topology."
```
<!-- HANDWRITE-END -->

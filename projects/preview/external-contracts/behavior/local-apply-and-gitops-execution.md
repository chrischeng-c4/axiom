---
id: local-apply-and-gitops-execution
summary: External contract for the GKE local apply and GitOps execution work root.
fill_sections: [e2e-test]
---

# EC: Local apply and GitOps execution

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: local-apply-and-gitops-execution
    capability_id: gke-uat-preview-environment-rendering
    claim_id: local-apply-and-gitops-execution
    contract_id: local-apply-and-gitops-execution
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract"
    assertions:
      - "`preview render` emits plans/manifest-inventory.json with deterministic Kubernetes object order."
      - "`preview apply --dir <rendered-dir> --plan-only` prints an ordered summary for MR comments and CI logs."
      - "`preview gitops render --dir <rendered-dir> --out <bundle-dir>` writes deterministic relative-path bundle artifacts."
      - "`preview apply` is covered against a local cluster by the kind lifecycle gate."
```

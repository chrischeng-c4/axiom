---
id: local-apply-gitops-execution-ec
summary: External contract for local apply and GitOps execution paths.
fill_sections: [e2e-test]
---

# EC: Local apply and GitOps execution EC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: local-apply-gitops-execution-ec
    capability_id: preview-external-contracts
    claim_id: local-apply-gitops-execution-ec
    contract_id: local-apply-gitops-execution-ec
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract local_apply_plan_and_gitops_bundle_are_deterministic"
    assertions:
      - "`preview render` emits plans/manifest-inventory.json with deterministic Kubernetes object order."
      - "`preview apply --dir <rendered-dir> --plan-only` prints an MR-comment-friendly ordered apply summary without contacting a cluster."
      - "`preview gitops render --dir <rendered-dir> --out <bundle-dir>` writes a deterministic relative-path GitOps bundle with no local absolute paths."
      - "The kind lifecycle gate covers `preview apply --dry-run`, direct apply, idempotent re-apply, and rollout against a local cluster."
```

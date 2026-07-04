---
id: mr-scoped-namespace-projection
summary: External contract for MR-scoped namespace projection.
fill_sections: [e2e-test]
---

# EC: MR-scoped namespace projection

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: mr-scoped-namespace-projection
    capability_id: gke-uat-preview-environment-rendering
    claim_id: mr-scoped-namespace-projection
    contract_id: mr-scoped-namespace-projection
    category: behavior
    command: "cargo test -p preview render_creates_gke_contract_files"
    assertions:
      - "`preview render` emits spec, workload clone plan, namespace, service account, quota, limits, RBAC, deployment, service, route-binding, MR comment, and cleanup-plan files."
      - "The rendered namespace is named `uat-mr-<id>`, carries preview labels, and records the base namespace/source workload."
```

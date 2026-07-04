---
id: cookie-header-route-binding-contract
summary: External contract for Cookie/header route binding contract.
fill_sections: [e2e-test]
---

# EC: Cookie/header route binding contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cookie-header-route-binding-contract
    capability_id: gke-uat-preview-environment-rendering
    claim_id: cookie-header-route-binding-contract
    contract_id: cookie-header-route-binding-contract
    category: behavior
    command: "cargo test -p preview route_binding_uses_target_not_namespace_cookie"
    assertions:
      - "Route binding keeps the public target `mr-<id>` separate from namespace `uat-mr-<id>`."
      - "Browser cookie selection uses `uat_target`, not a raw namespace value."
```

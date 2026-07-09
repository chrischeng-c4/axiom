---
id: local-router-adapter
summary: External contract for local router adapter target resolution.
fill_sections: [e2e-test]
---

# EC: Local router adapter

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: local-router-adapter
    capability_id: preview-external-contracts
    claim_id: local-router-adapter
    contract_id: local-router-adapter
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract local_router_resolve_proves_base_preview_and_fail_closed"
    assertions:
      - "`preview router resolve --dir <rendered-dir>` routes requests without target header/cookie to the base route."
      - "`preview router resolve` routes valid X-UAT-Target values to the preview namespace and service."
      - "`preview router resolve` lets X-UAT-Target override uat_target cookie."
      - "Invalid targets return a not-found decision and never silently fallback to base."
```

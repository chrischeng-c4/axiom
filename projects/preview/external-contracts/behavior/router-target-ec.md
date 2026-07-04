---
id: router-target-ec
summary: External contract for Router target EC.
fill_sections: [e2e-test]
---

# EC: Router target EC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: router-target-ec
    capability_id: preview-external-contracts
    claim_id: router-target-ec
    contract_id: router-target-ec
    category: behavior
    command: "cargo test -p preview --test router_contract"
    assertions:
      - "Cookie target resolution maps only through a known RouteBinding."
      - "Header target selection can override cookie selection for API/mobile/manual clients."
      - "Unknown targets do not guess or synthesize namespaces."
```

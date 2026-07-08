---
id: shared-service-durability-contract
summary: External contract for Shared Service Durability Contract.
fill_sections: [e2e-test]
---

# EC: Shared Service Durability Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-service-durability-contract
    capability_id: shared-service-durability-contract
    claim_id: shared-service-durability-contract
    contract_id: shared-service-durability-contract
    category: behavior
    command: "cargo test -p service-durability"
    assertions:
      - "Shared Service Durability Contract public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

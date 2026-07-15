---
id: shared-storage-durable-contract
summary: External contract for Shared Service Durability Contract.
fill_sections: [e2e-test]
---

# EC: Shared Service Durability Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-storage-durable-contract
    capability_id: shared-storage-durable-contract
    claim_id: shared-storage-durable-contract
    contract_id: shared-storage-durable-contract
    category: behavior
    command: "cargo test -p storage-durable"
    assertions:
      - "Shared Service Durability Contract public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

---
id: shared-service-backup-contract
summary: External contract for Shared Service Backup Contract.
fill_sections: [e2e-test]
---

# EC: Shared Service Backup Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-service-backup-contract
    capability_id: shared-service-backup-contract
    claim_id: shared-service-backup-contract
    contract_id: shared-service-backup-contract
    category: behavior
    command: "cargo test -p service-backup"
    assertions:
      - "Shared Service Backup Contract public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

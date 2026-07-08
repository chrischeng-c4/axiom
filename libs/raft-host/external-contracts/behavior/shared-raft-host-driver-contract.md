---
id: shared-raft-host-driver-contract
summary: External contract for Shared Raft Host Driver.
fill_sections: [e2e-test]
---

# EC: Shared Raft Host Driver

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-raft-host-driver-contract
    capability_id: shared-raft-host-driver
    claim_id: shared-raft-host-driver-contract
    contract_id: shared-raft-host-driver-contract
    category: behavior
    command: "cargo test -p raft-host"
    assertions:
      - "Shared Raft Host Driver public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

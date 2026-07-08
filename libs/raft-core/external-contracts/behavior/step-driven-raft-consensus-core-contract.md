---
id: step-driven-raft-consensus-core-contract
summary: External contract for Step-Driven Raft Consensus Core.
fill_sections: [e2e-test]
---

# EC: Step-Driven Raft Consensus Core

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: step-driven-raft-consensus-core-contract
    capability_id: step-driven-raft-consensus-core
    claim_id: step-driven-raft-consensus-core-contract
    contract_id: step-driven-raft-consensus-core-contract
    category: behavior
    command: "cargo test -p raft-core"
    assertions:
      - "Step-Driven Raft Consensus Core public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

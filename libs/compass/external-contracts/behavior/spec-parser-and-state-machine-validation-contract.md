---
id: spec-parser-and-state-machine-validation-contract
summary: External contract for Spec parser and state-machine validation contract.
fill_sections: [e2e-test]
---

# EC: Spec parser and state-machine validation contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: spec-parser-and-state-machine-validation-contract
    capability_id: spec-parsing-and-code-generation
    claim_id: spec-parser-and-state-machine-validation-contract
    contract_id: spec-parser-and-state-machine-validation-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Spec parser and state-machine validation contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

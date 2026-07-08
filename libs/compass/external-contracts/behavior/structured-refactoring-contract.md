---
id: structured-refactoring-contract
summary: External contract for Structured refactoring contract.
fill_sections: [e2e-test]
---

# EC: Structured refactoring contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: structured-refactoring-contract
    capability_id: semantic-navigation-search-and-refactoring
    claim_id: structured-refactoring-contract
    contract_id: structured-refactoring-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Structured refactoring contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

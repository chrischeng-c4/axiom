---
id: symbol-outline-and-propagated-type-query-contract
summary: External contract for Symbol outline and propagated type query contract.
fill_sections: [e2e-test]
---

# EC: Symbol outline and propagated type query contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: symbol-outline-and-propagated-type-query-contract
    capability_id: semantic-navigation-search-and-refactoring
    claim_id: symbol-outline-and-propagated-type-query-contract
    contract_id: symbol-outline-and-propagated-type-query-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Symbol outline and propagated type query contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

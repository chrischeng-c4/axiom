---
id: semantic-search-and-graph-query-contract
summary: External contract for Semantic search and graph query contract.
fill_sections: [e2e-test]
---

# EC: Semantic search and graph query contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: semantic-search-and-graph-query-contract
    capability_id: semantic-navigation-search-and-refactoring
    claim_id: semantic-search-and-graph-query-contract
    contract_id: semantic-search-and-graph-query-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Semantic search and graph query contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

---
id: multi-language-parser-and-checker-dispatch-contract
summary: External contract for Multi-language parser and checker dispatch contract.
fill_sections: [e2e-test]
---

# EC: Multi-language parser and checker dispatch contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-language-parser-and-checker-dispatch-contract
    capability_id: codebase-check-and-lint-pipeline
    claim_id: multi-language-parser-and-checker-dispatch-contract
    contract_id: multi-language-parser-and-checker-dispatch-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Multi-language parser and checker dispatch contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

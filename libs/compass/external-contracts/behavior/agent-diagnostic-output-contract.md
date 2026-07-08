---
id: agent-diagnostic-output-contract
summary: External contract for Agent diagnostic output contract.
fill_sections: [e2e-test]
---

# EC: Agent diagnostic output contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: agent-diagnostic-output-contract
    capability_id: codebase-check-and-lint-pipeline
    claim_id: agent-diagnostic-output-contract
    contract_id: agent-diagnostic-output-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Agent diagnostic output contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

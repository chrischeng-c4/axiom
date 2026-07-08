---
id: standard-agent-cli-commands-contract
summary: External contract for Standard Agent CLI Commands.
fill_sections: [e2e-test]
---

# EC: Standard Agent CLI Commands

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: standard-agent-cli-commands-contract
    capability_id: standard-agent-cli-commands
    claim_id: standard-agent-cli-commands-contract
    contract_id: standard-agent-cli-commands-contract
    category: behavior
    command: "cargo test -p cli-std"
    assertions:
      - "Standard Agent CLI Commands public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

---
id: dev-server-cli-contract
summary: External contract for Dev Server Cli Contract.
fill_sections: [e2e-test]
---

# EC: Dev Server Cli Contract

---
id: dev-server-cli-contract
summary: External contract for Dev Server Cli Contract.
fill_sections: [e2e-test]
---

# EC: Dev Server Cli Contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dev-server-cli-contract
    capability_id: dev-server-hmr
    claim_id: dev-server-cli-contract
    contract_id: dev-server-cli-contract
    category: behavior
    command: "cargo test -p jet --lib cli::e2e_command_contract_tests -- --nocapture"
    assertions:
      - "Jet dev-server command accepts the supported CLI contract and rejects invalid invocations."
```

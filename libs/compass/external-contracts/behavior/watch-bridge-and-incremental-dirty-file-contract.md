---
id: watch-bridge-and-incremental-dirty-file-contract
summary: External contract for Watch bridge and incremental dirty-file contract.
fill_sections: [e2e-test]
---

# EC: Watch bridge and incremental dirty-file contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: watch-bridge-and-incremental-dirty-file-contract
    capability_id: daemon-watch-and-incremental-analysis
    claim_id: watch-bridge-and-incremental-dirty-file-contract
    contract_id: watch-bridge-and-incremental-dirty-file-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Watch bridge and incremental dirty-file contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

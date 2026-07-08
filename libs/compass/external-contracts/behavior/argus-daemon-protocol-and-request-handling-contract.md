---
id: argus-daemon-protocol-and-request-handling-contract
summary: External contract for Argus daemon protocol and request handling contract.
fill_sections: [e2e-test]
---

# EC: Argus daemon protocol and request handling contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: argus-daemon-protocol-and-request-handling-contract
    capability_id: daemon-watch-and-incremental-analysis
    claim_id: argus-daemon-protocol-and-request-handling-contract
    contract_id: argus-daemon-protocol-and-request-handling-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Argus daemon protocol and request handling contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

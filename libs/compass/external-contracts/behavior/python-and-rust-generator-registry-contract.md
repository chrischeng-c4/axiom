---
id: python-and-rust-generator-registry-contract
summary: External contract for Python and Rust generator registry contract.
fill_sections: [e2e-test]
---

# EC: Python and Rust generator registry contract

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: python-and-rust-generator-registry-contract
    capability_id: spec-parsing-and-code-generation
    claim_id: python-and-rust-generator-registry-contract
    contract_id: python-and-rust-generator-registry-contract
    category: behavior
    command: "cargo test -p cclab-compass"
    assertions:
      - "Python and Rust generator registry contract remains covered by the configured Compass library test suite."
      - "The library contract stays usable through its documented README capability surface."
```

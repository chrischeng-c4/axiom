---
id: renderer-neutral-component-runtime-contract
summary: External contract for Renderer-Neutral Component Runtime.
fill_sections: [e2e-test]
---

# EC: Renderer-Neutral Component Runtime

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: renderer-neutral-component-runtime-contract
    capability_id: renderer-neutral-component-runtime
    claim_id: renderer-neutral-component-runtime-contract
    contract_id: renderer-neutral-component-runtime-contract
    category: behavior
    command: "cargo test -p cclab-ui-runtime"
    assertions:
      - "Renderer-Neutral Component Runtime public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
